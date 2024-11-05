use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;

use clap::Parser;

use esf_dogma_engine::calculate;
use esf_dogma_engine::calculate::item::Item;
use esf_dogma_engine::data_types::EsfSlotType;
use esf_dogma_engine::data_types::EsfState;
use esf_dogma_engine::eft;
use esf_dogma_engine::info::Info;
use esf_dogma_engine::rust;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct OutputCapacitor {
    stable: bool,
    depletes_in: f64,
    capacity: f64,
    recharge: f64,
    peak: f64,
    percentage: f64,
}

#[derive(Debug, Serialize)]
struct OutputOffense {
    dps: f64,
    dps_with_reload: f64,
    alpha: f64,
    drone_dps: f64,
}

#[derive(Debug, Serialize)]
struct OutputDefenseRecharge {
    passive: f64,
    shield: f64,
    armor: f64,
    hull: f64,
}

#[derive(Debug, Serialize)]
struct OutputDefenseResist {
    em: f64,
    therm: f64,
    kin: f64,
    expl: f64,
}

#[derive(Debug, Serialize)]
struct OutputDefenseShield {
    resist: OutputDefenseResist,
    hp: f64,
    recharge: f64,
}

#[derive(Debug, Serialize)]
struct OutputDefenseArmor {
    resist: OutputDefenseResist,
    hp: f64,
}

#[derive(Debug, Serialize)]
struct OutputDefenseStructure {
    resist: OutputDefenseResist,
    hp: f64,
}

#[derive(Debug, Serialize)]
struct OutputDefense {
    recharge: OutputDefenseRecharge,
    shield: OutputDefenseShield,
    armor: OutputDefenseArmor,
    structure: OutputDefenseStructure,
    ehp: f64,
}

#[derive(Debug, Serialize)]
struct OutputTargeting {
    lock_range: f64,
    sensor_strength: f64,
    scan_resolution: f64,
    signature_radius: f64,
    max_locked_targets: f64,
}

#[derive(Debug, Serialize)]
struct OutputNavigation {
    speed: f64,
    mass: f64,
    agility: f64,
    warp_speed: f64,
    align_time: f64,
}

#[derive(Debug, Serialize)]
struct OutputDrones {
    dps: f64,
    bandwidth_load: f64,
    bandwidth: f64,
    range: f64,
    active: f64,
    capacity_load: f64,
    capacity: f64,
}

#[derive(Debug, Serialize)]
struct OutputCpu {
    free: f64,
    capacity: f64,
}

#[derive(Debug, Serialize)]
struct OutputPower {
    free: f64,
    capacity: f64,
}

#[derive(Debug, Serialize)]
struct OutputSlots {
    hi_1: String,
    hi_2: String,
    hi_3: String,
    hi_4: String,
    hi_5: String,
    hi_6: String,
    hi_7: String,
    hi_8: String,
    med_1: String,
    med_2: String,
    med_3: String,
    med_4: String,
    med_5: String,
    med_6: String,
    med_7: String,
    med_8: String,
    lo_1: String,
    lo_2: String,
    lo_3: String,
    lo_4: String,
    lo_5: String,
    lo_6: String,
    lo_7: String,
    lo_8: String,
}

#[derive(Debug, Serialize)]
struct Output {
    capacitor: OutputCapacitor,
    offense: OutputOffense,
    defense: OutputDefense,
    targeting: OutputTargeting,
    navigation: OutputNavigation,
    drones: OutputDrones,
    cpu: OutputCpu,
    power: OutputPower,
    slots: OutputSlots,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    state: Option<String>,

    #[clap(short, long)]
    eft_filename: Option<PathBuf>,

    #[clap(short = 'f', long)]
    skills_filename: Option<PathBuf>,

    #[clap(short, long, default_value = "node_modules/@eveshipfit/data/dist/sde")]
    protobuf_location: PathBuf,

    #[clap(
        short,
        long,
        value_delimiter = ',',
        default_value = "0.25,0.25,0.25,0.25"
    )]
    damage_profile: Vec<f64>,
}

