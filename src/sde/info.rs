use std::collections::BTreeMap;

use flatbuffers::Vector;

use super::{Names, Sde, eve};
use crate::data_types;
use crate::info::{Info, InfoName};

pub struct InfoSde<'a> {
    pub fit: data_types::EsfFit,
    pub skills: BTreeMap<i32, i32>,
    pub sde: &'a Sde<'a>,
}

pub struct InfoNameSde<'a> {
    pub sde: &'a Sde<'a>,
    pub names: Option<&'a Names<'a>>,
}

impl Info for InfoSde<'_> {
    fn get_dogma_attributes(&self, type_id: i32) -> Option<Vector<'_, eve::TypeDogmaAttribute>> {
        self.sde.get_type(type_id)?.dogma_attributes()
    }

    fn get_dogma_attribute(&self, attribute_id: i32) -> Option<eve::DogmaAttribute<'_>> {
        self.sde.get_dogma_attribute(attribute_id)
    }

    fn get_dogma_effects(&self, type_id: i32) -> Option<Vector<'_, eve::TypeDogmaEffect>> {
        self.sde.get_type(type_id)?.dogma_effects()
    }

    fn get_dogma_effect(&self, effect_id: i32) -> Option<eve::DogmaEffect<'_>> {
        self.sde.get_dogma_effect(effect_id)
    }

    fn get_type(&self, type_id: i32) -> Option<eve::Type<'_>> {
        self.sde.get_type(type_id)
    }

    fn attribute_name_to_id(&self, name: &str) -> i32 {
        self.sde.attribute_name_to_id(name).unwrap_or(0)
    }

    fn skills(&self) -> &BTreeMap<i32, i32> {
        &self.skills
    }

    fn fit(&self) -> &data_types::EsfFit {
        &self.fit
    }
}

impl InfoName for InfoNameSde<'_> {
    fn get_dogma_effects(&self, type_id: i32) -> Option<Vector<'_, eve::TypeDogmaEffect>> {
        self.sde.get_type(type_id)?.dogma_effects()
    }

    fn get_type(&self, type_id: i32) -> Option<eve::Type<'_>> {
        self.sde.get_type(type_id)
    }

    fn type_name_to_id(&self, name: &str) -> i32 {
        self.sde
            .type_name_to_id(name)
            .or_else(|| self.names.and_then(|names| names.type_name_to_id(name)))
            .unwrap_or(0)
    }
}

impl<'a> InfoSde<'a> {
    pub fn new(
        fit: data_types::EsfFit,
        skills: BTreeMap<i32, i32>,
        sde: &'a Sde<'a>,
    ) -> InfoSde<'a> {
        InfoSde { fit, skills, sde }
    }
}

impl<'a> InfoNameSde<'a> {
    /// The type ids in `names.dat` only mean anything against the SDE it was
    /// built from, so refuse a pair that does not match.
    pub fn new(sde: &'a Sde<'a>, names: Option<&'a Names<'a>>) -> Result<InfoNameSde<'a>, String> {
        if let Some(names) = names
            && sde.build_number() != names.build_number()
        {
            return Err(format!(
                "SDE is build {} but the names are build {}",
                sde.build_number(),
                names.build_number()
            ));
        }

        Ok(InfoNameSde { sde, names })
    }
}
