use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::Parser;

use esf_dogma_engine::calculate;
use esf_dogma_engine::data_types;
use esf_dogma_engine::rust;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "node_modules/@eveshipfit/data/dist/sde")]
    protobuf_location: PathBuf,
}

pub fn main() {
    let args: Args = Args::parse();

    let fit: data_types::EsfFit = data_types::EsfFit {
        ship_type_id: 22452,
        modules: vec![],
        drones: vec![],
    };
    let skills: BTreeMap<i32, i32> = BTreeMap::new();

    let data = rust::Data::new(&args.protobuf_location);
    let info = rust::InfoMain::new(fit, skills, &data);

    let statistics = calculate::calculate(&info);

    let json = serde_json::to_string_pretty(&statistics.hull).unwrap();
    println!("{}", json);
}
