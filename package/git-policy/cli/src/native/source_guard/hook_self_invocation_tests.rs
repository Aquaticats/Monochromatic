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
//! A file outside those two folders and the three named files is one this ban does not read,
//! and `SCOPE_ANCHORS` is what proves the scope still reaches the commit path;
//! a file that moves a hook entry out of it is found by a reader and by no rule here.
//! Two files of those folders make a link for a reason that is not a hook,
//! and `MAY_HOLD` records each with the one use it holds and the reason it may.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(commitPathSources().flatMap(([file, text]) => linesNoEntryExcuses([...linesHoldingOneOf([[file, text]], FORBIDDEN), ...linesCallingOneOf([[file, text]], LINKS)], MAY_HOLD))).toEqual([]);
//! ```

/// The pins' reader and the comment rule: a line as written, and comments on their own line only.
use super::{
    HoldingLine,
    MayHold,
    anchors_the_scope_no_longer_holds,
    assert_whole_line_comments_only,
    entries_that_match_nothing,
    lines_calling_one_of,
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
/// Why the two link calls are read as calls rather than as these texts:
///  `symlink(` read as text is the tail of `is_symlink()`,
///  which reads a name's type and makes nothing,
///  so a link is banned by its name in `LINKS_A_FILE_MAY_NOT_MAKE`,
///  and every text here is one that means the same wherever it occurs.
const FORBIDDEN_IN_COMMIT_PATH: &[&str] = &[
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
///  `LINKS_A_FILE_MAY_NOT_MAKE` is every call that makes one name lead to another,
///  spelled as the standard library and the C library spell them.
/// Why:
///  The ban exists so no commit path writes a hook entry,
///  and an entry is a link written under one of these names:
///  the call behind its path,
///  the bare call of an imported name,
///  and the `at` form a directory descriptor goes through.
///  A name is read here where a name begins,
///  so `link(` is a finding and `hard_link(`, `read_link(` and `is_symlink(` are not,
///  and the control beside this list plants every spelling and every one of those readings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LINKS_A_FILE_MAY_NOT_MAKE = ['symlink(', 'symlinkat(', 'hard_link(', 'link(', 'linkat('];
/// ```
const LINKS_A_FILE_MAY_NOT_MAKE: &[&str] =
    &["symlink(", "symlinkat(", "hard_link(", "link(", "linkat("];

/// What:
///  `MAY_HOLD` is the two commit-path files that make a link for a reason that is not a hook,
///  each with the one use of its text that file holds.
/// Why:
///  The ban holds over the two folders rather than over ten named files,
///  so the links those folders make for their own work are recorded where a reader of the ban meets
///  them,
///  and an entry whose line went fails here instead of excusing a file for ever.
///  The count is what an entry names rather than the excused line:
///  a second link added to one of these files fails,
///  and a file whose link is reflowed or moved keeps its one use and passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAY_HOLD = [{ file: 'commit/conclusion_copy.rs', text: 'symlink(', uses: 1, why: '...' }];
/// ```
const MAY_HOLD: &[MayHold] = &[
    MayHold {
        file: "commit/conclusion_copy.rs",
        text: "symlink(",
        uses: 1,
        why: "it copies the source's own links into the private index so a commit sees the tree it was made \
              in, which writes a link beside the copy and is no hook entry; the entry goes when the copy stops \
              following the source's links",
    },
    MayHold {
        file: "landing/service_index_placement.rs",
        text: "hard_link(",
        uses: 1,
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

/// What:
///  `lines_the_commit_path_may_not_hold(sources)` is every code line of those sources that holds one of
///  `FORBIDDEN_IN_COMMIT_PATH` or calls one of `LINKS_A_FILE_MAY_NOT_MAKE`.
/// Why:
///  The ban is one list of findings over one scope,
///  so the two readings are joined here rather than in each control that asks what the ban found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linesTheCommitPathMayNotHold = (sources) => [...linesHoldingOneOf(sources, FORBIDDEN_IN_COMMIT_PATH), ...linesCallingOneOf(sources, LINKS_A_FILE_MAY_NOT_MAKE)];
/// ```
fn lines_the_commit_path_may_not_hold(sources: &[(String, String)]) -> Vec<HoldingLine> {
    let mut holding: Vec<HoldingLine> = lines_holding_one_of(sources, FORBIDDEN_IN_COMMIT_PATH);
    holding.extend(lines_calling_one_of(sources, LINKS_A_FILE_MAY_NOT_MAKE));
    return holding;
}

/// No release source of the commit path can write an entry,
/// name this executable,
/// run Git's hook runner or a shell,
/// make a link,
/// or build a
///  hooks path of the transaction,
/// beyond the links `MAY_HOLD` records.
#[test]
fn the_commit_path_writes_no_entry_and_runs_no_hook_program() {
    let sources: Vec<(String, String)> = commit_path_sources();
    let holding = lines_the_commit_path_may_not_hold(sources.as_slice());
    let unexplained: Vec<String> = lines_no_entry_excuses(holding.as_slice(), MAY_HOLD);
    assert!(
        unexplained.is_empty(),
        "the commit path never writes a hook entry, names this executable, runs Git's hook runner, starts a \
         program by a literal name or makes a link:\n{}",
        unexplained.join("\n")
    );
}

/// Every `MAY_HOLD` entry still excuses a line,
/// so a link that went leaves the list no longer than it is.
#[test]
fn every_entry_of_the_list_still_excuses_a_line() {
    let sources: Vec<(String, String)> = commit_path_sources();
    let holding = lines_the_commit_path_may_not_hold(sources.as_slice());
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
}

/// What:
///  The lines that make a link, one per spelling the ban has to fail on,
///  and the lines that read a name's type or its target, which make none.
/// Why:
///  A ban that failed on one spelling and passed another would be a ban a rename walks around,
///  and a ban that read `link(` as text would fail every file that asks `is_symlink()`.
const SPELLINGS_OF_A_LINK: &[&str] = &[
    "let _ = std::os::unix::fs::symlink(from, to);",
    "let _ = libc::symlink(from, to);",
    "let _ = symlink(from, to);",
    "let _ = std::os::unix::fs::symlinkat(from, to, at);",
    "let _ = std::fs::hard_link(from, to);",
    "let _ = std::fs::link(from, to);",
    "let _ = libc::linkat(from, to, at);",
];

/// What:
///  `READINGS_OF_A_NAME` is the names that end in a banned call's name and make no link.
/// Why:
///  `is_symlink`, `symlink_metadata` and `read_link` are how this crate asks what a name is and where it leads,
///  and a ban that failed on them would fail every file that asks.
const READINGS_OF_A_NAME: &[&str] = &[
    "let _ = kind.is_symlink();",
    "let _ = std::fs::symlink_metadata(path);",
    "let _ = std::fs::read_link(path);",
    "let _ = path.symlink_metadata();",
    "let _ = metadata.file_type().is_symlink();",
];

/// Every spelling of a call that makes a link is a finding,
/// every name the ban was narrowed away from is none,
/// and a comment naming either is no finding.
#[test]
fn the_ban_fails_on_every_spelling_of_a_link_and_on_no_reading_of_one() {
    let mut caught: Vec<&str> = Vec::new();
    for spelling in SPELLINGS_OF_A_LINK {
        let planted: Vec<(String, String)> = vec![(
            String::from("commit/planted.rs"),
            format!("fn planted() {{\n    {spelling}\n}}\n"),
        )];
        let holding = lines_calling_one_of(planted.as_slice(), LINKS_A_FILE_MAY_NOT_MAKE);
        assert_eq!(
            holding.len(),
            1,
            "{spelling} makes one link and is {} finding(s)",
            holding.len()
        );
        let unexplained: Vec<String> = lines_no_entry_excuses(holding.as_slice(), MAY_HOLD);
        assert_eq!(
            unexplained.len(),
            1,
            "{spelling} is a link no entry excuses, and this is a file no entry names: {unexplained:?}"
        );
        caught.extend(holding.iter().map(|held| return held.text));
    }
    for call in LINKS_A_FILE_MAY_NOT_MAKE {
        assert!(
            caught.contains(call),
            "{call} is banned and no spelling of it was planted"
        );
    }
    for reading in READINGS_OF_A_NAME {
        let planted: Vec<(String, String)> = vec![(
            String::from("commit/planted.rs"),
            format!("fn planted() {{\n    {reading}\n}}\n"),
        )];
        assert!(
            lines_calling_one_of(planted.as_slice(), LINKS_A_FILE_MAY_NOT_MAKE).is_empty(),
            "{reading} makes no link"
        );
    }
    for call in LINKS_A_FILE_MAY_NOT_MAKE {
        let commented: Vec<(String, String)> = vec![(
            String::from("commit/planted.rs"),
            format!("// a {call} here is explained and makes nothing\nfn fine() {{}}\n"),
        )];
        assert!(
            lines_calling_one_of(commented.as_slice(), LINKS_A_FILE_MAY_NOT_MAKE).is_empty(),
            "{call} in a comment"
        );
    }
}

/// A link a split carried into a receiver is a finding,
/// and a second link in a file `MAY_HOLD` excuses fails,
/// because the entry names the one use that file holds.
#[test]
fn the_ban_reads_a_receiver_of_the_split_and_counts_the_uses_one_entry_names() {
    let split: Vec<(String, String)> = vec![(
        String::from("landing/a_receiver_the_split_made.rs"),
        String::from(
            "fn planted(a: &std::path::Path, b: &std::path::Path) {\n    let _ = std::fs::hard_link(a, b);\n}\n",
        ),
    )];
    assert!(
        !lines_no_entry_excuses(
            lines_the_commit_path_may_not_hold(split.as_slice()).as_slice(),
            MAY_HOLD
        )
        .is_empty(),
        "a link in a file the ban's folders hold is a finding whatever the file is named"
    );
    let second_use: Vec<(String, String)> = vec![(
        String::from("commit/conclusion_copy.rs"),
        String::from(
            "fn copied(from: &std::path::Path, to: &std::path::Path) {\n    let _ = std::fs::symlink(from, to);\n    let _ = std::fs::symlink(from, to);\n}\n",
        ),
    )];
    let two: Vec<String> = lines_no_entry_excuses(
        lines_the_commit_path_may_not_hold(second_use.as_slice()).as_slice(),
        MAY_HOLD,
    );
    assert!(
        two.iter()
            .all(|sentence| return sentence.contains("use(s) of this text")),
        "a second link in an excused file fails, and the finding says what the file holds: {two:?}"
    );
    assert_eq!(two.len(), 2, "both uses are findings: {two:?}");
}
