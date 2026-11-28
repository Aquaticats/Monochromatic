//! What:
//!  The step each `final-newline` hint names,
//!  run through the built executable in a fixture,
//!  with the bytes that result read back.
//! Why:
//!  The hints are chosen and worded where the events are rendered,
//!  and the controls of `policy::final_newline_hint_of_each_lifecycle_tests` read that rendering in-process,
//!  so a change that stopped a path-naming commit from correcting the ending would leave every one of them green.
//!  Each control here performs the step its hint names and reads the bytes the repository holds afterwards,
//!  which is the only check that the words and the program agree.
//!  The commit needs the wrapper's own service,
//!  which this fixture starts itself
//!  (`binary_support_fixture_service`),
//!  and never a manager of this account's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const fixed = run(['fix', '--', 'a.txt']); expect(headBlob('a.txt')).toEqual('a-edited\n');
//! ```

use super::commit::{
    COMMIT_BOUND_SECONDS,
    assert_nothing_left,
    git_text,
    keep_maintenance_in_the_foreground,
    remove_settled,
};
/// Import the shared fixtures and the bounded runner.
use super::support::{
    Fixture,
    Observed,
    bounded_for,
    fixture,
    git,
    observe_raw,
    porcelain,
    repository,
    stderr_of,
    without_wait_notices,
    wrapper_first_path,
};
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;

/// The file every control here leaves or corrects.
const THE_FILE: &str = "a.txt";

/// The bytes `a.txt` holds in the base commit.
const THE_BASE_BYTES: &[u8] = b"a-base\n";

/// The bytes `a.txt` holds when its edit ends with no line feed.
const WITHOUT_A_LINE_FEED: &[u8] = b"a-edited";

/// The bytes `a.txt` holds once exactly one line feed follows its content.
const WITH_ONE_LINE_FEED: &[u8] = b"a-edited\n";

/// The words of the hint an `add` and a direct check print,
///  which name the two steps that correct the file now and the commit that names it.
const THE_CORRECTION_NOW: &str = "git monochromatic-git-policy-cli fix -- <path>";
/// The staging the `add` and the direct check name after the fix.
const THE_STAGING_AFTER_IT: &str = "git add -- <path>";
/// The commit form the `add` and the direct check name,
///  which is the one this wrapper accepts.
const THE_COMMIT_THAT_NAMES_THE_FILE: &str = "`git commit -- <path>`";
/// The read-only selection the `add` and the direct check name as the exception.
const THE_READ_ONLY_SELECTION: &str =
    "a commit whose selection is read-only (`-p`, `-i` or `--include`) records";
/// The hint of a commit whose selection is read-only,
///  where it lands.
const THE_LANDED_HINT: &str = "this file's bytes are already in a commit";

/// A wrapped command with the commit bound.
fn wrapped(
    fixture: &Fixture,
    directory: &Path,
    arguments: &[&str],
) -> Command {
    let mut command: Command = bounded_for(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        COMMIT_BOUND_SECONDS,
    );
    command.current_dir(directory).args(arguments);
    return command;
}

/// Run one wrapped command to its end,
///  keeping its `hint` fields and dropping the lines a healthy wrapper prints about a wait.
/// Why:
///  `observe` removes every `hint` field so a control can compare an event's bytes with the
///  incumbent's,
///  and the hint is what this file reads,
///  so the reading is the raw one with only the wait notices taken out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run(fixture, directory, args) { const raw = observeRaw(wrapped(fixture, directory, args)); return { ...raw, stderr: withoutWaitNotices(raw.stderr) }; }
/// ```
fn run(
    fixture: &Fixture,
    directory: &Path,
    arguments: &[&str],
) -> Observed {
    let raw: Observed = observe_raw(&mut wrapped(fixture, directory, arguments), b"");
    return Observed {
        code: raw.code,
        stdout: without_wait_notices(raw.stdout.as_slice()),
        stderr: without_wait_notices(raw.stderr.as_slice()),
    };
}

