use super::super::Ship;
use super::AttributeId;

const SCAN_STRENGTH_ATTRIBUTES: [i32; 4] = [
    AttributeId::scanRadarStrength as i32,
    AttributeId::scanLadarStrength as i32,
    AttributeId::scanMagnetometricStrength as i32,
    AttributeId::scanGravimetricStrength as i32,
];

pub fn attribute_scan_strength(ship: &mut Ship) {
    /* Scan Strength can be one of 4 scan strength values (radar, ladar, magnetometric, gravimetric). */

    let mut base_scan_strength = 0.0;
    let mut scan_strength = 0.0;
    for attribute_id in SCAN_STRENGTH_ATTRIBUTES.iter() {
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

    ship.add_attribute(AttributeId::scanStrength, base_scan_strength, scan_strength);
}
