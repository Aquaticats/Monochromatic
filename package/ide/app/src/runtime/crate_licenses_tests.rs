//! The crate license list on small inputs:
//!  parsing,
//!  headings,
//!  wrapped crate names,
//!  and source paths.

use super::{crate_relative, parse, wrap};

/// A list with a crate file text shared by two crates and a standard text used by one.
const LIST: &[u8] = br#"{"licenses": [
{"id": "MIT", "name": "MIT License", "source": "/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.104/LICENSE-MIT", "crates": [{"name": "anyhow", "version": "1.0.104"}, {"name": "itoa", "version": "1.0.15"}], "text": "Copyright (c) The anyhow authors\n"},
{"id": "MPL-2.0", "name": "Mozilla Public License 2.0", "source": null, "crates": [{"name": "helix-core", "version": "25.7.1"}], "text": "Mozilla Public License Version 2.0\n"}
]}"#;

#[test]
fn every_entry_gets_its_heading_crates_and_source() {
    let list = parse(LIST).expect("well-formed list");
    assert_eq!(list.licenses.len(), 2);
    let shared = &list.licenses[0];
    assert_eq!(
        shared.heading(),
        "Rust crates under MIT License (MIT): 2 crates"
    );
    assert_eq!(
        shared.origin(),
        vec![
            "Used by: anyhow 1.0.104, itoa 1.0.15".to_string(),
            "Text from the crate file anyhow-1.0.104/LICENSE-MIT".to_string(),
        ]
    );
    let standard = &list.licenses[1];
    assert_eq!(
        standard.heading(),
        "Rust crates under Mozilla Public License 2.0 (MPL-2.0): 1 crate"
    );
    assert_eq!(
        standard.origin(),
        vec![
            "Used by: helix-core 25.7.1".to_string(),
            "Text: the standard text of this license (no crate file was recognized)".to_string(),
        ]
    );
    assert_eq!(standard.text, "Mozilla Public License Version 2.0\n");
}

#[test]
fn long_crate_lists_wrap_under_the_prefix_and_keep_long_names_whole() {
    let mut items: Vec<String> = Vec::new();
    for index in 0..30 {
        items.push(format!("crate-{index} 1.0.0,"));
    }
    let lines = wrap("Used by: ", &items);
    assert!(lines.len() > 1, "{lines:?}");
    for line in &lines {
        assert!(line.len() <= 78, "{line}");
    }
    assert!(lines[1].starts_with("         crate-"), "{lines:?}");
    let joined = lines.join(" ");
    let mut crate_words = 0;
    for word in joined.split_whitespace() {
        if word.starts_with("crate-") {
            crate_words += 1;
        }
    }
    assert_eq!(crate_words, 30);
    let long = vec!["x".repeat(100)];
    assert_eq!(
        wrap("Used by: ", &long),
        vec![format!("Used by: {}", "x".repeat(100))]
    );
}

#[test]
fn registry_and_git_sources_are_shown_from_the_crate_folder_on() {
    assert_eq!(
        crate_relative(
            "/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sonic-number-0.1.3/licenses/LICENSE-sonic_cpp"
        ),
        "sonic-number-0.1.3/licenses/LICENSE-sonic_cpp"
    );
    assert_eq!(
        crate_relative(
            "/cargo/git/checkouts/helix-b99af130ded19729/ba40e54/helix-lsp-types/LICENSE"
        ),
        "helix-lsp-types/LICENSE"
    );
    assert_eq!(crate_relative("/elsewhere/LICENSE"), "/elsewhere/LICENSE");
}

#[test]
fn a_list_this_program_cannot_read_names_the_file_and_the_remedy() {
    let error = parse(br#"{"licenses": [{"id": "MIT"}]}"#).expect_err("missing fields");
    let message = format!("{error:#}");
    assert!(message.contains("LICENSES/crates.json"), "{message}");
    assert!(message.contains("fresh copy"), "{message}");
}
