//! The command line, run as a binary against the SDE in node_modules.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::Value;

const RIFTER: &str = "\
[Rifter, Test]
Damage Control II
Gyrostabilizer II

5MN Microwarpdrive II

200mm AutoCannon II, Republic Fleet EMP S

Warrior II x2
";

const HECATE: &str = "\
%esf/1
Hecate \"Test\" /Sharpshooter

200mm AutoCannon II :Republic Fleet EMP M
";

fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn run(args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_esf-cli"))
        .args(args)
        .current_dir(workspace())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin).unwrap();
    child.wait_with_output().unwrap()
}

fn succeed(args: &[&str], stdin: &[u8]) -> Vec<u8> {
    let output = run(args, stdin);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn stdout(args: &[&str], stdin: &str) -> String {
    String::from_utf8(succeed(args, stdin.as_bytes())).unwrap()
}

fn json(args: &[&str], stdin: &str) -> Value {
    serde_json::from_str(&stdout(args, stdin)).unwrap()
}

fn stderr(args: &[&str], stdin: &str) -> (i32, String) {
    let output = run(args, stdin.as_bytes());
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

/// A file in the temp dir, removed again on drop.
struct TempFile(PathBuf);

impl TempFile {
    fn new(name: &str, content: &str) -> Self {
        let path = std::env::temp_dir().join(format!("esf-cli-{}-{name}", std::process::id()));
        std::fs::write(&path, content).unwrap();
        Self(path)
    }

    fn path(&self) -> &str {
        self.0.to_str().unwrap()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// The value of an attribute in a table, by the line of the group it is in.
fn table_value(table: &str, header: &str, attribute: &str) -> f64 {
    let group = table
        .split("== ")
        .find(|group| group.starts_with(header))
        .unwrap_or_else(|| panic!("no group {header:?} in:\n{table}"));
    let line = group
        .lines()
        .find(|line| line.trim_start().starts_with(&format!("{attribute} ")))
        .unwrap_or_else(|| panic!("no {attribute} in:\n{group}"));
    line.rsplit(' ').next().unwrap().parse().unwrap()
}

#[test]
fn prints_a_table_with_sources() {
    let table = stdout(
        &["-l", "5", "-o", "table", "-a", "maxVelocity", "--sources"],
        RIFTER,
    );

    assert!(table.contains("== ship: Rifter\n"), "{table}");
    assert!(
        table.contains("== drone bay: Warrior II x2 (active)\n"),
        "{table}"
    );
    assert!(
        table.contains("      Navigation: post_percent 25.0000\n"),
        "{table}"
    );
    assert!(!table.contains("== character"), "{table}");
}

#[test]
fn prints_the_mode_and_charge_with_their_sources() {
    let table = stdout(&["-l", "5", "-o", "table", "--sources"], HECATE);

    assert!(
        table.contains("== mode: Hecate Sharpshooter Mode\n"),
        "{table}"
    );
    assert!(
        table.contains("== high 0 charge: Republic Fleet EMP M\n"),
        "{table}"
    );
    assert!(
        table.contains("      Republic Fleet EMP M: pre_mul 0.5000\n"),
        "{table}"
    );
    assert!(
        table.contains("      Hecate Sharpshooter Mode: "),
        "{table}"
    );
    assert!(table.contains("== character\n"), "{table}");
}

#[test]
fn prints_the_buffs_of_its_own_bursts() {
    let fit = "[Claymore, Test]\n\nShield Command Burst II, Shield Harmonizing Charge\n";
    let table = stdout(
        &[
            "-l",
            "5",
            "-o",
            "table",
            "-a",
            "shieldEmDamageResonance",
            "--sources",
        ],
        fit,
    );

    assert!(
        table.starts_with("== buff: Shield Resistance Bonus ("),
        "{table}"
    );
    assert!(table.contains("      Shield Resistance Bonus: "), "{table}");
}

#[test]
fn prints_json_when_not_on_a_terminal() {
    let calculation = json(&["-l", "5"], RIFTER);

    assert_eq!(calculation["items"].as_array().unwrap().len(), 5);
    assert!(
        calculation["ship"]["attributes"]["37"]["value"]
            .as_f64()
            .unwrap()
            > 365.0
    );
}

#[test]
fn keeps_only_changed_attributes() {
    let calculation = json(&["-l", "5", "--changed"], RIFTER);

    let attributes = calculation["ship"]["attributes"].as_object().unwrap();
    assert!(!attributes.is_empty());
    for attribute in attributes.values() {
        assert_ne!(attribute["base"], attribute["value"]);
    }
}

#[test]
fn prints_violations_as_a_table() {
    let table = stdout(&["-l", "0", "-o", "table", "--validate"], RIFTER);

    assert!(table.contains("drone_bay used 10.00 of 0.00\n"), "{table}");
    assert!(table.contains("low 1: Gyrostabilizer II "), "{table}");
    assert!(table.contains("Gunnery at 2, trained to 0\n"), "{table}");
}

#[test]
fn prints_a_charge_violation() {
    let table = stdout(&["-l", "5", "-o", "table", "--validate"], HECATE);

    assert!(
        table.contains("high 0 charge: Republic Fleet EMP M  charge size 2, module takes 1\n"),
        "{table}"
    );
}

#[test]
fn prints_violations_as_json() {
    let violations = json(&["-l", "5", "--validate"], RIFTER);

    assert!(!violations.as_array().unwrap().is_empty());
}

#[test]
fn skills_override_each_other_in_order() {
    let gunnery = |args: &[&str]| {
        let mut all = vec!["-o", "table", "--validate"];
        all.extend_from_slice(args);
        stdout(&all, RIFTER).contains("Gunnery at 2")
    };
    let skills = TempFile::new("skills.json", r#"{"3300": 5}"#);
    let file = skills.path();

    assert!(gunnery(&["-l", "0"]));
    assert!(!gunnery(&["-l", "0", "-f", file]));
    assert!(gunnery(&["-l", "0", "-f", file, "--skill", "Gunnery=1"]));
    assert!(!gunnery(&["-l", "0", "--skill", "Gunnery = 2"]));
}

#[test]
fn reads_the_fit_from_a_file() {
    let file = TempFile::new("fit.txt", RIFTER);

    let calculation = json(&["-l", "5", "-e", file.path()], "");

    assert_eq!(calculation["items"].as_array().unwrap().len(), 5);
}

#[test]
fn sets_module_states() {
    let speed = |args: &[&str]| {
        let mut all = vec!["-l", "5", "-o", "table", "-a", "maxVelocity"];
        all.extend_from_slice(args);
        table_value(&stdout(&all, RIFTER), "ship", "maxVelocity")
    };

    let active = speed(&["--state", "VPPPPPPPAPPPPPPPOOPPPPPP"]);
    let online = speed(&["--state", "OPPPPPPPOPPPPPPPOOPPPPPP"]);

    assert!(active > online * 2.0, "{active} vs {online}");
}

#[test]
fn rigs_are_stronger_outside_high_sec() {
    let fit = "[Astrahus, Test]\nStandup M-Set Missile Precision I\n";
    let bonus = |security: &str| {
        let table = stdout(
            &[
                "-l",
                "5",
                "-o",
                "table",
                "--security",
                security,
                "-a",
                "ExploVelo",
            ],
            fit,
        );
        table_value(&table, "rig 0", "structureRigMissileExploVeloBonus")
    };

    assert_eq!(bonus("high-sec"), 37.5);
    for security in ["low-sec", "null-sec", "wormhole"] {
        assert_eq!(bonus(security), 45.0, "{security}");
    }
}

#[test]
fn reactive_armor_follows_the_damage_profile() {
    let fit = "[Rifter, Test]\nReactive Armor Hardener\n";
    let kinetic = |args: &[&str]| {
        let mut all = vec![
            "-l",
            "5",
            "-o",
            "table",
            "-a",
            "armorKineticDamageResonance",
        ];
        all.extend_from_slice(args);
        table_value(&stdout(&all, fit), "ship", "armorKineticDamageResonance")
    };

    let as_shown = kinetic(&["--damage-profile", "0,0,1,0"]);
    let shifted = kinetic(&["--damage-profile", "0,0,1,0", "--reactive-armor"]);
    let away = kinetic(&["--damage-profile", "0,0,1,0", "--reactive-armor", "1,0,0,0"]);

    assert!(shifted < as_shown, "{shifted} vs {as_shown}");
    assert!(away > as_shown, "{away} vs {as_shown}");
}

#[test]
fn applies_a_beacon() {
    let shield = |args: &[&str]| {
        let mut all = vec!["-l", "5", "-o", "table", "-a", "shieldCapacity"];
        all.extend_from_slice(args);
        table_value(&stdout(&all, RIFTER), "ship", "shieldCapacity")
    };

    let base = shield(&[]);
    let pulsar = shield(&["--beacon", "Class 6 Pulsar Effects"]);

    assert_eq!(pulsar, base * 2.0);
}

#[test]
fn writes_the_fit_as_eft() {
    assert_eq!(stdout(&["--eft"], RIFTER), RIFTER);
}

#[test]
fn calculates_an_esf_fit() {
    let calculation = json(&["-l", "5"], HECATE);

    assert!(calculation["mode"]["attributes"].is_object());
}

#[test]
fn checks_and_converts_esf() {
    assert_eq!(stdout(&["--esf", "check"], HECATE), "");
    assert_eq!(stdout(&["--esf", "canonical"], HECATE), HECATE);

    let binary = succeed(&["--esf", "binary"], HECATE.as_bytes());
    let canonical = succeed(&["--esf", "canonical"], &binary);
    assert_eq!(String::from_utf8(canonical).unwrap(), HECATE);
}

#[test]
fn fails_on_a_broken_esf_fit() {
    let fit = "%esf/1\nRifter\nNo Such Module\n";

    for args in [&["--esf", "check"][..], &[]] {
        let (code, message) = stderr(args, fit);
        assert_eq!(code, 1);
        assert_eq!(message, "error: line 3: unknown type \"No Such Module\"\n");
    }
}

#[test]
fn fails_on_an_unknown_skill() {
    let (code, message) = stderr(&["--skill", "Rifter=3"], RIFTER);

    assert_eq!(code, 1);
    assert_eq!(message, "error: no such skill: Rifter\n");
}

#[test]
fn fails_on_an_unknown_beacon() {
    let (code, message) = stderr(&["--beacon", "Rifter"], RIFTER);

    assert_eq!(code, 1);
    assert_eq!(message, "error: no such beacon: Rifter\n");
}

#[test]
fn rejects_bad_arguments() {
    let cases: [(&[&str], &str); 5] = [
        (&["--skill", "Gunnery"], "expected NAME=LEVEL"),
        (&["--skill", "Gunnery=6"], "level should be 0-5, not \"6\""),
        (
            &["--damage-profile", "1,2"],
            "expected EM,EXPLOSIVE,KINETIC,THERMAL",
        ),
        (
            &["--damage-profile", "1,-2,0,0"],
            "expected a number of 0 or more, not \"-2\"",
        ),
        (
            &["--reactive-armor", "a,0,0,0"],
            "expected a number of 0 or more, not \"a\"",
        ),
    ];

    for (args, expected) in cases {
        let (code, message) = stderr(args, "");
        assert_eq!(code, 2, "{args:?}");
        assert!(message.contains(expected), "{args:?}: {message}");
    }
}
