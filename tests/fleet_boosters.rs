//! Boosters hand their command bursts to the fit they boost.

use esf_dogma_engine::{Booster, Fit, Options};

use crate::harness::{all, info, load, outgoing};

const RIFTER: &str = r#"%esf/1
Rifter "Roamer"
"#;

const CLAYMORE_T1: &str = r#"%esf/1
Claymore "Shield Boosts"

Shield Command Burst I :Shield Harmonizing Charge
"#;

const CLAYMORE_T2: &str = r#"%esf/1
Claymore "Better Shield Boosts"

Shield Command Burst II :Shield Harmonizing Charge
"#;

fn booster(text: &str, off: bool) -> Booster {
    Booster {
        fit: load(text).unwrap(),
        off,
    }
}

fn shield_em_resonance(fit: &Fit) -> f64 {
    let attribute_id = info()
        .attribute_name_to_id("shieldEmDamageResonance")
        .unwrap();
    let calculation = esf_dogma_engine::calculate(info(), fit, &Options::default());
    calculation.ship.attributes[&attribute_id].value
}

fn rifter(boosters: Vec<Booster>) -> Fit {
    Fit {
        boosters,
        ..load(RIFTER).unwrap()
    }
}

fn rifter_incoming(text: &str) -> f64 {
    let mut fit = load(RIFTER).unwrap();
    fit.incoming = outgoing(text, all(5));
    shield_em_resonance(&fit)
}

#[test]
fn booster_applies_its_bursts() {
    let unboosted = shield_em_resonance(&rifter(vec![]));
    let boosted = shield_em_resonance(&rifter(vec![booster(CLAYMORE_T1, false)]));

    assert!(boosted < unboosted);
    assert_eq!(boosted, rifter_incoming(CLAYMORE_T1));
}

#[test]
fn booster_that_is_off_applies_nothing() {
    let unboosted = shield_em_resonance(&rifter(vec![]));
    let off = shield_em_resonance(&rifter(vec![booster(CLAYMORE_T1, true)]));

    assert_eq!(off, unboosted);
}

#[test]
fn strongest_of_the_same_buff_wins() {
    let t1 = rifter_incoming(CLAYMORE_T1);
    let t2 = rifter_incoming(CLAYMORE_T2);
    assert!(t2 < t1);

    let both = rifter(vec![
        booster(CLAYMORE_T1, false),
        booster(CLAYMORE_T2, false),
    ]);
    assert_eq!(shield_em_resonance(&both), t2);

    let mut incoming = rifter(vec![booster(CLAYMORE_T1, false)]);
    incoming.incoming = outgoing(CLAYMORE_T2, all(5));
    assert_eq!(shield_em_resonance(&incoming), t2);
}

#[test]
fn booster_with_skills_flies_with_them() {
    let mut weak = booster(CLAYMORE_T1, false);
    weak.fit.character.skills = all(1).levels;

    assert!(shield_em_resonance(&rifter(vec![weak])) > rifter_incoming(CLAYMORE_T1));
}

#[test]
fn boosters_of_a_booster_are_ignored() {
    let mut nested = booster(RIFTER, false);
    nested.fit.boosters.push(booster(CLAYMORE_T1, false));

    let unboosted = shield_em_resonance(&rifter(vec![]));
    assert_eq!(shield_em_resonance(&rifter(vec![nested])), unboosted);
}
