//! The fit to calculate.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Deserializer, Serialize};

/// A ship, what is fitted to it, and the character flying it.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fit {
    /// Only for display; the calculation does not use it.
    pub name: Option<String>,
    /// The ship.
    pub ship: Ship,
    /// Modules, drones, fighters, implants, boosters and cargo.
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
    /// The type id of the active mode, for ships that have modes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<i32>,
}

/// A module, drone, fighter squadron, implant, booster or item in cargo.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FitItem {
    /// The type id of the item.
    pub type_id: i32,
    /// Where the item is.
    pub slot: Slot,
    /// 1 for modules. Stack size for drones, fighters and cargo; for fighters
    /// in a tube, the size of the squadron.
    #[serde(default = "one")]
    pub quantity: u32,
    /// The state asked for; the calculation lowers it when the item cannot
    /// reach it.
    pub state: State,
    /// The charge loaded in the module, if any.
    pub charge: Option<Charge>,
    /// Only for mutated modules and drones.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation: Option<Mutation>,
    /// Only for fighters: the abilities used, by effect id. `None` uses the
    /// abilities the fighter uses by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fighter_abilities: Option<BTreeSet<i32>>,
    /// Only for boosters: the side effects rolled, by effect id. Empty means
    /// none.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub booster_side_effects: BTreeSet<i32>,
    /// How far a module whose bonus grows every cycle has spooled.
    /// Only per-second stats use it; volley is always unspooled.
    /// `None` is fully spooled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spool: Option<Spool>,
}

/// Where an item is. The number is the position in its rack, starting at 0;
/// for implants and boosters, the slot as EVE numbers it, starting at 1.
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
    /// A fighter tube; a squadron that is not offline is in space.
    FighterTube(u8),
    /// The fighter bay; nothing in it is in space.
    FighterBay,
    /// An implant; the number is its `implantness`.
    Implant(u8),
    /// A booster; the number is its `boosterness`.
    Booster(u16),
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

/// How a module or drone was mutated.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mutation {
    /// The type id of the item before it was mutated.
    pub base: i32,
    /// The rolled value of each mutated attribute, by attribute id.
    #[serde(default, deserialize_with = "id_map")]
    pub attributes: BTreeMap<i32, f64>,
}

/// How far a module has spooled.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Spool {
    /// The bonus reached so far: 0.0 is unspooled, 2.125 is +212.5%.
    MultiplierBonus(f64),
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

        assert_eq!(fit.ship.mode, None);
        assert_eq!(fit.items[0].slot, Slot::High(0));
        assert_eq!(fit.items[0].quantity, 1);
        assert_eq!(fit.items[0].fighter_abilities, None);
        assert_eq!(fit.items[0].spool, None);
        assert_eq!(fit.items[1].slot, Slot::DroneBay);
        assert_eq!(fit.items[1].quantity, 5);
        assert_eq!(fit.character.skills[&3300], 5);
    }

    #[test]
    fn reads_fighters() {
        let fit: Fit = serde_json::from_str(
            r#"{
                "ship": {"type_id": 23911},
                "items": [
                    {"type_id": 40556, "slot": {"type": "fighter_tube", "index": 1}, "quantity": 6, "state": "active", "fighter_abilities": [6465, 6431]},
                    {"type_id": 40556, "slot": {"type": "fighter_bay"}, "quantity": 3, "state": "offline", "fighter_abilities": []}
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(fit.items[0].slot, Slot::FighterTube(1));
        assert_eq!(
            fit.items[0].fighter_abilities,
            Some(BTreeSet::from([6431, 6465]))
        );
        assert_eq!(fit.items[1].slot, Slot::FighterBay);
        assert_eq!(fit.items[1].fighter_abilities, Some(BTreeSet::new()));
    }

    #[test]
    fn reads_implants_and_boosters() {
        let fit: Fit = serde_json::from_str(
            r#"{
                "ship": {"type_id": 587},
                "items": [
                    {"type_id": 20499, "slot": {"type": "implant", "index": 1}, "state": "online"},
                    {"type_id": 9950, "slot": {"type": "booster", "index": 1}, "state": "online", "booster_side_effects": [2745, 2737]},
                    {"type_id": 57285, "slot": {"type": "booster", "index": 504}, "state": "online"}
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(fit.items[0].slot, Slot::Implant(1));
        assert_eq!(fit.items[1].slot, Slot::Booster(1));
        assert_eq!(
            fit.items[1].booster_side_effects,
            BTreeSet::from([2737, 2745])
        );
        assert_eq!(fit.items[2].slot, Slot::Booster(504));
        assert!(fit.items[2].booster_side_effects.is_empty());
    }

    #[test]
    fn reads_mutation() {
        let fit: Fit = serde_json::from_str(
            r#"{
                "ship": {"type_id": 587},
                "items": [
                    {"type_id": 47732, "slot": {"type": "medium", "index": 0}, "state": "active", "mutation": {"base": 448, "attributes": {"6": 7.5, "54": 10500}}}
                ]
            }"#,
        )
        .unwrap();

        let mutation = fit.items[0].mutation.as_ref().unwrap();
        assert_eq!(mutation.base, 448);
        assert_eq!(
            mutation.attributes,
            BTreeMap::from([(6, 7.5), (54, 10500.0)])
        );
    }

    #[test]
    fn reads_spool() {
        let fit: Fit = serde_json::from_str(
            r#"{
                "ship": {"type_id": 52250},
                "items": [
                    {"type_id": 47914, "slot": {"type": "high", "index": 0}, "state": "active", "spool": {"multiplier_bonus": 0.7}}
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(fit.items[0].spool, Some(Spool::MultiplierBonus(0.7)));
    }

    #[test]
    fn reads_mode() {
        let fit: Fit = serde_json::from_str(
            r#"{
                "ship": {"type_id": 34317, "mode": 34319},
                "items": []
            }"#,
        )
        .unwrap();

        assert_eq!(fit.ship.type_id, 34317);
        assert_eq!(fit.ship.mode, Some(34319));
    }

    #[test]
    fn round_trips_through_json() {
        let fit = Fit {
            name: Some("Rifter".to_string()),
            ship: Ship {
                type_id: 587,
                mode: None,
            },
            items: vec![FitItem {
                type_id: 47408,
                slot: Slot::Medium(2),
                quantity: 1,
                state: State::Overload,
                charge: None,
                mutation: None,
                fighter_abilities: None,
                booster_side_effects: BTreeSet::new(),
                spool: None,
            }],
            character: Character::default(),
        };

        let json = serde_json::to_string(&fit).unwrap();
        let parsed: Fit = serde_json::from_str(&json).unwrap();

        assert!(!json.contains("fighter_abilities"));
        assert!(!json.contains("booster_side_effects"));
        assert!(!json.contains("spool"));
        assert!(!json.contains("mutation"));
        assert!(!json.contains("mode"));
        assert_eq!(parsed.items[0].slot, Slot::Medium(2));
        assert_eq!(parsed.items[0].state, State::Overload);
    }
}