/// The bytes a path holds in `HEAD`.
fn head_bytes(
    fixture: &Fixture,
    repo: &Path,
) -> Vec<u8> {
    let spec: String = format!("HEAD:{THE_FILE}");
    return git(fixture, repo, &["cat-file", "blob", spec.as_str()]).stdout;
}

/// The bytes the index holds for `THE_FILE`.
fn index_bytes(
    fixture: &Fixture,
    repo: &Path,
) -> Vec<u8> {
    let spec: String = format!(":{THE_FILE}");
    return git(fixture, repo, &["cat-file", "blob", spec.as_str()]).stdout;
}

/// Both streams of an observed command as text,
/// for a failure that says what Git or the wrapper said on either one.
fn both_streams(observed: &Observed) -> String {
    return format!(
        "code {:?} stdout: {}stderr: {}",
        observed.code,
        String::from_utf8_lossy(&observed.stdout),
        stderr_of(observed)
    );
}

/// The bytes `THE_FILE` holds in the worktree.
fn worktree_bytes(repo: &Path) -> Vec<u8> {
    return std::fs::read(repo.join(THE_FILE)).expect("the worktree copy");
}

/// A repository whose one commit holds `a.txt` with its line feed,
///  and whose worktree copy of it lost that line feed.
fn a_repository(
    fixture: &Fixture,
    configuration: Option<&str>,
) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    std::fs::write(repo.join(THE_FILE), THE_BASE_BYTES).expect("the base file");
    git(fixture, repo.as_path(), &["add", "--", THE_FILE]);
    git(
        fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=base"],
    );
    if let Some(text) = configuration {
        std::fs::write(repo.join("monochromatic-git-policy-cli.config.jsonc"), text)
            .expect("repository configuration");
    }
    std::fs::write(repo.join(THE_FILE), WITHOUT_A_LINE_FEED).expect("the edit");
    return repo;
}

/// The configuration that makes this policy block a commit.
const BLOCKING: &str = "{ \"policies\": { \"final-newline\": \"error\" } }\n";

