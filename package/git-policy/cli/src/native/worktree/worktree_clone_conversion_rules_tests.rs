//! Controls for the conversion fence's decision:
//! one per conversion class Git's checkout can apply,
//! with the sizes that prove a carried file holds its blob.

use super::*;

/// The config reading: `core.autocrlf` in Git's three readings and `core.eol`.
fn rules_of(entries: &[(&str, Option<&str>)]) -> ConversionRules {
    let owned: Vec<ConfigEntry> = entries.iter().map(|(key, value)| return entry_of(key, *value)).collect();
    return conversion_rules_from_config(owned.as_slice());
}

fn entry_of(key: &str, value: Option<&str>) -> ConfigEntry {
    return ConfigEntry { scope: String::from("local"), key: String::from(key), value: value.map(|text| return text.as_bytes().to_vec()) };
}

/// The attributes one test names.
fn attributes_of(text: TextAttribute, eol: EolAttribute) -> PathAttributes {
    return PathAttributes { text, eol, ..PathAttributes::default() };
}

/// An ordinary file of 18 bytes whose blob is 18 bytes.
const EQUAL: (u64, Option<u64>) = (18, Some(18));

#[test]
fn no_rule_and_no_configuration_carries_the_file() {
    let rules = rules_of(&[]);
    assert_eq!(rules.action_of(TextAttribute::Unspecified, EolAttribute::Unspecified), CrlfAction::Binary);
    assert_eq!(decide(&PathAttributes::default(), &rules, false, EQUAL.0, EQUAL.1), FenceVerdict::Carry);
}

#[test]
fn text_alone_over_a_lf_platform_never_adds_a_carriage_return_and_needs_the_size() {
    let rules = rules_of(&[]);
    let held = attributes_of(TextAttribute::Set, EolAttribute::Unspecified);
    assert_eq!(rules.action_of(held.text, held.eol), CrlfAction::Text);
    assert_eq!(decide(&held, &rules, false, 18, Some(18)), FenceVerdict::Carry);
    assert!(matches!(decide(&held, &rules, false, 20, Some(18)), FenceVerdict::WriteThroughGit(_)));
    assert!(matches!(decide(&held, &rules, false, 18, None), FenceVerdict::WriteThroughGit(_)));
    assert!(needs_blob_size(&held, &rules));
}

#[test]
fn text_with_eol_crlf_may_add_carriage_returns_so_git_writes_it() {
    let rules = rules_of(&[]);
    let held = attributes_of(TextAttribute::Set, EolAttribute::Crlf);
    assert_eq!(rules.action_of(held.text, held.eol), CrlfAction::TextCrlf);
    assert_eq!(rules.output_ending(CrlfAction::TextCrlf), OutputEnding::Crlf);
    assert!(!needs_blob_size(&held, &rules));
    assert!(matches!(decide(&held, &rules, false, 18, Some(18)), FenceVerdict::WriteThroughGit(_)));
}

#[test]
fn text_auto_eol_lf_is_the_identity_on_the_way_out_and_needs_the_size() {
    let rules = rules_of(&[]);
    let held = attributes_of(TextAttribute::Auto, EolAttribute::Lf);
    assert_eq!(rules.action_of(held.text, held.eol), CrlfAction::AutoInput);
    assert_eq!(rules.output_ending(CrlfAction::AutoInput), OutputEnding::Lf);
    assert_eq!(decide(&held, &rules, false, 18, Some(18)), FenceVerdict::Carry);
    assert!(matches!(decide(&held, &rules, false, 20, Some(18)), FenceVerdict::WriteThroughGit(_)));
}

#[test]
fn text_auto_eol_crlf_may_add_carriage_returns() {
    let held = attributes_of(TextAttribute::Auto, EolAttribute::Crlf);
    let rules = rules_of(&[]);
    assert_eq!(rules.action_of(held.text, held.eol), CrlfAction::AutoCrlf);
    assert!(matches!(decide(&held, &rules, false, 18, Some(18)), FenceVerdict::WriteThroughGit(_)));
}

#[test]
fn core_autocrlf_true_writes_every_text_path_without_an_attribute() {
    let rules = rules_of(&[("core.autocrlf", Some("true"))]);
    assert_eq!(rules.action_of(TextAttribute::Unspecified, EolAttribute::Unspecified), CrlfAction::AutoCrlf);
    assert!(matches!(decide(&PathAttributes::default(), &rules, false, 18, Some(18)), FenceVerdict::WriteThroughGit(_)));
}

