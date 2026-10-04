//! Sets a loaded fit to the states EVE gives it on import.

use std::collections::BTreeMap;

use esf_data::Info;
use esf_dogma_engine::{
    Calculation, Fit, GroupLimit, ItemResult, Options, Resource, Rule, Slot, State, calculate,
};

const GROUP_CLOAKING_DEVICE: i32 = 330;
const GROUP_MASS_ENTANGLER: i32 = 2008;

const DAMAGE_ATTRIBUTES: [&str; 4] = [
    "emDamage",
    "thermalDamage",
    "kineticDamage",
    "explosiveDamage",
];

/// Lower the states of a fit loaded from EFT or an ESI fitting to the ones
/// EVE sets when it imports that fit.
///
/// - A cloak or mass entangler is online, not active.
/// - Of a group with a limit on how many can be active, like a
///   microwarpdrive and an afterburner, the first ones stay active and the
///   rest go online.
/// - Only the drones the ship can have in space are launched, the most
///   damaging first; the rest stay in the bay, offline.
///
/// How many drones fit depends on the skills, so set the character first.
pub fn post_load(info: &impl Info, fit: &mut Fit) {
    for item in &mut fit.items {
        let group_id = info.get_type(item.type_id).map(|r#type| r#type.group_id());
        if in_rack(item.slot)
            && matches!(group_id, Some(GROUP_CLOAKING_DEVICE | GROUP_MASS_ENTANGLER))
        {
            item.state = item.state.min(State::Online);
        }
    }

    let options = Options {
        sources: false,
        validate: true,
    };
    let calculation = calculate(info, fit, &options);

    let mut active_limits: BTreeMap<i32, u32> = BTreeMap::new();
    let mut too_many_drones = false;
    for violation in calculation.violations.iter().flatten() {
        match violation.rule {
            Rule::MaxGroup {
                group_id,
                limit: GroupLimit::Active,
                allowed,
                ..
            } => {
                active_limits.insert(group_id, allowed);
            }
            Rule::Resource {
                resource: Resource::LaunchedDrones | Resource::DroneBandwidth,
                ..
            } => too_many_drones = true,
            _ => {}
        }
    }

    for (group_id, allowed) in active_limits {
        let mut active = 0;
        for (item, result) in fit.items.iter_mut().zip(&calculation.items) {
            let group_id_of = info.get_type(item.type_id).map(|r#type| r#type.group_id());
            if !in_rack(item.slot) || group_id_of != Some(group_id) || result.state < State::Active
            {
                continue;
            }

            active += 1;
            if active > allowed {
                item.state = State::Online;
            }
        }
    }

    if too_many_drones {
        launch_drones(info, fit, &calculation);
    }
}

fn in_rack(slot: Slot) -> bool {
    matches!(
        slot,
        Slot::High(_)
            | Slot::Medium(_)
            | Slot::Low(_)
            | Slot::Rig(_)
            | Slot::Subsystem(_)
            | Slot::Service(_)
    )
}

/// Launch the most damaging drones while the ship has room for them, and
/// split off the rest of a stack as offline.
fn launch_drones(info: &impl Info, fit: &mut Fit, calculation: &Calculation) {
    let value = |result: &ItemResult, name: &str| {
        info.attribute_name_to_id(name)
            .and_then(|attribute_id| result.attributes.get(&attribute_id))
            .map_or(0.0, |attribute| attribute.value)
    };
    let damage = |result: &ItemResult| {
        let damage: f64 = DAMAGE_ATTRIBUTES
            .iter()
            .map(|name| value(result, name))
            .sum();
        damage * value(result, "damageMultiplier")
    };

    let mut drones: Vec<usize> = (0..fit.items.len())
        .filter(|&index| {
            fit.items[index].slot == Slot::DroneBay
                && calculation.items[index].state >= State::Active
        })
        .collect();
    drones
        .sort_by(|&a, &b| damage(&calculation.items[b]).total_cmp(&damage(&calculation.items[a])));

    let mut drones_left = value(&calculation.character, "maxActiveDrones") as u32;
    let mut bandwidth_left = value(&calculation.ship, "droneBandwidth");
    let mut launched = vec![None; fit.items.len()];
    for index in drones {
        let quantity = fit.items[index].quantity;
        let bandwidth = value(&calculation.items[index], "droneBandwidthUsed");
        let fits = if bandwidth > 0.0 {
            (bandwidth_left / bandwidth + 1e-9) as u32
        } else {
            quantity
        };

        let count = quantity.min(drones_left).min(fits);
        drones_left -= count;
        bandwidth_left -= f64::from(count) * bandwidth;
        launched[index] = Some(count);
    }

    let items = std::mem::take(&mut fit.items);
    for (item, launched) in items.into_iter().zip(launched) {
        let Some(count) = launched.filter(|&count| count < item.quantity) else {
            fit.items.push(item);
            continue;
        };

        if count > 0 {
            let mut active = item.clone();
            active.quantity = count;
            fit.items.push(active);
        }
        let mut offline = item;
        offline.quantity -= count;
        offline.state = State::Offline;
        fit.items.push(offline);
    }
}
