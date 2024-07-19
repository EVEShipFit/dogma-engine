use super::item::{Attribute, EffectCategory, Item, Slot, SlotType};
use super::{Info, Pass, Ship};
use crate::data_types::{EsfSlotType, EsfState};

const ATTRIBUTE_MASS_ID: i32 = 4;
const ATTRIBUTE_CAPACITY_ID: i32 = 38;
const ATTRIBUTE_VOLUME_ID: i32 = 161;
const ATTRIBUTE_RADIUS_ID: i32 = 162;
const ATTRIBUTE_SKILL_LEVEL_ID: i32 = 280;

pub struct PassOne {}

impl Item {
    pub fn set_attribute(&mut self, attribute_id: i32, value: f64) {
        self.attributes.insert(attribute_id, Attribute::new(value));
    }

    fn set_attributes(&mut self, info: &impl Info) {
        for dogma_attribute in info.get_dogma_attributes(self.type_id) {
            self.set_attribute(dogma_attribute.attributeID, dogma_attribute.value);
        }

        /* Some attributes of items come from the Type information. */
        let r#type = info.get_type(self.type_id);
        if let Some(mass) = r#type.mass {
            self.set_attribute(ATTRIBUTE_MASS_ID, mass);
        }
        if let Some(capacity) = r#type.capacity {
            self.set_attribute(ATTRIBUTE_CAPACITY_ID, capacity);
        }
        if let Some(volume) = r#type.volume {
            self.set_attribute(ATTRIBUTE_VOLUME_ID, volume);
        }
        if let Some(radius) = r#type.radius {
            self.set_attribute(ATTRIBUTE_RADIUS_ID, radius);
        }
    }
}

impl Pass for PassOne {
    fn pass(info: &impl Info, ship: &mut Ship) {
        ship.hull.set_attributes(info);

        for (skill_id, skill_level) in info.skills() {
            let mut skill = Item::new_fake(*skill_id);

            skill.set_attributes(info);
            skill.set_attribute(ATTRIBUTE_SKILL_LEVEL_ID, *skill_level as f64);

            ship.skills.push(skill);
        }

        for module in &info.fit().modules {
            let state = match module.state {
                EsfState::Passive => EffectCategory::Passive,
                EsfState::Online => EffectCategory::Online,
                EsfState::Active => EffectCategory::Active,
                EsfState::Overload => EffectCategory::Overload,
            };

            let mut item = Item::new_module(
                module.type_id,
                Slot {
                    r#type: match module.slot.r#type {
                        EsfSlotType::High => SlotType::High,
                        EsfSlotType::Medium => SlotType::Medium,
                        EsfSlotType::Low => SlotType::Low,
                        EsfSlotType::Rig => SlotType::Rig,
                        EsfSlotType::SubSystem => SlotType::SubSystem,
                    },
                    index: Some(module.slot.index),
                },
                module.charge.as_ref().map(|charge| charge.type_id),
                state,
            );

            item.set_attributes(info);
            item.charge
                .as_mut()
                .map(|charge| charge.set_attributes(info));

            ship.items.push(item);
        }

        for drone in &info.fit().drones {
            let state = match drone.state {
                EsfState::Passive => EffectCategory::Passive,
                _ => EffectCategory::Active,
            };

            let mut item = Item::new_drone(drone.type_id, state);

            item.set_attributes(info);

            ship.items.push(item);
        }
    }
}
