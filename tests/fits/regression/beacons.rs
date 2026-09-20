//! A beacon is a source like any other; the engine only puts it in the room.
//! Some hand over dogma effects, which do the work themselves. Others hand
//! over a buff instead: an id naming what it changes, and a strength.

use crate::harness::{all, beacon};

const RIFTER: &str = r#"
[Rifter, Beacons]
Damage Control II
Small Armor Repairer II
Overdrive Injector System II

Medium Shield Extender II
1MN Afterburner II
Small Capacitor Booster II, Navy Cap Booster 400

200mm AutoCannon II, Republic Fleet EMP S
Rocket Launcher II, Scourge Rage Rocket
"#;

const DRONE_BOAT: &str = r#"
[Vexor, Drones]
Damage Control II
Drone Damage Amplifier II

10MN Afterburner II

Hobgoblin II x5
"#;

regression! {
    black_hole_class_1 = RIFTER, skills: all(5), edit: |fit| { fit.incoming.extend(beacon("Class 1 Black Hole Effects")); };
    black_hole_class_6 = RIFTER, skills: all(5), edit: |fit| { fit.incoming.extend(beacon("Class 6 Black Hole Effects")); };
    cataclysmic_variable_class_6 = RIFTER, skills: all(5), edit: |fit| { fit.incoming.extend(beacon("Class 6 Cataclysmic Variable Effects")); };
    magnetar_class_6 = RIFTER, skills: all(5), edit: |fit| { fit.incoming.extend(beacon("Class 6 Magnetar Effects")); };
    pulsar_class_6 = RIFTER, skills: all(5), edit: |fit| { fit.incoming.extend(beacon("Class 6 Pulsar Effects")); };
    red_giant_class_6 = RIFTER, skills: all(5), edit: |fit| { fit.incoming.extend(beacon("Class 6 Red Giant Effects")); };
    wolf_rayet_class_6 = RIFTER, skills: all(5), edit: |fit| { fit.incoming.extend(beacon("Class 6 Wolf Rayet Effects")); };
    /* Abyssal weather: an EM resistance penalty and a capacitor bonus, both as buffs. */
    abyssal_electrical_2 = RIFTER, skills: all(5), edit: |fit| { fit.incoming.extend(beacon("electric_storm_weather_2")); };
    abyssal_electrical_3 = RIFTER, skills: all(5), edit: |fit| { fit.incoming.extend(beacon("electric_storm_weather_3")); };
    /* A cloud bloats the signature radius, and nothing else. */
    abyssal_bioluminescence_cloud = RIFTER, skills: all(5), edit: |fit| { fit.incoming.extend(beacon("Large Bioluminescence Cloud")); };
    /* Triglavian applies effects selectively. */
    triglavian_invasion_strong = DRONE_BOAT, skills: all(5), edit: |fit| {
        fit.incoming.extend(beacon("Triglavian Invasion Strong System Effects"));
    };
    /* Two of the same buff do not add up; the stronger weather wins. */
    abyssal_electrical_2_and_3 = RIFTER, skills: all(5), edit: |fit| {
        fit.incoming.extend(beacon("electric_storm_weather_2"));
        fit.incoming.extend(beacon("electric_storm_weather_3"));
    };
}
