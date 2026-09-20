use esf_data::Info;

use super::resource::{over, resource_violation};
use super::{Context, Resource, Rule, Target, Violation};

/// The attributes naming the charge groups a module takes.
const CHARGE_GROUPS: [&str; 5] = [
    "chargeGroup1",
    "chargeGroup2",
    "chargeGroup3",
    "chargeGroup4",
    "chargeGroup5",
];

pub(super) fn validate<I: Info>(context: &Context<'_, I>, found: &mut Vec<Violation>) {
    let groups: Vec<i32> = CHARGE_GROUPS
        .iter()
        .filter_map(|name| context.attribute_id(name))
        .collect();

    for item in &context.items {
        let (Some(loaded), Some(charge)) = (&item.fit.charge, &item.result.charge) else {
            continue;
        };
        let target = Target::Charge { index: item.index };

        let takes: Vec<i32> = groups
            .iter()
            .filter_map(|attribute_id| context.value(item.result, *attribute_id))
            .map(|value| value as i32)
            .collect();
        let group_id = context
            .info
            .get_type(loaded.type_id)
            .map_or(0, |r#type| r#type.group_id());
        if !takes.is_empty() && !takes.contains(&group_id) {
            found.push(Violation {
                target,
                rule: Rule::ChargeGroup,
            });
        }

        /* Only some modules sort their charges by size; a capacitor booster
         * goes by volume alone, and its charges do carry a size. */
        if let Some(attribute_id) = context.attribute_id("chargeSize")
            && let (Some(module), Some(size)) = (
                context.value(item.result, attribute_id),
                context.value(charge, attribute_id),
            )
            && module != size
        {
            found.push(Violation {
                target,
                rule: Rule::ChargeSize {
                    module: module as u8,
                    charge: size as u8,
                },
            });
        }

        /* A module the SDE gives no room at all, like a civilian gun, still
         * takes a charge in game; there is nothing to hold it against. */
        if let (Some(capacity), Some(volume)) = (
            context.attribute_id("capacity"),
            context.attribute_id("volume"),
        ) && let Some(available) = context.value(item.result, capacity)
        {
            let used = context.amount(charge, volume);
            if over(used, available) {
                found.push(resource_violation(
                    target,
                    Resource::ChargeCapacity,
                    used,
                    available,
                ));
            }
        }
    }
}
