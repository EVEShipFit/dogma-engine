use esf_data::Info;

use super::super::Objects;
use crate::fit::Slot;

/* Dogma counts a stack once per fighter in it, but a squadron takes one tube. */
pub fn attribute_fighter_tubes_used(info: &impl Info, objects: &mut Objects) {
    let tubes: Vec<_> = objects
        .items
        .iter()
        .filter(|item| matches!(item.slot, Some(Slot::FighterTube(_))))
        .collect();

    let has_tubes = info
        .attribute_name_to_id("fighterTubes")
        .is_some_and(|attribute_id| objects.ship.attributes.contains_key(&attribute_id));
    if tubes.is_empty() && !has_tubes {
        return;
    }

    let counts = [
        ("fighterTubesUsed", None),
        ("fighterLightSlotsUsed", Some("fighterSquadronIsLight")),
        ("fighterSupportSlotsUsed", Some("fighterSquadronIsSupport")),
        ("fighterHeavySlotsUsed", Some("fighterSquadronIsHeavy")),
    ];

    let mut used = Vec::new();
    for (used_name, role_name) in counts {
        let Some(used_id) = info.attribute_name_to_id(used_name) else {
            continue;
        };
        let role_id = match role_name {
            Some(role_name) => match info.attribute_name_to_id(role_name) {
                Some(role_id) => Some(role_id),
                None => continue,
            },
            None => None,
        };

        let count = tubes
            .iter()
            .filter(|item| {
                role_id.is_none_or(|role_id| {
                    item.attributes
                        .get(&role_id)
                        .is_some_and(|attribute| attribute.base_value != 0.0)
                })
            })
            .count();
        used.push((used_id, count as f64));
    }

    for (used_id, count) in used {
        objects.ship.add_attribute(used_id, 0.0, count);
    }
}
