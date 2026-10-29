//! A highlighting engine that could not start fails only files some language applies to.

/// The classification under test and the engine type it receives.
use super::{SyntaxEngine, classify};
/// What:
///  `Rope` is the text type highlighting reads;
///  `Path` names the file.
/// Why:
///  Recognition looks at the filename and the first line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Rope } from 'helix';
/// ```
use helix_core::Rope;
/// Paths for the fixture names;
///  nothing is read from disk.
use std::path::Path;

/// What:
///  An engine that failed to start,
///  as when the bundled manifest is missing.
///  `anyhow!` builds
///       an error value from text.
/// Why:
///  Both tests need the same failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const engine = failure(new Error('missing manifest'));
/// ```
fn unstarted() -> anyhow::Result<SyntaxEngine> {
    return Err(anyhow::anyhow!(
        "Cannot read the bundled language manifest: missing"
    ));
}

/// A plain text file loses nothing,
///  so the failed start is plain text,
///  not a failure.
#[test]
fn plain_text_is_not_a_failure_when_highlighting_cannot_start() {
    let reply = classify(
        &unstarted(),
        Path::new("/project/notes.txt"),
        &Rope::from_str("I am a big cat.\n"),
        3,
    );
    // `as_ref()` lends the result so the match does not take it apart.
    assert!(
        matches!(reply.result.as_ref(), Ok(None)),
        "a plain text file was reported as a highlighting failure"
    );
    assert_eq!(reply.revision, 3);
}

/// A file a language applies to loses its highlighting,
///  so the failure stays visible.
#[test]
fn a_language_file_keeps_the_failure_when_highlighting_cannot_start() {
    let reply = classify(
        &unstarted(),
        Path::new("/project/main.rs"),
        &Rope::from_str("fn main() {}\n"),
        1,
    );
    // What: `let Err(error) = ... else` binds the failure or ends the test with a message.
    // Why: The failure text names the file and the original cause.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (reply.result.ok) throw new Error('...');
    // ```
    let Err(error) = reply.result else {
        panic!("a Rust file lost its highlighting without a failure");
    };
    let text = format!("{error:#}");
    assert!(
        text.contains("Cannot initialize highlighting for /project/main.rs")
            && text.contains("missing"),
        "unexpected failure text: {text}"
    );
}

/// A shebang names a language too,
///  so a script without an extension keeps the failure visible.
#[test]
fn a_shebang_script_keeps_the_failure_when_highlighting_cannot_start() {
    let reply = classify(
        &unstarted(),
        Path::new("/project/run"),
        &Rope::from_str("#!/usr/bin/env bash\necho hi\n"),
        1,
    );
    assert!(
        reply.result.is_err(),
        "a shell script lost its highlighting without a failure"
    );
}
