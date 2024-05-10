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

    ship.hull.add_attribute(
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
            && item.attributes.contains_key(&attr_duration)
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

    ship.hull
        .add_attribute(AttributeId::capacitorPeakUsage, base_peak_usage, peak_usage);
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

    ship.hull
        .add_attribute(AttributeId::capacitorPeakDelta, base_peak_delta, peak_delta);
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

    ship.hull.add_attribute(
        AttributeId::capacitorPeakDeltaPercentage,
        base_peak_delta_percentage,
        peak_delta_percentage,
    );
}

struct Module {
    capacitor_need: f64,
    duration: f64,
    time_next: f64,
}

pub fn attribute_capacitor_depletes_in(ship: &mut Ship) {
    /* Amount of seconds it takes for the capacitor to deplete; or negative if it is stable. */

    let mut depletes_in = -1.0;

    let attr_capacitor_peak_delta = ship
        .hull
        .attributes
        .get(&(AttributeId::capacitorPeakDelta as i32))
        .unwrap();

    if attr_capacitor_peak_delta.value.unwrap() < 0.0 {
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

        let attr_capacitor_need = AttributeId::capacitorNeed as i32;
        let attr_duration = AttributeId::duration as i32;

        /* Find all modules consuming capacitor. */
        let mut modules = Vec::new();
        for item in &ship.items {
            if item.attributes.contains_key(&attr_capacitor_need)
                && item.attributes.contains_key(&attr_duration)
                && item.state != EffectCategory::Passive
            {
                let capacitor_need = item
                    .attributes
                    .get(&attr_capacitor_need)
                    .unwrap()
                    .value
                    .unwrap();
                let duration = item.attributes.get(&attr_duration).unwrap().value.unwrap();

                modules.push(Module {
                    capacitor_need,
                    duration: duration,
                    time_next: 0.0,
                });
            }
        }

        if modules.len() > 0 {
            let capacitor_capacity = attr_capacitor_capacity.value.unwrap();
            let recharge_rate = attr_recharge_rate.value.unwrap();

            let mut capacitor = capacitor_capacity;
            let mut time_last = 0.0;
            let mut time_next = 0.0;

            /* Simulate the capacitor to find out when it depletes. */
            while capacitor > 0.0 {
                capacitor = (1.0
                    + (f64::sqrt(capacitor / capacitor_capacity) - 1.0)
                        * f64::exp(5.0 * (time_last - time_next) / recharge_rate))
                .powi(2)
                    * capacitor_capacity;

                time_last = time_next;
                time_next = f64::INFINITY;

                for module in &mut modules {
                    if module.time_next <= time_last {
                        module.time_next += module.duration;
                        capacitor -= module.capacitor_need;
                    }

                    /* Find the next module that would use capacitor. */
                    time_next = f64::min(time_next, module.time_next);
                }
            }

            depletes_in = time_last;
        }
    }

    ship.hull
        .add_attribute(AttributeId::capacitorDepletesIn, 0.0, depletes_in / 1000.0);
}
