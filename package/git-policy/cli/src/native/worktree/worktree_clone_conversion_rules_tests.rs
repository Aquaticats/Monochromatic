//! Controls for the conversion fence's decision:
//! one per conversion class Git's checkout can apply,
//! with the sizes that prove a carried file holds its blob,
//! and the `eol`-without-`text` and legacy `crlf` forms Git's own `convert_attrs` reads.

use super::*;

/// The config reading:
/// `core.autocrlf` in Git's three readings and `core.eol`.
fn rules_of(entries: &[(&str, Option<&str>)]) -> ConversionRules {
    let owned: Vec<ConfigEntry> = entries
        .iter()
        .map(|(key, value)| return entry_of(key, *value))
        .collect();
    return conversion_rules_from_config(owned.as_slice());
}

fn entry_of(
    key: &str,
    value: Option<&str>,
) -> ConfigEntry {
    return ConfigEntry {
        scope: String::from("local"),
        key: String::from(key),
        value: value.map(|text| return text.as_bytes().to_vec()),
    };
}

/// The attributes one test names,
/// with no legacy `crlf` attribute.
fn attributes_of(
    text: TextAttribute,
    eol: EolAttribute,
) -> PathAttributes {
    return PathAttributes {
        text,
        eol,
        ..PathAttributes::default()
    };
}

/// An ordinary file of 18 bytes whose blob is 18 bytes.
const EQUAL: (u64, Option<u64>) = (18, Some(18));

/// A file of 20 bytes whose blob is 18:
/// a carriage return a rule took out on the way into the index.
const ADDED_CARRIAGE_RETURN: (u64, Option<u64>) = (20, Some(18));

#[test]
fn no_rule_and_no_configuration_carries_the_file() {
    let rules = rules_of(&[]);
    assert_eq!(
        rules.action_of(
            TextAttribute::Unspecified,
            TextAttribute::Unspecified,
            EolAttribute::Unspecified
        ),
        CrlfAction::Binary
    );
    assert_eq!(
        decide(&PathAttributes::default(), &rules, false, EQUAL.0, EQUAL.1),
        FenceVerdict::Carry
    );
    assert!(needs_blob_size(&PathAttributes::default(), &rules));
    assert!(!checkout_converts(&PathAttributes::default(), &rules));
}

#[test]
fn a_path_with_no_rule_whose_file_size_differs_from_its_blob_is_written_by_git() {
    // `core.autocrlf=input` and a CRLF file added (blob LF), then the setting unset: no rule is left, and the file
    // still holds the form the rule Git no longer reads wrote (the gate review's `m04`).
    let rules = rules_of(&[]);
    assert_eq!(
        decide(
            &PathAttributes::default(),
            &rules,
            false,
            ADDED_CARRIAGE_RETURN.0,
            ADDED_CARRIAGE_RETURN.1
        ),
        FenceVerdict::WriteThroughGit(String::from(
            "its recorded size differs from its blob's, so its bytes are not the blob's"
        ))
    );
}

#[test]
fn a_tracked_attributes_file_a_later_commit_deleted_leaves_the_size_to_catch_it() {
    // `* text=auto` when a CRLF file was added, the tracked `.gitattributes` deleted afterwards (the gate review's
    // `m06`): today the path has no rule, and its blob is the LF form the old rule made.
    let rules = rules_of(&[]);
    assert_eq!(
        rules.action_of(
            TextAttribute::Unspecified,
            TextAttribute::Unspecified,
            EolAttribute::Unspecified
        ),
        CrlfAction::Binary
    );
    assert!(matches!(
        decide(
            &PathAttributes::default(),
            &rules,
            false,
            ADDED_CARRIAGE_RETURN.0,
            ADDED_CARRIAGE_RETURN.1
        ),
        FenceVerdict::WriteThroughGit(_)
    ));
}

