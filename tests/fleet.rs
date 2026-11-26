//! What fleet/1 loads and saves: a fit and its boosters.

use esf_dogma_engine::{Booster, Fit};
use esf_format::esf::{
    Error, from_fit, load_esf, load_fit, main_fit, save_esf, save_esf_binary, save_fit,
    save_fit_binary,
};

use crate::harness::info;

const FLEET: &str = "\
%esf/1
Rifter \"Roamer\"

200mm AutoCannon I

%esf/1
Claymore \"Shield Boosts\"

Shield Command Burst I :Shield Harmonizing Charge

%esf/1
Damnation \"Armor Boosts\"

Armor Command Burst I :Armor Energizing Charge

%fleet/1
Rifter \"Roamer\"
Claymore \"Shield Boosts\"
Damnation \"Armor Boosts\" !off
";

const RIFTER: &str = "\
%esf/1
Rifter \"Roamer\"

200mm AutoCannon I
";

fn load(text: &str) -> Fit {
    load_fit(info(), text.as_bytes()).unwrap()
}

fn json(fit: &Fit) -> serde_json::Value {
    serde_json::to_value(fit).unwrap()
}

#[test]
fn loads_boosters() {
    let fit = load(FLEET);

    assert_eq!(fit.name.as_deref(), Some("Roamer"));
    let boosters: Vec<_> = fit
        .boosters
        .iter()
        .map(|booster| (booster.fit.name.as_deref(), booster.off))
        .collect();
    assert_eq!(
        boosters,
        [(Some("Shield Boosts"), false), (Some("Armor Boosts"), true)]
    );
}

#[test]
fn round_trips_text() {
    assert_eq!(save_fit(info(), &load(FLEET)).unwrap(), FLEET);
}

#[test]
fn round_trips_binary() {
    let fit = load(FLEET);
    let binary = save_fit_binary(info(), &fit).unwrap();

    assert_eq!(json(&load_fit(info(), &binary).unwrap()), json(&fit));
}

#[test]
fn old_readers_see_the_boosted_fit() {
    let fit = load(FLEET);
    let binary = save_fit_binary(info(), &fit).unwrap();

    for data in [FLEET.as_bytes(), &binary] {
        let fits = load_esf(info(), data).unwrap();
        assert_eq!(
            save_esf(info(), &[main_fit(&fits).unwrap().clone()]).unwrap(),
            RIFTER
        );
    }
}

#[test]
fn names_unnamed_and_clashing_boosters() {
    let mut fit = load(FLEET);
    let claymore = fit.boosters[0].fit.clone();
    let mut unnamed = claymore.clone();
    unnamed.name = None;
    let mut taken = claymore.clone();
    taken.name = Some("Claymore - Booster 1".to_string());
    fit.boosters = [unnamed.clone(), claymore, unnamed, load(RIFTER), taken]
        .into_iter()
        .map(|fit| Booster { fit, off: false })
        .collect();

    let text = save_fit(info(), &fit).unwrap();
    let fleet = text.split_once("%fleet/1\n").unwrap().1;
    assert_eq!(
        fleet,
        "\
Rifter \"Roamer\"
Claymore \"Claymore - Booster 2\"
Claymore \"Shield Boosts\"
Claymore \"Claymore - Booster 3\"
Rifter \"Rifter - Booster 1\"
Claymore \"Claymore - Booster 1\"
"
    );
    assert_eq!(load(&text).boosters.len(), 5);
}

#[test]
fn a_fit_without_boosters_saves_as_before() {
    let fit = load(RIFTER);
    let esf_fit = from_fit(info(), &fit).unwrap();

    assert!(fit.boosters.is_empty());
    assert_eq!(save_fit(info(), &fit).unwrap(), RIFTER);
    assert_eq!(
        save_fit_binary(info(), &fit).unwrap(),
        save_esf_binary(info(), &[esf_fit]).unwrap()
    );
}

#[test]
fn loads_an_old_link() {
    let esf_fit = from_fit(info(), &load(RIFTER)).unwrap();
    let binary = save_esf_binary(info(), &[esf_fit]).unwrap();

    let fit = load_fit(info(), &binary).unwrap();
    assert!(fit.boosters.is_empty());
    assert_eq!(save_fit(info(), &fit).unwrap(), RIFTER);
}

#[test]
fn reads_quotes_and_comments() {
    let text = "\
%esf/1
Rifter \"Say \"\"Hi\"\"\"

%esf/1
Claymore

%fleet/1
// The fit.
\"Rifter\" \"say \"\"hi\"\"\"

Claymore // Unnamed.
";
    let fit = load(text);
    assert_eq!(fit.name.as_deref(), Some("Say \"Hi\""));
    assert_eq!(fit.boosters.len(), 1);
    assert!(!fit.boosters[0].off);
}

#[test]
fn rejects_a_line_no_fit_matches() {
    let text = format!("{RIFTER}\n%fleet/1\nRifter \"Roamer\"\nClaymore \"Missing\"\n");

    assert_eq!(
        load_fit(info(), text.as_bytes()).unwrap_err(),
        Error::Fleet("fleet line 2: no fit is \"Claymore \\\"Missing\\\"\"".to_string())
    );
}

fn fleet_error(fleet: &str) -> String {
    let text = format!("{FLEET_FITS}%fleet/1\n{fleet}");
    load_fit(info(), text.as_bytes()).unwrap_err().to_string()
}

const FLEET_FITS: &str = "\
%esf/1
Rifter \"Roamer\"

%esf/1
Claymore \"Shield Boosts\"

";

#[test]
fn reads_a_hull_by_its_id() {
    let text = format!(
        "{FLEET_FITS}%fleet/1\n\"Rifter\"#587 \"Roamer\"\nClaymore#22468 \"Shield Boosts\"\n"
    );
    assert_eq!(load(&text).boosters.len(), 1);

    assert_eq!(
        fleet_error("Rifter \"Roamer\"\nClaymore#587 \"Shield Boosts\"\n"),
        "fleet line 2: type #587 is \"Rifter\", not \"Claymore\""
    );
    assert_eq!(
        fleet_error("\"Rifter#587\" \"Roamer\"\n"),
        "fleet line 1: no such type: Rifter#587"
    );
}

#[test]
fn starts_with_the_first_fit() {
    assert_eq!(
        fleet_error("Claymore \"Shield Boosts\"\nRifter \"Roamer\"\n"),
        "a fleet starts with the first fit"
    );
}

#[test]
fn names_each_fit_once() {
    assert_eq!(
        fleet_error("Rifter \"Roamer\"\nRifter \"roamer\"\n"),
        "a fleet names fit 1 more than once"
    );
    assert_eq!(
        fleet_error("Rifter \"Roamer\"\nClaymore \"Shield Boosts\"\nClaymore \"Shield Boosts\"\n"),
        "a fleet names fit 2 more than once"
    );
}

#[test]
fn reads_crlf() {
    let text = FLEET.replace('\n', "\r\n");
    assert_eq!(save_fit(info(), &load(&text)).unwrap(), FLEET);
}

#[test]
fn round_trips_an_unnamed_fit() {
    let mut fit = load(FLEET);
    fit.name = None;

    let text = save_fit(info(), &fit).unwrap();
    assert!(text.contains("%fleet/1\nRifter\nClaymore"));
    assert_eq!(save_fit(info(), &load(&text)).unwrap(), text);
}
