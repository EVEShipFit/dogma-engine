//! A modifier that reads the attribute it changes, directly or through
//! another item, still calculates.

use esf_data::Info;
use esf_dogma_engine::{Fit, Mutation, Options};

use crate::harness::{all, calculate, info};

const FIT: &str = r#"%esf/1
Tristan "Modifier cycle"

Light Entropic Disintegrator II
Light Entropic Disintegrator II
"#;

fn mutate_into_sinks(fit: &mut Fit) {
    for item in &mut fit.items {
        item.mutation = Some(Mutation {
            base: 47911,
            attributes: Default::default(),
        });
    }
}

#[test]
fn modifier_reads_what_it_changes() {
    let (_, calculation) = calculate(FIT, all(5), mutate_into_sinks, &Options::default());
    let attribute_id = info().attribute_name_to_id("damageMultiplier").unwrap();
    for item in &calculation.items {
        assert!(item.attributes[&attribute_id].value.is_finite());
    }
}
