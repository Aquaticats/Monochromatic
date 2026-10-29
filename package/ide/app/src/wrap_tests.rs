//! Unit tests of each IntelliJ break rule and the continuation indent,
//! one predicate at a time.

use super::*;

/// Blank rule:
/// a space or tab on either side of the pair gives an opportunity.
#[test]
fn blank_rule_breaks_on_both_sides_of_a_space_or_tab() {
    assert!(blank_break('a', ' '));
    assert!(blank_break(' ', 'a'));
    assert!(blank_break('a', '\t'));
    assert!(blank_break('\t', 'a'));
    assert!(!blank_break('a', 'b'));
    assert!(!blank_break('a', '\u{00A0}'));
}

/// Closing rule:
/// the break follows `,`,
/// `;`,
/// or `)` and nothing else.
#[test]
fn closing_rule_breaks_after_comma_semicolon_and_close_paren() {
    assert!(closing_symbol_break(',', 'a'));
    assert!(closing_symbol_break(';', 'a'));
    assert!(closing_symbol_break(')', 'a'));
    assert!(!closing_symbol_break('(', 'a'));
    assert!(!closing_symbol_break('a', ','));
}

/// Opening rule:
/// the break precedes `(` or `.`.
#[test]
fn opening_rule_breaks_before_open_paren_and_dot() {
    assert!(opening_symbol_break('a', '('));
    assert!(opening_symbol_break('a', '.'));
    assert!(!opening_symbol_break('(', 'a'));
    assert!(!opening_symbol_break('a', '['));
}

/// Keeps-blank rule:
/// a comma or semicolon with a following space takes no break before the space.
#[test]
fn blank_after_closing_symbol_stays_on_the_upper_row() {
    assert!(keeps_blank_with_closing_symbol(',', ' '));
    assert!(keeps_blank_with_closing_symbol(';', ' '));
    assert!(keeps_blank_with_closing_symbol(')', ' '));
    assert!(!keeps_blank_with_closing_symbol(',', '\t'));
    assert!(!keeps_blank_with_closing_symbol('a', ' '));
}

/// Eastern rule:
/// a BMP code point from U+2F00 on breaks on both sides,
/// and nothing outside that range does.
#[test]
fn eastern_rule_covers_bmp_code_points_from_u2f00() {
    assert!(is_eastern('\u{2F00}'));
    assert!(is_eastern('\u{4E2D}'));
    assert!(is_eastern('\u{FFFF}'));
    assert!(!is_eastern('\u{2EFF}'));
    assert!(!is_eastern('\u{10000}'));
    assert!(eastern_break('a', '\u{4E2D}'));
    assert!(eastern_break('\u{4E2D}', 'a'));
    assert!(!eastern_break('a', 'b'));
}

/// Identifier rule:
/// a break before a non-identifier symbol that follows an identifier character.
#[test]
fn identifier_rule_breaks_before_a_symbol_after_an_identifier() {
    assert!(identifier_symbol_break('a', '+'));
    assert!(identifier_symbol_break('7', '='));
    assert!(!identifier_symbol_break('+', '+'));
    assert!(!identifier_symbol_break('a', 'b'));
    assert!(!identifier_symbol_break('a', '('));
    assert!(!identifier_symbol_break('a', ','));
}

/// Grapheme guard:
/// no opportunity in front of an extender,
/// after a zero width joiner,
/// or inside a flag.
#[test]
fn grapheme_guard_blocks_marks_joiners_and_flags() {
    assert!(joins_grapheme('e', '\u{0301}'));
    assert!(joins_grapheme('\u{1F469}', '\u{1F3FB}'));
    assert!(joins_grapheme('\u{1F469}', '\u{200D}'));
    assert!(joins_grapheme('\u{200D}', '\u{1F4BB}'));
    assert!(joins_grapheme('\u{1F1EF}', '\u{1F1F5}'));
    assert!(joins_grapheme('\u{3053}', '\u{3099}'));
    assert!(joins_grapheme('\u{1100}', '\u{1161}'));
    assert!(!joins_grapheme('a', 'b'));
    assert!(!joins_grapheme('\u{1F1EF}', 'a'));
}

/// The composed decision:
/// the grapheme guard overrides every rule that would otherwise break.
#[test]
fn grapheme_guard_wins_over_the_other_rules() {
    assert!(!intellij_break('a', '\u{0301}'));
    assert!(!intellij_break(' ', '\u{0301}'));
    assert!(!intellij_break('\u{4E2D}', '\u{FE0F}'));
    assert!(!intellij_break('7', '\u{FE0F}'));
    assert!(!intellij_break('\u{200D}', ' '));
}

/// The composed decision:
/// the blank rule beats the closing rule,
/// except that the blank stays put after a symbol.
#[test]
fn composed_decision_keeps_the_blank_run_together_after_a_comma() {
    assert!(!intellij_break(',', ' '));
    assert!(intellij_break(' ', 'a'));
    assert!(intellij_break(',', 'a'));
    assert!(intellij_break('a', '('));
    assert!(intellij_break('a', '+'));
}

/// The composed decision:
/// a pair no rule names gets no opportunity,
/// even where Unicode would allow one.
#[test]
fn pairs_without_a_rule_get_no_opportunity() {
    assert!(!intellij_break('a', 'b'));
    assert!(!intellij_break('-', 'b'));
    assert!(!intellij_break('/', '/'));
    assert!(!intellij_break('\u{1F600}', '\u{1F600}'));
}

/// Wrap width rule:
/// only a positive finite width wraps.
#[test]
fn wrap_width_must_be_positive_and_finite() {
    assert!(has_wrap_width(0.5));
    assert!(!has_wrap_width(0.0));
    assert!(!has_wrap_width(-1.0));
    assert!(!has_wrap_width(f32::NAN));
    assert!(!has_wrap_width(f32::INFINITY));
}

/// Continuation indent:
/// an empty or whitespace-only line gets exactly the wrap mark.
#[test]
fn continuation_indent_of_blank_lines_is_one_mark() {
    assert_eq!(continuation_indent("", 8.0), 8.0);
    assert_eq!(continuation_indent("   ", 8.0), 8.0);
    assert_eq!(continuation_indent("\t\t", 8.0), 8.0);
}

/// Continuation indent:
/// spaces count one advance each,
/// plus the mark.
#[test]
fn continuation_indent_counts_leading_spaces() {
    assert_eq!(continuation_indent("abc", 8.0), 8.0);
    assert_eq!(continuation_indent("    x", 8.0), 40.0);
}

/// Continuation indent:
/// a leading tab ends on its stop,
/// which is two spaces wide,
/// so it counts 16 at a space of 8.
#[test]
fn continuation_indent_counts_leading_tabs_at_their_stops() {
    assert_eq!(continuation_indent("\tx", 8.0), 24.0);
    assert_eq!(continuation_indent("  \tx", 8.0), 40.0);
    assert_eq!(continuation_indent("\t\tx", 8.0), 40.0);
}

/// Continuation indent:
/// a tab after text in the leading part is measured from the tab's own start.
#[test]
fn continuation_indent_of_a_tab_at_an_odd_column_runs_to_the_next_stop() {
    assert_eq!(continuation_indent(" \tx", 8.0), 24.0);
}
