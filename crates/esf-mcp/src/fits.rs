//! Reading fits in any format, and what a calculation of one runs under.

use std::collections::BTreeMap;

use esf_dogma_engine::{
    Calculation, DamageProfile, Fit, Options, Rule, Security, Slot, Target, Violation, beacon,
    calculate,
};
use esf_format::{dna, eft, esf, esi, killmail, post_load};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::game::Game;

const CATEGORY_CELESTIAL: i32 = 2;

#[derive(Deserialize, JsonSchema, Default, Clone)]
pub struct Settings {
    /// Level of every skill, 0 to 5. Default 5.
    pub skill_level: Option<u8>,
    /// Skill levels by skill name, on top of `skill_level`, like {"Gunnery": 4}.
    #[serde(default)]
    pub skills: BTreeMap<String, u8>,
    /// Incoming damage for effective hitpoints; only the ratio matters. Default: equal parts.
    pub damage_profile: Option<DamageProfileInput>,
    /// Security of the solar system. Default high_sec.
    pub security: Option<SecurityInput>,
    /// A beacon in space by name, like "Class 6 Black Hole Effects".
    pub beacon: Option<String>,
}

#[derive(Deserialize, JsonSchema, Clone, Copy)]
pub struct DamageProfileInput {
    #[serde(default)]
    pub em: f64,
    #[serde(default)]
    pub explosive: f64,
    #[serde(default)]
    pub kinetic: f64,
    #[serde(default)]
    pub thermal: f64,
}

#[derive(Deserialize, JsonSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SecurityInput {
    HighSec,
    LowSec,
    NullSec,
    Wormhole,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    Esf,
    Eft,
    Dna,
    EsiFitting,
    Killmail,
}

pub fn detect(text: &str) -> Format {
    let text = text.trim();
    if text.starts_with('[') {
        return Format::Eft;
    }
    if text.starts_with('{') {
        let json: serde_json::Value = serde_json::from_str(text).unwrap_or_default();
        return match json.get("victim").is_some() {
            true => Format::Killmail,
            false => Format::EsiFitting,
        };
    }
    let dna = text.strip_prefix("fitting:").unwrap_or(text);
    let ship = dna.split(':').next().unwrap_or_default();
    if !text.contains('\n') && dna.contains(':') && ship.bytes().all(|byte| byte.is_ascii_digit()) {
        return Format::Dna;
    }
    Format::Esf
}

impl Game {
    /// A fit without skills or environment, as written.
    pub fn parse_fit(&self, text: &str) -> Result<(Fit, Format), String> {
        let format = detect(text);
        let fit = match format {
            Format::Eft => eft::load_eft(&self.info_name, text).map_err(|error| {
                format!("EFT: {error}{}", self.unknown_type_hint(&error.to_string()))
            })?,
            Format::Dna => {
                dna::load_dna(&self.info, text.trim()).map_err(|error| format!("DNA: {error}"))?
            }
            Format::EsiFitting => {
                let fitting: esi::EsiFitting =
                    serde_json::from_str(text).map_err(|error| format!("ESI fitting: {error}"))?;
                esi::load_esi_fitting(&self.info, &fitting)
            }
            Format::Killmail => {
                let killmail: killmail::EsiKillmail =
                    serde_json::from_str(text).map_err(|error| format!("killmail: {error}"))?;
                killmail::load_killmail(&self.info, &killmail)
            }
            Format::Esf => {
                let header = text.trim_start().starts_with("%esf/");
                let text = match header {
                    true => text.to_string(),
                    false => format!("%esf/1\n{text}"),
                };
                let fits = esf::load_esf(&self.info, text.as_bytes()).map_err(|error| {
                    let error = error.to_string();
                    let error = match header {
                        true => error,
                        false => without_header(&error),
                    };
                    format!("esf/1: {error}{}", self.unknown_type_hint(&error))
                })?;
                let main = esf::main_fit(&fits).ok_or("esf/1: the document holds no fit")?;
                esf::to_fit(&self.info, main).map_err(|error| format!("esf/1: {error}"))?
            }
        };
        Ok((fit, format))
    }

    /// Close names for the type an "unknown type" error names.
    fn unknown_type_hint(&self, error: &str) -> String {
        match error.split_once("unknown type ") {
            Some((_, name)) => {
                self.did_you_mean(name.lines().next().unwrap_or_default().trim_matches('"'))
            }
            None => String::new(),
        }
    }

