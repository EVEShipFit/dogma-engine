use super::super::item::{EffectCategory, Item};
use super::super::Ship;
use super::AttributeId;

fn calculate_alpha(
    item: &Item,
    charge: &Item,
    base_alpha_hp: &mut f64,
    alpha_hp: &mut f64,
) -> bool {
    let attr_damage_em = AttributeId::emDamage as i32;
    let attr_damage_explosive = AttributeId::explosiveDamage as i32;
    let attr_damage_kinetic = AttributeId::kineticDamage as i32;
    let attr_damage_thermal = AttributeId::thermalDamage as i32;
    let attr_damage_multiplier = AttributeId::damageMultiplier as i32;

    /* Only calculate for charges that do damage. */
    if !charge.attributes.contains_key(&attr_damage_em)
        || !charge.attributes.contains_key(&attr_damage_explosive)
        || !charge.attributes.contains_key(&attr_damage_kinetic)
        || !charge.attributes.contains_key(&attr_damage_thermal)
    {
        return false;
    }

    let base_damage_em = charge.attributes.get(&attr_damage_em).unwrap().base_value;
    let base_damage_explosive = charge
        .attributes
        .get(&attr_damage_explosive)
        .unwrap()
        .base_value;
    let base_damage_kinetic = charge
        .attributes
        .get(&attr_damage_kinetic)
        .unwrap()
        .base_value;
    let base_damage_thermal = charge
        .attributes
        .get(&attr_damage_thermal)
        .unwrap()
        .base_value;

    let damage_em = charge
        .attributes
        .get(&attr_damage_em)
        .unwrap()
        .value
        .unwrap();
    let damage_explosive = charge
        .attributes
        .get(&attr_damage_explosive)
        .unwrap()
        .value
        .unwrap();
    let damage_kinetic = charge
        .attributes
        .get(&attr_damage_kinetic)
        .unwrap()
        .value
        .unwrap();
    let damage_thermal = charge
        .attributes
        .get(&attr_damage_thermal)
        .unwrap()
        .value
        .unwrap();

    *base_alpha_hp +=
        base_damage_em + base_damage_explosive + base_damage_kinetic + base_damage_thermal;
    *alpha_hp += damage_em + damage_explosive + damage_kinetic + damage_thermal;

    if item.attributes.contains_key(&attr_damage_multiplier) {
        let damage_multiplier = item.attributes.get(&attr_damage_multiplier).unwrap();

        *base_alpha_hp *= damage_multiplier.base_value;
        *alpha_hp *= damage_multiplier.value.unwrap();
    }

    true
}

pub fn attribute_damage_alpha_hp(ship: &mut Ship) {
    /* Damage when all guns / drones fire at once, in hp. */

    let mut total_base_alpha_hp = 0.0;
    let mut total_alpha_hp = 0.0;

    for item in &mut ship.items {
        if item.state == EffectCategory::Passive {
            continue;
        }

        let mut base_alpha_hp = 0.0;
        let mut alpha_hp = 0.0;

        /* Check for drone damage. */
        calculate_alpha(item, item, &mut base_alpha_hp, &mut alpha_hp);
        /* Check for charge damage. */
        if let Some(charge) = item.charge.as_ref() {
            calculate_alpha(item, charge, &mut base_alpha_hp, &mut alpha_hp);
        }

        if base_alpha_hp > 0.0 || alpha_hp > 0.0 {
            item.add_attribute(AttributeId::damageAlphaHp, base_alpha_hp, alpha_hp);
        }

        total_base_alpha_hp += base_alpha_hp;
        total_alpha_hp += alpha_hp;
    }

    ship.hull.add_attribute(
        AttributeId::damageAlphaHp,
        total_base_alpha_hp,
        total_alpha_hp,
    );
}

pub fn attribute_damage_without_reload(ship: &mut Ship) {
    /* Damage (per second) all guns do without reloading. */

    let attr_rate_of_fire = AttributeId::speed as i32;
    let attr_damage_alpha_hp = AttributeId::damageAlphaHp as i32;

    let mut total_base_damage = 0.0;
    let mut total_damage = 0.0;

    for item in &mut ship.items {
        if item.state == EffectCategory::Passive {
            continue;
        }

        if item.attributes.contains_key(&attr_rate_of_fire)
            && item.attributes.contains_key(&attr_damage_alpha_hp)
        {
            let rate_of_fire = item.attributes.get(&attr_rate_of_fire).unwrap();
            let damage_alpha_hp = item.attributes.get(&attr_damage_alpha_hp).unwrap();

            let base_damage_dps = damage_alpha_hp.base_value / (rate_of_fire.base_value / 1000.0);
            let damage_dps =
                damage_alpha_hp.value.unwrap() / (rate_of_fire.value.unwrap() / 1000.0);

            item.add_attribute(
                AttributeId::damageWithoutReloadDps,
                base_damage_dps,
                damage_dps,
            );

            total_base_damage += base_damage_dps;
            total_damage += damage_dps;
        }
    }

    ship.hull.add_attribute(
        AttributeId::damageWithoutReloadDps,
        total_base_damage,
        total_damage,
    );
}

pub fn attribute_damage_with_reload(ship: &mut Ship) {
    /* Damage (per second) all guns do with reloading. */

    let attr_rate_of_fire = AttributeId::speed as i32;
    let attr_damage_alpha_hp = AttributeId::damageAlphaHp as i32;
    let attr_reload_time = AttributeId::reloadTime as i32;
    let attr_volume = AttributeId::volume as i32;
    let attr_capacity = AttributeId::capacity as i32;

    let mut total_base_damage = 0.0;
    let mut total_damage = 0.0;

    for item in &mut ship.items {
        if item.state == EffectCategory::Passive {
            continue;
        }

        if let Some(charge) = item.charge.as_ref() {
            if item.attributes.contains_key(&attr_rate_of_fire)
                && item.attributes.contains_key(&attr_damage_alpha_hp)
            {
                /* Most stats come from the module itself. */
                let rate_of_fire = item.attributes.get(&attr_rate_of_fire).unwrap();
                let damage_alpha_hp = item.attributes.get(&attr_damage_alpha_hp).unwrap();
                let reload_time = item.attributes.get(&attr_reload_time).unwrap();
                let capacity = item.attributes.get(&attr_capacity).unwrap();

                /* Volume of the charge. */
                let volume = charge.attributes.get(&attr_volume).unwrap();

                let charge_amount = (capacity.value.unwrap() / volume.value.unwrap()).floor();
                let reload_rof = reload_time.value.unwrap() / 1000.0 / charge_amount;

                let base_damage_dps =
                    damage_alpha_hp.base_value / ((rate_of_fire.base_value / 1000.0) + reload_rof);
                let damage_dps = damage_alpha_hp.value.unwrap()
                    / ((rate_of_fire.value.unwrap() / 1000.0) + reload_rof);

                item.add_attribute(
                    AttributeId::damageWithReloadDps,
                    base_damage_dps,
                    damage_dps,
                );

                total_base_damage += base_damage_dps;
                total_damage += damage_dps;
            }
        }
    }

    ship.hull.add_attribute(
        AttributeId::damageWithReloadDps,
        total_base_damage,
        total_damage,
    );
}
