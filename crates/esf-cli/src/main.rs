use std::collections::BTreeMap;
use std::io::{IsTerminal, Read};
use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use esf_data::{Info, InfoName, InfoNameSde, InfoSde, Names, Sde};
use esf_dogma_engine::{
    Calculation, DamageProfile, Fit, ItemResult, Options, ReactiveArmor, Rule, Security, Slot,
    SourceRef, State, Target, Violation,
};
use esf_format::eft;

const SKILL_CATEGORY_ID: i32 = 16;
const CELESTIAL_CATEGORY_ID: i32 = 2;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    state: Option<String>,

    #[clap(short, long)]
    eft_filename: Option<PathBuf>,

    #[clap(
        short = 'd',
        long,
        default_value = "node_modules/@eveshipfit/sde/dist/sde.dat"
    )]
    sde_filename: PathBuf,

    #[clap(
        short = 'n',
        long,
        default_value = "node_modules/@eveshipfit/sde/dist/names.dat"
    )]
    names_filename: PathBuf,

    /// Every skill at this level.
    #[clap(
        short = 'l',
        long,
        value_name = "0-5",
        value_parser = clap::value_parser!(u8).range(0..=5),
        help_heading = "Skills"
    )]
    skill_level: Option<u8>,

    /// JSON file of level per skill id; overrides --skill-level per skill.
    #[clap(short = 'f', long, help_heading = "Skills")]
    skills_filename: Option<PathBuf>,

    /// A skill by name, like "Gunnery=4"; overrides both of the above.
    #[clap(
        long = "skill",
        value_name = "NAME=LEVEL",
        value_parser = parse_skill,
        help_heading = "Skills"
    )]
    skills: Vec<(String, u8)>,

    /// Security of the solar system; structure rigs are stronger outside high-sec.
    #[clap(
        long,
        value_enum,
        default_value = "high-sec",
        help_heading = "Environment"
    )]
    security: SystemSecurity,

    /// A beacon in space by name, like "Class 6 Black Hole Effects".
    #[clap(long = "beacon", value_name = "NAME", help_heading = "Environment")]
    beacons: Vec<String>,

    /// Incoming damage, like "0,0,3,1"; only the ratio matters.
    #[clap(
        long,
        value_name = "EM,EXPLOSIVE,KINETIC,THERMAL",
        value_parser = parse_damage_profile,
        help_heading = "Environment"
    )]
    damage_profile: Option<DamageProfile>,

    /// What a Reactive Armor Hardener shifts towards; absent leaves it as EVE shows it.
    #[clap(
        long,
        value_name = "EM,EXPLOSIVE,KINETIC,THERMAL",
        num_args = 0..=1,
        default_missing_value = "",
        value_parser = parse_reactive_armor,
        help_heading = "Environment"
    )]
    reactive_armor: Option<ReactiveArmor>,

    /// Write the fit back out as EFT instead of calculating it.
    #[clap(long, help_heading = "Output")]
    eft: bool,

    /// Default: table when stdout is a terminal, json otherwise.
    #[clap(short, long, value_enum, help_heading = "Output")]
    output: Option<Output>,

    /// Only attributes whose name contains TEXT.
    #[clap(
        short = 'a',
        long = "attribute",
        value_name = "TEXT",
        help_heading = "Output"
    )]
    attributes: Vec<String>,

    /// Only attributes where value differs from base.
    #[clap(long, help_heading = "Output")]
    changed: bool,

    /// Report per attribute the modifiers its value was calculated from.
    #[clap(long, help_heading = "Output")]
    sources: bool,

    /// Report the fitting rules the fit breaks, instead of its attributes.
    #[clap(long, help_heading = "Output")]
    validate: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum Output {
    Json,
    Table,
}

#[derive(Clone, Copy, ValueEnum)]
enum SystemSecurity {
    HighSec,
    LowSec,
    NullSec,
    Wormhole,
}

fn parse_skill(value: &str) -> Result<(String, u8), String> {
    let (name, level) = value
        .rsplit_once('=')
        .ok_or_else(|| "expected NAME=LEVEL".to_string())?;
    let level = level
        .trim()
        .parse::<u8>()
        .ok()
        .filter(|level| *level <= 5)
        .ok_or_else(|| format!("level should be 0-5, not \"{}\"", level.trim()))?;
    Ok((name.trim().to_string(), level))
}

