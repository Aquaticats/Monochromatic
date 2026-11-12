//! Controls for the record of a failed backup push:
//! its text,
//! its masking,
//! its names,
//! and which commands show it.

use super::*;
use std::ffi::OsString;

/// A function-like value with the right shape is a namer in these controls:
/// the shipped caller writes a small struct that holds the state.
impl<Name: Fn(&str) -> String> CommitNamer for Name {
    fn name_of(&self, commit: &str) -> String {
        return self(commit);
    }
}
use crate::test_support::{fixture, remove};
/// The line every failed-push notice ends with.
use crate::auto_push::PUSH_FAILED_NOTE;

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER_COMMIT: &str = "89abcdef0123456789abcdef0123456789abcdef";
const BRANCH: &str = "refs/heads/main";

fn region(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// What:
///  The words of every record,
/// shown to a buffer and removed as a command shows and removes them.
/// Why:
///   A record goes only once its words are out,
/// so every control that read a record through the old take must
///  show it through the step the person's terminal receives it by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shownNotices(pushFailures(directory))
/// ```
fn take_push_failures(directory: &Path) -> Vec<String> {
    return crate::test_support::shown_notices(push_failures(directory));
}

/// `take_push_failures` for the records of one asking worktree.
fn take_push_failures_for(directory: &Path, worktrees: Option<&Worktrees>, abbreviate: &dyn CommitNamer) -> Vec<String> {
    return crate::test_support::shown_notices(push_failures_for(directory, worktrees, abbreviate));
}

/// `take_push_failures` for the records of removed worktrees.
fn take_orphaned_failures(directory: &Path, abbreviate: &dyn CommitNamer) -> Vec<String> {
    return crate::test_support::shown_notices(orphaned_push_failures(directory, abbreviate));
}

/// The text names the branch,
/// the commit,
/// Git's own reason,
/// that the commit is saved here,
/// and what to do.
#[test]
fn the_text_names_the_branch_the_commit_the_reason_and_the_remedy() {
    let said: &str = "! [rejected] main -> main (non-fast-forward)";
    let text: String = push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, said);
    assert!(text.contains("branch main"), "{text}");
    assert!(text.contains(COMMIT), "{text}");
    assert!(text.contains(said), "{text}");
    assert!(text.contains("saved in this repository"), "{text}");
    assert!(text.contains("`git push`"), "{text}");
    assert!(text.contains("`git pull`"), "{text}");
    assert!(text.ends_with(PUSH_FAILED_NOTE), "{text}");
}

/// A skipped push is worded as not made,
/// with the service's own reason,
/// and never as a refusal by the remote;
/// it is shown
///  once and a later successful push of the branch removes it with the failures.
#[test]
fn a_skipped_push_is_worded_as_not_made_and_cleared_by_a_later_push() {
    let directory: PathBuf = fixture("notice-skipped");
    let reason: &str = "monochromatic-git-policy-cli: the branch refs/heads/main tracks no remote branch that can be named; skipping auto-push.";
    record_push_skip(directory.as_path(), BRANCH, COMMIT, reason).expect("recorded");
    record_push_skip(directory.as_path(), "refs/heads/other", OTHER_COMMIT, reason).expect("recorded");
    clear_push_failures(directory.as_path(), BRANCH);
    let shown: Vec<String> = take_push_failures(directory.as_path());
    assert_eq!(shown.len(), 1, "only the other branch's record is left: {shown:?}");
    let text: &str = shown[0].as_str();
    for expected in ["branch other", OTHER_COMMIT, reason, "was not made", "saved in this repository", "`git push <remote> other`", "push.default"] {
        assert!(text.contains(expected), "missing {expected:?} in: {text}");
    }
    assert!(!text.contains("failed") && !text.contains("Git said"), "a skip is not a refusal: {text}");
    remove(directory.as_path());
}

/// A successful push removes the records written before it began and keeps one written after:
/// that one is about a commit the
///  push could not have carried.
#[test]
fn a_successful_push_keeps_a_record_written_after_it_began() {
    let directory: PathBuf = fixture("notice-cutoff");
    record_push_failure(directory.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "remote: older").expect("recorded");
    record_push_failure(directory.as_path(), BRANCH, OTHER_COMMIT, PushOutcome::Refused, "remote: newer").expect("recorded");
    let began: std::time::SystemTime = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
    let touch = |commit: &str, written: std::time::SystemTime| {
        let path: PathBuf = notice_directory(directory.as_path()).join(record_name(BRANCH, commit));
        std::fs::File::options().append(true).open(path).and_then(|file| { return file.set_modified(written); }).expect("aged");
    };
    touch(COMMIT, began - std::time::Duration::from_secs(60));
    touch(OTHER_COMMIT, began + std::time::Duration::from_secs(60));
    clear_push_failures_before(directory.as_path(), BRANCH, began);
    let shown: Vec<String> = take_push_failures(directory.as_path());
    assert_eq!(shown, vec![push_failure_text(BRANCH, OTHER_COMMIT, PushOutcome::Refused, "remote: newer")]);
    remove(directory.as_path());
}

