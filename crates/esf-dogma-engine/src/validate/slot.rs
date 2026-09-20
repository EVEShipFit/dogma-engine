use esf_data::Info;

use super::{Context, Item, Rule, SlotKind, Target, Violation};
use crate::fit::Slot;

/// A rack, and the ship attribute saying how many slots it has. A ship
/// without the attribute has no such rack at all.
const RACKS: [(SlotKind, &str); 6] = [
    (SlotKind::High, "hiSlots"),
    (SlotKind::Medium, "medSlots"),
    (SlotKind::Low, "lowSlots"),
    (SlotKind::Rig, "rigSlots"),
    (SlotKind::Subsystem, "maxSubSystems"),
    (SlotKind::Service, "serviceSlots"),
];

/// A hardpoint, and the ship attribute saying how many there are. A weapon
/// takes one on top of the slot it sits in.
const HARDPOINTS: [(SlotKind, &str); 2] = [
    (SlotKind::Turret, "turretSlotsLeft"),
    (SlotKind::Launcher, "launcherSlotsLeft"),
];

pub(super) fn validate<I: Info>(context: &Context<'_, I>, found: &mut Vec<Violation>) {
    for (slot, attribute_name) in RACKS {
        let used = context
            .items
            .iter()
            .filter(|item| item.rack() == Some(slot))
            .count();
        report(context, found, slot, attribute_name, used);
    }

    for (slot, attribute_name) in HARDPOINTS {
        let used = context
            .items
            .iter()
            .filter(|item| item.is_fitted() && item.hardpoint == Some(slot))
            .count();
        report(context, found, slot, attribute_name, used);
    }

    for item in &context.items {
        if let Some(rack) = item.rack.filter(|rack| item.rack() != Some(*rack)) {
            found.push(item.violation(Rule::WrongSlot { expected: rack }));
        }
        if holds_one(item.fit.slot) && shares_slot(context, item) {
            found.push(item.violation(Rule::SlotTaken));
        }
        if let Some(expected) = wrong_slot_index(context, item) {
            found.push(item.violation(Rule::WrongSlotIndex { expected }));
        }
        if subsystem_taken(context, item) {
            found.push(item.violation(Rule::SubsystemTaken));
        }
    }
}

fn report<I: Info>(
    context: &Context<'_, I>,
    found: &mut Vec<Violation>,
    slot: SlotKind,
    attribute_name: &str,
    used: usize,
) {
    let Some(attribute_id) = context.attribute_id(attribute_name) else {
        return;
    };

    let available = context.amount(context.ship(), attribute_id) as u32;
    let used = used as u32;
    if used > available {
        found.push(Violation {
            target: Target::Ship,
            rule: Rule::Slots {
                slot,
                used,
                available,
            },
        });
    }
}

/// The slots that hold a single item; a bay holds as many as fit in it.
fn holds_one(slot: Slot) -> bool {
    !matches!(slot, Slot::DroneBay | Slot::FighterBay | Slot::Cargo)
}

fn shares_slot<I: Info>(context: &Context<'_, I>, item: &Item<'_>) -> bool {
    context
        .items
        .iter()
        .any(|other| other.index != item.index && other.fit.slot == item.fit.slot)
}

/// An implant and a booster sit in a numbered slot of their own, which the
/// item names itself.
fn wrong_slot_index<I: Info>(context: &Context<'_, I>, item: &Item<'_>) -> Option<u16> {
    let (attribute_name, index) = match item.fit.slot {
        Slot::Implant(index) => ("implantness", u16::from(index)),
        Slot::Booster(index) => ("boosterness", index),
        _ => return None,
    };

    let attribute_id = context.attribute_id(attribute_name)?;
    let expected = context.value(item.result, attribute_id)? as u16;

    (expected != index).then_some(expected)
}

/// Every subsystem covers one part of the ship, and only one may.
fn subsystem_taken<I: Info>(context: &Context<'_, I>, item: &Item<'_>) -> bool {
    if !matches!(item.fit.slot, Slot::Subsystem(_)) {
        return false;
    }

    let Some(attribute_id) = context.attribute_id("subSystemSlot") else {
        return false;
    };
    let Some(part) = context.value(item.result, attribute_id) else {
        return false;
    };

    context.items.iter().any(|other| {
        other.index != item.index
            && matches!(other.fit.slot, Slot::Subsystem(_))
            && context.value(other.result, attribute_id) == Some(part)
    })
}
