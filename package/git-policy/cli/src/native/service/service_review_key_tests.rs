//! Controls for the credential type: it never prints, and it is chosen as the script chooses it.

use super::*;

/// A stand-in credential.
const SECRET: &str = "test-key-not-a-secret";

/// An empty text is no key, and a key's debug output hides its text.
#[test]
fn a_key_never_prints_its_text() {
    assert_eq!(ReviewKey::new(""), None);
    let key: ReviewKey = ReviewKey::new(SECRET).expect("a key");
    assert_eq!(key.expose(), SECRET);
    assert!(!format!("{key:?}").contains(SECRET));
    assert!(!format!("{:?}", Some(key.clone())).contains(SECRET));
    assert!(!format!("{:#?}", vec![key]).contains(SECRET));
}

/// The first variable that is set wins even when empty; the second is used only when the first is unset.
#[test]
fn the_first_set_variable_wins() {
    let both = |name: &str| -> Option<String> { return Some(format!("value-of-{name}")); };
    assert_eq!(key_from_environment(&both).expect("a key").expose(), format!("value-of-{}", KEY_VARIABLES[0]));
    let empty_first = |name: &str| -> Option<String> { return if name == KEY_VARIABLES[0] { Some(String::new()) } else { Some(String::from("second")) }; };
    assert_eq!(key_from_environment(&empty_first), None, "an empty first key does not fall back");
    let only_second = |name: &str| -> Option<String> { return if name == KEY_VARIABLES[1] { Some(String::from("second")) } else { None }; };
    assert_eq!(key_from_environment(&only_second).expect("a key").expose(), "second");
    let none = |_: &str| -> Option<String> { return None; };
    assert_eq!(key_from_environment(&none), None);
}