fn get_attribute_by_name(
    info: &impl Info,
    attributes: &BTreeMap<i32, calculate::item::Attribute>,
    name: &str,
) -> f64 {
    let attribute_id = info.attribute_name_to_id(name);
    let default_attribute = info.get_dogma_attribute(attribute_id);

    let attribute = attributes
        .iter()
        .find(|attribute| *attribute.0 == attribute_id);
    if let Some(attribute) = attribute {
        attribute.1.value.unwrap_or(default_attribute.defaultValue)
    } else {
        default_attribute.defaultValue
    }
}

fn effect_category_to_name(
    items: &Vec<Item>,
    slot_type: calculate::item::SlotType,
    index: i32,
) -> String {
    let item = items
        .iter()
        .find(|item| item.slot.index == Some(index) && item.slot.r#type == slot_type);

    if let Some(item) = item {
        match item.state {
            calculate::item::EffectCategory::Passive => "passive",
            calculate::item::EffectCategory::Online => "online",
            calculate::item::EffectCategory::Active => "active",
            calculate::item::EffectCategory::Overload => "overload",
            _ => "unknown",
        }
    } else {
        "empty"
    }
    .to_string()
}

pub fn main() {
    let args: Args = Args::parse();

    /* "eft" can come either from stdin, or from eft-file parameter. */
    let eft = match args.eft_filename {
        Some(filename) => std::fs::read_to_string(filename).unwrap(),
        None => {
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer).unwrap();
            buffer
        }
    };

    let data = rust::Data::new(&args.protobuf_location);
    let info_name = rust::InfoNameMain::new(&data);

    let mut fit = eft::load_eft(&info_name, &eft).unwrap().esf_fit;
    let mut skills: BTreeMap<i32, i32> = BTreeMap::new();

    /* Update the state of the modules. If a module is set to a state it
     * cannot reach, it will automatically be set to a lower state during
     * calculation. By default everything will be set to Active. */
    if let Some(state) = args.state {
        if state.len() != 24 {
            panic!("State should be 24 letters; 8 for each high/medium/low slot. P = Passive (Offline), O = Online, A = Active, V = Overload.");
        }

        let state = state.chars().collect::<Vec<char>>();
        for i in 0..8 {
            let module = fit
                .modules
                .iter_mut()
                .find(|module| module.slot.index == i && module.slot.r#type == EsfSlotType::High);
            if let Some(module) = module {
                module.state = match state[i as usize] {
                    'P' => EsfState::Passive,
                    'O' => EsfState::Online,
                    'A' => EsfState::Active,
                    'V' => EsfState::Overload,
                    _ => panic!("Invalid state character: {}", state[i as usize]),
                };
            }
        }
        for i in 8..16 {
            let module = fit.modules.iter_mut().find(|module| {
                module.slot.index == i - 8 && module.slot.r#type == EsfSlotType::Medium
            });
            if let Some(module) = module {
                module.state = match state[i as usize] {
                    'P' => EsfState::Passive,
                    'O' => EsfState::Online,
                    'A' => EsfState::Active,
                    'V' => EsfState::Overload,
                    _ => panic!("Invalid state character: {}", state[i as usize]),
                };
            }
        }
        for i in 16..24 {
            let module = fit.modules.iter_mut().find(|module| {
                module.slot.index == i - 16 && module.slot.r#type == EsfSlotType::Low
            });
            if let Some(module) = module {
                module.state = match state[i as usize] {
                    'P' => EsfState::Passive,
                    'O' => EsfState::Online,
                    'A' => EsfState::Active,
                    'V' => EsfState::Overload,
                    _ => panic!("Invalid state character: {}", state[i as usize]),
                };
            }
        }
    }

    /* Load the skills if a skills-file is given. Be mindful:
     * - Skills not in the list are assumed L1 (by dogma-data).
     * - Skills injected but not trained are L0.
     */
    if let Some(skills_filename) = args.skills_filename {
        let skills_file = std::fs::File::open(skills_filename).unwrap();
        let skills_file: BTreeMap<String, i32> = serde_json::from_reader(skills_file).unwrap();
        for (skill_id, level) in skills_file {
            let skill_id = skill_id.parse::<i32>().unwrap();
            skills.insert(skill_id, level);
        }
    }

    let info = rust::InfoMain::new(fit, skills, &data);
    let statistics = calculate::calculate(
        &info,
        calculate::DamageProfile {
            em: args.damage_profile[0],
            explosive: args.damage_profile[3],
            kinetic: args.damage_profile[2],
            thermal: args.damage_profile[1],
        },
    );

    let output = Output {
        capacitor: OutputCapacitor {
            stable: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "capacitorDepletesIn",
            ) == -1.0,
            depletes_in: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "capacitorDepletesIn",
            ),
            capacity: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "capacitorCapacity",
            )
            .floor(),
            recharge: get_attribute_by_name(&info, &statistics.hull.attributes, "rechargeRate")
                / 1000.0,
            peak: get_attribute_by_name(&info, &statistics.hull.attributes, "capacitorPeakDelta"),
            percentage: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "capacitorPeakDeltaPercentage",
            ),
        },
        offense: OutputOffense {
            dps: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "damagePerSecondWithoutReload",
            ),
            dps_with_reload: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "damagePerSecondWithReload",
            ),
            alpha: get_attribute_by_name(&info, &statistics.hull.attributes, "damageAlpha"),
            drone_dps: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "droneDamagePerSecond",
            ),
        },
        defense: OutputDefense {
            recharge: OutputDefenseRecharge {
                passive: get_attribute_by_name(
                    &info,
                    &statistics.hull.attributes,
                    "passiveShieldRechargeRate",
                ),
                shield: get_attribute_by_name(
                    &info,
                    &statistics.hull.attributes,
                    "shieldBoostRate",
                ),
                armor: get_attribute_by_name(&info, &statistics.hull.attributes, "armorRepairRate"),
                hull: get_attribute_by_name(&info, &statistics.hull.attributes, "hullRepairRate"),
            },
            shield: OutputDefenseShield {
                resist: OutputDefenseResist {
                    em: (1.0
                        - get_attribute_by_name(
                            &info,
                            &statistics.hull.attributes,
                            "shieldEmDamageResonance",
                        ))
                        * 100.0,
                    therm: (1.0
                        - get_attribute_by_name(
                            &info,
                            &statistics.hull.attributes,
                            "shieldThermalDamageResonance",
                        ))
                        * 100.0,
                    kin: (1.0
                        - get_attribute_by_name(
                            &info,
                            &statistics.hull.attributes,
                            "shieldKineticDamageResonance",
                        ))
                        * 100.0,
                    expl: (1.0
                        - get_attribute_by_name(
                            &info,
                            &statistics.hull.attributes,
                            "shieldExplosiveDamageResonance",
                        ))
                        * 100.0,
                },
                hp: get_attribute_by_name(&info, &statistics.hull.attributes, "shieldCapacity"),
                recharge: get_attribute_by_name(
                    &info,
                    &statistics.hull.attributes,
                    "shieldRechargeRate",
                ) / 1000.0,
            },
            armor: OutputDefenseArmor {
                resist: OutputDefenseResist {
                    em: (1.0
                        - get_attribute_by_name(
                            &info,
                            &statistics.hull.attributes,
                            "armorEmDamageResonance",
                        ))
                        * 100.0,
                    therm: (1.0
                        - get_attribute_by_name(
                            &info,
                            &statistics.hull.attributes,
                            "armorThermalDamageResonance",
                        ))
                        * 100.0,
                    kin: (1.0
                        - get_attribute_by_name(
                            &info,
                            &statistics.hull.attributes,
                            "armorKineticDamageResonance",
                        ))
                        * 100.0,
                    expl: (1.0
                        - get_attribute_by_name(
                            &info,
                            &statistics.hull.attributes,
                            "armorExplosiveDamageResonance",
                        ))
                        * 100.0,
                },
                hp: get_attribute_by_name(&info, &statistics.hull.attributes, "armorHP"),
            },
            structure: OutputDefenseStructure {
                resist: OutputDefenseResist {
                    em: (1.0
                        - get_attribute_by_name(
                            &info,
                            &statistics.hull.attributes,
                            "emDamageResonance",
                        ))
                        * 100.0,
                    therm: (1.0
                        - get_attribute_by_name(
                            &info,
                            &statistics.hull.attributes,
                            "thermalDamageResonance",
                        ))
                        * 100.0,
                    kin: (1.0
                        - get_attribute_by_name(
                            &info,
                            &statistics.hull.attributes,
                            "kineticDamageResonance",
                        ))
                        * 100.0,
                    expl: (1.0
                        - get_attribute_by_name(
                            &info,
                            &statistics.hull.attributes,
                            "explosiveDamageResonance",
                        ))
                        * 100.0,
                },
                hp: get_attribute_by_name(&info, &statistics.hull.attributes, "hp"),
            },
            ehp: get_attribute_by_name(&info, &statistics.hull.attributes, "ehp"),
        },
        targeting: OutputTargeting {
            lock_range: get_attribute_by_name(&info, &statistics.hull.attributes, "maxTargetRange")
                / 1000.0,
            sensor_strength: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "scanStrength",
            ),
            scan_resolution: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "scanResolution",
            ),
            signature_radius: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "signatureRadius",
            ),
            max_locked_targets: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "maxLockedTargets",
            ),
        },
        navigation: OutputNavigation {
            speed: get_attribute_by_name(&info, &statistics.hull.attributes, "maxVelocity"),
            mass: get_attribute_by_name(&info, &statistics.hull.attributes, "mass") / 1000.0,
            agility: get_attribute_by_name(&info, &statistics.hull.attributes, "agility"),
            warp_speed: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "warpSpeedMultiplier",
            ),
            align_time: get_attribute_by_name(&info, &statistics.hull.attributes, "alignTime"),
        },
        drones: OutputDrones {
            dps: get_attribute_by_name(&info, &statistics.hull.attributes, "droneDamagePerSecond"),
            bandwidth_load: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "droneBandwidthLoad",
            ),
            bandwidth: get_attribute_by_name(&info, &statistics.hull.attributes, "droneBandwidth"),
            capacity_load: get_attribute_by_name(
                &info,
                &statistics.hull.attributes,
                "droneCapacityLoad",
            ),
            capacity: get_attribute_by_name(&info, &statistics.hull.attributes, "droneCapacity"),
            range: get_attribute_by_name(
                &info,
                &statistics.char.attributes,
                "droneControlDistance",
            ) / 1000.0,
            active: get_attribute_by_name(&info, &statistics.hull.attributes, "droneActive"),
        },
        cpu: OutputCpu {
            free: get_attribute_by_name(&info, &statistics.hull.attributes, "cpuFree"),
            capacity: get_attribute_by_name(&info, &statistics.hull.attributes, "cpuOutput"),
        },
        power: OutputPower {
            free: get_attribute_by_name(&info, &statistics.hull.attributes, "powerFree"),
            capacity: get_attribute_by_name(&info, &statistics.hull.attributes, "powerOutput"),
        },
        slots: OutputSlots {
            hi_1: effect_category_to_name(&statistics.items, calculate::item::SlotType::High, 0),
            hi_2: effect_category_to_name(&statistics.items, calculate::item::SlotType::High, 1),
            hi_3: effect_category_to_name(&statistics.items, calculate::item::SlotType::High, 2),
            hi_4: effect_category_to_name(&statistics.items, calculate::item::SlotType::High, 3),
            hi_5: effect_category_to_name(&statistics.items, calculate::item::SlotType::High, 4),
            hi_6: effect_category_to_name(&statistics.items, calculate::item::SlotType::High, 5),
            hi_7: effect_category_to_name(&statistics.items, calculate::item::SlotType::High, 6),
            hi_8: effect_category_to_name(&statistics.items, calculate::item::SlotType::High, 7),
            med_1: effect_category_to_name(&statistics.items, calculate::item::SlotType::Medium, 0),
            med_2: effect_category_to_name(&statistics.items, calculate::item::SlotType::Medium, 1),
            med_3: effect_category_to_name(&statistics.items, calculate::item::SlotType::Medium, 2),
            med_4: effect_category_to_name(&statistics.items, calculate::item::SlotType::Medium, 3),
            med_5: effect_category_to_name(&statistics.items, calculate::item::SlotType::Medium, 4),
            med_6: effect_category_to_name(&statistics.items, calculate::item::SlotType::Medium, 5),
            med_7: effect_category_to_name(&statistics.items, calculate::item::SlotType::Medium, 6),
            med_8: effect_category_to_name(&statistics.items, calculate::item::SlotType::Medium, 7),
            lo_1: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 0),
            lo_2: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 1),
            lo_3: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 2),
            lo_4: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 3),
            lo_5: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 4),
            lo_6: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 5),
            lo_7: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 6),
            lo_8: effect_category_to_name(&statistics.items, calculate::item::SlotType::Low, 7),
        },
    };

    println!("{}", serde_json::to_string(&output).unwrap());
}
