use serde::Serialize;
use std::collections::BTreeMap;

mod info;
mod item;
mod pass_1;
mod pass_2;
mod pass_3;
mod pass_4;

use info::Info;
use item::Item;

#[derive(Serialize, Debug)]
pub struct Ship {
    pub hull: Item,
    pub items: Vec<Item>,

    #[serde(skip_serializing)]
    pub skills: Vec<Item>,
    #[serde(skip_serializing)]
    pub char: Item,
    #[serde(skip_serializing)]
    pub structure: Item,
}

impl Ship {
    pub fn new(ship_id: i32) -> Ship {
        Ship {
            hull: Item::new(ship_id),
            items: Vec::new(),
            skills: Vec::new(),
            char: Item::new(0),
            structure: Item::new(0),
        }
    }
}

trait Pass {
    fn pass(info: &Info, ship: &mut Ship);
}

pub fn calculate(ship_layout: &super::data_types::ShipLayout, skills: &BTreeMap<i32, i32>) -> Ship {
    let info = Info::new(ship_layout, skills);
    let mut ship = Ship::new(info.ship_layout.ship_id);

    pass_1::PassOne::pass(&info, &mut ship);
    pass_2::PassTwo::pass(&info, &mut ship);
    pass_3::PassThree::pass(&info, &mut ship);
    pass_4::PassFour::pass(&info, &mut ship);

    ship
}
