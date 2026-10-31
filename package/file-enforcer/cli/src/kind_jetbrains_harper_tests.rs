//! What:
//!  Tests for the `jetbrains-harper` rule kind through the real run entry point,
//! in disposable homes seeded with synthetic JetBrains files.
//! Why:
//!  The rule writes outside any repository,
//! so every test uses a scratch directory as the home and never the owner's `~/.config/JetBrains`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('jetbrains-harper rule', () => { it('applies the policy and is idempotent', () => {}); });
//! ```

/// Imports the fixtures.
use crate::harper_fixtures::{BASE_KEY, LANGUAGE_XML, POLICY_CONFIG, SCOPED_KEY, USER_XML};
/// Imports the XML readers used for decoded assertions.
use crate::harper_xml::{find_entry, option_value, parse};
/// Imports the kind under test.
use crate::kind_jetbrains_harper::JetbrainsHarperBuilder;
/// Imports the tree and run helpers.
use crate::test_support::{count_code, read_text, run_config, scratch_dir, write_files};
/// Imports `ConfigScope` from `monochromatic_file_enforcer::config_decode`.
use monochromatic_file_enforcer::config_decode::ConfigScope;
/// Imports `RuleConfig` from `monochromatic_file_enforcer::config_types`.
use monochromatic_file_enforcer::config_types::RuleConfig;
/// Imports `Event` from `monochromatic_file_enforcer::event`.
use monochromatic_file_enforcer::event::Event;
/// Imports `RuleRegistry` from `monochromatic_file_enforcer::registry`.
use monochromatic_file_enforcer::registry::RuleRegistry;
/// Imports `RunMode` from `monochromatic_file_enforcer::run`.
use monochromatic_file_enforcer::run::RunMode;
/// Imports `RuleBuilder` from `monochromatic_file_enforcer::rule`.
use monochromatic_file_enforcer::rule::RuleBuilder;
/// Imports `Value` from `serde_json`.
use serde_json::Value;
/// Imports `Path` from `std::path`.
use std::path::Path;

/// The newest product's options directory in the fixtures.
const NEWEST: &str = ".config/JetBrains/IntelliJIdea2026.2/options";

/// The registry under test.
fn registry() -> RuleRegistry {
    let mut registry = RuleRegistry::with_defaults();
    registry.register("jetbrains-harper", Box::new(JetbrainsHarperBuilder));
    return registry;
}

/// Run the policy config in the per-user scope.
fn run_user(home: &Path, mode: RunMode) -> (i32, Vec<Event>) {
    return run_config(home, POLICY_CONFIG, ConfigScope::User, mode, &registry());
}

/// Seed the newest product,
/// an older product and a backup copy of the newest.
fn seed_home(home: &Path) {
    let language_path = format!("{NEWEST}/LanguageServersSettings.xml");
    let user_path = format!("{NEWEST}/UserDefinedLanguageServerSettings.xml");
    write_files(home, &[(&language_path, LANGUAGE_XML), (&user_path, USER_XML)]);
    let older = ".config/JetBrains/IntelliJIdea2025.3/options";
    write_files(home, &[(&format!("{older}/LanguageServersSettings.xml"), LANGUAGE_XML), (&format!("{older}/UserDefinedLanguageServerSettings.xml"), USER_XML)]);
    let backup = ".config/JetBrains/IntelliJIdea2026.2-backup/options";
    write_files(home, &[(&format!("{backup}/LanguageServersSettings.xml"), LANGUAGE_XML), (&format!("{backup}/UserDefinedLanguageServerSettings.xml"), USER_XML)]);
}

/// The decoded JSON configuration of a language settings entry.
fn config_of(home: &Path, key: &str) -> serde_json::Map<String, Value> {
    let text = read_text(home, &format!("{NEWEST}/LanguageServersSettings.xml"));
    let document = parse("language", &text).expect("well-formed");
    let entry = find_entry(&document, key).expect("entry");
    let json = option_value(&document, entry, "configurationContent").expect("config");
    return serde_json::from_str::<Value>(&json).expect("JSON").as_object().expect("object").clone();
}

