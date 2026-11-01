//! Looking up types and mutaplasmids in the static data.

use std::collections::BTreeMap;

use esf_data::{InfoExport, InfoName};

use crate::harness::{info, info_name};

const WARP_SCRAMBLER_II: i32 = 448;
const MUTATED_WARP_SCRAMBLER: i32 = 47732;
const DECAYED: i32 = 47729;
const GLORIFIED_UNSTABLE: i32 = 85645;

/* Warp Scrambler II has 6.0, 36.0 and 9000.0 for these. */
fn rolls(capacitor_need: f64, range: f64, cpu: f64) -> BTreeMap<i32, f64> {
    BTreeMap::from([(6, capacitor_need), (54, range), (50, cpu)])
}

#[test]
fn finds_the_narrowest_mutaplasmid_that_holds_the_rolls() {
    let found = info().find_mutaplasmid(
        WARP_SCRAMBLER_II,
        MUTATED_WARP_SCRAMBLER,
        &rolls(9.0, 9000.0, 36.0),
    );

    assert_eq!(found, Some(DECAYED));
}

#[test]
fn falls_back_to_the_widest_mutaplasmid_when_none_holds_the_rolls() {
    let found = info().find_mutaplasmid(
        WARP_SCRAMBLER_II,
        MUTATED_WARP_SCRAMBLER,
        &rolls(18.0, 9000.0, 36.0),
    );

    assert_eq!(found, Some(GLORIFIED_UNSTABLE));
}

#[test]
fn falls_back_to_the_widest_mutaplasmid_when_a_roll_is_missing() {
    let found =
        info().find_mutaplasmid(WARP_SCRAMBLER_II, MUTATED_WARP_SCRAMBLER, &BTreeMap::new());

    assert_eq!(found, Some(GLORIFIED_UNSTABLE));
}

#[test]
fn finds_no_mutaplasmid_that_does_not_make_the_result() {
    let found = info().find_mutaplasmid(WARP_SCRAMBLER_II, 587, &rolls(9.0, 9000.0, 36.0));

    assert_eq!(found, None);
}

#[test]
fn prefers_an_exact_english_name_over_a_translation() {
    /* Unpublished, and also the translated name of 45030 and 45047. */
    assert_eq!(info_name().type_name_to_id("Sun G5 (Yellow)"), Some(6));
}

#[test]
fn prefers_a_published_type_among_translations() {
    /* A translation of both "Trading" (52, unpublished) and "Trade" (3443). */
    assert_eq!(info_name().type_name_to_id("Commerce"), Some(3443));
}

#[test]
fn takes_the_lowest_id_among_unpublished_translations() {
    assert_eq!(
        info_name().type_name_to_id("'Katia Sae의 여정' 기념비"),
        Some(52203)
    );
}

#[test]
fn finds_no_type_for_an_unknown_name() {
    assert_eq!(info_name().type_name_to_id("No Such Type"), None);
}