fn parse_damage_profile(value: &str) -> Result<DamageProfile, String> {
    let values = value
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<f64>()
                .ok()
                .filter(|value| *value >= 0.0)
                .ok_or_else(|| format!("expected a number of 0 or more, not \"{}\"", part.trim()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let [em, explosive, kinetic, thermal] = values[..] else {
        return Err("expected EM,EXPLOSIVE,KINETIC,THERMAL".to_string());
    };
    Ok(DamageProfile {
        em,
        explosive,
        kinetic,
        thermal,
    })
}

/* Without a profile of its own the hardener shifts towards the incoming damage. */
fn parse_reactive_armor(value: &str) -> Result<ReactiveArmor, String> {
    if value.is_empty() {
        return Ok(ReactiveArmor::DamageProfile);
    }
    Ok(ReactiveArmor::Profile(parse_damage_profile(value)?))
}

fn fail(message: String) -> ! {
    eprintln!("error: {message}");
    std::process::exit(1);
}

/// Set the state of every module from a 24-letter string; 8 letters for each
/// of the high, medium and low slots.
///
/// P = Passive (Offline), O = Online, A = Active, V = Overload.
///
/// A module set to a state it cannot reach is lowered again during calculation.
fn apply_state(fit: &mut Fit, state: &str) {
    let state: Vec<char> = state.chars().collect();
    if state.len() != 24 {
        panic!(
            "State should be 24 letters; 8 for each high/medium/low slot. P = Passive (Offline), O = Online, A = Active, V = Overload."
        );
    }

    type Rack = fn(u8) -> Slot;
    let racks: [(usize, Rack); 3] = [(0, Slot::High), (8, Slot::Medium), (16, Slot::Low)];

    for (offset, rack) in racks {
        for index in 0..8 {
            let Some(item) = fit.items.iter_mut().find(|item| item.slot == rack(index)) else {
                continue;
            };

            item.state = match state[offset + index as usize] {
                'P' => State::Offline,
                'O' => State::Online,
                'A' => State::Active,
                'V' => State::Overload,
                character => panic!("Invalid state character: {}", character),
            };
        }
    }
}

/// Later sources win: --skill-level, then the skills file, then --skill.
fn apply_skills(args: &Args, sde: &Sde, info_name: &InfoNameSde, fit: &mut Fit) {
    let skills = &mut fit.character.skills;

    if let Some(level) = args.skill_level {
        for r#type in sde.types() {
            if r#type.category_id() == SKILL_CATEGORY_ID {
                skills.insert(r#type.id(), level);
            }
        }
    }

    if let Some(skills_filename) = &args.skills_filename {
        let skills_file = std::fs::File::open(skills_filename).unwrap();
        let skills_file: BTreeMap<String, u8> = serde_json::from_reader(skills_file).unwrap();
        for (skill_id, level) in skills_file {
            let skill_id = skill_id.parse::<i32>().unwrap();
            skills.insert(skill_id, level);
        }
    }

    for (name, level) in &args.skills {
        let type_id = info_name
            .type_name_to_id(name)
            .filter(|type_id| {
                sde.get_type(*type_id)
                    .is_some_and(|r#type| r#type.category_id() == SKILL_CATEGORY_ID)
            })
            .unwrap_or_else(|| fail(format!("no such skill: {name}")));
        skills.insert(type_id, *level);
    }
}

struct Filter<'a> {
    info: &'a InfoSde<'a>,
    attributes: &'a [String],
    changed: bool,
}

impl Filter<'_> {
    fn is_active(&self) -> bool {
        !self.attributes.is_empty() || self.changed
    }

    fn apply(&self, result: &mut ItemResult) {
        result.attributes.retain(|attribute_id, attribute| {
            if self.changed && attribute.value == attribute.base {
                return false;
            }
            if self.attributes.is_empty() {
                return true;
            }
            let name = attribute_name(self.info, *attribute_id);
            self.attributes
                .iter()
                .any(|text| name.contains(text.as_str()))
        });
        if let Some(charge) = &mut result.charge {
            self.apply(charge);
        }
    }

    fn apply_all(&self, calculation: &mut Calculation) {
        self.apply(&mut calculation.ship);
        if let Some(mode) = &mut calculation.mode {
            self.apply(mode);
        }
        for item in &mut calculation.items {
            self.apply(item);
        }
        self.apply(&mut calculation.character);
    }
}

fn attribute_name(info: &InfoSde, attribute_id: i32) -> String {
    info.get_dogma_attribute(attribute_id).map_or_else(
        || attribute_id.to_string(),
        |attribute| attribute.name().to_string(),
    )
}

