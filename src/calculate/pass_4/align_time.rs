use super::super::Ship;
use super::AttributeId;

pub fn attribute_align_time(ship: &mut Ship) {
    /* Align-time is based on agility and mass. */

    let attr_agility = AttributeId::agility as i32;
    let attr_mass = AttributeId::mass as i32;

    let base_agility = ship.hull.attributes.get(&attr_agility).unwrap().base_value;
    let base_mass = ship.hull.attributes.get(&attr_mass).unwrap().base_value;
    let base_align_time = -(0.25 as f64).ln() * base_agility * base_mass / 1000000.0;

    let agility = ship
        .hull
        .attributes
        .get(&attr_agility)
        .unwrap()
        .value
        .unwrap();
    let mass = ship.hull.attributes.get(&attr_mass).unwrap().value.unwrap();
    let align_time = -(0.25 as f64).ln() * agility * mass / 1000000.0;

    ship.add_attribute(AttributeId::alignTime, base_align_time, align_time);
}
