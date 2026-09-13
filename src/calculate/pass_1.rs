use super::attribute_ids::{
    ATTRIBUTE_CAPACITY_ID, ATTRIBUTE_MASS_ID, ATTRIBUTE_RADIUS_ID, ATTRIBUTE_SKILL_LEVEL_ID,
    ATTRIBUTE_VOLUME_ID,
};
use super::item::{Attribute, EffectCategory, Item, Slot, SlotType};
use super::{Info, Pass, Ship};
use crate::data_types::{EsfSlotType, EsfState};

pub struct PassOne {}

impl Item {
    pub fn set_attribute(&mut self, attribute_id: i32, value: f64) {
        self.attributes.insert(attribute_id, Attribute::new(value));
    }

    fn set_type_ids(&mut self, info: &impl Info) {
        if let Some(r#type) = info.get_type(self.type_id) {
            self.group_id = r#type.group_id();
            self.category_id = r#type.category_id();
        }
    }

    fn set_attributes(&mut self, info: &impl Info) {
        self.set_type_ids(info);

        if let Some(dogma_attributes) = info.get_dogma_attributes(self.type_id) {
            for dogma_attribute in dogma_attributes {
                self.set_attribute(
                    dogma_attribute.attribute_id(),
                    dogma_attribute.value() as f64,
                );
            }
        }

        /* Some attributes of items come from the Type information. */
        let Some(r#type) = info.get_type(self.type_id) else {
            return;
        };
        if let Some(mass) = r#type.mass() {
            self.set_attribute(ATTRIBUTE_MASS_ID, mass as f64);
        }
        if let Some(capacity) = r#type.capacity() {
            self.set_attribute(ATTRIBUTE_CAPACITY_ID, capacity as f64);
        }
        if let Some(volume) = r#type.volume() {
            self.set_attribute(ATTRIBUTE_VOLUME_ID, volume as f64);
        }
        if let Some(radius) = r#type.radius() {
            self.set_attribute(ATTRIBUTE_RADIUS_ID, radius as f64);
        }
    }
}

impl Pass for PassOne {
    fn pass(info: &impl Info, ship: &mut Ship) {
        ship.hull.set_attributes(info);

        /* These carry no attributes, but pass 2 still wants their category. */
        ship.char.set_type_ids(info);
        ship.structure.set_type_ids(info);
        ship.target.set_type_ids(info);

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
                        EsfSlotType::Service => SlotType::Service,
                    },
                    index: Some(module.slot.index),
                },
                module.charge.as_ref().map(|charge| charge.type_id),
                state,
            );

            item.set_attributes(info);
            if let Some(charge) = item.charge.as_mut() {
                charge.set_attributes(info)
            }

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
