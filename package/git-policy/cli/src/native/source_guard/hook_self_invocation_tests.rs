//! Guard for the ban on Git starting the wrapper for a hook,
//! and on the wrapper running external hook programs.
//!
//! The commit path must write no hook entry that names the wrapper,
//! point `core.hooksPath` at a directory of its own
//! making,
//! or start Git's hook runner,
//! and the executable reads a start name in one comparison,
//! of the installed ZED history event's own name,
//! and nowhere else.
//! The binary controls (`binary_external_hook_tests.rs`) prove the behavior end to end:
//! no configured hook runs,
//! a
//! declared action does,
//! and a wrapper link named like a former entry behaves as `git`.
//! These controls prove the source shape,
//! so a reintroduced entry writer,
//! hook runner or `argv[0]` branch fails here
//! before any repository is built.

/// The pins' reader and the comment rule: a line as written, and comments on their own line only.
use super::{assert_whole_line_comments_only, pinned_code_lines};

/// The executable entry point, whole.
const MAIN_SOURCE: &str = include_str!("../main.rs");

/// `main.rs`'s code text, whole and in order: comments cut,
/// leading and trailing blanks trimmed.
/// A change to any line here is a change to what `main` may do with the argument vector,
/// so the pin and the code change in one commit,
/// and a reviewer reads the two diffs together.
const MAIN_CODE: [&str; 47] = [
    "use std::ffi::OsString;",
    "use git_policy_cli::service_log_lines_off_the_calling_thread::{WAIT_BEFORE_AN_EXIT, write_what_waits};",
    "use git_policy_cli::diagnostic_lines_off_the_calling_thread::write_what_waits as write_every_diagnostic_line_that_waits;",
    "#[cfg(target_os = \"linux\")]",
    "use git_policy_cli::worktree_clone_mount_handler::root_mode_names::{self, RootModeStart};",
    "fn main() {",
    "#[allow(clippy::disallowed_methods)]",
    "let arguments: Vec<OsString> = std::env::args_os().collect();",
    "#[cfg(target_os = \"linux\")]",
    "{",
    "match root_mode_names::root_mode_start(arguments.as_slice()) {",
    "RootModeStart::Wrapper => {}",
    "RootModeStart::Arguments(mode_arguments) => {",
    "git_policy_cli::panic_notice::install_payload_free_panic_hook();",
    "let code: i32 = git_policy_cli::worktree_clone_mount_handler::run(mode_arguments);",
    "end_after_the_log(code);",
    "}",
    "RootModeStart::Refused(misuse) => {",
    "git_policy_cli::panic_notice::install_payload_free_panic_hook();",
    "let code: i32 = git_policy_cli::worktree_clone_mount_handler::refuse_root_mode_misuse(misuse);",
    "end_after_the_log(code);",
    "}",
    "}",
    "}",
    "let handed: &[OsString] = arguments.get(1..).unwrap_or_default();",
    "if git_policy_cli::service_main::is_service_start(handed) {",
    "let code: i32 = match git_policy_cli::service_main::service_options(handed) {",
    "Ok(options) => git_policy_cli::service_main::run(options.as_slice()),",
    "Err(word) => {",
    "git_policy_cli::service_error!(\"an argument of the service is not text: {}\", word.to_string_lossy());",
    "git_policy_cli::service_main::EXIT_REFUSED",
    "}",
    "};",
    "end_after_the_log(code);",
    "}",
    "git_policy_cli::panic_notice::install_payload_free_panic_hook();",
    "let environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();",
    "let start_directory: git_policy_cli::start_directory::StartDirectory =",
    "git_policy_cli::start_directory::StartDirectory::read_from_the_process();",
    "let code: i32 = git_policy_cli::entry::run_process(handed, environment.as_slice(), &start_directory);",
    "end_after_the_log(code);",
    "}",
    "fn end_after_the_log(code: i32) -> ! {",
    "let _diagnosed: bool = write_every_diagnostic_line_that_waits(WAIT_BEFORE_AN_EXIT);",
    "let _written: bool = write_what_waits(WAIT_BEFORE_AN_EXIT);",
    "std::process::exit(code);",
    "}",
];