/// A reason that Git left empty is said so,
/// rather than printing an empty quotation.
#[test]
fn an_empty_reason_is_said_plainly() {
    let text: String = push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "  \n");
    assert!(text.contains(NO_REASON), "{text}");
    assert!(!text.contains("Git said:"), "{text}");
}

/// A remote URL's user name and password never reach the text,
/// including in a line that names the Git LFS endpoint
///  setting;
/// the line itself stays,
/// with its host.
#[test]
fn credentials_are_masked_and_the_lfs_endpoint_line_stays() {
    let said: &str = "fatal: unable to access 'https://user:s3cret@example.invalid/repo.git/': 403\n\
                      error: lfs.url=https://tok3n@lfs.example.invalid/x is not allowed\n\
                      remote: denied";
    let text: String = push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, said);
    for secret in ["s3cret", "tok3n", "user:"] {
        assert!(!text.contains(secret), "{secret} leaked: {text}");
    }
    assert!(text.contains(mask_url_secrets_text("https://user:s3cret@example.invalid/").as_str()), "{text}");
    assert!(text.contains("lfs.url=https://<credentials>@lfs.example.invalid/x is not allowed"), "{text}");
    assert!(text.contains("remote: denied"), "{text}");
}

/// The wrapper's own failure line for a Git LFS push,
/// which names the endpoint setting in prose and carries the exit status,
///  reaches the person whole.
#[test]
fn the_wrappers_own_lfs_failure_line_keeps_its_status() {
    let said: String = format!("batch response: 401\n{}", crate::lfs_pre_push::failed_message("exited with status 2"));
    let text: String = push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, said.as_str());
    assert!(text.contains("exited with status 2"), "{text}");
    assert!(text.contains("(`lfs.url`)"), "{text}");
    assert!(text.contains("batch response: 401"), "{text}");
}

/// Every URL shape that can carry a credential has it masked and keeps what is not a credential.
#[test]
fn userinfo_is_masked_in_every_shape_a_url_takes() {
    // (line said by Git, text that must be gone, text that must stay)
    let shapes: [(&str, &str, &str); 8] = [
        ("lfs.url=https://user:p@ss@host.invalid/x", "p@ss", "@host.invalid/x"),
        ("lfs.url=https://user:p%40ss%3Aw@host.invalid/x", "p%40ss", "@host.invalid/x"),
        ("lfs.url=https://us%40er@host.invalid/x", "us%40er", "@host.invalid/x"),
        ("lfs.url=https://user:pw@[2001:db8::1]:8443/path", "user:pw", "@[2001:db8::1]:8443/path"),
        ("lfs.url=https://[2001:db8::1]:8443/path@segment", "path@segment", "https://[2001:db8::1]:8443/<credentials>"),
        ("tried https://u1:s1@one.invalid/a then https://u2:s2@two.invalid/b and http://plain.invalid/c", "s1", "http://plain.invalid/c"),
        ("lfs.url=https://host.invalid/path?next=a@b#frag@x", "a@b", "https://host.invalid/path?next=<credentials>#<credentials>"),
        ("lfs.url=ssh://git:token@host.invalid:22/repo", "token", "@host.invalid:22/repo"),
    ];
    for (line, gone, stays) in shapes {
        let cleaned: String = clean_reason(line);
        assert!(!cleaned.contains(gone), "{gone} leaked from {line}: {cleaned}");
        assert!(cleaned.contains(stays), "{stays} missing from {cleaned}");
        assert!(cleaned.contains("lfs.url") || cleaned.starts_with("tried"), "the line was dropped: {cleaned}");
    }
    let several: String = clean_reason(shapes[5].0);
    assert!(!several.contains("s2") && !several.contains("u1:") && !several.contains("u2:"), "{several}");
    assert!(several.contains("one.invalid/a") && several.contains("two.invalid/b"), "{several}");
}

