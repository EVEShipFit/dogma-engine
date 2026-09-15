use std::collections::BTreeSet;

use esf_data::eve;

use super::attribute_ids::{ATTRIBUTE_CAPACITOR_NEED_ID, ATTRIBUTE_SKILLS};
use super::item::{Attribute, Effect, EffectCategory, EffectOperator, Item, Object};
use super::{Info, Objects, Pass};

/** Categories of the effect source which are exempt of stacking penalty.
 * Ship (6), Charge (8), Skill (16), Implant (20), Subsystem (32) and Structure (65) */
const EXEMPT_PENALTY_CATEGORY_IDS: [i32; 6] = [6, 8, 16, 20, 32, 65];

pub struct PassTwo {}

/* Variant names mirror the dogma modifier names used by EVE. */
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
enum Modifier {
    LocationRequiredSkillModifier(i32),
    LocationGroupModifier(i32),
    LocationModifier(),
    OwnerRequiredSkillModifier(i32),
    ItemModifier(),
}

#[derive(Debug)]
struct Pass2Effect {
    effect_id: i32,
    modifier: Modifier,
    operator: EffectOperator,
    source: Object,
    source_category: EffectCategory,
    source_attribute_id: i32,
    source_quantity: u32,
    target: Object,
    target_attribute_id: i32,
}

fn get_modifier_func(
    func: eve::ModifierFunc,
    skill_type_id: i32,
    group_id: i32,
) -> Option<Modifier> {
    match func {
        eve::ModifierFunc::LocationRequiredSkillModifier => {
            Some(Modifier::LocationRequiredSkillModifier(skill_type_id))
        }
        eve::ModifierFunc::LocationGroupModifier => Some(Modifier::LocationGroupModifier(group_id)),
        eve::ModifierFunc::LocationModifier => Some(Modifier::LocationModifier()),
        eve::ModifierFunc::ItemModifier => Some(Modifier::ItemModifier()),
        eve::ModifierFunc::OwnerRequiredSkillModifier => {
            Some(Modifier::OwnerRequiredSkillModifier(skill_type_id))
        }
        /* EffectStopper has no effect on the attributes; just on what you can bring online. */
        eve::ModifierFunc::EffectStopper => None,
        func => panic!("Unknown modifier func: {:?}", func),
    }
}

const STRUCTURE_CATEGORY_ID: i32 = 65;

fn get_target_object(domain: eve::ModifierDomain, origin: Object) -> Object {
    match domain {
        eve::ModifierDomain::ShipID => Object::Ship,
        eve::ModifierDomain::CharID => Object::Char,
        eve::ModifierDomain::OtherID => match origin {
            Object::Item(index) => Object::Charge(index),
            Object::Charge(index) => Object::Item(index),
            _ => panic!("Invalid origin for OtherID domain"),
        },
        /* On a structure fit the hull is the structure. */
        eve::ModifierDomain::StructureID => Object::Ship,
        eve::ModifierDomain::ItemID => origin,
        eve::ModifierDomain::TargetID => Object::Target,
        eve::ModifierDomain::Target => Object::Target,
        domain => panic!("Unknown modifier domain: {:?}", domain),
    }
}

fn for_each_in_location(
    objects: &mut Objects,
    location: Object,
    mut apply: impl FnMut(Object, &mut Item),
) {
    match location {
        Object::Ship => {
            apply(Object::Ship, &mut objects.ship);

            for (index, item) in objects.items.iter_mut().enumerate() {
                if !item.is_in_ship() {
                    continue;
                }

                apply(Object::Item(index), item);

                if let Some(charge) = &mut item.charge {
                    apply(Object::Charge(index), charge);
                }
            }
        }
        Object::Char => {
            apply(Object::Char, &mut objects.char);
            for (index, skill) in objects.skills.iter_mut().enumerate() {
                apply(Object::Skill(index), skill);
            }
            for (index, item) in objects.items.iter_mut().enumerate() {
                if item.is_on_char() {
                    apply(Object::Item(index), item);
                }
            }
        }
        Object::Mode | Object::Item(_) | Object::Charge(_) | Object::Skill(_) | Object::Target => {
            apply(location, objects.get_mut(location).unwrap())
        }
    }
}

fn get_effect_category(category: eve::EffectCategory) -> EffectCategory {
    match category {
        eve::EffectCategory::Passive => EffectCategory::Passive,
        eve::EffectCategory::Active => EffectCategory::Active,
        eve::EffectCategory::Target => EffectCategory::Target,
        eve::EffectCategory::Area => EffectCategory::Area,
        eve::EffectCategory::Online => EffectCategory::Online,
        eve::EffectCategory::Overload => EffectCategory::Overload,
        eve::EffectCategory::Dungeon => EffectCategory::Dungeon,
        eve::EffectCategory::System => EffectCategory::System,
        category => panic!("Unknown effect category: {:?}", category),
    }
}

