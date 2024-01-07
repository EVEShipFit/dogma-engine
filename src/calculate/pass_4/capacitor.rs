use super::super::item::EffectCategory;
use super::super::Ship;
use super::AttributeId;

pub fn attribute_capacitor_peak_recharge(ship: &mut Ship) {
    /* The peak recharge rate of the capacitor (in GJ/s). This happens at 25% of the capacity. */

    let attr_capacitor_capacity = ship
        .hull
        .attributes
        .get(&(AttributeId::capacitorCapacity as i32))
        .unwrap();
    let attr_recharge_rate = ship
        .hull
        .attributes
        .get(&(AttributeId::rechargeRate as i32))
        .unwrap();

    let base_capacitor_capacity = attr_capacitor_capacity.base_value;
    let base_recharge_rate = attr_recharge_rate.base_value / 1000.0;
    let base_peak_recharge = 5.0 / 2.0 * base_capacitor_capacity / base_recharge_rate;

    let capacitor_capacity = attr_capacitor_capacity.value.unwrap();
    let recharge_rate = attr_recharge_rate.value.unwrap() / 1000.0;
    let peak_recharge = 5.0 / 2.0 * capacitor_capacity / recharge_rate;

    ship.add_attribute(
        AttributeId::capacitorPeakRecharge,
        base_peak_recharge,
        peak_recharge,
    );
}

pub fn attribute_capacitor_peak_usage(ship: &mut Ship) {
    /* The peak capacitor usage if all modules would activate at the same time, in GJ/s. */

    let attr_capacitor_need = AttributeId::capacitorNeed as i32;
    let attr_duration = AttributeId::duration as i32;

    let mut base_peak_usage = 0.0;
    let mut peak_usage = 0.0;
    for item in &ship.items {
        if item.attributes.contains_key(&attr_capacitor_need)
            && item.state != EffectCategory::Passive
        {
            base_peak_usage += item
                .attributes
                .get(&attr_capacitor_need)
                .unwrap()
                .base_value
                / item.attributes.get(&attr_duration).unwrap().base_value
                * 1000.0;
            peak_usage += item
                .attributes
                .get(&attr_capacitor_need)
                .unwrap()
                .value
                .unwrap()
                / item.attributes.get(&attr_duration).unwrap().value.unwrap()
                * 1000.0;
        }
    }

    ship.add_attribute(AttributeId::capacitorPeakUsage, base_peak_usage, peak_usage);
}

pub fn attribute_capacitor_peak_delta(ship: &mut Ship) {
    /* The delta between peak recharge and peak usage, in GJ/s. */

    let attr_capacitor_peak_recharge = ship
        .hull
        .attributes
        .get(&(AttributeId::capacitorPeakRecharge as i32))
        .unwrap();
    let attr_capacitor_peak_usage = ship
        .hull
        .attributes
        .get(&(AttributeId::capacitorPeakUsage as i32))
        .unwrap();

    let base_peak_recharge = attr_capacitor_peak_recharge.base_value;
    let base_peak_usage = attr_capacitor_peak_usage.base_value;
    let base_peak_delta = base_peak_recharge - base_peak_usage;

    let peak_recharge = attr_capacitor_peak_recharge.value.unwrap();
    let peak_usage = attr_capacitor_peak_usage.value.unwrap();
    let peak_delta = peak_recharge - peak_usage;

    ship.add_attribute(AttributeId::capacitorPeakDelta, base_peak_delta, peak_delta);
}

pub fn attribute_capacitor_peak_delta_percentage(ship: &mut Ship) {
    /* The delta between peak recharge and peak usage, in % of peak recharge. */

    let attr_capacitor_peak_recharge = ship
        .hull
        .attributes
        .get(&(AttributeId::capacitorPeakRecharge as i32))
        .unwrap();
    let attr_capacitor_peak_usage = ship
        .hull
        .attributes
        .get(&(AttributeId::capacitorPeakUsage as i32))
        .unwrap();

    let base_peak_recharge = attr_capacitor_peak_recharge.base_value;
    let base_peak_usage = attr_capacitor_peak_usage.base_value;
    let base_peak_delta = base_peak_recharge - base_peak_usage;
    let base_peak_delta_percentage = base_peak_delta / base_peak_recharge * 100.0;

    let peak_recharge = attr_capacitor_peak_recharge.value.unwrap();
    let peak_usage = attr_capacitor_peak_usage.value.unwrap();
    let peak_delta = peak_recharge - peak_usage;
    let peak_delta_percentage = peak_delta / peak_recharge * 100.0;

    ship.add_attribute(
        AttributeId::capacitorPeakDeltaPercentage,
        base_peak_delta_percentage,
        peak_delta_percentage,
    );
}
