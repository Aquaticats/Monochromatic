//! Guard for the ban on Git starting the wrapper for a hook, and on the wrapper running external hook programs.
//!
//! The commit path must write no hook entry that names the wrapper, point `core.hooksPath` at a directory of its own
//! making, start Git's hook runner, or dispatch on the name the executable was started under.
//! The binary controls (`binary_external_hook_tests.rs`) prove the behavior end to end: no configured hook runs, a
//! declared action does, and a wrapper link named like a former entry behaves as `git`.
//! These controls prove the source shape, so a reintroduced entry writer, hook runner or `argv[0]` branch fails here
//! before any repository is built.

/// The executable entry point, which must hand its arguments to the wrapper without reading its start name.
const MAIN_SOURCE: &str = include_str!("main.rs");

/// Every source of the commit path that decides what Git is started with.
const COMMIT_PATH_SOURCES: &[(&str, &str)] = &[
    ("commit_conclusion.rs", include_str!("commit_conclusion.rs")),
    ("commit_preparation.rs", include_str!("commit_preparation.rs")),
    ("commit_revalidation.rs", include_str!("commit_revalidation.rs")),
    ("commit_workspace.rs", include_str!("commit_workspace.rs")),
    ("hooks_off.rs", include_str!("hooks_off.rs")),
    ("post_commit_action.rs", include_str!("post_commit_action.rs")),
    ("post_landing.rs", include_str!("post_landing.rs")),
    ("shadow_config.rs", include_str!("shadow_config.rs")),
];

/// What a commit-path source must never contain: a way to write an entry, a way to name this executable, Git's hook
///  runner, a hooks path built from a transaction directory, or the former entry vocabulary.
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
    "Command::new(\"sh\")",
    "Command::new(\"/bin/sh\")",
];

/// The code lines of a source: comments and doc comments are explanation, not behavior.
fn code_lines(source: &str) -> Vec<&str> {
    return source
        .lines()
        .map(str::trim_start)
        .filter(|line| { return !line.starts_with("//"); })
        .collect();
}

/// The executable never reads its start name: it collects `args_os` once, skipping the program name, and calls the
///  wrapper's entry.
#[test]
fn main_does_not_dispatch_on_its_start_name() {
    let code: Vec<&str> = code_lines(MAIN_SOURCE);
    let joined: String = code.join("\n");
    for forbidden in ["args_os().next()", "start_name", "current_exe", "arg0", "file_name", "hook_dispatch", "hook_entry"] {
        assert!(!joined.contains(forbidden), "main.rs contains {forbidden}");
    }
    assert_eq!(joined.matches("args_os()").count(), 1, "main.rs reads the arguments once: {joined}");
    assert!(joined.contains("args_os().skip(1)"), "the program name is skipped, not inspected");
    assert!(joined.contains("entry::run_process("), "main.rs hands over to the wrapper's entry");
}

/// No source of the commit path can write an entry, name this executable, run Git's hook runner or a shell, or build a
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

/// The guard sees the planted violations it exists for: a link writer, Git's hook runner, and a start-name read.
#[test]
fn the_guard_notices_a_planted_entry_writer_a_planted_runner_and_a_planted_start_name_read() {
    let planted_writer: String = code_lines("fn plant() { std::os::unix::fs::symlink(a, b); }").join("\n");
    assert!(FORBIDDEN_IN_COMMIT_PATH.iter().any(|forbidden| { return planted_writer.contains(forbidden); }));
    let planted_runner: String = code_lines("let words = [\"hook\", \"run\", \"post-commit\"];").join("\n");
    assert!(FORBIDDEN_IN_COMMIT_PATH.iter().any(|forbidden| { return planted_runner.contains(forbidden); }));
    let planted_main: String = code_lines("let start_name = std::env::args_os().next();").join("\n");
    assert!(planted_main.contains("args_os().next()"));
    let commented: String = code_lines("// symlink( in a comment is explanation\nfn ok() {}").join("\n");
    assert!(!FORBIDDEN_IN_COMMIT_PATH.iter().any(|forbidden| { return commented.contains(forbidden); }));
}
