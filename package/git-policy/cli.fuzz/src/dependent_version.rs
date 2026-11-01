//! What: Generators and invariants for the dependent-version planner's two text scanners:
//!       the manifest version rewrite and the module-specifier scan.
//! Why: Both read repository content byte by byte. Generated manifests come with the exact
//!      text the rewrite must produce, built beside the input rather than by the subject;
//!      raw text checks the round trip and, where the manifest reader accepts the text, that
//!      a plainly spelled version is always found. The specifier scan is compared with an
//!      independent restatement of its rule that uses no iterator adaptors.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { before, after, from, to } = generatedManifest(data); expect(replaceManifestVersion({ text: before, from, to })).toBe(after);
//! ```

/// Import the scanners under test, the manifest reader and the JSONC parser.
use git_policy_cli::dependent_version_content::PolicyIncomplete;
use git_policy_cli::dependent_version_imports::imports_package;
use git_policy_cli::dependent_version_manifest::read_manifest_facts;
use git_policy_cli::dependent_version_release::json_quote_units;
use git_policy_cli::dependent_version_text::replace_manifest_version;
/// The JSONC parser reads raw key and value spellings for the completeness invariant.
use monochromatic_jsonc_edit::{JsoncKind, parse_jsonc};

/// The path the rewrite names in its messages.
const PATH: &[u8] = b"package/module/a/package.json";

/// What: A generated manifest: its text, the text after the rewrite, and the versions.
/// Why:  `after` is assembled from the same parts as `before`, so it is an expectation
///       the rewrite did not compute.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GeneratedManifest = { before: string; after: string; from: string; to: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedManifest {
    /// The manifest text.
    pub before: String,
    /// The same text with the top-level version replaced.
    pub after: String,
    /// The version in `before`.
    pub from: String,
    /// The version in `after`.
    pub to: String,
}

/// Members that may precede or follow the version, including look-alikes of its key.
const DECOYS: &[&str] = &[
    "\"name\":\"@s/a\"",
    "\"publishConfig\":{\"version\":\"9.9.9\"}",
    "\"tail\":[\"version\",{\"version\":\"0.0.0\"}]",
    "\"description\":\"version \\\"x\\\" /* , } ]\"",
    "\"q\":\"say \\\"hi\"",
    "\"versions\":\"1\"",
    "\"a\":\"version\"",
    "\"x\":\"\\\\\"",
    "\"d\":{\"e\":[[],{}]}",
    "\"u\":\"\u{e9}\u{1f600}\"",
];

/// Whitespace JSON allows between tokens.
const SPACES: &[&str] = &["", " ", "\n  ", "\r\n\t", "  "];

/// What: Build a manifest from input bytes: decoys before and after a top-level version, in
///       a byte-chosen layout. `.get(i)` reads a byte or nothing past the end.
/// Why:  Every layout has exactly one top-level `"version"` key, so the rewrite must
///       succeed and must produce `after`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedManifest(data: Uint8Array): GeneratedManifest;
/// ```
pub fn generated_manifest(data: &[u8]) -> GeneratedManifest {
    let byte = |index: usize| return usize::from(data.get(index).copied().unwrap_or(0));
    let space: &str = SPACES[byte(0) % SPACES.len()];
    let colon: String = format!("{space}:{}", SPACES[byte(1) % SPACES.len()]);
    let from: String = format!("{}.{}.{}", byte(2) % 3, byte(3), byte(4) * 37);
    let to: String = format!("{}.{}.{}", byte(5) % 3, byte(6), byte(4) * 37 + 1);
    let mut before_members: Vec<String> = Vec::new();
    for offset in 0..byte(7) % 4 {
        before_members.push(String::from(DECOYS[byte(8 + offset) % DECOYS.len()]));
    }
    let mut after_members: Vec<String> = Vec::new();
    for offset in 0..byte(12) % 4 {
        after_members.push(String::from(DECOYS[byte(13 + offset) % DECOYS.len()]));
    }
    let assemble = |version: &str| {
        let mut members: Vec<String> = before_members.clone();
        members.push(format!("\"version\"{colon}\"{version}\""));
        members.extend(after_members.iter().cloned());
        return format!(
            "{{{space}{}{space}}}{}",
            members.join(&format!(",{space}")),
            SPACES[byte(17) % SPACES.len()]
        );
    };
    return GeneratedManifest {
        before: assemble(&from),
        after: assemble(&to),
        from,
        to,
    };
}

