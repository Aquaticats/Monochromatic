//! The application's language tick hands the worker's file-change queue to the change watcher,
//!  so a file
//! changed outside the IDE reaches the scripted server that registered a watcher for it.

/// Fixtures,
///  the scripted server's definitions,
///  bounded waiting,
///  and the readiness wait.
use super::test_support::{address, definitions, project, reader, ready};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// External writes and the server's report.
use std::fs;

/// A file written by another program reaches the server as `workspace/didChangeWatchedFiles`.
#[test]
fn an_external_change_reaches_the_server_through_the_application() {
    let fixture = project(&[
        ("main.scripted", "watched\n"),
        ("lib/other.scripted", "before\n"),
    ]);
    let report = fixture.outside.join("report.jsonl");
    let report_text = report.display().to_string();
    let reader = reader(
        &fixture,
        "main.scripted",
        definitions(
            &[
                ("WATCHERS", r#"[{"globPattern":"**/*.scripted"}]"#),
                ("REPORT", &report_text),
            ],
            None,
        ),
    );
    ready(&reader);
    let changed = fixture.root.join("lib/other.scripted");
    let expected = address(&changed);
    // The server registers after `initialized` and the folders are then scanned, so the write is
    // repeated every 100 ms until one lands in a watched folder.
    let mut ticks = 0;
    let mut waiting = Waiting::language();
    loop {
        if ticks % 50 == 0 {
            fs::write(&changed, format!("after {ticks}\n")).expect("external change");
        }
        ticks += 1;
        // What: `unwrap_or_default` reads an absent report as empty text.
        // Why: The server creates the report when it starts.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const text = existsSync(report) ? readFileSync(report, 'utf8') : '';
        // ```
        let text = fs::read_to_string(&report).unwrap_or_default();
        let mut reached = false;
        for line in text.lines() {
            if line.contains("workspace/didChangeWatchedFiles") && line.contains(&expected) {
                reached = true;
                break;
            }
        }
        if reached {
            break;
        }
        waiting.step("the external change did not reach the server");
    }
}
