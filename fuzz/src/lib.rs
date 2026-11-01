//! What every fuzz target shares: the SDE, and the checks on a loaded fit.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use esf_data::{InfoNameSde, InfoSde, Names, Sde};
use esf_dogma_engine::{Calculation, Fit, ItemResult, Options, Rule, calculate};
use esf_format::{eft, esf, esi};

pub mod fit;

const SKILL_CATEGORY_ID: i32 = 16;

fn read(variable: &str, default: &str) -> &'static [u8] {
    let filename = std::env::var(variable).unwrap_or_else(|_| default.to_string());
    let bytes =
        std::fs::read(&filename).unwrap_or_else(|error| panic!("cannot read {filename}: {error}"));
    Box::leak(bytes.into_boxed_slice())
}

pub static SDE: LazyLock<Sde<'static>> = LazyLock::new(|| {
    let bytes = read(
        "ESF_SDE",
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../node_modules/@eveshipfit/sde/dist/sde.dat"
        ),
    );
    Sde::new(bytes).unwrap()
});

static NAMES: LazyLock<Names<'static>> = LazyLock::new(|| {
    let bytes = read(
        "ESF_NAMES",
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../node_modules/@eveshipfit/sde/dist/names.dat"
        ),
    );
    Names::new(bytes).unwrap()
});

static ALL_SKILLS: LazyLock<BTreeMap<i32, u8>> = LazyLock::new(|| {
    SDE.types()
        .filter(|r#type| r#type.category_id() == SKILL_CATEGORY_ID)
        .map(|r#type| (r#type.id(), 5))
        .collect()
});

pub fn info() -> InfoSde<'static> {
    InfoSde::new(&SDE)
}

pub fn info_name() -> InfoNameSde<'static> {
    InfoNameSde::new(&SDE, Some(&NAMES)).unwrap()
}

/// Calculates a loaded fit with no and with all skills, before and after
/// `post_load`, and writes it back out to every format.
pub fn check_loaded(fit: Fit) {
    let info = info();

    for skills in [BTreeMap::new(), ALL_SKILLS.clone()] {
        let mut fit = fit.clone();
        fit.character.skills = skills;
        check(&fit);

        esf_format::post_load(&info, &mut fit);
        check(&fit);
    }
}

/// Calculates a fit, panics on any NaN or infinite number, and writes it back
/// out to every format.
pub fn check(fit: &Fit) {
    let info = info();
    let options = Options {
        sources: true,
        validate: true,
    };
    let calculation = calculate(&info, fit, &options);
    check_finite(&calculation, fit);

    let _ = eft::save_eft(&info, fit);
    let _ = esi::save_esi_fitting(&info, fit);
    if let Ok(esf_fit) = esf::from_fit(&info, fit) {
        let fits = [esf_fit];
        let _ = esf::save_esf(&info, &fits);
        let _ = esf::save_esf_binary(&info, &fits);
    }
}

fn check_finite(calculation: &Calculation, fit: &Fit) {
    let fail = |path: String, value: f64| -> ! {
        panic!(
            "{path} is {value}\nfit: {}",
            serde_json::to_string(fit).unwrap()
        )
    };
    let check = |path: String, value: f64| {
        if !value.is_finite() {
            fail(path, value);
        }
    };

    let mut results: Vec<(String, &ItemResult)> = vec![
        ("ship".to_string(), &calculation.ship),
        ("character".to_string(), &calculation.character),
    ];
    if let Some(mode) = &calculation.mode {
        results.push(("mode".to_string(), mode));
    }
    for (index, item) in calculation.items.iter().enumerate() {
        results.push((format!("items[{index}]"), item));
        if let Some(charge) = &item.charge {
            results.push((format!("items[{index}].charge"), charge));
        }
    }

    for (path, result) in results {
        for (attribute_id, attribute) in &result.attributes {
            let path = format!("{path}.attributes[{attribute_id}]");
            check(format!("{path}.base"), attribute.base);
            check(format!("{path}.value"), attribute.value);
            for (index, source) in attribute.sources.iter().enumerate() {
                check(format!("{path}.sources[{index}].value"), source.value);
                if let Some(penalty) = source.penalty {
                    check(format!("{path}.sources[{index}].penalty"), penalty);
                }
            }
        }
    }

    for buff in &calculation.buffs {
        check(format!("buffs[{}]", buff.id), buff.value);
    }
    for buff in &calculation.outgoing.buffs {
        check(format!("outgoing.buffs[{}]", buff.id), buff.value);
    }
    for (index, effect) in calculation.outgoing.effects.iter().enumerate() {
        for (attribute_id, value) in &effect.attributes {
            check(
                format!("outgoing.effects[{index}].attributes[{attribute_id}]"),
                *value,
            );
        }
    }

    for (index, violation) in calculation.violations.iter().flatten().enumerate() {
        if let Rule::Resource {
            used, available, ..
        } = violation.rule
        {
            check(format!("violations[{index}].used"), used);
            check(format!("violations[{index}].available"), available);
        }
    }
}
