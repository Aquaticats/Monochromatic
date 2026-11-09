//! Controls for marking the forbidden-strings rules text before and after a command that moves the
//! worktree:
//! the digest of a plain rules file,
//! the file behind a link of the person's own,
//! the named pipe that ends the command instead of holding it,
//! and the sentence that names the path.

use super::*;
use crate::config_file::CONFIG_FILE_NAME;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// Writes the repository configuration that names `rules` as the forbidden-strings rules file.
fn config_naming(root: &Path, rules: &str) {
    let config: String = format!(
        "{{\"policies\": {{\"security/forbidden-strings\": [\"error\", {{\"builtinRules\": true, \"rulesFile\": \"{rules}\"}}]}}}}\n"
    );
    std::fs::write(root.join(CONFIG_FILE_NAME), config).expect("the repository configuration");
}

/// What:
///  A rules file that is a plain file is marked.
/// Why:
///   The control the named pipe is measured against:
/// a mark that never happens would make the other controls pass
/// without proving anything.
#[test]
fn a_plain_rules_file_is_marked() {
    let root: PathBuf = fixture("rules-text-plain");
    std::fs::write(root.join("rules.txt"), b"AAA_RULE_LONG_ENOUGH\n").expect("rules");
    config_naming(root.as_path(), "rules.txt");
    let digest: Result<u64, NoDigest> = digest_of_rules_text(root.as_path());
    assert!(digest.is_ok(), "a plain rules file has a digest");
    // The same text under another name is another digest, so the file's own bytes reach it.
    std::fs::write(root.join("rules.txt"), b"BBB_RULE_LONG_ENOUGH\n").expect("rules");
    let changed: u64 = digest_of_rules_text(root.as_path()).expect("a plain rules file");
    assert_ne!(digest.ok(), Some(changed), "the changed text is marked as changed");
    remove(root.as_path());
}

/// What:
///  A rules file behind a link of the person's own is marked through the link,
/// and the link to a named pipe is refused at once.
/// Why:
///   A configuration names the path a person means,
/// which may be kept behind a link;
/// following it must not turn a plain file
///  into an unmarked one,
/// and the pipe the link reaches must not be waited on.
#[test]
fn a_rules_file_behind_a_link_is_marked_and_a_link_to_a_named_pipe_is_refused_at_once() {
    let root: PathBuf = fixture("rules-text-link");
    let real: PathBuf = root.join("real-rules.txt");
    std::fs::write(real.as_path(), b"AAA_RULE_LONG_ENOUGH\n").expect("rules");
    let link: PathBuf = root.join("linked-rules.txt");
    std::os::unix::fs::symlink(real.as_path(), link.as_path()).expect("the person's link");
    config_naming(root.as_path(), "linked-rules.txt");
    let through_link: Result<u64, NoDigest> = digest_of_rules_text(root.as_path());
    assert!(through_link.is_ok(), "a link to a plain rules file is the file the configuration means");
    std::fs::remove_file(link.as_path()).expect("the link goes");
    let pipe: PathBuf = root.join("linked-rules.txt");
    crate::subvolume_sync_attrs::make_fifo(pipe.as_path(), 0o600).expect("named pipe");
    let bounded: PathBuf = root.clone();
    let read: Option<Result<u64, NoDigest>> = crate::test_support::finished_within(move || { return digest_of_rules_text(bounded.as_path()); });
    let Some(read) = read else { panic!("marking a rules text whose file is a named pipe did not finish within five seconds") };
    let named: PathBuf = root.clone();
    match read {
        Err(NoDigest::Unreadable { path, reason }) => {
            assert_eq!(path, named.join("linked-rules.txt"), "the refusal names the file the configuration means");
            assert!(reason.to_string().contains("not a plain file"), "{reason}");
        }
        Err(NoDigest::Unselectable) => panic!("the configuration was read, so the rules file is the refusal"),
        Ok(_) => panic!("a named pipe is not a rules file"),
    }
    remove(root.as_path());
}

/// What:
///  A rules file that is a named pipe is refused at once,
/// and the sentence for the person names the path
///  and what follows.
/// Why:
///  Opening a named pipe for reading waits for a writer that may never come,
/// so this mark held
/// `git checkout`,
/// `git merge`,
/// `git pull`,
/// `git switch`,
/// `git rebase` and `git reset` for ever
/// (`doc/troubleshooting/a-fifo-where-a-file-is-expected-ends-no-command.md`).
#[test]
fn a_rules_file_that_is_a_named_pipe_is_refused_at_once_and_the_line_names_it() {
    let root: PathBuf = fixture("rules-text-pipe");
    let pipe: PathBuf = root.join("rules.fifo");
    crate::subvolume_sync_attrs::make_fifo(pipe.as_path(), 0o600).expect("named pipe");
    config_naming(root.as_path(), "rules.fifo");
    let bounded: PathBuf = root.clone();
    let read: Option<Result<u64, NoDigest>> = crate::test_support::finished_within(move || { return digest_of_rules_text(bounded.as_path()); });
    let Some(read) = read else { panic!("marking a rules text whose file is a named pipe did not finish within five seconds") };
    let Err(NoDigest::Unreadable { path, reason }) = read else { panic!("a named pipe is not a rules file") };
    assert_eq!(path, pipe, "the refusal names the rules file the configuration names");
    let line: String = unreadable_rules_line(path.as_path(), &reason);
    assert!(line.contains(pipe.display().to_string().as_str()), "{line}");
    assert!(line.contains("not a plain file"), "{line}");
    assert!(line.contains("next commit reads it"), "{line}");
    remove(root.as_path());
}

/// What:
///  An absent rules file is still told from an empty one,
/// and a selection that cannot be made marks nothing without a line.
/// Why:
///   The digest of a missing file must not change just because the file may not be read through a
///  pipe:
/// the default rules file is usually absent.
#[test]
fn an_absent_rules_file_is_still_a_mark_of_its_own() {
    let root: PathBuf = fixture("rules-text-absent");
    config_naming(root.as_path(), "rules.txt");
    let absent: Result<u64, NoDigest> = digest_of_rules_text(root.as_path());
    assert!(absent.is_ok(), "an absent rules file is marked as absent");
    std::fs::write(root.join("rules.txt"), b"").expect("an empty rules file");
    let empty: Result<u64, NoDigest> = digest_of_rules_text(root.as_path());
    assert!(empty.is_ok(), "an empty rules file is marked as empty");
    assert_ne!(absent.ok(), empty.ok(), "an absent file is not the empty one");
    remove(root.as_path());
}
