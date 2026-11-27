//! Guard for the ban on Git starting the wrapper for a hook,
//! and on the wrapper running external hook programs.
//!
//! The commit path must write no hook entry that names the wrapper,
//! point `core.hooksPath` at a directory of its own
//! making,
//! or start Git's hook runner,
//! and the executable reads a start name in one comparison,
//! of the ZED history event's own name,
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
//!
//! The ban holds over every release source of the commit path,
//! which is stated as the two folders that path is made of plus the three files outside them,
//! so a file a split makes inside those folders is covered without anyone remembering it
//! (issue 6599:
//! the ban named ten files,
//! and the split of `commit/commit_command.rs` and `landing/service_commit.rs` left four receivers
//! no ban read).
//! Two files of those folders make a link for a reason that is not a hook,
//! and `MAY_HOLD` records each with it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(commitPathSources().flatMap(([file, text]) => linesNoEntryExcuses(linesHoldingOneOf(file, text, FORBIDDEN), MAY_HOLD))).toEqual([]);
//! ```

/// The pins' reader and the comment rule: a line as written, and comments on their own line only.
use super::{
    MayHold,
    anchors_the_scope_no_longer_holds,
    assert_whole_line_comments_only,
    entries_that_match_nothing,
    lines_holding_one_of,
    lines_no_entry_excuses,
    pinned_code_lines,
    release_sources_under,
};

/// The executable entry point, whole.
const MAIN_SOURCE: &str = include_str!("../main.rs");

/// `main.rs`'s code text, whole and in order: comments cut,
/// leading and trailing blanks trimmed.
/// A change to any line here is a change to what `main` may do with the argument vector,
/// so the pin and the code change in one commit,
/// and a reviewer reads the two diffs together.
const MAIN_CODE: &[&str] = &[
    "use git_policy_cli::diagnostic_lines_off_the_calling_thread::write_what_waits as write_every_diagnostic_line_that_waits;",
    "use git_policy_cli::service_log_lines_off_the_calling_thread::{",
    "WAIT_BEFORE_AN_EXIT,",
    "write_what_waits,",
    "};",
    "#[cfg(target_os = \"linux\")]",
    "use git_policy_cli::worktree_clone_mount_handler::root_mode_names::{",
    "self,",
    "RootModeStart,",
    "};",
    "use std::ffi::OsString;",
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
    "let code: i32 =",
    "git_policy_cli::worktree_clone_mount_handler::refuse_root_mode_misuse(misuse);",
    "end_after_the_log(code);",
    "}",
    "}",
    "}",
    "let handed: &[OsString] = arguments.get(1..).unwrap_or_default();",
    "if git_policy_cli::service_main::is_service_start(handed) {",
    "let code: i32 = match git_policy_cli::service_main::service_options(handed) {",
    "Ok(options) => git_policy_cli::service_main::run(options.as_slice()),",
    "Err(word) => {",
    "git_policy_cli::service_error!(",
    "\"an argument of the service is not text: {}\",",
    "word.to_string_lossy()",
    ");",
    "git_policy_cli::service_main::EXIT_REFUSED",
    "}",
    "};",
    "end_after_the_log(code);",
    "}",
    "git_policy_cli::panic_notice::install_payload_free_panic_hook();",
    "let environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();",
    "let start_directory: git_policy_cli::start_directory::StartDirectory =",
    "git_policy_cli::start_directory::StartDirectory::read_from_the_process();",
    "let code: i32 =",
    "git_policy_cli::entry::run_process(handed, environment.as_slice(), &start_directory);",
    "end_after_the_log(code);",
    "}",
    "fn end_after_the_log(code: i32) -> ! {",
    "let _diagnosed: bool = write_every_diagnostic_line_that_waits(WAIT_BEFORE_AN_EXIT);",
    "let _written: bool = write_what_waits(WAIT_BEFORE_AN_EXIT);",
    "std::process::exit(code);",
    "}",
];

/// What:
///  `COMMIT_PATH_SCOPE` names the release sources of the commit path:
///  the two folders the path is made of,
///  and the three files outside them that decide what a commit's Git is started with or how a hook is found.
/// Why:
///  The ban is over the commit path and not over the whole tree,
///  because the wrapper starts Git and links names elsewhere by design,
///  and a folder prefix covers every file a later split makes inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COMMIT_PATH_SCOPE = ['commit/', 'landing/', 'health/external_hooks.rs', 'invocation/hooks_off.rs', 'shadow/shadow_config.rs'];
/// ```
const COMMIT_PATH_SCOPE: &[&str] = &[
    "commit/",
    "landing/",
    "health/external_hooks.rs",
    "invocation/hooks_off.rs",
    "shadow/shadow_config.rs",
];