#[test]
fn text_alone_over_a_lf_platform_never_adds_a_carriage_return_and_needs_the_size() {
    let rules = rules_of(&[]);
    let held = attributes_of(TextAttribute::Set, EolAttribute::Unspecified);
    // `text` alone makes `CRLF_TEXT`, which `convert_attrs` then resolves to the ending the configuration names.
    assert_eq!(
        rules.action_of(held.text, held.crlf, held.eol),
        CrlfAction::TextInput
    );
    assert_eq!(
        decide(&held, &rules, false, 18, Some(18)),
        FenceVerdict::Carry
    );
    assert!(matches!(
        decide(&held, &rules, false, 20, Some(18)),
        FenceVerdict::WriteThroughGit(_)
    ));
    assert!(matches!(
        decide(&held, &rules, false, 18, None),
        FenceVerdict::WriteThroughGit(_)
    ));
    assert!(needs_blob_size(&held, &rules));
}

#[test]
fn text_with_eol_crlf_may_add_carriage_returns_so_git_writes_it() {
    let rules = rules_of(&[]);
    let held = attributes_of(TextAttribute::Set, EolAttribute::Crlf);
    assert_eq!(
        rules.action_of(held.text, held.crlf, held.eol),
        CrlfAction::TextCrlf
    );
    assert_eq!(
        rules.output_ending(CrlfAction::TextCrlf),
        OutputEnding::Crlf
    );
    assert!(!needs_blob_size(&held, &rules));
    assert!(matches!(
        decide(&held, &rules, false, 18, Some(18)),
        FenceVerdict::WriteThroughGit(_)
    ));
}

#[test]
fn eol_crlf_with_no_text_attribute_makes_the_checkout_write_carriage_returns() {
    // `convert_attrs`: `text` is undefined, and the `eol` attribute refines an undefined action into `CRLF_TEXT_CRLF`
    // before `core.autocrlf` is consulted, so `*.txt eol=crlf` alone writes carriage returns.
    let rules = rules_of(&[]);
    let held = attributes_of(TextAttribute::Unspecified, EolAttribute::Crlf);
    assert_eq!(
        rules.action_of(held.text, held.crlf, held.eol),
        CrlfAction::TextCrlf
    );
    assert_eq!(
        rules.output_ending(CrlfAction::TextCrlf),
        OutputEnding::Crlf
    );
    assert!(checkout_converts(&held, &rules));
    assert!(!needs_blob_size(&held, &rules));
    assert!(matches!(
        decide(&held, &rules, false, 18, Some(18)),
        FenceVerdict::WriteThroughGit(_)
    ));
}

#[test]
fn eol_lf_with_no_text_attribute_strips_carriage_returns_so_the_size_decides() {
    // `convert_attrs`: an undefined action with `eol=lf` becomes `CRLF_TEXT_INPUT`, so the checkout writes the blob and
    // the clean side only drops carriage returns, even with no `text` attribute at all.
    let rules = rules_of(&[]);
    let held = attributes_of(TextAttribute::Unspecified, EolAttribute::Lf);
    assert_eq!(
        rules.action_of(held.text, held.crlf, held.eol),
        CrlfAction::TextInput
    );
    assert_eq!(rules.output_ending(CrlfAction::TextInput), OutputEnding::Lf);
    assert!(!checkout_converts(&held, &rules));
    assert!(needs_blob_size(&held, &rules));
    assert_eq!(
        decide(&held, &rules, false, 18, Some(18)),
        FenceVerdict::Carry
    );
    assert!(matches!(
        decide(&held, &rules, false, 20, Some(18)),
        FenceVerdict::WriteThroughGit(_)
    ));
}

