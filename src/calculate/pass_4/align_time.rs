use super::super::Ship;
use super::AttributeId;

pub fn attribute_align_time(ship: &mut Ship) {
    /* Align-time is based on agility and mass. */

    /* Structures do not have agility, and as such, no align-time. */
    if !ship
        .hull
        .attributes
        .contains_key(&(AttributeId::agility as i32))
    {
        return;
    }

    let attr_agility = ship
        .hull
        .attributes
        .get(&(AttributeId::agility as i32))
        .unwrap();
    let attr_mass = ship
        .hull
        .attributes
        .get(&(AttributeId::mass as i32))
        .unwrap();

    let base_agility = attr_agility.base_value;
    let base_mass = attr_mass.base_value;
    let base_align_time = -(0.25 as f64).ln() * base_agility * base_mass / 1000000.0;

    let agility = attr_agility.value.unwrap();
    let mass = attr_mass.value.unwrap();
    let align_time = -(0.25 as f64).ln() * agility * mass / 1000000.0;

    ship.hull
        .add_attribute(AttributeId::alignTime, base_align_time, align_time);
}
