//! What:
//!  Controls for `--rules`,
//!  `--init` and `--print-config`.
//! Why:
//!  These modes describe the tool and its configuration;
//!  their output must agree with what
//! configuration parsing and matching actually do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('modes', () => { /* listing matches registry, starter parses, effective config states */ });
//! ```

/// Import the modes under test and the registries they must agree with.
use super::{
    RULES, STARTER_CONFIGURATION, SetupError, init_configuration, print_configuration, rule_info,
    rules_listing,
};
use crate::config_lookup::CONFIG_NAME;
use crate::configuration::{ConfigBlock, parse_configuration};
use crate::configuration_rules::RULE_IDS;
use crate::directive_findings::UNUSED;
use crate::run_failure::PROCESSING_FAILURE;
use crate::run_plan::ConfigStore;
use crate::run_test_support::{read, records, write};
use crate::rust_rule_settings::DEFAULT_MAX_LINES;
use crate::test_fs::Fixture;
use monochromatic_jsonc_edit::{
    JsoncPathSegment, JsoncValue, jsonc_key_path, jsonc_lookup, parse_jsonc,
};
use std::path::Path;

/// Markdown rules without a fixer:
///  their remedy is structural,
///  so no grouped edit can repair them.
const UNFIXABLE_MARKDOWN: [&str; 5] = [
    "markdown/no-pipe-tables",
    "markdown/heading-increment",
    "markdown/no-duplicate-heading",
    "markdown/single-h1",
    "markdown/no-emphasis-as-heading",
];

/// Rust rules no inline directive can suppress:
///  the budget and documentation ratchets,
///  the compiler-backed closure check,
///  which rustc's own `expect` attribute controls,
///  and the two crate-wide layout checks,
///  whose findings belong to a folder or a loop of files rather than to one source line.
const UNSUPPRESSIBLE_RUST: [&str; 5] = [
    "rust/max-lines",
    "rust/require-rustdoc",
    "rust/avoidable-closure",
    "rust/folder-uses-links",
    "rust/file-order-in-folder",
];

/// The listing has one JSON object per registered rule,
///  in registry order,
///  with its capabilities.
#[test]
fn the_listing_describes_every_registered_rule_in_order() {
    let listing: String = rules_listing().expect("listing");
    let lines: Vec<serde_json::Value> = records(listing.as_str());
    assert_eq!(lines.len(), RULE_IDS.len());
    assert_eq!(RULES.len(), RULE_IDS.len());
    for (index, id) in RULE_IDS.iter().enumerate() {
        assert_eq!(lines[index]["id"], *id);
        assert_eq!(RULES[index].id, *id);
    }
    // The first and last lines, byte for byte.
    assert_eq!(
        listing.lines().next(),
        Some(r#"{"id":"rust/max-lines","fixable":false,"options":["max"],"suppressible":false}"#)
    );
    assert_eq!(
        listing.lines().last(),
        Some(
            r#"{"id":"shell/no-ampersand","fixable":false,"options":["max"],"suppressible":false}"#
        )
    );
    // Fixability is exactly the Markdown rules whose remedy is a grouped edit.
    for id in RULE_IDS {
        let rule = rule_info(*id).expect("registered rule is described");
        let fixable: bool = id.starts_with("markdown/") && !UNFIXABLE_MARKDOWN.contains(id);
        assert_eq!(rule.fixable, fixable, "{id}");
    }
    assert!(listing.ends_with('\n'));
}

/// Exactly the line budget,
///  the documentation requirement,
///  the compiler-backed closure check and every shell rule refuse inline directives,
///  and the lookup answers by the identifier a finding carries.
///  The closure check is suppressed by rustc's own `expect` attribute only,
///  and the shell rules are held by configuration ratchets only.
#[test]
fn only_the_budget_rustdoc_closure_and_shell_rules_declare_themselves_unsuppressible() {
    // Suppressibility is exactly the rules outside the shell namespace and the five crate-wide Rust ratchets.
    for id in RULE_IDS {
        let rule = rule_info(*id).expect("registered rule is described");
        let suppressible: bool = !(id.starts_with("shell/") || UNSUPPRESSIBLE_RUST.contains(id));
        assert_eq!(rule.suppressible, suppressible, "{id}");
    }
    // Core and directive findings are not rules, so they are outside the registry and never suppressible.
    assert!(rule_info(PROCESSING_FAILURE).is_none());
    assert!(rule_info(UNUSED).is_none());
}

/// The starter is valid configuration that selects every rule except the Cargo-backed one,
///  the closure check,
///  which needs a library path that a starter cannot know,
///  and the shell rules,
///  which select workflow step names that only a repository with workflows can write.
#[test]
fn the_starter_configuration_is_valid_and_selects_the_documented_rules() {
    let blocks: Vec<ConfigBlock> =
        parse_configuration(STARTER_CONFIGURATION).expect("starter parses");
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].files, ["**/*.rs"]);
    assert_eq!(blocks[1].files, ["**/*.md", "**/*.mdx"]);
    let mut selected: Vec<String> = Vec::<String>::new();
    for block in &blocks {
        for entry in block.rules.entries().expect("rules record") {
            selected.push(String::from_utf16(&entry.key.units).expect("rule id"));
        }
    }
    let mut expected: Vec<String> = Vec::<String>::new();
    for id in RULE_IDS {
        if *id != "rust/require-explicit-types"
            && *id != "rust/avoidable-closure"
            && *id != "rust/folder-uses-links"
            && *id != "rust/file-order-in-folder"
            && !id.starts_with("shell/")
        {
            expected.push(String::from(*id));
        }
    }
    assert_eq!(selected, expected);
}