/// Every source of the commit path that decides what Git is started with.
const COMMIT_PATH_SOURCES: &[(&str, &str)] = &[
    ("commit/commit_conclusion.rs", include_str!("../commit/commit_conclusion.rs")),
    ("commit/commit_preparation.rs", include_str!("../commit/commit_preparation.rs")),
    ("invocation/hooks_off.rs", include_str!("../invocation/hooks_off.rs")),
    ("landing/service_commit.rs", include_str!("../landing/service_commit.rs")),
    ("landing/service_native.rs", include_str!("../landing/service_native.rs")),
    ("landing/service_prepare.rs", include_str!("../landing/service_prepare.rs")),
    ("health/external_hooks.rs", include_str!("../health/external_hooks.rs")),
    ("shadow/shadow_config.rs", include_str!("../shadow/shadow_config.rs")),
];

/// What a commit-path source must never contain:
/// a way to write an entry,
/// a way to name this executable,
/// Git's hook
///  runner,
/// a hooks path built from a transaction directory,
/// or the former entry vocabulary.
const FORBIDDEN_IN_COMMIT_PATH: &[&str] = &[
    "symlink(",
    "hard_link(",
    "current_exe",
    "write_hook_dispatch",
    "hook_entry_name",
    "hook_dispatch::",
    "format!(\"core.hooksPath={}\"",
    "\"hook\", \"run\"",
    "hook run",
    "PreparationHooks",
    "Command::new(\"",
    "Command::new(r",
];

/// The code lines of a source:
/// comments and doc comments are explanation,
/// not behavior.
fn code_lines(source: &str) -> Vec<&str> {
    return source
        .lines()
        .map(str::trim_start)
        .filter(|line| { return !line.starts_with("//"); })
        .collect();
}

/// What:
///  The executable is the reviewed handful of statements, whole.
/// Why:
///  `main` is where a start becomes a root mode,
///  so its code text is pinned line for line:
///  any line added, removed, reordered, reworded or re-spelled fails here,
///  and the commit that changes it changes this list in the same diff.
///  A whole-line comment and a blank line are not code, so an explanation
///  re-pins nothing.
#[test]
fn main_is_the_reviewed_handful_of_statements() {
    assert_whole_line_comments_only("main.rs", MAIN_SOURCE);
    let code: Vec<String> = pinned_code_lines(MAIN_SOURCE, "main.rs").into_iter().filter(|line| { return !line.is_empty(); }).collect();
    assert_eq!(code.len(), MAIN_CODE.len(), "main.rs has {} code line(s), and the reviewed text has {}: {code:#?}", code.len(), MAIN_CODE.len());
    for (at, (found, want)) in code.iter().zip(MAIN_CODE.iter()).enumerate() {
        assert_eq!(found.as_str(), *want, "main.rs's code line {} is not the reviewed one", at + 1);
    }
}

/// No source of the commit path can write an entry,
/// name this executable,
/// run Git's hook runner or a shell,
/// or build a
///  hooks path of the transaction.
#[test]
fn the_commit_path_writes_no_entry_and_runs_no_hook_program() {
    for (name, source) in COMMIT_PATH_SOURCES.iter().copied() {
        let joined: String = code_lines(source).join("\n");
        for forbidden in FORBIDDEN_IN_COMMIT_PATH.iter().copied() {
            assert!(!joined.contains(forbidden), "{name} contains {forbidden}");
        }
    }
}

/// The forbidden tokens a source contains,
/// comments excluded.
fn violations(source: &str) -> Vec<&'static str> {
    let joined: String = code_lines(source).join("\n");
    return FORBIDDEN_IN_COMMIT_PATH.iter().copied().filter(|forbidden| { return joined.contains(forbidden); }).collect();
}

/// The scan flags each kind of violation it exists for,
/// one planted source per token,
/// and ignores comments:
/// a guard
///  that flagged nothing would pass every real source.
#[test]
fn the_scan_flags_every_forbidden_token_in_code_and_none_in_comments() {
    for token in FORBIDDEN_IN_COMMIT_PATH {
        let planted_code: String = format!("fn planted() {{ let _ = {token}; }}");
        assert!(violations(planted_code.as_str()).contains(token), "{token} in code");
        let planted_comment: String = format!("// {token} is explained here\nfn fine() {{}}");
        assert!(violations(planted_comment.as_str()).is_empty(), "{token} in a comment");
    }
    assert!(violations("let shell = Command::new(\"sh\");").contains(&"Command::new(\""));
}
