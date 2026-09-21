use std::collections::BTreeMap;

use flatbuffers::Vector;

use super::{Names, Sde, eve};
use crate::Error;
use crate::info::{Info, InfoExport, InfoName};

/// [`Info`] answered from `sde.dat`.
pub struct InfoSde<'a> {
    sde: &'a Sde<'a>,
}

/// [`InfoName`] answered from `sde.dat`, and from `names.dat` when given, to
/// also match names in the other languages EVE supports.
pub struct InfoNameSde<'a> {
    sde: &'a Sde<'a>,
    names: Option<&'a Names<'a>>,
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

    fn get_dbuff_collection(&self, buff_id: i32) -> Option<eve::DbuffCollection<'_>> {
        self.sde.get_dbuff_collection(buff_id)
    }

    fn attribute_name_to_id(&self, name: &str) -> Option<i32> {
        self.sde.attribute_name_to_id(name)
    }
}

/* Only a handful of mutaplasmids exist, and only a mutated item looks one up,
 * so a scan beats indexing them. */
impl InfoExport for InfoSde<'_> {
    fn find_mutaplasmid(&self, base: i32, result: i32, rolls: &BTreeMap<i32, f64>) -> Option<i32> {
        let candidates = self
            .sde
            .mutaplasmids()
            .filter(|mutaplasmid| makes(*mutaplasmid, base, result));

        let (fitting, rest): (Vec<_>, Vec<_>) =
            candidates.partition(|mutaplasmid| self.holds_rolls(*mutaplasmid, base, result, rolls));

        /* The ranges of the mutaplasmids of one item nest, so the narrowest
         * one that holds the rolls is the closest guess at the one used. */
        let narrowest = fitting
            .into_iter()
            .min_by(|left, right| roll_width(*left).total_cmp(&roll_width(*right)));
        let widest = || {
            rest.into_iter()
                .max_by(|left, right| roll_width(*left).total_cmp(&roll_width(*right)))
        };

        narrowest
            .or_else(widest)
            .map(|mutaplasmid| mutaplasmid.id())
    }
}

impl InfoSde<'_> {
    /// Whether every attribute the mutaplasmid rolls has a value in `rolls`
    /// that it could have rolled. Its range is a factor of the unmutated
    /// value.
    fn holds_rolls(
        &self,
        mutaplasmid: eve::Mutaplasmid,
        base: i32,
        result: i32,
        rolls: &BTreeMap<i32, f64>,
    ) -> bool {
        mutaplasmid
            .attributes()
            .into_iter()
            .flatten()
            .all(|attribute| {
                let Some(rolled) = rolls.get(&attribute.attribute_id()) else {
                    return false;
                };
                let Some(value) = self
                    .base_value(base, attribute.attribute_id())
                    .or_else(|| self.base_value(result, attribute.attribute_id()))
                    .filter(|value| *value != 0.0)
                else {
                    return false;
                };

                /* Rounding trails along the way from the roll to the SDE and
                 * back, so the edges of the range are not exact. */
                let factor = rolled / value;
                factor >= f64::from(attribute.min()) - FACTOR_SLACK
                    && factor <= f64::from(attribute.max()) + FACTOR_SLACK
            })
    }

    /// The value a type gives an attribute before anything modifies it.
    fn base_value(&self, type_id: i32, attribute_id: i32) -> Option<f64> {
        self.sde
            .get_type(type_id)?
            .dogma_attributes()?
            .iter()
            .find(|attribute| attribute.attribute_id() == attribute_id)
            .map(|attribute| f64::from(attribute.value()))
    }
}

/// How far outside its range a roll may land and still count.
const FACTOR_SLACK: f64 = 1e-6;

/// Whether the mutaplasmid turns `base` into `result`.
fn makes(mutaplasmid: eve::Mutaplasmid, base: i32, result: i32) -> bool {
    mutaplasmid.mappings().into_iter().flatten().any(|mapping| {
        mapping.resulting_type_id() == result
            && mapping
                .applicable_type_ids()
                .is_some_and(|type_ids| type_ids.iter().any(|type_id| type_id == base))
    })
}

/// How far the mutaplasmid rolls, over all the attributes it touches.
fn roll_width(mutaplasmid: eve::Mutaplasmid) -> f64 {
    mutaplasmid
        .attributes()
        .into_iter()
        .flatten()
        .map(|attribute| f64::from(attribute.max() - attribute.min()))
        .sum()
}

impl InfoName for InfoNameSde<'_> {
    fn get_dogma_effects(&self, type_id: i32) -> Option<Vector<'_, eve::TypeDogmaEffect>> {
        self.sde.get_type(type_id)?.dogma_effects()
    }

    fn get_dogma_attributes(&self, type_id: i32) -> Option<Vector<'_, eve::TypeDogmaAttribute>> {
        self.sde.get_type(type_id)?.dogma_attributes()
    }

    fn get_type(&self, type_id: i32) -> Option<eve::Type<'_>> {
        self.sde.get_type(type_id)
    }

    fn attribute_name_to_id(&self, name: &str) -> Option<i32> {
        self.sde.attribute_name_to_id(name)
    }

    fn get_mutaplasmid(&self, type_id: i32) -> Option<eve::Mutaplasmid<'_>> {
        self.sde.get_mutaplasmid(type_id)
    }

    /// An exact English name wins over a translation, even when unpublished;
    /// among translations, a published type wins.
    fn type_name_to_id(&self, name: &str) -> Option<i32> {
        self.sde.type_name_to_id(name).or_else(|| {
            let mut type_ids = self.names?.type_name_to_ids(name).peekable();
            let first = *type_ids.peek()?;
            Some(
                type_ids
                    .find(|type_id| {
                        self.sde
                            .get_type(*type_id)
                            .is_some_and(|r#type| r#type.published())
                    })
                    .unwrap_or(first),
            )
        })
    }
}

impl<'a> InfoSde<'a> {
    /// Answer [`Info`] from this SDE.
    pub fn new(sde: &'a Sde<'a>) -> InfoSde<'a> {
        InfoSde { sde }
    }
}

impl<'a> InfoNameSde<'a> {
    /// The type ids in `names.dat` only mean anything against the SDE it was
    /// built from, so refuse a pair that does not match.
    pub fn new(sde: &'a Sde<'a>, names: Option<&'a Names<'a>>) -> Result<InfoNameSde<'a>, Error> {
        if let Some(names) = names
            && sde.build_number() != names.build_number()
        {
            return Err(Error::BuildMismatch {
                sde: sde.build_number(),
                names: names.build_number(),
            });
        }

        Ok(InfoNameSde { sde, names })
    }
}
