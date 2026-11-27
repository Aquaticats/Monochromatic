//! What:
//!  The binary test for the watch command,
//! and the wait its lines arrive through.
//! Why:
//! Watch is the one long-running command:
//! it prints a line per event,
//! regenerates on an input change,
//! and warns when a generated file was edited by hand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe("the watch command", () => { it("...", () => { ... }); });
//! ```

use crate::support::{
    CONFIG,
    Harness,
};

/// What:
///  Wait for a line with the given `code` from a watch child's line channel,
/// within a time limit.
/// Why:
///  Watch output arrives asynchronously after file-system events.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function waitForCode(lines: Channel<object>, code: string, seconds: number): Promise<object>
/// ```
fn wait_for_code(
    lines: &std::sync::mpsc::Receiver<serde_json::Value>,
    code: &str,
    seconds: u64,
) -> serde_json::Value {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    while std::time::Instant::now() < deadline {
        if let Ok(line) = lines.recv_timeout(std::time::Duration::from_millis(200))
            && line["code"] == code
        {
            return line;
        }
    }
    panic!("no line with code {code} within {seconds} seconds");
}

#[test]
fn watch_regenerates_on_input_changes_and_warns_on_hand_edits() {
    use std::io::BufRead;
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let mut child = harness
        .command(&["watch"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");
    let stdout = child.stdout.take().expect("piped stdout");
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout)
            .lines()
            .map_while(Result::ok)
        {
            if let Ok(value) = serde_json::from_str(&line) {
                let _ = sender.send(value);
            }
        }
    });
    wait_for_code(&receiver, "watch-started", 15);
    wait_for_code(&receiver, "file-written", 15);
    assert_eq!(harness.read("CLAUDE.md"), "head\nbody\n");
    harness.write("AGENTS.md", "changed\n");
    wait_for_code(&receiver, "file-written", 15);
    assert_eq!(harness.read("CLAUDE.md"), "head\nchanged\n");
    harness.write("CLAUDE.md", "edited by hand\n");
    let warning = wait_for_code(&receiver, "generated-file-edited", 15);
    assert!(
        warning["message"]
            .as_str()
            .unwrap_or_default()
            .contains("CLAUDE.md"),
        "{warning}"
    );
    wait_for_code(&receiver, "file-written", 15);
    assert_eq!(
        harness.read("CLAUDE.md"),
        "head\nchanged\n",
        "the hand edit was regenerated away"
    );
    child.kill().expect("kill the watch client");
    let _ = child.wait();
    assert!(
        harness.servers_gone_within(15),
        "the server notices the client left and exits when idle"
    );
}
