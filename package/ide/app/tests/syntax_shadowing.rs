//! Highlighting reads only the application's own runtime:
//!  a query file below
//! `$XDG_CONFIG_HOME/helix/runtime`,
//!  which Helix's own lookup prefers over every other runtime
//! directory,
//!  must not replace a bundled query.
//!
//! The case comes from the packaged-application check of 2026-10-05:
//!  an empty
//! `queries/sql/highlights.scm` there turned SQL into plain text (16 colored spans without it,
//!  0 with it).

/// Canonical text and the application-owned highlighting engine.
use helix_core::Rope;
use ide_app::syntax::SyntaxEngine;
/// What:
///  Files,
///  paths,
///  and child processes.
/// Why:
///  The check runs in a child process whose `XDG_CONFIG_HOME` points at a disposable folder,
///      because Helix reads that variable once per process and the test must not change its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import fs from 'node:fs'; import { spawnSync } from 'node:child_process';
/// ```
use std::{fs, path::Path, process::Command};

/// Set in the child process,
///  which performs the check instead of starting another child.
const CHILD: &str = "IDE_SYNTAX_SHADOWING_CHILD";

/// A statement with keywords,
///  an identifier,
///  a number,
///  and punctuation for the SQL rules to color.
const SQL: &str = "SELECT name FROM cats WHERE age > 3;\n";

#[test]
fn a_user_helix_runtime_query_does_not_replace_the_bundled_one() {
    if std::env::var_os(CHILD).is_some() {
        let engine = SyntaxEngine::new().expect("language runtime");
        let text = Rope::from_str(SQL);
        let spans = engine
            .highlight(Path::new("/project/fixture.sql"), &text)
            .expect("bundled SQL grammar and rules")
            .expect("SQL is a bundled language");
        let colored = spans.iter().count();
        println!("colored spans with a shadowing user query present: {colored}");
        assert!(
            colored > 0,
            "an empty highlights.scm below XDG_CONFIG_HOME/helix/runtime replaced the bundled SQL rules"
        );
        return;
    }
    let directory = tempfile::tempdir().expect("disposable configuration home");
    let queries = directory.path().join("helix/runtime/queries/sql");
    fs::create_dir_all(&queries).expect("user Helix query folder");
    fs::write(queries.join("highlights.scm"), "").expect("empty shadowing query");
    let status = Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "a_user_helix_runtime_query_does_not_replace_the_bundled_one",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD, "1")
        .env("XDG_CONFIG_HOME", directory.path())
        .status()
        .expect("child test process");
    assert!(
        status.success(),
        "the shadowing check failed in its child process: {status}"
    );
}