fn type_name(info: &InfoSde, type_id: i32) -> String {
    info.get_type(type_id)
        .map_or_else(|| type_id.to_string(), |r#type| r#type.name().to_string())
}

fn buff_name(info: &InfoSde, buff_id: i32) -> String {
    info.get_dbuff_collection(buff_id)
        .and_then(|buff| buff.display_name())
        .filter(|name| !name.is_empty())
        .map_or_else(|| format!("buff {buff_id}"), str::to_string)
}

fn slot_label(slot: Slot) -> String {
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

fn source_label(info: &InfoSde, fit: &Fit, from: SourceRef) -> String {
    match from {
        SourceRef::Ship => type_name(info, fit.ship.type_id),
        SourceRef::Mode => fit
            .ship
            .mode
            .map_or_else(|| "mode".to_string(), |mode| type_name(info, mode)),
        SourceRef::Character => "character".to_string(),
        SourceRef::Item { index } => type_name(info, fit.items[index].type_id),
        SourceRef::Charge { index } => fit.items[index].charge.as_ref().map_or_else(
            || "charge".to_string(),
            |charge| type_name(info, charge.type_id),
        ),
        SourceRef::Skill { type_id } => type_name(info, type_id),
        SourceRef::Projected { index } => type_name(info, fit.incoming.effects[index].type_id),
        SourceRef::Buff { id } => buff_name(info, id),
    }
}

fn violation_target(info: &InfoSde, fit: &Fit, target: Target) -> String {
    match target {
        Target::Ship => format!("ship: {}", type_name(info, fit.ship.type_id)),
        Target::Item { index } => format!(
            "{}: {}",
            slot_label(fit.items[index].slot),
            type_name(info, fit.items[index].type_id)
        ),
        Target::Charge { index } => format!(
            "{} charge: {}",
            slot_label(fit.items[index].slot),
            fit.items[index].charge.as_ref().map_or_else(
                || "charge".to_string(),
                |charge| type_name(info, charge.type_id),
            )
        ),
    }
}

/// The name serde gives a value, which is the one the JSON output shows.
fn label(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn violation_rule(info: &InfoSde, rule: Rule) -> String {
    match rule {
        Rule::Resource {
            resource,
            used,
            available,
        } => format!("{} used {used:.2} of {available:.2}", label(resource)),
        Rule::Slots {
            slot,
            used,
            available,
        } => format!("{} slots used {used} of {available}", label(slot)),
        Rule::WrongSlot { expected } => format!("belongs in a {} slot", label(expected)),
        Rule::SlotTaken => "another item is in this slot".to_string(),
        Rule::WrongSlotIndex { expected } => format!("belongs in slot {expected}"),
        Rule::SubsystemTaken => "another subsystem covers the same part".to_string(),
        Rule::Skill {
            type_id,
            required,
            level,
        } => format!(
            "{} at {required}, trained to {level}",
            type_name(info, type_id)
        ),
        Rule::RigSize { ship, item } => format!("rig size {item}, ship takes {ship}"),
        Rule::ShipRestricted => "cannot go on this ship".to_string(),
        Rule::CapitalItem => "a capital item on a ship that is not one".to_string(),
        Rule::MaxGroup {
            group_id,
            limit,
            used,
            allowed,
        } => format!(
            "{used} of group {group_id} {}, {allowed} allowed",
            label(limit)
        ),
        Rule::MaxType {
            type_id,
            used,
            allowed,
        } => format!("{used} of {}, {allowed} allowed", type_name(info, type_id)),
        Rule::ChargeGroup => "a charge the module does not take".to_string(),
        Rule::ChargeSize { module, charge } => {
            format!("charge size {charge}, module takes {module}")
        }
        rule => format!("{rule:?}"),
    }
}

fn print_violations(info: &InfoSde, fit: &Fit, violations: &[Violation]) {
    let targets: Vec<String> = violations
        .iter()
        .map(|violation| violation_target(info, fit, violation.target))
        .collect();
    let width = targets.iter().map(String::len).max().unwrap_or(0);

    for (target, violation) in targets.iter().zip(violations) {
        println!(
            "  {target:width$}  {}",
            violation_rule(info, violation.rule)
        );
    }
}

fn print_table(info: &InfoSde, fit: &Fit, calculation: &Calculation, hide_empty: bool) {
    let mut groups: Vec<(String, &ItemResult)> = Vec::new();

    groups.push((
        format!("ship: {}", type_name(info, fit.ship.type_id)),
        &calculation.ship,
    ));
    if let (Some(mode), Some(result)) = (fit.ship.mode, &calculation.mode) {
        groups.push((format!("mode: {}", type_name(info, mode)), result));
    }
    for (item, result) in fit.items.iter().zip(&calculation.items) {
        let quantity = match item.quantity {
            1 => String::new(),
            quantity => format!(" x{quantity}"),
        };
        let header = format!(
            "{}: {}{} ({})",
            slot_label(item.slot),
            type_name(info, item.type_id),
            quantity,
            format!("{:?}", result.state).to_lowercase()
        );
        groups.push((header, result));

        if let (Some(charge), Some(charge_result)) = (&item.charge, &result.charge) {
            let header = format!(
                "{} charge: {}",
                slot_label(item.slot),
                type_name(info, charge.type_id)
            );
            groups.push((header, charge_result));
        }
    }
    groups.push(("character".to_string(), &calculation.character));

    let width = groups
        .iter()
        .flat_map(|(_, result)| result.attributes.keys())
        .map(|attribute_id| attribute_name(info, *attribute_id).len())
        .max()
        .unwrap_or(0);

    /* Above the ship, as they are why some of its numbers moved. */
    for buff in &calculation.buffs {
        println!("== buff: {} ({:.4})", buff_name(info, buff.id), buff.value);
    }

    for (header, result) in groups {
        if hide_empty && result.attributes.is_empty() {
            continue;
        }

        println!("== {header}");
        for (attribute_id, attribute) in &result.attributes {
            println!(
                "  {:width$}  {:>14.4} -> {:>14.4}",
                attribute_name(info, *attribute_id),
                attribute.base,
                attribute.value
            );

            for source in &attribute.sources {
                let operator = serde_json::to_value(source.operator).unwrap();
                let quantity = match source.quantity {
                    1 => String::new(),
                    quantity => format!(" x{quantity}"),
                };
                let penalty = source
                    .penalty
                    .map_or_else(String::new, |penalty| format!("  penalty {penalty:.4}"));
                let applied = match source.applied {
                    true => "",
                    false => "  (not applied)",
                };
                println!(
                    "      {}{}: {} {:.4}{}{}",
                    source_label(info, fit, source.from),
                    quantity,
                    operator.as_str().unwrap_or_default(),
                    source.value,
                    penalty,
                    applied
                );
            }
        }
    }
}

pub fn main() {
    let args: Args = Args::parse();

    /* "eft" can come either from stdin, or from eft-file parameter. */
    let eft = match &args.eft_filename {
        Some(filename) => std::fs::read_to_string(filename).unwrap(),
        None => {
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer).unwrap();
            buffer
        }
    };

    let sde_bytes = std::fs::read(&args.sde_filename).unwrap();
    let sde = Sde::new(&sde_bytes).unwrap();

    /* English names come from the SDE; the names file only widens that to the
     * other seven languages, so a missing one is not fatal. */
    let names_bytes = std::fs::read(&args.names_filename).ok();
    let names = names_bytes.as_ref().map(|bytes| Names::new(bytes).unwrap());

    let info_name = InfoNameSde::new(&sde, names.as_ref()).unwrap();

    let mut fit = eft::load_eft(&info_name, &eft).unwrap();

    /* Without this the states from the EFT are used. */
    if let Some(state) = &args.state {
        apply_state(&mut fit, state);
    }

    apply_skills(&args, &sde, &info_name, &mut fit);

    fit.environment.security = match args.security {
        SystemSecurity::HighSec => Security::HighSec,
        SystemSecurity::LowSec => Security::LowSec,
        SystemSecurity::NullSec => Security::NullSec,
        SystemSecurity::Wormhole => Security::Wormhole,
    };
    if let Some(damage_profile) = args.damage_profile {
        fit.environment.damage_profile = damage_profile;
    }
    if let Some(reactive_armor) = args.reactive_armor {
        fit.environment.reactive_armor = reactive_armor;
    }
    let mut beacons = Vec::new();
    for name in &args.beacons {
        /* Beacons are spread over several groups, but never leave Celestial. */
        let type_id = info_name
            .type_name_to_id(name)
            .filter(|type_id| {
                sde.get_type(*type_id)
                    .is_some_and(|r#type| r#type.category_id() == CELESTIAL_CATEGORY_ID)
            })
            .unwrap_or_else(|| fail(format!("no such beacon: {name}")));
        beacons.push(type_id);
    }

    let info = InfoSde::new(&sde);
    for type_id in beacons {
        fit.incoming
            .extend(esf_dogma_engine::beacon(&info, type_id));
    }
    if args.eft {
        let eft = eft::save_eft(&info, &fit).unwrap_or_else(|error| fail(error.to_string()));
        print!("{eft}");
        return;
    }

    let options = Options {
        sources: args.sources,
        validate: args.validate,
    };
    let mut calculation = esf_dogma_engine::calculate(&info, &fit, &options);

    let output = args
        .output
        .unwrap_or(match std::io::stdout().is_terminal() {
            true => Output::Table,
            false => Output::Json,
        });

    if let Some(violations) = &calculation.violations {
        match output {
            Output::Json => println!("{}", serde_json::to_string(violations).unwrap()),
            Output::Table => print_violations(&info, &fit, violations),
        }
        return;
    }

    let filter = Filter {
        info: &info,
        attributes: &args.attributes,
        changed: args.changed,
    };
    filter.apply_all(&mut calculation);

    match output {
        Output::Json => println!("{}", serde_json::to_string(&calculation).unwrap()),
        Output::Table => print_table(&info, &fit, &calculation, filter.is_active()),
    }
}
