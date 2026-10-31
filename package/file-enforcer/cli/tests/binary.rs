//! What:
//!  Tests that run the real `file-enforcer` binary:
//! JSON-lines output,
//! exit codes,
//! the auto-started server.
//! Why:
//!  The command line,
//! the server lifecycle and the JSON-lines contract only exist in the built executable,
//! so these
//!  tests start it as a user would,
//! in a scratch worktree with its own runtime directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('file-enforcer binary', () => { it('prints help as one JSON line', () => { ... }); });
//! ```

mod support;

use std::os::unix::fs::PermissionsExt;
use support::{CONFIG, Harness, config_path, count_code};

#[test]
fn help_version_and_a_bad_flag_are_each_one_json_line_with_exit_codes_zero_zero_two() {
    let harness = Harness::new();
    let (help_code, help) = harness.run(&["--help"]);
    assert_eq!((help_code, help.len(), help[0]["type"].as_str()), (0, 1, Some("help")));
    assert!(help[0]["message"].as_str().unwrap_or_default().contains("write"));
    let (version_code, version) = harness.run(&["--version"]);
    assert_eq!((version_code, version.len(), version[0]["type"].as_str()), (0, 1, Some("version")));
    let (bad_code, bad) = harness.run(&["--bogus"]);
    assert_eq!((bad_code, bad.len(), bad[0]["code"].as_str()), (2, 1, Some("usage-error")));
    let outside = harness.command(&["write"]).current_dir(harness.runtime.path()).output().expect("runs");
    let missing = support::parse_lines(&String::from_utf8_lossy(&outside.stdout));
    assert_eq!(outside.status.code(), Some(2), "no config anywhere above is a usage error: {missing:?}");
    assert_eq!(missing[0]["code"], "config-missing");
    assert!(harness.server_pids().is_empty(), "no command above should have started a server");
}

#[test]
fn every_output_line_has_the_event_shape_and_increasing_sequence_numbers() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let (code, lines) = harness.run(&["check"]);
    assert_eq!(code, 1);
    for (index, line) in lines.iter().enumerate() {
        assert_eq!(line["schemaVersion"], 1);
        assert_eq!(line["sequence"], index);
        assert!(line["type"].is_string() && line["code"].is_string() && line["message"].is_string(), "{line}");
    }
}

#[test]
fn write_then_check_through_the_auto_started_server_and_the_server_exits_when_idle() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let (check_code, check) = harness.run(&["check"]);
    assert_eq!(check_code, 1);
    assert_eq!(count_code(&check, "drift-missing"), 1, "{check:?}");
    assert_eq!(harness.server_pids().len(), 1, "the first command started one detached server");
    let (write_code, write) = harness.run(&["write"]);
    assert_eq!(write_code, 0, "{write:?}");
    assert_eq!(harness.read("CLAUDE.md"), "head\nbody\n");
    assert_eq!(harness.server_pids().len(), 1, "later commands reuse the server");
    let (clean_code, _) = harness.run(&["check"]);
    assert_eq!(clean_code, 0);
    assert!(harness.servers_gone_within(10), "the server exits after its idle time");
}

#[test]
fn several_clients_at_once_share_one_server_and_all_succeed() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let mut children = Vec::new();
    for _ in 0..4 {
        children.push(harness.command(&["write"]).stdout(std::process::Stdio::piped()).spawn().expect("spawn"));
    }
    for child in children {
        let output = child.wait_with_output().expect("wait");
        assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stdout));
    }
    assert_eq!(harness.read("CLAUDE.md"), "head\nbody\n");
    assert_eq!(harness.server_pids().len(), 1, "the startup lock elects exactly one server");
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
    assert!(harness.servers_gone_within(5), "signal {signal} ends the server");
    let leftovers: Vec<_> = std::fs::read_dir(harness.runtime.path().join("file-enforcer"))
        .expect("the runtime directory exists")
        .flatten()
        .filter(|entry| return entry.file_name().to_string_lossy().ends_with(".sock"))
        .collect();
    assert_eq!(leftovers.len(), 1, "a server ended by signal {signal} leaves its socket file behind");
    let (second_code, second) = harness.run(&["check"]);
    assert_eq!(second_code, 1, "{second:?}");
    assert_eq!(count_code(&second, "drift-missing"), 1);
    let replacement = harness.server_pids();
    assert_eq!(replacement.len(), 1, "one new server serves the second command");
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
    let scratch = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join("test-scratch");
    std::fs::create_dir_all(&scratch).expect("the scratch base");
    let worktree = tempfile::TempDir::new_in(&scratch).expect("a scratch worktree");
    std::fs::write(worktree.path().join("file-enforcer.config.jsonc"), CONFIG).expect("config");
    std::fs::write(worktree.path().join("AGENTS.md"), "body\n").expect("input");
    let config = worktree.path().join("file-enforcer.config.jsonc").display().to_string();
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
    assert!(!still_running, "dropping the harness must stop a server it started");
}

