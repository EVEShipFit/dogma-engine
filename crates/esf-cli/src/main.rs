use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;

use clap::Parser;

use esf_data::sde;
use esf_dogma::calculate;
use esf_dogma::fit::{Fit, Slot, State};
use esf_format::eft;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    state: Option<String>,

    #[clap(short, long)]
    eft_filename: Option<PathBuf>,

    #[clap(short = 'f', long)]
    skills_filename: Option<PathBuf>,

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

    let sde_bytes = std::fs::read(&args.sde_filename).unwrap();
    let sde = sde::Sde::new(&sde_bytes).unwrap();

    /* English names come from the SDE; the names file only widens that to the
     * other seven languages, so a missing one is not fatal. */
    let names_bytes = std::fs::read(&args.names_filename).ok();
    let names = names_bytes
        .as_ref()
        .map(|bytes| sde::Names::new(bytes).unwrap());

    let info_name = sde::InfoNameSde::new(&sde, names.as_ref()).unwrap();

    let mut fit = eft::load_eft(&info_name, &eft).unwrap();

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
        let skills_file: BTreeMap<String, u8> = serde_json::from_reader(skills_file).unwrap();
        for (skill_id, level) in skills_file {
            let skill_id = skill_id.parse::<i32>().unwrap();
            fit.character.skills.insert(skill_id, level);
        }
    }

    let info = sde::InfoSde::new(&sde);
    let calculation = calculate::calculate(&info, &fit);

    println!("{}", serde_json::to_string(&calculation).unwrap());
}
