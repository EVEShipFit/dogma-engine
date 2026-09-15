//! A module whose bonus grows every cycle spools its damage per second, fully
//! unless the fit says otherwise. Its volley is always the first shot.

use esf_dogma_engine::Spool;

use crate::harness::{Skills, all};

const NERGAL: &str = r#"
[Nergal, Spool]
Light Entropic Disintegrator II, Occult S
"#;

const LESHAK: &str = r#"
[Leshak, Spool]
Supratidal Entropic Disintegrator II, Occult L
Heavy Mutadaptive Remote Armor Repairer II
"#;

/* In game, this character shows a volley of 136 and 163.1 damage per second. */
fn in_game() -> Skills {
    all(0)
        .with("Precursor Frigate", 1)
        .with("Gunnery", 4)
        .with("Rapid Firing", 2)
        .with("Assault Frigates", 1)
}

regression! {
    nergal_in_game = NERGAL, skills: in_game();
    nergal_unspooled = NERGAL, skills: in_game(), edit: |fit| fit.items[0].spool = Some(Spool::MultiplierBonus(0.0));
    nergal_ten_cycles = NERGAL, skills: in_game(), edit: |fit| fit.items[0].spool = Some(Spool::MultiplierBonus(0.7));
    leshak_skills_5 = LESHAK, skills: all(5);
}
