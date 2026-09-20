use std::collections::{BTreeMap, BTreeSet};

use esf_data::{Info, eve};

use super::Objects;
use super::attribute_ids::{ATTRIBUTE_REMOTE_RESISTANCE_ID, ATTRIBUTE_WARFARE_BUFFS};
use super::item::Item;
use super::pass_2::get_effect_category;
use crate::projection::{ProjectedBuff, ProjectedEffect, Projection};

/// What a beacon in space hands to every fit in there with it.
///
/// A beacon has no fit of its own, so nothing about it is calculated; it is
/// the type as the SDE has it. Put the result in
/// [`Fit::incoming`](crate::Fit::incoming).
pub fn beacon(info: &impl Info, type_id: i32) -> Projection {
    let mut item = Item::new_projected(type_id);
    item.set_attributes(info);

    /* A beacon aims everything it has: there is no ship under it to keep. */
    collect(info, &item, |_| true)
}

/* What the fit hands to others: the effects with a target domain, and the
 * buffs its modules offer. Both only count while the item is running. */
pub(super) fn outgoing(info: &impl Info, objects: &Objects) -> Projection {
    let mut projection = Projection::default();

    let charges = objects
        .items
        .iter()
        .filter_map(|item| item.charge.as_deref());
    for item in [&objects.ship]
        .into_iter()
        .chain(objects.mode.as_ref())
        .chain(objects.items.iter())
        .chain(charges)
    {
        projection.extend(collect(info, item, aims_at_target));
    }

    projection
}

fn aims_at_target(effect: &eve::DogmaEffect) -> bool {
    effect.modifiers().into_iter().flatten().any(|modifier| {
        matches!(
            modifier.domain(),
            eve::ModifierDomain::Target | eve::ModifierDomain::TargetID
        )
    })
}

fn collect(info: &impl Info, item: &Item, aims: impl Fn(&eve::DogmaEffect) -> bool) -> Projection {
    let mut projection = Projection::default();

    for (id, value) in
        ATTRIBUTE_WARFARE_BUFFS.map(|(id, value)| (item.value_of(id) as i32, item.value_of(value)))
    {
        if id != 0 && value != 0.0 {
            projection.buffs.push(ProjectedBuff { id, value });
        }
    }

    for type_dogma_effect in info.get_dogma_effects(item.type_id).into_iter().flatten() {
        let effect_id = type_dogma_effect.effect_id();
        let Some(effect) = info.get_dogma_effect(effect_id) else {
            continue;
        };
        if !get_effect_category(effect.effect_category()).runs_at(item.state) || !aims(&effect) {
            continue;
        }

        let projected = ProjectedEffect {
            type_id: item.type_id,
            effect_id,
            attributes: attributes_read(item, &effect),
        };

        projection
            .effects
            .extend(std::iter::repeat_n(projected, item.quantity as usize));
    }

    projection
}

/* Everything relevant to calculate the effect's strength:
 * - The values of all attributes involved in the effect.
 * - The values of range/fallof (if set).
 * - The value of the resistance attribute. */
fn attributes_read(item: &Item, effect: &eve::DogmaEffect) -> BTreeMap<i32, f64> {
    let mut attribute_ids: BTreeSet<i32> = effect
        .modifiers()
        .into_iter()
        .flatten()
        .map(|modifier| modifier.modifying_attribute_id())
        .collect();
    attribute_ids.insert(effect.range_attribute_id());
    attribute_ids.insert(effect.falloff_attribute_id());
    attribute_ids.insert(ATTRIBUTE_REMOTE_RESISTANCE_ID);

    attribute_ids
        .into_iter()
        .filter(|attribute_id| item.attributes.contains_key(attribute_id))
        .map(|attribute_id| (attribute_id, item.value_of(attribute_id)))
        .collect()
}
