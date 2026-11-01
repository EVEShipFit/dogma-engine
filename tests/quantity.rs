//! A stack of items calculates as one item, however large.

use std::collections::BTreeMap;

use esf_data::Info;
use esf_dogma_engine::{Calculation, Fit, FitItem, Options, Slot};

use crate::harness::{all, calculate, info};

const SOURCES: Options = Options {
    sources: true,
    validate: false,
};

const DRONES: &str = r#"%esf/1
Vexor "Huge drone stack"

4294967295x Hobgoblin II
"#;

const AMPLIFIERS: &str = r#"%esf/1
Vexor "Amplifier stack"

Drone Damage Amplifier II

Hammerhead II
"#;

const STACK: &str = r#"%esf/1
Vexor "Drone stack"

5x Hobgoblin II
"#;

fn stack_amplifiers(fit: &mut Fit, quantity: u32) {
    let amplifier = fit
        .items
        .iter_mut()
        .find(|item| item.slot != Slot::DroneBay)
        .unwrap();
    amplifier.quantity = quantity;
}

fn drone_damage(calculation: &Calculation, fit: &Fit) -> (f64, Vec<(u32, Option<f64>)>) {
    let attribute_id = info().attribute_name_to_id("damageMultiplier").unwrap();
    let drone = fit
        .items
        .iter()
        .position(|item| item.slot == Slot::DroneBay)
        .unwrap();
    let attribute = &calculation.items[drone].attributes[&attribute_id];
    let sources = attribute
        .sources
        .iter()
        .filter(|source| source.penalty.is_some())
        .map(|source| (source.quantity, source.penalty))
        .collect();
    (attribute.value, sources)
}

#[test]
fn huge_drone_stack() {
    let (_, calculation) = calculate(DRONES, all(5), |_| {}, &SOURCES);
    assert!(
        calculation
            .ship
            .attributes
            .values()
            .all(|attribute| !attribute.value.is_nan())
    );
}

#[test]
fn penalised_stack_past_the_cap() {
    let (fit, eight) = calculate(AMPLIFIERS, all(5), |fit| stack_amplifiers(fit, 8), &SOURCES);
    let (eight, _) = drone_damage(&eight, &fit);

    let (fit, twenty) = calculate(
        AMPLIFIERS,
        all(5),
        |fit| stack_amplifiers(fit, 20),
        &SOURCES,
    );
    let (twenty, sources) = drone_damage(&twenty, &fit);

    assert_eq!(twenty, eight);
    assert_eq!(sources.len(), 9);
    assert!(sources[..8].iter().all(|(quantity, _)| *quantity == 1));
    assert_eq!(sources[8], (12, Some(0.0)));
}

fn split_stack(fit: &mut Fit) {
    let drone = fit.items.pop().unwrap();
    for _ in 0..drone.quantity {
        fit.items.push(FitItem {
            quantity: 1,
            ..drone.clone()
        });
    }
}

fn ship_values(calculation: &Calculation) -> BTreeMap<i32, f64> {
    calculation
        .ship
        .attributes
        .iter()
        .map(|(id, attribute)| (*id, attribute.value))
        .collect()
}

fn assert_close(left: &BTreeMap<i32, f64>, right: &BTreeMap<i32, f64>) {
    for (id, left) in left {
        let right = right[id];
        let close = *left == right || (left - right).abs() <= 1e-9 * left.abs().max(right.abs());
        assert!(close, "attribute {id}: {left} != {right}");
    }
}

#[test]
fn stack_is_its_items() {
    let (_, stack) = calculate(STACK, all(5), |_| {}, &SOURCES);
    let (fit, split) = calculate(STACK, all(5), split_stack, &SOURCES);
    assert_eq!(fit.items.len(), 5);
    assert_eq!(stack.ship.attributes.len(), split.ship.attributes.len());
    assert_close(&ship_values(&stack), &ship_values(&split));
}
