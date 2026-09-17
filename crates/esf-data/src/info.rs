//! The lookups the engine and the importers do, as traits, so the data does
//! not have to come from [`Sde`](crate::Sde).

use flatbuffers::Vector;

use crate::sde::eve;

/// What the dogma engine looks up while calculating a fit.
///
/// [`InfoSde`](crate::InfoSde) answers these from `sde.dat`.
pub trait Info {
    /// The attribute values a type starts with.
    fn get_dogma_attributes(&self, type_id: i32) -> Option<Vector<'_, eve::TypeDogmaAttribute>>;
    /// An attribute by id.
    fn get_dogma_attribute(&self, attribute_id: i32) -> Option<eve::DogmaAttribute<'_>>;
    /// The effects a type has.
    fn get_dogma_effects(&self, type_id: i32) -> Option<Vector<'_, eve::TypeDogmaEffect>>;
    /// An effect by id.
    fn get_dogma_effect(&self, effect_id: i32) -> Option<eve::DogmaEffect<'_>>;
    /// A type by id.
    fn get_type(&self, type_id: i32) -> Option<eve::Type<'_>>;
    /// A buff by id.
    fn get_dbuff_collection(&self, buff_id: i32) -> Option<eve::DbuffCollection<'_>>;
    /// The id of the attribute with exactly this name, like `"cycleTime"`.
    fn attribute_name_to_id(&self, name: &str) -> Option<i32>;
}

/// What an importer looks up to turn a fit written with names into type ids.
///
/// [`InfoNameSde`](crate::InfoNameSde) answers these from `sde.dat`, and
/// optionally `names.dat`.
pub trait InfoName {
    /// The effects a type has.
    fn get_dogma_effects(&self, type_id: i32) -> Option<Vector<'_, eve::TypeDogmaEffect>>;
    /// The attribute values a type starts with.
    fn get_dogma_attributes(&self, type_id: i32) -> Option<Vector<'_, eve::TypeDogmaAttribute>>;
    /// A type by id.
    fn get_type(&self, type_id: i32) -> Option<eve::Type<'_>>;
    /// The id of the type with this name.
    fn type_name_to_id(&self, name: &str) -> Option<i32>;
    /// The id of the attribute with exactly this name, like `"cycleTime"`.
    fn attribute_name_to_id(&self, name: &str) -> Option<i32>;
    /// A mutaplasmid by its type id.
    fn get_mutaplasmid(&self, type_id: i32) -> Option<eve::Mutaplasmid<'_>>;
}
