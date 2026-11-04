//! Controls for the pieces of the merged listing; the whole listing is compared with `git config --list` in
//! `repo_read_query_refs_tests.rs`.

use super::*;

/// Environment keys are canonicalized as Git does: section and variable lower-cased, subsection kept.
#[test]
fn environment_keys_are_canonicalized() {
    assert_eq!(canonical_key("Core.HooksPath").as_deref(), Some("core.hookspath"));
    assert_eq!(canonical_key("Remote.Origin Mixed.URL").as_deref(), Some("remote.Origin Mixed.url"));
    assert!(canonical_key("nodot").is_none());
    assert!(canonical_key("core.").is_none());
    assert!(canonical_key("core.1bad").is_none());
    assert!(canonical_key(".name").is_none());
}

/// The entries print as `scope NUL key LF value NUL`, with no line feed for a bare name.
#[test]
fn entries_print_in_the_listing_spelling() {
    let listed = vec![
        ListedEntry { scope: "local", entry: ConfigEntry { key: String::from("a.b"), value: Some(b"v".to_vec()) } },
        ListedEntry { scope: "command", entry: ConfigEntry { key: String::from("c.d"), value: None } },
    ];
    assert_eq!(listing_bytes(listed.as_slice()), b"local\0a.b\nv\0command\0c.d\0".to_vec());
}
