use super::names_generated::eve;
use crate::Error;

/// Reader for `names.dat`, the type names in every language EVE supports.
///
/// Only an EFT import needs this; everything else can be answered from
/// `sde.dat` alone, which is why it is a separate file.
pub struct Names {
    bytes: Vec<u8>,
}

impl Names {
    /// Check the bytes are a valid names file.
    pub fn new(bytes: Vec<u8>) -> Result<Names, Error> {
        eve::root_as_names(&bytes).map_err(Error::InvalidNames)?;
        Ok(Names { bytes })
    }

    fn root(&self) -> eve::Names<'_> {
        // SAFETY: `new` verified these bytes, and nothing changes them after.
        unsafe { eve::root_as_names_unchecked(&self.bytes) }
    }

    /// The EVE build the names were exported from; it has to match the SDE.
    pub fn build_number(&self) -> i32 {
        self.root().build_number()
    }

    /// Names are lowercased and sorted by UTF-8 bytes, which is what `str`
    /// compares by. Several types can share a name; this returns all of them,
    /// lowest id first. `names.dat` does not know which ones are published.
    pub fn type_name_to_ids(&self, name: &str) -> impl Iterator<Item = i32> {
        let root = self.root();
        let names = root.names();
        let type_ids = root.type_ids();
        let wanted = name.to_lowercase();

        let mut low = 0;
        let mut high = names.len();
        while low < high {
            let middle = low + (high - low) / 2;
            if names.get(middle) < wanted.as_str() {
                low = middle + 1;
            } else {
                high = middle;
            }
        }

        (low..names.len())
            .take_while(move |index| names.get(*index) == wanted)
            .map(move |index| type_ids.get(index))
    }
}