#[test]
fn the_legacy_crlf_attribute_is_the_fallback_text_has_in_git() {
    // `git_path_check_crlf` reads the `crlf` attribute with the same function as `text`, and `convert_attrs` consults it
    // only while `text` names no action.
    let rules = rules_of(&[("core.eol", Some("crlf"))]);
    let legacy = PathAttributes {
        crlf: TextAttribute::Set,
        ..PathAttributes::default()
    };
    assert_eq!(
        rules.action_of(legacy.text, legacy.crlf, legacy.eol),
        CrlfAction::TextCrlf
    );
    assert!(matches!(
        decide(&legacy, &rules, false, EQUAL.0, EQUAL.1),
        FenceVerdict::WriteThroughGit(_)
    ));
    // What `text` names still decides first; where the printed token is `unset`, Git may have been given `-text`
    // (binary, and the `crlf` attribute is then never read) or the literal value `unset` (undefined, so `crlf` is), and
    // the fence takes the undefined reading, which sends this path to Git under `core.eol=crlf`.
    let both = PathAttributes {
        text: TextAttribute::Unset,
        crlf: TextAttribute::Set,
        ..PathAttributes::default()
    };
    assert_eq!(
        rules.action_of(both.text, both.crlf, both.eol),
        CrlfAction::TextCrlf
    );
    assert!(matches!(
        decide(&both, &rules, false, EQUAL.0, EQUAL.1),
        FenceVerdict::WriteThroughGit(_)
    ));
    // Its own values are read like `text`'s.
    assert_eq!(
        rules.action_of(
            TextAttribute::Unspecified,
            TextAttribute::Input,
            EolAttribute::Unspecified
        ),
        CrlfAction::TextInput
    );
    assert_eq!(
        rules.action_of(
            TextAttribute::Unspecified,
            TextAttribute::Auto,
            EolAttribute::Crlf
        ),
        CrlfAction::AutoCrlf
    );
    assert_eq!(
        rules.action_of(
            TextAttribute::Unspecified,
            TextAttribute::Unset,
            EolAttribute::Crlf
        ),
        CrlfAction::TextCrlf
    );
}

#[test]
fn a_legacy_crlf_attribute_over_an_added_crlf_file_is_written_by_git() {
    // `*.txt crlf` over a file that was `git add`ed with carriage returns: the blob is the LF form, the file the CRLF
    // one, so the checkout writes the blob and the clean side drops the carriage returns (the gate review's `m03b`).
    let rules = rules_of(&[]);
    let legacy = PathAttributes {
        crlf: TextAttribute::Set,
        ..PathAttributes::default()
    };
    assert_eq!(
        rules.action_of(legacy.text, legacy.crlf, legacy.eol),
        CrlfAction::TextInput
    );
    assert!(matches!(
        decide(
            &legacy,
            &rules,
            false,
            ADDED_CARRIAGE_RETURN.0,
            ADDED_CARRIAGE_RETURN.1
        ),
        FenceVerdict::WriteThroughGit(_)
    ));
}

#[test]
fn text_auto_eol_lf_is_the_identity_on_the_way_out_and_needs_the_size() {
    let rules = rules_of(&[]);
    let held = attributes_of(TextAttribute::Auto, EolAttribute::Lf);
    assert_eq!(
        rules.action_of(held.text, held.crlf, held.eol),
        CrlfAction::AutoInput
    );
    assert_eq!(rules.output_ending(CrlfAction::AutoInput), OutputEnding::Lf);
    assert_eq!(
        decide(&held, &rules, false, 18, Some(18)),
        FenceVerdict::Carry
    );
    assert!(matches!(
        decide(&held, &rules, false, 20, Some(18)),
        FenceVerdict::WriteThroughGit(_)
    ));
}

#[test]
fn text_auto_eol_crlf_may_add_carriage_returns() {
    let held = attributes_of(TextAttribute::Auto, EolAttribute::Crlf);
    let rules = rules_of(&[]);
    assert_eq!(
        rules.action_of(held.text, held.crlf, held.eol),
        CrlfAction::AutoCrlf
    );
    assert!(matches!(
        decide(&held, &rules, false, 18, Some(18)),
        FenceVerdict::WriteThroughGit(_)
    ));
}

#[test]
fn core_autocrlf_true_writes_every_text_path_without_an_attribute() {
    let rules = rules_of(&[("core.autocrlf", Some("true"))]);
    assert_eq!(
        rules.action_of(
            TextAttribute::Unspecified,
            TextAttribute::Unspecified,
            EolAttribute::Unspecified
        ),
        CrlfAction::AutoCrlf
    );
    assert!(matches!(
        decide(&PathAttributes::default(), &rules, false, 18, Some(18)),
        FenceVerdict::WriteThroughGit(_)
    ));
}

#[test]
fn core_autocrlf_input_only_strips_carriage_returns_so_the_size_decides() {
    let rules = rules_of(&[("core.autocrlf", Some("input"))]);
    assert_eq!(
        rules.action_of(
            TextAttribute::Unspecified,
            TextAttribute::Unspecified,
            EolAttribute::Unspecified
        ),
        CrlfAction::AutoInput
    );
    assert_eq!(
        decide(&PathAttributes::default(), &rules, false, 18, Some(18)),
        FenceVerdict::Carry
    );
    assert!(matches!(
        decide(&PathAttributes::default(), &rules, false, 20, Some(18)),
        FenceVerdict::WriteThroughGit(_)
    ));
}

