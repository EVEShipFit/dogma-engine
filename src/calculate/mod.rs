use serde::Serialize;

pub mod item;
mod pass_1;
mod pass_2;
mod pass_3;
mod pass_4;

use crate::info::Info;
use item::Item;

#[derive(Serialize, Debug)]
pub struct DamageProfile {
    pub em: f64,
    pub explosive: f64,
    pub kinetic: f64,
    pub thermal: f64,
}

impl Default for DamageProfile {
    fn default() -> Self {
        Self {
            em: 0.25,
            explosive: 0.25,
            kinetic: 0.25,
            thermal: 0.25,
        }
    }
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
    pub fn new(ship_type_id: i32, damage_profile: DamageProfile) -> Ship {
        Ship {
            hull: Item::new_fake(ship_type_id),
            items: Vec::new(),
            skills: Vec::new(),
            char: Item::new_fake(1373),
            structure: Item::new_fake(0),
            target: Item::new_fake(0),
            damage_profile,
        }
    }
}

trait Pass {
    fn pass(info: &impl Info, ship: &mut Ship);
}

pub fn calculate(info: &impl Info, damage_profile: DamageProfile) -> Ship {
    let mut ship = Ship::new(info.fit().ship_type_id, damage_profile);

    pass_1::PassOne::pass(info, &mut ship);
    pass_2::PassTwo::pass(info, &mut ship);
    pass_3::PassThree::pass(info, &mut ship);
    pass_4::PassFour::pass(info, &mut ship);

    ship
}
