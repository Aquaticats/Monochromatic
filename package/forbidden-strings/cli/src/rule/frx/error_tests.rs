// Unit tests for the redacted load-error rendering.

use super::LoadError;
use forbidden_regex::CompileError;

#[test]
fn no_rules_renders_fixed_text() {
    assert_eq!(LoadError::NoRules.to_string(), "no rules loaded");
}

#[test]
fn unsupported_flag_names_index_and_flag() {
    let rendered = LoadError::UnsupportedFlag { index: 3, flag: 'i' }.to_string();
    assert!(rendered.contains("rule 3"), "{rendered}");
    assert!(rendered.contains('i'), "{rendered}");
    assert!(rendered.contains("only 'm' and 'x'"), "{rendered}");
}

#[test]
fn compile_renders_index_plus_engine_reason() {
    let reason = CompileError::Syntax {
        pos: 7,
        message: "'*' is unsupported; use {1,n}".to_string(),
    };
    let rendered = LoadError::Compile { index: 2, reason }.to_string();
    assert!(rendered.starts_with("rule 2: "), "{rendered}");
    // The engine's static Display is surfaced verbatim (no pattern text in it).
    assert!(rendered.contains("'*' is unsupported"), "{rendered}");
}

#[test]
fn precompiled_renders_engine_reason() {
    let reason = CompileError::Invalid {
        message: "unexpected end of input".to_string(),
    };
    let rendered = LoadError::Precompiled { reason }.to_string();
    assert!(rendered.contains("precompiled ruleset failed to load"), "{rendered}");
    assert!(rendered.contains("unexpected end of input"), "{rendered}");
}

#[test]
fn near_header_renders_line_number_only() {
    let rendered = LoadError::NearHeader { line: 7 }.to_string();
    assert!(rendered.contains("line 7"), "{rendered}");
    assert!(rendered.contains("section header"), "{rendered}");
}

#[test]
fn pre_header_content_renders_line_number() {
    let rendered = LoadError::PreHeaderContent { line: 2 }.to_string();
    assert!(rendered.contains("line 2"), "{rendered}");
    assert!(rendered.contains("before the first section header"), "{rendered}");
}

#[test]
fn empty_section_renders_line_number() {
    let rendered = LoadError::EmptySection { line: 4 }.to_string();
    assert!(rendered.contains("line 4"), "{rendered}");
    assert!(rendered.contains("no rule body"), "{rendered}");
}

#[test]
fn duplicate_name_renders_both_line_numbers() {
    let rendered = LoadError::DuplicateName { first_line: 1, line: 9 }.to_string();
    assert!(rendered.contains("line 9"), "{rendered}");
    assert!(rendered.contains("line 1"), "{rendered}");
    assert!(rendered.contains("duplicate section name"), "{rendered}");
}

#[test]
fn tail_variants_debug_holds_no_free_text_field() {
    // Every tail-format variant carries only line-number positions, so the derived
    // Debug is leak-safe by construction like the rest of the type.
    let debugged = format!("{:?}", LoadError::DuplicateName { first_line: 3, line: 8 });
    assert!(debugged.contains('3') && debugged.contains('8'), "{debugged}");
}

#[test]
fn debug_is_derived_and_holds_no_free_text_field() {
    // LoadError carries only an index, a flag char, and the engine's CompileError;
    // none is rule text, so the derived Debug is leak-safe by construction.
    let err = LoadError::Compile {
        index: 0,
        reason: CompileError::EmptyMatchable,
    };
    let debugged = format!("{err:?}");
    assert!(debugged.contains("EmptyMatchable"), "{debugged}");
}
