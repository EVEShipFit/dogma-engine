use serde::Serialize;

mod attribute_ids;
pub mod item;
mod pass_1;
mod pass_2;
mod pass_3;
mod pass_4;

use crate::info::Info;
use item::{Item, Object};

#[derive(Serialize, Debug)]
pub struct DamageProfile {
    pub em: f64,
    pub explosive: f64,
    pub kinetic: f64,
    pub thermal: f64,
}

#[derive(Serialize, Debug)]
pub struct Ship {
    pub hull: Item,
    pub items: Vec<Item>,
    pub skills: Vec<Item>,
    pub char: Item,
    pub structure: Item,
    pub target: Item,

    pub damage_profile: DamageProfile,
}

impl Ship {
    fn get(&self, object: Object) -> Option<&Item> {
        match object {
            Object::Ship => Some(&self.hull),
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
            Object::Ship => Some(&mut self.hull),
            Object::Char => Some(&mut self.char),
            Object::Structure => Some(&mut self.structure),
            Object::Target => Some(&mut self.target),
            Object::Item(index) => Some(&mut self.items[index]),
            Object::Charge(index) => self.items[index].charge.as_deref_mut(),
            Object::Skill(index) => Some(&mut self.skills[index]),
        }
    }

    pub fn new(ship_type_id: i32) -> Ship {
        Ship {
            hull: Item::new_fake(ship_type_id),
            items: Vec::new(),
            skills: Vec::new(),
            char: Item::new_fake(1373),
            structure: Item::new_fake(0),
            target: Item::new_fake(0),
            damage_profile: DamageProfile {
                em: 0.25,
                explosive: 0.25,
                kinetic: 0.25,
                thermal: 0.25,
            },
        }
    }
}

trait Pass {
    fn pass(info: &impl Info, ship: &mut Ship);
}

pub fn calculate(info: &impl Info) -> Ship {
    let mut ship = Ship::new(info.fit().ship_type_id);

    pass_1::PassOne::pass(info, &mut ship);
    pass_2::PassTwo::pass(info, &mut ship);
    pass_3::PassThree::pass(info, &mut ship);
    pass_4::PassFour::pass(info, &mut ship);

    ship
}