/// Control characters from a remote's words cannot move the cursor or recolour the terminal;
/// tabs and lines stay.
#[test]
fn control_characters_are_dropped_and_long_output_is_cut() {
    let cleaned: String = clean_reason("remote: \u{1b}[31mred\u{1b}[0m\u{7}\n\tindented");
    assert_eq!(cleaned, "remote: [31mred[0m\n\tindented");
    let long: String = "x".repeat(LONGEST_REASON * 2);
    let cut: String = clean_reason(long.as_str());
    assert!(cut.len() < LONGEST_REASON + 100, "{}", cut.len());
    assert!(cut.ends_with("(Git's output is cut here.)"));
}

/// The record name belongs to its branch;
/// one branch's name is never the start of another's,
/// and a very long branch
///  name still makes a file name the file system accepts.
#[test]
fn record_names_separate_branches_and_stay_valid_file_names() {
    let short: String = record_name("refs/heads/a", COMMIT);
    let longer: String = record_name("refs/heads/a-b", COMMIT);
    assert!(!longer.starts_with(branch_prefix("refs/heads/a").as_str()), "{short} / {longer}");
    let huge: String = format!("refs/heads/{}", "feature/".repeat(60));
    assert!(record_name(huge.as_str(), COMMIT).len() < 255);
    assert_ne!(record_name(huge.as_str(), COMMIT), record_name(format!("{huge}x").as_str(), COMMIT));
}

/// A failure is taken once;
/// a second take finds nothing,
/// and a repeat of the same failure leaves one record.
#[test]
fn a_record_is_taken_once_and_a_repeat_leaves_one() {
    let root: PathBuf = fixture("push-failure-once");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "first").expect("record");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "again").expect("record");
    let shown: Vec<String> = take_push_failures(root.as_path());
    assert_eq!(shown, vec![push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "again")]);
    assert!(take_push_failures(root.as_path()).is_empty(), "a record is shown once");
    remove(root.as_path());
}

/// A repository where nothing failed has no notice directory,
/// and taking from it shows nothing and creates nothing.
#[test]
fn nothing_failed_means_nothing_to_read_and_nothing_created() {
    let root: PathBuf = fixture("push-failure-none");
    assert!(take_push_failures(root.as_path()).is_empty());
    clear_push_failures(root.as_path(), BRANCH);
    assert!(!notice_directory(root.as_path()).exists());
    remove(root.as_path());
}

/// Clearing a branch removes that branch's records and no other branch's.
#[test]
fn clearing_a_branch_keeps_the_records_of_other_branches() {
    let root: PathBuf = fixture("push-failure-clear");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "main failed").expect("record");
    record_push_failure(root.as_path(), "refs/heads/main-two", OTHER_COMMIT, PushOutcome::Refused, "two failed").expect("record");
    clear_push_failures(root.as_path(), BRANCH);
    let shown: Vec<String> = take_push_failures(root.as_path());
    assert_eq!(shown, vec![push_failure_text("refs/heads/main-two", OTHER_COMMIT, PushOutcome::Refused, "two failed")]);
    remove(root.as_path());
}

/// Plumbing and the forms that ask for output made for a program are machine-read;
/// ordinary human commands are not.
#[test]
fn machine_read_commands_are_told_from_human_ones() {
    for command in ["rev-parse", "cat-file", "for-each-ref", "ls-files", "config", "upload-pack", "update-ref", "diff-index"] {
        assert!(is_machine_read(command.as_bytes(), &[]), "{command}");
    }
    for words in [&["--porcelain"][..], &["--porcelain=v2", "-b"], &["-z"], &["--null"], &["--format=%H"], &["--pretty=format:%h"], &["-s", "-z"]] {
        assert!(is_machine_read(b"status", &region(words)), "{words:?}");
    }
    for (command, words) in [("status", &[][..]), ("log", &["--oneline"]), ("diff", &["--stat"]), ("branch", &[]), ("pull", &[])] {
        assert!(!is_machine_read(command.as_bytes(), &region(words)), "{command} {words:?}");
    }
    // After the separator a word is a name, never an option.
    assert!(!is_machine_read(b"log", &region(&["--", "-z"])));
}

/// The record names the commit as the asker's abbreviation gives it,
/// and the text does not call it the reader's own.
#[test]
fn a_record_is_worded_with_the_askers_abbreviation() {
    let root: PathBuf = fixture("push-failure-abbreviated");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "rejected").expect("record");
    let shown: Vec<String> = take_push_failures_for(root.as_path(), None, &|commit: &str| { return commit[..12].to_string(); });
    assert_eq!(shown, vec![push_failure_text(BRANCH, &COMMIT[..12], PushOutcome::Refused, "rejected")]);
    assert!(!shown[0].contains("your commit"), "{}", shown[0]);
    assert!(!shown[0].contains(COMMIT), "{}", shown[0]);
    remove(root.as_path());
}

