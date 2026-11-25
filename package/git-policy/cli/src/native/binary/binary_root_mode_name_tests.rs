//! What:
//!  The refusal path of the installed name and the words,
//!  as a binary control:
//!  a throwaway repository,
//!  the built wrapper linked under the installed ZED history event's name,
//!  one bare word started under that name,
//!  and the marker the wrapper's own start would have written.
//! Why:
//!  The dispatch's two refusals are the one thing no source text can prove:
//!  what a start prints and what it leaves on disk,
//!  and the marker is what says no part of the wrapper ran,
//!  because every start that reached `entry::run_process` writes
//!  `.git/cli-git-lock-holders/` into the repository it stands in
//!  (`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
//!  "Every start-time action of today's wrapper").
//!  The control's green arm is the same link answering the modes' own
//!  `--help` and the words' `--version`,
//!  and its control is a second link named `git`,
//!  whose same bare word reaches the wrapper and writes the marker,
//!  which proves the marker check can see one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const start = spawnSync(link, ['boot-reconcile'], { cwd: repository });
//! expect(start.status).toBe(2); expect(existsSync(join(repository, '.git/cli-git-lock-holders'))).toBe(false);
//! ```

use crate::support_installed_name::{installed_name, root_mode_words};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The marker every start that reached the wrapper's own entry writes,
/// relative to the repository it stands in.
const LOCK_HOLDERS: &str = ".git/cli-git-lock-holders";

/// The line every refusal opens with,
/// which carries the words and the colon.
fn refusal_prefix() -> String {
    return format!("{} {}: ", root_mode_words()[0], root_mode_words()[1]);
}

/// The words themselves,
/// which the version line starts with.
fn words_text() -> String {
    return format!("{} {} ", root_mode_words()[0], root_mode_words()[1]);
}

/// A throwaway repository with one commit's folder,
/// so `git init` needs no template of the caller's own.
fn throwaway_repository(link: &Path, name: &str) -> PathBuf {
    let repository: PathBuf = link.parent().expect("the link's folder").join(name);
    std::fs::create_dir_all(repository.as_path()).expect("the throwaway repository's folder");
    let initialized: Output = Command::new("git").arg("init").arg("--quiet").arg(repository.as_path()).output().expect("git runs");
    assert!(initialized.status.success(), "the throwaway repository is made: {}", String::from_utf8_lossy(initialized.stderr.as_slice()));
    return repository;
}

/// One start's status and both streams.
fn started(program: &Path, arguments: &[&str], in_repository: &Path) -> (Option<i32>, String) {
    let output: Output = Command::new(program).args(arguments).current_dir(in_repository).output().expect("the start runs");
    return (output.status.code(), format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice())));
}

/// What:
///  One bare word under the installed name is refused before the grammar,
///  prints the misuse and the usage,
///  and writes no marker into the repository it stands in.
/// Why:
///  A stale unit line and a person's guess both start this file with a bare
///  mode word,
///  and neither may reach Git as root.
#[test]
fn a_bare_word_under_the_installed_name_is_refused_and_writes_no_marker() {
    let link: PathBuf = installed_name("bare-word");
    let repository: PathBuf = throwaway_repository(link.as_path(), "bare-word");
    let (status, said): (Option<i32>, String) = started(link.as_path(), &["boot-reconcile"], repository.as_path());
    assert_eq!(status, Some(2), "a misuse answers the refusal status: {said}");
    assert!(said.starts_with(refusal_prefix().as_str()), "the refusal opens with the words: {said}");
    assert!(said.contains("`boot-reconcile`"), "the refusal names the word a person typed: {said}");
    assert!(said.contains("usage: "), "the usage follows the refusal: {said}");
    assert!(!repository.join(LOCK_HOLDERS).exists(), "no part of the wrapper ran, so no lock holder was recorded: {}", repository.join(LOCK_HOLDERS).display());
}

/// What:
///  The group word alone is refused,
///  because no unit line and no person can mean the sweep by the words.
#[test]
fn the_group_word_alone_is_refused_and_names_the_modes() {
    let link: PathBuf = installed_name("group-word-alone");
    let repository: PathBuf = throwaway_repository(link.as_path(), "group-word-alone");
    let words: [&str; 2] = root_mode_words();
    let (status, said): (Option<i32>, String) = started(link.as_path(), &words, repository.as_path());
    assert_eq!(status, Some(2), "{said}");
    assert!(said.starts_with(refusal_prefix().as_str()), "{said}");
    for mode in ["boot-reconcile", "start-probe", "shutdown-unmount", "holder-scan", "adopt-worktree-parent", "record-checkout", "mount-path-descendant"] {
        assert!(said.contains(mode), "the refusal names every mode and lacks {mode}: {said}");
    }
    assert!(!repository.join(LOCK_HOLDERS).exists(), "nothing of the wrapper ran");
}

/// What:
///  The two words the installed name answers are the modes' own,
///  and the words answer `--version` on the same file.
/// Why:
///  A release smoke and a person's check of the installed bytes are this
///  command,
///  and it carries no word a mode would read.
#[test]
fn the_installed_name_answers_help_and_version_and_the_words_answer_version() {
    let link: PathBuf = installed_name("answers");
    let repository: PathBuf = throwaway_repository(link.as_path(), "answers");
    let (help_status, help): (Option<i32>, String) = started(link.as_path(), &["--help"], repository.as_path());
    assert_eq!(help_status, Some(0), "{help}");
    assert!(help.starts_with("usage: "), "`--help` prints the modes' usage: {help}");
    let (version_status, version): (Option<i32>, String) = started(link.as_path(), &["--version"], repository.as_path());
    assert_eq!(version_status, Some(0), "{version}");
    assert!(version.starts_with(words_text().as_str()), "`--version` prints the words and the release: {version}");
    let words: [&str; 2] = root_mode_words();
    let (words_status, words_version): (Option<i32>, String) = started(link.as_path(), &[words[0], words[1], "--version"], repository.as_path());
    assert_eq!(words_status, Some(0), "{words_version}");
    assert_eq!(words_version, version, "the name and the words answer one line");
    assert!(!repository.join(LOCK_HOLDERS).exists(), "neither answer reached the wrapper's own entry");
}

/// What:
///  A link named `git` is the wrapper,
///  so the same bare word reaches the wrapper's own entry and writes the
///  marker.
/// Why:
///  This is the control that proves the marker check above can see a marker:
///  a check that never sees one would pass a start that ran the whole wrapper.
#[test]
fn a_link_named_git_answers_the_bare_word_as_the_wrapper_and_writes_the_marker() {
    let link: PathBuf = installed_name("as-git");
    let as_git: PathBuf = link.parent().expect("the link's folder").join("git");
    let _removed: std::io::Result<()> = std::fs::remove_file(as_git.as_path());
    std::os::unix::fs::symlink(crate::support_installed_name::BUILT_WRAPPER, as_git.as_path()).expect("the wrapper's own link");
    let repository: PathBuf = throwaway_repository(link.as_path(), "as-git");
    let (_status, _said): (Option<i32>, String) = started(as_git.as_path(), &["boot-reconcile"], repository.as_path());
    assert!(repository.join(LOCK_HOLDERS).exists(), "a start that reached the wrapper's entry records the lock holder, which is what the arms above read for");
}
