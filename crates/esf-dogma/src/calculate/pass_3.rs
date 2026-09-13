use strum::IntoEnumIterator;

use super::item::{Attribute, EffectOperator, Item};
use super::{Info, Objects, Pass};

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

fn apply_penalized(mut current_value: f64, values: &mut [f64]) -> f64 {
    /* The highest absolute value goes first. */
    values.sort_by(|x, y| y.abs().partial_cmp(&x.abs()).unwrap());

    for (position, value) in values.iter().enumerate() {
        current_value *= 1.0 + value * PENALTY_FACTOR.powi(position.pow(2) as i32);
    }

    current_value
}

impl Attribute {
    fn calculate_value(&self, info: &impl Info, objects: &Objects, attribute_id: i32) -> f64 {
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

                let Some(source) = objects.get(effect.source) else {
                    continue;
                };

                if effect.source_category > source.state {
                    continue;
                }

                let source_value = match source.attributes.get(&effect.source_attribute_id) {
                    Some(attribute) => {
                        attribute.calculate_value(info, objects, effect.source_attribute_id)
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

                    current_value = apply_penalized(current_value, &mut values.positive);
                    current_value = apply_penalized(current_value, &mut values.negative);
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
    fn calculate_values(&self, info: &impl Info, objects: &Objects) {
        for (attribute_id, attribute) in &self.attributes {
            attribute.calculate_value(info, objects, *attribute_id);
        }
    }
}

impl Pass for PassThree {
    fn pass(info: &impl Info, objects: &mut Objects) {
        objects.ship.calculate_values(info, objects);
        objects.char.calculate_values(info, objects);
        objects.structure.calculate_values(info, objects);
        objects.target.calculate_values(info, objects);
        for item in &objects.items {
            item.calculate_values(info, objects);
            if let Some(charge) = &item.charge {
                charge.calculate_values(info, objects);
            }
        }
        for skill in &objects.skills {
            skill.calculate_values(info, objects);
        }
    }
}
