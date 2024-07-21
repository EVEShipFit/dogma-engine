use super::item::Attribute;
use super::{Info, Item, Pass, Ship};

pub struct PassFour {}

mod capacitor;

impl Item {
    pub fn add_attribute(&mut self, attribute_id: i32, base_value: f64, value: f64) {
        let mut attribute = Attribute::new(base_value);
        attribute.value = Some(value);
        self.attributes.insert(attribute_id as i32, attribute);
    }
}

/* Attributes don't contain all information displayed, so we calculate some fake attributes with those values. */
impl Pass for PassFour {
    fn pass(info: &impl Info, ship: &mut Ship) {
        capacitor::attribute_capacitor_depletes_in(info, ship);
    }
}