/// What: The rewrite of a generated manifest is exactly its expectation, and the manifest
///       reader then sees the new version.
/// Why:  This is the differential check of the rewrite against an independent builder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(replaceManifestVersion({ path, text: before, from, to })).toBe(after);
/// ```
pub fn check_generated_manifest(generated: &GeneratedManifest) {
    let rewritten: Result<String, PolicyIncomplete> =
        replace_manifest_version(PATH, &generated.before, &generated.from, &generated.to);
    assert_eq!(rewritten.as_ref(), Ok(&generated.after), "{generated:?}");
    check_version_edit(&generated.before, &generated.from, &generated.to);
}

/// What: Split raw input into `from`, `to` and the text at the first two NUL bytes, with
///       default versions when there are fewer parts. Text that is not UTF-8 is replaced.
/// Why:  Raw input reaches every scanner state, including unterminated literals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function versionEditView(data: Uint8Array): [from: string, to: string, text: string];
/// ```
pub fn version_edit_view(data: &[u8]) -> (String, String, String) {
    let parts: Vec<&[u8]> = data.splitn(3, |byte: &u8| return *byte == 0).collect();
    let lossy = |bytes: &[u8]| return String::from_utf8_lossy(bytes).into_owned();
    match parts.as_slice() {
        [from, to, text] => return (lossy(from), lossy(to), lossy(text)),
        _ => return (String::from("1.0.0"), String::from("1.0.1"), lossy(data)),
    }
}

/// What: The rewrite's invariants on any text: a success replaces exactly one occurrence of
///       the quoted `from` and the reverse rewrite restores the text; a failure is a shape
///       problem; a parsable manifest with one plainly spelled top-level version is found.
/// Why:  Together these say that only the version's literal changes, that it is the first
///       top-level one, and that nothing parsable is refused for its layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkVersionEdit(text: string, from: string, to: string): void;
/// ```
pub fn check_version_edit(text: &str, from: &str, to: &str) -> Option<bool> {
    let quote = |value: &str| return json_quote_units(&value.encode_utf16().collect::<Vec<u16>>());
    let (quoted_from, quoted_to): (String, String) = (quote(from), quote(to));
    let outcome: Result<String, PolicyIncomplete> = replace_manifest_version(PATH, text, from, to);
    match &outcome {
        Ok(rewritten) => {
            let replaced: bool = (0..=text.len()).any(|start| {
                return text.is_char_boundary(start)
                    && text[start..].starts_with(&quoted_from)
                    && *rewritten
                        == format!(
                            "{}{quoted_to}{}",
                            &text[..start],
                            &text[start + quoted_from.len()..]
                        );
            });
            assert!(replaced, "{text:?} became {rewritten:?}");
            assert_eq!(
                replace_manifest_version(PATH, rewritten, to, from).as_deref(),
                Ok(text)
            );
        }
        Err(error) => assert!(
            matches!(error, PolicyIncomplete::ManifestShape { .. }),
            "{error:?}"
        ),
    }
    let Ok(facts) = read_manifest_facts(PATH, text) else {
        return None;
    };
    let Ok(parsed) = parse_jsonc(text) else {
        return None;
    };
    let versions: Vec<_> = parsed
        .entries()
        .unwrap_or(&[])
        .iter()
        .filter(|entry| return entry.key.units.iter().copied().eq("version".encode_utf16()))
        .collect();
    let plain: bool = match versions.as_slice() {
        [only] => {
            matches!(&only.value.kind, JsoncKind::Text { raw, units } if only.key.raw == "\"version\"" && *raw == json_quote_units(units) && String::from_utf16(units).is_ok_and(|version: String| return version == from))
        }
        _ => false,
    };
    if plain {
        let rewritten: String =
            outcome.unwrap_or_else(|error| panic!("{text:?} was refused: {error:?}"));
        let after =
            read_manifest_facts(PATH, &rewritten).unwrap_or_else(|error| panic!("{error:?}"));
        assert_eq!(after.version, Some(to.encode_utf16().collect()));
        assert_eq!(
            (
                after.name,
                after.runtime_dependency_names,
                after.dev_dependency_names
            ),
            (
                facts.name,
                facts.runtime_dependency_names,
                facts.dev_dependency_names
            )
        );
    }
    return Some(plain);
}

