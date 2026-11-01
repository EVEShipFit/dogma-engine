//! Builds a `Fit` from fuzzer bytes, out of types, effects and attributes the
//! SDE has.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use arbitrary::{Result, Unstructured};
use esf_data::eve::EffectCategory;
use esf_data::{Info, InfoEsf};
use esf_dogma_engine::{
    Character, Charge, DamageProfile, Environment, Fit, FitItem, Mutation, ProjectedBuff,
    ProjectedEffect, Projection, ReactiveArmor, Security, Ship, Slot, Spool, State,
};

use crate::{SDE, info};

const CATEGORY_SHIP: i32 = 6;
const CATEGORY_CHARGE: i32 = 8;
const CATEGORY_SKILL: i32 = 16;
const CATEGORY_STRUCTURE: i32 = 65;

const MAX_BUFF_ID: i32 = 10_000;

/// A mutaplasmid: the item it applies to, what it makes, and the range of
/// each attribute it rolls.
struct Mutaplasmid {
    base: i32,
    result: i32,
    rolls: Vec<(i32, f64, f64)>,
}

struct Pools {
    ships: Vec<i32>,
    charges: Vec<i32>,
    skills: Vec<i32>,
    items: Vec<i32>,
    /// Per effect, the types that have it.
    by_effect: Vec<Vec<i32>>,
    /// Effects of category target, with a type that has it.
    projectable: Vec<(i32, i32)>,
    buffs: Vec<i32>,
    mutaplasmids: Vec<Mutaplasmid>,
}

