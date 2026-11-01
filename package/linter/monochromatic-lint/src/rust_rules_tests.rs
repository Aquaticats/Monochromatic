//! What:
//!  Rust-rule behavior and carve-out controls.
//! Why:
//!  Namespace migration must not change documentability,
//!  messages or line-budget boundaries.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('ported Rust rules', () => { /* kind catalog and boundary controls */ });
//! ```

/// Import the production checks,
///  source context and effective severities.
use super::{check_max_lines, check_rustdoc};
use crate::diagnostic::Severity;
use crate::rust_source::RustSource;

/// Build a standalone source context without touching a real file.
fn context(source: &str) -> RustSource {
    return RustSource::new(String::from("fixture.rs"), String::from(source));
}

/// Exact-budget files pass;
///  the first over-budget code line receives the finding.
#[test]
fn max_lines_preserves_boundaries_message_and_severity() {
    let source = context("// comment\nfn main() {\n\n  work();\n}\n");
    assert!(check_max_lines(&source, 3, Severity::Error).is_empty());
    let findings = check_max_lines(&source, 2, Severity::Warn);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "rust/max-lines");
    assert_eq!(findings[0].severity, Severity::Warn);
    assert_eq!(
        findings[0].message,
        "file has 3 code lines, limit is 2 (blank and comment lines excluded)"
    );
    assert_eq!(findings[0].labels[0].span.line, 5);
    assert_eq!(findings[0].filename, "fixture.rs");
    assert_eq!(
        check_max_lines(&source, 0, Severity::Error)[0].labels[0]
            .span
            .line,
        2
    );
}

/// Every listed kind has a concrete source fixture rather than relying on one function case.
#[test]
fn documentation_kind_catalog_is_exercised() {
    let cases = [
        ("fn f() {}", "function \"f\""),
        ("struct S;", "struct \"S\""),
        ("enum E { V }", "enum \"E\""),
        ("union U { x: u32 }", "union \"U\""),
        ("trait T {}", "trait \"T\""),
        ("type Alias = u8;", "type alias \"Alias\""),
        ("const C: u8 = 0;", "constant \"C\""),
        ("static S: u8 = 0;", "static \"S\""),
        ("mod m {}", "module \"m\""),
        // Extern-crate identifiers are NAME_REF nodes; the incumbent reports this item without a name.
        ("extern crate other;", "extern crate"),
        ("use other::Thing;", "use"),
        ("impl S {}", "impl block"),
        ("enum E { V }", "enum variant \"V\""),
        ("struct S { field: u8 }", "field \"field\""),
        ("struct S(u8);", "field"),
    ];
    for (source, label) in cases {
        let input = context(format!("//! file docs\n{source}").as_str());
        let expected = format!("Missing rustdoc on {label}.");
        let findings = check_rustdoc(&input, Severity::Error);
        assert!(
            findings
                .iter()
                .any(|finding| return finding.message == expected),
            "{source}: missing expected {expected}"
        );
    }
    let empty = check_rustdoc(&context(""), Severity::Error);
    assert_eq!(empty.len(), 1);
    assert_eq!(empty[0].message, "Missing rustdoc on file.");
}

/// Rustdoc forms satisfy the rule,
///  while ordinary comments and directive-looking comments do not.
/// The rule itself never reads directives:
///  suppression is a filter applied after the rules,
///  and the rustdoc rule declares itself unsuppressible there.
#[test]
fn only_real_doc_comments_satisfy_documentation() {
    for source in [
        "//! file\n/// function\nfn f() {}",
        "/*! file */\n/** function */\nfn f() {}",
    ] {
        assert!(check_rustdoc(&context(source), Severity::Error).is_empty());
    }
    let findings = check_rustdoc(
        &context("// ordinary\n// monochromatic-lint-disable-next-line -- r\nfn f() {}"),
        Severity::Warn,
    );
    assert_eq!(findings.len(), 2);
    assert!(
        findings
            .iter()
            .all(|finding| return finding.severity == Severity::Warn)
    );
    assert!(
        findings
            .iter()
            .all(|finding| return finding.labels[0].span.line == 3)
    );
}

/// Macro invocations and extern blocks are excluded,
///  but their documentable foreign items remain covered.
#[test]
fn unsatisfiable_macro_and_extern_block_docs_are_not_required() {
    let macros = context("//! file\nmacro_rules! m { () => {} }\nm!();");
    assert!(check_rustdoc(&macros, Severity::Error).is_empty());
    let foreign = context("//! file\nunsafe extern \"C\" { fn foreign(); static VALUE: u8; }");
    let findings = check_rustdoc(&foreign, Severity::Error);
    assert_eq!(findings.len(), 2);
    assert!(
        findings
            .iter()
            .any(|finding| return finding.message == "Missing rustdoc on function \"foreign\".")
    );
    assert!(
        findings
            .iter()
            .any(|finding| return finding.message == "Missing rustdoc on static \"VALUE\".")
    );
}

/// cxx-qt exempts imports and trait-impl members,
///  not inherent methods.
#[test]
fn cxx_qt_carve_out_is_scoped_to_the_established_items() {
    let bridge = context(
        "//! file\nuse cxx_qt_lib::QString;\n/// S\nstruct S;\n/// impl\nimpl Default for S { fn default() -> Self { S } }",
    );
    assert!(check_rustdoc(&bridge, Severity::Error).is_empty());
    let inherent = context(
        "//! file\nuse cxx_qt::Thing;\n/// S\nstruct S;\n/// impl\nimpl S { fn work() {} }",
    );
    let findings = check_rustdoc(&inherent, Severity::Error);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].message, "Missing rustdoc on function \"work\".");
}

/// Identifiers appearing only in strings or comments cannot activate the bridge carve-out.
#[test]
fn comments_and_strings_cannot_enable_cxx_qt_exemptions() {
    for source in [
        "//! cxx_qt\nuse other::Thing;",
        "//! file\n/// text\nconst TEXT: &str = \"cxx_qt_lib\";\nuse other::Thing;",
    ] {
        let findings = check_rustdoc(&context(source), Severity::Error);
        assert!(
            findings
                .iter()
                .any(|finding| return finding.message == "Missing rustdoc on use.")
        );
    }
}

/// Findings start at the declaration or attribute,
///  not at attached ordinary comments.
#[test]
fn declaration_offsets_skip_trivia_but_keep_attributes() {
    let source = context("//! file\n// ordinary\n  #[inline]\nfn f() {}");
    let findings = check_rustdoc(&source, Severity::Error);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].labels[0].span.line, 3);
    assert_eq!(findings[0].labels[0].span.column, 3);
    assert_eq!(findings[0].labels[0].span.length, "#[inline]".len());
}