/// A record is for the worktree that has its branch checked out;
/// another worktree leaves it,
/// and when no worktree has the
///  branch any command takes it.
#[test]
fn a_record_waits_for_the_worktree_that_has_its_branch() {
    let root: PathBuf = fixture("push-failure-worktrees");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "rejected").expect("record");
    record_push_failure(root.as_path(), "refs/heads/other", OTHER_COMMIT, PushOutcome::Refused, "rejected too").expect("record");
    let all: Vec<String> = vec![BRANCH.to_string(), "refs/heads/other".to_string()];
    let identity = |commit: &str| { return commit.to_string(); };
    let elsewhere: Worktrees = Worktrees { here: Some(String::from("refs/heads/unrelated")), all: all.clone() };
    assert!(take_push_failures_for(root.as_path(), Some(&elsewhere), &identity).is_empty(), "both branches are held by other worktrees");
    assert_eq!(std::fs::read_dir(notice_directory(root.as_path())).expect("directory").count(), 2, "nothing was taken");
    let detached: Worktrees = Worktrees { here: None, all: all.clone() };
    assert!(take_push_failures_for(root.as_path(), Some(&detached), &identity).is_empty());
    let owner: Worktrees = Worktrees { here: Some(BRANCH.to_string()), all: all.clone() };
    assert_eq!(take_push_failures_for(root.as_path(), Some(&owner), &identity), vec![push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "rejected")]);
    let nobody_holds: Worktrees = Worktrees { here: Some(String::from("refs/heads/unrelated")), all: vec![BRANCH.to_string()] };
    assert_eq!(
        take_push_failures_for(root.as_path(), Some(&nobody_holds), &identity),
        vec![push_failure_text("refs/heads/other", OTHER_COMMIT, PushOutcome::Refused, "rejected too")],
        "a branch no worktree holds is shown to anyone"
    );
    remove(root.as_path());
}

/// A record an earlier release wrote,
/// which is finished text,
/// is still shown,
/// to anyone,
/// and once.
#[test]
fn a_record_of_an_earlier_release_is_shown_as_it_was_written() {
    let root: PathBuf = fixture("push-failure-earlier");
    let directory: PathBuf = notice_directory(root.as_path());
    std::fs::create_dir_all(&directory).expect("directory");
    std::fs::write(directory.join(record_name(BRANCH, COMMIT)), "The backup push of branch main failed for your commit 0123456.\n").expect("record");
    let nowhere: Worktrees = Worktrees { here: Some(String::from("refs/heads/unrelated")), all: vec![BRANCH.to_string()] };
    let shown: Vec<String> = take_push_failures_for(root.as_path(), Some(&nowhere), &|commit: &str| { return commit.to_string(); });
    assert_eq!(shown, vec![String::from("The backup push of branch main failed for your commit 0123456.\n")]);
    assert!(take_push_failures(root.as_path()).is_empty());
    remove(root.as_path());
}

/// Found by the audit of the names the rename changed:
/// the three magic first lines of the records name a format,
/// and the records
///  an earlier release wrote carry its spelling.
/// They are read as the records they are (worded for the asking worktree,
/// held
///  for the worktree that has the branch),
/// not shown as raw text with their first line.
#[test]
fn a_record_of_an_earlier_release_with_its_own_first_line_is_read_as_a_record() {
    let root: PathBuf = fixture("push-failure-earlier-magic");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "rejected").expect("record");
    record_push_skip(root.as_path(), "refs/heads/other", OTHER_COMMIT, "no remote branch").expect("record");
    record_lost_push(root.as_path(), "refs/heads/third", COMMIT).expect("record");
    let orphans: PathBuf = root.join(ORPHAN_NOTICE_DIRECTORY);
    for entry in std::fs::read_dir(notice_directory(root.as_path())).expect("directory").flatten().chain(std::fs::read_dir(&orphans).expect("orphans").flatten()) {
        let text: String = std::fs::read_to_string(entry.path()).expect("record text");
        let (magic, rest) = text.split_once('\n').expect("a first line");
        let earlier: String = magic.replacen("monochromatic-git-policy-cli ", "cli-git ", 1);
        assert_ne!(earlier, magic, "the record names its format by the current spelling: {magic}");
        std::fs::write(entry.path(), format!("{earlier}\n{rest}")).expect("rewritten with the earlier first line");
    }
    let identity = |commit: &str| { return commit.to_string(); };
    let owner: Worktrees = Worktrees { here: Some(BRANCH.to_string()), all: vec![BRANCH.to_string(), "refs/heads/other".to_string(), "refs/heads/third".to_string()] };
    assert_eq!(take_push_failures_for(root.as_path(), Some(&owner), &identity), vec![push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "rejected")], "only the record of the asking worktree's branch, worded");
    let mut rest: Vec<String> = take_push_failures(root.as_path());
    rest.extend(take_orphaned_failures(root.as_path(), &identity));
    rest.sort();
    assert_eq!(rest.len(), 2, "{rest:?}");
    assert!(rest.iter().all(|text| { return !text.contains("cli-git push"); }), "no raw first line is shown: {rest:?}");
    remove(root.as_path());
}

