//! What esf/1 reads from the SDE.

use esf::Kind;
use esf_data::{InfoEsf, eve};

const CATEGORY_SHIP: i32 = 6;
const CATEGORY_MODULE: i32 = 7;
const CATEGORY_CHARGE: i32 = 8;
const CATEGORY_DRONE: i32 = 18;
const CATEGORY_IMPLANT: i32 = 20;
const CATEGORY_SUBSYSTEM: i32 = 32;
const CATEGORY_STRUCTURE: i32 = 65;
const CATEGORY_STRUCTURE_MODULE: i32 = 66;
const CATEGORY_FIGHTER: i32 = 87;

const GROUP_BOOSTER: i32 = 303;
const GROUP_SHIP_MODIFIERS: i32 = 1306;
const GROUP_CONTAINERS: [i32; 4] = [12, 340, 448, 649];

const EFFECT_LO_POWER: i32 = 11;
const EFFECT_HI_POWER: i32 = 12;
const EFFECT_MED_POWER: i32 = 13;
const EFFECT_ONLINE: i32 = 16;
const EFFECT_RIG_SLOT: i32 = 2663;
const EFFECT_SERVICE_SLOT: i32 = 6306;

const ATTRIBUTE_SQUADRON_SIZE: i32 = 2215;

pub(crate) struct Lookup<'a, I> {
    pub info: &'a I,
}

/// An SDE number as the decimal it was written as.
pub(crate) fn sde_decimal(value: f32) -> f64 {
    value.to_string().parse().unwrap_or(value.into())
}

impl<'a, I: InfoEsf> Lookup<'a, I> {
    pub fn r#type(&self, type_id: i32) -> Option<eve::Type<'a>> {
        self.info.get_type(type_id)
    }

    pub fn classify(&self, type_id: i32) -> Kind {
        let Some(r#type) = self.r#type(type_id) else {
            return Kind::Other;
        };
        if r#type.group_id() == GROUP_SHIP_MODIFIERS {
            return Kind::Mode;
        }
        if GROUP_CONTAINERS.contains(&r#type.group_id()) {
            return Kind::Container;
        }
        match r#type.category_id() {
            CATEGORY_SHIP | CATEGORY_STRUCTURE => Kind::Hull,
            CATEGORY_SUBSYSTEM => Kind::Sub,
            CATEGORY_MODULE | CATEGORY_STRUCTURE_MODULE => self
                .info
                .get_dogma_effects(type_id)
                .into_iter()
                .flatten()
                .find_map(|effect| match effect.effect_id() {
                    EFFECT_HI_POWER => Some(Kind::High),
                    EFFECT_MED_POWER => Some(Kind::Mid),
                    EFFECT_LO_POWER => Some(Kind::Low),
                    EFFECT_RIG_SLOT => Some(Kind::Rig),
                    EFFECT_SERVICE_SLOT => Some(Kind::Svc),
                    _ => None,
                })
                .unwrap_or(Kind::Other),
            CATEGORY_IMPLANT if r#type.group_id() == GROUP_BOOSTER => Kind::Booster,
            CATEGORY_IMPLANT => Kind::Implant,
            CATEGORY_DRONE => Kind::Drone,
            CATEGORY_FIGHTER => Kind::Fighter,
            CATEGORY_CHARGE => Kind::Charge,
            _ => Kind::Other,
        }
    }

    /// Whether a type has an effect in the active or target category.
    pub fn is_active(&self, type_id: i32) -> bool {
        self.info
            .get_dogma_effects(type_id)
            .into_iter()
            .flatten()
            .filter(|effect| effect.effect_id() != EFFECT_ONLINE)
            .filter_map(|effect| self.info.get_dogma_effect(effect.effect_id()))
            .any(|effect| {
                matches!(
                    effect.effect_category(),
                    eve::EffectCategory::Active | eve::EffectCategory::Target
                )
            })
    }

    pub fn modes(&self, hull: i32) -> Vec<i32> {
        self.info.mode_type_ids(hull).collect()
    }

    /// The type a mutaplasmid turns `base` into.
    pub fn mutated(&self, mutaplasmid: i32, base: i32) -> Option<i32> {
        self.info
            .get_mutaplasmid(mutaplasmid)?
            .mappings()?
            .iter()
            .find(|mapping| {
                mapping
                    .applicable_type_ids()
                    .is_some_and(|type_ids| type_ids.iter().any(|type_id| type_id == base))
            })
            .map(|mapping| mapping.resulting_type_id())
    }

    pub fn rollable(&self, mutaplasmid: i32) -> Vec<i32> {
        self.info
            .get_mutaplasmid(mutaplasmid)
            .and_then(|mutaplasmid| mutaplasmid.attributes())
            .into_iter()
            .flatten()
            .map(|attribute| attribute.attribute_id())
            .collect()
    }

    pub fn attribute(&self, type_id: i32, attribute_id: i32) -> Option<f64> {
        self.info
            .get_dogma_attributes(type_id)?
            .iter()
            .find(|attribute| attribute.attribute_id() == attribute_id)
            .map(|attribute| sde_decimal(attribute.value()))
    }

    pub fn base_value(&self, type_id: i32, attribute_id: i32) -> f64 {
        self.attribute(type_id, attribute_id).unwrap_or_else(|| {
            self.info
                .get_dogma_attribute(attribute_id)
                .map_or(0.0, |attribute| sde_decimal(attribute.default_value()))
        })
    }

    pub fn squadron_size(&self, type_id: i32) -> Option<u32> {
        self.attribute(type_id, ATTRIBUTE_SQUADRON_SIZE)
            .map(|size| size as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sde_decimals() {
        assert_eq!(sde_decimal(0.1), 0.1);
        assert_eq!(sde_decimal(524.0), 524.0);
    }
}