/// Drift is reported,
/// `write` applies the policy to the newest product only,
/// and the result is stable.
#[test]
fn drift_is_detected_and_write_applies_the_policy_to_the_newest_product() {
    let home = scratch_dir();
    seed_home(home.path());
    let (check_code, check) = run_user(home.path(), RunMode::Check { expected_dir: None });
    assert_eq!(check_code, 1, "{check:?}");
    assert_eq!(count_code(&check, "drift-content"), 2, "both settings files of the newest product drift: {check:?}");
    assert_eq!(count_code(&check, "kind-not-wired"), 0);
    let (write_code, write) = run_user(home.path(), RunMode::Write);
    assert_eq!(write_code, 0, "{write:?}");
    let base = config_of(home.path(), BASE_KEY);
    for linter in ["UseTitleCase", "SplitWords", "PhrasalVerbAsCompoundNoun"] {
        assert_eq!(base[&format!("harper-ls.linters.{linter}")], Value::Bool(false), "{linter}");
    }
    assert_eq!(base["harper-ls.linters.Keep"], Value::Bool(true), "the user's own key is kept");
    let excludes: Vec<&str> = base["harper-ls.excludePatterns"].as_array().expect("array").iter().filter_map(Value::as_str).collect();
    assert_eq!(excludes, vec!["**/mine.md", "**/AGENTS.md", "**/CLAUDE.md"]);
    let scoped = config_of(home.path(), SCOPED_KEY);
    assert!(!scoped.contains_key("harper-ls.excludePatterns"), "a scoped server does not inherit the exclude list");
    assert_eq!(scoped["harper-ls.linters.OxfordComma"], Value::Bool(false));
    assert_eq!(scoped["harper-ls.linters.SplitWords"], Value::Bool(false), "scoped servers inherit the base linter switches");
    let language = read_text(home.path(), &format!("{NEWEST}/LanguageServersSettings.xml"));
    assert!(language.contains("<!-- keep this comment -->"), "a comment inside the managed base entry survives");
    let document = parse("language", &language).expect("well-formed");
    let schema = option_value(&document, find_entry(&document, BASE_KEY).expect("entry"), "configurationSchemaContent").expect("schema");
    assert!(schema.contains("harper-ls.linters.UseTitleCase"), "{schema}");
    let user = read_text(home.path(), &format!("{NEWEST}/UserDefinedLanguageServerSettings.xml"));
    let user_document = parse("user", &user).expect("well-formed");
    let scoped_entry = find_entry(&user_document, SCOPED_KEY).expect("scoped entry");
    assert_eq!(option_value(&user_document, scoped_entry, "commandLine").as_deref(), Some("harper-ls --stdio"));
    assert_eq!(option_value(&user_document, scoped_entry, "workingDir").as_deref(), Some("/work"));
    assert_eq!(option_value(&user_document, scoped_entry, "serverId").as_deref(), Some(SCOPED_KEY));
    assert!(user.contains("<option value=\"AGENTS.md\" />"));
    assert_eq!(read_text(home.path(), ".config/JetBrains/IntelliJIdea2025.3/options/LanguageServersSettings.xml"), LANGUAGE_XML, "an older product is untouched");
    assert_eq!(read_text(home.path(), ".config/JetBrains/IntelliJIdea2026.2-backup/options/UserDefinedLanguageServerSettings.xml"), USER_XML, "a backup is untouched");
    let (clean_code, clean) = run_user(home.path(), RunMode::Check { expected_dir: None });
    assert_eq!(clean_code, 0, "{clean:?}");
    let (again_code, again) = run_user(home.path(), RunMode::Write);
    assert_eq!((again_code, count_code(&again, "file-written")), (0, 0), "a conforming home is not rewritten: {again:?}");
}

/// Drift in one file only is repaired in that file only.
#[test]
fn a_single_drifting_file_is_repaired_alone() {
    let home = scratch_dir();
    seed_home(home.path());
    run_user(home.path(), RunMode::Write);
    let user_path = format!("{NEWEST}/UserDefinedLanguageServerSettings.xml");
    let language_before = read_text(home.path(), &format!("{NEWEST}/LanguageServersSettings.xml"));
    let edited = read_text(home.path(), &user_path).replace("Harper Language Server (AGENTS.md and CLAUDE.md)", "Hand edited");
    write_files(home.path(), &[(&user_path, &edited)]);
    let (check_code, check) = run_user(home.path(), RunMode::Check { expected_dir: None });
    assert_eq!((check_code, count_code(&check, "drift-content")), (1, 1), "{check:?}");
    let (write_code, write) = run_user(home.path(), RunMode::Write);
    assert_eq!((write_code, count_code(&write, "file-written")), (0, 1), "{write:?}");
    assert_eq!(read_text(home.path(), &format!("{NEWEST}/LanguageServersSettings.xml")), language_before);
}

