use std::collections::BTreeMap;

use esf_dogma_engine::info::InfoName;
use esf_dogma_engine::sde;

use super::{NAMES, SDE};

const SKILL_CATEGORY_ID: i32 = 16;

pub struct Skills {
    pub levels: BTreeMap<i32, u8>,
}

pub fn all(level: u8) -> Skills {
    Skills {
        levels: SDE
            .types()
            .filter(|r#type| r#type.category_id() == SKILL_CATEGORY_ID)
            .map(|r#type| (r#type.id(), level))
            .collect(),
    }
}

pub fn none() -> Skills {
    Skills {
        levels: BTreeMap::new(),
    }
}

impl Skills {
    pub fn with(mut self, name: &str, level: u8) -> Skills {
        let type_id = sde::InfoNameSde::new(&SDE, Some(&NAMES))
            .unwrap()
            .type_name_to_id(name);
        assert!(type_id != 0, "no such skill: {name}");

        self.levels.insert(type_id, level);
        self
    }
}
