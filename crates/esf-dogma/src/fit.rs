//! The fit to calculate.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fit {
    pub name: Option<String>,
    pub ship: Ship,
    pub items: Vec<FitItem>,
    #[serde(default)]
    pub character: Character,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ship {
    pub type_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FitItem {
    pub type_id: i32,
    pub slot: Slot,
    /// 1 for modules. Stack size for drones and cargo.
    #[serde(default = "one")]
    pub quantity: u32,
    pub state: State,
    pub charge: Option<Charge>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "type", content = "index", rename_all = "snake_case")]
pub enum Slot {
    High(u8),
    Medium(u8),
    Low(u8),
    Rig(u8),
    Subsystem(u8),
    Service(u8),
    DroneBay,
    Cargo,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Offline,
    Online,
    Active,
    Overload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Charge {
    pub type_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Character {
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
