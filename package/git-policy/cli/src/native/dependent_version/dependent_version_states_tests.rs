//! What: Manifest reading order, raised-version detection and duplicate names.
//! Why: Which failure a broken workspace reports, and which packages count as raised,
//!      must follow the incumbent.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(bumpedNames(states)).toEqual(['@s/base']);
//! ```

/// The reader, the decisions and the in-memory workspace.
use super::{ManifestState, bumped_names, duplicate_name, read_manifest_states};
use crate::dependent_version_content::{
    PlanError, PolicyIncomplete, TrackedMode, TrackedPath, WorkspaceContent,
};
use crate::dependent_version_manifest::shape;
use crate::dependent_version_test_support::{MemoryWorkspace, Read, changed, unchanged, workspace};

/// Read every state of a workspace.
fn states(memory: &mut MemoryWorkspace) -> Result<Vec<ManifestState>, PlanError> {
    let tracked: Vec<TrackedPath> = memory.tracked_paths().unwrap_or_default();
    return read_manifest_states(memory, &tracked);
}

/// Read states that must succeed.
fn read_ok(memory: &mut MemoryWorkspace) -> Vec<ManifestState> {
    return states(memory).unwrap_or_else(|error: PlanError| panic!("{error:?}"));
}

/// Only manifests are read: candidate then base, in listing order.
#[test]
fn reads_only_manifests_in_listing_order() {
    let mut memory: MemoryWorkspace = workspace(vec![
        unchanged("package/m/b/package.json", r#"{"name":"b"}"#),
        unchanged("package/m/b/src/index.ts", "x"),
        unchanged("README.md", "x"),
        changed("package/m/a/package.json", r#"{"name":"a"}"#, None),
    ]);
    let read: Vec<ManifestState> = read_ok(&mut memory);
    assert_eq!(
        memory.reads,
        vec![
            Read::Listing,
            Read::Candidate(b"package/m/b/package.json".to_vec()),
            Read::Base(b"package/m/b/package.json".to_vec()),
            Read::Candidate(b"package/m/a/package.json".to_vec()),
            Read::Base(b"package/m/a/package.json".to_vec()),
        ]
    );
    assert_eq!(read.len(), 2);
    assert_eq!(read[0].directory, b"package/m/b");
    assert_eq!(read[1].base_version, None);
}

/// Every manifest is decoded before any is parsed.
#[test]
fn decodes_every_manifest_before_parsing_any() {
    let mut memory: MemoryWorkspace = workspace(vec![
        unchanged("package/m/a/package.json", "{"),
        changed("package/m/b/package.json", r#"{"name":"b"}"#, Some("\u{0}")),
    ]);
    memory.files[1].base = Some(b"\xff".to_vec());
    assert_eq!(
        states(&mut memory),
        Err(PlanError::PolicyIncomplete(PolicyIncomplete::NotUtf8 {
            path: b"package/m/b/package.json".to_vec()
        }))
    );
    let mut current: MemoryWorkspace = workspace(vec![
        unchanged("package/m/a/package.json", "{"),
        unchanged("package/m/b/package.json", r#"{"name":"b"}"#),
    ]);
    current.files[1].current = b"\xff".to_vec();
    assert_eq!(
        states(&mut current),
        Err(PlanError::PolicyIncomplete(PolicyIncomplete::NotUtf8 {
            path: b"package/m/b/package.json".to_vec()
        }))
    );
}

/// The base text is parsed before the current text, and a broken base stops the plan.
#[test]
fn parses_the_base_before_the_current_text() {
    let mut memory: MemoryWorkspace = workspace(vec![changed(
        "package/m/a/package.json",
        "[]",
        Some(r#"{"version":"1.0.0"}"#),
    )]);
    assert_eq!(
        states(&mut memory),
        Err(PlanError::PolicyIncomplete(shape(
            b"package/m/a/package.json",
            "has no string \"name\""
        )))
    );
}

/// A failed read is content that is unavailable, whichever state it was.
#[test]
fn reports_unreadable_manifests() {
    let mut memory: MemoryWorkspace = workspace(vec![unchanged("package/m/a/package.json", "{}")]);
    memory.unreadable.push(b"package/m/a/package.json".to_vec());
    assert!(matches!(
        states(&mut memory),
        Err(PlanError::ContentUnavailable(_))
    ));
}

/// The state keeps exact bytes, text without a byte-order mark, mode and base version.
#[test]
fn keeps_bytes_text_mode_and_base_version() {
    let mut memory: MemoryWorkspace = workspace(vec![changed(
        "package/m/a/package.json",
        "\u{feff}{\"name\":\"a\",\"version\":\"2.0.0\"}",
        Some("{\"name\":\"old\",\"version\":\"1.0.0\"}"),
    )]);
    memory.files[0].mode = TrackedMode::Executable;
    let read: Vec<ManifestState> = read_ok(&mut memory);
    assert_eq!(
        read[0].bytes,
        "\u{feff}{\"name\":\"a\",\"version\":\"2.0.0\"}".as_bytes()
    );
    assert_eq!(read[0].text, "{\"name\":\"a\",\"version\":\"2.0.0\"}");
    assert_eq!(read[0].mode, TrackedMode::Executable);
    assert_eq!(read[0].base_version, Some("1.0.0".encode_utf16().collect()));
    assert_eq!(read[0].facts.name, "a");
}

/// Raised: a changed or removed version; not raised: a new manifest, no base version, same.
#[test]
fn detects_raised_versions() {
    let mut memory: MemoryWorkspace = workspace(vec![
        changed(
            "package/m/a/package.json",
            r#"{"name":"a","version":"2.0.0"}"#,
            Some(r#"{"name":"a","version":"1.0.0"}"#),
        ),
        changed(
            "package/m/b/package.json",
            r#"{"name":"b"}"#,
            Some(r#"{"name":"b","version":"1.0.0"}"#),
        ),
        changed(
            "package/m/c/package.json",
            r#"{"name":"c","version":"1.0.0"}"#,
            None,
        ),
        changed(
            "package/m/d/package.json",
            r#"{"name":"d","version":"1.0.0"}"#,
            Some(r#"{"name":"d"}"#),
        ),
        changed(
            "package/m/e/package.json",
            r#"{"name":"e","version":"1.0.0","x":1}"#,
            Some(r#"{"name":"e","version":"1.0.0"}"#),
        ),
    ]);
    assert_eq!(bumped_names(&read_ok(&mut memory)), ["a", "b"]);
}

/// The first later manifest whose name an earlier one already declared is reported.
#[test]
fn finds_the_first_duplicate_name() {
    let mut memory: MemoryWorkspace = workspace(vec![
        unchanged("package/m/a/package.json", r#"{"name":"x"}"#),
        unchanged("package/m/b/package.json", r#"{"name":"y"}"#),
        unchanged("package/m/c/package.json", r#"{"name":"x"}"#),
        unchanged("package/m/d/package.json", r#"{"name":"y"}"#),
    ]);
    assert_eq!(
        duplicate_name(&read_ok(&mut memory)),
        Some(PolicyIncomplete::DuplicateName {
            name: String::from("x"),
            first: b"package/m/a/package.json".to_vec(),
            second: b"package/m/c/package.json".to_vec(),
        })
    );
    let mut unique: MemoryWorkspace = workspace(vec![
        unchanged("package/m/a/package.json", r#"{"name":"x"}"#),
        unchanged("package/m/b/package.json", r#"{"name":"y"}"#),
    ]);
    assert_eq!(duplicate_name(&read_ok(&mut unique)), None);
}
