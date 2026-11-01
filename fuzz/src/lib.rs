//! What every fuzz target shares: the SDE, and the checks on a loaded fit.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use esf_data::{Info, InfoName, InfoNameSde, InfoSde, Names, Sde};
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

/// The type ID of the QA Immunity Module.
static QA_IMMUNITY_MODULE: LazyLock<i32> = LazyLock::new(|| {
    info_name()
        .type_name_to_id("QA Immunity Module")
        .expect("no QA Immunity Module in the SDE")
});

/// Attributes that are NaN when the ship is immune to the damage profile.
static IMMUNE_NAN: LazyLock<Vec<i32>> = LazyLock::new(|| {
    [
        "shieldEffectiveBoostRate",
        "armorEffectiveRepairRate",
        "hullEffectiveRepairRate",
    ]
    .iter()
    .map(|name| {
        info()
            .attribute_name_to_id(name)
            .unwrap_or_else(|| panic!("no attribute named {name}"))
    })
    .collect()
});

/// Per ship layer, each damage profile attribute with its resonance attribute.
static RESONANCES: LazyLock<Vec<Vec<(i32, i32)>>> = LazyLock::new(|| {
    [
        [
            ("damageProfileEm", "shieldEmDamageResonance"),
            ("damageProfileExplosive", "shieldExplosiveDamageResonance"),
            ("damageProfileKinetic", "shieldKineticDamageResonance"),
            ("damageProfileThermal", "shieldThermalDamageResonance"),
        ],
        [
            ("damageProfileEm", "armorEmDamageResonance"),
            ("damageProfileExplosive", "armorExplosiveDamageResonance"),
            ("damageProfileKinetic", "armorKineticDamageResonance"),
            ("damageProfileThermal", "armorThermalDamageResonance"),
        ],
        [
            ("damageProfileEm", "emDamageResonance"),
            ("damageProfileExplosive", "explosiveDamageResonance"),
            ("damageProfileKinetic", "kineticDamageResonance"),
            ("damageProfileThermal", "thermalDamageResonance"),
        ],
    ]
    .iter()
    .map(|layer| {
        layer
            .iter()
            .map(|(profile, resonance)| (attribute_id(profile), attribute_id(resonance)))
            .collect()
    })
    .collect()
});

fn attribute_id(name: &str) -> i32 {
    info()
        .attribute_name_to_id(name)
        .unwrap_or_else(|| panic!("no attribute named {name}"))
}

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

/// A mutation override above this may overflow to infinity.
const HUGE_OVERRIDE: f64 = 1e15;

/// A mutation override below this, zero included, may divide to infinity.
const TINY_OVERRIDE: f64 = 1e-15;

/// Calculates a fit, panics on a NaN or infinite number, and writes it back
/// out to every format.
pub fn check(fit: &Fit) {
    calculate_and_save(fit, true);
}

/// Calculates a fit and writes it back out to every format.
pub fn check_no_crash(fit: &Fit) {
    calculate_and_save(fit, false);
}

fn calculate_and_save(fit: &Fit, finite: bool) {
    let info = info();
    let options = Options {
        sources: true,
        validate: true,
    };
    let calculation = calculate(&info, fit, &options);
    let extreme_override = fit
        .items
        .iter()
        .filter_map(|item| item.mutation.as_ref())
        .flat_map(|mutation| mutation.attributes.values())
        .any(|value| !(TINY_OVERRIDE..=HUGE_OVERRIDE).contains(&value.abs()));
    if finite && !extreme_override {
        check_finite(&calculation, fit);
    }

    let _ = eft::save_eft(&info, fit);
    let _ = esi::save_esi_fitting(&info, fit);
    if let Ok(esf_fit) = esf::from_fit(&info, fit) {
        let fits = [esf_fit];
        let _ = esf::save_esf(&info, &fits);
        if let Ok(binary) = esf::save_esf_binary(&info, &fits) {
            let _ = esf::decode_base64url(&esf::encode_base64url(&binary));
        }
    }
}

/// Whether a layer of the ship resists all of the damage profile.
fn immune_layer(calculation: &Calculation) -> bool {
    let value = |attribute_id| {
        calculation
            .ship
            .attributes
            .get(&attribute_id)
            .map_or(1.0, |attribute| attribute.value)
    };
    RESONANCES.iter().any(|layer| {
        layer
            .iter()
            .map(|(profile, resonance)| value(*profile) * value(*resonance))
            .sum::<f64>()
            == 0.0
    })
}

fn check_finite(calculation: &Calculation, fit: &Fit) {
    let qa_immune = fit
        .items
        .iter()
        .any(|item| item.type_id == *QA_IMMUNITY_MODULE);
    let hull_immune = immune_layer(calculation);
    let mut problems = Vec::new();
    let mut flag = |path: String, value: f64| {
        let immune = qa_immune || (hull_immune && path.starts_with("ship."));
        if value.is_nan() || (value.is_infinite() && !immune) {
            problems.push(format!("{path} is {value}"));
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
        let immune = qa_immune || (hull_immune && path == "ship");
        for (attribute_id, attribute) in &result.attributes {
            let path = format!("{path}.attributes[{attribute_id}]");
            flag(format!("{path}.base"), attribute.base);
            if !(immune && attribute.value.is_nan() && IMMUNE_NAN.contains(attribute_id)) {
                flag(format!("{path}.value"), attribute.value);
            }
            for (index, source) in attribute.sources.iter().enumerate() {
                let path = format!("{path}.sources[{index}]");
                flag(format!("{path}.value"), source.value);
                if let Some(penalty) = source.penalty {
                    flag(format!("{path}.penalty"), penalty);
                }
            }
        }
    }

    for buff in &calculation.buffs {
        flag(format!("buffs[{}]", buff.id), buff.value);
    }
    for buff in &calculation.outgoing.buffs {
        flag(format!("outgoing.buffs[{}]", buff.id), buff.value);
    }
    for (index, effect) in calculation.outgoing.effects.iter().enumerate() {
        for (attribute_id, value) in &effect.attributes {
            flag(
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
            flag(format!("violations[{index}].used"), used);
            flag(format!("violations[{index}].available"), available);
        }
    }

    if !problems.is_empty() {
        panic!(
            "{}\nfit: {}",
            problems.join("\n"),
            serde_json::to_string(fit).unwrap()
        );
    }
}
