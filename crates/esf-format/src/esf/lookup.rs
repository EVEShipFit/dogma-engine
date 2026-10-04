//! What esf/1 reads from the SDE.

use esf_data::{InfoEsf, eve};

use super::model::Error;

const CATEGORY_SHIP: i32 = 6;
const CATEGORY_MODULE: i32 = 7;
const CATEGORY_CHARGE: i32 = 8;
const CATEGORY_DRONE: i32 = 18;
const CATEGORY_IMPLANT: i32 = 20;
const CATEGORY_SUBSYSTEM: i32 = 32;
const CATEGORY_STRUCTURE: i32 = 65;
const CATEGORY_STRUCTURE_MODULE: i32 = 66;
const CATEGORY_FIGHTER: i32 = 87;

const GROUP_BOOSTER: i32 = 303;
const GROUP_SHIP_MODIFIERS: i32 = 1306;
const GROUP_CONTAINERS: [i32; 4] = [12, 340, 448, 649];

const EFFECT_LO_POWER: i32 = 11;
const EFFECT_HI_POWER: i32 = 12;
const EFFECT_MED_POWER: i32 = 13;
const EFFECT_ONLINE: i32 = 16;
const EFFECT_RIG_SLOT: i32 = 2663;
const EFFECT_SERVICE_SLOT: i32 = 6306;

const ATTRIBUTE_SQUADRON_SIZE: i32 = 2215;

/// What a type is, for placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Hull,
    Modifier,
    Sub,
    High,
    Mid,
    Low,
    Rig,
    Svc,
    Drone,
    Fighter,
    Charge,
    Implant,
    Booster,
    Other,
}

pub(super) struct Lookup<'a, I> {
    pub info: &'a I,
}

/// An SDE number as the decimal it was written as.
pub(super) fn sde_decimal(value: f32) -> f64 {
    value.to_string().parse().unwrap_or(value.into())
}