/// What:
///  `COMMIT_PATH_FILES_OUTSIDE_THE_FOLDERS` is the part of that scope a folder does not cover.
/// Why:
///  The three files are named one by one,
///  so a run that no longer reads one of them fails here rather than passing over a file that moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COMMIT_PATH_FILES_OUTSIDE_THE_FOLDERS = ['health/external_hooks.rs', 'invocation/hooks_off.rs', 'shadow/shadow_config.rs'];
/// ```
const COMMIT_PATH_FILES_OUTSIDE_THE_FOLDERS: &[&str] = &[
    "health/external_hooks.rs",
    "invocation/hooks_off.rs",
    "shadow/shadow_config.rs",
];

/// What:
///  `SCOPE_ANCHORS` is the text that proves the scope still reaches the code this ban is about.
/// Why:
///  A scope that reached nothing would pass every tree,
///  and the hook paths the commit path neutralises are what the ban exists beside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SCOPE_ANCHORS = [['hooksPath', 'the commit path still decides where Git looks for a hook']];
/// ```
const SCOPE_ANCHORS: &[(&str, &str)] = &[(
    "hooksPath",
    "no release file of the commit path still decides where Git looks for a hook, so this ban reaches nothing",
)];

/// What a commit-path source must never contain:
/// a way to write an entry,
/// a way to name this executable,
/// Git's hook
///  runner,
/// a hooks path built from a transaction directory,
/// or the former entry vocabulary.
/// Why the two link calls are spelled with their path:
///  `symlink(` alone is the tail of `is_symlink()`,
///  which reads a name's type and makes nothing,
///  and this crate calls both through `fs::`.
const FORBIDDEN_IN_COMMIT_PATH: &[&str] = &[
    "fs::symlink(",
    "fs::hard_link(",
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

/// What:
///  `MAY_HOLD` is the two commit-path files that make a link for a reason that is not a hook.
/// Why:
///  The ban holds over the two folders rather than over ten named files,
///  so the links those folders make for their own work are recorded where a reader of the ban meets
///  them,
///  and an entry whose line went fails here instead of excusing a file for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAY_HOLD = [{ file: 'commit/conclusion_copy.rs', text: 'fs::symlink(', why: '...' }];
/// ```
const MAY_HOLD: &[MayHold] = &[
    MayHold {
        file: "commit/conclusion_copy.rs",
        text: "fs::symlink(",
        why: "it copies the source's own links into the private index so a commit sees the tree it was made \
              in, which writes a link beside the copy and is no hook entry; the entry goes when the copy stops \
              following the source's links",
    },
    MayHold {
        file: "landing/service_index_placement.rs",
        text: "fs::hard_link(",
        why: "it places an index file into the private index by linking it, which names no hook and starts \
              nothing; the entry goes when that placement stops linking",
    },
];

/// What:
///  `commit_path_sources()` is every release source of the commit path,
///  each as its `/`-separated identity and its text,
///  read through the one walk of the crate's sources.
/// Why:
///  A ban over a folder reads the folder,
///  never a list of the files somebody remembered,
///  so a split's receivers are covered the day the split lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commitPathSources = () => releaseSourcesUnder(COMMIT_PATH_SCOPE);
/// ```
fn commit_path_sources() -> Vec<(String, String)> {
    return release_sources_under(COMMIT_PATH_SCOPE).expect("the crate's release sources");
}

/// What:
///  `files_the_scope_no_longer_reads(sources)` is every one of the three named files the walk did not answer
///  with.
/// Why:
///  Those three are the whole of the scope a folder does not cover,
///  so a run that stopped reading one of them has a scope which quietly shrank.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const filesTheScopeNoLongerReads = (sources) => COMMIT_PATH_FILES_OUTSIDE_THE_FOLDERS.filter((name) => !sources.some(([file]) => file === name));
/// ```
fn files_the_scope_no_longer_reads(sources: &[(String, String)]) -> Vec<String> {
    return COMMIT_PATH_FILES_OUTSIDE_THE_FOLDERS
        .iter()
        .filter(|name| {
            return !sources.iter().any(|(file, _)| {
                return file == *name;
            });
        })
        .map(|name| return String::from(*name))
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
    let code: Vec<String> = pinned_code_lines(MAIN_SOURCE, "main.rs")
        .into_iter()
        .filter(|line| {
            return !line.is_empty();
        })
        .collect();
    assert_eq!(
        code.len(),
        MAIN_CODE.len(),
        "main.rs has {} code line(s), and the reviewed text has {}: {code:#?}",
        code.len(),
        MAIN_CODE.len()
    );
    for (at, (found, want)) in code.iter().zip(MAIN_CODE.iter()).enumerate() {
        assert_eq!(
            found.as_str(),
            *want,
            "main.rs's code line {} is not the reviewed one",
            at + 1
        );
    }
}

/// No release source of the commit path can write an entry,
/// name this executable,
/// run Git's hook runner or a shell,
/// or build a
///  hooks path of the transaction,
/// beyond the two links `MAY_HOLD` records.
#[test]
fn the_commit_path_writes_no_entry_and_runs_no_hook_program() {
    let sources: Vec<(String, String)> = commit_path_sources();
    let holding = lines_holding_one_of(sources.as_slice(), FORBIDDEN_IN_COMMIT_PATH);
    let unexplained: Vec<String> = lines_no_entry_excuses(holding.as_slice(), MAY_HOLD);
    assert!(
        unexplained.is_empty(),
        "the commit path never writes a hook entry, names this executable, runs Git's hook runner or starts a \
         program by a literal name:\n{}",
        unexplained.join("\n")
    );
}

/// Every `MAY_HOLD` entry still excuses a line,
/// so a link that went leaves the list no longer than it is.
#[test]
fn every_entry_of_the_list_still_excuses_a_line() {
    let sources: Vec<(String, String)> = commit_path_sources();
    let holding = lines_holding_one_of(sources.as_slice(), FORBIDDEN_IN_COMMIT_PATH);
    let stale: Vec<String> = entries_that_match_nothing(holding.as_slice(), MAY_HOLD);
    assert!(
        stale.is_empty(),
        "an entry excuses a file that no longer holds the text, so the list can only shrink:\n{}",
        stale.join("\n")
    );
}

/// The scope still reads the three files a folder does not cover,
/// and still reaches the hook paths the commit path decides.
#[test]
fn the_scope_still_reads_the_commit_path_it_is_about() {
    let sources: Vec<(String, String)> = commit_path_sources();
    let unread: Vec<String> = files_the_scope_no_longer_reads(sources.as_slice());
    assert!(
        unread.is_empty(),
        "these files are outside the two folders and are no longer release sources, so the ban reaches less \
         than it says:\n{}",
        unread.join("\n")
    );
    let unanchored: Vec<String> =
        anchors_the_scope_no_longer_holds(sources.as_slice(), SCOPE_ANCHORS);
    assert!(
        unanchored.is_empty(),
        "the scope reached no file holding what this ban is about:\n{}",
        unanchored.join("\n")
    );
}

/// The scan flags each kind of violation it exists for,
/// one planted line per token,
/// and ignores comments:
/// a guard
///  that flagged nothing would pass every real source.
#[test]
fn the_scan_flags_every_forbidden_token_in_code_and_none_in_comments() {
    for token in FORBIDDEN_IN_COMMIT_PATH {
        let planted_code: String = format!("fn planted() {{\n    let _ = {token};\n}}\n");
        let sources: Vec<(String, String)> =
            vec![(String::from("commit/planted.rs"), planted_code)];
        let holding = lines_holding_one_of(sources.as_slice(), FORBIDDEN_IN_COMMIT_PATH);
        assert!(
            lines_no_entry_excuses(holding.as_slice(), MAY_HOLD)
                .iter()
                .any(|sentence| return sentence.contains(token)),
            "{token} in code"
        );
        let planted_comment: String = format!("// {token} is explained here\nfn fine() {{}}\n");
        let commented: Vec<(String, String)> =
            vec![(String::from("commit/planted.rs"), planted_comment)];
        assert!(
            lines_holding_one_of(commented.as_slice(), FORBIDDEN_IN_COMMIT_PATH).is_empty(),
            "{token} in a comment"
        );
    }
    let planted_link: Vec<(String, String)> = vec![(
        String::from("commit/planted.rs"),
        String::from(
            "fn makes() -> bool {\n    return std::os::unix::fs::symlink(from, to).is_ok();\n}\n",
        ),
    )];
    let holding = lines_holding_one_of(planted_link.as_slice(), FORBIDDEN_IN_COMMIT_PATH);
    assert_eq!(
        holding.len(),
        1,
        "a line that makes a symlink is the one finding, and `is_symlink()` is none"
    );
    let reads_a_type: Vec<(String, String)> = vec![(
        String::from("commit/planted.rs"),
        String::from(
            "fn asked(kind: std::fs::FileType) -> bool {\n    return kind.is_symlink();\n}\n",
        ),
    )];
    assert!(
        lines_holding_one_of(reads_a_type.as_slice(), FORBIDDEN_IN_COMMIT_PATH).is_empty(),
        "`is_symlink()` reads a name's type and makes no link"
    );
}
