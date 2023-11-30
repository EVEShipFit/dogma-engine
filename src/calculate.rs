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

    #[serde(skip_serializing)]
    pub skills: Vec<Item>,
    #[serde(skip_serializing)]
    pub char: Item,
    #[serde(skip_serializing)]
    pub structure: Item,
    #[serde(skip_serializing)]
    pub target: Item,

    pub damage_profile: DamageProfile,
}

impl Ship {
    pub fn new(ship_type_id: i32) -> Ship {
        Ship {
            hull: Item::new_fake(ship_type_id),
            items: Vec::new(),
            skills: Vec::new(),
            char: Item::new_fake(0),
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
    fn pass(info: &Info, ship: &mut Ship);
}

pub fn calculate(esi_fit: &super::data_types::EsiFit, skills: &BTreeMap<i32, i32>) -> Ship {
    let info = Info::new(esi_fit, skills);
    let mut ship = Ship::new(info.esi_fit.ship_type_id);

    pass_1::PassOne::pass(&info, &mut ship);
    pass_2::PassTwo::pass(&info, &mut ship);
    pass_3::PassThree::pass(&info, &mut ship);
    pass_4::PassFour::pass(&info, &mut ship);

    ship
}
