//! What:
//! Tests for the parts of the contract that a JSON vectors file cannot express:
//! raw bytes,
//! entry points and printed text.
//! Why:
//! Invalid UTF-8,
//! a byte order mark and the wording of findings are properties of bytes and of `Display`,
//! not of JSON text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // check.unit.test.ts
//! describe('checkManifest', () => { it('refuses invalid UTF-8', () => { /* ... */ }); });
//! ```

/// Imports the public functions and types through the crate root,
/// the same path a consumer uses.
use super::{FindingKind, check_manifest, check_manifest_bytes, check_manifest_path};

/// What:
/// `fn kinds(findings: &[Finding]) -> Vec<&'static str>` maps findings to their kebab-case kind names.
/// `&[Finding]` is a borrowed view of a list.
/// The loop reads each element by reference and `.push(...)` appends its kind name
/// to a growable `Vec`.
/// Why:
/// Assertions then compare short lists of words.
/// A plain loop is used because the crate's clippy
/// configuration denies the implicit returns that short closures would need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kinds = (findings: Finding[]) => findings.map((finding) => finding.kind);
/// ```
fn kinds(findings: &[super::Finding]) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = Vec::new();
    for finding in findings {
        names.push(finding.kind.as_str());
    }
    return names;
}

/// What:
/// `#[test]` marks a test function.
/// `vec![0xFF, 0xFE]` builds a `Vec<u8>` of two bytes that are not valid UTF-8.
/// Why:
/// Invalid UTF-8 is one `invalid-manifest` finding,
/// not a panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('refuses invalid UTF-8', () => { expect(kinds(checkManifestBytes('package.json', new Uint8Array([0xff, 0xfe])))).toEqual(['invalid-manifest']); });
/// ```
#[test]
fn invalid_utf8_is_one_invalid_manifest_finding() {
    let findings = check_manifest_bytes("package.json", &[0xFF, 0xFE]);
    assert_eq!(kinds(&findings), vec!["invalid-manifest"]);
}

/// One leading byte order mark is removed,
/// and a second one is not valid JSON.
#[test]
fn one_byte_order_mark_is_removed_and_a_second_is_invalid() {
    let one: Vec<u8> = [&[0xEF_u8, 0xBB, 0xBF][..], b"{\"dependencies\":{\"a\":\"catalog:\"}}"].concat();
    assert!(check_manifest_bytes("package.json", &one).is_empty());
    let two: Vec<u8> = [&[0xEF_u8, 0xBB, 0xBF, 0xEF, 0xBB, 0xBF][..], b"{}"].concat();
    assert_eq!(kinds(&check_manifest_bytes("package.json", &two)), vec!["invalid-manifest"]);
}

/// An unsupported file name is refused without reading the bytes,
/// even when the bytes are garbage.
#[test]
fn unsupported_name_is_refused_without_reading_the_bytes() {
    let findings = check_manifest("package/a/package.yaml", &[0xFF]);
    assert_eq!(kinds(&findings), vec!["unsupported-manifest-format"]);
    assert!(check_manifest_path("package/a/package.json").is_none());
}

/// A supported name routes to the byte check.
#[test]
fn supported_name_routes_to_the_byte_check() {
    let findings = check_manifest("package/a/package.json", b"{\"dependencies\":{\"zod\":\"^4.6.7\"}}");
    assert_eq!(kinds(&findings), vec!["disallowed-specifier"]);
}

/// What:
/// The printed text names the file,
/// the section,
/// the dependency and the specifier,
/// and states the remedy.
/// `format!("{finding}")` builds a `String` through the `Display` implementation;
/// `.contains("...")` is a substring test.
/// Why:
/// The text is what a person sees in a commit refusal or a warning.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(String(finding)).toContain('package/x/package.json');
/// ```
#[test]
fn display_text_names_file_section_dependency_specifier_and_remedy() {
    let findings = check_manifest_bytes("package/x/package.json", b"{\"devDependencies\":{\"zod\":\"^4.6.7\"}}");
    let text = format!("{}", findings[0]);
    assert!(text.contains("package/x/package.json"), "{text}");
    assert!(text.contains("devDependencies"), "{text}");
    assert!(text.contains("`zod`"), "{text}");
    assert!(text.contains("^4.6.7"), "{text}");
    assert!(text.contains("pnpm-workspace.yaml"), "{text}");
    assert!(text.contains("workspace:*"), "{text}");
}

