//! Controls for the record of a failed backup push: its text, its masking, its names, and which commands show it.

use super::*;
use crate::test_support::{fixture, remove};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER_COMMIT: &str = "89abcdef0123456789abcdef0123456789abcdef";
const BRANCH: &str = "refs/heads/main";

fn region(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// The text names the branch, the commit, Git's own reason, that the commit is saved here, and what to do.
#[test]
fn the_text_names_the_branch_the_commit_the_reason_and_the_remedy() {
    let said: &str = "! [rejected] main -> main (non-fast-forward)";
    let text: String = push_failure_text(BRANCH, COMMIT, said);
    assert!(text.contains("branch main"), "{text}");
    assert!(text.contains(&COMMIT[..SHORT_COMMIT]), "{text}");
    assert!(text.contains(said), "{text}");
    assert!(text.contains("saved in this repository"), "{text}");
    assert!(text.contains("`git push`"), "{text}");
    assert!(text.contains("`git pull`"), "{text}");
    assert!(text.ends_with(PUSH_FAILED_NOTE), "{text}");
}

/// A reason that Git left empty is said so, rather than printing an empty quotation.
#[test]
fn an_empty_reason_is_said_plainly() {
    let text: String = push_failure_text(BRANCH, COMMIT, "  \n");
    assert!(text.contains(NO_REASON), "{text}");
    assert!(!text.contains("Git said:"), "{text}");
}

/// A remote URL's user name and password never reach the text, including in a line that names the Git LFS endpoint
///  setting; the line itself stays, with its host.
#[test]
fn credentials_are_masked_and_the_lfs_endpoint_line_stays() {
    let said: &str = "fatal: unable to access 'https://user:s3cret@example.invalid/repo.git/': 403\n\
                      error: lfs.url=https://tok3n@lfs.example.invalid/x is not allowed\n\
                      remote: denied";
    let text: String = push_failure_text(BRANCH, COMMIT, said);
    for secret in ["s3cret", "tok3n", "user:"] {
        assert!(!text.contains(secret), "{secret} leaked: {text}");
    }
    assert!(text.contains(mask_userinfo_text("https://user:s3cret@example.invalid/").as_str()), "{text}");
    assert!(text.contains("lfs.url=https://<credentials>@lfs.example.invalid/x is not allowed"), "{text}");
    assert!(text.contains("remote: denied"), "{text}");
}

/// The wrapper's own failure line for a Git LFS push, which names the endpoint setting in prose and carries the exit status,
///  reaches the person whole.
#[test]
fn the_wrappers_own_lfs_failure_line_keeps_its_status() {
    let said: String = format!("batch response: 401\n{}", crate::lfs_pre_push::failed_message("exited with status 2"));
    let text: String = push_failure_text(BRANCH, COMMIT, said.as_str());
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
        ("lfs.url=https://[2001:db8::1]:8443/path@segment", "NOTHING-TO-HIDE", "https://[2001:db8::1]:8443/path@segment"),
        ("tried https://u1:s1@one.invalid/a then https://u2:s2@two.invalid/b and http://plain.invalid/c", "s1", "http://plain.invalid/c"),
        ("lfs.url=https://host.invalid/path?next=a@b#frag@x", "NOTHING-TO-HIDE", "https://host.invalid/path?next=a@b#frag@x"),
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

/// Control characters from a remote's words cannot move the cursor or recolour the terminal; tabs and lines stay.
#[test]
fn control_characters_are_dropped_and_long_output_is_cut() {
    let cleaned: String = clean_reason("remote: \u{1b}[31mred\u{1b}[0m\u{7}\n\tindented");
    assert_eq!(cleaned, "remote: [31mred[0m\n\tindented");
    let long: String = "x".repeat(LONGEST_REASON * 2);
    let cut: String = clean_reason(long.as_str());
    assert!(cut.len() < LONGEST_REASON + 100, "{}", cut.len());
    assert!(cut.ends_with("(Git's output is cut here.)"));
}

/// The record name belongs to its branch; one branch's name is never the start of another's, and a very long branch
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

/// A failure is taken once; a second take finds nothing, and a repeat of the same failure leaves one record.
#[test]
fn a_record_is_taken_once_and_a_repeat_leaves_one() {
    let root: PathBuf = fixture("push-failure-once");
    record_push_failure(root.as_path(), BRANCH, COMMIT, "first").expect("record");
    record_push_failure(root.as_path(), BRANCH, COMMIT, "again").expect("record");
    let shown: Vec<String> = take_push_failures(root.as_path());
    assert_eq!(shown, vec![push_failure_text(BRANCH, COMMIT, "again")]);
    assert!(take_push_failures(root.as_path()).is_empty(), "a record is shown once");
    remove(root.as_path());
}

/// A repository where nothing failed has no notice directory, and taking from it shows nothing and creates nothing.
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
    record_push_failure(root.as_path(), BRANCH, COMMIT, "main failed").expect("record");
    record_push_failure(root.as_path(), "refs/heads/main-two", OTHER_COMMIT, "two failed").expect("record");
    clear_push_failures(root.as_path(), BRANCH);
    let shown: Vec<String> = take_push_failures(root.as_path());
    assert_eq!(shown, vec![push_failure_text("refs/heads/main-two", OTHER_COMMIT, "two failed")]);
    remove(root.as_path());
}

/// Plumbing and the forms that ask for output made for a program are machine-read; ordinary human commands are not.
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