/// The credentials in Git's words never reach the file the service writes.
#[test]
fn the_record_on_disk_holds_no_credential() {
    let root: PathBuf = fixture("push-failure-disk");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "fatal: unable to access 'https://user:s3cret@example.invalid/x'").expect("record");
    for entry in std::fs::read_dir(notice_directory(root.as_path())).expect("directory").flatten() {
        let text: String = std::fs::read_to_string(entry.path()).expect("record");
        assert!(!text.contains("s3cret") && !text.contains("user:"), "{text}");
    }
    remove(root.as_path());
}

/// Only the plain `ref:` line of a `HEAD` file names a branch,
/// in the main worktree and in each linked one.
#[test]
fn the_branches_of_the_worktrees_are_read_from_their_head_files() {
    let root: PathBuf = fixture("push-failure-heads");
    std::fs::write(root.join("HEAD"), "ref: refs/heads/main\n").expect("main head");
    assert_eq!(branch_of_head(root.as_path()), Some(String::from("refs/heads/main")));
    for (name, head) in [("linked-a", "ref: refs/heads/feature/a\n"), ("detached", "0123456789abcdef0123456789abcdef01234567\n"), ("tag", "ref: refs/tags/v1\n")] {
        let directory: PathBuf = root.join("worktrees").join(name);
        std::fs::create_dir_all(&directory).expect("directory");
        std::fs::write(directory.join("HEAD"), head).expect("head");
    }
    let mut branches: Vec<String> = checked_out_branches(root.as_path());
    branches.sort();
    assert_eq!(branches, vec![String::from("refs/heads/feature/a"), String::from("refs/heads/main")]);
    assert_eq!(branch_of_head(root.join("missing").as_path()), None);
    remove(root.as_path());
}

/// A notice is still on disk while its commit is being named,
/// so a command killed there,
/// or one whose standard error cannot be written,
/// leaves the only durable statement that the commit is not backed up for the next command.
#[test]
fn a_notice_survives_being_worded() {
    let root: PathBuf = fixture("push-failure-word-then-remove");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "rejected").expect("record");
    let record: PathBuf = notice_directory(root.as_path()).join(record_name(BRANCH, COMMIT));
    let naming = |commit: &str| {
        // The record is claimed under a name only this command holds while its words are made; whichever of the two exists,
        //  the notice is still on disk and a command killed here loses nothing.
        let held: usize = std::fs::read_dir(notice_directory(root.as_path())).map_or(0, |entries| { return entries.flatten().count(); });
        assert_eq!(held, 1, "the notice is on disk while its commit is named");
        let _ = &record;
        return commit.to_string();
    };
    let shown: Vec<String> = take_push_failures_for(root.as_path(), None, &naming);
    assert_eq!(shown, vec![push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "rejected")]);
    assert!(!record.exists(), "and it is gone once its words exist");
    assert!(take_push_failures(root.as_path()).is_empty(), "a record is shown once");
    remove(root.as_path());
}

/// What:
///  Two commands that read a record before either showed it show it once,
/// because only one of them wins the claim on it.
/// Why:
///   A record is shown once,
/// and a command that finds the record already claimed by another shows nothing,
/// rather
///  than printing words the other command is printing.
#[test]
fn two_commands_that_read_a_record_show_it_once() {
    let root: PathBuf = fixture("push-failure-one-claim");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "rejected").expect("record");
    let first: Vec<Notice> = push_failures(root.as_path());
    let second: Vec<Notice> = push_failures(root.as_path());
    assert_eq!(first.len(), 1, "the first command holds the record");
    assert!(second.is_empty(), "the second finds it claimed and shows nothing");
    assert_eq!(crate::test_support::shown_notices(first), vec![push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "rejected")]);
    assert!(notice_directory(root.as_path()).join(record_name(BRANCH, COMMIT)).exists() == false, "the claim went with the words");
    remove(root.as_path());
}

