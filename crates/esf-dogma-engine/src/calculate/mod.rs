mod attribute_ids;
mod item;
mod outgoing;
mod output;
mod pass_1;
mod pass_2;
mod pass_3;
mod pass_4;

use serde::Deserialize;

use crate::fit::Fit;
use esf_data::Info;
use item::{Item, Object};

pub use item::EffectOperator;
pub use output::{
    AttributeValue, BuffResult, BuffSource, Calculation, ItemResult, Source, SourceRef,
};

/// What [`calculate()`] reports on top of the values.
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct Options {
    /// Report per attribute the modifiers its value was calculated from.
    pub sources: bool,
}

#[derive(Debug)]
pub(crate) struct Objects {
    pub ship: Item,
    pub mode: Option<Item>,
    pub items: Vec<Item>,
    pub skills: Vec<Item>,
    pub char: Item,
    pub target: Item,
    pub beacons: Vec<Item>,
    pub buffs: Vec<BuffResult>,
    pub sources: bool,
}

impl Objects {
    fn get(&self, object: Object) -> Option<&Item> {
        match object {
            Object::Ship => Some(&self.ship),
            Object::Mode => self.mode.as_ref(),
            Object::Char => Some(&self.char),
            Object::Target => Some(&self.target),
            Object::Item(index) => Some(&self.items[index]),
            Object::Charge(index) => self.items[index].charge.as_deref(),
            Object::Skill(index) => Some(&self.skills[index]),
            Object::Beacon(index) => Some(&self.beacons[index]),
        }
    }

    fn get_mut(&mut self, object: Object) -> Option<&mut Item> {
        match object {
            Object::Ship => Some(&mut self.ship),
            Object::Mode => self.mode.as_mut(),
            Object::Char => Some(&mut self.char),
            Object::Target => Some(&mut self.target),
            Object::Item(index) => Some(&mut self.items[index]),
            Object::Charge(index) => self.items[index].charge.as_deref_mut(),
            Object::Skill(index) => Some(&mut self.skills[index]),
            Object::Beacon(index) => Some(&mut self.beacons[index]),
        }
    }

    pub fn new(ship_type_id: i32, mode_type_id: Option<i32>) -> Objects {
        Objects {
            ship: Item::new_fake(ship_type_id),
            mode: mode_type_id.map(Item::new_fake),
            items: Vec::new(),
            skills: Vec::new(),
            char: Item::new_fake(1373),
            target: Item::new_fake(0),
            beacons: Vec::new(),
            buffs: Vec::new(),
            sources: false,
        }
    }
}

trait Pass {
    fn pass(info: &impl Info, objects: &mut Objects);
}

/// Calculate every attribute of the ship, its items and the character.
pub fn calculate(info: &impl Info, fit: &Fit, options: &Options) -> Calculation {
    let mut objects = pass_1::PassOne::pass(info, fit);
    objects.sources = options.sources;

    pass_2::PassTwo::pass(info, &mut objects);
    pass_3::PassThree::pass(info, &mut objects);
    pass_4::PassFour::pass(info, &mut objects);

    Calculation::new(info, &objects)
}