/// A machine without the IDE,
/// without the settings files or without the base server is skipped with a warning.
#[test]
fn absent_ide_files_or_server_are_skipped_with_a_warning() {
    let bare = scratch_dir();
    let (bare_code, bare_events) = run_user(bare.path(), RunMode::Check { expected_dir: None });
    assert_eq!((bare_code, count_code(&bare_events, "jetbrains-harper-skipped")), (0, 1), "{bare_events:?}");
    let no_files = scratch_dir();
    write_files(no_files.path(), &[(&format!("{NEWEST}/LanguageServersSettings.xml"), LANGUAGE_XML)]);
    let (files_code, files_events) = run_user(no_files.path(), RunMode::Check { expected_dir: None });
    assert_eq!((files_code, count_code(&files_events, "jetbrains-harper-skipped")), (0, 1), "{files_events:?}");
    let no_server = scratch_dir();
    let stranger = USER_XML.replace("harper-ls", "other").replace("Harper Language Server", "Other");
    write_files(no_server.path(), &[(&format!("{NEWEST}/LanguageServersSettings.xml"), LANGUAGE_XML), (&format!("{NEWEST}/UserDefinedLanguageServerSettings.xml"), &stranger)]);
    let (server_code, server_events) = run_user(no_server.path(), RunMode::Write);
    assert_eq!((server_code, count_code(&server_events, "jetbrains-harper-skipped")), (0, 1), "{server_events:?}");
    assert_eq!(read_text(no_server.path(), &format!("{NEWEST}/LanguageServersSettings.xml")), LANGUAGE_XML, "nothing was written");
}

/// A newer product without settings files is a miss;
/// the older product is not edited instead.
#[test]
fn a_newer_product_without_files_is_not_replaced_by_an_older_one() {
    let home = scratch_dir();
    seed_home(home.path());
    std::fs::create_dir_all(home.path().join(".config/JetBrains/IntelliJIdea2026.3/options")).expect("newer product");
    let (code, events) = run_user(home.path(), RunMode::Write);
    assert_eq!((code, count_code(&events, "jetbrains-harper-skipped")), (0, 1), "{events:?}");
    assert_eq!(read_text(home.path(), &format!("{NEWEST}/LanguageServersSettings.xml")), LANGUAGE_XML);
}

/// Malformed XML,
/// malformed embedded JSON and a missing language entry each fail the run and write nothing.
#[test]
fn malformed_settings_fail_the_run_and_write_nothing() {
    let broken_xml = scratch_dir();
    write_files(broken_xml.path(), &[(&format!("{NEWEST}/LanguageServersSettings.xml"), "<application><map></application>"), (&format!("{NEWEST}/UserDefinedLanguageServerSettings.xml"), USER_XML)]);
    let (xml_code, xml_events) = run_user(broken_xml.path(), RunMode::Write);
    assert_eq!((xml_code, count_code(&xml_events, "harper-xml-invalid")), (1, 1), "{xml_events:?}");
    let broken_json = scratch_dir();
    let bad_config = LANGUAGE_XML.replace("{&#10;  &quot;harper-ls.linters.Keep&quot;: true,", "{ nope,");
    write_files(broken_json.path(), &[(&format!("{NEWEST}/LanguageServersSettings.xml"), &bad_config), (&format!("{NEWEST}/UserDefinedLanguageServerSettings.xml"), USER_XML)]);
    let (json_code, json_events) = run_user(broken_json.path(), RunMode::Write);
    assert_eq!((json_code, count_code(&json_events, "harper-json-invalid")), (1, 1), "{json_events:?}");
    assert_eq!(read_text(broken_json.path(), &format!("{NEWEST}/UserDefinedLanguageServerSettings.xml")), USER_XML, "the other file is not written either");
    let no_entry = scratch_dir();
    let other_key = LANGUAGE_XML.replace("srv-1", "srv-other");
    write_files(no_entry.path(), &[(&format!("{NEWEST}/LanguageServersSettings.xml"), &other_key), (&format!("{NEWEST}/UserDefinedLanguageServerSettings.xml"), USER_XML)]);
    let (entry_code, entry_events) = run_user(no_entry.path(), RunMode::Write);
    assert_eq!((entry_code, count_code(&entry_events, "harper-xml-invalid")), (1, 1), "{entry_events:?}");
}

/// The repository config refuses this kind:
/// it belongs to the per-user config.
#[test]
fn the_repository_scope_refuses_the_kind() {
    let dir = scratch_dir();
    let (code, events) = run_config(dir.path(), POLICY_CONFIG, ConfigScope::Repository, RunMode::Check { expected_dir: None }, &registry());
    assert_eq!(code, 2, "{events:?}");
}

/// The builder refuses an entry of another kind.
#[test]
fn the_builder_refuses_another_kind() {
    let other = RuleConfig::Mirror { id: "m".to_string(), from: "a".to_string(), to: "b".to_string() };
    let error = JetbrainsHarperBuilder.build(&other).err().expect("a mirror entry is refused");
    assert_eq!(error.code, "kind-unknown");
}
