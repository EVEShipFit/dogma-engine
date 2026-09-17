//! Wormhole effects. A beacon is a source like any other, so the SDE effects do
//! the work; the engine only puts the beacon in the room.

use crate::harness::{all, system_effect};

const RIFTER: &str = r#"
[Rifter, System effects]
Damage Control II
Small Armor Repairer II
Overdrive Injector System II

Medium Shield Extender II
1MN Afterburner II
Small Capacitor Booster II, Navy Cap Booster 400

200mm AutoCannon II, Republic Fleet EMP S
Rocket Launcher II, Scourge Rage Rocket
"#;

regression! {
    black_hole_class_1 = RIFTER, skills: all(5), edit: |fit| { fit.environment.system_effects.insert(system_effect("Class 1 Black Hole Effects")); };
    black_hole_class_6 = RIFTER, skills: all(5), edit: |fit| { fit.environment.system_effects.insert(system_effect("Class 6 Black Hole Effects")); };
    cataclysmic_variable_class_6 = RIFTER, skills: all(5), edit: |fit| { fit.environment.system_effects.insert(system_effect("Class 6 Cataclysmic Variable Effects")); };
    magnetar_class_6 = RIFTER, skills: all(5), edit: |fit| { fit.environment.system_effects.insert(system_effect("Class 6 Magnetar Effects")); };
    pulsar_class_6 = RIFTER, skills: all(5), edit: |fit| { fit.environment.system_effects.insert(system_effect("Class 6 Pulsar Effects")); };
    red_giant_class_6 = RIFTER, skills: all(5), edit: |fit| { fit.environment.system_effects.insert(system_effect("Class 6 Red Giant Effects")); };
    wolf_rayet_class_6 = RIFTER, skills: all(5), edit: |fit| { fit.environment.system_effects.insert(system_effect("Class 6 Wolf Rayet Effects")); };
}