/// What:
///  A record the service writes in the instant before a claim is removed stays,
/// and the next command shows it.
/// Why:
///   The claim is this command's own name,
/// so removing it cannot delete a record the service wrote meanwhile;
/// the
///  service's statement is a different file.
#[test]
fn a_record_written_in_the_instant_before_removal_stays() {
    let root: PathBuf = fixture("push-failure-written-then-removed");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "rejected for now").expect("record");
    let recording: PathBuf = root.to_path_buf();
    let _played = crate::push_failure_notice_shown_then_removed::before_removal_in_a_control(move || {
        record_push_failure(recording.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "the remote wants a login").expect("the newer record");
    });
    let shown: Vec<String> = take_push_failures(root.as_path());
    drop(_played);
    assert_eq!(shown, vec![push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "rejected for now")], "the words are the ones this command took");
    let record: PathBuf = notice_directory(root.as_path()).join(record_name(BRANCH, COMMIT));
    assert!(record.exists(), "the record the service wrote as the claim went stays");
    let next: Vec<String> = take_push_failures(root.as_path());
    assert_eq!(next, vec![push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "the remote wants a login")], "and the next command shows it");
    remove(root.as_path());
}

/// What:
///  A claim left by a command that is gone is taken over,
/// so the notice reaches the person.
/// Why:
///   A command killed between taking a record and showing it must not take the person's only word about the commit with
///  it,
/// and a claim whose holder is gone is the next command's to show.
#[test]
fn a_claim_left_by_a_command_that_is_gone_is_taken_over() {
    let root: PathBuf = fixture("push-failure-claim-orphan");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "rejected").expect("record");
    // The claim of a command that no longer runs: no process has this number, and its start time is none this kernel made.
    let directory: PathBuf = notice_directory(root.as_path());
    let record: PathBuf = directory.join(record_name(BRANCH, COMMIT));
    let claim: PathBuf = directory.join(format!("{}.4294967294~1~gone.claim", record_name(BRANCH, COMMIT)));
    std::fs::rename(record.as_path(), claim.as_path()).expect("the claim a killed command left");
    let shown: Vec<String> = take_push_failures(root.as_path());
    assert_eq!(shown, vec![push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "rejected")], "the claim of a dead command is shown");
    assert_eq!(std::fs::read_dir(directory).expect("directory").count(), 0, "its claim went with the words");
    remove(root.as_path());
}

/// What:
///  Taking over a claim whose command is gone never replaces the record this command claimed first,
/// when both are statements about the same branch and commit.
/// Why:
///   One name for every claim of a command made the take-over compute the name the first claim already holds,
/// and the
///  rename replaced the newer statement with the older one on disk;
/// a command killed before it printed would leave only
///  the obsolete words for the next command to find.
#[test]
fn taking_over_an_abandoned_claim_keeps_the_record_it_claimed_first() {
    let root: PathBuf = fixture("push-failure-claim-takeover");
    let directory: PathBuf = notice_directory(root.as_path());
    std::fs::create_dir_all(directory.as_path()).expect("directory");
    // The newer statement, under the record's own name.
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "the remote wants a login").expect("record");
    // The older statement of the same branch and commit, under a claim whose command is gone.
    let older: PathBuf = directory.join(format!("{}.4294967294~1~gone~0.claim", record_name(BRANCH, COMMIT)));
    std::fs::write(older.as_path(), format!("{RECORD_MAGIC}\n{BRANCH_FIELD}{BRANCH}\n{COMMIT_FIELD}{COMMIT}\n\nrejected for now\n")).expect("the older claim");
    let notices: Vec<Notice> = push_failures(root.as_path());
    // Nothing has been shown yet: what is on the disk is what a later command finds if this one is killed before it prints.
    let on_disk: String = std::fs::read_dir(directory.as_path())
        .expect("directory")
        .flatten()
        .map(|entry| { return std::fs::read_to_string(entry.path()).unwrap_or_default(); })
        .collect();
    assert!(on_disk.contains("the remote wants a login"), "the newer statement is still on disk under this command's claim: {on_disk}");
    assert!(on_disk.contains("rejected for now"), "and the older one is held too: {on_disk}");
    let shown: Vec<String> = crate::test_support::shown_notices(notices);
    assert_eq!(shown.len(), 2, "both statements are shown: {shown:?}");
    assert!(shown.contains(&push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "the remote wants a login")), "the newer statement is shown: {shown:?}");
    assert!(shown.contains(&push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "rejected for now")), "and the older one: {shown:?}");
    assert_eq!(std::fs::read_dir(directory).expect("directory").count(), 0, "the claims went with their words");
    remove(root.as_path());
}

