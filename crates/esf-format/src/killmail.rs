//! ESI killmails: the ship that died, as it was fitted.

use esf_data::Info;
use esf_dogma_engine::Fit;
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use tsify::Tsify;

use crate::flags::place_of_flag;
use crate::listed::{ByInfo, Listed, fit, to_fit_items};

/// A killmail as ESI returns it; only what a fit is made from. Anything else
/// in it is ignored.
#[cfg_attr(feature = "typescript", derive(Tsify))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EsiKillmail {
    /// The id of the killmail.
    pub killmail_id: i32,
    /// The ship that died.
    pub victim: EsiKillmailVictim,
}

/// The victim of a killmail.
#[cfg_attr(feature = "typescript", derive(Tsify))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EsiKillmailVictim {
    /// The type id of the ship.
    pub ship_type_id: i32,
    /// Everything on and in the ship.
    #[serde(default)]
    #[cfg_attr(feature = "typescript", tsify(optional))]
    pub items: Vec<EsiKillmailItem>,
}

/// An item of a killmail.
#[cfg_attr(feature = "typescript", derive(Tsify))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EsiKillmailItem {
    /// An inventory flag, like 27 for the first high slot.
    pub flag: i32,
    /// The type id of the item.
    pub item_type_id: i32,
    /// How many of the item were destroyed.
    #[serde(default)]
    #[cfg_attr(feature = "typescript", tsify(optional))]
    pub quantity_destroyed: u32,
    /// How many of the item dropped.
    #[serde(default)]
    #[cfg_attr(feature = "typescript", tsify(optional))]
    pub quantity_dropped: u32,
}

/// Load the fit of the ship that died from a killmail, with what it carried.
/// The fit has no skills, and is named after the killmail.
///
/// A charge in the slot of a module is loaded in it; what was destroyed and
/// what dropped are added up. Items under a flag a fit has no place for, and
/// what was inside a container, are left out.
pub fn load_killmail(info: &impl Info, killmail: &EsiKillmail) -> Fit {
    let listed = killmail
        .victim
        .items
        .iter()
        .filter_map(|item| {
            let place = place_of_flag(item.flag)?;
            let quantity = item.quantity_destroyed + item.quantity_dropped;
            Some(Listed::new(place, item.item_type_id, quantity))
        })
        .collect();

    fit(
        Some(format!("Killmail {}", killmail.killmail_id)),
        killmail.victim.ship_type_id,
        to_fit_items(&ByInfo(info), listed),
    )
}
