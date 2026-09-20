use esf_data::Info;

use super::{Context, Resource, Rule, Target, Violation};
use crate::fit::{Slot, State};

/// The resources the dogma adds up itself: the attribute holding what is
/// used, against the one holding how much there is.
const LOADS: [(Resource, &str, &str); 10] = [
    (Resource::Cpu, "cpuLoad", "cpuOutput"),
    (Resource::Powergrid, "powerLoad", "powerOutput"),
    (Resource::Calibration, "upgradeLoad", "upgradeCapacity"),
    (Resource::DroneBay, "droneCapacityLoad", "droneCapacity"),
    (
        Resource::DroneBandwidth,
        "droneBandwidthLoad",
        "droneBandwidth",
    ),
    (
        Resource::FighterBay,
        "fighterCapacityLoad",
        "fighterCapacity",
    ),
    (Resource::FighterTubes, "fighterTubesUsed", "fighterTubes"),
    (
        Resource::LightFighterTubes,
        "fighterLightSlotsUsed",
        "fighterLightSlots",
    ),
    (
        Resource::SupportFighterTubes,
        "fighterSupportSlotsUsed",
        "fighterSupportSlots",
    ),
    (
        Resource::HeavyFighterTubes,
        "fighterHeavySlotsUsed",
        "fighterHeavySlots",
    ),
];

pub(super) fn validate<I: Info>(context: &Context<'_, I>, found: &mut Vec<Violation>) {
    for (resource, load_name, output_name) in LOADS {
        let (Some(load), Some(output)) = (
            context.attribute_id(load_name),
            context.attribute_id(output_name),
        ) else {
            continue;
        };

        let used = context.amount(context.ship(), load);
        let available = context.amount(context.ship(), output);
        if over(used, available) {
            found.push(resource_violation(Target::Ship, resource, used, available));
        }
    }

    launched_drones(context, found);
    cargo(context, found);
}

pub(super) fn resource_violation(
    target: Target,
    resource: Resource,
    used: f64,
    available: f64,
) -> Violation {
    Violation {
        target,
        rule: Rule::Resource {
            resource,
            used,
            available,
        },
    }
}

/// A chain of modifiers can land just off from the limit, but is still
/// a valid fit.
pub(super) fn over(used: f64, available: f64) -> bool {
    used > available + available.abs() * 1e-9
}

/// How many drones are in space, rather than how much room they take up.
fn launched_drones<I: Info>(context: &Context<'_, I>, found: &mut Vec<Violation>) {
    let Some(max_active) = context.attribute_id("maxActiveDrones") else {
        return;
    };

    let used = context
        .items
        .iter()
        .filter(|item| item.fit.slot == Slot::DroneBay && item.result.state >= State::Active)
        .map(|item| item.fit.quantity)
        .sum::<u32>();
    let available = context.amount(&context.calculation.character, max_active);

    if over(f64::from(used), available) {
        found.push(resource_violation(
            Target::Ship,
            Resource::LaunchedDrones,
            f64::from(used),
            available,
        ));
    }
}

/// Cargo is not calculated, so what it takes up comes from the type itself.
fn cargo<I: Info>(context: &Context<'_, I>, found: &mut Vec<Violation>) {
    let Some(capacity) = context.attribute_id("capacity") else {
        return;
    };

    let used = context
        .items
        .iter()
        .filter(|item| item.fit.slot == Slot::Cargo)
        .map(|item| {
            let volume = context
                .info
                .get_type(item.fit.type_id)
                .and_then(|r#type| r#type.volume())
                .unwrap_or(0.0);
            f64::from(volume) * f64::from(item.fit.quantity)
        })
        .sum::<f64>();
    let available = context.amount(context.ship(), capacity);

    if over(used, available) {
        found.push(resource_violation(
            Target::Ship,
            Resource::CargoBay,
            used,
            available,
        ));
    }
}