/// A repeated section and a repeated dependency print different sentences.
#[test]
fn duplicate_findings_print_distinct_sentences() {
    let section = check_manifest_bytes("p.json", b"{\"dependencies\":{},\"dependencies\":{}}");
    assert!(format!("{}", section[0]).contains("appears more than once"));
    let name = check_manifest_bytes("p.json", b"{\"dependencies\":{\"a\":\"catalog:\",\"a\":\"catalog:\"}}");
    assert!(format!("{}", name[0]).contains("lists `a` more than once"));
}

/// An unparseable manifest prints the parser's reason,
/// and an empty document is refused.
#[test]
fn invalid_manifest_text_carries_a_reason() {
    let findings = check_manifest_bytes("p.json", b"");
    assert_eq!(kinds(&findings), vec!["invalid-manifest"]);
    let text = format!("{}", findings[0]);
    assert!(text.contains("not a readable package.json object"), "{text}");
}

/// A dependency name holding a lone surrogate is shown with the replacement character instead of failing.
#[test]
fn lone_surrogate_name_prints_with_the_replacement_character() {
    let findings = check_manifest_bytes("p.json", b"{\"dependencies\":{\"a\\ud800\":\"1.0.0\"}}");
    assert_eq!(kinds(&findings), vec!["disallowed-specifier"]);
    let text = format!("{}", findings[0]);
    assert!(text.contains('\u{FFFD}'), "{text}");
}

/// What:
/// Findings keep source order across sections:
/// members are walked in the order they appear,
/// not in the table's order.
/// `assert_eq!(left, right)` panics when the two values differ and prints both.
/// Why:
/// The contract promises source order,
/// so a consumer can print findings top to bottom.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(sections).toEqual(['peerDependencies', 'dependencies']);
/// ```
#[test]
fn findings_follow_source_order_across_sections() {
    let findings = check_manifest_bytes("p.json", b"{\"peerDependencies\":{\"p\":\"1\"},\"dependencies\":{\"d\":\"1\"}}");
    let mut sections: Vec<&str> = Vec::new();
    for finding in &findings {
        sections.push(finding.section.as_deref().unwrap_or(""));
    }
    assert_eq!(sections, vec!["peerDependencies", "dependencies"]);
}

/// Findings carry the caller's path unchanged and the exact dependency name and specifier.
#[test]
fn findings_carry_path_name_and_specifier() {
    let findings = check_manifest_bytes("package/dsh-plugin/cost-dismissal/package.json", b"{\"dependencies\":{\"zod\":\"^4.6.7\"}}");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].kind, FindingKind::DisallowedSpecifier);
    assert_eq!(findings[0].path, "package/dsh-plugin/cost-dismissal/package.json");
    assert_eq!(findings[0].section.as_deref(), Some("dependencies"));
    assert_eq!(findings[0].name.as_deref(), Some("zod"));
    assert_eq!(findings[0].specifier.as_deref(), Some("^4.6.7"));
}

/// A non-string specifier finding names the path,
/// the section and the dependency,
/// and holds no specifier text.
#[test]
fn non_string_findings_carry_path_section_and_name() {
    let findings = check_manifest_bytes("package/x/package.json", b"{\"optionalDependencies\":{\"a\":[\"catalog:team\"]}}");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].kind, FindingKind::NonStringSpecifier);
    assert_eq!(findings[0].path, "package/x/package.json");
    assert_eq!(findings[0].section.as_deref(), Some("optionalDependencies"));
    assert_eq!(findings[0].name.as_deref(), Some("a"));
    assert_eq!(findings[0].specifier, None);
}

/// A large dependency list with a single bad entry in the middle yields exactly one finding,
/// in linear time.
#[test]
fn a_large_section_with_one_bad_entry_yields_one_finding() {
    let mut text = String::from("{\"dependencies\":{");
    for index in 0..5000 {
        let specifier = if index == 2500 { "^1.0.0" } else { "catalog:" };
        text.push_str(&format!("\"dep-{index}\":\"{specifier}\","));
    }
    text.push_str("\"last\":\"catalog:\"}}");
    let findings = check_manifest_bytes("big.json", text.as_bytes());
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].name.as_deref(), Some("dep-2500"));
}
