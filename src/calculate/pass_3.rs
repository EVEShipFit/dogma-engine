use std::collections::BTreeMap;
use strum::IntoEnumIterator;

use super::item::{Attribute, EffectCategory, EffectOperator, Item, Object};
use super::{Info, Pass, Ship};

/* Penalty factor: 1 / math.exp((1 / 2.67) ** 2) */
const PENALTY_FACTOR: f32 = 0.8691199808003974;

const OPERATOR_HAS_PENALTY: [EffectOperator; 5] = [
    EffectOperator::PreMul,
    EffectOperator::PostMul,
    EffectOperator::PostPercent,
    EffectOperator::PreDiv,
    EffectOperator::PostDiv,
];

pub struct PassThree {}

struct Cache {
    hull: BTreeMap<i32, f32>,
    items: BTreeMap<usize, BTreeMap<i32, f32>>,
    skills: BTreeMap<usize, BTreeMap<i32, f32>>,
}

impl Attribute {
    fn calculate_value(
        &self,
        info: &Info,
        ship: &Ship,
        categories: &Vec<EffectCategory>,
        cache: &mut Cache,
        item: Object,
        attribute_id: i32,
    ) -> f32 {
        if self.value.is_some() {
            return self.value.unwrap();
        }
        let cache_value = match item {
            Object::Ship => cache.hull.get(&attribute_id),
            Object::Item(index) => cache.items.get(&index).and_then(|x| x.get(&attribute_id)),
            Object::Skill(index) => cache.skills.get(&index).and_then(|x| x.get(&attribute_id)),
            _ => None,
        };
        if cache_value.is_some() {
            return *cache_value.unwrap();
        }

        let mut current_value = self.base_value;

        for operator in EffectOperator::iter() {
            let mut values = (Vec::new(), Vec::new(), Vec::new());

            /* Collect all the values for this operator. */
            for effect in &self.effects {
                if effect.operator != operator {
                    continue;
                }
                if !categories.contains(&effect.source_category) {
                    continue;
                }

                let source = match effect.source {
                    Object::Ship => &ship.hull,
                    Object::Item(index) => &ship.items[index],
                    Object::Skill(index) => &ship.skills[index],
                    _ => panic!("Unknown source object"),
                };

                let source_value = match source.attributes.get(&effect.source_attribute_id) {
                    Some(attribute) => attribute.calculate_value(
                        info,
                        ship,
                        categories,
                        cache,
                        effect.source,
                        effect.source_attribute_id,
                    ),
                    None => {
                        let dogma_attribute = info.get_dogma_attribute(effect.source_attribute_id);
                        dogma_attribute.defaultValue
                    }
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
                    EffectOperator::PostAssignment => source_value,
                };

                /* Check whether stacking penalty counts; negative and positive values have their own penalty. */
                if effect.penalty && OPERATOR_HAS_PENALTY.contains(&effect.operator) {
                    if source_value < 0.0 {
                        values.2.push(source_value);
                    } else {
                        values.1.push(source_value);
                    }
                } else {
                    values.0.push(source_value);
                }
            }

            if values.0.is_empty() && values.1.is_empty() && values.2.is_empty() {
                continue;
            }

            /* Apply the operator on the values. */
            match operator {
                EffectOperator::PreAssign | EffectOperator::PostAssignment => {
                    let dogma_attribute = info.get_dogma_attribute(attribute_id);

                    current_value = if dogma_attribute.highIsGood {
                        *values
                            .0
                            .iter()
                            .max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
                            .unwrap()
                    } else {
                        *values
                            .0
                            .iter()
                            .min_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
                            .unwrap()
                    };

                    assert!(values.1.is_empty());
                    assert!(values.2.is_empty());
                }

                EffectOperator::PreMul
                | EffectOperator::PreDiv
                | EffectOperator::PostMul
                | EffectOperator::PostDiv
                | EffectOperator::PostPercent => {
                    for value in values.0 {
                        current_value *= 1.0 + value;
                    }

                    /* Sort values.1 (positive values) based on value; highest value gets lowest penalty. */
                    values
                        .1
                        .sort_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap());
                    /* Sort values.2 (negative values) based on value; lowest value gets lowest penalty. */
                    values
                        .2
                        .sort_by(|x, y| y.abs().partial_cmp(&x.abs()).unwrap());

                    /* Apply positive stacking penalty. */
                    for (index, value) in values.1.iter().enumerate() {
                        current_value *= 1.0 + value * PENALTY_FACTOR.powi(index.pow(2) as i32);
                    }
                    /* Apply negative stacking penalty. */
                    for (index, value) in values.2.iter().enumerate() {
                        current_value *= 1.0 + value * PENALTY_FACTOR.powi(index.pow(2) as i32);
                    }
                }

                EffectOperator::ModAdd | EffectOperator::ModSub => {
                    for value in values.0 {
                        current_value += value;
                    }

                    assert!(values.1.is_empty());
                    assert!(values.2.is_empty());
                }
            }
        }

        match item {
            Object::Ship => {
                cache.hull.insert(attribute_id, current_value);
            }
            Object::Item(index) => {
                if !cache.items.contains_key(&index) {
                    cache.items.insert(index, BTreeMap::new());
                }
                cache
                    .items
                    .get_mut(&index)
                    .unwrap()
                    .insert(attribute_id, current_value);
            }
            Object::Skill(index) => {
                if !cache.skills.contains_key(&index) {
                    cache.skills.insert(index, BTreeMap::new());
                }
                cache
                    .skills
                    .get_mut(&index)
                    .unwrap()
                    .insert(attribute_id, current_value);
            }
            _ => {}
        }

        current_value
    }
}

impl Item {
    fn calculate_values(
        &self,
        info: &Info,
        ship: &Ship,
        categories: &Vec<EffectCategory>,
        cache: &mut Cache,
        item: Object,
    ) {
        for attribute_id in self.attributes.keys() {
            self.attributes[&attribute_id].calculate_value(
                info,
                ship,
                &categories,
                cache,
                item,
                *attribute_id,
            );
        }
    }

    fn store_cached_values(&mut self, info: &Info, cache: &BTreeMap<i32, f32>) {
        for (attribute_id, value) in cache {
            if let Some(attribute) = self.attributes.get_mut(&attribute_id) {
                attribute.value = Some(*value);
            } else {
                let dogma_attribute = info.get_dogma_attribute(*attribute_id);

                let mut attribute = Attribute::new(dogma_attribute.defaultValue);
                attribute.value = Some(*value);

                self.attributes.insert(*attribute_id, attribute);
            }
        }
    }
}

impl Pass for PassThree {
    fn pass(info: &Info, ship: &mut Ship) {
        let categories = vec![
            EffectCategory::Passive,
            EffectCategory::Active,
            EffectCategory::Online,
        ];

        let mut cache = Cache {
            hull: BTreeMap::new(),
            items: BTreeMap::new(),
            skills: BTreeMap::new(),
        };

        ship.hull
            .calculate_values(info, ship, &categories, &mut cache, Object::Ship);
        for (index, item) in ship.items.iter().enumerate() {
            item.calculate_values(info, ship, &categories, &mut cache, Object::Item(index));
        }
        /* No need to calculate skills; recursively they will resolve what is needed. */

        ship.hull.store_cached_values(info, &cache.hull);
        for (index, item) in ship.items.iter_mut().enumerate() {
            item.store_cached_values(info, &cache.items[&index]);
        }
    }
}
