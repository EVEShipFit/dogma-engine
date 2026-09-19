use std::collections::BTreeSet;

use esf_data::eve;

use super::attribute_ids::{
    ATTRIBUTE_CAPACITY_ID, ATTRIBUTE_MASS_ID, ATTRIBUTE_PILOT_SECURITY_STATUS_ID,
    ATTRIBUTE_RADIUS_ID, ATTRIBUTE_SKILL_LEVEL_ID, ATTRIBUTE_VOLUME_ID,
};
use super::item::{Attribute, Item};
use super::output::{BuffResult, BuffSource};
use super::{Info, Objects};
use crate::fit::{DamageProfile, Fit, Mutation, Security, Spool};

/* warfareBuff1 to warfareBuff4, as their (id, value) attribute pair. */
const WARFARE_BUFF_ATTRIBUTE_IDS: [(i32, i32); 4] =
    [(2468, 2469), (2470, 2471), (2472, 2473), (2536, 2537)];

pub struct PassOne {}

/// A buff on offer, before it is known whether it wins.
struct Candidate {
    id: i32,
    value: f64,
    from: BuffSource,
}

impl Item {
    pub fn set_attribute(&mut self, attribute_id: i32, value: f64) {
        self.attributes.insert(attribute_id, Attribute::new(value));
    }

    /// The buffs this item offers, read off its four `warfareBuff` pairs.
    fn warfare_buffs(&self, from: BuffSource) -> impl Iterator<Item = Candidate> {
        WARFARE_BUFF_ATTRIBUTE_IDS.into_iter().filter_map(
            move |(id_attribute_id, value_attribute_id)| match self.value_of(id_attribute_id) as i32
            {
                0 => None,
                id => Some(Candidate {
                    id,
                    value: self.value_of(value_attribute_id),
                    from,
                }),
            },
        )
    }

    /* What pass 3 worked out, or the base value until it has run. */
    fn value_of(&self, attribute_id: i32) -> f64 {
        self.attributes.get(&attribute_id).map_or(0.0, |attribute| {
            attribute.value.get().unwrap_or(attribute.base_value)
        })
    }

    fn set_type_ids(&mut self, info: &impl Info) {
        if let Some(r#type) = info.get_type(self.type_id) {
            self.group_id = r#type.group_id();
            self.category_id = r#type.category_id();
        }
    }

    fn set_attributes(&mut self, info: &impl Info) {
        self.set_type_ids(info);
        self.set_type_attributes(info, self.type_id);
    }

    /* The mutated type holds only what sets it apart from its bases, like its
     * skill requirements; the rest comes from the base. */
    fn set_mutated_attributes(&mut self, info: &impl Info, mutation: &Mutation) {
        self.set_type_ids(info);
        self.set_type_attributes(info, mutation.base);
        self.set_type_attributes(info, self.type_id);

        for (attribute_id, value) in &mutation.attributes {
            self.set_attribute(*attribute_id, *value);
        }
    }

    fn set_type_attributes(&mut self, info: &impl Info, type_id: i32) {
        if let Some(dogma_attributes) = info.get_dogma_attributes(type_id) {
            for dogma_attribute in dogma_attributes {
                self.set_attribute(
                    dogma_attribute.attribute_id(),
                    dogma_attribute.value() as f64,
                );
            }
        }

        /* Some attributes of items come from the Type information. */
        let Some(r#type) = info.get_type(type_id) else {
            return;
        };
        if let Some(mass) = r#type.mass() {
            self.set_attribute(ATTRIBUTE_MASS_ID, mass as f64);
        }
        if let Some(capacity) = r#type.capacity() {
            self.set_attribute(ATTRIBUTE_CAPACITY_ID, capacity as f64);
        }
        if let Some(volume) = r#type.volume() {
            self.set_attribute(ATTRIBUTE_VOLUME_ID, volume as f64);
        }
        if let Some(radius) = r#type.radius() {
            self.set_attribute(ATTRIBUTE_RADIUS_ID, radius as f64);
        }
    }

    fn set_damage_profile(&mut self, info: &impl Info, profile: DamageProfile) {
        let total = profile.em + profile.explosive + profile.kinetic + profile.thermal;
        /* Effective hitpoints are only right when the four add up to one. */
        let (profile, total) = if total > 0.0 {
            (profile, total)
        } else {
            (DamageProfile::default(), 1.0)
        };

        for (name, value) in [
            ("damageProfileEm", profile.em),
            ("damageProfileExplosive", profile.explosive),
            ("damageProfileKinetic", profile.kinetic),
            ("damageProfileThermal", profile.thermal),
        ] {
            if let Some(attribute_id) = info.attribute_name_to_id(name) {
                self.set_attribute(attribute_id, value / total);
            }
        }
    }
}

