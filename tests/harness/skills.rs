use std::collections::BTreeMap;

use esf_dogma_engine::info::InfoName;
use esf_dogma_engine::rust;

use super::DATA;

const SKILL_CATEGORY_ID: i32 = 16;

pub struct Skills {
    pub levels: BTreeMap<i32, i32>,
}

pub fn all(level: i32) -> Skills {
    Skills {
        levels: DATA
            .types
            .iter()
            .filter(|(_, r#type)| r#type.category_id == SKILL_CATEGORY_ID)
            .map(|(type_id, _)| (*type_id, level))
            .collect(),
    }
}

pub fn none() -> Skills {
    Skills {
        levels: BTreeMap::new(),
    }
}

impl Skills {
    pub fn with(mut self, name: &str, level: i32) -> Skills {
        let type_id = rust::InfoNameMain::new(&DATA).type_name_to_id(name);
        assert!(type_id != 0, "no such skill: {name}");

        self.levels.insert(type_id, level);
        self
    }
}
