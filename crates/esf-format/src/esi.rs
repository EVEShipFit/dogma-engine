//! ESI fittings, the fits a character saves in game.

use esf_data::Info;
use esf_dogma_engine::{Fit, Slot};
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use tsify::Tsify;

use crate::flags::{flag_name, place_of_flag_name};
use crate::listed::{ByInfo, Listed, fit, to_fit_items};

/// A fitting as ESI saves it for a character.
#[cfg_attr(feature = "typescript", derive(Tsify))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EsiFitting {
    /// Only on fittings ESI returns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "typescript", tsify(optional))]
    pub fitting_id: Option<i32>,
    /// The name of the fitting.
    pub name: String,
    /// The description of the fitting.
    #[serde(default)]
    pub description: String,
    /// The type id of the ship.
    pub ship_type_id: i32,
    /// Everything on and in the ship.
    pub items: Vec<EsiFittingItem>,
}

/// An item of an ESI fitting.
#[cfg_attr(feature = "typescript", derive(Tsify))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EsiFittingItem {
    /// An inventory flag by name, like `HiSlot0`, `DroneBay` or `Cargo`.
    pub flag: String,
    /// How many of the item.
    pub quantity: u32,
    /// The type id of the item.
    pub type_id: i32,
}

/* ESI refuses a longer name. */
const MAX_NAME_LENGTH: usize = 50;

/// Load a fit from an ESI fitting. The fit has no skills.
///
/// A charge in the slot of a module is loaded in it. Items under a flag a fit
/// has no place for are left out.
pub fn load_esi_fitting(info: &impl Info, fitting: &EsiFitting) -> Fit {
    let listed = fitting
        .items
        .iter()
        .filter_map(|item| {
            let place = place_of_flag_name(&item.flag)?;
            Some(Listed::new(place, item.type_id, item.quantity))
        })
        .collect();

    fit(
        Some(fitting.name.clone()),
        fitting.ship_type_id,
        to_fit_items(&ByInfo(info), listed),
    )
}

/// Write a fit as an ESI fitting.
///
/// A charge is listed in the slot of the module it is loaded in. ESI has no
/// fighter tubes, so a squadron goes in the fighter bay; nor implants or
/// boosters, which go in the cargo. States are not kept. A fit without a name
/// is named after its ship.
pub fn save_esi_fitting(info: &impl Info, fit: &Fit) -> EsiFitting {
    let mut items: Vec<EsiFittingItem> = Vec::new();
    let mut add = |flag: &str, type_id: i32, quantity: u32| {
        let listed = items
            .iter_mut()
            .find(|item| item.flag == flag && item.type_id == type_id);
        match listed {
            Some(item) => item.quantity += quantity,
            None => items.push(EsiFittingItem {
                flag: flag.to_string(),
                quantity,
                type_id,
            }),
        }
    };

    for item in &fit.items {
        let slot = match item.slot {
            Slot::FighterTube(_) => Slot::FighterBay,
            Slot::Implant(_) | Slot::Booster(_) => Slot::Cargo,
            slot => slot,
        };
        let flag = flag_name(slot);

        add(&flag, item.type_id, item.quantity);
        if let Some(charge) = &item.charge {
            add(&flag, charge.type_id, 1);
        }
    }

    let name = fit
        .name
        .as_deref()
        .filter(|name| !name.is_empty())
        .or_else(|| info.get_type(fit.ship.type_id).map(|r#type| r#type.name()))
        .unwrap_or("Fit");

    EsiFitting {
        fitting_id: None,
        name: name.chars().take(MAX_NAME_LENGTH).collect(),
        description: String::new(),
        ship_type_id: fit.ship.type_id,
        items,
    }
}
