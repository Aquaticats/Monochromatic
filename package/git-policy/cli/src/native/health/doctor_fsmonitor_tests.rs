//! Controls for the fsmonitor section of `git monochromatic-git-policy-cli doctor`:
//! parsing the listing,
//! the wording for each state,
//! and
//! the doctor's output for real repositories.
//! No control starts a fsmonitor daemon:
//! the doctor only lists the
//! configuration,
//! and every home and global file is a fixture.

use super::*;
use crate::action::Action;
use crate::doctor::doctor_action;
use crate::test_support::{fixture, git, git_context, remove, repository};
use std::ffi::OsString;
use std::path::PathBuf;

/// The fsmonitor section for `cwd`,
/// read by a task of the runtime,
/// because reading it asks Git.
fn fsmonitor_of(context: &GitContext, cwd: &Path) -> String {
    return crate::lazy_runtime::run_on_runtime(fsmonitor_report(context, cwd)).expect("the runtime");
}

/// A listing record as `git config --list --show-origin --show-scope -z` prints it.
fn record(scope: &str, origin: &str, key_and_value: &str) -> Vec<u8> {
    return format!("{scope}\0{origin}\0{key_and_value}\0").into_bytes();
}

/// Standard error of a doctor run that must exit with `code`.
fn doctor_stderr(context: &GitContext, repo: &Path, code: i32) -> String {
    let Action::Exit { code: actual, stderr, .. } = doctor_action(context, repo) else {
        panic!("doctor must end in an exit action");
    };
    assert_eq!(actual, code, "{stderr}");
    return stderr;
}

/// A context whose global configuration file is `file`.
fn context_with_global(file: &Path) -> GitContext {
    let mut context: GitContext = git_context();
    context.overlay.retain(|(name, _)| { return name != "GIT_CONFIG_GLOBAL"; });
    context.overlay.push((OsString::from("GIT_CONFIG_GLOBAL"), file.as_os_str().to_os_string()));
    return context;
}

/// The line doctor prints when the setting is off.
fn off_line(state: &str) -> String {
    return format!(
        "git monochromatic-git-policy-cli doctor: Git's fsmonitor is off by the owner's decision of {DECISION_DATE} ({state}). {FSMONITOR_REASON} See {FSMONITOR_DOCUMENT}.\n"
    );
}

/// The finding doctor prints when a scope turns the setting on.
fn on_finding(setting: &str, remedy: &str) -> String {
    return format!(
        "git monochromatic-git-policy-cli doctor: Git's fsmonitor is on, against the owner's decision of {DECISION_DATE} to keep it off: {setting}. {FSMONITOR_REASON} See {FSMONITOR_DOCUMENT}. {remedy}\n"
    );
}

/// Only entries of `core.fsmonitor` are kept,
/// in listing order,
/// with scope,
/// origin and value;
/// a key without `=` has
/// no value.
#[test]
fn the_listing_keeps_the_fsmonitor_entries_with_scope_origin_and_value() {
    let mut listing: Vec<u8> = Vec::new();
    listing.extend(record("system", "file:/etc/gitconfig", "core.bare\nfalse"));
    listing.extend(record("global", "file:/home/a/.gitconfig", "core.fsmonitor\ntrue"));
    listing.extend(record("local", "file:.git/config", "core.fsmonitor"));
    listing.extend(record("command", "command line:", "core.fsmonitor\n/hook\nwith newline"));
    let entries: Vec<FsmonitorEntry> = parse_fsmonitor_entries(listing.as_slice()).expect("a well-formed listing");
    assert_eq!(
        entries,
        vec![
            FsmonitorEntry { scope: "global".into(), origin: "file:/home/a/.gitconfig".into(), value: Some(b"true".to_vec()) },
            FsmonitorEntry { scope: "local".into(), origin: "file:.git/config".into(), value: None },
            FsmonitorEntry { scope: "command".into(), origin: "command line:".into(), value: Some(b"/hook\nwith newline".to_vec()) },
        ]
    );
    assert_eq!(parse_fsmonitor_entries(b"").expect("an empty listing"), Vec::new());
    assert_eq!(parse_fsmonitor_entries(b"global\0file:/x"), None, "a listing in another shape is refused whole");
}

/// No entry:
/// off,
/// and the line says the key is not set.
#[test]
fn an_unset_key_is_off_and_says_not_set() {
    assert_eq!(render_fsmonitor(&[]), off_line("core.fsmonitor is not set"));
}

/// The last entry decides:
/// a later `false` turns an earlier `true` off,
/// and the line names the entry that decided.
#[test]
fn the_last_entry_decides_and_false_is_off() {
    let entries: Vec<FsmonitorEntry> = vec![
        FsmonitorEntry { scope: "global".into(), origin: "file:/home/a/.gitconfig".into(), value: Some(b"true".to_vec()) },
        FsmonitorEntry { scope: "local".into(), origin: "file:.git/config".into(), value: Some(b"false".to_vec()) },
    ];
    assert_eq!(render_fsmonitor(entries.as_slice()), off_line("core.fsmonitor is false in file:.git/config, local scope"));
}