impl<'a, I: InfoEsf> Lookup<'a, I> {
    pub fn r#type(&self, type_id: i32) -> Option<eve::Type<'a>> {
        self.info.get_type(type_id)
    }

    pub fn name(&self, type_id: i32) -> &'a str {
        self.r#type(type_id).map_or("", |r#type| r#type.name())
    }

    pub fn type_by_name(&self, name: &str, line: usize) -> Result<i32, Error> {
        self.info
            .type_name_to_id(name)
            .ok_or_else(|| Error::at(line, format!("unknown type {name:?}")))
    }

    pub fn attribute_name(&self, attribute_id: i32) -> Option<&'a str> {
        self.info
            .get_dogma_attribute(attribute_id)
            .map(|attribute| attribute.name())
    }

    pub fn category(&self, type_id: i32) -> i32 {
        self.r#type(type_id)
            .map_or(0, |r#type| r#type.category_id())
    }

    pub fn classify(&self, type_id: i32) -> Kind {
        let Some(r#type) = self.r#type(type_id) else {
            return Kind::Other;
        };
        if r#type.group_id() == GROUP_SHIP_MODIFIERS {
            return Kind::Modifier;
        }
        match r#type.category_id() {
            CATEGORY_SHIP | CATEGORY_STRUCTURE => Kind::Hull,
            _ if self.is_container(type_id) => Kind::Hull,
            CATEGORY_SUBSYSTEM => Kind::Sub,
            CATEGORY_MODULE | CATEGORY_STRUCTURE_MODULE => self
                .info
                .get_dogma_effects(type_id)
                .into_iter()
                .flatten()
                .find_map(|effect| match effect.effect_id() {
                    EFFECT_HI_POWER => Some(Kind::High),
                    EFFECT_MED_POWER => Some(Kind::Mid),
                    EFFECT_LO_POWER => Some(Kind::Low),
                    EFFECT_RIG_SLOT => Some(Kind::Rig),
                    EFFECT_SERVICE_SLOT => Some(Kind::Svc),
                    _ => None,
                })
                .unwrap_or(Kind::Other),
            CATEGORY_IMPLANT if r#type.group_id() == GROUP_BOOSTER => Kind::Booster,
            CATEGORY_IMPLANT => Kind::Implant,
            CATEGORY_DRONE => Kind::Drone,
            CATEGORY_FIGHTER => Kind::Fighter,
            CATEGORY_CHARGE => Kind::Charge,
            _ => Kind::Other,
        }
    }

    pub fn is_charge(&self, type_id: i32) -> bool {
        self.category(type_id) == CATEGORY_CHARGE
    }

    pub fn is_container(&self, type_id: i32) -> bool {
        self.r#type(type_id)
            .is_some_and(|r#type| GROUP_CONTAINERS.contains(&r#type.group_id()))
    }

    /// Whether a type has an effect in the active or target category.
    pub fn is_active(&self, type_id: i32) -> bool {
        self.info
            .get_dogma_effects(type_id)
            .into_iter()
            .flatten()
            .filter(|effect| effect.effect_id() != EFFECT_ONLINE)
            .filter_map(|effect| self.info.get_dogma_effect(effect.effect_id()))
            .any(|effect| {
                matches!(
                    effect.effect_category(),
                    eve::EffectCategory::Active | eve::EffectCategory::Target
                )
            })
    }

    /// The hull a tactical mode belongs to.
    fn mode_hull(&self, mode: i32) -> Option<i32> {
        let words: Vec<&str> = self.name(mode).split(' ').collect();
        (1..words.len()).rev().find_map(|count| {
            self.info
                .type_name_to_id(&words[..count].join(" "))
                .filter(|hull| self.category(*hull) == CATEGORY_SHIP)
        })
    }

    pub fn modes(&self, hull: i32) -> Vec<i32> {
        self.info
            .group_type_ids(GROUP_SHIP_MODIFIERS)
            .filter(|mode| self.mode_hull(*mode) == Some(hull))
            .collect()
    }

    /// A tactical mode's name without the hull's name.
    pub fn mode_name(&self, hull: i32, mode: i32) -> &'a str {
        let name = self.name(mode);
        name.strip_prefix(self.name(hull))
            .unwrap_or(name)
            .trim_start_matches(' ')
    }

    pub fn mutaplasmids(&self, base: i32) -> Vec<i32> {
        self.info
            .mutaplasmids()
            .into_iter()
            .filter(|mutaplasmid| {
                mutaplasmid.mappings().into_iter().flatten().any(|mapping| {
                    mapping
                        .applicable_type_ids()
                        .is_some_and(|type_ids| type_ids.iter().any(|type_id| type_id == base))
                })
            })
            .map(|mutaplasmid| mutaplasmid.id())
            .collect()
    }

    /// The type a mutaplasmid turns `base` into.
    pub fn mutated(&self, mutaplasmid: i32, base: i32) -> Option<i32> {
        self.info
            .mutaplasmids()
            .into_iter()
            .find(|candidate| candidate.id() == mutaplasmid)?
            .mappings()?
            .iter()
            .find(|mapping| {
                mapping
                    .applicable_type_ids()
                    .is_some_and(|type_ids| type_ids.iter().any(|type_id| type_id == base))
            })
            .map(|mapping| mapping.resulting_type_id())
    }

    pub fn rollable(&self, mutaplasmid: i32) -> Vec<i32> {
        self.info
            .mutaplasmids()
            .into_iter()
            .find(|candidate| candidate.id() == mutaplasmid)
            .and_then(|mutaplasmid| mutaplasmid.attributes())
            .into_iter()
            .flatten()
            .map(|attribute| attribute.attribute_id())
            .collect()
    }

    pub fn attribute(&self, type_id: i32, attribute_id: i32) -> Option<f64> {
        self.info
            .get_dogma_attributes(type_id)?
            .iter()
            .find(|attribute| attribute.attribute_id() == attribute_id)
            .map(|attribute| sde_decimal(attribute.value()))
    }

    pub fn base_value(&self, type_id: i32, attribute_id: i32) -> f64 {
        self.attribute(type_id, attribute_id).unwrap_or_else(|| {
            self.info
                .get_dogma_attribute(attribute_id)
                .map_or(0.0, |attribute| sde_decimal(attribute.default_value()))
        })
    }

    pub fn squadron_size(&self, type_id: i32) -> Option<u32> {
        self.attribute(type_id, ATTRIBUTE_SQUADRON_SIZE)
            .map(|size| size as u32)
    }

    /// How many charges fit in an item.
    pub fn full_load(&self, item: i32, charge: i32) -> Option<u32> {
        let capacity = self.r#type(item)?.capacity()?;
        let volume = self.r#type(charge)?.volume()?;
        if capacity <= 0.0 || volume <= 0.0 {
            return None;
        }
        divide_decimals(&capacity.to_string(), &volume.to_string())
    }
}

/// A decimal as an integer and the power of ten it is divided by.
fn scaled(decimal: &str) -> Option<(u128, u32)> {
    let (whole, fraction) = decimal.split_once('.').unwrap_or((decimal, ""));
    let digits = format!("{whole}{fraction}").parse().ok()?;
    Some((digits, fraction.len() as u32))
}

/// `numerator / denominator`, both decimals, exactly and rounded down.
fn divide_decimals(numerator: &str, denominator: &str) -> Option<u32> {
    let (numerator, numerator_scale) = scaled(numerator)?;
    let (denominator, denominator_scale) = scaled(denominator)?;
    let numerator = numerator.checked_mul(10u128.checked_pow(denominator_scale)?)?;
    let denominator = denominator.checked_mul(10u128.checked_pow(numerator_scale)?)?;
    (numerator / denominator).try_into().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divides_exactly() {
        assert_eq!(divide_decimals("2.3", "0.05"), Some(46));
        assert_eq!(divide_decimals("2.3", "0.06"), Some(38));
        assert_eq!(divide_decimals("10", "3"), Some(3));
    }

    #[test]
    fn sde_decimals() {
        assert_eq!(sde_decimal(0.1), 0.1);
        assert_eq!(sde_decimal(524.0), 524.0);
    }
}
