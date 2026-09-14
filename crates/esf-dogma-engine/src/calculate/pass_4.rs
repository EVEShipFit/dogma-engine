use super::item::Attribute;
use super::{Info, Item, Objects, Pass};

pub struct PassFour {}

mod capacitor;
mod fighter;

impl Item {
    pub fn add_attribute(&mut self, attribute_id: i32, base_value: f64, value: f64) {
        let attribute = Attribute::new(base_value);
        attribute.value.set(Some(value));
        self.attributes.insert(attribute_id, attribute);
    }
}

/* Attributes don't contain all information displayed, so we calculate some fake attributes with those values. */
impl Pass for PassFour {
    fn pass(info: &impl Info, objects: &mut Objects) {
        capacitor::attribute_capacitor_depletes_in(info, objects);
        fighter::attribute_fighter_tubes_used(info, objects);
    }
}
