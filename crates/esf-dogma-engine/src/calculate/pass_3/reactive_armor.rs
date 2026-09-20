use esf_data::Info;

use super::super::Objects;
use super::super::item::Item;

/* The Reactive Armor Hardener shifts its own resistances over time. The
 * effect from the SDE carries nothing but the four resonances. Working out
 * where they settle is done here. */
const EFFECT_ADAPTIVE_ARMOR_HARDENER_ID: i32 = 4928;

/* Each armor resonance, in the order EVE falls back on when two damage types
 * took the same amount. `Objects::reactive_armor` is in that order too. */
const RESONANCES: [&str; 4] = [
    "armorEmDamageResonance",
    "armorExplosiveDamageResonance",
    "armorKineticDamageResonance",
    "armorThermalDamageResonance",
];

/* Some combination don't settle, and can run for ever. Put on a cap
 * to prevent this from happening. */
const MAX_CYCLES: usize = 500;

/* A resonance is a sum of a few multiples of the shift amount, so anything
 * closer than this is the same state coming round again. */
const SAME_STATE: f64 = 1e-9;

struct Ids {
    resonance: [i32; 4],
    shift_amount: i32,
}

impl Ids {
    fn new(info: &impl Info) -> Option<Ids> {
        let mut resonance = [0; 4];

        for (index, name) in RESONANCES.iter().enumerate() {
            resonance[index] = info.attribute_name_to_id(name)?;
        }

        Some(Ids {
            resonance,
            shift_amount: info.attribute_name_to_id("resistanceShiftAmount")?,
        })
    }
}

/// Settle the Reactive Armor Hardener on the resistances it ends up holding,
/// and write them to the module.
pub fn simulate(info: &impl Info, objects: &Objects) {
    /* Without damage to shift against it stays where it starts, which is what
     * EVE shows: a plain hardener. */
    let Some(profile) = objects.reactive_armor else {
        return;
    };
    let Some(ids) = Ids::new(info) else {
        return;
    };

    let hardener = objects
        .items
        .iter()
        .find(|item| is_hardener(info, item, &ids));
    let Some(hardener) = hardener else {
        return;
    };

    let shift_amount = hardener.calculated_value(info, objects, ids.shift_amount) / 100.0;
    if shift_amount <= 0.0 {
        return;
    }

    let start = ids
        .resonance
        .map(|attribute_id| hardener.calculated_value(info, objects, attribute_id));
    let mut resonances = start;

    /* Every state the module started a cycle in, oldest first. */
    let mut history: Vec<[f64; 4]> = Vec::new();

    let settled = loop {
        if history.len() >= MAX_CYCLES {
            break None;
        }
        /* Back where it has been: from here on it repeats. */
        if let Some(index) = history.iter().position(|seen| is_same(seen, &resonances)) {
            break Some(average(&history[index..]));
        }
        history.push(resonances);

        let ship = ship_resonances(info, objects, &ids, hardener, &resonances);
        let damage = std::array::from_fn(|index| profile[index] * ship[index]);

        resonances = shifted(resonances, &damage, shift_amount);
    };

    let resonances = settled.unwrap_or_else(|| {
        /* Never settled, so take the run, minus the cycles it spent adapting.
         * That is over once the deepest resistance it started with could have
         * been given away, plus half as long again for the finer shifts. */
        let deepest = start.iter().copied().fold(1.0, f64::min);
        let adapting = (((1.0 - deepest) / shift_amount).ceil() * 1.5).ceil() as usize;

        average(&history[usize::min(adapting, history.len() / 2)..])
    });

    set_resonances(objects, &ids, hardener, &resonances);
}

fn is_hardener(info: &impl Info, item: &Item, ids: &Ids) -> bool {
    item.state.is_active()
        && ids
            .resonance
            .iter()
            .all(|attribute_id| item.attributes.contains_key(attribute_id))
        && info
            .get_dogma_effects(item.type_id)
            .into_iter()
            .flatten()
            .any(|effect| effect.effect_id() == EFFECT_ADAPTIVE_ARMOR_HARDENER_ID)
}

/* What the ship's armor resonances come to while the module holds these. The
 * module is only one of the modifiers on them, and a penalised one at that,
 * so the whole sum is worked out again per cycle. */
fn ship_resonances(
    info: &impl Info,
    objects: &Objects,
    ids: &Ids,
    hardener: &Item,
    resonances: &[f64; 4],
) -> [f64; 4] {
    set_resonances(objects, ids, hardener, resonances);

    ids.resonance
        .map(|attribute_id| objects.ship.calculated_value(info, objects, attribute_id))
}

/* Pin the module's resonances, and drop what the ship made of the previous
 * ones. Nothing else has been calculated off them yet, so they are all that
 * has to go. */
fn set_resonances(objects: &Objects, ids: &Ids, hardener: &Item, resonances: &[f64; 4]) {
    for (attribute_id, resonance) in ids.resonance.iter().zip(resonances) {
        if let Some(attribute) = hardener.attributes.get(attribute_id) {
            attribute.value.set(Some(*resonance));
        }
        if let Some(attribute) = objects.ship.attributes.get(attribute_id) {
            attribute.value.set(None);
        }
    }
}

/* One cycle: the types taking the least damage give resistance to the types
 * taking the most, leaving the total untouched. */
fn shifted(resonances: [f64; 4], damage: &[f64; 4], shift_amount: f64) -> [f64; 4] {
    /* Two always give, and a type taking nothing at all gives as well. */
    let donors = usize::max(2, damage.iter().filter(|taken| **taken == 0.0).count());
    let recipients = 4 - donors;
    if recipients == 0 {
        return resonances;
    }

    /* Least damaged first; a tie keeps the order the types are listed in. */
    let mut order = [0, 1, 2, 3];
    order.sort_by(|left, right| damage[*left].total_cmp(&damage[*right]));

    let mut shifted = resonances;
    let mut given = 0.0;

    for index in &order[..donors] {
        /* None of them can give more resistance than it still has. */
        let give = f64::min(1.0 - resonances[*index], shift_amount);
        given += give;
        shifted[*index] = resonances[*index] + give;
    }
    for index in &order[donors..] {
        shifted[*index] = resonances[*index] - given / recipients as f64;
    }

    shifted
}

fn is_same(left: &[f64; 4], right: &[f64; 4]) -> bool {
    left.iter()
        .zip(right)
        .all(|(left, right)| (left - right).abs() < SAME_STATE)
}

fn average(states: &[[f64; 4]]) -> [f64; 4] {
    let mut total = [0.0; 4];

    for state in states {
        for (sum, resonance) in total.iter_mut().zip(state) {
            *sum += resonance;
        }
    }

    total.map(|sum| sum / states.len() as f64)
}
