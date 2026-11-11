//! Lookups on the SDE by name.

use std::collections::HashMap;

use esf_data::{Info, InfoName, InfoNameSde, InfoSde, Names, Sde, eve, fold_case};
use serde::Serialize;
use serde_json::{Value, json};

const CATEGORY_SKILL: i32 = 16;
const FITTABLE_CATEGORIES: [i32; 10] = [6, 7, 8, 16, 18, 20, 32, 65, 66, 87];
const ATTRIBUTE_META_LEVEL: i32 = 633;
const EFFECT_SLOTS: [(i32, &str); 6] = [
    (12, "high"),
    (13, "medium"),
    (11, "low"),
    (2663, "rig"),
    (3772, "subsystem"),
    (6306, "service"),
];

const UNIT_MILLISECONDS: i32 = 101;
const UNIT_INVERSE_ABSOLUTE_PERCENT: i32 = 108;
const UNIT_MODIFIER_PERCENT: i32 = 109;
const UNIT_INVERSED_MODIFIER_PERCENT: i32 = 111;
const UNIT_ABSOLUTE_PERCENT: i32 = 127;
const UNIT_GROUP_ID: i32 = 115;
const UNIT_TYPE_ID: i32 = 116;
const UNIT_ATTRIBUTE_ID: i32 = 119;

