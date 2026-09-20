use strum::IntoEnumIterator;

use super::item::{Attribute, EffectOperator, Item, Origin};
use super::output::{Source, SourceRef};
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
    /* Paired with the index into the recorded sources, to fill in the penalty once sorted. */
    positive: Vec<(f64, Option<usize>)>,
    negative: Vec<(f64, Option<usize>)>,
}

fn apply_penalized(
    mut current_value: f64,
    values: &mut [(f64, Option<usize>)],
    sources: &mut [Source],
) -> f64 {
    /* The highest absolute value goes first. */
    values.sort_by(|x, y| y.0.abs().partial_cmp(&x.0.abs()).unwrap());

    for (position, (value, source_index)) in values.iter().enumerate() {
        let penalty = PENALTY_FACTOR.powi(position.pow(2) as i32);
        current_value *= 1.0 + value * penalty;

        if let Some(source_index) = source_index {
            sources[*source_index].penalty = Some(penalty);
        }
    }

    current_value
}

impl Attribute {
    fn calculate_value(&self, info: &impl Info, objects: &Objects, attribute_id: i32) -> f64 {
        if let Some(value) = self.value.get() {
            return value;
        }

        let mut current_value = self.base_value;
        let mut sources = Vec::new();

        for operator in EffectOperator::iter() {
            let mut values = Values::default();

            /* Collect all the values for this operator. */
            for effect in &self.effects {
                if effect.operator != operator {
                    continue;
                }

                /* A buff has already won, so it always applies, and it carries
                 * its own strength rather than reading one off an object. */
                let (from, source_value, applied) = match effect.origin {
                    Origin::Buff { buff_id, value } => {
                        (SourceRef::Buff { id: buff_id }, value, true)
                    }
                    Origin::Effect {
                        source,
                        source_category,
                        attribute_id,
                        ..
                    } => {
                        let Some(item) = objects.get(source) else {
                            continue;
                        };

                        let applied = source_category.runs_at(item.state);
                        if !applied && !objects.sources {
                            continue;
                        }

                        let value = match item.attributes.get(&attribute_id) {
                            Some(attribute) => {
                                attribute.calculate_value(info, objects, attribute_id)
                            }
                            None => info
                                .get_dogma_attribute(attribute_id)
                                .map_or(0.0, |dogma_attribute| {
                                    dogma_attribute.default_value() as f64
                                }),
                        };

                        (SourceRef::new(source, item.type_id), value, applied)
                    }
                };

                let to_source = |quantity, penalty| Source {
                    from,
                    effect_id: effect.origin.effect_id(),
                    source_attribute_id: effect.origin.source_attribute_id(),
                    operator,
                    value: source_value,
                    quantity,
                    penalty,
                    applied,
                };

                if !applied {
                    sources.push(to_source(effect.quantity, None));
                    continue;
                }

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

                if !effect.penalty || !OPERATOR_HAS_PENALTY.contains(&effect.operator) {
                    values
                        .unpenalized
                        .extend(std::iter::repeat_n(source_value, effect.quantity as usize));
                    if objects.sources {
                        sources.push(to_source(effect.quantity, None));
                    }
                    continue;
                }

                /* Check whether stacking penalty counts; negative and positive values have their own penalty. */
                let bucket = if source_value < 0.0 {
                    &mut values.negative
                } else {
                    &mut values.positive
                };
                for _ in 0..effect.quantity {
                    let source_index = objects.sources.then(|| {
                        sources.push(to_source(1, None));
                        sources.len() - 1
                    });
                    bucket.push((source_value, source_index));
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

                    current_value =
                        apply_penalized(current_value, &mut values.positive, &mut sources);
                    current_value =
                        apply_penalized(current_value, &mut values.negative, &mut sources);
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
        if objects.sources {
            self.sources.replace(sources);
        }
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
        if let Some(mode) = &objects.mode {
            mode.calculate_values(info, objects);
        }
        objects.char.calculate_values(info, objects);
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
        for beacon in &objects.beacons {
            beacon.calculate_values(info, objects);
        }
    }
}
