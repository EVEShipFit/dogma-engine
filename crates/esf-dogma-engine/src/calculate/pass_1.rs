use super::attribute_ids::{
    ATTRIBUTE_CAPACITY_ID, ATTRIBUTE_MASS_ID, ATTRIBUTE_RADIUS_ID, ATTRIBUTE_SKILL_LEVEL_ID,
    ATTRIBUTE_VOLUME_ID,
};
use super::item::{Attribute, Item};
use super::{Info, Objects};
use crate::fit::{Fit, Mutation};

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
        self.set_type_attributes(info, self.type_id);
    }

    /* The mutated type holds only what sets it apart from its bases, like its
     * skill requirements; the rest comes from the base. */
    fn set_mutated_attributes(&mut self, info: &impl Info, mutation: &Mutation) {
        self.set_type_ids(info);
        self.set_type_attributes(info, mutation.base);
        self.set_type_attributes(info, self.type_id);

        for (attribute_id, value) in &mutation.attributes {
            self.set_attribute(*attribute_id, *value);
        }
    }

    fn set_type_attributes(&mut self, info: &impl Info, type_id: i32) {
        if let Some(dogma_attributes) = info.get_dogma_attributes(type_id) {
            for dogma_attribute in dogma_attributes {
                self.set_attribute(
                    dogma_attribute.attribute_id(),
                    dogma_attribute.value() as f64,
                );
            }
        }

        /* Some attributes of items come from the Type information. */
        let Some(r#type) = info.get_type(type_id) else {
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

impl PassOne {
    pub fn pass(info: &impl Info, fit: &Fit) -> Objects {
        let mut objects = Objects::new(fit.ship.type_id, fit.ship.mode);

        objects.ship.set_attributes(info);
        if let Some(mode) = objects.mode.as_mut() {
            mode.set_attributes(info);
        }

        /* These carry no attributes, but pass 2 still wants their category. */
        objects.char.set_type_ids(info);
        objects.target.set_type_ids(info);

        for (skill_id, skill_level) in &fit.character.skills {
            let mut skill = Item::new_fake(*skill_id);

            skill.set_attributes(info);
            skill.set_attribute(ATTRIBUTE_SKILL_LEVEL_ID, *skill_level as f64);

            objects.skills.push(skill);
        }

        for fit_item in &fit.items {
            let mut item = Item::new_fit(fit_item);

            if item.is_calculated() {
                match &fit_item.mutation {
                    Some(mutation) => item.set_mutated_attributes(info, mutation),
                    None => item.set_attributes(info),
                }
                if let Some(charge) = item.charge.as_mut() {
                    charge.set_attributes(info)
                }
            }

            objects.items.push(item);
        }

        objects
    }
}
