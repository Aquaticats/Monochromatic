//! What:
//!  Controls of the toggle words,
//!  the recursion reading and the section text.
//! Why:
//!  Sanoid reads two word lists rather than Perl truth and treats `zfs`
//!  specially,
//!  so a reading that accepted any non-empty value would pass a configuration
//!  sanoid snapshots,
//!  and the text a correction prints must be the text the reading reads back.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(recursionOf('yes')).toEqual({ kind: 'yes' });
//! expect(renderSection(section)).toBe('[tank/home]\n  autosnap = no\n');
//! ```

use super::{AUTOSNAP_SETTING, FALSE_WORDS, RECURSIVE_SETTING, Recursion, SanoidSection, SanoidSetting, TEMPLATE_SETTING, TRUE_WORDS, is_false_word, is_true_word, recursion_of, render_section, render_sections};

/// A section of one dataset and two settings.
fn a_section() -> SanoidSection {
    let settings: Vec<SanoidSetting> = vec![
        SanoidSetting { key: String::from(TEMPLATE_SETTING), value: String::from("home") },
        SanoidSetting { key: String::from(RECURSIVE_SETTING), value: String::from("yes") },
    ];
    return SanoidSection::new("tank/home", settings);
}

/// Every true word is true and none of them is false.
#[test]
fn the_true_words_are_true_and_not_false() {
    for word in TRUE_WORDS {
        assert!(is_true_word(word), "{word} is true");
        assert!(!is_false_word(word), "{word} is not false");
    }
}

/// Every false word is false and none of them is true.
#[test]
fn the_false_words_are_false_and_not_true() {
    for word in FALSE_WORDS {
        assert!(is_false_word(word), "{word} is false");
        assert!(!is_true_word(word), "{word} is not true");
    }
}

/// A word neither list holds is neither,
/// as `autosnap = no # keep` is.
#[test]
fn a_word_neither_list_holds_is_neither() {
    assert!(!is_true_word("no # keep"));
    assert!(!is_false_word("no # keep"));
    assert!(!is_true_word("maybe"));
    assert!(!is_false_word("maybe"));
}

/// A section with no recursion value is off with no value.
#[test]
fn no_recursion_value_reads_off_with_nothing() {
    assert_eq!(recursion_of(None), Recursion::Off { value: None });
}

/// Every true word reads as recursion.
#[test]
fn every_true_word_reads_as_recursion() {
    for word in TRUE_WORDS {
        assert_eq!(recursion_of(Some(word)), Recursion::Yes, "{word} recurses");
    }
}

/// A value naming `zfs` reads as zfs recursion,
/// whatever its case and wherever the word stands.
#[test]
fn a_value_naming_zfs_reads_as_zfs_recursion() {
    for written in [ "zfs", "ZFS", "zfs # one recursive snapshot" ] {
        assert_eq!(recursion_of(Some(written)), Recursion::Zfs, "{written} is zfs recursion");
    }
}

/// A value sanoid does not read as true is off and keeps what was written.
#[test]
fn a_value_that_is_not_a_true_word_reads_off_with_the_value() {
    assert_eq!(recursion_of(Some("no")), Recursion::Off { value: Some(String::from("no")) });
    assert_eq!(recursion_of(Some("yess")), Recursion::Off { value: Some(String::from("yess")) });
}

/// One section renders as its header and its settings with a two-space indent.
#[test]
fn one_section_renders_as_the_text_a_person_writes() {
    assert_eq!(render_section(&a_section()), "[tank/home]\n  use_template = home\n  recursive = yes\n");
}

/// Several sections render in order with a blank line between them.
#[test]
fn several_sections_render_in_order_between_blank_lines() {
    let cache: SanoidSection = SanoidSection::new("tank/varcache", vec![SanoidSetting { key: String::from(AUTOSNAP_SETTING), value: String::from("no") }]);
    assert_eq!(render_sections(&[ a_section(), cache ]), "[tank/home]\n  use_template = home\n  recursive = yes\n\n[tank/varcache]\n  autosnap = no\n");
}

/// A section's value is the first it writes for the key,
/// and nothing for a key it does not write.
#[test]
fn a_sections_value_is_the_first_it_writes_for_the_key() {
    let section: SanoidSection = a_section();
    assert_eq!(section.value_of(RECURSIVE_SETTING), Some("yes"));
    assert_eq!(section.value_of(AUTOSNAP_SETTING), None);
}
