//! Sets up a loaded fit the way EVE does on import.

use crate::esf::lookup::Lookup;
use esf_data::{Info, InfoEsf};
use esf_dogma_engine::{
    Calculation, Fit, GroupLimit, ItemResult, Options, Resource, Rule, Slot, State, calculate,
};

const GROUP_CLOAKING_DEVICE: i32 = 330;
const GROUP_MASS_ENTANGLER: i32 = 2008;

/// The tubes each class of squadron takes, as (role on the fighter, tubes on the ship).
const FIGHTER_CLASSES: [(&str, &str); 3] = [
    ("fighterSquadronIsLight", "fighterLightSlots"),
    ("fighterSquadronIsSupport", "fighterSupportSlots"),
    ("fighterSquadronIsHeavy", "fighterHeavySlots"),
];

const DAMAGE_ATTRIBUTES: [&str; 4] = [
    "emDamage",
    "thermalDamage",
    "kineticDamage",
    "explosiveDamage",
];

/// Set up a fit loaded from EFT or an ESI fitting the way EVE does when it
/// imports that fit.
///
/// - A ship with modes and none set starts in its first mode.
/// - A cloak or mass entangler is online, not active.
/// - Of a group with a limit on how many can be online or active, like a
///   microwarpdrive and an afterburner, the first ones keep their state and
///   the rest go a state lower.
/// - Only the drones the ship can have in space are launched, the most
///   damaging first; the rest stay in the bay, offline.
/// - Fighters in the bay fill the empty tubes their class may use, a full
///   squadron each.
///
/// How many drones fit depends on the skills, so set the character first.
pub fn post_load(info: &impl InfoEsf, fit: &mut Fit) {
    if fit.ship.mode.is_none() {
        fit.ship.mode = Lookup { info }.modes(fit.ship.type_id).first().copied();
    }

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

    let mut limits: Vec<(State, i32, u32)> = Vec::new();
    let mut too_many_drones = false;
    for violation in calculation.violations.iter().flatten() {
        match violation.rule {
            Rule::MaxGroup {
                group_id,
                limit,
                allowed,
                ..
            } => {
                let counted = match limit {
                    GroupLimit::Online => State::Online,
                    GroupLimit::Active => State::Active,
                    GroupLimit::Fitted => continue,
                };
                if !limits.contains(&(counted, group_id, allowed)) {
                    limits.push((counted, group_id, allowed));
                }
            }
            Rule::Resource {
                resource: Resource::LaunchedDrones | Resource::DroneBandwidth,
                ..
            } => too_many_drones = true,
            _ => {}
        }
    }

    limits.sort_by_key(|(counted, _, _)| *counted);
    for (counted, group_id, allowed) in limits {
        let mut used = 0;
        for (item, result) in fit.items.iter_mut().zip(&calculation.items) {
            let group_id_of = info.get_type(item.type_id).map(|r#type| r#type.group_id());
            if !in_rack(item.slot)
                || group_id_of != Some(group_id)
                || item.state.min(result.state) < counted
            {
                continue;
            }

            used += 1;
            if used > allowed {
                item.state = match counted {
                    State::Online => State::Offline,
                    _ => State::Online,
                };
            }
        }
    }

    if too_many_drones {
        launch_drones(info, fit, &calculation);
    }
    launch_fighters(info, fit, &calculation);
}

fn value(info: &impl Info, result: &ItemResult, name: &str) -> f64 {
    info.attribute_name_to_id(name)
        .and_then(|attribute_id| result.attributes.get(&attribute_id))
        .map_or(0.0, |attribute| attribute.value)
}

fn base_value(info: &impl Info, type_id: i32, name: &str) -> f64 {
    let Some(attribute_id) = info.attribute_name_to_id(name) else {
        return 0.0;
    };
    info.get_dogma_attributes(type_id)
        .into_iter()
        .flatten()
        .find(|attribute| attribute.attribute_id() == attribute_id)
        .map_or(0.0, |attribute| f64::from(attribute.value()))
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
    let value = |result: &ItemResult, name: &str| value(info, result, name);
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

/// Move fighters from the bay into the empty tubes, a full squadron per tube,
/// as far as the tubes for their class allow.
fn launch_fighters(info: &impl Info, fit: &mut Fit, calculation: &Calculation) {
    let tubes = value(info, &calculation.ship, "fighterTubes") as u8;
    let class_of = |type_id: i32| {
        FIGHTER_CLASSES
            .iter()
            .position(|(role, _)| base_value(info, type_id, role) != 0.0)
    };

    let mut class_left =
        FIGHTER_CLASSES.map(|(_, slots)| value(info, &calculation.ship, slots) as u32);
    let mut taken = vec![false; usize::from(tubes)];
    for item in &fit.items {
        let Slot::FighterTube(index) = item.slot else {
            continue;
        };
        if let Some(taken) = taken.get_mut(usize::from(index)) {
            *taken = true;
        }
        if let Some(class) = class_of(item.type_id) {
            class_left[class] = class_left[class].saturating_sub(1);
        }
    }

    let mut launched = Vec::new();
    for item in fit
        .items
        .iter_mut()
        .filter(|item| item.slot == Slot::FighterBay)
    {
        let class = class_of(item.type_id);
        let squadron = (base_value(info, item.type_id, "fighterSquadronMaxSize") as u32).max(1);

        while item.quantity > 0 && class.is_none_or(|class| class_left[class] > 0) {
            let Some(tube) = taken.iter().position(|taken| !taken) else {
                break;
            };
            taken[tube] = true;
            if let Some(class) = class {
                class_left[class] -= 1;
            }

            let mut squad = item.clone();
            squad.slot = Slot::FighterTube(tube as u8);
            squad.quantity = item.quantity.min(squadron);
            squad.state = State::Active;
            item.quantity -= squad.quantity;
            launched.push(squad);
        }
    }

    fit.items
        .retain(|item| item.slot != Slot::FighterBay || item.quantity > 0);
    fit.items.extend(launched);
}
