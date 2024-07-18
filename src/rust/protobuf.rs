use std::{fs::File, io::Read, path::PathBuf};

use prost::Message;

pub fn load_from_npm<T: Message + std::default::Default>(
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
