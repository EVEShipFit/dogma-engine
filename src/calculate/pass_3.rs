use std::collections::BTreeMap;
use strum::IntoEnumIterator;

use super::item::{Attribute, EffectOperator, Item, Object};
use super::{Info, Pass, Ship};

/* Penalty factor: 1 / math.exp((1 / 2.67) ** 2) */
const PENALTY_FACTOR: f64 = 0.8691199808003974;

const OPERATOR_HAS_PENALTY: [EffectOperator; 5] = [
    EffectOperator::PreMul,
    EffectOperator::PostMul,
    EffectOperator::PostPercent,
    EffectOperator::PreDiv,
    EffectOperator::PostDiv,
];

pub struct PassThree {}

#[derive(Default)]
struct Values {
    unpenalized: Vec<f64>,
    positive: Vec<f64>,
    negative: Vec<f64>,
}

struct Cache {
    hull: BTreeMap<i32, f64>,
    char: BTreeMap<i32, f64>,
    structure: BTreeMap<i32, f64>,
    target: BTreeMap<i32, f64>,

    items: Vec<BTreeMap<i32, f64>>,
    charge: Vec<BTreeMap<i32, f64>>,
    skills: Vec<BTreeMap<i32, f64>>,
}

impl Cache {
    fn new(ship: &Ship) -> Cache {
        Cache {
            hull: BTreeMap::new(),
            char: BTreeMap::new(),
            structure: BTreeMap::new(),
            target: BTreeMap::new(),

            items: vec![BTreeMap::new(); ship.items.len()],
            charge: vec![BTreeMap::new(); ship.items.len()],
            skills: vec![BTreeMap::new(); ship.skills.len()],
        }
    }
}

impl Attribute {
    fn calculate_value(
        &self,
        info: &impl Info,
        ship: &Ship,
        cache: &mut Cache,
        item: Object,
        attribute_id: i32,
    ) -> f64 {
        if let Some(value) = self.value {
            return value;
        }
        let cache_value = match item {
            Object::Ship => cache.hull.get(&attribute_id),
            Object::Char => cache.char.get(&attribute_id),
            Object::Structure => cache.structure.get(&attribute_id),
            Object::Target => cache.target.get(&attribute_id),
            Object::Item(index) => cache.items[index].get(&attribute_id),
            Object::Charge(index) => cache.charge[index].get(&attribute_id),
            Object::Skill(index) => cache.skills[index].get(&attribute_id),
        };
        if let Some(cache_value) = cache_value {
            return *cache_value;
        }

        let mut current_value = self.base_value;

        for operator in EffectOperator::iter() {
            let mut values = Values::default();

            /* Collect all the values for this operator. */
            for effect in &self.effects {
                if effect.operator != operator {
                    continue;
                }

                let source = match effect.source {
                    Object::Ship => &ship.hull,
                    Object::Item(index) => &ship.items[index],
                    Object::Charge(index) => match &ship.items[index].charge {
                        Some(charge) => charge,
                        None => continue,
                    },
                    Object::Skill(index) => &ship.skills[index],
                    Object::Char => &ship.char,
                    Object::Structure => &ship.structure,
                    Object::Target => &ship.target,
                };

                if effect.source_category > source.state {
                    continue;
                }

                let source_value = match source.attributes.get(&effect.source_attribute_id) {
                    Some(attribute) => attribute.calculate_value(
                        info,
                        ship,
                        cache,
                        effect.source,
                        effect.source_attribute_id,
                    ),
                    None => info
                        .get_dogma_attribute(effect.source_attribute_id)
                        .map_or(0.0, |dogma_attribute| {
                            dogma_attribute.default_value() as f64
                        }),
                };

                /* Simplify the values so we can do the math easier later on. */
                let source_value = match operator {
                    EffectOperator::PreAssign => source_value,
                    EffectOperator::PreMul => source_value - 1.0,
                    EffectOperator::PreDiv => 1.0 / source_value - 1.0,
                    EffectOperator::ModAdd => source_value,
                    EffectOperator::ModSub => -source_value,
                    EffectOperator::PostMul => source_value - 1.0,
                    EffectOperator::PostDiv => 1.0 / source_value - 1.0,
                    EffectOperator::PostPercent => source_value / 100.0,
                    EffectOperator::PostAssign => source_value,
                };

                /* Check whether stacking penalty counts; negative and positive values have their own penalty. */
                if effect.penalty && OPERATOR_HAS_PENALTY.contains(&effect.operator) {
                    if source_value < 0.0 {
                        values.negative.push(source_value);
                    } else {
                        values.positive.push(source_value);
                    }
                } else {
                    values.unpenalized.push(source_value);
                }
            }

            if values.unpenalized.is_empty()
                && values.positive.is_empty()
                && values.negative.is_empty()
            {
                continue;
            }

            /* Apply the operator on the values. */
            match operator {
                EffectOperator::PreAssign | EffectOperator::PostAssign => {
                    let high_is_good = info
                        .get_dogma_attribute(attribute_id)
                        .is_some_and(|dogma_attribute| dogma_attribute.high_is_good());

                    current_value = if high_is_good {
                        *values
                            .unpenalized
                            .iter()
                            .max_by(|x, y| x.partial_cmp(y).unwrap())
                            .unwrap()
                    } else {
                        *values
                            .unpenalized
                            .iter()
                            .min_by(|x, y| x.partial_cmp(y).unwrap())
                            .unwrap()
                    };

                    assert!(values.positive.is_empty());
                    assert!(values.negative.is_empty());
                }

                EffectOperator::PreMul
                | EffectOperator::PreDiv
                | EffectOperator::PostMul
                | EffectOperator::PostDiv
                | EffectOperator::PostPercent => {
                    for value in values.unpenalized {
                        current_value *= 1.0 + value;
                    }

                    /* For positive values, the highest number goes first. For negative values, the lowest number. */
                    let sort_func = |x: &f64, y: &f64| y.abs().partial_cmp(&x.abs()).unwrap();
                    values.positive.sort_by(sort_func);
                    values.negative.sort_by(sort_func);

                    /* Apply positive stacking penalty. */
                    for (index, value) in values.positive.iter().enumerate() {
                        current_value *= 1.0 + value * PENALTY_FACTOR.powi(index.pow(2) as i32);
                    }
                    /* Apply negative stacking penalty. */
                    for (index, value) in values.negative.iter().enumerate() {
                        current_value *= 1.0 + value * PENALTY_FACTOR.powi(index.pow(2) as i32);
                    }
                }

                EffectOperator::ModAdd | EffectOperator::ModSub => {
                    for value in values.unpenalized {
                        current_value += value;
                    }

                    assert!(values.positive.is_empty());
                    assert!(values.negative.is_empty());
                }
            }
        }

        match item {
            Object::Ship => {
                cache.hull.insert(attribute_id, current_value);
            }
            Object::Char => {
                cache.char.insert(attribute_id, current_value);
            }
            Object::Structure => {
                cache.structure.insert(attribute_id, current_value);
            }
            Object::Target => {
                cache.target.insert(attribute_id, current_value);
            }
            Object::Item(index) => {
                cache.items[index].insert(attribute_id, current_value);
            }
            Object::Charge(index) => {
                cache.charge[index].insert(attribute_id, current_value);
            }
            Object::Skill(index) => {
                cache.skills[index].insert(attribute_id, current_value);
            }
        }

        current_value
    }
}

