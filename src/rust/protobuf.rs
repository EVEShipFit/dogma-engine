use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use prost::Message;

use super::esf_data;

fn load_protobuf<T: Message + std::default::Default>(
    path: &PathBuf,
    name: &str,
) -> Result<T, String> {
    let mut filename = path.join(name);
    filename.set_extension("pb2");

    let mut file = File::open(filename).unwrap();

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    let object = T::decode(buf.as_slice());
    match object {
        Ok(object) => Ok(object),
        Err(e) => Err(format!("Error: {:?}", e)),
    }
}

pub struct Data {
    pub types: HashMap<i32, esf_data::types::Type>,
    pub type_dogma: HashMap<i32, esf_data::type_dogma::TypeDogmaEntry>,
    pub dogma_attributes: HashMap<i32, esf_data::dogma_attributes::DogmaAttribute>,
    pub dogma_effects: HashMap<i32, esf_data::dogma_effects::DogmaEffect>,
}

impl Data {
    pub fn new(path: &PathBuf) -> Data {
        let dogma_attributes: esf_data::DogmaAttributes =
            load_protobuf(path, "dogmaAttributes").unwrap();
        let dogma_effects: esf_data::DogmaEffects = load_protobuf(path, "dogmaEffects").unwrap();
        let type_dogma: esf_data::TypeDogma = load_protobuf(path, "typeDogma").unwrap();
        let types: esf_data::Types = load_protobuf(path, "types").unwrap();

        Data {
            types: types.entries,
            type_dogma: type_dogma.entries,
            dogma_attributes: dogma_attributes.entries,
            dogma_effects: dogma_effects.entries,
        }
    }
}