/// `true` in the global file is a finding that names the file,
/// the scope and the command for that scope.
#[test]
fn true_in_the_global_file_names_the_file_the_scope_and_the_command() {
    let entries: Vec<FsmonitorEntry> =
        vec![FsmonitorEntry { scope: "global".into(), origin: "file:/home/a/.gitconfig".into(), value: Some(b"true".to_vec()) }];
    assert_eq!(
        render_fsmonitor(entries.as_slice()),
        on_finding(
            "core.fsmonitor is true in file:/home/a/.gitconfig, global scope",
            "Turn it off with: git config --global core.fsmonitor false"
        )
    );
}

/// A hook path and a key without a value are on too,
/// and each scope has its own command flag.
#[test]
fn a_hook_path_or_a_valueless_key_is_on_and_each_scope_has_its_command() {
    let hook: Vec<FsmonitorEntry> =
        vec![FsmonitorEntry { scope: "system".into(), origin: "file:/etc/gitconfig".into(), value: Some(b"/opt/hook".to_vec()) }];
    assert_eq!(
        render_fsmonitor(hook.as_slice()),
        on_finding(
            "core.fsmonitor names the hook /opt/hook in file:/etc/gitconfig, system scope",
            "Turn it off with: git config --system core.fsmonitor false"
        )
    );
    let bare: Vec<FsmonitorEntry> =
        vec![FsmonitorEntry { scope: "worktree".into(), origin: "file:/r/.git/config.worktree".into(), value: None }];
    assert_eq!(
        render_fsmonitor(bare.as_slice()),
        on_finding(
            "core.fsmonitor is true (the key has no value) in file:/r/.git/config.worktree, worktree scope",
            "Turn it off with: git config --worktree core.fsmonitor false"
        )
    );
}

/// A value from `-c` or the environment has no file to change,
/// so the finding says where to remove it instead.
#[test]
fn a_command_scope_value_names_no_config_command() {
    let entries: Vec<FsmonitorEntry> =
        vec![FsmonitorEntry { scope: "command".into(), origin: "command line:".into(), value: Some(b"true".to_vec()) }];
    assert_eq!(
        render_fsmonitor(entries.as_slice()),
        on_finding(
            "core.fsmonitor is true in command line:, command scope",
            "No configuration file sets it: remove the `-c core.fsmonitor=...` option or the GIT_CONFIG_COUNT or GIT_CONFIG_PARAMETERS environment variable that passes it."
        )
    );
}

/// A repository that does not mention the key gets the off line and the exit status is unchanged.
#[test]
fn doctor_says_fsmonitor_is_off_for_a_repository_that_does_not_set_it() {
    let root: PathBuf = fixture("doctor-fsmonitor-unset");
    let repo: PathBuf = repository(root.as_path(), "repo");
    assert_eq!(doctor_stderr(&git_context(), repo.as_path(), 0), off_line("core.fsmonitor is not set"));
    remove(root.as_path());
}

/// `false` in the repository's own file is off,
/// and the line names that file's origin and scope.
#[test]
fn doctor_says_fsmonitor_is_off_when_the_repository_sets_false() {
    let root: PathBuf = fixture("doctor-fsmonitor-false");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["config", "core.fsmonitor", "false"]);
    assert_eq!(
        doctor_stderr(&git_context(), repo.as_path(), 0),
        off_line("core.fsmonitor is false in file:.git/config, local scope")
    );
    remove(root.as_path());
}

/// `true` in the repository's own file is a finding with the local command;
/// the exit status stays 0 because the
/// finding is informational,
/// like the retired-key notes.
#[test]
fn doctor_names_the_repository_file_that_turns_fsmonitor_on() {
    let root: PathBuf = fixture("doctor-fsmonitor-local-true");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["config", "core.fsmonitor", "true"]);
    assert_eq!(
        doctor_stderr(&git_context(), repo.as_path(), 0),
        on_finding(
            "core.fsmonitor is true in file:.git/config, local scope",
            "Turn it off with: git config --local core.fsmonitor false"
        )
    );
    remove(root.as_path());
}

/// `true` in the global file of an injected home is found there,
/// with the global command;
/// nothing writes the file.
#[test]
fn doctor_names_the_global_file_that_turns_fsmonitor_on_and_never_edits_it() {
    let root: PathBuf = fixture("doctor-fsmonitor-global-true");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let global: PathBuf = root.join("home-gitconfig");
    let text: &str = "[core]\n\tfsmonitor = true\n";
    std::fs::write(&global, text).expect("global file");
    let stderr: String = doctor_stderr(&context_with_global(global.as_path()), repo.as_path(), 0);
    assert_eq!(
        stderr,
        on_finding(
            format!("core.fsmonitor is true in file:{}, global scope", global.display()).as_str(),
            "Turn it off with: git config --global core.fsmonitor false"
        )
    );
    assert_eq!(std::fs::read_to_string(&global).expect("global file"), text);
    remove(root.as_path());
}