#[test]
fn core_autocrlf_input_only_strips_carriage_returns_so_the_size_decides() {
    let rules = rules_of(&[("core.autocrlf", Some("input"))]);
    assert_eq!(rules.action_of(TextAttribute::Unspecified, EolAttribute::Unspecified), CrlfAction::AutoInput);
    assert_eq!(decide(&PathAttributes::default(), &rules, false, 18, Some(18)), FenceVerdict::Carry);
    assert!(matches!(decide(&PathAttributes::default(), &rules, false, 20, Some(18)), FenceVerdict::WriteThroughGit(_)));
}

#[test]
fn text_unset_keeps_the_file_even_under_core_autocrlf_true() {
    let rules = rules_of(&[("core.autocrlf", Some("true"))]);
    let held = attributes_of(TextAttribute::Unset, EolAttribute::Crlf);
    assert_eq!(rules.action_of(held.text, held.eol), CrlfAction::Binary);
    assert_eq!(decide(&held, &rules, false, 18, Some(18)), FenceVerdict::Carry);
}

#[test]
fn core_eol_crlf_gives_text_alone_the_carriage_return_ending() {
    let rules = rules_of(&[("core.eol", Some("crlf"))]);
    assert_eq!(rules.action_of(TextAttribute::Set, EolAttribute::Unspecified), CrlfAction::Text);
    assert_eq!(rules.output_ending(CrlfAction::Text), OutputEnding::Crlf);
    assert!(matches!(decide(&attributes_of(TextAttribute::Set, EolAttribute::Unspecified), &rules, false, 18, Some(18)), FenceVerdict::WriteThroughGit(_)));
}

#[test]
fn ident_filter_and_encoding_are_written_by_git() {
    let rules = rules_of(&[]);
    for held in [
        PathAttributes { ident: true, ..PathAttributes::default() },
        PathAttributes { filter: true, ..PathAttributes::default() },
        PathAttributes { encoding: true, ..PathAttributes::default() },
    ] {
        assert!(matches!(decide(&held, &rules, false, 18, Some(18)), FenceVerdict::WriteThroughGit(_)), "{held:?}");
        assert!(!needs_blob_size(&held, &rules), "{held:?}");
    }
}

#[test]
fn a_text_value_git_does_not_name_is_written_by_git() {
    let rules = rules_of(&[]);
    let held = attributes_of(TextAttribute::Other, EolAttribute::Unspecified);
    assert!(matches!(decide(&held, &rules, false, 18, Some(18)), FenceVerdict::WriteThroughGit(_)));
}

#[test]
fn a_link_is_carried_and_never_converted() {
    let rules = rules_of(&[("core.autocrlf", Some("true"))]);
    let held = PathAttributes { ident: true, ..PathAttributes::default() };
    assert_eq!(decide(&held, &rules, true, 11, Some(11)), FenceVerdict::Carry);
}

#[test]
fn a_setting_git_does_not_know_is_read_as_the_careful_one() {
    let rules = rules_of(&[("core.autocrlf", Some("perhaps"))]);
    assert_eq!(rules.auto_crlf, AutoCrlf::True);
    assert_eq!(rules_of(&[("core.autocrlf", Some("no"))]).auto_crlf, AutoCrlf::False);
    assert_eq!(rules_of(&[("core.autocrlf", None)]).auto_crlf, AutoCrlf::False);
    assert_eq!(rules_of(&[("core.eol", Some("native"))]).core_eol, EolSetting::Unset);
}

#[test]
fn the_attribute_triples_of_git_become_the_rules_of_one_path() {
    let mut held = PathAttributes::default();
    apply_attribute(&mut held, b"text", b"auto");
    apply_attribute(&mut held, b"eol", b"crlf");
    apply_attribute(&mut held, b"ident", b"set");
    apply_attribute(&mut held, b"filter", b"lfs");
    apply_attribute(&mut held, b"working-tree-encoding", b"UTF-16LE");
    assert_eq!(held, PathAttributes { text: TextAttribute::Auto, eol: EolAttribute::Crlf, filter: true, ident: true, encoding: true });
    let mut reset = PathAttributes { text: TextAttribute::Set, eol: EolAttribute::Crlf, filter: true, ident: true, encoding: true };
    apply_attribute(&mut reset, b"text", b"unset");
    apply_attribute(&mut reset, b"eol", b"unspecified");
    apply_attribute(&mut reset, b"ident", b"unset");
    apply_attribute(&mut reset, b"filter", b"unset");
    apply_attribute(&mut reset, b"working-tree-encoding", b"unset");
    assert_eq!(reset, PathAttributes { text: TextAttribute::Unset, eol: EolAttribute::Unspecified, filter: false, ident: false, encoding: false });
}
