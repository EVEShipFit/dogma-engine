use super::super::Ship;
use super::AttributeId;

const EHP_ATTRIBUTES: [i32; 3] = [
    AttributeId::shieldEhp as i32,
    AttributeId::armorEhp as i32,
    AttributeId::hullEhp as i32,
];

pub fn attribute_shield_ehp_multiplier(ship: &mut Ship) {
    /* Based on the damage profile, calculate the shield eHP multiplier (how much damage is mitigated due to the resistance). */

    let attr_shield_em = ship
        .hull
        .attributes
        .get(&(AttributeId::shieldEmDamageResonance as i32))
        .unwrap();
    let attr_shield_explosive = ship
        .hull
        .attributes
        .get(&(AttributeId::shieldExplosiveDamageResonance as i32))
        .unwrap();
    let attr_shield_kinetic = ship
        .hull
        .attributes
        .get(&(AttributeId::shieldKineticDamageResonance as i32))
        .unwrap();
    let attr_shield_thermal = ship
        .hull
        .attributes
        .get(&(AttributeId::shieldThermalDamageResonance as i32))
        .unwrap();

    let base_shield_em = attr_shield_em.base_value * ship.damage_profile.em;
    let base_shield_explosive = attr_shield_explosive.base_value * ship.damage_profile.explosive;
    let base_shield_kinetic = attr_shield_kinetic.base_value * ship.damage_profile.kinetic;
    let base_shield_thermal = attr_shield_thermal.base_value * ship.damage_profile.thermal;

    let shield_em_multiplier = attr_shield_em.value.unwrap() * ship.damage_profile.em;
    let shield_explosive_multiplier =
        attr_shield_explosive.value.unwrap() * ship.damage_profile.explosive;
    let shield_kinetic_multiplier =
        attr_shield_kinetic.value.unwrap() * ship.damage_profile.kinetic;
    let shield_thermal_multiplier =
        attr_shield_thermal.value.unwrap() * ship.damage_profile.thermal;

    /* Damage profile added up is always one; the initial damage divided by the actual damage is how much more eHP we have compared to HP. */
    let base_shield_ehp_multiplier =
        1.0 / (base_shield_em + base_shield_explosive + base_shield_kinetic + base_shield_thermal);
    let shield_ehp_multiplier = 1.0
        / (shield_em_multiplier
            + shield_explosive_multiplier
            + shield_kinetic_multiplier
            + shield_thermal_multiplier);

    ship.add_attribute(
        AttributeId::shieldEhpMultiplier,
        base_shield_ehp_multiplier,
        shield_ehp_multiplier,
    );
}

pub fn attribute_armor_ehp_multiplier(ship: &mut Ship) {
    /* Based on the damage profile, calculate the armor eHP multiplier (how much damage is mitigated due to the resistance). */

    let attr_armor_em = ship
        .hull
        .attributes
        .get(&(AttributeId::armorEmDamageResonance as i32))
        .unwrap();
    let attr_armor_explosive = ship
        .hull
        .attributes
        .get(&(AttributeId::armorExplosiveDamageResonance as i32))
        .unwrap();
    let attr_armor_kinetic = ship
        .hull
        .attributes
        .get(&(AttributeId::armorKineticDamageResonance as i32))
        .unwrap();
    let attr_armor_thermal = ship
        .hull
        .attributes
        .get(&(AttributeId::armorThermalDamageResonance as i32))
        .unwrap();

    let base_armor_em = attr_armor_em.base_value * ship.damage_profile.em;
    let base_armor_explosive = attr_armor_explosive.base_value * ship.damage_profile.explosive;
    let base_armor_kinetic = attr_armor_kinetic.base_value * ship.damage_profile.kinetic;
    let base_armor_thermal = attr_armor_thermal.base_value * ship.damage_profile.thermal;

    let armor_em_multiplier = attr_armor_em.value.unwrap() * ship.damage_profile.em;
    let armor_explosive_multiplier =
        attr_armor_explosive.value.unwrap() * ship.damage_profile.explosive;
    let armor_kinetic_multiplier = attr_armor_kinetic.value.unwrap() * ship.damage_profile.kinetic;
    let armor_thermal_multiplier = attr_armor_thermal.value.unwrap() * ship.damage_profile.thermal;

    /* Damage profile added up is always one; the initial damage divided by the actual damage is how much more eHP we have compared to HP. */
    let base_armor_ehp_multiplier =
        1.0 / (base_armor_em + base_armor_explosive + base_armor_kinetic + base_armor_thermal);
    let armor_ehp_multiplier = 1.0
        / (armor_em_multiplier
            + armor_explosive_multiplier
            + armor_kinetic_multiplier
            + armor_thermal_multiplier);

    ship.add_attribute(
        AttributeId::armorEhpMultiplier,
        base_armor_ehp_multiplier,
        armor_ehp_multiplier,
    );
}

