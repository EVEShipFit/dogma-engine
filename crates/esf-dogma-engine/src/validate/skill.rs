use esf_data::Info;

use super::{Context, ItemResult, Rule, Target, Violation};

/// A skill an item asks for, as the pair of attributes naming it and the
/// level it wants.
const REQUIRED: [(&str, &str); 6] = [
    ("requiredSkill1", "requiredSkill1Level"),
    ("requiredSkill2", "requiredSkill2Level"),
    ("requiredSkill3", "requiredSkill3Level"),
    ("requiredSkill4", "requiredSkill4Level"),
    ("requiredSkill5", "requiredSkill5Level"),
    ("requiredSkill6", "requiredSkill6Level"),
];

pub(super) fn validate<I: Info>(context: &Context<'_, I>, found: &mut Vec<Violation>) {
    let required: Vec<(i32, i32)> = REQUIRED
        .iter()
        .filter_map(|(skill, level)| {
            Some((context.attribute_id(skill)?, context.attribute_id(level)?))
        })
        .collect();

    report(context, &required, context.ship(), Target::Ship, found);

    for item in &context.items {
        if !item.is_used() {
            continue;
        }

        let target = Target::Item { index: item.index };
        report(context, &required, item.result, target, found);

        if let Some(charge) = &item.result.charge {
            let target = Target::Charge { index: item.index };
            report(context, &required, charge, target, found);
        }
    }
}

fn report<I: Info>(
    context: &Context<'_, I>,
    required: &[(i32, i32)],
    result: &ItemResult,
    target: Target,
    found: &mut Vec<Violation>,
) {
    for (skill, level) in required {
        let Some(type_id) = context.value(result, *skill) else {
            continue;
        };
        let type_id = type_id as i32;

        let wanted = context.value(result, *level).unwrap_or(0.0) as u8;
        let trained = context
            .fit
            .character
            .skills
            .get(&type_id)
            .copied()
            .unwrap_or(0);

        if trained < wanted {
            found.push(Violation {
                target,
                rule: Rule::Skill {
                    type_id,
                    required: wanted,
                    level: trained,
                },
            });
        }
    }
}
