use serde::Serialize;

mod attribute_ids;
pub mod item;
mod output;
mod pass_1;
mod pass_2;
mod pass_3;
mod pass_4;

use crate::fit::Fit;
use esf_data::info::Info;
use item::{Item, Object};

pub use output::{AttributeValue, Calculation, ItemResult};

#[derive(Serialize, Debug)]
pub struct Objects {
    pub ship: Item,
    pub items: Vec<Item>,
    pub skills: Vec<Item>,
    pub char: Item,
    pub structure: Item,
    pub target: Item,
}

impl Objects {
    fn get(&self, object: Object) -> Option<&Item> {
        match object {
            Object::Ship => Some(&self.ship),
            Object::Char => Some(&self.char),
            Object::Structure => Some(&self.structure),
            Object::Target => Some(&self.target),
            Object::Item(index) => Some(&self.items[index]),
            Object::Charge(index) => self.items[index].charge.as_deref(),
            Object::Skill(index) => Some(&self.skills[index]),
        }
    }

    fn get_mut(&mut self, object: Object) -> Option<&mut Item> {
        match object {
            Object::Ship => Some(&mut self.ship),
            Object::Char => Some(&mut self.char),
            Object::Structure => Some(&mut self.structure),
            Object::Target => Some(&mut self.target),
            Object::Item(index) => Some(&mut self.items[index]),
            Object::Charge(index) => self.items[index].charge.as_deref_mut(),
            Object::Skill(index) => Some(&mut self.skills[index]),
        }
    }

    pub fn new(ship_type_id: i32) -> Objects {
        Objects {
            ship: Item::new_fake(ship_type_id),
            items: Vec::new(),
            skills: Vec::new(),
            char: Item::new_fake(1373),
            structure: Item::new_fake(0),
            target: Item::new_fake(0),
        }
    }
}

trait Pass {
    fn pass(info: &impl Info, objects: &mut Objects);
}

pub fn calculate(info: &impl Info, fit: &Fit) -> Calculation {
    let mut objects = pass_1::PassOne::pass(info, fit);

    pass_2::PassTwo::pass(info, &mut objects);
    pass_3::PassThree::pass(info, &mut objects);
    pass_4::PassFour::pass(info, &mut objects);

    Calculation::new(&objects)
}
