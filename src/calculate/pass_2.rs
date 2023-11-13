use crate::data_types::{DogmaEffectModifierInfoDomain, DogmaEffectModifierInfoFunc};

use super::item::{Effect, EffectCategory, EffectOperator, Item, Object};
use super::{Info, Pass, Ship};

/** AttributeIDs for requiredSkill1, requiredSkill2, .. */
const ATTRIBUTE_SKILLS: [i32; 6] = [182, 183, 184, 1285, 1289, 1290];
/** Categories of the effect source which are exempt of stacking penalty.
 * Ship (6), Charge (8), Skill (16), Implant (20) and Subsystem (32) */
const EXEMPT_PENALTY_CATEGORY_IDS: [i32; 5] = [6, 8, 16, 20, 32];

pub struct PassTwo {}

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
    modifier: Modifier,
    operator: EffectOperator,
    source: Object,
    source_category: EffectCategory,
    source_attribute_id: i32,
    target: Object,
    target_attribute_id: i32,
}

fn get_modifier_func(
    func: DogmaEffectModifierInfoFunc,
    skill_type_id: Option<i32>,
    group_id: Option<i32>,
) -> Modifier {
    match func {
        DogmaEffectModifierInfoFunc::LocationRequiredSkillModifier => {
            Modifier::LocationRequiredSkillModifier(skill_type_id.unwrap())
        }
        DogmaEffectModifierInfoFunc::LocationGroupModifier => {
            Modifier::LocationGroupModifier(group_id.unwrap())
        }
        DogmaEffectModifierInfoFunc::LocationModifier => Modifier::LocationModifier(),
        DogmaEffectModifierInfoFunc::ItemModifier => Modifier::ItemModifier(),
        DogmaEffectModifierInfoFunc::OwnerRequiredSkillModifier => {
            Modifier::OwnerRequiredSkillModifier(skill_type_id.unwrap())
        }
        _ => panic!("Unknown modifier function: {:?}", func),
    }
}

fn get_target_object(domain: DogmaEffectModifierInfoDomain, origin: Object) -> Object {
    match domain {
        DogmaEffectModifierInfoDomain::ShipID => Object::Ship,
        DogmaEffectModifierInfoDomain::CharID => Object::Char,
        DogmaEffectModifierInfoDomain::OtherID => Object::Char, // TODO -- This is incorrect
        DogmaEffectModifierInfoDomain::StructureID => Object::Structure,
        DogmaEffectModifierInfoDomain::ItemID => origin,
        _ => panic!("Unknown domain: {:?}", domain),
    }
}

fn get_effect_category(category: i32) -> EffectCategory {
    match category {
        0 => EffectCategory::Passive,
        1 => EffectCategory::Active,
        2 => EffectCategory::Target,
        3 => EffectCategory::Area,
        4 => EffectCategory::Online,
        5 => EffectCategory::Overload,
        6 => EffectCategory::Dungeon,
        7 => EffectCategory::System,
        _ => panic!("Unknown effect category: {}", category),
    }
}

fn get_effect_operator(operation: i32) -> EffectOperator {
    match operation {
        -1 => EffectOperator::PreAssign,
        0 => EffectOperator::PreMul,
        1 => EffectOperator::PreDiv,
        2 => EffectOperator::ModAdd,
        3 => EffectOperator::ModSub,
        4 => EffectOperator::PostMul,
        5 => EffectOperator::PostDiv,
        6 => EffectOperator::PostPercent,
        7 => EffectOperator::PostAssignment,
        _ => panic!("Unknown effect operation: {}", operation),
    }
}

impl Item {
    fn add_effect(
        &mut self,
        info: &Info,
        attribute_id: i32,
        source_category_id: i32,
        effect: &Pass2Effect,
    ) {
        let attr = info.get_dogma_attribute(attribute_id);

        if !self.attributes.contains_key(&attribute_id) {
            self.set_attribute(attribute_id, attr.defaultValue);
        }

        /* Penalties are only count when an attribute is not stackable and when the item is not in the exempt category. */
        let penalty = !attr.stackable && !EXEMPT_PENALTY_CATEGORY_IDS.contains(&source_category_id);

        let attribute = self.attributes.get_mut(&attribute_id).unwrap();
        attribute.effects.push(Effect {
            operator: effect.operator,
            penalty,
            source: effect.source,
            source_category: effect.source_category,
            source_attribute_id: effect.source_attribute_id,
        });
    }

