use super::super::Ship;
use super::AttributeId;

struct Module {
    capacitor_need: f64,
    duration: f64,
    time_next: f64,
}

pub fn attribute_capacitor_depletes_in(ship: &mut Ship) {
    /* Amount of seconds it takes for the capacitor to deplete; or negative if it is stable. */

    if !ship
        .hull
        .attributes
        .contains_key(&(AttributeId::capacitorDepletesIn as i32))
    {
        return;
    }

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
        let attr_rate_of_fire = AttributeId::speed as i32;

        /* Find all modules consuming capacitor. */
        let mut modules = Vec::new();
        for item in &ship.items {
            if !item.slot.is_module() || !item.state.is_active() {
                continue;
            }

            if !item.attributes.contains_key(&attr_capacitor_need) {
                continue;
            }

            /* Depending on the module, the duration is based either on "duration" or on "speed" (read: rate-of-fire). */
            let duration = if item.attributes.contains_key(&attr_duration) {
                item.attributes.get(&attr_duration).unwrap().value.unwrap()
            } else if item.attributes.contains_key(&attr_rate_of_fire) {
                item.attributes
                    .get(&attr_rate_of_fire)
                    .unwrap()
                    .value
                    .unwrap()
            } else {
                /* Neither speed nor duration; so no cap use. */
                continue;
            };

            let capacitor_need = item
                .attributes
                .get(&attr_capacitor_need)
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
        .add_attribute(AttributeId::capacitorDepletesIn, 0.0, depletes_in / 1000.0);
}
