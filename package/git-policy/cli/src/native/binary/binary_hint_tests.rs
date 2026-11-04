//! What:
//!  The hints a caller sees from the built executable for blocks outside the discard guard: a bulk add, an invalid
//!  configuration, and the forbidden root context file.
//! Why:
//!  Every block must say what the person was probably doing and the next step; the unit guard proves each code has a
//!  hint, and these prove the hint reaches the caller beside an unchanged message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(JSON.parse(stderr).hint).toMatch(/^Hint: /);
//! ```

use super::support::{Fixture, Observed, fixture, observe_raw, remove, repository, wrapped};
use std::ffi::OsStr;
use std::path::PathBuf;

/// Both streams of an observation as text.
fn streams(observed: &Observed) -> String {
    return format!(
        "{}{}",
        String::from_utf8_lossy(observed.stdout.as_slice()),
        String::from_utf8_lossy(observed.stderr.as_slice())
    );
}

/// A bulk add is rejected with its own alternatives in the message, and the hint field adds the intent guess.
#[test]
fn a_bulk_add_carries_an_intent_hint() {
    let fixture: Fixture = fixture("hint-bulk-add");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    let observed: Observed = observe_raw(wrapped(&fixture).current_dir(repo.as_path()).args(["add", "."]), b"");
    let text: String = streams(&observed);
    assert_eq!(observed.code, Some(1), "{text}");
    assert!(text.contains("\"code\":\"add-explicit/bulk-add-rejected\""), "{text}");
    assert!(
        text.contains("\"hint\":\"Hint: if you meant to stage everything you changed, run `git status --short`"),
        "{text}"
    );
    remove(&fixture);
}

/// An invalid configuration stops the command with an engine failure whose hint names the file to fix.
#[test]
fn an_invalid_configuration_carries_an_intent_hint() {
    let fixture: Fixture = fixture("hint-config");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("cli-git.config.jsonc"), b"{ \"notAKey\": true }\n").expect("config");
    let observed: Observed = observe_raw(wrapped(&fixture).current_dir(repo.as_path()).args(["cli-git", "check", "--all"]), b"");
    let text: String = streams(&observed);
    assert_eq!(observed.code, Some(2), "{text}");
    assert!(text.contains("\"code\":\"config-invalid\""), "{text}");
    assert!(
        text.contains("\"hint\":\"Hint: if you just edited cli-git.config.jsonc, fix the key the message names"),
        "{text}"
    );
    remove(&fixture);
}

/// The forbidden root context file is reported with a hint that keeps the file on disk and drops it from the commit.
#[test]
fn a_root_context_file_carries_an_intent_hint() {
    let fixture: Fixture = fixture("hint-root-context");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(
        repo.join("cli-git.config.jsonc"),
        b"{ \"policies\": { \"final-newline\": \"off\", \"mono/forbidden-root-context\": \"error\" } }\n",
    )
    .expect("config");
    std::fs::write(repo.join("CONTEXT.md"), b"context\n").expect("context");
    let observed: Observed = observe_raw(
        wrapped(&fixture).current_dir(repo.as_path()).args(["cli-git", "check", "--all"]),
        b"",
    );
    let text: String = streams(&observed);
    assert_eq!(observed.code, Some(1), "{text}");
    assert!(text.contains("\"message\":\"Root CONTEXT.md is forbidden; read source code directly.\",\"hint\":\"Hint: if you meant to keep project notes"), "{text}");
    assert!(text.contains("git rm --cached CONTEXT.md"), "{text}");
    remove(&fixture);
}
