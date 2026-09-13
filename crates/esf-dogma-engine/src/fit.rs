//! The fit to calculate.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

/// A ship, what is fitted to it, and the character flying it.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fit {
    /// Only for display; the calculation does not use it.
    pub name: Option<String>,
    /// The ship.
    pub ship: Ship,
    /// Modules, drones and cargo.
    pub items: Vec<FitItem>,
    /// The character flying the ship.
    #[serde(default)]
    pub character: Character,
}

/// The ship of a fit.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ship {
    /// The type id of the ship.
    pub type_id: i32,
}

/// A module, drone or item in cargo.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FitItem {
    /// The type id of the item.
    pub type_id: i32,
    /// Where the item is.
    pub slot: Slot,
    /// 1 for modules. Stack size for drones and cargo.
    #[serde(default = "one")]
    pub quantity: u32,
    /// The state asked for; the calculation lowers it when the item cannot
    /// reach it.
    pub state: State,
    /// The charge loaded in the module, if any.
    pub charge: Option<Charge>,
}

/// Where an item is. The number is the position in its rack, starting at 0.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "type", content = "index", rename_all = "snake_case")]
pub enum Slot {
    /// A high slot.
    High(u8),
    /// A medium slot.
    Medium(u8),
    /// A low slot.
    Low(u8),
    /// A rig slot.
    Rig(u8),
    /// A subsystem slot.
    Subsystem(u8),
    /// A service slot, on a structure.
    Service(u8),
    /// The drone bay; a drone that is not offline is in space.
    DroneBay,
    /// The cargo hold; nothing in it is calculated.
    Cargo,
}

/// The state of an item, lowest first.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// Only passive effects apply.
    Offline,
    /// Online effects apply too.
    Online,
    /// Active effects apply too.
    Active,
    /// Overload effects apply too.
    Overload,
}

/// A charge loaded in a module.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Charge {
    /// The type id of the charge.
    pub type_id: i32,
}

/// The character flying the ship.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Character {
    /// The level of each skill, by type id. A skill that is not listed is not
    /// trained, and gives no bonus.
    #[serde(default, deserialize_with = "id_map")]
    pub skills: BTreeMap<i32, u8>,
}

fn one() -> u32 {
    1
}

/* JSON and JavaScript objects only have string keys. */
fn id_map<'de, D, V>(deserializer: D) -> Result<BTreeMap<i32, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    BTreeMap::<String, V>::deserialize(deserializer)?
        .into_iter()
        .map(|(key, value)| {
            let key = key.parse().map_err(|_| {
                serde::de::Error::custom(format!("expected an identifier, found {key:?}"))
            })?;
            Ok((key, value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_fit_fills_defaults() {
        let fit: Fit = serde_json::from_str(
            r#"{
                "ship": {"type_id": 587},
                "items": [
                    {"type_id": 2873, "slot": {"type": "high", "index": 0}, "state": "active"},
                    {"type_id": 2488, "slot": {"type": "drone_bay"}, "quantity": 5, "state": "active"}
                ],
                "character": {"skills": {"3300": 5}}
            }"#,
        )
        .unwrap();

        assert_eq!(fit.items[0].slot, Slot::High(0));
        assert_eq!(fit.items[0].quantity, 1);
        assert_eq!(fit.items[1].slot, Slot::DroneBay);
        assert_eq!(fit.items[1].quantity, 5);
        assert_eq!(fit.character.skills[&3300], 5);
    }

    #[test]
    fn round_trips_through_json() {
        let fit = Fit {
            name: Some("Rifter".to_string()),
            ship: Ship { type_id: 587 },
            items: vec![FitItem {
                type_id: 47408,
                slot: Slot::Medium(2),
                quantity: 1,
                state: State::Overload,
                charge: None,
            }],
            character: Character::default(),
        };

        let json = serde_json::to_string(&fit).unwrap();
        let parsed: Fit = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.items[0].slot, Slot::Medium(2));
        assert_eq!(parsed.items[0].state, State::Overload);
    }
}
