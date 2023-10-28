use super::item::{Attribute, Item};
use super::{Info, Pass, Ship};

const ATTRIBUTE_MASS_ID: i32 = 4;
const ATTRIBUTE_CAPACITY_ID: i32 = 38;
const ATTRIBUTE_VOLUME_ID: i32 = 161;
const ATTRIBUTE_RADIUS_ID: i32 = 162;
const ATTRIBUTE_SKILL_LEVEL_ID: i32 = 280;

pub struct PassOne {}

impl Item {
    pub fn set_attribute(&mut self, attribute_id: i32, value: f32) {
        self.attributes.insert(attribute_id, Attribute::new(value));
    }

    fn set_attributes(&mut self, info: &Info) {
        for dogma_attribute in info.get_dogma_attributes(self.type_id) {
            self.set_attribute(dogma_attribute.attributeID, dogma_attribute.value);
        }
    }
}

impl Pass for PassOne {
    fn pass(info: &Info, ship: &mut Ship) {
        ship.hull.set_attributes(info);

        /* Some attributes of ships come from the TypeID information. */
        let type_id = info.get_type_id(info.ship_layout.ship_id);
        ship.hull
            .set_attribute(ATTRIBUTE_MASS_ID, type_id.mass.unwrap());
        ship.hull
            .set_attribute(ATTRIBUTE_CAPACITY_ID, type_id.capacity.unwrap());
        ship.hull
            .set_attribute(ATTRIBUTE_VOLUME_ID, type_id.volume.unwrap());
        ship.hull
            .set_attribute(ATTRIBUTE_RADIUS_ID, type_id.radius.unwrap());

        for (skill_id, skill_level) in info.skills {
            let mut skill = Item::new(*skill_id);

            skill.set_attributes(info);
            skill.set_attribute(ATTRIBUTE_SKILL_LEVEL_ID, *skill_level as f32);

            ship.skills.push(skill);
        }

        for item_id in &info.ship_layout.items {
            let mut item = Item::new(*item_id);

            item.set_attributes(info);

            ship.items.push(item);
        }
    }
}