    /// A fit ready to calculate: skills and environment applied.
    pub fn load_fit(&self, text: &str, settings: &Settings) -> Result<Fit, String> {
        let (mut fit, format) = self.parse_fit(text)?;
        self.apply(&mut fit, settings)?;
        if format != Format::Esf {
            post_load(&self.info, &mut fit);
        }
        Ok(fit)
    }

    fn apply(&self, fit: &mut Fit, settings: &Settings) -> Result<(), String> {
        let level = settings.skill_level.unwrap_or(5);
        if level > 5 {
            return Err(format!("skill_level should be 0 to 5, not {level}"));
        }
        let skills = &mut fit.character.skills;
        for skill_id in &self.skills {
            skills.insert(*skill_id, level);
        }
        for (name, level) in &settings.skills {
            if *level > 5 {
                return Err(format!("{name} should be 0 to 5, not {level}"));
            }
            let skill_id = self
                .find_type_id(name)
                .filter(|type_id| self.skills.contains(type_id))
                .ok_or_else(|| format!("no skill is named \"{name}\""))?;
            skills.insert(skill_id, *level);
        }

        if let Some(profile) = settings.damage_profile {
            let parts = [
                profile.em,
                profile.explosive,
                profile.kinetic,
                profile.thermal,
            ];
            if parts.iter().any(|part| part.is_nan() || *part < 0.0)
                || parts.iter().sum::<f64>() <= 0.0
            {
                return Err("damage_profile takes parts of 0 or more, not all 0".to_string());
            }
            fit.environment.damage_profile = DamageProfile {
                em: profile.em,
                explosive: profile.explosive,
                kinetic: profile.kinetic,
                thermal: profile.thermal,
            };
        }
        if let Some(security) = settings.security {
            fit.environment.security = match security {
                SecurityInput::HighSec => Security::HighSec,
                SecurityInput::LowSec => Security::LowSec,
                SecurityInput::NullSec => Security::NullSec,
                SecurityInput::Wormhole => Security::Wormhole,
            };
        }
        if let Some(name) = &settings.beacon {
            let type_id = self
                .find_type_id(name)
                .filter(|type_id| {
                    self.sde
                        .get_type(*type_id)
                        .is_some_and(|r#type| r#type.category_id() == CATEGORY_CELESTIAL)
                })
                .ok_or_else(|| format!("no beacon is named \"{name}\""))?;
            fit.incoming.extend(beacon(&self.info, type_id));
        }
        Ok(())
    }

    pub fn calculate(&self, fit: &Fit, sources: bool) -> Calculation {
        let options = Options {
            sources,
            validate: true,
        };
        calculate(&self.info, fit, &options)
    }

    /// The fit as canonical esf/1 text.
    pub fn to_esf(&self, fit: &Fit) -> Result<String, String> {
        let esf_fit = esf::from_fit(&self.info, fit).map_err(|error| error.to_string())?;
        esf::save_esf(&self.info, &[esf_fit]).map_err(|error| error.to_string())
    }

    pub fn item_label(&self, fit: &Fit, index: usize) -> String {
        let item = &fit.items[index];
        format!(
            "{}: {}",
            slot_label(item.slot),
            self.type_name(item.type_id)
        )
    }

    /// What a violation is about, without its numbers.
    pub fn violation_kind(&self, violation: &Violation) -> String {
        let rule = serde_json::to_value(violation.rule).unwrap_or_default();
        format!(
            "{:?} {} {} {}",
            violation.target, rule["type"], rule["resource"], rule["slot"]
        )
    }

    pub fn violation(&self, fit: &Fit, violation: &Violation) -> String {
        let target = match violation.target {
            Target::Ship => format!("ship {}", self.type_name(fit.ship.type_id)),
            Target::Item { index } => self.item_label(fit, index),
            Target::Charge { index } => format!("charge of {}", self.item_label(fit, index)),
        };
        format!("{target}: {}", self.rule(violation.rule))
    }

    fn rule(&self, rule: Rule) -> String {
        match rule {
            Rule::Resource {
                resource,
                used,
                available,
            } => format!(
                "not enough {}: {used:.2} used of {available:.2}",
                label(resource)
            ),
            Rule::Slots {
                slot,
                used,
                available,
            } => format!("too many in {} slots: {used} of {available}", label(slot)),
            Rule::WrongSlot { expected } => format!("belongs in a {} slot", label(expected)),
            Rule::NotAModule => "only a module goes in this slot".to_string(),
            Rule::NotADrone => "only a drone goes in the drone bay".to_string(),
            Rule::NotAFighter => "only a fighter goes here".to_string(),
            Rule::NotAnImplant => "only an implant or booster goes here".to_string(),
            Rule::SlotTaken => "another item is in this slot".to_string(),
            Rule::WrongSlotIndex { expected } => format!("belongs in slot {expected}"),
            Rule::SubsystemTaken => "another subsystem covers the same part".to_string(),
            Rule::Skill {
                type_id,
                required,
                level,
            } => format!(
                "needs {} at {required}, trained to {level}",
                self.type_name(type_id)
            ),
            Rule::RigSize { ship, item } => format!("rig size {item}, the ship takes {ship}"),
            Rule::ShipRestricted => "cannot go on this ship".to_string(),
            Rule::CapitalItem => "a capital item on a ship that is not a capital".to_string(),
            Rule::MaxGroup {
                group_id,
                limit,
                used,
                allowed,
            } => format!(
                "{used} of group {} {}, {allowed} allowed",
                self.group_name(group_id),
                label(limit)
            ),
            Rule::MaxType {
                type_id,
                used,
                allowed,
            } => format!(
                "{used} of {} fitted, {allowed} allowed",
                self.type_name(type_id)
            ),
            Rule::NotAHull => "not a ship, structure or container".to_string(),
            Rule::NotAMode => "the mode is not a tactical mode".to_string(),
            Rule::NotASkill { type_id } => format!("{} is not a skill", self.type_name(type_id)),
            Rule::ChargeGroup => "a charge the module does not take".to_string(),
            Rule::ChargeSize { module, charge } => {
                format!("charge size {charge}, the module takes {module}")
            }
            rule => format!("{rule:?}"),
        }
    }
}

/// An error about a line, numbered as if the `%esf/1` line was not added.
fn without_header(error: &str) -> String {
    let Some((before, after)) = error.split_once("line ") else {
        return error.to_string();
    };
    let digits = after.bytes().take_while(u8::is_ascii_digit).count();
    match after[..digits].parse::<u64>() {
        Ok(line) if line > 1 => format!("{before}line {}{}", line - 1, &after[digits..]),
        _ => error.to_string(),
    }
}

pub fn slot_label(slot: Slot) -> String {
    match slot {
        Slot::High(index) => format!("high {index}"),
        Slot::Medium(index) => format!("medium {index}"),
        Slot::Low(index) => format!("low {index}"),
        Slot::Rig(index) => format!("rig {index}"),
        Slot::Subsystem(index) => format!("subsystem {index}"),
        Slot::Service(index) => format!("service {index}"),
        Slot::FighterTube(index) => format!("fighter tube {index}"),
        Slot::FighterBay => "fighter bay".to_string(),
        Slot::Implant(index) => format!("implant {index}"),
        Slot::Booster(index) => format!("booster {index}"),
        Slot::DroneBay => "drone bay".to_string(),
        Slot::Cargo => "cargo".to_string(),
    }
}

/// The name serde gives a value.
pub fn label(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_formats() {
        assert_eq!(detect("%esf/1\nRifter\n"), Format::Esf);
        assert_eq!(detect("Rifter\n200mm AutoCannon II\n"), Format::Esf);
        assert_eq!(detect("  [Rifter, Test]\n"), Format::Eft);
        assert_eq!(detect("587:2873;3::"), Format::Dna);
        assert_eq!(detect("fitting:587:2873;3::"), Format::Dna);
        assert_eq!(detect("587:2873;3::\n"), Format::Dna);
        assert_eq!(
            detect(r#"{"ship_type_id": 587, "description": "\"victim\"", "items": []}"#),
            Format::EsiFitting
        );
        assert_eq!(
            detect(r#"{"ship_type_id": 587, "items": []}"#),
            Format::EsiFitting
        );
        assert_eq!(
            detect(r#"{"killmail_id": 1, "victim": {}}"#),
            Format::Killmail
        );
    }

    #[test]
    fn renumbers_lines() {
        assert_eq!(
            without_header("line 2: unknown type \"Riftr\""),
            "line 1: unknown type \"Riftr\""
        );
        assert_eq!(without_header("no fit"), "no fit");
    }
}