#[test]
fn a_printed_unset_under_core_autocrlf_true_is_written_by_git() {
    // `check-attr`'s `unset` is `-text` or the literal `text=unset`, and Git writes the latter under
    // `core.autocrlf=true`; the fence takes that reading for both, so this one is written by Git.
    let rules = rules_of(&[("core.autocrlf", Some("true"))]);
    let held = attributes_of(TextAttribute::Unset, EolAttribute::Unspecified);
    assert_eq!(
        rules.action_of(held.text, held.crlf, held.eol),
        CrlfAction::AutoCrlf
    );
    assert!(matches!(
        decide(&held, &rules, false, 18, Some(18)),
        FenceVerdict::WriteThroughGit(_)
    ));
}

#[test]
fn core_eol_crlf_gives_text_alone_the_carriage_return_ending() {
    let rules = rules_of(&[("core.eol", Some("crlf"))]);
    assert_eq!(
        rules.action_of(
            TextAttribute::Set,
            TextAttribute::Unspecified,
            EolAttribute::Unspecified
        ),
        CrlfAction::TextCrlf
    );
    assert_eq!(rules.output_ending(CrlfAction::Text), OutputEnding::Crlf);
    assert!(matches!(
        decide(
            &attributes_of(TextAttribute::Set, EolAttribute::Unspecified),
            &rules,
            false,
            18,
            Some(18)
        ),
        FenceVerdict::WriteThroughGit(_)
    ));
}

#[test]
fn ident_filter_and_encoding_are_written_by_git() {
    let rules = rules_of(&[]);
    for held in [
        PathAttributes {
            ident: true,
            ..PathAttributes::default()
        },
        PathAttributes {
            filter: true,
            ..PathAttributes::default()
        },
        PathAttributes {
            encoding: true,
            ..PathAttributes::default()
        },
    ] {
        assert!(
            matches!(
                decide(&held, &rules, false, 18, Some(18)),
                FenceVerdict::WriteThroughGit(_)
            ),
            "{held:?}"
        );
        assert!(!needs_blob_size(&held, &rules), "{held:?}");
        assert!(checkout_converts(&held, &rules), "{held:?}");
    }
}

#[test]
fn a_text_value_git_does_not_name_is_written_by_git() {
    let rules = rules_of(&[]);
    let held = attributes_of(TextAttribute::Other, EolAttribute::Unspecified);
    assert!(matches!(
        decide(&held, &rules, false, 18, Some(18)),
        FenceVerdict::WriteThroughGit(_)
    ));
    assert!(checkout_converts(&held, &rules));
}

#[test]
fn a_link_is_carried_and_never_converted() {
    let rules = rules_of(&[("core.autocrlf", Some("true"))]);
    let held = PathAttributes {
        ident: true,
        ..PathAttributes::default()
    };
    assert_eq!(
        decide(&held, &rules, true, 11, Some(11)),
        FenceVerdict::Carry
    );
}

#[test]
fn a_setting_git_does_not_know_is_read_as_the_careful_one() {
    let rules = rules_of(&[("core.autocrlf", Some("perhaps"))]);
    assert_eq!(rules.auto_crlf, AutoCrlf::True);
    assert_eq!(
        rules_of(&[("core.autocrlf", Some("no"))]).auto_crlf,
        AutoCrlf::False
    );
    assert_eq!(
        rules_of(&[("core.eol", Some("native"))]).core_eol,
        EolSetting::Unset
    );
}

#[test]
fn a_key_written_without_a_value_is_true_as_gits_boolean_parser_reads_it() {
    // `git_parse_maybe_bool_text(NULL)` is 1 (`parse.c`) and `core.autocrlf` goes through `git_config_bool`
    // (`config.c`), so a `[core]` line holding only `autocrlf` reads as true; the fence read it as false and carried
    // CRLF-written files before the gate re-check's `p01`.
    let rules = rules_of(&[("core.autocrlf", None)]);
    assert_eq!(rules.auto_crlf, AutoCrlf::True);
    assert_eq!(
        rules.action_of(
            TextAttribute::Unspecified,
            TextAttribute::Unspecified,
            EolAttribute::Unspecified
        ),
        CrlfAction::AutoCrlf
    );
}