/// An `add` of a file without its line feed prints the hint whose two steps,
///  run in the order it names them,
///  leave the worktree and the index holding exactly one line feed.
#[test]
fn the_add_hint_s_two_steps_leave_one_line_feed_in_the_worktree_and_the_index() {
    let fixture: Fixture = fixture("final-newline-hint-steps-add");
    let repo: PathBuf = a_repository(&fixture, None);
    let added: Observed = run(&fixture, repo.as_path(), &["add", "--", THE_FILE]);
    assert_eq!(added.code, Some(0), "{}", both_streams(&added));
    let stderr: String = stderr_of(&added);
    for wording in [
        THE_CORRECTION_NOW,
        THE_STAGING_AFTER_IT,
        THE_COMMIT_THAT_NAMES_THE_FILE,
        THE_READ_ONLY_SELECTION,
    ] {
        assert!(stderr.contains(wording), "{wording} in {stderr}");
    }
    let fixed: Observed = run(
        &fixture,
        repo.as_path(),
        &["monochromatic-git-policy-cli", "fix", "--", THE_FILE],
    );
    assert_eq!(fixed.code, Some(0), "{}", both_streams(&fixed));
    let staged: Observed = run(&fixture, repo.as_path(), &["add", "--", THE_FILE]);
    assert_eq!(staged.code, Some(0), "{}", both_streams(&staged));
    assert_eq!(worktree_bytes(repo.as_path()), WITH_ONE_LINE_FEED);
    assert_eq!(index_bytes(&fixture, repo.as_path()), WITH_ONE_LINE_FEED);
    assert_eq!(porcelain(&fixture, repo.as_path()), "M  a.txt\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A direct check reads the same worktree bytes an `add` stages,
///  so it gives the same hint,
///  and the commit it names lands bytes ending with one line feed.
#[test]
fn a_direct_check_gives_the_hint_whose_commit_corrects_the_bytes_it_read() {
    let fixture: Fixture = fixture("final-newline-hint-steps-check");
    let repo: PathBuf = a_repository(&fixture, None);
    let checked: Observed = run(
        &fixture,
        repo.as_path(),
        &["monochromatic-git-policy-cli", "check", "--", THE_FILE],
    );
    assert_eq!(checked.code, Some(0), "{}", both_streams(&checked));
    let reported: String = String::from_utf8_lossy(&checked.stdout).into_owned();
    assert!(
        reported.contains("\"trigger\":\"direct-check\""),
        "{reported}"
    );
    for wording in [
        THE_CORRECTION_NOW,
        THE_STAGING_AFTER_IT,
        THE_COMMIT_THAT_NAMES_THE_FILE,
        THE_READ_ONLY_SELECTION,
    ] {
        assert!(reported.contains(wording), "{wording} in {reported}");
    }
    let landed: Observed = run(
        &fixture,
        repo.as_path(),
        &["commit", "--message=checked", "--", THE_FILE],
    );
    assert_eq!(landed.code, Some(0), "{}", both_streams(&landed));
    assert_eq!(head_bytes(&fixture, repo.as_path()), WITH_ONE_LINE_FEED);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// The commit the hint names is the one form the wrapper accepts,
///  and it corrects the ending it records:
///  a commit that names no path is refused,
///  and a commit that names the file lands bytes ending with one line feed.
#[test]
fn the_commit_the_hint_names_is_the_form_that_corrects_the_ending() {
    let fixture: Fixture = fixture("final-newline-hint-steps-commit");
    let repo: PathBuf = a_repository(&fixture, None);
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    // The index holds the bytes, as it does after the add the hint is printed for.
    let staged: Observed = run(&fixture, repo.as_path(), &["add", "--", THE_FILE]);
    assert_eq!(staged.code, Some(0), "{}", both_streams(&staged));
    let refused: Observed = run(&fixture, repo.as_path(), &["commit", "--message=plain"]);
    assert_eq!(refused.code, Some(1), "{}", both_streams(&refused));
    let refusal: String = both_streams(&refused);
    assert!(
        refusal.contains("commit-only/pathspec-required"),
        "{refusal}"
    );
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]),
        head
    );
    let landed: Observed = run(
        &fixture,
        repo.as_path(),
        &["commit", "--message=named", "--", THE_FILE],
    );
    assert_eq!(landed.code, Some(0), "{}", both_streams(&landed));
    assert_eq!(head_bytes(&fixture, repo.as_path()), WITH_ONE_LINE_FEED);
    assert_eq!(worktree_bytes(repo.as_path()), WITH_ONE_LINE_FEED);
    // What: the correction is reported as the commit's own summary, never as a finding with a hint.
    // Why:  the pass a run reports is the one after the proposal, which holds no finding at all.
    assert!(
        stderr_of(&landed).contains("\"type\":\"fix-summary\",\"trigger\":\"pre-forward\""),
        "{}",
        stderr_of(&landed)
    );
    assert!(
        !stderr_of(&landed).contains("\"type\":\"finding\""),
        "{}",
        stderr_of(&landed)
    );
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A direct fix reports a correction of its own and never a finding,
///  and the worktree it leaves ends with exactly one line feed.
#[test]
fn a_direct_fix_reports_a_summary_and_leaves_one_line_feed() {
    let fixture: Fixture = fixture("final-newline-hint-steps-fix");
    let repo: PathBuf = a_repository(&fixture, Some(BLOCKING));
    let fixed: Observed = run(
        &fixture,
        repo.as_path(),
        &["monochromatic-git-policy-cli", "fix", "--", THE_FILE],
    );
    assert_eq!(fixed.code, Some(0), "{}", both_streams(&fixed));
    // What: a direct command reports its events on standard output, and a correcting one reports a summary alone.
    // Why:  a lifecycle that applies its own correction reports the pass after the proposal,
    //       which holds no finding, so no hint of it reaches a person.
    let reported: String = String::from_utf8_lossy(&fixed.stdout).into_owned();
    assert!(
        reported.contains("\"type\":\"fix-summary\",\"trigger\":\"direct-fix\""),
        "{reported}"
    );
    assert!(!reported.contains("\"type\":\"finding\""), "{reported}");
    assert_eq!(worktree_bytes(repo.as_path()), WITH_ONE_LINE_FEED);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// The read-only selection the hint names records the bytes it takes exactly as they are,
///  and the step the hint of that finding tells the person,
///  editing the file and then staging it again,
///  is what puts a corrected copy in the next commit.
#[test]
fn a_read_only_selection_records_the_bytes_it_takes_until_the_file_is_staged_again() {
    let fixture: Fixture = fixture("final-newline-hint-steps-read-only");
    let repo: PathBuf = a_repository(&fixture, None);
    let landed: Observed = run(
        &fixture,
        repo.as_path(),
        &["commit", "--include", "--message=include", "--", THE_FILE],
    );
    assert_eq!(landed.code, Some(0), "{}", both_streams(&landed));
    assert_eq!(
        head_bytes(&fixture, repo.as_path()),
        WITHOUT_A_LINE_FEED,
        "a read-only selection records what it takes"
    );
    // What: the run says what the landed commit holds, once, and never the step for bytes still in the index.
    // Why:  the pass that read the index ran before the commit was made, and its sentence is stale here.
    let stderr: String = stderr_of(&landed);
    assert!(stderr.contains(THE_LANDED_HINT), "{stderr}");
    assert_eq!(
        stderr.matches("\"type\":\"finding\"").count(),
        1,
        "one finding for one file, not the staged sentence beside the landed one: {stderr}"
    );
    std::fs::write(repo.join(THE_FILE), WITH_ONE_LINE_FEED).expect("the correction");
    let staged: Observed = run(&fixture, repo.as_path(), &["add", "--", THE_FILE]);
    assert_eq!(staged.code, Some(0), "{}", both_streams(&staged));
    let corrected: Observed = run(
        &fixture,
        repo.as_path(),
        &["commit", "--message=corrected", "--", THE_FILE],
    );
    assert_eq!(corrected.code, Some(0), "{}", both_streams(&corrected));
    assert_eq!(head_bytes(&fixture, repo.as_path()), WITH_ONE_LINE_FEED);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A commit whose selection is read-only and which does not land tells the person to edit the file and stage it again,
///  and those two steps land the corrected bytes.
#[test]
fn the_read_only_selection_that_does_not_land_says_to_edit_and_stage_again() {
    let fixture: Fixture = fixture("final-newline-hint-steps-blocked");
    let repo: PathBuf = a_repository(&fixture, Some(BLOCKING));
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    // The index holds the bytes, as it does after the selection a refused commit leaves behind.
    let staged: Observed = run(&fixture, repo.as_path(), &["add", "--", THE_FILE]);
    assert_eq!(staged.code, Some(1), "{}", stderr_of(&staged));
    git(&fixture, repo.as_path(), &["add", "--", THE_FILE]);
    let blocked: Observed = run(
        &fixture,
        repo.as_path(),
        &["commit", "--include", "--message=blocked", "--", THE_FILE],
    );
    assert_eq!(blocked.code, Some(1), "{}", both_streams(&blocked));
    let stderr: String = stderr_of(&blocked);
    assert!(
        stderr.contains("edit it so it ends with exactly one newline, then stage it again"),
        "{stderr}"
    );
    assert!(!stderr.contains(THE_LANDED_HINT), "{stderr}");
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]),
        head
    );
    std::fs::write(repo.join(THE_FILE), WITH_ONE_LINE_FEED).expect("the correction");
    let restaged: Observed = run(&fixture, repo.as_path(), &["add", "--", THE_FILE]);
    assert_eq!(restaged.code, Some(0), "{}", both_streams(&restaged));
    let landed: Observed = run(
        &fixture,
        repo.as_path(),
        &["commit", "--message=corrected", "--", THE_FILE],
    );
    assert_eq!(landed.code, Some(0), "{}", both_streams(&landed));
    assert_eq!(head_bytes(&fixture, repo.as_path()), WITH_ONE_LINE_FEED);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}
