use super::super::item::{EffectCategory, SlotType};
use super::super::Ship;
use super::AttributeId;

pub fn attribute_drone_active(ship: &mut Ship) {
    /* The amount of active drones. */

    let mut active = 0;
    for item in &ship.items {
        if item.slot.r#type != SlotType::DroneBay || item.state != EffectCategory::Active {
            continue;
        }

        active += 1;
    }

    ship.hull
        .add_attribute(AttributeId::droneActive, 0.0, active as f64);
}

pub fn attribute_drone_capacity_used(ship: &mut Ship) {
    /* The capacity of the dronebay used by drones. */

    let attr_drone_capacity = AttributeId::volume as i32;

    let mut capacity_used = 0.0;
    for item in &ship.items {
        if item.slot.r#type != SlotType::DroneBay {
            continue;
        }

        capacity_used += item
            .attributes
            .get(&attr_drone_capacity)
            .unwrap()
            .value
            .unwrap();
    }

    ship.hull
        .add_attribute(AttributeId::droneCapacityUsed, 0.0, capacity_used);
}

pub fn attribute_drone_bandwidth_used(ship: &mut Ship) {
    /* The bandwidth used by drones. */

    let attr_drone_bandwidth = AttributeId::droneBandwidthUsed as i32;

    let mut bandwidth_used_total = 0.0;
    for item in &ship.items {
        if item.slot.r#type != SlotType::DroneBay || item.state != EffectCategory::Active {
            continue;
        }

        bandwidth_used_total += item
            .attributes
            .get(&attr_drone_bandwidth)
            .unwrap()
            .value
            .unwrap();
    }

    ship.hull.add_attribute(
        AttributeId::droneBandwidthUsedTotal,
        0.0,
        bandwidth_used_total,
    );
}

pub fn attribute_drone_damage(ship: &mut Ship) {
    /* The total damage of drones. */

    let mut total_alpha_hp = 0.0;
    let mut total_base_alpha_hp = 0.0;
    let mut total_alpha_hp_without_reload = 0.0;
    let mut total_base_alpha_hp_without_reload = 0.0;

    let attr_damage_alpha_hp = AttributeId::damageAlphaHp as i32;
    let attr_damage_without_reload_dps = AttributeId::damageWithoutReloadDps as i32;

    for item in &mut ship.items {
        if item.slot.r#type != SlotType::DroneBay || item.state != EffectCategory::Active {
            continue;
        }
        if !item.attributes.contains_key(&attr_damage_alpha_hp) {
            continue;
        }

        let damage_alpha_hp = item.attributes.get(&attr_damage_alpha_hp).unwrap();
        total_base_alpha_hp += damage_alpha_hp.base_value;
        total_alpha_hp += damage_alpha_hp.value.unwrap();

        let damage_without_reload_dps = item
            .attributes
            .get(&attr_damage_without_reload_dps)
            .unwrap();
        total_base_alpha_hp_without_reload += damage_without_reload_dps.base_value;
        total_alpha_hp_without_reload += damage_without_reload_dps.value.unwrap();
    }

    ship.hull.add_attribute(
        AttributeId::droneDamageAlphaHp,
        total_base_alpha_hp,
        total_alpha_hp,
    );
    ship.hull.add_attribute(
        AttributeId::droneDamageDps,
        total_base_alpha_hp_without_reload,
        total_alpha_hp_without_reload,
    );
}