#[test]
fn an_unset_runtime_directory_falls_back_to_an_owner_only_directory_under_the_cache() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let mut command = harness.command(&["check"]);
    command.env_remove("XDG_RUNTIME_DIR").env("XDG_CACHE_HOME", harness.home.path());
    let output = command.output().expect("runs");
    assert_eq!(output.status.code(), Some(1), "{}", String::from_utf8_lossy(&output.stdout));
    let fallback = harness.home.path().join("file-enforcer").join("run");
    let mode = std::fs::metadata(&fallback).expect("the fallback exists").permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);
    assert_eq!(harness.server_pids().len(), 1, "the server listens in the fallback directory");
    assert!(harness.servers_gone_within(10), "and exits when idle, so a container step ends deterministically");
}

#[test]
fn a_config_syntax_error_is_exit_two_with_path_line_and_column() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", "{\n  \"rules\": [,]\n}");
    let (code, lines) = harness.run(&["check"]);
    assert_eq!(code, 2);
    assert_eq!(lines[0]["code"], "config-syntax");
    let message = lines[0]["message"].as_str().unwrap_or_default();
    assert!(message.contains("file-enforcer.config.jsonc:2:"), "{message}");
    assert!(config_path(&harness).exists());
}

#[test]
fn a_config_field_error_is_exit_two_with_path_line_column_and_pointer() {
    let harness = Harness::new();
    harness.write(
        "file-enforcer.config.jsonc",
        "{\n  \"rules\": [\n    {\"kind\": \"mirror\", \"id\": \"m\", \"from\": \"a\", \"to\": \"/abs\"}\n  ]\n}",
    );
    let (code, lines) = harness.run(&["check"]);
    assert_eq!(code, 2, "{lines:?}");
    assert_eq!(lines[0]["code"], "config-invalid");
    let message = lines[0]["message"].as_str().unwrap_or_default();
    assert!(message.contains("file-enforcer.config.jsonc:3:54: /rules/0/to: "), "{message}");
}

#[test]
fn check_with_an_expected_dir_writes_paired_trees_for_diff_tools() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    harness.write("CLAUDE.md", "stale\n");
    let diff_dir = harness.home.path().join("diff");
    let diff_arg = diff_dir.display().to_string();
    let (code, lines) = harness.run(&["check", "--expected-dir", &diff_arg]);
    assert_eq!(code, 1, "{lines:?}");
    assert_eq!(std::fs::read_to_string(diff_dir.join("actual/CLAUDE.md")).expect("actual"), "stale\n");
    assert_eq!(std::fs::read_to_string(diff_dir.join("expected/CLAUDE.md")).expect("expected"), "head\nbody\n");
}

/// One committed output and one output the repository's ignore rules cover.
const SPLIT_CONFIG: &str = r#"{"rules": [
    {"kind": "concat", "id": "committed-rule", "output": "committed.txt", "parts": ["c\n"]},
    {"kind": "concat", "id": "local-rule", "output": "out.local.txt", "parts": ["l\n"]}]}"#;

/// The `path` members of the drift lines.
fn drift_paths(lines: &[serde_json::Value]) -> Vec<&str> {
    let mut paths: Vec<&str> = Vec::new();
    for line in lines {
        if line["type"] == "drift" {
            paths.push(line["path"].as_str().expect("a drift names its path"));
        }
    }
    return paths;
}

#[test]
fn outputs_limits_check_and_write_to_committed_or_local_files() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", SPLIT_CONFIG);
    harness.write(".gitignore", "*.local.*\n");
    let (all_code, all) = harness.run(&["check"]);
    assert_eq!((all_code, drift_paths(&all)), (1, vec!["committed.txt", "out.local.txt"]), "{all:?}");
    let (committed_code, committed) = harness.run(&["check", "--outputs", "committed"]);
    assert_eq!((committed_code, drift_paths(&committed)), (1, vec!["committed.txt"]), "{committed:?}");
    let (local_code, local) = harness.run(&["check", "--outputs=local"]);
    assert_eq!((local_code, drift_paths(&local)), (1, vec!["out.local.txt"]), "{local:?}");
    assert_eq!(local.last().expect("a result line")["outputs"], "local");
    let (written_code, written) = harness.run(&["write", "--outputs", "committed"]);
    assert_eq!(written_code, 0, "{written:?}");
    assert_eq!(harness.read("committed.txt"), "c\n");
    assert!(!harness.repo.path().join("out.local.txt").exists(), "the local output is not written by a committed-only write");
    let (clean_code, clean) = harness.run(&["check", "--outputs", "committed"]);
    assert_eq!(clean_code, 0, "the missing local output is outside the scope: {clean:?}");
    assert_eq!(clean.last().expect("a result line")["code"], "check-clean");
    let (still_code, still) = harness.run(&["check", "--outputs", "all"]);
    assert_eq!((still_code, drift_paths(&still)), (1, vec!["out.local.txt"]), "{still:?}");
    let (local_write_code, local_write) = harness.run(&["write", "--outputs", "local"]);
    assert_eq!(local_write_code, 0, "{local_write:?}");
    assert_eq!(harness.read("out.local.txt"), "l\n");
    let (final_code, final_lines) = harness.run(&["check", "--outputs", "local"]);
    assert_eq!(final_code, 0, "{final_lines:?}");
}