/// Once the global file says `false`,
/// doctor says it is off and names that file.
#[test]
fn doctor_names_the_global_file_that_holds_false() {
    let root: PathBuf = fixture("doctor-fsmonitor-global-false");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let global: PathBuf = root.join("home-gitconfig");
    std::fs::write(&global, "[core]\n\tfsmonitor = false\n").expect("global file");
    assert_eq!(
        doctor_stderr(&context_with_global(global.as_path()), repo.as_path(), 0),
        off_line(format!("core.fsmonitor is false in file:{}, global scope", global.display()).as_str())
    );
    remove(root.as_path());
}

/// `0`,
/// `off` and an empty value are false to Git,
/// and the line names the value as written and says how Git reads it.
#[test]
fn a_false_spelling_other_than_false_is_named_as_written() {
    for (written, shown) in [("0", "0 (Git reads it as false)"), ("off", "off (Git reads it as false)"), ("", "empty (Git reads it as false)")] {
        let entries: Vec<FsmonitorEntry> =
            vec![FsmonitorEntry { scope: "global".into(), origin: "file:/home/a/.gitconfig".into(), value: Some(written.as_bytes().to_vec()) }];
        assert_eq!(
            render_fsmonitor(entries.as_slice()),
            off_line(format!("core.fsmonitor is {shown} in file:/home/a/.gitconfig, global scope").as_str())
        );
    }
}

/// A Git that cannot start is said plainly in place of the line,
/// and so is a configuration Git refuses to list.
#[test]
fn a_listing_that_cannot_be_read_is_said_plainly() {
    let root: PathBuf = fixture("doctor-fsmonitor-unreadable");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut missing: GitContext = git_context();
    missing.real_git = root.join("no-such-git");
    let not_started: String = fsmonitor_of(&missing, repo.as_path());
    assert!(
        not_started.starts_with("git monochromatic-git-policy-cli doctor: could not read Git's fsmonitor setting: git config --list could not start: ")
            && not_started.ends_with('\n'),
        "{not_started}"
    );
    std::fs::write(repo.join(".git/config"), "[core\n").expect("a broken configuration");
    let refused: String = fsmonitor_of(&git_context(), repo.as_path());
    assert!(
        refused.starts_with("git monochromatic-git-policy-cli doctor: could not read Git's fsmonitor setting: git config --list failed: "),
        "{refused}"
    );
    remove(root.as_path());
}

/// What:
///  A `core.fsmonitor` of `00` is off,
/// as Git reads it,
/// and the line names the spelling as written.
/// Why:
///  Git parses this value as an integer first (`strtoimax` with base 0),
/// and only a value that integer reader
///  refuses is a hook path;
/// the section called unmodelled text a hook and reported an owner's-decision violation
///  that does not exist.
#[test]
fn a_false_spelled_as_a_number_is_off() {
    for written in ["00", "0x0", "-0", "+0", "0k"] {
        let entries: Vec<FsmonitorEntry> =
            vec![FsmonitorEntry { scope: "local".into(), origin: "file:.git/config".into(), value: Some(written.as_bytes().to_vec()) }];
        assert_eq!(
            render_fsmonitor(entries.as_slice()),
            off_line(format!("core.fsmonitor is {written} (Git reads it as false) in file:.git/config, local scope").as_str()),
            "{written}"
        );
    }
}

/// A hook path is still a hook path:
/// the integer reader refuses it and Git starts it.
#[test]
fn a_value_the_integer_reader_refuses_is_still_a_hook() {
    let entries: Vec<FsmonitorEntry> =
        vec![FsmonitorEntry { scope: "local".into(), origin: "file:.git/config".into(), value: Some(b"08".to_vec()) }];
    assert!(
        render_fsmonitor(entries.as_slice()).contains("core.fsmonitor names the hook 08 in file:.git/config, local scope"),
        "{}",
        render_fsmonitor(entries.as_slice())
    );
}

/// What:
///  The doctor says fsmonitor is off for a repository whose value Git reads as false through an integer spelling.
/// Why:
///  This is the whole defect at its own boundary:
/// real Git,
/// a real repository,
/// and the report the person reads.
#[test]
fn doctor_says_fsmonitor_is_off_for_a_numeric_false() {
    let root: PathBuf = fixture("doctor-fsmonitor-numeric-false");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["config", "core.fsmonitor", "00"]);
    assert_eq!(
        doctor_stderr(&git_context(), repo.as_path(), 0),
        off_line("core.fsmonitor is 00 (Git reads it as false) in file:.git/config, local scope")
    );
    remove(root.as_path());
}
