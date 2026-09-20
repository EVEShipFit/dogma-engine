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
use crate::projection::ProjectedBuff;
use esf_data::Info;
use item::{Item, Object};

pub use item::EffectOperator;
pub use outgoing::beacon;
pub use output::{AttributeValue, Calculation, ItemResult, Source, SourceRef};

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
    pub projected: Vec<Projected>,
    pub buffs: Vec<ProjectedBuff>,
    pub sources: bool,
    pub reactive_armor: Option<[f64; 4]>,
}

impl Objects {
    fn get(&self, object: Object) -> Option<&Item> {
        match object {
            Object::Ship => Some(&self.ship),
            Object::Mode => self.mode.as_ref(),
            Object::Char => Some(&self.char),
            Object::Projected(index) => Some(&self.projected[index].item),
            Object::Item(index) => Some(&self.items[index]),
            Object::Charge(index) => self.items[index].charge.as_deref(),
            Object::Skill(index) => Some(&self.skills[index]),
        }
    }

    fn get_mut(&mut self, object: Object) -> Option<&mut Item> {
        match object {
            Object::Ship => Some(&mut self.ship),
            Object::Mode => self.mode.as_mut(),
            Object::Char => Some(&mut self.char),
            Object::Projected(index) => Some(&mut self.projected[index].item),
            Object::Item(index) => Some(&mut self.items[index]),
            Object::Charge(index) => self.items[index].charge.as_deref_mut(),
            Object::Skill(index) => Some(&mut self.skills[index]),
        }
    }

    pub fn new(ship_type_id: i32, mode_type_id: Option<i32>) -> Objects {
        Objects {
            ship: Item::new_fake(ship_type_id),
            mode: mode_type_id.map(Item::new_fake),
            items: Vec::new(),
            skills: Vec::new(),
            char: Item::new_fake(1373),
            projected: Vec::new(),
            buffs: Vec::new(),
            sources: false,
            reactive_armor: None,
        }
    }
}

/// One effect aimed at the fit, and the source it reads its strength off.
#[derive(Debug)]
pub(crate) struct Projected {
    pub effect_id: i32,
    pub item: Item,
}

trait Pass {
    fn pass(info: &impl Info, objects: &mut Objects);
}

/// Calculate every attribute of the ship, its items and the character.
pub fn calculate(info: &impl Info, fit: &Fit, options: &Options) -> Calculation {
    let calculation = calculate_once(info, fit, options);

    /* A burst reaches the whole fleet, and the ship running it is part of that
     * fleet. How strong it is only shows once calculated, so the fit is done
     * over with its own buffs handed back to it. */
    if calculation.outgoing.buffs.is_empty() {
        return calculation;
    }

    let mut fit = fit.clone();
    fit.incoming.buffs.extend(calculation.outgoing.buffs);

    calculate_once(info, &fit, options)
}

fn calculate_once(info: &impl Info, fit: &Fit, options: &Options) -> Calculation {
    let mut objects = pass_1::PassOne::pass(info, fit);
    objects.sources = options.sources;

    pass_2::PassTwo::pass(info, &mut objects);
    pass_3::PassThree::pass(info, &mut objects);
    pass_4::PassFour::pass(info, &mut objects);

    Calculation::new(info, &objects)
}