impl PassOne {
    pub fn pass(info: &impl Info, fit: &Fit) -> Objects {
        let mut objects = Objects::new(fit.ship.type_id, fit.ship.mode);

        objects.ship.set_attributes(info);
        /* Only ships with a bonus that scales with it carry the attribute. */
        if objects
            .ship
            .attributes
            .contains_key(&ATTRIBUTE_PILOT_SECURITY_STATUS_ID)
        {
            objects.ship.set_attribute(
                ATTRIBUTE_PILOT_SECURITY_STATUS_ID,
                fit.character.security_status,
            );
        }
        objects
            .ship
            .set_damage_profile(info, fit.environment.damage_profile);
        if let Some(mode) = objects.mode.as_mut() {
            mode.set_attributes(info);
        }

        /* These carry no attributes, but pass 2 still wants their category. */
        objects.char.set_type_ids(info);
        objects.target.set_type_ids(info);

        let mut candidates = Vec::new();
        for type_id in &fit.environment.beacons {
            let mut beacon = Item::new_beacon(*type_id);

            beacon.set_attributes(info);
            candidates.extend(beacon.warfare_buffs(BuffSource::Beacon { type_id: *type_id }));

            objects.beacons.push(beacon);
        }
        objects.buffs = resolve(info, candidates);

        for (skill_id, skill_level) in &fit.character.skills {
            let mut skill = Item::new_fake(*skill_id);

            skill.set_attributes(info);
            skill.set_attribute(ATTRIBUTE_SKILL_LEVEL_ID, *skill_level as f64);

            objects.skills.push(skill);
        }

        let attr_spool_multiplier_bonus_id = info.attribute_name_to_id("spoolMultiplierBonus");
        let attr_security_modifier_id = info.attribute_name_to_id("securityModifier");
        let attr_system_modifier_id = info.attribute_name_to_id(match fit.environment.security {
            Security::HighSec => "hiSecModifier",
            Security::LowSec => "lowSecModifier",
            Security::NullSec | Security::Wormhole => "nullSecModifier",
        });

        for fit_item in &fit.items {
            let mut item = Item::new_fit(fit_item);

            if item.is_calculated() {
                match &fit_item.mutation {
                    Some(mutation) => item.set_mutated_attributes(info, mutation),
                    None => item.set_attributes(info),
                }
                if let Some(charge) = item.charge.as_mut() {
                    charge.set_attributes(info)
                }

                /* Set as the final value, so it replaces the maximum dogma
                 * would otherwise calculate. */
                if let (Some(Spool::MultiplierBonus(bonus)), Some(attribute_id)) =
                    (fit_item.spool, attr_spool_multiplier_bonus_id)
                {
                    item.add_attribute(attribute_id, bonus, bonus);
                }

                let system_modifier = attr_system_modifier_id
                    .and_then(|attribute_id| item.attributes.get(&attribute_id))
                    .map(|attribute| attribute.base_value);
                if let (Some(modifier), Some(attribute_id)) =
                    (system_modifier, attr_security_modifier_id)
                {
                    item.set_attribute(attribute_id, modifier);
                }
            }

            objects.items.push(item);
        }

        objects
    }
}

/* Two sources of the same buff do not add up; the buff says whether the
 * highest or the lowest of them wins. The rest are kept all the same, so that
 * a result shows what was on offer. */
fn resolve(info: &impl Info, mut candidates: Vec<Candidate>) -> Vec<BuffResult> {
    candidates.retain(|candidate| info.get_dbuff_collection(candidate.id).is_some());

    let strongest = |candidate: &Candidate| match info
        .get_dbuff_collection(candidate.id)
        .map(|collection| collection.aggregate_mode())
    {
        Some(eve::DbuffAggregateMode::Minimum) => candidate.value,
        _ => -candidate.value,
    };
    candidates.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then(strongest(left).total_cmp(&strongest(right)))
    });

    let mut won = BTreeSet::new();
    candidates
        .into_iter()
        .map(|candidate| BuffResult {
            id: candidate.id,
            value: candidate.value,
            from: candidate.from,
            /* Sorted, so the first of an id is the one that wins. */
            applied: won.insert(candidate.id),
        })
        .collect()
}