pub fn attribute_hull_ehp_multiplier(ship: &mut Ship) {
    /* Based on the damage profile, calculate the hull eHP multiplier (how much damage is mitigated due to the resistance). */

    let attr_hull_em = ship
        .hull
        .attributes
        .get(&(AttributeId::emDamageResonance as i32))
        .unwrap();
    let attr_hull_explosive = ship
        .hull
        .attributes
        .get(&(AttributeId::explosiveDamageResonance as i32))
        .unwrap();
    let attr_hull_kinetic = ship
        .hull
        .attributes
        .get(&(AttributeId::kineticDamageResonance as i32))
        .unwrap();
    let attr_hull_thermal = ship
        .hull
        .attributes
        .get(&(AttributeId::thermalDamageResonance as i32))
        .unwrap();

    let base_hull_em = attr_hull_em.base_value * ship.damage_profile.em;
    let base_hull_explosive = attr_hull_explosive.base_value * ship.damage_profile.explosive;
    let base_hull_kinetic = attr_hull_kinetic.base_value * ship.damage_profile.kinetic;
    let base_hull_thermal = attr_hull_thermal.base_value * ship.damage_profile.thermal;

    let hull_em_multiplier = attr_hull_em.value.unwrap() * ship.damage_profile.em;
    let hull_explosive_multiplier =
        attr_hull_explosive.value.unwrap() * ship.damage_profile.explosive;
    let hull_kinetic_multiplier = attr_hull_kinetic.value.unwrap() * ship.damage_profile.kinetic;
    let hull_thermal_multiplier = attr_hull_thermal.value.unwrap() * ship.damage_profile.thermal;

    /* Damage profile added up is always one; the initial damage divided by the actual damage is how much more eHP we have compared to HP. */
    let base_hull_ehp_multiplier =
        1.0 / (base_hull_em + base_hull_explosive + base_hull_kinetic + base_hull_thermal);
    let hull_ehp_multiplier = 1.0
        / (hull_em_multiplier
            + hull_explosive_multiplier
            + hull_kinetic_multiplier
            + hull_thermal_multiplier);

    ship.add_attribute(
        AttributeId::hullEhpMultiplier,
        base_hull_ehp_multiplier,
        hull_ehp_multiplier,
    );
}

pub fn attribute_shield_ehp(ship: &mut Ship) {
    /* Calculate the shield eHP based on the shield HP and shield eHP multiplier. */

    let attr_shield_ehp_multiplier = ship
        .hull
        .attributes
        .get(&(AttributeId::shieldEhpMultiplier as i32))
        .unwrap();
    let attr_shield_hp = ship
        .hull
        .attributes
        .get(&(AttributeId::shieldCapacity as i32))
        .unwrap();

    let base_shield_ehp_multiplier = attr_shield_ehp_multiplier.base_value;
    let base_shield_hp = attr_shield_hp.base_value;
    let base_shield_ehp = base_shield_hp * base_shield_ehp_multiplier;

    let shield_ehp_multiplier = attr_shield_ehp_multiplier.value.unwrap();
    let shield_hp = attr_shield_hp.value.unwrap();
    let shield_ehp = shield_hp * shield_ehp_multiplier;

    ship.add_attribute(AttributeId::shieldEhp, base_shield_ehp, shield_ehp);
}

pub fn attribute_armor_ehp(ship: &mut Ship) {
    /* Calculate the armor eHP based on the armor HP and armor eHP multiplier. */

    let attr_armor_ehp_multiplier = ship
        .hull
        .attributes
        .get(&(AttributeId::armorEhpMultiplier as i32))
        .unwrap();
    let attr_armor_hp = ship
        .hull
        .attributes
        .get(&(AttributeId::armorHp as i32))
        .unwrap();

    let base_armor_ehp_multiplier = attr_armor_ehp_multiplier.base_value;
    let base_armor_hp = attr_armor_hp.base_value;
    let base_armor_ehp = base_armor_hp * base_armor_ehp_multiplier;

    let armor_ehp_multiplier = attr_armor_ehp_multiplier.value.unwrap();
    let armor_hp = attr_armor_hp.value.unwrap();
    let armor_ehp = armor_hp * armor_ehp_multiplier;

    ship.add_attribute(AttributeId::armorEhp, base_armor_ehp, armor_ehp);
}

pub fn attribute_hull_ehp(ship: &mut Ship) {
    /* Calculate the hull eHP based on the hull HP and hull eHP multiplier. */

    let attr_hull_ehp_multiplier = ship
        .hull
        .attributes
        .get(&(AttributeId::hullEhpMultiplier as i32))
        .unwrap();
    let attr_hull_hp = ship.hull.attributes.get(&(AttributeId::hp as i32)).unwrap();

    let base_hull_ehp_multiplier = attr_hull_ehp_multiplier.base_value;
    let base_hull_hp = attr_hull_hp.base_value;
    let base_hull_ehp = base_hull_hp * base_hull_ehp_multiplier;

    let hull_ehp_multiplier = attr_hull_ehp_multiplier.value.unwrap();
    let hull_hp = attr_hull_hp.value.unwrap();
    let hull_ehp = hull_hp * hull_ehp_multiplier;

    ship.add_attribute(AttributeId::hullEhp, base_hull_ehp, hull_ehp);
}

pub fn attribute_ehp(ship: &mut Ship) {
    /* Adds up the shield + armor + hull ehp to come to a total ehp of the ship. */

    let mut base_ehp = 0.0;
    let mut ehp = 0.0;

    for attribute_id in EHP_ATTRIBUTES.iter() {
        let attribute = ship.hull.attributes.get(attribute_id).unwrap();

        base_ehp += attribute.base_value;
        ehp += attribute.value.unwrap();
    }

    ship.add_attribute(AttributeId::ehp, base_ehp, ehp);
}
