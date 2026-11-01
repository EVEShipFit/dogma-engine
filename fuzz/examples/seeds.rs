//! Writes every fit of the regression tests into `corpus/`, in every format.

use std::fs;
use std::path::Path;

use esf_dogma_engine::{Character, Environment, Fit, Ship, Slot, beacon};
use esf_format::{eft, esf, esi};

const RIFTER: i32 = 587;

fn esf_fits(source: &str) -> Vec<&str> {
    source
        .split("r#\"")
        .skip(1)
        .filter_map(|rest| rest.split_once("\"#").map(|(fit, _)| fit))
        .filter(|fit| fit.starts_with("%esf/1"))
        .collect()
}

fn dna(fit: &Fit) -> String {
    let mut dna = fit.ship.type_id.to_string();
    for item in &fit.items {
        let cargo = match item.slot {
            Slot::Cargo => "_",
            _ => "",
        };
        dna += &format!(":{}{cargo};{}", item.type_id, item.quantity);
    }
    dna + "::"
}

fn walk(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        match path.is_dir() {
            true => walk(&path, files),
            false => files.push(path),
        }
    }
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let corpus = root.join("corpus");
    for target in [
        "dna", "eft", "esf", "esi", "fit", "fit_json", "killmail", "link",
    ] {
        fs::create_dir_all(corpus.join(target)).unwrap();
    }
    let write = |target: &str, name: &str, data: &[u8]| {
        let path = corpus.join(target).join(name);
        if fs::read(&path).ok().as_deref() != Some(data) {
            fs::write(path, data).unwrap();
        }
    };

    let info = esf_fuzz::info();
    let mut files = Vec::new();
    walk(&root.join("../tests/fits"), &mut files);

    let mut count = 0;
    for file in files {
        let source = fs::read_to_string(&file).unwrap();
        for text in esf_fits(&source) {
            let Ok(fits) = esf::load_esf(&info, text.as_bytes()) else {
                continue;
            };
            let name = format!("seed-{count}");
            count += 1;

            write("esf", &name, text.as_bytes());
            if let Ok(binary) = esf::save_esf_binary(&info, &fits) {
                write("esf", &format!("{name}-binary"), &binary);
                let link = format!("link:{}", esf::encode_base64url(&binary));
                write("esf", &format!("{name}-link"), link.as_bytes());
            }

            let Some(fit) = esf::main_fit(&fits).and_then(|fit| esf::to_fit(&info, fit).ok())
            else {
                continue;
            };
            write(
                "fit_json",
                &name,
                serde_json::to_string(&fit).unwrap().as_bytes(),
            );
            write("dna", &name, dna(&fit).as_bytes());
            let fitting = esi::save_esi_fitting(&info, &fit);
            write(
                "esi",
                &name,
                serde_json::to_string(&fitting).unwrap().as_bytes(),
            );
            if let Ok(text) = eft::save_eft(&info, &fit) {
                write("eft", &name, text.as_bytes());
                write("link", &name, format!("eft:{text}").as_bytes());
            }
        }
    }
    for &type_id in esf_fuzz::fit::BEACONS.iter() {
        let fit = Fit {
            name: None,
            ship: Ship {
                type_id: RIFTER,
                mode: None,
            },
            items: Vec::new(),
            character: Character::default(),
            environment: Environment::default(),
            incoming: beacon(&info, type_id),
        };
        let json = serde_json::to_string(&fit).unwrap();
        write("fit_json", &format!("beacon-{type_id}"), json.as_bytes());
    }

    println!(
        "{count} fits and {} beacons written to {}",
        esf_fuzz::fit::BEACONS.len(),
        corpus.display()
    );
}
