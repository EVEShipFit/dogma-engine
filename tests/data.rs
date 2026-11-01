//! Loading the static data, and why it can fail.

use std::error::Error as _;

use esf_data::{Error, Names, Sde};

#[test]
fn refuses_what_is_not_an_sde() {
    let error = Sde::new(b"").err().unwrap();

    assert!(matches!(error, Error::InvalidSde(_)));
    assert_eq!(error.to_string(), "not a valid SDE file");
    assert!(error.source().is_some());
}

#[test]
fn refuses_what_is_not_a_names_file() {
    let error = Names::new(b"").err().unwrap();

    assert!(matches!(error, Error::InvalidNames(_)));
    assert_eq!(error.to_string(), "not a valid names file");
    assert!(error.source().is_some());
}

#[test]
fn explains_each_error() {
    let mismatch = Error::BuildMismatch { sde: 1, names: 2 };
    let too_old = Error::SdeTooOld {
        found: 3,
        needed: 4,
    };

    assert_eq!(
        mismatch.to_string(),
        "SDE is build 1 but the names are build 2"
    );
    assert_eq!(
        too_old.to_string(),
        "SDE is major version 3 but at least 4 is needed"
    );
    assert!(mismatch.source().is_none());
    assert!(too_old.source().is_none());
}
