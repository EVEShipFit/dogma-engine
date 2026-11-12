use std::collections::HashMap;
use std::sync::OnceLock;

use super::eve;
use crate::Error;
use crate::fold::fold_case;

/// Names folded to compare case-insensitively, sorted, published first, then lowest id.
struct NameIndex {
    folded: String,
    /// Where each name ends in `folded`, and its id.
    entries: Vec<(u32, i32)>,
}

impl NameIndex {
    fn new<'a>(names: impl Iterator<Item = (&'a str, bool, i32)>) -> NameIndex {
        let mut sorted: Vec<(String, bool, i32)> = names
            .map(|(name, published, id)| (fold_case(name), !published, id))
            .collect();
        sorted.sort_unstable();

        let mut folded = String::with_capacity(sorted.iter().map(|(name, _, _)| name.len()).sum());
        let entries = sorted
            .into_iter()
            .map(|(name, _, id)| {
                folded.push_str(&name);
                (folded.len() as u32, id)
            })
            .collect();
        NameIndex { folded, entries }
    }

    fn name(&self, index: usize) -> &str {
        let start = index
            .checked_sub(1)
            .map_or(0, |previous| self.entries[previous].0);
        &self.folded[start as usize..self.entries[index].0 as usize]
    }

    fn find(&self, name: &str) -> Option<i32> {
        let wanted = fold_case(name);
        let (mut low, mut high) = (0, self.entries.len());
        while low < high {
            let middle = low + (high - low) / 2;
            if self.name(middle) < wanted.as_str() {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        (low < self.entries.len() && self.name(low) == wanted).then(|| self.entries[low].1)
    }
}

/// The position of each entry in a vector, by id.
struct IdIndex {
    first: i32,
    /// `u32::MAX` where no entry has that id.
    positions: Vec<u32>,
}

impl IdIndex {
    fn new(ids: impl Iterator<Item = i32> + Clone) -> IdIndex {
        let first = ids.clone().min().unwrap_or(0);
        let mut positions = Vec::new();
        for (position, id) in ids.enumerate() {
            let offset = id.abs_diff(first) as usize;
            if positions.len() <= offset {
                positions.resize(offset + 1, u32::MAX);
            }
            positions[offset] = position as u32;
        }
        IdIndex { first, positions }
    }

    fn get(&self, id: i32) -> Option<usize> {
        let offset = usize::try_from(id.checked_sub(self.first)?).ok()?;
        let position = *self.positions.get(offset)?;
        (position != u32::MAX).then_some(position as usize)
    }
}

/// The lowest major version of `sde.dat` this crate reads.
pub const MIN_SDE_VERSION: i32 = 12;

/// Reader for `sde.dat`, everything needed to calculate a fit.
///
/// It reads straight from the bytes it holds, without copying them out.
pub struct Sde {
    bytes: Vec<u8>,
    attribute_ids: HashMap<Box<str>, i32>,
    attribute_positions: IdIndex,
    effect_positions: IdIndex,
    /// Built on first use: only an import looks a name up.
    type_names: OnceLock<NameIndex>,
    attribute_names: OnceLock<NameIndex>,
    base_mutaplasmids: OnceLock<HashMap<i32, Vec<i32>>>,
}

impl Sde {
    /// Check the bytes are a valid SDE, and index the attributes and effects.
    pub fn new(bytes: Vec<u8>) -> Result<Sde, Error> {
        let sde = eve::root_as_sde(&bytes).map_err(Error::InvalidSde)?;
        if sde.major_version() < MIN_SDE_VERSION {
            return Err(Error::SdeTooOld {
                found: sde.major_version(),
                needed: MIN_SDE_VERSION,
            });
        }

        let mut attribute_ids = HashMap::new();
        if let Some(attributes) = sde.dogma_attributes() {
            for attribute in attributes {
                attribute_ids.insert(attribute.name().into(), attribute.id());
            }
        }

        let attribute_positions = IdIndex::new(
            sde.dogma_attributes()
                .into_iter()
                .flatten()
                .map(|attribute| attribute.id()),
        );
        let effect_positions = IdIndex::new(
            sde.dogma_effects()
                .into_iter()
                .flatten()
                .map(|effect| effect.id()),
        );

        Ok(Sde {
            bytes,
            attribute_ids,
            attribute_positions,
            effect_positions,
            type_names: OnceLock::new(),
            attribute_names: OnceLock::new(),
            base_mutaplasmids: OnceLock::new(),
        })
    }

    /// The root of the flatbuffer.
    pub fn root(&self) -> eve::Sde<'_> {
        // SAFETY: `new` verified these bytes, and nothing changes them after.
        unsafe { eve::root_as_sde_unchecked(&self.bytes) }
    }

    /// The EVE build the data was exported from.
    pub fn build_number(&self) -> i32 {
        self.root().build_number()
    }

    /// Every type, lowest id first.
    pub fn types(&self) -> impl Iterator<Item = eve::Type<'_>> {
        self.root().types().into_iter().flatten()
    }

    /// A type by id.
    pub fn get_type(&self, type_id: i32) -> Option<eve::Type<'_>> {
        self.root()
            .types()?
            .lookup_by_key(type_id, |entry, key| entry.key_compare_with_value(*key))
    }

    /// An attribute by id.
    pub fn get_dogma_attribute(&self, attribute_id: i32) -> Option<eve::DogmaAttribute<'_>> {
        let position = self.attribute_positions.get(attribute_id)?;
        Some(self.root().dogma_attributes()?.get(position))
    }

    /// An effect by id.
    pub fn get_dogma_effect(&self, effect_id: i32) -> Option<eve::DogmaEffect<'_>> {
        let position = self.effect_positions.get(effect_id)?;
        Some(self.root().dogma_effects()?.get(position))
    }

    /// A buff by id.
    pub fn get_dbuff_collection(&self, buff_id: i32) -> Option<eve::DbuffCollection<'_>> {
        self.root()
            .dbuff_collections()?
            .lookup_by_key(buff_id, |entry, key| entry.key_compare_with_value(*key))
    }

    /// Every mutaplasmid, lowest type id first.
    pub fn mutaplasmids(&self) -> impl Iterator<Item = eve::Mutaplasmid<'_>> {
        self.root().mutaplasmids().into_iter().flatten()
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
    pub fn get_mutaplasmid(&self, type_id: i32) -> Option<eve::Mutaplasmid<'_>> {
        self.root()
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
        self.type_names
            .get_or_init(|| {
                NameIndex::new(
                    self.types()
                        .map(|r#type| (r#type.name(), r#type.published(), r#type.id())),
                )
            })
            .find(name)
    }

    /// Look an attribute up by its name ignoring case; published first, then lowest id.
    pub fn attribute_name_to_id_ignoring_case(&self, name: &str) -> Option<i32> {
        self.attribute_names
            .get_or_init(|| {
                NameIndex::new(
                    self.root()
                        .dogma_attributes()
                        .into_iter()
                        .flatten()
                        .map(|attribute| (attribute.name(), attribute.published(), attribute.id())),
                )
            })
            .find(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sde_bytes(major_version: i32) -> Vec<u8> {
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let sde = eve::Sde::create(
            &mut builder,
            &eve::SdeArgs {
                major_version,
                ..Default::default()
            },
        );
        eve::finish_sde_buffer(&mut builder, sde);
        builder.finished_data().to_vec()
    }

    #[test]
    fn rejects_an_sde_too_old() {
        let bytes = sde_bytes(MIN_SDE_VERSION - 1);
        assert_eq!(
            Sde::new(bytes).err(),
            Some(Error::SdeTooOld {
                found: MIN_SDE_VERSION - 1,
                needed: MIN_SDE_VERSION,
            })
        );
    }

    #[test]
    fn accepts_an_sde_new_enough() {
        assert!(Sde::new(sde_bytes(MIN_SDE_VERSION)).is_ok());
        assert!(Sde::new(sde_bytes(MIN_SDE_VERSION + 1)).is_ok());
    }
}
