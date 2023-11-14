use super::item::{Attribute, EffectCategory};
use super::{Info, Pass, Ship};

pub struct PassFour {}

fn add_attribute(ship: &mut Ship, attribute_id: i32, base_value: f64, value: f64) {
    let mut attribute = Attribute::new(base_value);
    attribute.value = Some(value);
    ship.hull.attributes.insert(attribute_id, attribute);
}

fn calculate_align_time(ship: &mut Ship) -> (f64, f64) {
    /* Align-time is based on agility (70) and mass (4). */

    let base_agility = ship.hull.attributes.get(&70).unwrap().base_value;
    let base_mass = ship.hull.attributes.get(&4).unwrap().base_value;
    let base_align_time = -(0.25 as f64).ln() * base_agility * base_mass / 1000000.0;

    let agility = ship.hull.attributes.get(&70).unwrap().value.unwrap();
    let mass = ship.hull.attributes.get(&4).unwrap().value.unwrap();
    let align_time = -(0.25 as f64).ln() * agility * mass / 1000000.0;

    (base_align_time, align_time)
}

fn add_scan_strength(ship: &mut Ship) -> (f64, f64) {
    /* Scan Strength can be one of 4 values. */

    let mut base_scan_strength = 0.0;
    let mut scan_strength = 0.0;
    for attribute_id in vec![208, 209, 210, 211].iter() {
        if ship.hull.attributes.contains_key(attribute_id) {
            let attribute = ship.hull.attributes.get(attribute_id).unwrap();

            if attribute.base_value > base_scan_strength {
                base_scan_strength = attribute.base_value;
            }
            if attribute.value.unwrap() > scan_strength {
                scan_strength = attribute.value.unwrap();
            }
        }
    }

    (base_scan_strength, scan_strength)
}

fn add_cpu_used(ship: &mut Ship) -> (f64, f64) {
    /* How much CPU is being used, which is adding up cpuOuput (50) from all items. */

    let mut cpu_used = 0.0;
    for item in &ship.items {
        if item.attributes.contains_key(&50) && item.state != EffectCategory::Passive {
            cpu_used += item.attributes.get(&50).unwrap().value.unwrap();
        }
    }

    (0.0, cpu_used)
}

fn add_pg_used(ship: &mut Ship) -> (f64, f64) {
    /* How much PG is being used, which is adding up powerOutput (30) from all items. */

    let mut pg_used = 0.0;
    for item in &ship.items {
        if item.attributes.contains_key(&30) && item.state != EffectCategory::Passive {
            pg_used += item.attributes.get(&30).unwrap().value.unwrap();
        }
    }

    (0.0, pg_used)
}

fn add_cpu_unused(ship: &mut Ship) -> (f64, f64) {
    /* How much CPU is left, which is the total CPU minus the usage. */

    let cpu_used = ship.hull.attributes.get(&-3).unwrap().value.unwrap();
    let cpu_output = ship.hull.attributes.get(&48).unwrap().value.unwrap();
    let cpu_unused = cpu_output - cpu_used;

    (0.0, cpu_unused)
}

fn add_pg_unused(ship: &mut Ship) -> (f64, f64) {
    /* How much PG is left, which is the total PG minus the usage. */

    let pg_used = ship.hull.attributes.get(&-4).unwrap().value.unwrap();
    let pg_output = ship.hull.attributes.get(&11).unwrap().value.unwrap();
    let pg_unused = pg_output - pg_used;

    (0.0, pg_unused)
}

/* Attributes don't contain all information displayed, so we calculate some fake attributes with those values. */
impl Pass for PassFour {
    fn pass(_info: &Info, ship: &mut Ship) {
        let align_time = calculate_align_time(ship);
        add_attribute(ship, -1, align_time.0, align_time.1);

        let scan_strength = add_scan_strength(ship);
        add_attribute(ship, -2, scan_strength.0, scan_strength.1);

        let cpu_used = add_cpu_used(ship);
        add_attribute(ship, -3, cpu_used.0, cpu_used.1);

        let pg_used = add_pg_used(ship);
        add_attribute(ship, -4, pg_used.0, pg_used.1);

        let cpu_unused = add_cpu_unused(ship);
        add_attribute(ship, -5, cpu_unused.0, cpu_unused.1);

        let pg_unused = add_pg_unused(ship);
        add_attribute(ship, -6, pg_unused.0, pg_unused.1);
    }
}
