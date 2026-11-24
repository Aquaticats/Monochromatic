//! Controls for the one locked Git invocation's words:
//!  the three options sit where Git reads options,
//!  the person's own words keep their order,
//!  and a person's own lock or checkout is refused.

use super::{ROUTE_OPTIONS, person_lock_word, the_locked_add_words};
use crate::worktree_clone_binding_record::lock_words::{LOCK_WORDS_PREFIX, lock_words};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::ffi::OsString;

/// One add's words: `worktree add -b topic ../topic main`.
fn an_add() -> Vec<OsString> {
    return vec![OsString::from("worktree"), OsString::from("add"), OsString::from("-b"), OsString::from("topic"), OsString::from("../topic"), OsString::from("main")];
}

/// A nonce for the sentence.
fn a_nonce() -> RequestNonce {
    return RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f").expect("the nonce");
}

#[test]
fn the_route_options_sit_directly_after_the_add() {
    let words: Vec<OsString> = the_locked_add_words(an_add().as_slice(), 2, &a_nonce()).expect("the words are built");
    assert_eq!(words[0], OsString::from("worktree"));
    assert_eq!(words[1], OsString::from("add"));
    assert_eq!(words[2], OsString::from(ROUTE_OPTIONS[0]));
    assert_eq!(words[3], OsString::from(ROUTE_OPTIONS[1]));
    assert_eq!(words[4], OsString::from("--reason"));
    assert_eq!(words[5], OsString::from(lock_words(&a_nonce())));
    assert_eq!(&words[6..], ["-b", "topic", "../topic", "main"].iter().map(OsString::from).collect::<Vec<OsString>>().as_slice());
    assert!(words[5].to_string_lossy().starts_with(LOCK_WORDS_PREFIX), "the reason is the recomputable sentence");
    assert!(!words[5].to_string_lossy().contains('\n'), "Git completes the line itself");
}

#[test]
fn a_person_lock_reason_or_checkout_is_refused() {
    for option in ["--lock", "--reason", "--reason=custom", "--checkout", "--no-checkout"] {
        let mut given: Vec<OsString> = an_add();
        given.push(OsString::from(option));
        let found: Option<&OsString> = person_lock_word(given.as_slice());
        assert_eq!(found.map(|word| return word.to_string_lossy().into_owned()), Some(String::from(option)), "{option} is the person's");
        let refusal: String = the_locked_add_words(given.as_slice(), 2, &a_nonce()).expect_err("the route refuses the person's lock");
        assert!(refusal.contains("takes the managed lock itself"), "{refusal}");
    }
    assert_eq!(person_lock_word(an_add().as_slice()), None, "an ordinary add carries none of them");
}

#[test]
fn an_insertion_point_outside_the_arguments_refuses() {
    for at in [0, 99] {
        let refusal: String = the_locked_add_words(an_add().as_slice(), at, &a_nonce()).expect_err("the point is outside the words");
        assert!(refusal.contains("insertion point is outside"), "{refusal}");
    }
}