static POOLS: LazyLock<Pools> = LazyLock::new(|| {
    let info = info();
    let of = |categories: &[i32]| -> Vec<i32> {
        SDE.types()
            .filter(|r#type| categories.contains(&r#type.category_id()))
            .map(|r#type| r#type.id())
            .collect()
    };

    let items: Vec<i32> = SDE
        .types()
        .filter(|r#type| info.get_dogma_attributes(r#type.id()).is_some())
        .map(|r#type| r#type.id())
        .collect();

    let mut by_effect: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
    for &type_id in &items {
        for effect_id in effects_of(type_id) {
            by_effect.entry(effect_id).or_default().push(type_id);
        }
    }

    let projectable = by_effect
        .iter()
        .filter(|(effect_id, _)| {
            info.get_dogma_effect(**effect_id)
                .is_some_and(|effect| effect.effect_category() == EffectCategory::Target)
        })
        .flat_map(|(effect_id, types)| types.iter().map(|type_id| (*type_id, *effect_id)))
        .collect();

    let mutaplasmids = SDE
        .mutaplasmids()
        .flat_map(|mutaplasmid| {
            let rolls: Vec<(i32, f64, f64)> = mutaplasmid
                .attributes()
                .into_iter()
                .flatten()
                .map(|roll| {
                    (
                        roll.attribute_id(),
                        f64::from(roll.min()),
                        f64::from(roll.max()),
                    )
                })
                .collect();
            mutaplasmid
                .mappings()
                .into_iter()
                .flatten()
                .flat_map(|mapping| {
                    let result = mapping.resulting_type_id();
                    mapping
                        .applicable_type_ids()
                        .into_iter()
                        .flatten()
                        .map(move |base| (base, result))
                })
                .map(|(base, result)| Mutaplasmid {
                    base,
                    result,
                    rolls: rolls.clone(),
                })
                .collect::<Vec<_>>()
        })
        .collect();

    Pools {
        ships: of(&[CATEGORY_SHIP, CATEGORY_STRUCTURE]),
        charges: of(&[CATEGORY_CHARGE]),
        skills: of(&[CATEGORY_SKILL]),
        items,
        by_effect: by_effect.into_values().collect(),
        projectable,
        buffs: (0..=MAX_BUFF_ID)
            .filter(|id| info.get_dbuff_collection(*id).is_some())
            .collect(),
        mutaplasmids,
    }
});

fn pick(u: &mut Unstructured, pool: &[i32]) -> Result<i32> {
    Ok(*u.choose(pool)?)
}

/// Any type, or a type of a randomly picked effect.
fn pick_item(u: &mut Unstructured) -> Result<i32> {
    let pools = &*POOLS;
    match u.arbitrary()? {
        true => pick(u, &pools.items),
        false => {
            let types = u.choose(&pools.by_effect)?;
            pick(u, types)
        }
    }
}

fn attribute_value(type_id: i32, attribute_id: i32) -> f64 {
    info()
        .get_dogma_attributes(type_id)
        .and_then(|attributes| {
            attributes
                .iter()
                .find(|attribute| attribute.attribute_id() == attribute_id)
        })
        .map_or(0.0, |attribute| f64::from(attribute.value()))
}

/// Either a real mutaplasmid roll, or any base and attributes.
fn mutated(u: &mut Unstructured, type_id: i32) -> Result<(i32, Mutation)> {
    let pools = &*POOLS;
    if u.arbitrary()? {
        return Ok((
            type_id,
            Mutation {
                base: pick(u, &pools.items)?,
                attributes: attributes_of(u, type_id)?,
            },
        ));
    }

    let mutaplasmid = u.choose(&pools.mutaplasmids)?;
    let mut attributes = BTreeMap::new();
    for &(attribute_id, min, max) in &mutaplasmid.rolls {
        let base = attribute_value(mutaplasmid.base, attribute_id);
        attributes.insert(attribute_id, base * float(u, min, max)?);
    }
    Ok((
        mutaplasmid.result,
        Mutation {
            base: mutaplasmid.base,
            attributes,
        },
    ))
}

fn float(u: &mut Unstructured, min: f64, max: f64) -> Result<f64> {
    let step = f64::from(u.arbitrary::<u16>()?) / f64::from(u16::MAX);
    Ok(min + (max - min) * step)
}

fn effects_of(type_id: i32) -> Vec<i32> {
    info()
        .get_dogma_effects(type_id)
        .map(|effects| effects.iter().map(|effect| effect.effect_id()).collect())
        .unwrap_or_default()
}

/// Some attributes of a type, each at a value near its own.
fn attributes_of(u: &mut Unstructured, type_id: i32) -> Result<BTreeMap<i32, f64>> {
    let attributes: Vec<(i32, f64)> = info()
        .get_dogma_attributes(type_id)
        .map(|attributes| {
            attributes
                .iter()
                .map(|attribute| (attribute.attribute_id(), f64::from(attribute.value())))
                .collect()
        })
        .unwrap_or_default();

    let mut out = BTreeMap::new();
    if attributes.is_empty() {
        return Ok(out);
    }
    for _ in 0..u.int_in_range(0..=6)? {
        let (attribute_id, value) = *u.choose(&attributes)?;
        out.insert(attribute_id, value * float(u, 0.0, 2.0)?);
    }
    Ok(out)
}

fn some_of(u: &mut Unstructured, pool: &[i32]) -> Result<BTreeSet<i32>> {
    let mut out = BTreeSet::new();
    if pool.is_empty() {
        return Ok(out);
    }
    for _ in 0..u.int_in_range(0..=3)? {
        out.insert(*u.choose(pool)?);
    }
    Ok(out)
}

fn slot(u: &mut Unstructured) -> Result<Slot> {
    Ok(match u.int_in_range(0..=11)? {
        0 => Slot::High(u.int_in_range(0..=8)?),
        1 => Slot::Medium(u.int_in_range(0..=8)?),
        2 => Slot::Low(u.int_in_range(0..=8)?),
        3 => Slot::Rig(u.int_in_range(0..=3)?),
        4 => Slot::Subsystem(u.int_in_range(0..=4)?),
        5 => Slot::Service(u.int_in_range(0..=5)?),
        6 => Slot::FighterTube(u.int_in_range(0..=5)?),
        7 => Slot::FighterBay,
        8 => Slot::Implant(u.int_in_range(1..=10)?),
        9 => Slot::Booster(u.int_in_range(1..=4)?),
        10 => Slot::DroneBay,
        _ => Slot::Cargo,
    })
}

fn state(u: &mut Unstructured) -> Result<State> {
    Ok(*u.choose(&[
        State::Offline,
        State::Online,
        State::Active,
        State::Overload,
    ])?)
}

fn damage_profile(u: &mut Unstructured) -> Result<DamageProfile> {
    Ok(DamageProfile {
        em: float(u, 0.0, 100.0)?,
        explosive: float(u, 0.0, 100.0)?,
        kinetic: float(u, 0.0, 100.0)?,
        thermal: float(u, 0.0, 100.0)?,
    })
}

fn item(u: &mut Unstructured) -> Result<FitItem> {
    let pools = &*POOLS;
    let mut type_id = pick_item(u)?;
    let mutation = match u.ratio(1, 8)? {
        true => {
            let (result, mutation) = mutated(u, type_id)?;
            type_id = result;
            Some(mutation)
        }
        false => None,
    };
    let effects = effects_of(type_id);

    Ok(FitItem {
        type_id,
        slot: slot(u)?,
        quantity: u.int_in_range(1..=20)?,
        state: state(u)?,
        charge: match u.ratio(1, 3)? {
            true => Some(Charge {
                type_id: pick(u, &pools.charges)?,
            }),
            false => None,
        },
        mutation,
        fighter_abilities: match u.ratio(1, 4)? {
            true => Some(some_of(u, &effects)?),
            false => None,
        },
        booster_side_effects: some_of(u, &effects)?,
        spool: match u.ratio(1, 4)? {
            true => Some(Spool::MultiplierBonus(float(u, 0.0, 5.0)?)),
            false => None,
        },
    })
}

fn incoming(u: &mut Unstructured) -> Result<Projection> {
    let pools = &*POOLS;
    let mut projection = Projection::default();

    for _ in 0..u.int_in_range(0..=3)? {
        projection.buffs.push(ProjectedBuff {
            id: match u.ratio(1, 8)? {
                true => u.int_in_range(-1..=MAX_BUFF_ID)?,
                false => pick(u, &pools.buffs)?,
            },
            value: float(u, -100.0, 100.0)?,
        });
    }
    for _ in 0..u.int_in_range(0..=3)? {
        let (type_id, effect_id) = match u.ratio(1, 8)? {
            true => {
                let type_id = pick_item(u)?;
                let Ok(&effect_id) = u.choose(&effects_of(type_id)) else {
                    continue;
                };
                (type_id, effect_id)
            }
            false => *u.choose(&pools.projectable)?,
        };
        projection.effects.push(ProjectedEffect {
            type_id,
            effect_id,
            attributes: attributes_of(u, type_id)?,
        });
    }
    Ok(projection)
}

/// A fit of real types, in any slot and state, with anything the `Fit` can
/// carry.
pub fn fit(u: &mut Unstructured) -> Result<Fit> {
    let pools = &*POOLS;
    let ship = pick(u, &pools.ships)?;
    let modes: Vec<i32> = info().mode_type_ids(ship).collect();

    let mut items = Vec::new();
    for _ in 0..u.int_in_range(0..=20)? {
        items.push(item(u)?);
    }

    let skills = match u.int_in_range(0..=2)? {
        0 => BTreeMap::new(),
        1 => pools.skills.iter().map(|&skill| (skill, 5)).collect(),
        _ => {
            let mut skills = BTreeMap::new();
            for _ in 0..u.int_in_range(0..=50)? {
                skills.insert(pick(u, &pools.skills)?, u.int_in_range(0..=5)?);
            }
            skills
        }
    };

    Ok(Fit {
        name: None,
        ship: Ship {
            type_id: ship,
            mode: match modes.is_empty() || u.ratio(1, 4)? {
                true => None,
                false => Some(*u.choose(&modes)?),
            },
        },
        items,
        character: Character {
            skills,
            security_status: float(u, -10.0, 5.0)?,
        },
        environment: Environment {
            damage_profile: match u.arbitrary()? {
                true => damage_profile(u)?,
                false => DamageProfile::default(),
            },
            security: *u.choose(&[
                Security::HighSec,
                Security::LowSec,
                Security::NullSec,
                Security::Wormhole,
            ])?,
            reactive_armor: match u.int_in_range(0..=2)? {
                0 => ReactiveArmor::DoNotAdapt,
                1 => ReactiveArmor::DamageProfile,
                _ => ReactiveArmor::Profile(damage_profile(u)?),
            },
        },
        incoming: incoming(u)?,
    })
}