fn get_effect_operator(operation: eve::ModifierOperation) -> Option<EffectOperator> {
    match operation {
        eve::ModifierOperation::PreAssign => Some(EffectOperator::PreAssign),
        eve::ModifierOperation::PreMul => Some(EffectOperator::PreMul),
        eve::ModifierOperation::PreDiv => Some(EffectOperator::PreDiv),
        eve::ModifierOperation::ModAdd => Some(EffectOperator::ModAdd),
        eve::ModifierOperation::ModSub => Some(EffectOperator::ModSub),
        eve::ModifierOperation::PostMul => Some(EffectOperator::PostMul),
        eve::ModifierOperation::PostDiv => Some(EffectOperator::PostDiv),
        eve::ModifierOperation::PostPercent => Some(EffectOperator::PostPercent),
        eve::ModifierOperation::PostAssign => Some(EffectOperator::PostAssign),
        /* Operator 9 calculates Skill Level based on Skill Points; irrelevant
         * for fits. It has no name in the schema, so match on the raw value. */
        eve::ModifierOperation(9) => None,
        eve::ModifierOperation::Unset => panic!("Effect operation is not set"),
        operation => panic!("Unknown effect operation: {:?}", operation),
    }
}

impl Item {
    fn required_skills(&self) -> BTreeSet<i32> {
        ATTRIBUTE_SKILLS
            .iter()
            .filter_map(|attribute_skill_id| self.attributes.get(attribute_skill_id))
            .map(|attribute| attribute.base_value as i32)
            .collect()
    }

    fn add_effect(
        &mut self,
        info: &impl Info,
        target: Object,
        attribute_id: i32,
        source_category_id: i32,
        effect: &Pass2Effect,
    ) {
        let attr = info.get_dogma_attribute(attribute_id);

        /* Penalties are only count when an attribute is not stackable and when the item is not in the exempt category. */
        let stackable = attr.is_some_and(|attr| attr.stackable());
        let penalty = !stackable && !EXEMPT_PENALTY_CATEGORY_IDS.contains(&source_category_id);

        let attribute = self.attributes.entry(attribute_id).or_insert_with(|| {
            Attribute::new(attr.map_or(0.0, |attr| attr.default_value() as f64))
        });
        attribute.effects.push(Effect {
            effect_id: effect.effect_id,
            operator: effect.operator,
            penalty,
            source: effect.source,
            source_category: effect.source_category,
            source_attribute_id: effect.source_attribute_id,
            /* A stack bonuses others once per item, but itself only once. */
            quantity: if target == effect.source {
                1
            } else {
                effect.source_quantity
            },
        });
    }

    fn collect_effects(
        &mut self,
        info: &impl Info,
        origin: Object,
        skip_ship_domain: bool,
        effects: &mut Vec<Pass2Effect>,
    ) {
        for dogma_effect in info.get_dogma_effects(self.type_id).into_iter().flatten() {
            let Some(type_dogma_effect) = info.get_dogma_effect(dogma_effect.effect_id()) else {
                continue;
            };
            let category = get_effect_category(type_dogma_effect.effect_category());

            /* Find the highest state an item can be in. */
            if category > self.max_state && category <= EffectCategory::Overload {
                self.max_state = category;
            }

            /* Every non-passive effect of a fighter is an ability, and the fit picks which run. */
            if self.is_fighter() && category != EffectCategory::Passive {
                let used = match &self.fighter_abilities {
                    Some(abilities) => abilities.contains(&dogma_effect.effect_id()),
                    None => dogma_effect.is_default(),
                };
                if !used {
                    continue;
                }
            }

            if type_dogma_effect.fitting_usage_chance_attribute_id() != 0
                && !self
                    .booster_side_effects
                    .contains(&dogma_effect.effect_id())
            {
                continue;
            }

            let modifiers = type_dogma_effect
                .modifiers()
                .filter(|modifiers| !modifiers.is_empty());

            let Some(modifiers) = modifiers else {
                self.effects.push(dogma_effect.effect_id());
                continue;
            };

            for modifier in modifiers {
                let effect_modifier = get_modifier_func(
                    modifier.func(),
                    modifier.skill_type_id(),
                    modifier.group_id(),
                );
                let Some(effect_modifier) = effect_modifier else {
                    continue;
                };

                let Some(operator) = get_effect_operator(modifier.operation()) else {
                    continue;
                };

                /* If the origin is an Item(), the domain is OtherID, but there is no charge, skip the effect. */
                if matches!(origin, Object::Item(_))
                    && modifier.domain() == eve::ModifierDomain::OtherID
                    && self.charge.is_none()
                {
                    continue;
                }

                if skip_ship_domain && modifier.domain() == eve::ModifierDomain::ShipID {
                    continue;
                }

                let target = get_target_object(modifier.domain(), origin);
                effects.push(Pass2Effect {
                    effect_id: dogma_effect.effect_id(),
                    modifier: effect_modifier,
                    operator,
                    source: origin,
                    source_category: category,
                    source_attribute_id: modifier.modifying_attribute_id(),
                    source_quantity: self.quantity,
                    target,
                    target_attribute_id: modifier.modified_attribute_id(),
                });
            }
        }

        /* Any module that has a capacitorNeed, can be activated. */
        if self.attributes.contains_key(&ATTRIBUTE_CAPACITOR_NEED_ID)
            && self.max_state < EffectCategory::Active
        {
            self.max_state = EffectCategory::Active;
        }

        if self.state > self.max_state {
            self.state = self.max_state;
        }
    }
}

