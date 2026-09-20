use std::collections::HashMap;

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
    for (type_id, wanted) in wanted_skills(context, required, result) {
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

/// Check what skills are required, and walk the whole tree of skills depending
/// on skills to find if any are missing.
fn wanted_skills<I: Info>(
    context: &Context<'_, I>,
    required: &[(i32, i32)],
    result: &ItemResult,
) -> Vec<(i32, u8)> {
    let mut levels: HashMap<i32, u8> = HashMap::new();
    let mut order: Vec<i32> = Vec::new();

    for (skill, level) in required {
        if let Some(type_id) = context.value(result, *skill) {
            let level = context.value(result, *level).unwrap_or(0.0) as u8;
            note(&mut levels, &mut order, type_id as i32, level);
        }
    }

    /* `order` grows while it is walked; a skill is added once, so this ends
     * even where skills point at one another. */
    let mut next = 0;
    while next < order.len() {
        let type_id = order[next];
        next += 1;

        for (skill, level) in required {
            if let Some(prerequisite) = context.base_value(type_id, *skill) {
                let level = context.base_value(type_id, *level).unwrap_or(0.0) as u8;
                note(&mut levels, &mut order, prerequisite as i32, level);
            }
        }
    }

    order
        .into_iter()
        .map(|type_id| (type_id, levels[&type_id]))
        .collect()
}

/// Keep the highest level asked of a skill, in the order it was first met.
fn note(levels: &mut HashMap<i32, u8>, order: &mut Vec<i32>, type_id: i32, level: u8) {
    match levels.get_mut(&type_id) {
        Some(known) => *known = (*known).max(level),
        None => {
            levels.insert(type_id, level);
            order.push(type_id);
        }
    }
}