#[test]
fn outputs_rejects_unknown_words_repeats_and_the_user_commands_as_usage_errors() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", SPLIT_CONFIG);
    for arguments in [
        vec!["check", "--outputs", "bogus"],
        vec!["check", "--outputs"],
        vec!["write", "--outputs", "local", "--outputs", "committed"],
        vec!["explain", "--outputs", "local", "committed.txt"],
        vec!["user", "check", "--outputs", "local"],
    ] {
        let (code, lines) = harness.run(&arguments);
        assert_eq!((code, lines.len(), lines[0]["code"].as_str()), (2, 1, Some("usage-error")), "{arguments:?}: {lines:?}");
    }
    let (help_code, help) = harness.run(&["check", "--help"]);
    assert_eq!(help_code, 0);
    let text = help[0]["message"].as_str().unwrap_or_default();
    assert!(text.contains("--outputs") && text.contains("committed") && text.contains("local"), "{text}");
    assert!(harness.server_pids().is_empty(), "a usage error never starts a server");
}

#[test]
fn explain_names_the_rule_that_generates_a_path() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let (code, lines) = harness.run(&["explain", "CLAUDE.md"]);
    assert_eq!(code, 0);
    assert_eq!(count_code(&lines, "explain-output"), 1, "{lines:?}");
    assert_eq!(lines[0]["rule"], "claude");
}

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
fn wait_for_code(lines: &std::sync::mpsc::Receiver<serde_json::Value>, code: &str, seconds: u64) -> serde_json::Value {
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
    let mut child = harness.command(&["watch"]).stdout(std::process::Stdio::piped()).spawn().expect("spawn");
    let stdout = child.stdout.take().expect("piped stdout");
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines().map_while(Result::ok) {
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
    assert!(warning["message"].as_str().unwrap_or_default().contains("CLAUDE.md"), "{warning}");
    wait_for_code(&receiver, "file-written", 15);
    assert_eq!(harness.read("CLAUDE.md"), "head\nchanged\n", "the hand edit was regenerated away");
    child.kill().expect("kill the watch client");
    let _ = child.wait();
    assert!(harness.servers_gone_within(15), "the server notices the client left and exits when idle");
}

#[test]
fn user_commands_use_their_own_server_and_the_per_user_config() {
    let harness = Harness::new();
    let user_config = harness.home.path().join("user.config.jsonc");
    let user_text = r#"{"rules": [{"kind": "jetbrains-harper", "id": "harper", "productPrefixes": ["IdeaIC"],
        "disabledLinters": [], "excludePatterns": [], "scopedServers": []}]}"#;
    std::fs::write(&user_config, user_text).expect("user config");
    let config_argument = user_config.display().to_string();
    let (user_code, user_lines) = harness.run(&["--user-config", &config_argument, "user", "check"]);
    assert_eq!(user_code, 0, "{user_lines:?}");
    assert_eq!(count_code(&user_lines, "kind-not-wired"), 0, "the XML crate is linked: {user_lines:?}");
    assert_eq!(count_code(&user_lines, "jetbrains-harper-skipped"), 1, "the disposable home has no IDE: {user_lines:?}");
    assert_eq!(harness.server_pids().len(), 1);
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let (repository_code, _) = harness.run(&["check"]);
    assert_eq!(repository_code, 1);
    assert_eq!(harness.server_pids().len(), 2, "the repository and the per-user config have separate servers");
    let (missing_code, missing) = harness.run(&["--user-config", "does-not-exist.jsonc", "user", "write"]);
    assert_eq!(missing_code, 2, "{missing:?}");
    assert_eq!(missing[0]["code"], "config-unreadable");
}

#[test]
fn the_rust_catalog_kind_is_wired_into_the_binary() {
    let harness = Harness::new();
    let config = r#"{"rules": [{"kind": "rust-catalog", "id": "catalog", "catalog": "rust-catalog.toml",
        "manifests": ["package/*/*/Cargo.toml"]}]}"#;
    harness.write("file-enforcer.config.jsonc", config);
    harness.write("rust-catalog.toml", "[catalog]\n");
    harness.write("package/cli/a/Cargo.toml", "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nanyhow = \"1\"\n\n[workspace]\n");
    let (check_code, check) = harness.run(&["check"]);
    assert_eq!((check_code, count_code(&check, "kind-not-wired"), count_code(&check, "drift-content")), (1, 0, 2), "{check:?}");
    let (write_code, write) = harness.run(&["write"]);
    assert_eq!(write_code, 0, "{write:?}");
    assert!(harness.read("rust-catalog.toml").contains("anyhow = \">=1\""));
    let (clean_code, clean) = harness.run(&["check"]);
    assert_eq!(clean_code, 0, "{clean:?}");
}
