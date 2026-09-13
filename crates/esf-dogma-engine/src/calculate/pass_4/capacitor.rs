use esf_data::Info;

use super::super::Objects;

struct Module {
    capacitor_need: f64,
    duration: f64,
    time_next: f64,
}

pub fn attribute_capacitor_depletes_in(info: &impl Info, objects: &mut Objects) {
    /* Amount of seconds it takes for the capacitor to deplete; or negative if it is stable. */

    let (
        Some(attr_capacitor_peak_delta_id),
        Some(attr_capacitor_capacity_id),
        Some(attr_recharge_rate_id),
        Some(attr_capacitor_peak_load_id),
        Some(attr_cycle_time_id),
        Some(attr_capacitor_depletes_in_id),
    ) = (
        info.attribute_name_to_id("capacitorPeakDelta"),
        info.attribute_name_to_id("capacitorCapacity"),
        info.attribute_name_to_id("rechargeRate"),
        info.attribute_name_to_id("capacitorPeakLoad"),
        info.attribute_name_to_id("cycleTime"),
        info.attribute_name_to_id("capacitorDepletesIn"),
    )
    else {
        return;
    };

    if !objects
        .ship
        .attributes
        .contains_key(&attr_capacitor_peak_delta_id)
    {
        return;
    }

    let mut depletes_in = -1000.0;

    let attr_capacitor_peak_delta = objects
        .ship
        .attributes
        .get(&attr_capacitor_peak_delta_id)
        .unwrap();

    if attr_capacitor_peak_delta.value.get().unwrap() < 0.0 {
        let attr_capacitor_capacity = objects
            .ship
            .attributes
            .get(&attr_capacitor_capacity_id)
            .unwrap();
        let attr_recharge_rate = objects.ship.attributes.get(&attr_recharge_rate_id).unwrap();

        /* Find all modules consuming or bringing in capacitor. */
        let mut modules = Vec::new();
        for item in &objects.items {
            if !item.is_module() || !item.state.is_active() {
                continue;
            }

            if !item.attributes.contains_key(&attr_capacitor_peak_load_id)
                || !item.attributes.contains_key(&attr_cycle_time_id)
            {
                continue;
            }

            let duration = item
                .attributes
                .get(&attr_cycle_time_id)
                .unwrap()
                .value
                .get()
                .unwrap();

            /* Unlike capacitorNeed, peak load is net of capacitor a nosferatu brings in. */
            let capacitor_peak_load = item
                .attributes
                .get(&attr_capacitor_peak_load_id)
                .unwrap()
                .value
                .get()
                .unwrap();

            modules.push(Module {
                capacitor_need: capacitor_peak_load * duration / 1000.0 * item.quantity as f64,
                duration,
                time_next: 0.0,
            });
        }

        if !modules.is_empty() {
            let capacitor_capacity = attr_capacitor_capacity.value.get().unwrap();
            let recharge_rate = attr_recharge_rate.value.get().unwrap();

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

                /* Clamped after the whole step, so module order does not matter. */
                capacitor = f64::min(capacitor, capacitor_capacity);
            }

            depletes_in = time_last;
        }
    }

    objects
        .ship
        .add_attribute(attr_capacitor_depletes_in_id, 0.0, depletes_in / 1000.0);
}
