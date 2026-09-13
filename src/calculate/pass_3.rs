use strum::IntoEnumIterator;

use super::item::{Attribute, EffectOperator, Item};
use super::{Info, Pass, Ship};
use crate::fit::Fit;

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

impl Attribute {
    fn calculate_value(&self, info: &impl Info, ship: &Ship, attribute_id: i32) -> f64 {
        if let Some(value) = self.value.get() {
            return value;
        }

        let mut current_value = self.base_value;

        for operator in EffectOperator::iter() {
            let mut values = Values::default();

            /* Collect all the values for this operator. */
            for effect in &self.effects {
                if effect.operator != operator {
                    continue;
                }

                let Some(source) = ship.get(effect.source) else {
                    continue;
                };

                if effect.source_category > source.state {
                    continue;
                }

                let source_value = match source.attributes.get(&effect.source_attribute_id) {
                    Some(attribute) => {
                        attribute.calculate_value(info, ship, effect.source_attribute_id)
                    }
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
                let bucket = if !effect.penalty || !OPERATOR_HAS_PENALTY.contains(&effect.operator)
                {
                    &mut values.unpenalized
                } else if source_value < 0.0 {
                    &mut values.negative
                } else {
                    &mut values.positive
                };
                bucket.extend(std::iter::repeat_n(source_value, effect.quantity as usize));
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

        self.value.set(Some(current_value));
        current_value
    }
}

impl Item {
    fn calculate_values(&self, info: &impl Info, ship: &Ship) {
        for (attribute_id, attribute) in &self.attributes {
            attribute.calculate_value(info, ship, *attribute_id);
        }
    }
}

impl Pass for PassThree {
    fn pass(info: &impl Info, _fit: &Fit, ship: &mut Ship) {
        ship.hull.calculate_values(info, ship);
        ship.char.calculate_values(info, ship);
        ship.structure.calculate_values(info, ship);
        ship.target.calculate_values(info, ship);
        for item in &ship.items {
            item.calculate_values(info, ship);
            if let Some(charge) = &item.charge {
                charge.calculate_values(info, ship);
            }
        }
        for skill in &ship.skills {
            skill.calculate_values(info, ship);
        }
    }
}
