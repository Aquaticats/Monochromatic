//! What:
//!  The binary tests for the server a command starts on demand,
//! from the client that starts it to the signal or idle time that ends it.
//! Why:
//! One server per worktree serves every client of that worktree,
//! and it ends by its idle time,
//! by a signal,
//! or by the harness guard that stops one a test leaves behind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe("the server a command starts on demand", () => { it("...", () => { ... }); });
//! ```

use crate::support::{
    self,
    CONFIG,
    Harness,
    count_code,
};

#[test]
fn write_then_check_through_the_auto_started_server_and_the_server_exits_when_idle() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let (check_code, check) = harness.run(&["check"]);
    assert_eq!(check_code, 1);
    assert_eq!(count_code(&check, "drift-missing"), 1, "{check:?}");
    assert_eq!(
        harness.server_pids().len(),
        1,
        "the first command started one detached server"
    );
    let (write_code, write) = harness.run(&["write"]);
    assert_eq!(write_code, 0, "{write:?}");
    assert_eq!(harness.read("CLAUDE.md"), "head\nbody\n");
    assert_eq!(
        harness.server_pids().len(),
        1,
        "later commands reuse the server"
    );
    let (clean_code, _) = harness.run(&["check"]);
    assert_eq!(clean_code, 0);
    assert!(
        harness.servers_gone_within(10),
        "the server exits after its idle time"
    );
}

#[test]
fn a_malformed_idle_seconds_value_is_a_usage_error_naming_the_variable_and_the_value() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let output = harness
        .command(&["check"])
        .env("FILE_ENFORCER_IDLE_SECONDS", "12s")
        .output()
        .expect("the binary runs");
    let lines = support::parse_lines(&String::from_utf8_lossy(&output.stdout));
    assert_eq!(output.status.code(), Some(2), "{lines:?}");
    assert_eq!(lines.len(), 1, "one line: {lines:?}");
    assert_eq!(lines[0]["type"].as_str(), Some("error"), "{lines:?}");
    let message = lines[0]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("FILE_ENFORCER_IDLE_SECONDS"),
        "the message names the variable: {message}"
    );
    assert!(
        message.contains("12s"),
        "the message names the rejected value: {message}"
    );
    assert!(
        harness.server_pids().is_empty(),
        "a rejected value starts no server"
    );
}

#[test]
fn several_clients_at_once_share_one_server_and_all_succeed() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let mut children = Vec::new();
    for _ in 0..4 {
        children.push(
            harness
                .command(&["write"])
                .stdout(std::process::Stdio::piped())
                .spawn()
                .expect("spawn"),
        );
    }
    for child in children {
        let output = child.wait_with_output().expect("wait");
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
    assert_eq!(harness.read("CLAUDE.md"), "head\nbody\n");
    assert_eq!(
        harness.server_pids().len(),
        1,
        "the startup lock elects exactly one server"
    );
}

/// What:
///  Start a server,
/// end it with `signal`,
/// check it left its socket file behind,
/// and check the next command replaces the socket and gets a new server.
/// Why:
///  The server installs no signal handler,
/// so any terminating signal skips its cleanup;
/// recovery has to come from the next client's verified stale-socket removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoversAfterSignal(signal: number): void
/// ```
fn assert_recovers_after_signal(signal: i32) {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let (first_code, _) = harness.run(&["check"]);
    assert_eq!(first_code, 1);
    let pid = harness.server_pids()[0];
    // SAFETY: `kill` with a positive pid and a terminating signal has no memory-safety preconditions;
    // the signal ends the server without its cleanup, so the socket file stays.
    unsafe {
        libc::kill(pid, signal);
    }
    assert!(
        harness.servers_gone_within(5),
        "signal {signal} ends the server"
    );
    let leftovers: Vec<_> = std::fs::read_dir(harness.runtime.path().join("file-enforcer"))
        .expect("the runtime directory exists")
        .flatten()
        .filter(|entry| return entry.file_name().to_string_lossy().ends_with(".sock"))
        .collect();
    assert_eq!(
        leftovers.len(),
        1,
        "a server ended by signal {signal} leaves its socket file behind"
    );
    let (second_code, second) = harness.run(&["check"]);
    assert_eq!(second_code, 1, "{second:?}");
    assert_eq!(count_code(&second, "drift-missing"), 1);
    let replacement = harness.server_pids();
    assert_eq!(
        replacement.len(),
        1,
        "one new server serves the second command"
    );
    assert_ne!(replacement[0], pid, "the replacement is a new process");
}

#[test]
fn a_dead_servers_leftover_socket_is_replaced() {
    assert_recovers_after_signal(libc::SIGKILL);
}

#[test]
fn a_server_ended_by_sigterm_also_leaves_a_socket_the_next_command_replaces() {
    assert_recovers_after_signal(libc::SIGTERM);
}

#[test]
fn the_harness_guard_stops_a_stray_server_with_sigterm_when_it_is_dropped() {
    let harness = Harness::new();
    // The worktree is a directory the harness does not own: a server also exits when its worktree is deleted,
    // and deleting the harness's own worktree would end the server without the guard ever being needed.
    let scratch = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-scratch");
    std::fs::create_dir_all(&scratch).expect("the scratch base");
    let worktree = tempfile::TempDir::new_in(&scratch).expect("a scratch worktree");
    std::fs::write(worktree.path().join("file-enforcer.config.jsonc"), CONFIG).expect("config");
    std::fs::write(worktree.path().join("AGENTS.md"), "body\n").expect("input");
    let config = worktree
        .path()
        .join("file-enforcer.config.jsonc")
        .display()
        .to_string();
    // A long idle time means only the guard, not the server's own idle exit, can end it within the wait below.
    let mut command = harness.command(&["--config", &config, "check"]);
    command.env("FILE_ENFORCER_IDLE_SECONDS", "600");
    let output = command.output().expect("runs");
    assert_eq!(output.status.code(), Some(1));
    let pids = harness.server_pids();
    assert_eq!(pids.len(), 1, "the check started one server");
    let pid = pids[0];
    drop(harness);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline && support::process_is_running(pid) {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let still_running = support::process_is_running(pid);
    if still_running {
        // SAFETY: `kill` with a positive pid and SIGKILL has no memory-safety preconditions; this cleans up a failed run.
        unsafe {
            libc::kill(pid, libc::SIGKILL);
        }
    }
    assert!(
        !still_running,
        "dropping the harness must stop a server it started"
    );
}
