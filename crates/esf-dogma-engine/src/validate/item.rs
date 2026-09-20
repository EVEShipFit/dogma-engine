use esf_data::Info;

use super::{Context, GroupLimit, Item, Rule, SlotKind, Violation};
use crate::fit::State;

/// The attributes naming the ship types an item may go on.
const SHIP_TYPES: [&str; 13] = [
    "fitsToShipType",
    "canFitShipType1",
    "canFitShipType2",
    "canFitShipType3",
    "canFitShipType4",
    "canFitShipType5",
    "canFitShipType6",
    "canFitShipType7",
    "canFitShipType8",
    "canFitShipType9",
    "canFitShipType10",
    "canFitShipType11",
    "canFitShipType12",
];

/// The attributes naming the ship groups an item may go on.
const SHIP_GROUPS: [&str; 20] = [
    "canFitShipGroup01",
    "canFitShipGroup02",
    "canFitShipGroup03",
    "canFitShipGroup04",
    "canFitShipGroup05",
    "canFitShipGroup06",
    "canFitShipGroup07",
    "canFitShipGroup08",
    "canFitShipGroup09",
    "canFitShipGroup10",
    "canFitShipGroup11",
    "canFitShipGroup12",
    "canFitShipGroup13",
    "canFitShipGroup14",
    "canFitShipGroup15",
    "canFitShipGroup16",
    "canFitShipGroup17",
    "canFitShipGroup18",
    "canFitShipGroup19",
    "canFitShipGroup20",
];

/// A group limit, and the state it counts.
const GROUP_LIMITS: [(GroupLimit, &str); 3] = [
    (GroupLimit::Fitted, "maxGroupFitted"),
    (GroupLimit::Online, "maxGroupOnline"),
    (GroupLimit::Active, "maxGroupActive"),
];

/// Anything bigger than this is a capital module, and only a capital ship
/// takes one.
const CAPITAL_VOLUME: f64 = 3500.0;
const STRUCTURE_CATEGORY_ID: i32 = 65;

pub(super) fn validate<I: Info>(context: &Context<'_, I>, found: &mut Vec<Violation>) {
    rig_size(context, found);
    ship_restricted(context, found);
    capital_item(context, found);
    group_limits(context, found);
    type_limit(context, found);
}

fn rig_size<I: Info>(context: &Context<'_, I>, found: &mut Vec<Violation>) {
    let Some(attribute_id) = context.attribute_id("rigSize") else {
        return;
    };
    let Some(ship) = context.value(context.ship(), attribute_id) else {
        return;
    };

    for item in &context.items {
        let Some(size) = context.value(item.result, attribute_id) else {
            continue;
        };
        if item.rack() == Some(SlotKind::Rig) && size != ship {
            found.push(item.violation(Rule::RigSize {
                ship: ship as u8,
                item: size as u8,
            }));
        }
    }
}

fn ship_restricted<I: Info>(context: &Context<'_, I>, found: &mut Vec<Violation>) {
    let type_limits: Vec<i32> = SHIP_TYPES
        .iter()
        .filter_map(|name| context.attribute_id(name))
        .collect();
    let group_limits: Vec<i32> = SHIP_GROUPS
        .iter()
        .filter_map(|name| context.attribute_id(name))
        .collect();

    let ship_type_id = context.fit.ship.type_id;
    let ship_group_id = context.ship_group_id();

    for item in &context.items {
        if !item.is_fitted() {
            continue;
        }

        let types = listed(context, item, &type_limits);
        let groups = listed(context, item, &group_limits);
        if types.is_empty() && groups.is_empty() {
            continue;
        }

        if !types.contains(&ship_type_id) && !groups.contains(&ship_group_id) {
            found.push(item.violation(Rule::ShipRestricted));
        }
    }
}

/// What the item lists under a set of attributes; empty when it lists nothing,
/// which is what an item that goes on any ship looks like.
fn listed<I: Info>(context: &Context<'_, I>, item: &Item<'_>, attribute_ids: &[i32]) -> Vec<i32> {
    attribute_ids
        .iter()
        .filter_map(|attribute_id| context.value(item.result, *attribute_id))
        .map(|value| value as i32)
        .collect()
}

fn capital_item<I: Info>(context: &Context<'_, I>, found: &mut Vec<Violation>) {
    let (Some(volume), Some(capital)) = (
        context.attribute_id("volume"),
        context.attribute_id("isCapitalSize"),
    ) else {
        return;
    };

    if context.value(context.ship(), capital).unwrap_or(0.0) != 0.0 {
        return;
    }
    if context.ship_category_id() == STRUCTURE_CATEGORY_ID {
        return;
    }

    for item in &context.items {
        /* A capital rig is held apart by its rig size instead. */
        if !item.is_fitted() || item.rack() == Some(SlotKind::Rig) {
            continue;
        }
        if context.amount(item.result, volume) > CAPITAL_VOLUME {
            found.push(item.violation(Rule::CapitalItem));
        }
    }
}

fn group_limits<I: Info>(context: &Context<'_, I>, found: &mut Vec<Violation>) {
    for (limit, attribute_name) in GROUP_LIMITS {
        let Some(attribute_id) = context.attribute_id(attribute_name) else {
            continue;
        };

        for item in &context.items {
            let Some(allowed) = context.value(item.result, attribute_id) else {
                continue;
            };
            if !item.is_fitted() {
                continue;
            }

            let used = context
                .items
                .iter()
                .filter(|other| {
                    other.is_fitted() && other.group_id == item.group_id && counts(other, limit)
                })
                .count() as u32;

            let allowed = allowed as u32;
            if used > allowed {
                found.push(item.violation(Rule::MaxGroup {
                    group_id: item.group_id,
                    limit,
                    used,
                    allowed,
                }));
            }
        }
    }
}

fn type_limit<I: Info>(context: &Context<'_, I>, found: &mut Vec<Violation>) {
    let Some(attribute_id) = context.attribute_id("maxTypeFitted") else {
        return;
    };

    for item in &context.items {
        let Some(allowed) = context.value(item.result, attribute_id) else {
            continue;
        };
        if !item.is_fitted() {
            continue;
        }

        let used = context
            .items
            .iter()
            .filter(|other| other.is_fitted() && other.fit.type_id == item.fit.type_id)
            .count() as u32;

        let allowed = allowed as u32;
        if used > allowed {
            found.push(item.violation(Rule::MaxType {
                type_id: item.fit.type_id,
                used,
                allowed,
            }));
        }
    }
}

/// Whether an item counts towards a limit that only looks at one state.
fn counts(item: &Item<'_>, limit: GroupLimit) -> bool {
    match limit {
        GroupLimit::Fitted => true,
        GroupLimit::Online => item.result.state >= State::Online,
        GroupLimit::Active => item.result.state >= State::Active,
    }
}
