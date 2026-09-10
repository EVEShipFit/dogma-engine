use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;

use clap::Parser;

use esf_dogma_engine::calculate;
use esf_dogma_engine::data_types::{EsfFit, EsfSlotType, EsfState};
use esf_dogma_engine::eft;
use esf_dogma_engine::rust;

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
}

/// Set the state of every module from a 24-letter string; 8 letters for each
/// of the high, medium and low slots.
///
/// P = Passive (Offline), O = Online, A = Active, V = Overload.
///
/// A module set to a state it cannot reach is lowered again during calculation.
fn apply_state(fit: &mut EsfFit, state: &str) {
    let state: Vec<char> = state.chars().collect();
    if state.len() != 24 {
        panic!(
            "State should be 24 letters; 8 for each high/medium/low slot. P = Passive (Offline), O = Online, A = Active, V = Overload."
        );
    }

    for (offset, slot_type) in [
        (0, EsfSlotType::High),
        (8, EsfSlotType::Medium),
        (16, EsfSlotType::Low),
    ]
    .iter()
    {
        for index in 0..8 {
            let module = fit
                .modules
                .iter_mut()
                .find(|module| module.slot.index == index && module.slot.r#type == *slot_type);

            let Some(module) = module else { continue };

            module.state = match state[offset + index as usize] {
                'P' => EsfState::Passive,
                'O' => EsfState::Online,
                'A' => EsfState::Active,
                'V' => EsfState::Overload,
                character => panic!("Invalid state character: {}", character),
            };
        }
    }
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

    /* Without this the states from the EFT are used. */
    if let Some(state) = args.state {
        apply_state(&mut fit, &state);
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
    let statistics = calculate::calculate(&info);
    let output = rust::Output::new(&info, &statistics);

    println!("{}", serde_json::to_string(&output).unwrap());
}
