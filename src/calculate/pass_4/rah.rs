use crate::calculate::item::Object;
use crate::calculate::pass_3::Cache;
use crate::calculate::Ship;
use crate::info::Info;

const RAH_TYPE_ID: i32 = 4403;
const RESISTANCE_SHIFT_AMOUNT_ID: i32 = 1849;
const RESONANCE_IDS: [i32; 4] = [267, 268, 269, 270]; // Order: Em, Explo, Kin, Therm

pub fn adapt_rah_to_damage_pattern(info: &impl Info, ship: &mut Ship) {
    let rah_index = if let Some(i) = ship
        .items
        .iter_mut()
        .position(|item| item.type_id == RAH_TYPE_ID)
    {
        i
    } else {
        return;
    };

    let effective_resonance_ids = [
        info.attribute_name_to_id("armorEmDamageEffectiveResonance"),
        info.attribute_name_to_id("armorExplosiveDamageEffectiveResonance"),
        info.attribute_name_to_id("armorKineticDamageEffectiveResonance"),
        info.attribute_name_to_id("armorThermalDamageEffectiveResonance"),
    ];

    let rah_affected_attributes = rah_affected_attributes(ship, rah_index);
    let shift_amount = ship.items[rah_index]
        .attributes
        .get(&RESISTANCE_SHIFT_AMOUNT_ID)
        .and_then(|a| a.value)
        .unwrap()
        / 100.0;

    let mut cache = Cache::default();
    let mut cycle_list: Vec<[f64; 4]> = Vec::new();
    let mut cycle_start = None;

    'sim: for _ in 0..50 {
        let effective_resonance =
            get_effective_resonance(ship, &mut cache, effective_resonance_ids);
        let rah_resonance = get_rah_resonance(ship, &mut cache, rah_index);

        let shift = calculate_rah_shift(effective_resonance, rah_resonance, shift_amount);
        let shifted = std::array::from_fn(|i| rah_resonance[i] + shift[i]);

        for (i, val) in cycle_list.iter().enumerate() {
            if shifted.iter().zip(val).all(|(a, b)| (a - b).abs() <= 1e-6) {
                cycle_start = Some(i);
                break 'sim;
            }
        }
        cycle_list.push(shifted);

        set_rah_resonance(ship, &mut cache, rah_index, shifted);
        recalculate_values(info, ship, &mut cache, &rah_affected_attributes);
    }

    let cycle = if let Some(cycle_start) = cycle_start {
        &cycle_list[cycle_start..]
    } else {
        &cycle_list[cycle_list.len() - 20..]
    };

    let mut avg = [0.0; 4];
    for val in cycle {
        for i in 0..4 {
            avg[i] += val[i];
        }
    }
    for i in 0..4 {
        avg[i] = (avg[i] / cycle.len() as f64 * 1000.0).round() / 1000.0;
    }

    set_rah_resonance(ship, &mut cache, rah_index, avg);
    recalculate_values(info, ship, &mut cache, &rah_affected_attributes);
    ship.items[rah_index].store_cached_values(info, &cache.items[&rah_index]);
    ship.hull.store_cached_values(info, &cache.hull);
}

fn damage_type_order(resonance: &[f64; 4]) -> [usize; 4] {
    let mut k = [0, 1, 2, 3];
    k.sort_by(|&a, &b| resonance[b].partial_cmp(&resonance[a]).unwrap());
    k
}

fn calculate_rah_shift(
    effective_resonance: [f64; 4],
    rah_resonance: [f64; 4],
    max_shift_amount: f64,
) -> [f64; 4] {
    let order = damage_type_order(&effective_resonance);

    let mut shift = [0.0; 4];
    if effective_resonance[order[1]] == 0.0 {
        // One damage type: the top damage type takes from the other three
        // shift amount can be ignored since they will all shift to one damage type
        shift[order[1]] = 1.0 - rah_resonance[order[1]];
        shift[order[2]] = 1.0 - rah_resonance[order[2]];
        shift[order[3]] = 1.0 - rah_resonance[order[3]];
        shift[order[0]] = -(shift[order[1]] + shift[order[2]] + shift[order[3]]);
    } else if effective_resonance[order[2]] == 0.0 {
        // Two damage types: the top two damage types take from the other two
        // shift amount can be ignored since they will all shift to two damage types
        shift[order[2]] = 1.0 - rah_resonance[order[2]];
        shift[order[3]] = 1.0 - rah_resonance[order[3]];
        shift[order[0]] = -(shift[order[2]] + shift[order[3]]) / 2.0;
        shift[order[1]] = -(shift[order[2]] + shift[order[3]]) / 2.0;
    } else {
        // Three or four damage types: the top two damage types take from the other two
        shift[order[2]] = max_shift_amount.min(1.0 - rah_resonance[order[2]]);
        shift[order[3]] = max_shift_amount.min(1.0 - rah_resonance[order[3]]);
        shift[order[0]] = -(shift[order[2]] + shift[order[3]]) / 2.0;
        shift[order[1]] = -(shift[order[2]] + shift[order[3]]) / 2.0;
    }

    shift
}

fn set_rah_resonance(ship: &mut Ship, cache: &mut Cache, rah_index: usize, resonance: [f64; 4]) {
    for (id, resonance) in RESONANCE_IDS.iter().zip(resonance) {
        ship.items[rah_index].attributes.get_mut(id).unwrap().value = None;
        cache
            .items
            .entry(rah_index)
            .or_default()
            .insert(*id, resonance);
    }
}

// assumes that the rah only affects the hull
fn rah_affected_attributes(ship: &Ship, rah_index: usize) -> Vec<i32> {
    let mut ret = Vec::new();
    let mut source = Vec::from(RESONANCE_IDS.map(|id| (Object::Item(rah_index), id)));
    while !source.is_empty() {
        let mut attrs = Vec::new();
        for (id, attr) in ship.hull.attributes.iter() {
            if attr
                .effects
                .iter()
                .find(|e| source.contains(&(e.source, e.source_attribute_id)))
                .is_some()
            {
                attrs.push((Object::Ship, *id));
            }
        }
        ret.extend(attrs.iter().map(|(_, id)| id));
        source = attrs;
    }
    ret
}

fn recalculate_values(info: &impl Info, ship: &mut Ship, cache: &mut Cache, attribute_ids: &[i32]) {
    for id in attribute_ids {
        ship.hull.attributes.get_mut(id).unwrap().value = None;
        cache.hull.remove(id);
    }
    for id in attribute_ids {
        ship.hull.attributes[id].calculate_value(info, ship, cache, Object::Ship, *id);
    }
}

fn get_effective_resonance(ship: &mut Ship, cache: &mut Cache, ids: [i32; 4]) -> [f64; 4] {
    ids.map(|id| {
        *cache
            .hull
            .get(&id)
            .or(ship.hull.attributes.get(&id).and_then(|a| a.value.as_ref()))
            .unwrap()
    })
}

fn get_rah_resonance(ship: &mut Ship, cache: &mut Cache, rah_index: usize) -> [f64; 4] {
    RESONANCE_IDS.map(|id| {
        *cache
            .items
            .get(&rah_index)
            .and_then(|m| m.get(&id))
            .or(ship.items[rah_index]
                .attributes
                .get(&id)
                .and_then(|a| a.value.as_ref()))
            .unwrap()
    })
}