    fn collect_effects(&mut self, info: &Info, origin: Object, effects: &mut Vec<Pass2Effect>) {
        for dogma_effect in info.get_dogma_effects(self.type_id) {
            let type_dogma_effect = info.get_dogma_effect(dogma_effect.effectID);
            let category = get_effect_category(type_dogma_effect.effectCategory);

            /* Find the highest state an item can be in. */
            if category > self.max_state {
                self.max_state = category;
            }

            if !type_dogma_effect.modifierInfo.is_empty() {
                for modifier in type_dogma_effect.modifierInfo {
                    /* We ignore operator 9 (calculates Skill Level based on Skill Points; irrelevant for fits). */
                    if modifier.operation.unwrap() == 9 {
                        continue;
                    }

                    let effect_modifier =
                        get_modifier_func(modifier.func, modifier.skillTypeID, modifier.groupID);
                    let target = get_target_object(modifier.domain, origin);
                    let operator = get_effect_operator(modifier.operation.unwrap());

                    effects.push(Pass2Effect {
                        modifier: effect_modifier,
                        operator,
                        source: origin,
                        source_category: category,
                        source_attribute_id: modifier.modifyingAttributeID.unwrap(),
                        target,
                        target_attribute_id: modifier.modifiedAttributeID.unwrap(),
                    });
                }
            } else {
                self.effects.push(dogma_effect.effectID);
            }
        }

        if self.state > self.max_state {
            self.state = self.max_state;
        }
    }
}

impl Pass for PassTwo {
    fn pass(info: &Info, ship: &mut Ship) {
        let mut effects = Vec::new();

        /* Collect all the effects in a single list. */
        ship.hull.collect_effects(info, Object::Ship, &mut effects);
        for (index, item) in ship.items.iter_mut().enumerate() {
            item.collect_effects(info, Object::Item(index), &mut effects);
        }
        for (index, skill) in ship.skills.iter_mut().enumerate() {
            skill.collect_effects(info, Object::Skill(index), &mut effects);
        }

        /* Depending on the modifier, move the effects to the correct attribute. */
        for effect in effects {
            let source_type_id = match effect.source {
                Object::Ship => info.esi_fit.ship_type_id,
                Object::Item(index) => ship.items[index].type_id,
                Object::Skill(index) => ship.skills[index].type_id,
                _ => panic!("Unknown source object"),
            };
            let category_id = info.get_type_id(source_type_id).categoryID;

            match effect.modifier {
                Modifier::ItemModifier() => {
                    let target = match effect.target {
                        Object::Ship => &mut ship.hull,
                        Object::Char => &mut ship.char,
                        Object::Structure => &mut ship.structure,
                        Object::Item(index) => &mut ship.items[index],
                        Object::Skill(index) => &mut ship.skills[index],
                    };

                    target.add_effect(info, effect.target_attribute_id, category_id, &effect);
                }
                Modifier::LocationModifier() => {
                    // TODO
                }
                Modifier::LocationGroupModifier(group_id) => {
                    let type_id = info.get_type_id(ship.hull.type_id);
                    if type_id.groupID == group_id {
                        ship.hull.add_effect(
                            info,
                            effect.target_attribute_id,
                            category_id,
                            &effect,
                        );
                    }

                    for item in &mut ship.items {
                        let type_id = info.get_type_id(item.type_id);

                        if type_id.groupID == group_id {
                            item.add_effect(info, effect.target_attribute_id, category_id, &effect);
                        }
                    }
                }
                Modifier::LocationRequiredSkillModifier(skill_type_id) => {
                    for attribute_skill_id in &ATTRIBUTE_SKILLS {
                        if ship.hull.attributes.contains_key(attribute_skill_id)
                            && ship.hull.attributes[attribute_skill_id].base_value
                                == skill_type_id as f32
                        {
                            ship.hull.add_effect(
                                info,
                                effect.target_attribute_id,
                                category_id,
                                &effect,
                            );
                        }

                        for item in &mut ship.items {
                            if item.attributes.contains_key(attribute_skill_id)
                                && item.attributes[attribute_skill_id].base_value
                                    == skill_type_id as f32
                            {
                                item.add_effect(
                                    info,
                                    effect.target_attribute_id,
                                    category_id,
                                    &effect,
                                );
                            }
                        }
                    }
                }
                Modifier::OwnerRequiredSkillModifier(_skill_type_id) => {
                    // TODO
                }
            }
        }
    }
}