/// What:
///  A notice given back after its words could not be written never replaces a newer record of the same name.
/// Why:
///   `rename` replaces its destination,
/// so giving the older claim back over a terminal failure's record deleted that
///  record,
/// and the next command read the obsolete statement that promised a retry.
#[test]
fn a_notice_given_back_keeps_a_newer_record_of_the_same_name() {
    let root: PathBuf = fixture("push-failure-give-back");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "rejected for now").expect("record");
    let notices: Vec<Notice> = push_failures(root.as_path());
    assert_eq!(notices.len(), 1);
    // The service writes the terminal failure while the claim is held, and then the words cannot be written.
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "the remote wants a login").expect("the newer record");
    assert!(notices[0].show(&mut ClosedTerminal).is_err(), "the terminal refused the words");
    let record: PathBuf = notice_directory(root.as_path()).join(record_name(BRANCH, COMMIT));
    let held: String = std::fs::read_to_string(record.as_path()).expect("the record's own name");
    assert!(held.contains("the remote wants a login"), "the newer statement is still under its name: {held}");
    remove(root.as_path());
}

/// What:
///  A notice whose words cannot be written out stays for the next command.
/// Why:
///   A record removed before its words reach the person is gone with nothing shown,
/// and the next command cannot
///  find it;
/// a standard error that nobody reads is what that looks like.
#[test]
fn a_notice_whose_words_cannot_be_written_stays() {
    let root: PathBuf = fixture("push-failure-unwritten");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "rejected").expect("record");
    let record: PathBuf = notice_directory(root.as_path()).join(record_name(BRANCH, COMMIT));
    let notices: Vec<Notice> = push_failures(root.as_path());
    assert_eq!(notices.len(), 1);
    assert!(notices[0].show(&mut ClosedTerminal).is_err(), "the terminal refused the words");
    assert!(record.exists(), "the record is kept for the next command");
    let shown: Vec<String> = take_push_failures(root.as_path());
    assert_eq!(shown, vec![push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "rejected")], "the next command shows it");
    assert!(!record.exists(), "and takes it once its words are out");
    remove(root.as_path());
}

/// A terminal that takes no words,
/// as a standard error whose reader is gone.
struct ClosedTerminal;

impl std::io::Write for ClosedTerminal {
    fn write(&mut self, _bytes: &[u8]) -> std::io::Result<usize> {
        return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
///  A record the service replaced while a command was wording it is kept,
/// so the person sees both statements.
/// Why:
///   A retryable failure's record is replaced by the terminal failure's of the same branch and commit;
/// a reader that
///  removed the pathname would delete the words nobody has read,
/// and the promise it replaced would be the last thing shown.
#[test]
fn a_newer_record_at_the_same_name_survives_the_older_reader() {
    let root: PathBuf = fixture("push-failure-replaced");
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, "rejected for now").expect("record");
    // The service replaces the record while the command words it: the namer stands for that pause.
    let replacing = |commit: &str| {
        record_push_failure(root.as_path(), BRANCH, commit, PushOutcome::Refused, "the remote wants a login").expect("the newer record");
        return commit.to_string();
    };
    let first: Vec<String> = take_push_failures_for(root.as_path(), None, &replacing);
    assert_eq!(first, vec![push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "rejected for now")], "the words are the ones this command read");
    let second: Vec<String> = take_push_failures(root.as_path());
    assert_eq!(second, vec![push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "the remote wants a login")], "the newer statement is still there to show");
    remove(root.as_path());
}

/// The sentence the wrapper used to print for a push it had proved delivered,
/// kept here as text a remote's hook might print.
const A_REMOTE_SPEAKING_OF_DELIVERY: &str = "git push delivered every reference it was asked for";

/// What:
///  A remote's own words never become the wrapper's claim that a push was delivered.
/// Why:
///   A remote's hook controls what the push's output carries,
/// so a hook that prints the sentence the wrapper uses for a proven delivery
///  could make the notice say the commit is on the remote and nothing is left to deliver,
/// although the remote refused the
///  update.
/// The sentence is therefore no input to the wording:
/// what the push came to decides it.
#[test]
fn a_remote_that_prints_the_delivery_sentence_is_not_a_delivery() {
    let forged: String = format!(
        "remote: {}\nremote: nothing is left to deliver\n! [remote rejected] main -> main (hook declined)",
        A_REMOTE_SPEAKING_OF_DELIVERY
    );
    for outcome in [PushOutcome::Unfinished, PushOutcome::Refused] {
        let text: String = push_failure_text(BRANCH, COMMIT, outcome, forged.as_str());
        for absent in ["delivered commit", "nothing is left to deliver, so nothing here needs fixing", "does not restart that work"] {
            assert!(!text.contains(absent), "{absent} in {text}");
        }
        // The remote's line is still shown, quoted as what Git said, so the person can read it for what it is.
        assert!(text.contains("remote: "), "{text}");
    }
}

