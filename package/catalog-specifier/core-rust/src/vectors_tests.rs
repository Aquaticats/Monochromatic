//! What:
//! Runs every case in `fixtures/vectors.json` against the public functions.
//! Why:
//! The vectors file is the contract's executable form:
//! any consumer in another language reads the same file,
//! so a case added
//! there is enforced here without touching Rust.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // vectors.unit.test.ts
//! const vectors = JSON.parse(readFileSync('fixtures/vectors.json', 'utf8'));
//! for (const text of vectors.install_specifiers.accepted) it(`accepts ${text}`, () => { /* ... */ });
//! ```

/// What:
/// Imports `serde_json::Value`,
/// a type that holds any parsed JSON (object,
/// array,
/// string,
/// number,
/// boolean or null).
/// Why:
/// The vectors file has several shapes,
/// so it is read as a loose tree and walked by key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // JSON.parse already returns `unknown`
/// ```
use serde_json::Value;

/// Imports the functions and types under test through the crate root.
use super::{FindingKind, SectionKind, check_manifest_bytes, check_manifest_path, is_allowed_specifier};

/// What:
/// `fn vectors() -> Value` loads and parses the vectors file.
/// `include_str!("...")` is a compile-time macro that pastes the
/// file's text into the program,
/// with the path relative to this source file.
/// `.expect("...")` unwraps an `Ok` and panics with
/// the message on an `Err`.
/// Why:
/// The fixture ships inside the test binary,
/// so the test needs no run-time file path.
/// Gotcha:
/// `.expect` panics on failure;
/// that is the right behavior in a test,
/// and wrong in library code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const vectors = (): unknown => JSON.parse(readFileSync(new URL('../fixtures/vectors.json', import.meta.url), 'utf8'));
/// ```
fn vectors() -> Value {
    return serde_json::from_str(include_str!("../fixtures/vectors.json")).expect("fixtures/vectors.json is valid JSON");
}

/// What:
/// `fn texts(vectors: &Value, group: &str, side: &str) -> Vec<String>` reads `vectors[group][side]` as a list of strings.
/// `.as_array()` returns `Option<&Vec<Value>>`;
/// `.expect(...)` unwraps it.
/// The loop walks the elements by reference
/// (`&Vec` lends the list without moving it) and `.push(...)` appends each converted string to a growable `Vec`.
/// Why:
/// Both specifier groups have the same accepted and rejected lists.
/// A plain loop is used because the crate's clippy
/// configuration denies the implicit returns that short closures would need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = (vectors: any, group: string, side: string): string[] => vectors[group][side];
/// ```
fn texts(vectors: &Value, group: &str, side: &str) -> Vec<String> {
    let list = vectors[group][side].as_array().expect("the vector list is an array");
    let mut collected: Vec<String> = Vec::new();
    for item in list {
        collected.push(item.as_str().expect("each vector is a string").to_string());
    }
    return collected;
}

/// What:
/// `#[test]` marks a test function.
/// This one runs the install-section vectors.
/// Why:
/// Both sides of the list must hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('install specifier vectors', () => { /* ... */ });
/// ```
#[test]
fn install_specifier_vectors_hold() {
    let all = vectors();
    for accepted in texts(&all, "install_specifiers", "accepted") {
        assert!(is_allowed_specifier(SectionKind::Install, &accepted), "install refused {accepted:?}");
    }
    for rejected in texts(&all, "install_specifiers", "rejected") {
        assert!(!is_allowed_specifier(SectionKind::Install, &rejected), "install accepted {rejected:?}");
    }
}

/// Runs the peer-section vectors.
#[test]
fn peer_specifier_vectors_hold() {
    let all = vectors();
    for accepted in texts(&all, "peer_specifiers", "accepted") {
        assert!(is_allowed_specifier(SectionKind::Peer, &accepted), "peer refused {accepted:?}");
    }
    for rejected in texts(&all, "peer_specifiers", "rejected") {
        assert!(!is_allowed_specifier(SectionKind::Peer, &rejected), "peer accepted {rejected:?}");
    }
}

/// Runs the path vectors:
/// unsupported names yield the format finding and supported names yield nothing.
#[test]
fn path_vectors_hold() {
    let all = vectors();
    for path in texts(&all, "paths", "unsupported") {
        let Some(finding) = check_manifest_path(&path) else {
            panic!("{path:?} was not refused");
        };
        assert_eq!(finding.kind, FindingKind::UnsupportedManifestFormat, "wrong kind for {path:?}");
    }
    for path in texts(&all, "paths", "supported") {
        assert!(check_manifest_path(&path).is_none(), "{path:?} was refused");
    }
}

/// What:
/// Runs the manifest vectors:
/// each case's `text` is checked as bytes and the finding kinds must equal `kinds` in order.
/// `.as_bytes()` borrows a `String`'s UTF-8 bytes as `&[u8]`.
/// `.as_str()` gives the kind's stable kebab-case name.
/// Why:
/// Order is part of the contract (source order,
/// duplicate before the entry's own result).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const c of vectors.manifests) expect(checkManifestBytes('package.json', encode(c.text)).map(f => f.kind)).toEqual(c.kinds);
/// ```
#[test]
fn manifest_vectors_hold() {
    let all = vectors();
    let cases = all["manifests"].as_array().expect("manifests is an array");
    for case in cases {
        let name = case["name"].as_str().expect("each case has a name");
        let text = case["text"].as_str().expect("each case has text");
        let mut expected: Vec<String> = Vec::new();
        for kind in case["kinds"].as_array().expect("each case lists kinds") {
            expected.push(kind.as_str().expect("each kind is a string").to_string());
        }
        let mut actual: Vec<String> = Vec::new();
        for finding in check_manifest_bytes("package/x/package.json", text.as_bytes()) {
            actual.push(finding.kind.as_str().to_string());
        }
        assert_eq!(actual, expected, "case {name:?}");
    }
}