impl Pass for PassTwo {
    fn pass(info: &impl Info, objects: &mut Objects) {
        let mut effects = Vec::new();

        /* Collect all the effects in a single list. */
        objects
            .ship
            .collect_effects(info, Object::Ship, false, &mut effects);
        if let Some(mode) = &mut objects.mode {
            mode.collect_effects(info, Object::Mode, false, &mut effects);
        }
        objects
            .char
            .collect_effects(info, Object::Char, false, &mut effects);

        /* A structure is not the pilot's ship; only the structure skills, via
         * the structure domain, reach it. Implants and boosters not at all. */
        let structure_fit = objects.ship.category_id == STRUCTURE_CATEGORY_ID;

        for (index, item) in objects.items.iter_mut().enumerate() {
            if structure_fit && item.is_on_char() {
                item.state = EffectCategory::Passive;
                continue;
            }
            if !item.is_calculated() {
                continue;
            }

            item.collect_effects(info, Object::Item(index), false, &mut effects);
            if let Some(charge) = &mut item.charge {
                charge.collect_effects(info, Object::Charge(index), false, &mut effects);
            }
        }
        for (index, skill) in objects.skills.iter_mut().enumerate() {
            skill.collect_effects(info, Object::Skill(index), structure_fit, &mut effects);
        }

        let ship_skills = objects.ship.required_skills();
        let item_skills: Vec<_> = objects
            .items
            .iter()
            .map(|item| {
                let charge_skills = item.charge.as_ref().map(|charge| charge.required_skills());
                (item.required_skills(), charge_skills.unwrap_or_default())
            })
            .collect();

        /* Depending on the modifier, move the effects to the correct attribute. */
        for effect in effects {
            let source = objects.get(effect.source).unwrap();
            let source_type_id = source.type_id;
            let category_id = source.category_id;

            match effect.modifier {
                Modifier::ItemModifier() => {
                    let target = objects.get_mut(effect.target).unwrap();

                    target.add_effect(
                        info,
                        effect.target,
                        effect.target_attribute_id,
                        category_id,
                        &effect,
                    );
                }
                Modifier::LocationModifier() => {
                    for_each_in_location(objects, effect.target, |target, item| {
                        item.add_effect(
                            info,
                            target,
                            effect.target_attribute_id,
                            category_id,
                            &effect,
                        );
                    });
                }
                Modifier::LocationGroupModifier(group_id) => {
                    for_each_in_location(objects, effect.target, |target, item| {
                        if item.group_id == group_id {
                            item.add_effect(
                                info,
                                target,
                                effect.target_attribute_id,
                                category_id,
                                &effect,
                            );
                        }
                    });
                }
                Modifier::OwnerRequiredSkillModifier(skill_type_id)
                | Modifier::LocationRequiredSkillModifier(skill_type_id) => {
                    /* Some skills apply on -1, indicating they should apply on anything that uses that skill. */
                    let skill_type_id = if skill_type_id == -1 {
                        source_type_id
                    } else {
                        skill_type_id
                    };
                    let location_only =
                        matches!(effect.modifier, Modifier::LocationRequiredSkillModifier(_));
                    let char_location = location_only && effect.target == Object::Char;
                    let in_location = |item: &Item| match char_location {
                        true => item.is_on_char(),
                        false => item.is_in_ship(),
                    };

                    if !char_location && ship_skills.contains(&skill_type_id) {
                        objects.ship.add_effect(
                            info,
                            Object::Ship,
                            effect.target_attribute_id,
                            category_id,
                            &effect,
                        );
                    }

                    for (index, (item, (skills, charge_skills))) in objects
                        .items
                        .iter_mut()
                        .zip(&item_skills)
                        .enumerate()
                        .filter(|(_, (item, _))| !location_only || in_location(item))
                    {
                        if skills.contains(&skill_type_id) {
                            item.add_effect(
                                info,
                                Object::Item(index),
                                effect.target_attribute_id,
                                category_id,
                                &effect,
                            );
                        }

                        if let Some(charge) = &mut item.charge
                            && charge_skills.contains(&skill_type_id)
                        {
                            charge.add_effect(
                                info,
                                Object::Charge(index),
                                effect.target_attribute_id,
                                category_id,
                                &effect,
                            );
                        }
                    }
                }
            }
        }
    }
}
