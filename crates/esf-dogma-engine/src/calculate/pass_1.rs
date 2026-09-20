use esf_data::eve;

use super::attribute_ids::{
    ATTRIBUTE_CAPACITY_ID, ATTRIBUTE_MASS_ID, ATTRIBUTE_PILOT_SECURITY_STATUS_ID,
    ATTRIBUTE_RADIUS_ID, ATTRIBUTE_SKILL_LEVEL_ID, ATTRIBUTE_VOLUME_ID,
};
use super::item::{Attribute, Item};
use super::{Info, Objects, Projected};
use crate::fit::{DamageProfile, Fit, Mutation, ReactiveArmor, Security, Spool};
use crate::projection::ProjectedBuff;

pub struct PassOne {}

impl Item {
    pub fn set_attribute(&mut self, attribute_id: i32, value: f64) {
        self.attributes.insert(attribute_id, Attribute::new(value));
    }

    /* What pass 3 worked out, or the base value until it has run. */
    pub(super) fn value_of(&self, attribute_id: i32) -> f64 {
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

    pub(super) fn set_attributes(&mut self, info: &impl Info) {
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
        let names = [
            "damageProfileEm",
            "damageProfileExplosive",
            "damageProfileKinetic",
            "damageProfileThermal",
        ];

        for (name, value) in names.iter().zip(normalized(profile)) {
            if let Some(attribute_id) = info.attribute_name_to_id(name) {
                self.set_attribute(attribute_id, value);
            }
        }
    }
}

/* Only the ratio between the four matters, and whoever reads them wants them
 * to add up to one. */
fn normalized(profile: DamageProfile) -> [f64; 4] {
    let total = profile.em + profile.explosive + profile.kinetic + profile.thermal;
    let (profile, total) = if total > 0.0 {
        (profile, total)
    } else {
        (DamageProfile::default(), 1.0)
    };

    [
        profile.em,
        profile.explosive,
        profile.kinetic,
        profile.thermal,
    ]
    .map(|value| value / total)
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
        objects.reactive_armor = match fit.environment.reactive_armor {
            ReactiveArmor::DoNotAdapt => None,
            ReactiveArmor::DamageProfile => Some(normalized(fit.environment.damage_profile)),
            ReactiveArmor::Profile(profile) => Some(normalized(profile)),
        };
        if let Some(mode) = objects.mode.as_mut() {
            mode.set_attributes(info);
        }

        /* This carries no attributes, but pass 2 still wants its category. */
        objects.char.set_type_ids(info);

        /* What is aimed at the fit brings its own values; the type behind it
         * is only read for the category the stacking penalty needs. */
        for projected in &fit.incoming.effects {
            let mut item = Item::new_projected(projected.type_id);

            item.set_type_ids(info);
            for (attribute_id, value) in &projected.attributes {
                item.set_attribute(*attribute_id, *value);
            }

            objects.projected.push(Projected {
                effect_id: projected.effect_id,
                item,
            });
        }

        objects.buffs = resolve(info, &fit.incoming.buffs);

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
 * highest or the lowest of them wins. Only the winner is kept: what lost is
 * still in `Fit::incoming` for whoever wants to know. */
fn resolve(info: &impl Info, buffs: &[ProjectedBuff]) -> Vec<ProjectedBuff> {
    let strongest = |buff: &ProjectedBuff| match info
        .get_dbuff_collection(buff.id)
        .map(|collection| collection.aggregate_mode())
    {
        Some(eve::DbuffAggregateMode::Minimum) => buff.value,
        _ => -buff.value,
    };

    let mut buffs: Vec<ProjectedBuff> = buffs
        .iter()
        .filter(|buff| info.get_dbuff_collection(buff.id).is_some())
        .copied()
        .collect();
    buffs.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then(strongest(left).total_cmp(&strongest(right)))
    });

    /* Sorted, so the first of an id is the one that wins. */
    buffs.dedup_by_key(|buff| buff.id);
    buffs
}