#[test]
fn the_printed_unset_takes_gits_undefined_reading_so_a_converting_configuration_writes_it() {
    // `git check-attr` prints `unset` both for `-text` (`ATTR_FALSE`, which `git_path_check_crlf` reads as
    // `CRLF_BINARY`) and for the literal value `text=unset` (a string it does not know, which it reads as
    // `CRLF_UNDEFINED` and `convert_attrs` then gives to `core.autocrlf`). The fence cannot tell the two apart, so it
    // takes Git's undefined reading: exact for `text=unset`, and only more careful for `-text`.
    let mut held = PathAttributes::default();
    apply_attribute(&mut held, b"text", b"unset");
    assert_eq!(held.text, TextAttribute::Unset);
    let converting = rules_of(&[("core.autocrlf", Some("true"))]);
    assert_eq!(
        rules_of(&[("core.autocrlf", None)]).action_of(held.text, held.crlf, held.eol),
        CrlfAction::AutoCrlf
    );
    assert_eq!(
        converting.action_of(held.text, held.crlf, held.eol),
        CrlfAction::AutoCrlf
    );
    assert_eq!(
        converting.output_ending(converting.action_of(held.text, held.crlf, held.eol)),
        OutputEnding::Crlf
    );
    assert!(matches!(
        decide(&held, &converting, false, 18, Some(18)),
        FenceVerdict::WriteThroughGit(_)
    ));
    // An undefined action is refined by `eol` before `core.autocrlf` is read, exactly as `convert_attrs` does.
    let crlf = PathAttributes {
        eol: EolAttribute::Crlf,
        ..held.clone()
    };
    assert_eq!(
        rules_of(&[]).action_of(crlf.text, crlf.crlf, crlf.eol),
        CrlfAction::TextCrlf
    );
    // Without a converting configuration the token still needs the size proof: `core.autocrlf=input` normalizes on
    // the way in only, and unset is binary.
    assert_eq!(
        rules_of(&[("core.autocrlf", Some("input"))]).action_of(held.text, held.crlf, held.eol),
        CrlfAction::AutoInput
    );
    let quiet = rules_of(&[]);
    assert_eq!(
        quiet.action_of(held.text, held.crlf, held.eol),
        CrlfAction::Binary
    );
    assert_eq!(
        decide(&held, &quiet, false, 18, Some(18)),
        FenceVerdict::Carry
    );
    assert!(matches!(
        decide(&held, &quiet, false, 20, Some(18)),
        FenceVerdict::WriteThroughGit(_)
    ));
}

#[test]
fn the_attribute_triples_of_git_become_the_rules_of_one_path() {
    let mut held = PathAttributes::default();
    apply_attribute(&mut held, b"text", b"auto");
    apply_attribute(&mut held, b"eol", b"crlf");
    apply_attribute(&mut held, b"crlf", b"set");
    apply_attribute(&mut held, b"ident", b"set");
    apply_attribute(&mut held, b"filter", b"lfs");
    apply_attribute(&mut held, b"working-tree-encoding", b"UTF-16LE");
    assert_eq!(
        held,
        PathAttributes {
            text: TextAttribute::Auto,
            crlf: TextAttribute::Set,
            eol: EolAttribute::Crlf,
            filter: true,
            ident: true,
            encoding: true
        }
    );
    let mut reset = PathAttributes {
        text: TextAttribute::Set,
        crlf: TextAttribute::Input,
        eol: EolAttribute::Crlf,
        filter: true,
        ident: true,
        encoding: true,
    };
    apply_attribute(&mut reset, b"text", b"unset");
    apply_attribute(&mut reset, b"eol", b"unspecified");
    apply_attribute(&mut reset, b"crlf", b"unset");
    apply_attribute(&mut reset, b"ident", b"unset");
    apply_attribute(&mut reset, b"filter", b"unset");
    apply_attribute(&mut reset, b"working-tree-encoding", b"unset");
    assert_eq!(
        reset,
        PathAttributes {
            text: TextAttribute::Unset,
            crlf: TextAttribute::Unset,
            eol: EolAttribute::Unspecified,
            filter: false,
            ident: false,
            encoding: false
        }
    );
}
