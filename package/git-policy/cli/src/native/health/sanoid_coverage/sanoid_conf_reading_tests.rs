//! What:
//!  Controls of the `sanoid.conf` reading:
//!  file order,
//!  the values it keeps,
//!  the duplicate key sanoid resolves itself,
//!  and every text it refuses.
//! Why:
//!  The section order is the behaviour the exclusions depend on,
//!  and a reader that normalized a value or reordered a section would answer
//!  for a file sanoid reads differently,
//!  so each reading is proved against the text that produces it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readSanoidConf('[a]\n  autosnap = no # keep\n').sections[0].settings[0].value).toBe('no # keep');
//! ```

use super::{ConfigRefusal, ReadConfig, SettingLookup, read_sanoid_conf};

/// The sample the design's own section carries,
/// with a comment between two sections.
const SAMPLE: &str = "# /etc/sanoid/sanoid.conf\n[tank/home]\n  use_template = home\n  recursive = yes\n\n; the cache\n[tank/varcache]\n  autosnap = no\n";

/// The sections come in file order with their lines.
#[test]
fn the_sections_come_in_file_order_with_their_lines() {
    let config: ReadConfig = read_sanoid_conf(SAMPLE).expect("the sample reads");
    assert_eq!(config.sections.len(), 2);
    assert_eq!(config.sections[0].dataset, "tank/home");
    assert_eq!(config.sections[0].line, 2);
    assert_eq!(config.sections[1].dataset, "tank/varcache");
    assert_eq!(config.sections[1].line, 7);
    assert_eq!(config.position("tank/varcache"), Some(1));
    assert_eq!(config.position("tank/absent"), None);
}

/// A setting keeps its value trimmed of its ends and keeps a trailing comment
/// as part of it,
/// which is what sanoid does.
#[test]
fn a_setting_keeps_its_value_and_a_trailing_comment() {
    let config: ReadConfig = read_sanoid_conf("  [tank/home]  \r\n   autosnap   =   no # keep   \r\n").expect("the text reads");
    let section = &config.sections[0];
    assert_eq!(section.dataset, "tank/home");
    assert_eq!(section.settings[0].setting.key, "autosnap");
    assert_eq!(section.settings[0].setting.value, "no # keep");
    assert!(matches!(section.setting("autosnap"), SettingLookup::Found { ref value, line } if value == "no # keep" && line == 2));
    assert!(!section.snapshots_off(), "a trailing comment is part of the value");
}

/// A key sanoid reads as false turns a section's snapshots off.
#[test]
fn a_false_word_turns_the_sections_snapshots_off() {
    let config: ReadConfig = read_sanoid_conf("[tank/varcache]\n  autosnap = Off\n").expect("the text reads");
    assert!(config.sections[0].snapshots_off());
}

/// A key the section does not write is absent.
#[test]
fn a_key_the_section_does_not_write_is_absent() {
    let config: ReadConfig = read_sanoid_conf("[tank/home]\n  recursive = yes\n").expect("the text reads");
    assert_eq!(config.sections[0].setting("autosnap"), SettingLookup::Absent);
}

/// A key spelled with other capitals is kept apart from an absent one,
/// because sanoid's keys are case-sensitive.
#[test]
fn a_key_with_other_capitals_is_a_case_variant() {
    let config: ReadConfig = read_sanoid_conf("[tank/home]\n  Autosnap = no\n").expect("the text reads");
    assert_eq!(config.sections[0].setting("autosnap"), SettingLookup::CaseVariant { spelling: String::from("Autosnap"), line: 2 });
    assert!(!config.sections[0].snapshots_off(), "a case variant is not the key sanoid reads");
}

/// A repeated key keeps its first value and names both lines,
/// as sanoid resolves it.
#[test]
fn a_repeated_key_keeps_its_first_value_and_names_both_lines() {
    let config: ReadConfig = read_sanoid_conf("[tank/home]\n  autosnap = no\n  autosnap = yes\n").expect("the text reads");
    let section = &config.sections[0];
    assert_eq!(section.settings.len(), 1);
    assert_eq!(section.written_value("autosnap"), Some("no"));
    assert_eq!(section.duplicates.len(), 1);
    assert_eq!(section.duplicates[0].key, "autosnap");
    assert_eq!(section.duplicates[0].first_line, 2);
    assert_eq!(section.duplicates[0].line, 3);
}

/// Two sections for one dataset are refused with both lines.
#[test]
fn two_sections_for_one_dataset_are_refused_with_both_lines() {
    let refused: ConfigRefusal = read_sanoid_conf("[tank/home]\n  recursive = yes\n[tank/home]\n  autosnap = no\n").expect_err("a repeated section is refused");
    assert_eq!(refused, ConfigRefusal::DuplicateSection { dataset: String::from("tank/home"), first_line: 1, again_line: 3 });
    assert!(refused.reason().contains("line 1") && refused.reason().contains("line 3"));
}

/// A setting before the first section is refused with its line.
#[test]
fn a_setting_before_the_first_section_is_refused_with_its_line() {
    let refused: ConfigRefusal = read_sanoid_conf("autosnap = no\n[tank/home]\n").expect_err("a setting without a section is refused");
    assert!(matches!(refused, ConfigRefusal::Line { line: 1, .. }), "{refused:?}");
    assert!(refused.reason().contains("before the first section"));
}

/// A header that does not close its bracket is refused.
#[test]
fn a_header_that_does_not_close_its_bracket_is_refused() {
    let refused: ConfigRefusal = read_sanoid_conf("[tank/home\n  autosnap = no\n").expect_err("an unclosed header is refused");
    assert!(matches!(refused, ConfigRefusal::Line { line: 1, .. }), "{refused:?}");
}

/// A header that names no dataset is refused.
#[test]
fn a_header_that_names_no_dataset_is_refused() {
    let refused: ConfigRefusal = read_sanoid_conf("[   ]\n").expect_err("an empty header is refused");
    assert!(matches!(refused, ConfigRefusal::Line { line: 1, .. }), "{refused:?}");
}

/// A line that is neither a setting nor a comment is refused.
#[test]
fn a_line_that_is_not_a_setting_is_refused_with_its_line() {
    let refused: ConfigRefusal = read_sanoid_conf("[tank/home]\n  autosnap no\n").expect_err("a line without a separator is refused");
    assert!(matches!(refused, ConfigRefusal::Line { line: 2, .. }), "{refused:?}");
    assert!(refused.reason().contains("key = value"));
}

/// A setting that names no key is refused.
#[test]
fn a_setting_that_names_no_key_is_refused() {
    let refused: ConfigRefusal = read_sanoid_conf("[tank/home]\n  = no\n").expect_err("an empty key is refused");
    assert!(matches!(refused, ConfigRefusal::Line { line: 2, .. }), "{refused:?}");
}

/// A comment line and a blank line are skipped wherever they stand.
#[test]
fn a_comment_and_a_blank_line_are_skipped() {
    let config: ReadConfig = read_sanoid_conf("\n# one\n   ; two\n[tank/home]\n\n# three\n  recursive = yes\n").expect("the text reads");
    assert_eq!(config.sections.len(), 1);
    assert_eq!(config.sections[0].settings.len(), 1);
}
