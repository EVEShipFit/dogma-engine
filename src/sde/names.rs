use super::names_generated::eve;

/// Reader for `names.dat`, the type names in every language EVE supports.
///
/// Only an EFT import needs this; everything else can be answered from
/// `sde.dat` alone, which is why it is a separate file.
pub struct Names<'a> {
    names: eve::Names<'a>,
}

impl<'a> Names<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Names<'a>, String> {
        let names = eve::root_as_names(bytes).map_err(|error| format!("{:?}", error))?;
        Ok(Names { names })
    }

    pub fn build_number(&self) -> i32 {
        self.names.build_number()
    }

    /// Names are lowercased and sorted by UTF-8 bytes, which is what `str`
    /// compares by. Several types can share a name; this returns the first,
    /// matching how the SDE orders them.
    pub fn type_name_to_id(&self, name: &str) -> Option<i32> {
        let names = self.names.names();
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

        if low >= names.len() || names.get(low) != wanted {
            return None;
        }
        Some(self.names.type_ids().get(low))
    }
}
