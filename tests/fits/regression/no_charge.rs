//! Only a module that holds a charge reloads. A smart bomb, civilian turret or
//! Miner II gets no reload time, and its damage counts once, not as drone damage.

use crate::harness::all;

const COMBAT: &str = r#"%esf/1
Tristan "No charge"

Small EMP Smartbomb I
Civilian Gatling Autocannon
125mm Gatling AutoCannon II :EMP S

2x Hobgoblin II
"#;

const MINING: &str = r#"%esf/1
Venture "No charge"

Miner II
Modulated Deep Core Miner II :Veldspar Mining Crystal II
"#;

regression! {
    combat_skills_0 = COMBAT, skills: all(0);
    combat_skills_5 = COMBAT, skills: all(5);
    mining_skills_5 = MINING, skills: all(5);
}
