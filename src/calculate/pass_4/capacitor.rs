use crate::info::Info;

use super::super::Ship;

struct Module {
    capacitor_need: f64,
    duration: f64,
    time_next: f64,
}

pub fn attribute_capacitor_depletes_in(info: &impl Info, ship: &mut Ship) {
    /* Amount of seconds it takes for the capacitor to deplete; or negative if it is stable. */

    let attr_capacitor_peak_delta_id = info.attribute_name_to_id("capacitorPeakDelta");
    let attr_capacitor_capacity_id = info.attribute_name_to_id("capacitorCapacity");
    let attr_recharge_rate_id = info.attribute_name_to_id("rechargeRate");
    let attr_capacitor_need_id = info.attribute_name_to_id("capacitorNeed");
    let attr_cycle_time_id = info.attribute_name_to_id("cycleTime");
    let attr_capacitor_depletes_in_id = info.attribute_name_to_id("capacitorDepletesIn");

    if !ship
        .hull
        .attributes
        .contains_key(&attr_capacitor_peak_delta_id)
    {
        return;
    }

    let mut depletes_in = -1000.0;

    let attr_capacitor_peak_delta = ship
        .hull
        .attributes
        .get(&attr_capacitor_peak_delta_id)
        .unwrap();

    if attr_capacitor_peak_delta.value.unwrap() < 0.0 {
        let attr_capacitor_capacity = ship
            .hull
            .attributes
            .get(&attr_capacitor_capacity_id)
            .unwrap();
        let attr_recharge_rate = ship.hull.attributes.get(&attr_recharge_rate_id).unwrap();

        /* Find all modules consuming capacitor. */
        let mut modules = Vec::new();
        for item in &ship.items {
            if !item.slot.is_module() || !item.state.is_active() {
                continue;
            }

            if !item.attributes.contains_key(&attr_capacitor_need_id)
                || !item.attributes.contains_key(&attr_cycle_time_id)
            {
                continue;
            }

            let duration = item
                .attributes
                .get(&attr_cycle_time_id)
                .unwrap()
                .value
                .unwrap();

            let capacitor_need = item
                .attributes
                .get(&attr_capacitor_need_id)
                .unwrap()
                .value
                .unwrap();

            modules.push(Module {
                capacitor_need,
                duration: duration,
                time_next: 0.0,
            });
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
        .add_attribute(attr_capacitor_depletes_in_id, 0.0, depletes_in / 1000.0);
}