/// What: Whether the bytes before `end`, after trailing JSON whitespace, end with a whole
///       keyword. Written with index loops, independently of the subject.
/// Why:  The reference rule for specifier position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keywordBefore(text: Uint8Array, end: number, keywords: string[]): boolean;
/// ```
fn keyword_before(text: &[u8], end: usize, keywords: &[&str]) -> bool {
    let mut stop: usize = end;
    while stop > 0 && matches!(text[stop - 1], b' ' | b'\t' | b'\r' | b'\n') {
        stop -= 1;
    }
    for keyword in keywords {
        let length: usize = keyword.len();
        if stop >= length && &text[stop - length..stop] == keyword.as_bytes() {
            let boundary: bool = stop == length
                || !(text[stop - length - 1].is_ascii_alphanumeric()
                    || matches!(text[stop - length - 1], b'_' | b'$'));
            if boundary {
                return true;
            }
        }
    }
    return false;
}

/// What: The reference specifier scan: visit each non-overlapping occurrence from the left
///       (every character boundary for an empty name) and apply the quote and position rules.
/// Why:  An independent restatement to compare the subject with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function referenceImports(source: string, name: string): boolean;
/// ```
pub fn reference_imports(source: &str, name: &str) -> bool {
    let text: &[u8] = source.as_bytes();
    let needle: &[u8] = name.as_bytes();
    let mut start: usize = 0;
    while start <= text.len() {
        if !(source.is_char_boundary(start) && text[start..].starts_with(needle)) {
            start += 1;
            continue;
        }
        if start > 0 && matches!(text[start - 1], b'\'' | b'"' | b'`') {
            let quote: u8 = text[start - 1];
            let following: Option<u8> = text.get(start + needle.len()).copied();
            let keyword: bool = keyword_before(text, start - 1, &["from", "import"]);
            let mut paren: usize = start - 1;
            while paren > 0 && matches!(text[paren - 1], b' ' | b'\t' | b'\r' | b'\n') {
                paren -= 1;
            }
            let call: bool = paren > 0
                && text[paren - 1] == b'('
                && keyword_before(text, paren - 1, &["import", "require"]);
            if (following == Some(quote) || following == Some(b'/')) && (keyword || call) {
                return true;
            }
        }
        start += needle.len().max(1);
    }
    return false;
}

/// What: Split raw input into a package name and source text at the first NUL byte.
/// Why:  Raw names include empty and odd ones; raw text includes every byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function importView(data: Uint8Array): [name: string, source: string];
/// ```
pub fn import_view(data: &[u8]) -> (String, String) {
    let mut parts = data.splitn(2, |byte: &u8| return *byte == 0);
    let name: String = String::from_utf8_lossy(parts.next().unwrap_or(&[])).into_owned();
    let source: String = String::from_utf8_lossy(parts.next().unwrap_or(&[])).into_owned();
    return (name, source);
}

/// What: The subject's scan agrees with the reference on any name and text.
/// Why:  The differential check of the specifier scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(importsPackage({ sourceText, packageName })).toBe(referenceImports(sourceText, packageName));
/// ```
pub fn check_imports(source: &str, name: &str) -> bool {
    let found: bool = imports_package(source, name);
    assert_eq!(
        found,
        reference_imports(source, name),
        "{name:?} in {source:?}"
    );
    return found;
}

/// Controls proving both scanners' invariants are reached and can fail.
#[cfg(test)]
#[path = "dependent_version_tests.rs"]
mod tests;
