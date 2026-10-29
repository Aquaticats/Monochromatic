//! The catch-up rule without a server:
//!  what is asked again,
//!  and when asking stops.

use super::{MAX_CATCH_UPS, Owed};

#[test]
fn nothing_owed_asks_nothing() {
    let mut owed = Owed::default();
    assert_eq!(
        owed.catch_up(),
        None,
        "a server that owes nothing was asked again"
    );
    assert_eq!(owed, Owed::default(), "asking nothing spent a catch-up ask");
}

#[test]
fn owed_answers_are_asked_again_by_kind() {
    let mut hints = Owed {
        hints: true,
        ..Owed::default()
    };
    let asked = hints.catch_up().expect("owed hints are asked again");
    assert!(
        asked.hints && !asked.pull,
        "only the owed kind is asked again"
    );
    let mut pull = Owed {
        pull: true,
        ..Owed::default()
    };
    let pulled = pull
        .catch_up()
        .expect("owed pull diagnostics are asked again");
    assert!(
        pulled.pull && !pulled.hints,
        "only the owed kind is asked again"
    );
}

#[test]
fn catch_up_asks_stop_at_the_limit_until_the_text_changes() {
    let mut owed = Owed {
        hints: true,
        pull: true,
        ..Owed::default()
    };
    for round in 0..MAX_CATCH_UPS {
        assert!(
            owed.catch_up().is_some(),
            "catch-up ask {round} was refused below the limit"
        );
    }
    assert_eq!(
        owed.catch_up(),
        None,
        "a server was asked again beyond the limit for one displayed text"
    );
    assert!(
        owed.hints && owed.pull,
        "reaching the limit must not forget what is owed"
    );
    // A reload or a newly displayed file replaces the record, which renews the limit.
    let mut renewed = Owed {
        hints: true,
        ..Owed::default()
    };
    assert!(
        renewed.catch_up().is_some(),
        "a new text did not renew the catch-up asks"
    );
}