pub struct Game {
    pub sde: &'static Sde<'static>,
    pub info: InfoSde<'static>,
    pub info_name: InfoNameSde<'static>,
    pub skills: Vec<i32>,
    groups: HashMap<i32, (&'static str, i32)>,
    categories: HashMap<i32, &'static str>,
    meta_groups: HashMap<i32, &'static str>,
    units: HashMap<i32, &'static str>,
    /// Every published type of a variation family, by the family's base type.
    variations: HashMap<i32, Vec<i32>>,
    /// Folded name, id and category of every published type.
    search: Vec<(String, i32, i32)>,
}

#[derive(Serialize)]
pub struct TypeSummary {
    pub name: String,
    pub type_id: i32,
    pub category: String,
    pub group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<&'static str>,
}

impl Game {
    pub fn new(sde_bytes: Vec<u8>, names_bytes: Option<Vec<u8>>) -> Result<Game, String> {
        let sde_bytes: &'static [u8] = Box::leak(sde_bytes.into_boxed_slice());
        let sde: &'static Sde = Box::leak(Box::new(
            Sde::new(sde_bytes).map_err(|error| format!("sde.dat: {error}"))?,
        ));
        let names: Option<&'static Names> = match names_bytes {
            Some(bytes) => {
                let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
                let names = Names::new(bytes).map_err(|error| format!("names.dat: {error}"))?;
                Some(Box::leak(Box::new(names)))
            }
            None => None,
        };
        let info_name = InfoNameSde::new(sde, names).map_err(|error| error.to_string())?;

        let root = eve::root_as_sde(sde_bytes).map_err(|error| error.to_string())?;
        let groups = root
            .groups()
            .into_iter()
            .flatten()
            .map(|group| (group.id(), (group.name(), group.category_id())))
            .collect();
        let categories = root
            .categories()
            .into_iter()
            .flatten()
            .map(|category| (category.id(), category.name()))
            .collect();
        let meta_groups = root
            .meta_groups()
            .into_iter()
            .flatten()
            .map(|meta_group| (meta_group.id(), meta_group.name()))
            .collect();
        let units = root
            .dogma_units()
            .into_iter()
            .flatten()
            .map(|unit| (unit.id(), unit.display_name().unwrap_or_default()))
            .collect();

        let mut variations: HashMap<i32, Vec<i32>> = HashMap::new();
        let mut search = Vec::new();
        let mut skills = Vec::new();
        for r#type in sde.types() {
            if r#type.category_id() == CATEGORY_SKILL {
                skills.push(r#type.id());
            }
            if !r#type.published() {
                continue;
            }
            search.push((fold_case(r#type.name()), r#type.id(), r#type.category_id()));
            let parent = r#type.variation_parent_type_id();
            if parent != 0 {
                let family = variations.entry(parent).or_default();
                family.push(r#type.id());
                if sde
                    .get_type(parent)
                    .is_some_and(|parent| parent.published())
                {
                    family.push(parent);
                }
            }
        }
        for family in variations.values_mut() {
            family.sort_unstable();
            family.dedup();
        }

        Ok(Game {
            sde,
            info: InfoSde::new(sde),
            info_name,
            skills,
            groups,
            categories,
            meta_groups,
            units,
            variations,
            search,
        })
    }

    pub fn type_name(&self, type_id: i32) -> String {
        self.sde.get_type(type_id).map_or_else(
            || format!("type {type_id}"),
            |r#type| r#type.name().to_string(),
        )
    }

    pub fn attribute_name(&self, attribute_id: i32) -> String {
        self.sde.get_dogma_attribute(attribute_id).map_or_else(
            || format!("attribute {attribute_id}"),
            |attribute| attribute.name().to_string(),
        )
    }

    pub fn effect_name(&self, effect_id: i32) -> String {
        self.sde.get_dogma_effect(effect_id).map_or_else(
            || format!("effect {effect_id}"),
            |effect| effect.name().to_string(),
        )
    }

    pub fn buff_name(&self, buff_id: i32) -> String {
        self.sde
            .get_dbuff_collection(buff_id)
            .and_then(|buff| buff.display_name())
            .filter(|name| !name.is_empty())
            .map_or_else(|| format!("buff {buff_id}"), str::to_string)
    }

    pub fn group_name(&self, group_id: i32) -> String {
        self.groups
            .get(&group_id)
            .map_or_else(|| format!("group {group_id}"), |(name, _)| name.to_string())
    }

    /// A type by name, in any language EVE supports.
    pub fn find_type_id(&self, name: &str) -> Option<i32> {
        self.info_name.type_name_to_id(name.trim())
    }

    /// A type by name, or an error that suggests close names.
    pub fn require_type_id(&self, name: &str) -> Result<i32, String> {
        self.find_type_id(name)
            .ok_or_else(|| format!("no type is named \"{name}\"{}", self.did_you_mean(name)))
    }

    /// "; did you mean: ..." with the closest names, or nothing.
    pub fn did_you_mean(&self, name: &str) -> String {
        let close: Vec<String> = self
            .search(name, None, 5)
            .into_iter()
            .map(|summary| summary.name)
            .collect();
        match close.is_empty() {
            true => String::new(),
            false => format!("; did you mean: {}", close.join(", ")),
        }
    }

    pub fn summarize_type(&self, type_id: i32) -> TypeSummary {
        let r#type = self.sde.get_type(type_id);
        let category_id = r#type
            .map(|r#type| r#type.category_id())
            .unwrap_or_default();
        let group_id = r#type.map(|r#type| r#type.group_id()).unwrap_or_default();
        TypeSummary {
            name: self.type_name(type_id),
            type_id,
            category: self.categories.get(&category_id).map_or_else(
                || format!("category {category_id}"),
                |name| name.to_string(),
            ),
            group: self.group_name(group_id),
            meta_group: r#type
                .and_then(|r#type| self.meta_groups.get(&r#type.meta_group_id()))
                .map(|name| name.to_string()),
            slot: self.slot_of(type_id),
        }
    }

    /// The rack a module goes in.
    pub fn slot_of(&self, type_id: i32) -> Option<&'static str> {
        let effects = self.info.get_dogma_effects(type_id)?;
        EFFECT_SLOTS.iter().find_map(|(effect_id, slot)| {
            effects
                .iter()
                .any(|effect| effect.effect_id() == *effect_id)
                .then_some(*slot)
        })
    }

    pub fn has_effect(&self, type_id: i32, effect_id: i32) -> bool {
        self.info
            .get_dogma_effects(type_id)
            .is_some_and(|effects| effects.iter().any(|effect| effect.effect_id() == effect_id))
    }

    /// Published types by name: every word of `query`, else the closest names.
    pub fn search(&self, query: &str, category: Option<&str>, limit: usize) -> Vec<TypeSummary> {
        let query = fold_case(query.trim());
        let words: Vec<&str> = query.split_whitespace().collect();
        let category = category.map(fold_case);
        let in_category = |category_id: i32| match &category {
            None => FITTABLE_CATEGORIES.contains(&category_id),
            Some(category) => self
                .categories
                .get(&category_id)
                .is_some_and(|name| fold_case(name) == *category),
        };
        let candidates = self
            .search
            .iter()
            .filter(|(_, _, category_id)| in_category(*category_id));

        let mut matches: Vec<(usize, usize, i32)> = candidates
            .clone()
            .filter(|(name, _, _)| words.iter().all(|word| name.contains(word)))
            .map(|(name, type_id, _)| {
                let rank = if *name == query {
                    0
                } else if name.starts_with(&query) {
                    1
                } else if name.contains(&query) {
                    2
                } else {
                    3
                };
                (rank, name.len(), *type_id)
            })
            .collect();
        if matches.is_empty() {
            let most = query.chars().count() / 3 + 1;
            matches = candidates
                .filter_map(|(name, type_id, _)| {
                    let distance = edit_distance(name, &query);
                    (distance <= most).then_some((distance, name.len(), *type_id))
                })
                .collect();
        }
        matches.sort_unstable();
        matches
            .into_iter()
            .take(limit)
            .map(|(_, _, type_id)| self.summarize_type(type_id))
            .collect()
    }

    /// Every published type in the variation family of `type_id`, itself included.
    pub fn variations(&self, type_id: i32) -> Vec<i32> {
        let parent = self
            .sde
            .get_type(type_id)
            .map(|r#type| r#type.variation_parent_type_id())
            .filter(|parent| *parent != 0)
            .unwrap_or(type_id);
        match self.variations.get(&parent) {
            Some(family) => family.clone(),
            None => vec![type_id],
        }
    }

    pub fn meta_level(&self, type_id: i32) -> Option<f64> {
        self.base_value(type_id, ATTRIBUTE_META_LEVEL)
    }

    /// The value an attribute of a type starts with, before anything modifies it.
    pub fn base_value(&self, type_id: i32, attribute_id: i32) -> Option<f64> {
        self.info
            .get_dogma_attributes(type_id)?
            .iter()
            .find(|attribute| attribute.attribute_id() == attribute_id)
            .map(|attribute| f64::from(attribute.value()))
    }

    /// Every attribute of a type, as it starts.
    pub fn info_attributes(&self, type_id: i32) -> Vec<(i32, f64)> {
        self.info
            .get_dogma_attributes(type_id)
            .into_iter()
            .flatten()
            .map(|attribute| (attribute.attribute_id(), f64::from(attribute.value())))
            .collect()
    }

    /// A value as EVE shows it, with its unit.
    pub fn display(&self, attribute_id: i32, value: f64) -> (f64, &'static str) {
        let unit_id = self
            .sde
            .get_dogma_attribute(attribute_id)
            .map(|attribute| attribute.unit_id())
            .unwrap_or_default();
        let unit = self.units.get(&unit_id).copied().unwrap_or_default();
        let shown = match unit_id {
            UNIT_MILLISECONDS => value / 1000.0,
            UNIT_INVERSE_ABSOLUTE_PERCENT | UNIT_INVERSED_MODIFIER_PERCENT => (1.0 - value) * 100.0,
            UNIT_MODIFIER_PERCENT => (value - 1.0) * 100.0,
            UNIT_ABSOLUTE_PERCENT if attribute_id > 0 => value * 100.0,
            _ => value,
        };
        (round(shown), unit)
    }

    /// A value as EVE shows it, as one JSON value: a name for an id, else the number and unit.
    pub fn display_value(&self, attribute_id: i32, value: f64) -> Value {
        let unit_id = self
            .sde
            .get_dogma_attribute(attribute_id)
            .map(|attribute| attribute.unit_id())
            .unwrap_or_default();
        match unit_id {
            UNIT_TYPE_ID if value > 0.0 => json!(self.type_name(value as i32)),
            UNIT_GROUP_ID if value > 0.0 => json!(self.group_name(value as i32)),
            UNIT_ATTRIBUTE_ID if value > 0.0 => json!(self.attribute_name(value as i32)),
            _ => match self.display(attribute_id, value) {
                (value, "") => json!(value),
                (value, unit) => json!(format!("{value} {unit}")),
            },
        }
    }

    pub fn attribute_id(&self, name: &str) -> Option<i32> {
        self.sde.attribute_name_to_id(name)
    }

    pub fn attribute_published(&self, attribute_id: i32) -> bool {
        attribute_id < 0
            || self
                .sde
                .get_dogma_attribute(attribute_id)
                .is_some_and(|attribute| attribute.published())
    }

    pub fn attribute_display_name(&self, attribute_id: i32) -> Option<&'static str> {
        self.sde
            .get_dogma_attribute(attribute_id)
            .and_then(|attribute| attribute.display_name())
            .filter(|name| !name.is_empty())
    }
}

/// Round to what is worth reading: two decimals, or four below one.
pub fn round(value: f64) -> f64 {
    let scale = match value.abs() < 1.0 {
        true => 10_000.0,
        false => 100.0,
    };
    (value * scale).round() / scale
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right: Vec<char> = right.chars().collect();
    let mut row: Vec<usize> = (0..=right.len()).collect();
    for (i, left) in left.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, right) in right.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (above + 1)
                .min(row[j] + 1)
                .min(diagonal + usize::from(left != *right));
            diagonal = above;
        }
    }
    row[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_distances() {
        assert_eq!(edit_distance("warp scrambler ii", "warp scrambler ii"), 0);
        assert_eq!(edit_distance("warp scrambler ii", "warp scramblr ii"), 1);
        assert_eq!(edit_distance("rifter", "rifetr"), 2);
        assert_eq!(edit_distance("", "abc"), 3);
    }

    #[test]
    fn rounds() {
        assert_eq!(round(1234.5678), 1234.57);
        assert_eq!(round(0.123456), 0.1235);
    }
}
