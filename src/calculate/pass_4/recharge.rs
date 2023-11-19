use super::super::item::EffectCategory;
use super::super::Ship;
use super::AttributeId;

pub fn attribute_passive_shield_recharge(ship: &mut Ship) {
    /* The passive recharge rate of shield, in HP/s and eHP/s. */

    let attr_shield_recharge_rate = ship
        .hull
        .attributes
        .get(&(AttributeId::shieldRechargeRate as i32))
        .unwrap();
    let attr_shield_capacity = ship
        .hull
        .attributes
        .get(&(AttributeId::shieldCapacity as i32))
        .unwrap();
    let attr_shield_ehp_multiplier = ship
        .hull
        .attributes
        .get(&(AttributeId::shieldEhpMultiplier as i32))
        .unwrap();

    let base_shield_recharge_rate = attr_shield_recharge_rate.base_value / 1000.0;
    let base_shield_capacity = attr_shield_capacity.base_value;
    let base_passive_shield_recharge = 2.5 / base_shield_recharge_rate * base_shield_capacity;
    let base_shield_ehp_multiplier = attr_shield_ehp_multiplier.base_value;

    let shield_recharge_rate = attr_shield_recharge_rate.value.unwrap() / 1000.0;
    let shield_capacity = attr_shield_capacity.value.unwrap();
    let passive_shield_recharge = 2.5 / shield_recharge_rate * shield_capacity;
    let shield_ehp_multiplier = attr_shield_ehp_multiplier.value.unwrap();

    ship.add_attribute(
        AttributeId::passiveShieldRecharge,
        base_passive_shield_recharge,
        passive_shield_recharge,
    );
    ship.add_attribute(
        AttributeId::passiveShieldRechargeEhp,
        base_passive_shield_recharge * base_shield_ehp_multiplier,
        passive_shield_recharge * shield_ehp_multiplier,
    );
}

pub fn attribute_shield_recharge(ship: &mut Ship) {
    /* The (active) recharge rate of shield, in HP/s and eHP/s. */

    let attr_shield_ehp_multiplier = ship
        .hull
        .attributes
        .get(&(AttributeId::shieldEhpMultiplier as i32))
        .unwrap();
    let attr_duration = AttributeId::duration as i32;
    let attr_shield_bonus = AttributeId::shieldBonus as i32;

    let mut base_shield_recharge = 0.0;
    for item in &ship.items {
        if item.attributes.contains_key(&attr_shield_bonus) && item.state >= EffectCategory::Active
        {
            let duration = item.attributes.get(&attr_duration).unwrap().base_value / 1000.0;
            base_shield_recharge +=
                item.attributes.get(&attr_shield_bonus).unwrap().base_value / duration;
        }
    }
    let base_shield_ehp_multiplier = attr_shield_ehp_multiplier.base_value;

    let mut shield_recharge = 0.0;
    for item in &ship.items {
        if item.attributes.contains_key(&attr_shield_bonus) && item.state >= EffectCategory::Active
        {
            let duration = item.attributes.get(&attr_duration).unwrap().value.unwrap() / 1000.0;
            shield_recharge += item
                .attributes
                .get(&attr_shield_bonus)
                .unwrap()
                .value
                .unwrap()
                / duration;
        }
    }
    let shield_ehp_multiplier = attr_shield_ehp_multiplier.value.unwrap();

    ship.add_attribute(
        AttributeId::shieldRecharge,
        base_shield_recharge,
        shield_recharge,
    );
    ship.add_attribute(
        AttributeId::shieldRechargeEhp,
        base_shield_recharge * base_shield_ehp_multiplier,
        shield_recharge * shield_ehp_multiplier,
    );
}

pub fn attribute_armor_recharge(ship: &mut Ship) {
    /* The recharge rate of armor, in HP/s and eHP/s. */

    let attr_armor_ehp_multiplier = ship
        .hull
        .attributes
        .get(&(AttributeId::armorEhpMultiplier as i32))
        .unwrap();
    let attr_duration = AttributeId::duration as i32;
    let attr_armor_bonus = AttributeId::armorDamageAmount as i32;

    let mut base_armor_recharge = 0.0;
    for item in &ship.items {
        if item.attributes.contains_key(&attr_armor_bonus) && item.state >= EffectCategory::Active {
            let duration = item.attributes.get(&attr_duration).unwrap().base_value / 1000.0;
            base_armor_recharge +=
                item.attributes.get(&attr_armor_bonus).unwrap().base_value / duration;
        }
    }
    let base_armor_ehp_multiplier = attr_armor_ehp_multiplier.base_value;

    let mut armor_recharge = 0.0;
    for item in &ship.items {
        if item.attributes.contains_key(&attr_armor_bonus) && item.state >= EffectCategory::Active {
            let duration = item.attributes.get(&attr_duration).unwrap().value.unwrap() / 1000.0;
            armor_recharge += item
                .attributes
                .get(&attr_armor_bonus)
                .unwrap()
                .value
                .unwrap()
                / duration;
        }
    }
    let armor_ehp_multiplier = attr_armor_ehp_multiplier.value.unwrap();

    ship.add_attribute(
        AttributeId::armorRecharge,
        base_armor_recharge,
        armor_recharge,
    );
    ship.add_attribute(
        AttributeId::armorRechargeEhp,
        base_armor_recharge * base_armor_ehp_multiplier,
        armor_recharge * armor_ehp_multiplier,
    );
}

pub fn attribute_hull_recharge(ship: &mut Ship) {
    /* The recharge rate of hull, in HP/s and eHP/s. */

    let attr_hull_ehp_multiplier = ship
        .hull
        .attributes
        .get(&(AttributeId::hullEhpMultiplier as i32))
        .unwrap();
    let attr_duration = AttributeId::duration as i32;
    let attr_hull_bonus = AttributeId::structureDamageAmount as i32;

    let mut base_hull_recharge = 0.0;
    for item in &ship.items {
        if item.attributes.contains_key(&attr_hull_bonus) && item.state >= EffectCategory::Active {
            let duration = item.attributes.get(&attr_duration).unwrap().base_value / 1000.0;
            base_hull_recharge +=
                item.attributes.get(&attr_hull_bonus).unwrap().base_value / duration;
        }
    }
    let base_hull_ehp_multiplier = attr_hull_ehp_multiplier.base_value;

    let mut hull_recharge = 0.0;
    for item in &ship.items {
        if item.attributes.contains_key(&attr_hull_bonus) && item.state >= EffectCategory::Active {
            let duration = item.attributes.get(&attr_duration).unwrap().value.unwrap() / 1000.0;
            hull_recharge += item
                .attributes
                .get(&attr_hull_bonus)
                .unwrap()
                .value
                .unwrap()
                / duration;
        }
    }
    let hull_ehp_multiplier = attr_hull_ehp_multiplier.value.unwrap();

    ship.add_attribute(AttributeId::hullRecharge, base_hull_recharge, hull_recharge);
    ship.add_attribute(
        AttributeId::hullRechargeEhp,
        base_hull_recharge * base_hull_ehp_multiplier,
        hull_recharge * hull_ehp_multiplier,
    );
}