impl Item {
    fn calculate_values(&self, info: &impl Info, ship: &Ship, cache: &mut Cache, item: Object) {
        for attribute_id in self.attributes.keys() {
            self.attributes[attribute_id].calculate_value(info, ship, cache, item, *attribute_id);
        }
    }

    fn store_cached_values(&mut self, info: &impl Info, cache: &BTreeMap<i32, f64>) {
        for (attribute_id, value) in cache {
            if let Some(attribute) = self.attributes.get_mut(attribute_id) {
                attribute.value = Some(*value);
            } else {
                let default_value = info
                    .get_dogma_attribute(*attribute_id)
                    .map_or(0.0, |dogma_attribute| {
                        dogma_attribute.default_value() as f64
                    });

                let mut attribute = Attribute::new(default_value);
                attribute.value = Some(*value);

                self.attributes.insert(*attribute_id, attribute);
            }
        }
    }
}

impl Pass for PassThree {
    fn pass(info: &impl Info, ship: &mut Ship) {
        let mut cache = Cache::new(ship);

        ship.hull
            .calculate_values(info, ship, &mut cache, Object::Ship);
        ship.char
            .calculate_values(info, ship, &mut cache, Object::Char);
        ship.structure
            .calculate_values(info, ship, &mut cache, Object::Structure);
        ship.target
            .calculate_values(info, ship, &mut cache, Object::Target);
        for (index, item) in ship.items.iter().enumerate() {
            item.calculate_values(info, ship, &mut cache, Object::Item(index));
            if let Some(charge) = &item.charge {
                charge.calculate_values(info, ship, &mut cache, Object::Charge(index));
            }
        }
        for (index, skill) in ship.skills.iter().enumerate() {
            skill.calculate_values(info, ship, &mut cache, Object::Skill(index));
        }

        ship.hull.store_cached_values(info, &cache.hull);
        ship.char.store_cached_values(info, &cache.char);
        ship.structure.store_cached_values(info, &cache.structure);
        ship.target.store_cached_values(info, &cache.target);
        for (index, item) in ship.items.iter_mut().enumerate() {
            item.store_cached_values(info, &cache.items[index]);
            if let Some(charge) = &mut item.charge {
                charge.store_cached_values(info, &cache.charge[index]);
            }
        }
        for (index, skill) in ship.skills.iter_mut().enumerate() {
            skill.store_cached_values(info, &cache.skills[index]);
        }
    }
}
