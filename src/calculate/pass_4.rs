use super::item::Attribute;
use super::{Info, Pass, Ship};

pub struct PassFour {}

mod align_time;
mod cpu_power;
mod ehp;
mod scan_strength;

#[allow(non_camel_case_types)]
pub enum AttributeId {
    alignTime = -1,
    scanStrength = -2,
    cpuUsed = -3,
    powerUsed = -4,
    cpuUnused = -5,
    powerUnused = -6,
    #[allow(dead_code)]
    velocityBoost = -7, // Is taken care of with effects.
    shieldEhpMultiplier = -8,
    armorEhpMultiplier = -9,
    hullEhpMultiplier = -10,
    shieldEhp = -11,
    armorEhp = -12,
    hullEhp = -13,
    ehp = -14,

    mass = 4,
    hp = 9,
    powerOutput = 11,
    power = 30,
    cpuOutput = 48,
    cpu = 50,
    agility = 70,

    kineticDamageResonance = 109,
    thermalDamageResonance = 110,
    explosiveDamageResonance = 111,
    emDamageResonance = 113,

    scanRadarStrength = 208,
    scanLadarStrength = 209,
    scanMagnetometricStrength = 210,
    scanGravimetricStrength = 211,

    shieldCapacity = 263,
    armorHp = 265,

    armorEmDamageResonance = 267,
    armorExplosiveDamageResonance = 268,
    armorKineticDamageResonance = 269,
    armorThermalDamageResonance = 270,
    shieldEmDamageResonance = 271,
    shieldExplosiveDamageResonance = 272,
    shieldKineticDamageResonance = 273,
    shieldThermalDamageResonance = 274,
}

impl Ship {
    pub fn add_attribute(&mut self, attribute_id: AttributeId, base_value: f64, value: f64) {
        let mut attribute = Attribute::new(base_value);
        attribute.value = Some(value);
        self.hull.attributes.insert(attribute_id as i32, attribute);
    }
}

/* Attributes don't contain all information displayed, so we calculate some fake attributes with those values. */
impl Pass for PassFour {
    fn pass(_info: &Info, ship: &mut Ship) {
        align_time::attribute_align_time(ship);
        scan_strength::attribute_scan_strength(ship);

        cpu_power::attribute_cpu_used(ship);
        cpu_power::attribute_power_used(ship);
        cpu_power::attribute_cpu_unused(ship);
        cpu_power::attribute_power_unused(ship);

        ehp::attribute_shield_ehp_multiplier(ship);
        ehp::attribute_armor_ehp_multiplier(ship);
        ehp::attribute_hull_ehp_multiplier(ship);
        ehp::attribute_shield_ehp(ship);
        ehp::attribute_armor_ehp(ship);
        ehp::attribute_hull_ehp(ship);
        ehp::attribute_ehp(ship);
    }
}
