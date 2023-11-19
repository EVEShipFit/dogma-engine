use super::super::item::EffectCategory;
use super::super::Ship;
use super::AttributeId;

pub fn attribute_cpu_used(ship: &mut Ship) {
    /* How much CPU is being used, which is adding up cpu from all items. */

    let attr_cpu = AttributeId::cpu as i32;

    let mut cpu_used = 0.0;
    for item in &ship.items {
        if item.attributes.contains_key(&attr_cpu) && item.state != EffectCategory::Passive {
            cpu_used += item.attributes.get(&attr_cpu).unwrap().value.unwrap();
        }
    }

    ship.add_attribute(AttributeId::cpuUsed, 0.0, cpu_used);
}

pub fn attribute_power_used(ship: &mut Ship) {
    /* How much PG is being used, which is adding up power from all items. */

    let attr_power = AttributeId::power as i32;

    let mut power_used = 0.0;
    for item in &ship.items {
        if item.attributes.contains_key(&attr_power) && item.state != EffectCategory::Passive {
            power_used += item.attributes.get(&attr_power).unwrap().value.unwrap();
        }
    }

    ship.add_attribute(AttributeId::powerUsed, 0.0, power_used);
}

pub fn attribute_cpu_unused(ship: &mut Ship) {
    /* How much CPU is left, which is the total CPU minus the usage. */

    let attr_cpu_used = AttributeId::cpuUsed as i32;
    let attr_cpu_output = AttributeId::cpuOutput as i32;

    let cpu_used = ship
        .hull
        .attributes
        .get(&attr_cpu_used)
        .unwrap()
        .value
        .unwrap();
    let cpu_output = ship
        .hull
        .attributes
        .get(&attr_cpu_output)
        .unwrap()
        .value
        .unwrap();
    let cpu_unused = cpu_output - cpu_used;

    ship.add_attribute(AttributeId::cpuUnused, 0.0, cpu_unused);
}

pub fn attribute_power_unused(ship: &mut Ship) {
    /* How much PG is left, which is the total PG minus the usage. */

    let attr_power_used = AttributeId::powerUsed as i32;
    let attr_power_output = AttributeId::powerOutput as i32;

    let power_used = ship
        .hull
        .attributes
        .get(&attr_power_used)
        .unwrap()
        .value
        .unwrap();
    let power_output = ship
        .hull
        .attributes
        .get(&attr_power_output)
        .unwrap()
        .value
        .unwrap();
    let power_unused = power_output - power_used;

    ship.add_attribute(AttributeId::powerUnused, 0.0, power_unused);
}
