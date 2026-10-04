use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::OnceLock;

use super::eve;
use crate::Error;
use crate::fold::{fold_case, fold_char, sort_by_text};

/// Compare two names case-insensitively without allocating.
fn compare_folded(left: &str, right: &str) -> Ordering {
    if left.is_ascii() && right.is_ascii() {
        let left = left.bytes().map(|byte| byte.to_ascii_lowercase());
        return left.cmp(right.bytes().map(|byte| byte.to_ascii_lowercase()));
    }
    left.chars()
        .map(fold_char)
        .cmp(right.chars().map(fold_char))
}

/// Names sorted case-insensitively, published first, then lowest id.
fn sorted_names(entries: Vec<(&str, bool, i32)>) -> Vec<(&str, bool, i32)> {
    sort_by_text(entries, |(name, published, _)| {
        let mut key = fold_case(name);
        key.push(if *published { '\0' } else { '\u{1}' });
        key
    })
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
    group_types: OnceLock<HashMap<i32, Vec<i32>>>,
    base_mutaplasmids: OnceLock<HashMap<i32, Vec<i32>>>,
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
            group_types: OnceLock::new(),
            base_mutaplasmids: OnceLock::new(),
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

    /// The ids of the types in a group, lowest first.
    pub fn group_type_ids(&self, group_id: i32) -> &[i32] {
        let group_types = self.group_types.get_or_init(|| {
            let mut group_types: HashMap<i32, Vec<i32>> = HashMap::new();
            for r#type in self.types() {
                group_types
                    .entry(r#type.group_id())
                    .or_default()
                    .push(r#type.id());
            }
            group_types
        });
        group_types.get(&group_id).map_or(&[], Vec::as_slice)
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

    /// The type ids of the mutaplasmids that apply to `base`, lowest first.
    pub fn mutaplasmids_of(&self, base: i32) -> &[i32] {
        let base_mutaplasmids = self.base_mutaplasmids.get_or_init(|| {
            let mut base_mutaplasmids: HashMap<i32, Vec<i32>> = HashMap::new();
            for mutaplasmid in self.mutaplasmids() {
                let bases = mutaplasmid
                    .mappings()
                    .into_iter()
                    .flatten()
                    .flat_map(|mapping| mapping.applicable_type_ids().into_iter().flatten());
                for base in bases {
                    let mutaplasmids = base_mutaplasmids.entry(base).or_default();
                    if mutaplasmids.last() != Some(&mutaplasmid.id()) {
                        mutaplasmids.push(mutaplasmid.id());
                    }
                }
            }
            base_mutaplasmids
        });
        base_mutaplasmids.get(&base).map_or(&[], Vec::as_slice)
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
