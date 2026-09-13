use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::OnceLock;

use super::eve;

/// Compare two names case-insensitively without allocating.
fn compare_lowercase(left: &str, right: &str) -> Ordering {
    let left = left.chars().flat_map(char::to_lowercase);
    let right = right.chars().flat_map(char::to_lowercase);
    left.cmp(right)
}

pub struct Sde<'a> {
    sde: eve::Sde<'a>,
    attribute_ids: HashMap<&'a str, i32>,
    /// Built on first use: only an EFT import looks a type up by name, and
    /// the borrowed names mean building it allocates one vector and no more.
    type_names: OnceLock<Vec<(&'a str, i32)>>,
}

impl<'a> Sde<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Sde<'a>, String> {
        let sde = eve::root_as_sde(bytes).map_err(|error| format!("{:?}", error))?;

        let mut attribute_ids = HashMap::new();
        if let Some(attributes) = sde.dogma_attributes() {
            for attribute in attributes {
                attribute_ids.insert(attribute.name(), attribute.id());
            }
        }

        Ok(Sde {
            sde,
            attribute_ids,
            type_names: OnceLock::new(),
        })
    }

    pub fn build_number(&self) -> i32 {
        self.sde.build_number()
    }

    pub fn types(&self) -> impl Iterator<Item = eve::Type<'a>> {
        self.sde.types().into_iter().flatten()
    }

    pub fn get_type(&self, type_id: i32) -> Option<eve::Type<'a>> {
        self.sde
            .types()?
            .lookup_by_key(type_id, |entry, key| entry.key_compare_with_value(*key))
    }

    pub fn get_dogma_attribute(&self, attribute_id: i32) -> Option<eve::DogmaAttribute<'a>> {
        self.sde
            .dogma_attributes()?
            .lookup_by_key(attribute_id, |entry, key| {
                entry.key_compare_with_value(*key)
            })
    }

    pub fn get_dogma_effect(&self, effect_id: i32) -> Option<eve::DogmaEffect<'a>> {
        self.sde
            .dogma_effects()?
            .lookup_by_key(effect_id, |entry, key| entry.key_compare_with_value(*key))
    }

    pub fn attribute_name_to_id(&self, name: &str) -> Option<i32> {
        self.attribute_ids.get(name).copied()
    }

    /// Look a type up by its English name. Several types can share one; this
    /// returns the lowest id, the same one `names.dat` would give.
    pub fn type_name_to_id(&self, name: &str) -> Option<i32> {
        let type_names = self.type_names.get_or_init(|| {
            let mut entries: Vec<(&'a str, i32)> = self
                .types()
                .map(|r#type| (r#type.name(), r#type.id()))
                .collect();
            /* Lowercasing during the sort would redo it on every comparison;
             * types arrive in id order and the sort is stable, so a shared
             * name still ends up lowest id first. */
            entries.sort_by_cached_key(|entry| entry.0.to_lowercase());
            entries
        });

        let position =
            type_names.partition_point(|entry| compare_lowercase(entry.0, name) == Ordering::Less);

        let entry = type_names.get(position)?;
        (compare_lowercase(entry.0, name) == Ordering::Equal).then_some(entry.1)
    }
}
