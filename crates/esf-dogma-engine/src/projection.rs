//! Projection of buffs and effects.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::fit::id_map;

/// The external buffs and effects that can be applied to another ship.
///
/// A calculation reports what the fit hands out; put that in
/// [`Fit::incoming`](crate::Fit::incoming) of another fit to have it applied.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Projection {
    /// Buffs, like command burst hands.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buffs: Vec<ProjectedBuff>,
    /// Effects, like webifiers, remote reps, etc.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<ProjectedEffect>,
}

/// A projected buff (like command bursts).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ProjectedBuff {
    /// Which buff, as `dbuffCollections` in the SDE numbers them.
    pub id: i32,
    /// How strong it is, in whatever the buff's operation reads.
    pub value: f64,
}

/// A projected dogma effect.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProjectedEffect {
    /// The type the effect belongs to.
    pub type_id: i32,
    /// Which effect, as `dogmaEffects` in the SDE numbers them.
    pub effect_id: i32,
    /// Every attribute and its value the effect reads.
    #[serde(default, deserialize_with = "id_map")]
    pub attributes: BTreeMap<i32, f64>,
}

impl Projection {
    /// Whether nothing is projected.
    pub fn is_empty(&self) -> bool {
        self.buffs.is_empty() && self.effects.is_empty()
    }

    /// Take everything from another projection.
    pub fn extend(&mut self, other: Projection) {
        self.buffs.extend(other.buffs);
        self.effects.extend(other.effects);
    }
}
