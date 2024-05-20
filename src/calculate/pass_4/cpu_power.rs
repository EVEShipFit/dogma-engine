use super::super::item::EffectCategory;
use super::super::Ship;
use super::AttributeId;

pub fn attribute_cpu_used(ship: &mut Ship) {
    /* How much CPU is being used, which is adding up cpu from all items. */

    let attr_cpu = AttributeId::cpu as i32;

    let mut cpu_used = 0.0;
    for item in &ship.items {
        if !item.slot.is_module() || item.state == EffectCategory::Passive {
            continue;
        }

        if item.attributes.contains_key(&attr_cpu) {
            cpu_used += item.attributes.get(&attr_cpu).unwrap().value.unwrap();
        }
    }

    ship.hull.add_attribute(AttributeId::cpuUsed, 0.0, cpu_used);
}

pub fn attribute_power_used(ship: &mut Ship) {
    /* How much PG is being used, which is adding up power from all items. */

    let attr_power = AttributeId::power as i32;

    let mut power_used = 0.0;
    for item in &ship.items {
        if !item.slot.is_module() || item.state == EffectCategory::Passive {
            continue;
        }

        if item.attributes.contains_key(&attr_power) {
            power_used += item.attributes.get(&attr_power).unwrap().value.unwrap();
        }
    }

    ship.hull
        .add_attribute(AttributeId::powerUsed, 0.0, power_used);
}

pub fn attribute_cpu_unused(ship: &mut Ship) {
    /* How much CPU is left, which is the total CPU minus the usage. */

    let attr_cpu_used = ship
        .hull
        .attributes
        .get(&(AttributeId::cpuUsed as i32))
        .unwrap();
    let attr_cpu_output = ship
        .hull
        .attributes
        .get(&(AttributeId::cpuOutput as i32))
        .unwrap();

    let cpu_used = attr_cpu_used.value.unwrap();
    let cpu_output = attr_cpu_output.value.unwrap();
    let cpu_unused = cpu_output - cpu_used;

    ship.hull
        .add_attribute(AttributeId::cpuUnused, 0.0, cpu_unused);
}

pub fn attribute_power_unused(ship: &mut Ship) {
    /* How much PG is left, which is the total PG minus the usage. */

    let attr_power_used = ship
        .hull
        .attributes
        .get(&(AttributeId::powerUsed as i32))
        .unwrap();
    let attr_power_output = ship
        .hull
        .attributes
        .get(&(AttributeId::powerOutput as i32))
        .unwrap();

    let power_used = attr_power_used.value.unwrap();
    let power_output = attr_power_output.value.unwrap();
    let power_unused = power_output - power_used;

    ship.hull
        .add_attribute(AttributeId::powerUnused, 0.0, power_unused);
}
