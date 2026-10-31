//! What:
//!  Tests that run the real `file-enforcer` binary: JSON-lines output, exit codes, the auto-started server.
//! Why:
//!  The command line, the server lifecycle and the JSON-lines contract only exist in the built executable, so these
//!  tests start it as a user would, in a scratch worktree with its own runtime directory.
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

#[test]
fn a_dead_servers_leftover_socket_is_replaced() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let (first_code, _) = harness.run(&["check"]);
    assert_eq!(first_code, 1);
    let pid = harness.server_pids()[0];
    // SAFETY: `kill` with a positive pid and SIGKILL has no memory-safety preconditions; SIGKILL leaves the socket file.
    unsafe {
        libc::kill(pid, libc::SIGKILL);
    }
    assert!(harness.servers_gone_within(5));
    let leftovers: Vec<_> = std::fs::read_dir(harness.runtime.path().join("file-enforcer"))
        .expect("the runtime directory exists")
        .flatten()
        .filter(|entry| return entry.file_name().to_string_lossy().ends_with(".sock"))
        .collect();
    assert_eq!(leftovers.len(), 1, "a killed server leaves its socket file behind");
    let (second_code, second) = harness.run(&["check"]);
    assert_eq!(second_code, 1, "{second:?}");
    assert_eq!(count_code(&second, "drift-missing"), 1);
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
///  Wait for a line with the given `code` from a watch child's line channel, within a time limit.
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
    assert_eq!(count_code(&user_lines, "kind-not-wired"), 1, "the XML crate is not linked yet: {user_lines:?}");
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