/// `--init` writes the starter once and refuses to overwrite,
///  leaving the existing bytes alone.
#[test]
fn init_creates_once_and_never_overwrites() {
    let fixture: Fixture = Fixture::new();
    let created = init_configuration(&fixture.path).expect("first init");
    assert_eq!(created, fixture.path.join(CONFIG_NAME));
    assert_eq!(read(&fixture.path, CONFIG_NAME), STARTER_CONFIGURATION);
    write(&fixture.path, CONFIG_NAME, "[] // mine\n");
    let error: SetupError = init_configuration(&fixture.path).expect_err("second init");
    assert!(error.message.contains(CONFIG_NAME), "{}", error.message);
    assert!(
        error.message.contains("never overwrites"),
        "{}",
        error.message
    );
    assert_eq!(read(&fixture.path, CONFIG_NAME), "[] // mine\n");
    let missing = init_configuration(&fixture.path.join("absent-directory"));
    assert!(missing.is_err());
}

/// Read one string member from printed JSONC.
fn member(document: &JsoncValue, key: &str) -> Option<String> {
    let value: &JsoncValue = jsonc_lookup(document, jsonc_key_path([key]).as_slice()).ok()?;
    return Some(String::from_utf16(value.text_units()?).expect("text"));
}

/// The four states are distinguished,
///  and configured rules are the merged,
///  defaulted settings.
#[test]
fn effective_configuration_distinguishes_its_four_states() {
    let fixture: Fixture = Fixture::new();
    let root: &Path = fixture.path.as_path();
    let mut empty: ConfigStore = ConfigStore::new(root, None).expect("store");
    let none: JsoncValue = parse_jsonc(
        print_configuration(&mut empty, root, Path::new("src/a.rs"))
            .expect("print")
            .as_str(),
    )
    .expect("printed JSONC parses");
    assert_eq!(member(&none, "file").as_deref(), Some("src/a.rs"));
    assert_eq!(member(&none, "state").as_deref(), Some("no-configuration"));
    assert_eq!(member(&none, "configuration"), None);
    write(
        root,
        CONFIG_NAME,
        r#"[
          { "ignores": ["vendor/"] },
          { "files": ["**/*.rs"], "rules": { "rust/max-lines": { "severity": "error" } } },
          { "files": ["src/**/*.rs"], "rules": { "rust/max-lines": { "severity": "warn" } } }
        ]"#,
    );
    let mut store: ConfigStore = ConfigStore::new(root, None).expect("store");
    let configured_text: String =
        print_configuration(&mut store, root, Path::new("src/a.rs")).expect("print");
    assert!(configured_text.ends_with("}\n"));
    // The document is strict JSON: a parser that rejects trailing commas and comments accepts it.
    let strict: serde_json::Value =
        serde_json::from_str::<serde_json::Value>(configured_text.as_str()).expect("strict JSON");
    assert_eq!(strict["rules"]["rust/max-lines"]["severity"], "warn");
    assert_eq!(
        strict["rules"]["rust/max-lines"]["max"].as_u64(),
        Some(DEFAULT_MAX_LINES as u64)
    );
    assert_eq!(strict["state"], "configured");
    let configured: JsoncValue = parse_jsonc(configured_text.as_str()).expect("parses");
    assert_eq!(member(&configured, "state").as_deref(), Some("configured"));
    assert_eq!(
        member(&configured, "configuration"),
        Some(root.join(CONFIG_NAME).to_string_lossy().into_owned())
    );
    assert_eq!(
        member(&configured, "base"),
        Some(root.to_string_lossy().into_owned())
    );
    let path: Vec<JsoncPathSegment> = jsonc_key_path(["rules", "rust/max-lines", "severity"]);
    let severity: &JsoncValue = jsonc_lookup(&configured, path.as_slice()).expect("severity");
    // The later block's severity wins, and the rule's default option is filled in.
    assert_eq!(
        String::from_utf16(severity.text_units().expect("text")).expect("text"),
        "warn"
    );
    let max_path: Vec<JsoncPathSegment> = jsonc_key_path(["rules", "rust/max-lines", "max"]);
    let max: &JsoncValue = jsonc_lookup(&configured, max_path.as_slice()).expect("max");
    let default_max: String = DEFAULT_MAX_LINES.to_string();
    assert_eq!(max.number_token(), Some(default_max.as_str()));
    for (file, state) in [
        ("vendor/a.rs", "ignored"),
        ("doc/a.md", "unconfigured"),
        ("doc/a.md/0.rs", "configured"),
    ] {
        let printed: JsoncValue = parse_jsonc(
            print_configuration(&mut store, root, Path::new(file))
                .expect("print")
                .as_str(),
        )
        .expect("parses");
        assert_eq!(member(&printed, "state").as_deref(), Some(state), "{file}");
        assert_eq!(member(&printed, "file").as_deref(), Some(file));
    }
}
