use std::collections::HashMap;

use crate::data_types;
use crate::info::InfoName;

pub struct EftCargo {
    pub type_id: i32,
    pub quantity: i32,
}

pub struct EftFit {
    pub name: String,
    pub esf_fit: data_types::EsfFit,
    pub cargo: Vec<EftCargo>,
}

fn section_iter(eft_lines: Vec<&str>) -> impl Iterator<Item = Vec<&str>> {
    let mut section: Vec<&str> = Vec::new();

    let mut eft_lines = eft_lines
        .into_iter()
        .skip(1)
        .fold(Vec::new(), |mut sections, line| {
            if line.is_empty() {
                if section.len() > 0 {
                    sections.push(section.clone());
                    section.clear();
                }
            } else {
                section.push(line);
            }

            sections
        });

    if section.len() > 0 {
        eft_lines.push(section);
    }

    eft_lines.into_iter()
}

fn find_slot_type_index(
    info: &impl InfoName,
    type_id: i32,
    module_slots: &mut HashMap<data_types::EsfSlotType, i32>,
) -> Option<(data_types::EsfSlotType, i32)> {
    let effects = info.get_dogma_effects(type_id);

    for effect in &effects {
        match effect.effectID {
            11 => {
                let index = module_slots
                    .entry(data_types::EsfSlotType::Low)
                    .or_insert(0);
                *index += 1;
                return Some((data_types::EsfSlotType::Low, *index - 1));
            }
            12 => {
                let index = module_slots
                    .entry(data_types::EsfSlotType::High)
                    .or_insert(0);
                *index += 1;
                return Some((data_types::EsfSlotType::High, *index - 1));
            }
            13 => {
                let index = module_slots
                    .entry(data_types::EsfSlotType::Medium)
                    .or_insert(0);
                *index += 1;
                return Some((data_types::EsfSlotType::Medium, *index - 1));
            }
            2663 => {
                let index = module_slots
                    .entry(data_types::EsfSlotType::Rig)
                    .or_insert(0);
                *index += 1;
                return Some((data_types::EsfSlotType::Rig, *index - 1));
            }
            3772 => {
                let index = module_slots
                    .entry(data_types::EsfSlotType::SubSystem)
                    .or_insert(0);
                *index += 1;
                return Some((data_types::EsfSlotType::SubSystem, *index - 1));
            }
            6306 => {
                let index = module_slots
                    .entry(data_types::EsfSlotType::Service)
                    .or_insert(0);
                *index += 1;
                return Some((data_types::EsfSlotType::Service, *index - 1));
            }
            _ => {}
        }
    }

    None
}

/* Load an EFT string and return an ESF fit structure. */
pub fn load_eft(info: &impl InfoName, eft: &String) -> Result<EftFit, String> {
    let eft_lines: Vec<&str> = eft.lines().collect();

    /* First line of an EFT always start with "[ship-type,name]". */
    let header = eft_lines[0];
    if !header.starts_with("[") || !header.ends_with("]") {
        return Err("Invalid EFT header".to_string());
    }
    let header = header.trim_start_matches("[").trim_end_matches("]");

    let header = header.split(",").collect::<Vec<&str>>();
    let ship_type_name = header[0];
    let name = header[1];

    let mut eft_fit = EftFit {
        name: name.to_string(),
        esf_fit: data_types::EsfFit {
            ship_type_id: info.type_name_to_id(ship_type_name),
            modules: Vec::new(),
            drones: Vec::new(),
        },
        cargo: Vec::new(),
    };

    /* An EFT has sections, which are seperated by a new line. */
    for section in section_iter(eft_lines) {
        /* This is a module section if none of the strings end with "x<quantity>". */
        let is_module_section = !section.iter().all(|line| {
            let x_pos = line.find("x");
            x_pos.map_or(false, |x_pos| {
                line[x_pos + 1..].chars().all(|c| c.is_numeric())
            })
        });

        match is_module_section {
            true => {
                let mut module_slots: HashMap<data_types::EsfSlotType, i32> = HashMap::new();

                for line in section {
                    if line.starts_with("[Empty") {
                        let slot_type = match line {
                            "[Empty High Slots]" => data_types::EsfSlotType::High,
                            "[Empty Medium Slots]" => data_types::EsfSlotType::Medium,
                            "[Empty Low Slots]" => data_types::EsfSlotType::Low,
                            "[Empty Rig Slots]" => data_types::EsfSlotType::Rig,
                            "[Empty Subsystem Slots]" => data_types::EsfSlotType::SubSystem,
                            _ => panic!("Invalid slot type"),
                        };

                        let index = module_slots.entry(slot_type).or_insert(0);
                        *index += 1;
                        continue;
                    }

                    /* Can either be "<Module Name>" or "<Module Name>, <Charge Name>". */
                    let comma_pos = line.find(",");

                    let (module_name, charge_name) = match comma_pos {
                        Some(comma_pos) => {
                            let module_name = line[..comma_pos].trim();
                            let charge_name = line[comma_pos + 1..].trim();
                            (module_name, Some(charge_name))
                        }
                        None => {
                            let module_name = line.trim();
                            (module_name, None)
                        }
                    };

                    let module_type_id = info.type_name_to_id(module_name);
                    let charge_type_id =
                        charge_name.map(|charge_name| info.type_name_to_id(charge_name));

                    let slot_type_index =
                        find_slot_type_index(info, module_type_id, &mut module_slots);
                    if slot_type_index.is_none() {
                        return Err(format!("Module {} does not fit in any slot", module_name));
                    }
                    let (slot_type, index) = slot_type_index.unwrap();

                    let module = data_types::EsfModule {
                        type_id: module_type_id,
                        slot: data_types::EsfSlot {
                            r#type: slot_type,
                            index,
                        },
                        state: data_types::EsfState::Active,
                        charge: charge_type_id.map(|charge_type_id| data_types::EsfCharge {
                            type_id: charge_type_id,
                        }),
                    };

                    eft_fit.esf_fit.modules.push(module);
                }
            }
            false => {
                let mut items = Vec::new();

                let mut are_drones = true;

                for line in section {
                    /* Always in the form "<Type Name> x<Quantity>" */
                    let x_pos = line.find("x").unwrap();
                    let type_name = line[..x_pos].trim();
                    let quantity = line[x_pos + 1..].parse::<i32>().unwrap();

                    let type_id = info.type_name_to_id(type_name);

                    let r#type = info.get_type(type_id);
                    are_drones = are_drones && r#type.categoryID == 18; // Drone

                    items.push((type_id, quantity));
                }

                if are_drones {
                    for (type_id, quantity) in items {
                        for _ in 0..quantity {
                            let drone = data_types::EsfDrone {
                                type_id,
                                state: data_types::EsfState::Active,
                            };

                            eft_fit.esf_fit.drones.push(drone);
                        }
                    }
                } else {
                    for (type_id, quantity) in items {
                        let cargo = EftCargo { type_id, quantity };

                        eft_fit.cargo.push(cargo);
                    }
                }
            }
        }
    }

    Ok(eft_fit)
}
