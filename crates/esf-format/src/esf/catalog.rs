//! The SDE as an esf/1 catalog, built as it is looked up.

use elsa::FrozenMap;
use esf::{AttributeRecord, Catalog, MutaplasmidRecord, TypeRecord};
use esf_data::InfoEsf;

use super::lookup::{Lookup, sde_decimal};

pub(super) struct SdeCatalog<'a, I> {
    lookup: Lookup<'a, I>,
    types: FrozenMap<u32, Box<TypeRecord>>,
    attributes: FrozenMap<u32, Box<AttributeRecord>>,
    modes: FrozenMap<u32, Vec<TypeRecord>>,
    mutaplasmids: FrozenMap<u32, Vec<MutaplasmidRecord>>,
}

impl<'a, I: InfoEsf> SdeCatalog<'a, I> {
    pub fn new(info: &'a I) -> Self {
        SdeCatalog {
            lookup: Lookup { info },
            types: FrozenMap::new(),
            attributes: FrozenMap::new(),
            modes: FrozenMap::new(),
            mutaplasmids: FrozenMap::new(),
        }
    }

    fn type_record(&self, type_id: i32) -> Option<TypeRecord> {
        let r#type = self.lookup.r#type(type_id)?;
        Some(TypeRecord {
            id: u32::try_from(type_id).ok()?,
            name: r#type.name().to_string(),
            kind: self.lookup.classify(type_id),
            active: self.lookup.is_active(type_id),
            capacity: r#type.capacity().map_or(0.0, sde_decimal),
            volume: r#type.volume().map_or(0.0, sde_decimal),
            squadron_size: self.lookup.squadron_size(type_id),
        })
    }

    fn mutaplasmid_record(&self, base: i32, mutaplasmid: i32) -> Option<MutaplasmidRecord> {
        let rolls = self.lookup.rollable(mutaplasmid).into_iter();
        Some(MutaplasmidRecord {
            mutaplasmid: self.type_record(mutaplasmid)?,
            rolls: rolls
                .filter_map(|attribute_id| {
                    let value = self.lookup.base_value(base, attribute_id);
                    Some((u32::try_from(attribute_id).ok()?, value))
                })
                .collect(),
        })
    }
}

impl<I: InfoEsf> Catalog for SdeCatalog<'_, I> {
    fn type_by_name(&self, folded: &str) -> Option<&TypeRecord> {
        let type_id = self.lookup.info.type_name_to_id(folded)?;
        self.type_by_id(u32::try_from(type_id).ok()?)
    }

    fn type_by_id(&self, type_id: u32) -> Option<&TypeRecord> {
        if let Some(record) = self.types.get(&type_id) {
            return Some(record);
        }
        let record = self.type_record(i32::try_from(type_id).ok()?)?;
        Some(self.types.insert(type_id, Box::new(record)))
    }

    fn attribute_by_name(&self, folded: &str) -> Option<&AttributeRecord> {
        let attribute_id = self
            .lookup
            .info
            .attribute_name_to_id_ignoring_case(folded)?;
        self.attribute_by_id(u32::try_from(attribute_id).ok()?)
    }

    fn attribute_by_id(&self, attribute_id: u32) -> Option<&AttributeRecord> {
        if let Some(record) = self.attributes.get(&attribute_id) {
            return Some(record);
        }
        let attribute = self
            .lookup
            .info
            .get_dogma_attribute(i32::try_from(attribute_id).ok()?)?;
        let record = AttributeRecord {
            id: attribute_id,
            name: attribute.name().to_string(),
        };
        Some(self.attributes.insert(attribute_id, Box::new(record)))
    }

    fn modes(&self, hull_id: u32) -> &[TypeRecord] {
        if let Some(modes) = self.modes.get(&hull_id) {
            return modes;
        }
        let modes = i32::try_from(hull_id).map_or(Vec::new(), |hull| {
            let modes = self.lookup.modes(hull).into_iter();
            modes.filter_map(|mode| self.type_record(mode)).collect()
        });
        self.modes.insert(hull_id, modes)
    }

    fn mutaplasmids(&self, type_id: u32) -> &[MutaplasmidRecord] {
        if let Some(mutaplasmids) = self.mutaplasmids.get(&type_id) {
            return mutaplasmids;
        }
        let mutaplasmids = i32::try_from(type_id).map_or(Vec::new(), |base| {
            let mutaplasmids = self.lookup.info.mutaplasmids_of(base).iter();
            mutaplasmids
                .filter_map(|&mutaplasmid| self.mutaplasmid_record(base, mutaplasmid))
                .collect()
        });
        self.mutaplasmids.insert(type_id, mutaplasmids)
    }
}
