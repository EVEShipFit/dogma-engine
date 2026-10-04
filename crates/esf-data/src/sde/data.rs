use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::OnceLock;

use super::eve;
use crate::Error;
use crate::fold::{fold_case, fold_char};

/// Compare two names case-insensitively without allocating.
fn compare_folded(left: &str, right: &str) -> Ordering {
    left.chars()
        .map(fold_char)
        .cmp(right.chars().map(fold_char))
}

/// Names sorted case-insensitively, published first, then lowest id.
fn sorted_names(mut entries: Vec<(&str, bool, i32)>) -> Vec<(&str, bool, i32)> {
    /* Folding during the sort would redo it on every comparison; entries
     * arrive in id order and the sort is stable. */
    entries.sort_by_cached_key(|entry| (fold_case(entry.0), !entry.1));
    entries
}

fn find_name(names: &[(&str, bool, i32)], name: &str) -> Option<i32> {
    let position = names.partition_point(|entry| compare_folded(entry.0, name) == Ordering::Less);
    let entry = names.get(position)?;
    (compare_folded(entry.0, name) == Ordering::Equal).then_some(entry.2)
}

/// Reader for `sde.dat`, everything needed to calculate a fit.
///
/// It borrows the bytes it reads from instead of copying them.
pub struct Sde<'a> {
    sde: eve::Sde<'a>,
    attribute_ids: HashMap<&'a str, i32>,
    /// Built on first use: only an import looks a name up, and
    /// the borrowed names mean building it allocates one vector and no more.
    type_names: OnceLock<Vec<(&'a str, bool, i32)>>,
    attribute_names: OnceLock<Vec<(&'a str, bool, i32)>>,
}

impl<'a> Sde<'a> {
    /// Check the bytes are a valid SDE, and index the attributes by name.
    pub fn new(bytes: &'a [u8]) -> Result<Sde<'a>, Error> {
        let sde = eve::root_as_sde(bytes).map_err(Error::InvalidSde)?;

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
            attribute_names: OnceLock::new(),
        })
    }

    /// The EVE build the data was exported from.
    pub fn build_number(&self) -> i32 {
        self.sde.build_number()
    }

    /// Every type, lowest id first.
    pub fn types(&self) -> impl Iterator<Item = eve::Type<'a>> {
        self.sde.types().into_iter().flatten()
    }

    /// A type by id.
    pub fn get_type(&self, type_id: i32) -> Option<eve::Type<'a>> {
        self.sde
            .types()?
            .lookup_by_key(type_id, |entry, key| entry.key_compare_with_value(*key))
    }

    /// An attribute by id.
    pub fn get_dogma_attribute(&self, attribute_id: i32) -> Option<eve::DogmaAttribute<'a>> {
        self.sde
            .dogma_attributes()?
            .lookup_by_key(attribute_id, |entry, key| {
                entry.key_compare_with_value(*key)
            })
    }

    /// An effect by id.
    pub fn get_dogma_effect(&self, effect_id: i32) -> Option<eve::DogmaEffect<'a>> {
        self.sde
            .dogma_effects()?
            .lookup_by_key(effect_id, |entry, key| entry.key_compare_with_value(*key))
    }

    /// A buff by id.
    pub fn get_dbuff_collection(&self, buff_id: i32) -> Option<eve::DbuffCollection<'a>> {
        self.sde
            .dbuff_collections()?
            .lookup_by_key(buff_id, |entry, key| entry.key_compare_with_value(*key))
    }

    /// Every mutaplasmid, lowest type id first.
    pub fn mutaplasmids(&self) -> impl Iterator<Item = eve::Mutaplasmid<'a>> {
        self.sde.mutaplasmids().into_iter().flatten()
    }

    /// A mutaplasmid by its type id.
    pub fn get_mutaplasmid(&self, type_id: i32) -> Option<eve::Mutaplasmid<'a>> {
        self.sde
            .mutaplasmids()?
            .lookup_by_key(type_id, |entry, key| entry.key_compare_with_value(*key))
    }

    /// The id of the attribute with exactly this name, like `"cycleTime"`.
    pub fn attribute_name_to_id(&self, name: &str) -> Option<i32> {
        self.attribute_ids.get(name).copied()
    }

    /// Look a type up by its English name. Several types can share one; this
    /// prefers a published type, then the lowest id.
    pub fn type_name_to_id(&self, name: &str) -> Option<i32> {
        let type_names = self.type_names.get_or_init(|| {
            sorted_names(
                self.types()
                    .map(|r#type| (r#type.name(), r#type.published(), r#type.id()))
                    .collect(),
            )
        });
        find_name(type_names, name)
    }

    /// Look an attribute up by its name ignoring case; published first, then lowest id.
    pub fn attribute_name_to_id_ignoring_case(&self, name: &str) -> Option<i32> {
        let attribute_names = self.attribute_names.get_or_init(|| {
            sorted_names(
                self.sde
                    .dogma_attributes()
                    .into_iter()
                    .flatten()
                    .map(|attribute| (attribute.name(), attribute.published(), attribute.id()))
                    .collect(),
            )
        });
        find_name(attribute_names, name)
    }
}