/// What:
///  A push that did not finish is worded as a push whose delivery is not known,
///  never as one that failed to deliver.
/// Why:
///   A stop or a failure that can pass proves nothing about the remote,
/// and saying the commit exists only on this
///  computer would be false when it landed;
/// the service keeps trying the push,
/// so the text says that and no more.
#[test]
fn a_push_that_did_not_finish_is_worded_as_one_whose_delivery_is_not_known() {
    let text: String = push_failure_text(
        BRANCH,
        COMMIT,
        PushOutcome::Unfinished,
        "monochromatic-git-policy-cli: git push made no progress for 30 seconds and was stopped",
    );
    assert!(text.contains(COMMIT) && text.contains("did not finish"), "{text}");
    assert!(text.contains("may or may not have taken that commit"), "the remote's state is not known: {text}");
    assert!(text.contains("retry plan"), "the push is kept: {text}");
    for absent in ["exists only on this computer", "The backup push of branch main failed", PUSH_FAILED_NOTE] {
        assert!(!text.contains(absent), "{absent} in {text}");
    }
    // A push nothing will try again says so, and promises no retry it cannot keep.
    let alone: String = push_failure_text(BRANCH, COMMIT, PushOutcome::UnfinishedWithNoRetry, "fatal: could not read from remote repository");
    assert!(alone.contains("did not finish") && alone.contains("may or may not have taken that commit"), "{alone}");
    assert!(alone.contains("is not trying this push again by itself"), "{alone}");
    assert!(!alone.contains("keeps the push in its retry plan"), "{alone}");
    // A refusal is worded as one: the remote answered and nothing was delivered.
    let refused: String = push_failure_text(BRANCH, COMMIT, PushOutcome::Refused, "fatal: could not read from remote repository");
    assert!(refused.contains("failed for commit") && refused.contains(PUSH_FAILED_NOTE), "{refused}");
}

/// What:
///  The record on disk carries what the push came to,
///  and the notice is worded from that and not from its words.
/// Why:
///   The record is written by one process and shown by another,
/// so the kind has to survive the file;
/// a record whose
///  text holds the delivery sentence but whose kind is a refusal keeps the refusal's wording.
#[test]
fn the_record_keeps_what_the_push_came_to() {
    let root: PathBuf = fixture("notice-outcome");
    let forged: String = format!("remote: {A_REMOTE_SPEAKING_OF_DELIVERY}");
    // One branch per kind, so the three records and the three wordings can be told apart.
    const THIRD: &str = "fedcba9876543210fedcba9876543210fedcba98";
    record_push_failure(root.as_path(), BRANCH, COMMIT, PushOutcome::Refused, forged.as_str()).expect("record");
    record_push_failure(root.as_path(), "refs/heads/other", OTHER_COMMIT, PushOutcome::Unfinished, forged.as_str()).expect("record");
    record_push_failure(root.as_path(), "refs/heads/third", THIRD, PushOutcome::UnfinishedWithNoRetry, forged.as_str()).expect("record");
    let shown: Vec<String> = take_push_failures(root.as_path());
    assert_eq!(shown.len(), 3, "{shown:?}");
    let main: &str = shown.iter().find(|text| { return text.contains(COMMIT); }).expect("the refused record");
    assert!(main.contains("failed for commit"), "{main}");
    let other: &str = shown.iter().find(|text| { return text.contains(OTHER_COMMIT); }).expect("the unfinished record");
    assert!(other.contains("did not finish") && other.contains("keeps the push in its retry plan"), "{other}");
    let third: &str = shown.iter().find(|text| { return text.contains(THIRD); }).expect("the record of a push nothing tries again");
    assert!(third.contains("did not finish") && third.contains("is not trying this push again by itself"), "{third}");
    for notice in [main, other, third] {
        assert!(notice.starts_with("The backup push of branch"), "the framing is the wrapper's own: {notice}");
    }
    // The kind is in the record's own first line, not in the words Git printed.
    for entry in std::fs::read_dir(notice_directory(root.as_path())).expect("directory").flatten() {
        let text: String = std::fs::read_to_string(entry.path()).expect("record");
        assert!(
            text.starts_with(RECORD_MAGIC) || text.starts_with(UNFINISHED_MAGIC) || text.starts_with(UNFINISHED_NO_RETRY_MAGIC),
            "a record names what the push came to: {text}"
        );
    }
    remove(root.as_path());
}
