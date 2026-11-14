//! What:
//!  The audits' two matrices, kept:
//!  every discarding form against every kind of file in the way,
//!  refused rather than saved,
//!  and the same form under the guard's own opt-out beside real Git.
//! Why:
//!  Two independent audits of 0.6.1-alpha.12 found the same hole from two directions:
//!  the guard decided from a status that names none of the files these forms overwrite,
//!  so an untracked file, an ignored file, a nested repository's work and an edit behind
//!  an `assume-unchanged` mark were all lost with no refusal and no copy.
//!  A matrix is the only shape that keeps every form covered while the code beneath changes:
//!  each row says what real Git does to the file (the positive control, without which a
//!  passing row proves nothing),
//!  that the wrapper refuses while the file stands there,
//!  and that the same command with `--no-enforce-worktree` leaves the worktree and the index
//!  exactly as real Git leaves them.
//!  `skip-worktree` is beside the marks because it must stay the one case Git itself keeps.
//!  The file in the way is untracked, ignored, a file where the target needs a directory,
//!  and a nested repository's own work.
//!  `read-tree` has rows of its own:
//!  with `-m -u` it writes over an ignored file at exit 0,
//!  and with `--reset -u` it writes over an untracked one as well,
//!  which is the hole the reading of written paths closes.
//!  `clean` has no row here: its deletion of ignored files runs and is logged by the owner's
//!  answer, so the promise in these rows is not the one it keeps;
//!  its nested repositories and its pathspec forms are covered by the `discard::` controls.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // every form x every kind of file in the way: refused, then the opt-out beside real Git
//! ```

use super::support::{
    Fixture, Observed, bounded_for, fixture, git, observe, porcelain, remove, repository, run_direct, run_wrapped, stderr_of,
    wrapper_first_path,
};
use git_policy_cli::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The bytes a person wrote at the path the form writes.
const PRECIOUS: &[u8] = b"PRECIOUS UNTRACKED\n";

/// What stands where the target tree writes,
/// as the audits shaped it.
#[derive(Clone, Copy)]
enum InTheWay {
    /// A file the tree holds as a file.
    File,
    /// An ignored file the tree holds as a file: Git writes over it without a word.
    Ignored,
    /// A file where the tree needs a directory (`b.txt/inner.txt`).
    FileWhereADirectoryMustGo,
    /// A nested Git repository with uncommitted work where the tree needs a directory.
    Repository,
}

/// The forms the matrices run,
/// each as the audit wrote it.
fn forms() -> Vec<Vec<&'static str>> {
    return vec![
        vec!["reset", "--hard", "other"],
        vec!["reset", "--merge", "other"],
        vec!["reset", "--keep", "other"],
        vec!["checkout", "--force", "other"],
        vec!["switch", "--discard-changes", "other"],
        vec!["switch", "--force", "other"],
        vec!["checkout", "other"],
        vec!["checkout", "other", "--", "b.txt"],
        vec!["restore", "--source=other", "--", "b.txt"],
        vec!["read-tree", "-m", "-u", "HEAD", "other"],
        vec!["read-tree", "--reset", "-u", "other"],
    ];
}

/// A repository whose `other` branch tracks what `shape` needs,
/// with a linked worktree that holds the person's file where that path goes.
///  The person's work is always at `b.txt`,
///  which the branch never tracks as the same kind of thing.
fn prepared(name: &str, shape: InTheWay) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    if matches!(shape, InTheWay::File) {
        std::fs::write(repo.join("b.txt"), b"tracked on other\n").expect("tracked file");
        git(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
    } else if matches!(shape, InTheWay::Repository) {
        // The nested repository's own edited file stands at the path the target tree writes,
        // so Git writes over the person's edit and the row measures a loss that happens.
        std::fs::create_dir_all(repo.join("b.txt")).expect("directory on the branch");
        std::fs::write(repo.join("b.txt/tracked.txt"), b"tracked on other\n").expect("tracked file");
        git(&fixture, repo.as_path(), &["add", "--", "b.txt/tracked.txt"]);
    } else {
        std::fs::create_dir_all(repo.join("b.txt")).expect("directory on the branch");
        std::fs::write(repo.join("b.txt/inner.txt"), b"tracked on other\n").expect("tracked file");
        git(&fixture, repo.as_path(), &["add", "--", "b.txt/inner.txt"]);
    }
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=other"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    if matches!(shape, InTheWay::Ignored) {
        std::fs::write(repo.join(".gitignore"), b"b.txt\n").expect("ignore rules");
        git(&fixture, repo.as_path(), &["add", "--", ".gitignore"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ignore"]);
    }
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    match shape {
        InTheWay::File | InTheWay::Ignored | InTheWay::FileWhereADirectoryMustGo => {
            std::fs::write(linked.join("b.txt"), PRECIOUS).expect("the person's file");
        }
        InTheWay::Repository => {
            let nested: PathBuf = linked.join("b.txt");
            std::fs::create_dir(nested.as_path()).expect("nested repository directory");
            git(&fixture, nested.as_path(), &["init", "--quiet", "--initial-branch=inner"]);
            std::fs::write(nested.join("tracked.txt"), b"committed\n").expect("nested tracked file");
            git(&fixture, nested.as_path(), &["add", "--", "tracked.txt"]);
            git(&fixture, nested.as_path(), &["commit", "--quiet", "--message=inner"]);
            std::fs::write(nested.join("tracked.txt"), PRECIOUS).expect("the person's edit");
        }
    }
    return (fixture, repo, linked);
}

/// Every entry of a worktree, as `<kind> <path> <content>`,
/// in one stable order,
/// with every `.git` entry left out:
/// a Git directory names absolute paths and commit times each fixture spells differently,
/// and it is not the person's worktree.
fn worktree_state(root: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::<String>::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.file_name() == OsStr::new(".git") {
                continue;
            }
            let path: PathBuf = entry.path();
            let shown: String = path.strip_prefix(root).unwrap_or(path.as_path()).display().to_string();
            let Ok(metadata) = std::fs::symlink_metadata(path.as_path()) else {
                continue;
            };
            if metadata.file_type().is_symlink() {
                let target: PathBuf = std::fs::read_link(path.as_path()).unwrap_or_default();
                found.push(format!("link {shown} {}", target.display()));
                continue;
            }
            if metadata.is_dir() {
                found.push(format!("dir {shown}"));
                pending.push(path);
                continue;
            }
            let content: Vec<u8> = std::fs::read(path.as_path()).unwrap_or_default();
            found.push(format!(
                "file {shown} {}",
                content.iter().map(|byte| return format!("{byte:02x}")).collect::<String>()
            ));
        }
    }
    found.sort();
    return found;
}

/// What one arm of one row left behind:
/// its exit status, its worktree bytes and the index Git reports.
fn state_after(fixture: &Fixture, linked: &Path, observed: &Observed) -> (Option<i32>, Vec<String>, String) {
    return (observed.code, worktree_state(linked), porcelain(fixture, linked));
}

/// Every file the wrapper keeps below its discarded copies,
/// which a refusal must leave empty.
fn saved_files(repo: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::<PathBuf>::new();
    let mut pending: Vec<PathBuf> = vec![repo.join(".git").join("monochromatic-git-policy-cli").join("discarded")];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            found.push(path);
        }
    }
    return found;
}

/// Whether the wrapper itself refused the command,
/// rather than letting Git answer for it.
fn refused_by_the_guard(observed: &Observed) -> bool {
    let said: String = stderr_of(observed);
    return said.contains("\"policyId\":\"linked-worktree-only\"") && said.contains("\"type\":\"finding\"");
}

/// Whether the person's bytes are at `relative` in the linked worktree.
fn still_where_it_was(linked: &Path, relative: &str, needle: &[u8]) -> bool {
    return std::fs::read(linked.join(relative)).is_ok_and(|bytes| return bytes == needle);
}

/// Whether the person's bytes are anywhere the form would have kept them:
/// at `b.txt` or inside the nested repository below it.
fn the_work_is_there(linked: &Path) -> bool {
    return still_where_it_was(linked, "b.txt", PRECIOUS) || still_where_it_was(linked, "b.txt/tracked.txt", PRECIOUS);
}

/// The same command with the guard's own opt-out directly after the subcommand,
/// which is where the wrapper looks for it.
fn with_the_hatch(arguments: &[&str]) -> Vec<String> {
    let mut hatched: Vec<String> = vec![String::from(arguments[0]), String::from(WORKTREE_ENFORCEMENT_ESCAPE_HATCH)];
    for argument in arguments.iter().skip(1) {
        hatched.push(String::from(*argument));
    }
    return hatched;
}

/// Run one form with the opt-out and keep the words alive for the call.
fn run_hatched(fixture: &Fixture, linked: &Path, arguments: &[&str]) -> Observed {
    let owned: Vec<String> = with_the_hatch(arguments);
    let words: Vec<&str> = owned.iter().map(|word| return word.as_str()).collect::<Vec<&str>>();
    return run_wrapped(fixture, linked, words.as_slice());
}

/// What one row found in real Git:
/// whether the person's bytes were lost,
/// and the state that arm left.
struct Control {
    /// Whether the bytes were gone after real Git ran.
    lost: bool,
    /// The exit status and the state the direct run left,
    ///  which the opt-out arm must reproduce.
    state: (Option<i32>, Vec<String>, String),
}

/// What real Git does with one form over one shape,
/// measured before the wrapper is asked about it.
fn control(name: &str, shape: InTheWay, arguments: &[&str]) -> Control {
    let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = prepared(name, shape);
    let observed: Observed = run_direct(&fixture, linked.as_path(), arguments);
    let found: Control = Control {
        lost: !the_work_is_there(linked.as_path()),
        state: state_after(&fixture, linked.as_path(), &observed),
    };
    remove(&fixture);
    return found;
}

/// Every row of one shape:
/// the wrapper refuses while the person's bytes stand in the way and nothing is written,
/// a run it let through loses nothing,
/// and the same command with the opt-out leaves exactly what real Git leaves.
fn run_shape(shape: InTheWay, label_of_shape: &str) {
    let mut refused: usize = 0;
    let mut lost_by_git: usize = 0;
    for (index, arguments) in forms().into_iter().enumerate() {
        let label: String = format!("discard-matrix-{label_of_shape}-{index}");
        let measured: Control = control(format!("{label}-direct").as_str(), shape, arguments.as_slice());
        if measured.lost {
            lost_by_git += 1;
        }
        let (wrapped_fixture, repo, linked): (Fixture, PathBuf, PathBuf) = prepared(format!("{label}-wrapped").as_str(), shape);
        let before: Vec<String> = worktree_state(linked.as_path());
        let observed: Observed = run_wrapped(&wrapped_fixture, linked.as_path(), arguments.as_slice());
        if refused_by_the_guard(&observed) {
            refused += 1;
            let said: String = stderr_of(&observed);
            assert!(said.contains("b.txt"), "{arguments:?} over {label} refused without naming b.txt: {said}");
            assert!(said.contains(WORKTREE_ENFORCEMENT_ESCAPE_HATCH), "{arguments:?} over {label}: {said}");
            assert!(said.contains("Nothing in your worktree was changed"), "{arguments:?} over {label}: {said}");
            assert_eq!(worktree_state(linked.as_path()), before, "{arguments:?} over {label} wrote something before refusing");
            assert!(the_work_is_there(linked.as_path()), "{arguments:?} over {label} lost the person's work while refusing");
            assert!(saved_files(repo.as_path()).is_empty(), "{arguments:?} over {label} kept a copy the owner's answer refuses");
            // The same command under the opt-out runs exactly as real Git runs it.
            let (hatch_fixture, _hatch_repo, hatch_linked): (Fixture, PathBuf, PathBuf) =
                prepared(format!("{label}-hatch").as_str(), shape);
            let passed: Observed = run_hatched(&hatch_fixture, hatch_linked.as_path(), arguments.as_slice());
            assert_eq!(
                state_after(&hatch_fixture, hatch_linked.as_path(), &passed),
                measured.state,
                "{arguments:?} over {label} under the opt-out differs from real Git"
            );
            assert!(!refused_by_the_guard(&passed), "{arguments:?} over {label} was refused under its own opt-out: {}", stderr_of(&passed));
            remove(&hatch_fixture);
        } else {
            // A form the wrapper let through loses nothing: Git either keeps the entry itself or refuses the form.
            assert!(
                the_work_is_there(linked.as_path()),
                "{arguments:?} over {label} exited without a refusal and lost the person's work: {}",
                stderr_of(&observed)
            );
        }
        remove(&wrapped_fixture);
    }
    assert!(refused > 0, "no form of shape {label_of_shape} is refused, so this matrix proves nothing");
    assert!(lost_by_git > 0, "no form of shape {label_of_shape} loses the person's work in real Git, so this matrix proves nothing");
}

/// Every form over a file Git did not put in the worktree:
/// the wrapper refuses and names it,
/// and the opt-out runs the form exactly as real Git runs it.
#[test]
fn every_form_over_an_untracked_file_is_refused_and_the_opt_out_runs_it() {
    run_shape(InTheWay::File, "file");
    run_shape(InTheWay::FileWhereADirectoryMustGo, "file-where-a-directory-must-go");
    run_shape(InTheWay::Repository, "nested-repository");
}

/// The same forms over an ignored file the target tree tracks,
/// which is where a person keeps their environment and their local secrets.
#[test]
fn every_form_over_an_ignored_file_is_refused_and_the_opt_out_runs_it() {
    run_shape(InTheWay::Ignored, "ignored");
}

/// The edit a person made,
/// at a path the index may mark.
const EDIT: &[u8] = b"PRECIOUS EDIT\n";

/// A linked worktree whose `a.txt` holds the person's edit,
/// marked with `--<mark>` when one is named.
fn marked(name: &str, mark: Option<&str>) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("tracked file");
    std::fs::write(repo.join("b.txt"), b"bee\n").expect("tracked file");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt", "b.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["branch", "other"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    if let Some(named) = mark {
        git(&fixture, linked.as_path(), &["update-index", &format!("--{named}"), "a.txt"]);
    }
    std::fs::write(linked.join("a.txt"), EDIT).expect("the person's edit");
    return (fixture, repo, linked);
}

/// The forms the marked matrix runs,
/// including a forced form spelled with the shortest abbreviation Git's own table accepts:
/// a reading that carried its own minimum length read `--f` as no force at all.
fn marked_forms() -> Vec<Vec<&'static str>> {
    return vec![
        vec!["reset", "--hard"],
        vec!["reset", "--hard", "HEAD"],
        vec!["reset", "--merge"],
        vec!["checkout", "--force"],
        vec!["checkout", "--force", "other"],
        vec!["checkout", "--", "a.txt"],
        vec!["checkout", "HEAD", "--", "."],
        vec!["restore", "."],
        vec!["restore", "--source=other", "."],
        vec!["switch", "--discard-changes", "other"],
        vec!["switch", "other"],
        vec!["checkout", "other"],
        vec!["read-tree", "--reset", "-u", "HEAD"],
        vec!["checkout-index", "-f", "-a"],
        vec!["checkout-index", "--f", "-a"],
        vec!["rm", "-f", "a.txt"],
        vec!["rm", "--f", "a.txt"],
        vec!["mv", "--f", "a.txt", "b.txt"],
    ];
}

/// The forms whose opt-out is compared with real Git over the marked edit.
/// Why:
///   The comparison starts four more fixtures per row,
///  so it names the forms that cover a tree write,
///  a path write,
///  a force written short,
///  and the `read-tree` form the hole was in.
fn marked_parity_forms() -> Vec<Vec<&'static str>> {
    return vec![
        vec!["reset", "--hard"],
        vec!["checkout", "--", "a.txt"],
        vec!["rm", "--f", "a.txt"],
        vec!["read-tree", "--reset", "-u", "HEAD"],
    ];
}

/// An edit behind the `assume-unchanged` mark gets exactly what the same edit gets without it:
/// every form is refused while the edit stands,
/// the edit is still there,
/// and the opt-out runs the form as real Git runs it.
///  `skip-worktree` is the control on the other side:
///  Git keeps that file,
///  so a form over it must not be refused for it.
#[test]
fn a_marked_edit_is_treated_as_the_edit_it_is() {
    let parity: Vec<Vec<&'static str>> = marked_parity_forms();
    for (index, arguments) in marked_forms().into_iter().enumerate() {
        for mark in ["assume-unchanged", "skip-worktree"] {
            let label: String = format!("discard-marked-{mark}-{index}");
            let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = marked(label.as_str(), Some(mark));
            let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
            // A `mv` that runs carries the edit to the path it names, which is no loss of it.
            let kept: bool = still_where_it_was(linked.as_path(), "a.txt", EDIT)
                || still_where_it_was(linked.as_path(), "b.txt", EDIT);
            assert!(
                kept || observed.code != Some(0),
                "{arguments:?} with {mark} exited 0 and lost the edit: {}",
                stderr_of(&observed)
            );
            remove(&fixture);
            if mark != "assume-unchanged" || !parity.contains(&arguments) {
                continue;
            }
            // The same form over the same edit with no mark, so the two answers are one rule.
            let (plain, _plain_repo, plain_linked): (Fixture, PathBuf, PathBuf) =
                marked(format!("{label}-plain").as_str(), None);
            let unmarked: Observed = run_wrapped(&plain, plain_linked.as_path(), arguments.as_slice());
            assert_ne!(unmarked.code, Some(0), "the control needs an unmarked edit to be refused: {arguments:?}");
            assert_ne!(observed.code, Some(0), "{arguments:?} with the mark ran where the unmarked edit is refused");
            // The opt-out runs it and leaves what real Git leaves over the same marked edit.
            let (hatch_fixture, _hatch_repo, hatch_linked): (Fixture, PathBuf, PathBuf) =
                marked(format!("{label}-hatch").as_str(), Some(mark));
            let passed: Observed = run_hatched(&hatch_fixture, hatch_linked.as_path(), arguments.as_slice());
            let (direct_fixture, _direct_repo, direct_linked): (Fixture, PathBuf, PathBuf) =
                marked(format!("{label}-direct").as_str(), Some(mark));
            let direct: Observed = run_direct(&direct_fixture, direct_linked.as_path(), arguments.as_slice());
            assert_eq!(
                state_after(&hatch_fixture, hatch_linked.as_path(), &passed),
                state_after(&direct_fixture, direct_linked.as_path(), &direct),
                "{arguments:?} under the opt-out differs from real Git"
            );
            remove(&direct_fixture);
            remove(&hatch_fixture);
            remove(&plain);
        }
    }
}

/// A mistyped word Git would correct into a discarding command never runs:
/// the wrapper forwards the call with autocorrection off,
/// Git prints its own error and suggestion,
/// and the edit is still there.
#[test]
fn a_mistyped_word_is_never_corrected_into_a_discarding_form() {
    let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = marked("discard-autocorrect", None);
    git(&fixture, linked.as_path(), &["config", "help.autocorrect", "immediate"]);
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rset", "--hard"]);
    assert_ne!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert!(still_where_it_was(linked.as_path(), "a.txt", EDIT), "the mistyped hard reset lost the edit");
    assert!(stderr_of(&observed).contains("rset"), "{}", stderr_of(&observed));
    // The same mistyped word in real Git runs the command it guessed, which is the loss this row prevents.
    let replacement: Observed = run_direct(&fixture, linked.as_path(), &["-c", "help.autocorrect=immediate", "rset", "--hard"]);
    assert_eq!(replacement.code, Some(0), "the control needs real Git to correct the word: {}", stderr_of(&replacement));
    assert!(!still_where_it_was(linked.as_path(), "a.txt", EDIT), "the control needs real Git to lose the edit");
    remove(&fixture);
}

/// A clean that deletes only ignored files is the owner's own answer of 2026-11-10:
/// it deletes, exits 0 and leaves one line,
/// with no refusal and no copy.
#[test]
fn a_clean_of_ignored_files_deletes_and_leaves_one_line() {
    for (index, arguments) in [vec!["clean", "-fX"], vec!["clean", "-fX", "-d"], vec!["clean", "-fx"]].into_iter().enumerate() {
        let label: String = format!("discard-clean-ignored-{index}");
        let fixture: Fixture = fixture(label.as_str());
        let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
        std::fs::write(repo.join(".gitignore"), b"ignored.txt\n").expect("ignore rules");
        std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
        git(&fixture, repo.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=files"]);
        let linked: PathBuf = fixture.root.join("linked");
        git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
        std::fs::write(linked.join("ignored.txt"), b"IGNORED\n").expect("ignored file");
        let audit: PathBuf = repo.join(".git/monochromatic-git-policy-cli-audit.jsonl");
        let before: usize = std::fs::read_to_string(audit.as_path()).map(|text| return text.lines().count()).unwrap_or(0);
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert_eq!(observed.code, Some(0), "{arguments:?}: {}", stderr_of(&observed));
        assert!(!refused_by_the_guard(&observed), "{arguments:?}: {}", stderr_of(&observed));
        assert!(!linked.join("ignored.txt").exists(), "{arguments:?} left the ignored file");
        assert!(saved_files(repo.as_path()).is_empty(), "{arguments:?} kept a copy the owner's answer refuses");
        let text: String = std::fs::read_to_string(audit.as_path()).unwrap_or_default();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), before + 1, "{arguments:?} left {} lines: {text}", lines.len().saturating_sub(before));
        assert!(lines.last().unwrap_or(&"").contains("\"outcome\":\"allowed\""), "{arguments:?}: {text}");
        remove(&fixture);
    }
}

/// The same clean without `-x` or `-X` keeps the ignored file, as it always did.
#[test]
fn a_clean_without_the_ignored_modes_keeps_an_ignored_file() {
    let fixture: Fixture = fixture("discard-clean-keeps-ignored");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(".gitignore"), b"ignored.txt\n").expect("ignore rules");
    git(&fixture, repo.as_path(), &["add", "--", ".gitignore"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ignore"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::write(linked.join("ignored.txt"), b"IGNORED\n").expect("ignored file");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["clean", "-fd"]);
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert!(linked.join("ignored.txt").exists(), "{}", stderr_of(&observed));
    remove(&fixture);
}

/// `read-tree -m -u` writes over an ignored file at exit 0 and `--reset -u` writes over an
/// untracked one too,
/// so both are refused with the file named while it stands there,
/// and the opt-out leaves exactly what real Git leaves.
#[test]
fn read_tree_in_every_writing_form_is_refused_with_the_file_named() {
    for (index, arguments) in [
        vec!["read-tree", "-m", "-u", "HEAD", "other"],
        vec!["read-tree", "-m", "-u", "other"],
        vec!["read-tree", "--reset", "-u", "other"],
        vec!["read-tree", "--prefix=sub/", "-u", "other"],
    ]
    .into_iter()
    .enumerate()
    {
        for (shape, shape_name) in [(InTheWay::Ignored, "ignored"), (InTheWay::File, "untracked")] {
            let label: String = format!("discard-read-tree-{index}-{shape_name}");
            let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = prepared(label.as_str(), shape);
            put_the_file_under_the_prefix(linked.as_path(), arguments.as_slice());
            let before: Vec<String> = worktree_state(linked.as_path());
            let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
            if refused_by_the_guard(&observed) {
                assert!(
                    stderr_of(&observed).contains("b.txt"),
                    "{arguments:?} over {label} refused without naming the file: {}",
                    stderr_of(&observed)
                );
                assert_eq!(worktree_state(linked.as_path()), before, "{arguments:?} over {label} wrote something");
            } else {
                assert!(
                    still_where_it_was(linked.as_path(), the_persons_path(arguments.as_slice()), PRECIOUS),
                    "{arguments:?} over {label} exited {:?} and lost the person's work, left in {}: {}",
                    observed.code,
                    fixture.root.display(),
                    stderr_of(&observed)
                );
            }
            let (hatch_fixture, _hatch_repo, hatch_linked): (Fixture, PathBuf, PathBuf) =
                prepared(format!("{label}-hatch").as_str(), shape);
            put_the_file_under_the_prefix(hatch_linked.as_path(), arguments.as_slice());
            let (direct_fixture, _direct_repo, direct_linked): (Fixture, PathBuf, PathBuf) =
                prepared(format!("{label}-direct").as_str(), shape);
            put_the_file_under_the_prefix(direct_linked.as_path(), arguments.as_slice());
            let passed: Observed = run_hatched(&hatch_fixture, hatch_linked.as_path(), arguments.as_slice());
            let direct: Observed = run_direct(&direct_fixture, direct_linked.as_path(), arguments.as_slice());
            assert_eq!(
                state_after(&hatch_fixture, hatch_linked.as_path(), &passed),
                state_after(&direct_fixture, direct_linked.as_path(), &direct),
                "{arguments:?} over {label} under the opt-out differs from real Git"
            );
            remove(&direct_fixture);
            remove(&hatch_fixture);
            remove(&fixture);
        }
    }
}

/// The path the person's file stands at for one `read-tree` form:
/// below the directory a `--prefix` names,
/// and at the tree's own path otherwise.
fn the_persons_path(arguments: &[&str]) -> &'static str {
    return if arguments.iter().any(|word| return word.starts_with("--prefix")) {
        "sub/b.txt"
    } else {
        "b.txt"
    };
}

/// A `--prefix` form writes below the directory it names,
/// so the entry in its way is put there,
/// at the path the tree's own name lands on.
fn put_the_file_under_the_prefix(linked: &Path, arguments: &[&str]) {
    if !arguments.iter().any(|word| return word.starts_with("--prefix")) {
        return;
    }
    std::fs::create_dir_all(linked.join("sub")).expect("the directory the prefix writes into");
    std::fs::remove_file(linked.join("b.txt")).expect("the file at the tree's own path");
    std::fs::write(linked.join("sub/b.txt"), PRECIOUS).expect("the person's file under the prefix");
}

//region The alpha.14 audit's findings

/// A `skip-worktree` edit is the file Git keeps for a whole-tree write and writes over for a form that names
/// its paths (measured on this host with native Git 2.56.0),
/// so exactly those path forms are refused,
/// the forms Git keeps the file for are let through,
/// and the opt-out leaves what real Git leaves.
#[test]
fn a_skip_worktree_edit_is_refused_where_git_writes_over_it() {
    // `--ignore-skip-worktree-bits` is the person telling Git to ignore the mark: real Git writes the file
    //  by that instruction, and the guard leaves the command to Git.
    for (index, arguments, disposition) in [
        (1, vec!["checkout", "other", "--", "a.txt"], "refused"),
        (2, vec!["restore", "--source", "other", "--", "a.txt"], "refused"),
        (3, vec!["reset", "--hard"], "kept"),
        (4, vec!["checkout", "--force"], "kept"),
        (5, vec!["read-tree", "--reset", "-u", "HEAD"], "kept"),
        (6, vec!["checkout", "--ignore-skip-worktree-bits", "--", "a.txt"], "chosen"),
    ] {
        let label: String = format!("discard-skip-worktree-{index}");
        let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = marked(label.as_str(), Some("skip-worktree"));
        let kept_before: bool = still_where_it_was(linked.as_path(), "a.txt", EDIT);
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        let kept: bool = still_where_it_was(linked.as_path(), "a.txt", EDIT)
            || still_where_it_was(linked.as_path(), "b.txt", EDIT);
        if disposition == "refused" {
            assert!(refused_by_the_guard(&observed), "{arguments:?} was not refused: {}", stderr_of(&observed));
            assert!(kept, "{arguments:?} lost the edit while refusing");
            assert!(stderr_of(&observed).contains("a.txt"), "{arguments:?}: {}", stderr_of(&observed));
            let (hatch_fixture, _hatch_repo, hatch_linked): (Fixture, PathBuf, PathBuf) =
                marked(format!("{label}-hatch").as_str(), Some("skip-worktree"));
            let passed: Observed = run_hatched(&hatch_fixture, hatch_linked.as_path(), arguments.as_slice());
            let (direct_fixture, _direct_repo, direct_linked): (Fixture, PathBuf, PathBuf) =
                marked(format!("{label}-direct").as_str(), Some("skip-worktree"));
            let direct: Observed = run_direct(&direct_fixture, direct_linked.as_path(), arguments.as_slice());
            assert_eq!(
                state_after(&hatch_fixture, hatch_linked.as_path(), &passed),
                state_after(&direct_fixture, direct_linked.as_path(), &direct),
                "{arguments:?} under the opt-out differs from real Git"
            );
            remove(&direct_fixture);
            remove(&hatch_fixture);
        } else {
            // Git keeps the file for this form, or the person told Git to ignore the mark.
            let (direct_fixture, _direct_repo, direct_linked): (Fixture, PathBuf, PathBuf) =
                marked(format!("{label}-direct").as_str(), Some("skip-worktree"));
            let direct: Observed = run_direct(&direct_fixture, direct_linked.as_path(), arguments.as_slice());
            let git_kept: bool = still_where_it_was(direct_linked.as_path(), "a.txt", EDIT);
            assert!(
                !refused_by_the_guard(&observed),
                "{arguments:?} was refused where the guard leaves it to Git (exit {:?}): {}",
                direct.code,
                stderr_of(&observed)
            );
            if disposition == "kept" {
                assert!(git_kept, "the control needs real Git to keep the file for {arguments:?}");
            } else {
                assert!(!git_kept, "the control needs real Git to write the file for {arguments:?}");
            }
            remove(&direct_fixture);
        }
        assert!(kept_before, "the fixture must hold the edit before the form runs");
        remove(&fixture);
    }
}

/// A form that names paths writes over an edit the `skip-worktree` mark hides,
/// so the question the guard asks about a linked worktree reads that mark,
/// and the whole-tree forms ask the tracked state alone.
#[test]
fn the_skip_worktree_mark_is_read_only_for_the_forms_that_write_over_it() {
    for (arguments, skip_worktree_edits) in [
        (vec!["checkout", "other", "--", "a.txt"], true),
        (vec!["restore", "--source", "other", "--", "a.txt"], true),
        (vec!["checkout", "--force", "other"], false),
        (vec!["reset", "--hard"], false),
        (vec!["checkout", "--ignore-skip-worktree-bits", "--", "a.txt"], false),
        (vec!["restore", "a.txt"], false),
    ] {
        let command: git_policy_cli::rule_linked_worktree::GuardedCommand = if arguments[0] == "restore" {
            git_policy_cli::rule_linked_worktree::GuardedCommand::RestoreWorktree
        } else {
            git_policy_cli::rule_linked_worktree::GuardedCommand::CheckoutPaths
        };
        let words: Vec<std::ffi::OsString> = arguments.iter().skip(1).map(std::ffi::OsString::from).collect();
        let asked: Option<git_policy_cli::rule_linked_worktree::DirtyQuestion> =
            git_policy_cli::rule_linked_worktree::dirty_question(command, words.as_slice());
        let expected: git_policy_cli::rule_linked_worktree::DirtyQuestion =
            git_policy_cli::rule_linked_worktree::DirtyQuestion::Worktree(if skip_worktree_edits {
                git_policy_cli::rule_worktree_dirty::DirtyScope::TrackedAndSkipWorktreeEdits
            } else {
                git_policy_cli::rule_worktree_dirty::DirtyScope::Tracked
            });
        assert_eq!(asked, Some(expected), "{arguments:?}");
    }
}

/// The opt-out gives what real Git gives for a `checkout` or `switch` over an ignored file the target branch
/// tracks: `--no-overwrite-ignore` is not added for that call,
/// so the command switches where real Git switches instead of stopping.
#[test]
fn the_opt_out_switches_where_real_git_switches_over_an_ignored_file() {
    for (index, arguments) in [
        (1, vec!["checkout", "tracks"]),
        (2, vec!["switch", "tracks"]),
        (3, vec!["checkout", "--detach", "tracks"]),
        (4, vec!["switch", "--discard-changes", "tracks"]),
        (5, vec!["checkout", "--force", "tracks"]),
    ] {
        let label: String = format!("discard-opt-out-ignored-{index}");
        let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = prepared_with_an_ignored_tracked_file(label.as_str());
        let before: Vec<String> = worktree_state(linked.as_path());
        let refused: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        // Whatever the wrapper answers, the file is untouched and the branch has not moved.
        assert_eq!(worktree_state(linked.as_path()), before, "{arguments:?} changed the worktree: {}", stderr_of(&refused));
        let (hatch_fixture, _hatch_repo, hatch_linked): (Fixture, PathBuf, PathBuf) =
            prepared_with_an_ignored_tracked_file(format!("{label}-hatch").as_str());
        let passed: Observed = run_hatched(&hatch_fixture, hatch_linked.as_path(), arguments.as_slice());
        let (direct_fixture, _direct_repo, direct_linked): (Fixture, PathBuf, PathBuf) =
            prepared_with_an_ignored_tracked_file(format!("{label}-direct").as_str());
        let direct: Observed = run_direct(&direct_fixture, direct_linked.as_path(), arguments.as_slice());
        if index <= 5 {
            assert_eq!(
                state_after(&hatch_fixture, hatch_linked.as_path(), &passed),
                state_after(&direct_fixture, direct_linked.as_path(), &direct),
                "{arguments:?} under the opt-out differs from real Git: {}",
                stderr_of(&passed)
            );
        }
        remove(&direct_fixture);
        remove(&hatch_fixture);
        remove(&fixture);
    }
}

/// A linked worktree whose ignored `.env` holds the person's value while `tracks` tracks that path.
fn prepared_with_an_ignored_tracked_file(name: &str) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(".gitignore"), b".env\n").expect("ignore rules");
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(&fixture, repo.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=files"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "tracks"]);
    std::fs::write(repo.join(".env"), b"the tracked version\n").expect("tracked environment file");
    git(&fixture, repo.as_path(), &["add", "--force", "--", ".env"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=tracks"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::write(linked.join(".env"), b"SECRET=the person value\n").expect("the ignored file");
    return (fixture, repo, linked);
}

/// The advice a refusal prints runs as printed:
/// the command it names, with the opt-out, leaves what real Git leaves,
/// and no refusal asks for a patch `git diff` makes,
/// which holds no staged edit, no staged new file and no untracked file.
#[test]
fn the_advice_a_refusal_prints_runs_as_printed() {
    // The path form's advice is the exact command, so it is taken from the message and run.
    let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = prepared("discard-advice-run", InTheWay::Ignored);
    let refused: Observed = run_wrapped(&fixture, linked.as_path(), &["reset", "--hard", "other"]);
    let said: String = stderr_of(&refused);
    let printed: &str = said
        .split("run `")
        .nth(1)
        .and_then(|rest| return rest.split('`').next())
        .unwrap_or_default();
    assert!(printed.starts_with("git "), "the refusal prints no command: {said}");
    let words: Vec<&str> = printed.split(' ').skip(1).collect();
    let (hatch_fixture, _hatch_repo, hatch_linked): (Fixture, PathBuf, PathBuf) = prepared("discard-advice-hatch", InTheWay::Ignored);
    let passed: Observed = run_wrapped(&hatch_fixture, hatch_linked.as_path(), words.as_slice());
    let (direct_fixture, _direct_repo, direct_linked): (Fixture, PathBuf, PathBuf) = prepared("discard-advice-direct", InTheWay::Ignored);
    let direct: Observed = run_direct(&direct_fixture, direct_linked.as_path(), &["reset", "--hard", "other"]);
    assert_eq!(
        state_after(&hatch_fixture, hatch_linked.as_path(), &passed),
        state_after(&direct_fixture, direct_linked.as_path(), &direct),
        "the command the refusal prints left something else: {}",
        stderr_of(&passed)
    );
    remove(&direct_fixture);
    remove(&hatch_fixture);
    remove(&fixture);
    // Every refusal text a linked-worktree form prints is free of the patch advice that saves nothing.
    for command in git_policy_cli::rule_linked_worktree::GuardedCommand::ALL {
        for text in [
            git_policy_cli::rule_linked_worktree::dirty_worktree_message(command),
            git_policy_cli::rule_linked_worktree::main_worktree_message(command),
        ] {
            assert!(!text.contains("backup.patch"), "{command:?} advises a patch that saves nothing: {text}");
        }
    }
}

//endregion

//region The paths no other word of the command names

/// A forced `git mv` writes its destination,
/// and a forced `git checkout-index --prefix` writes the index's paths below a directory,
/// so an untracked file or a directory there is refused and named,
/// and the opt-out leaves exactly what real Git leaves.
#[test]
fn a_destination_no_word_names_is_refused_and_named() {
    for (index, arguments, person_path) in [
        (1, vec!["mv", "--force", "a.txt", "kept.txt"], "kept.txt"),
        (2, vec!["mv", "--f", "a.txt", "kept.txt"], "kept.txt"),
        (3, vec!["checkout-index", "--force", "--prefix=out/", "-a"], "out/a.txt"),
    ] {
        let label: String = format!("discard-destination-{index}");
        let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = tracked_pair(label.as_str());
        std::fs::create_dir_all(linked.join(person_path).parent().unwrap_or(linked.as_path()))
            .expect("the directory the written path is in");
        std::fs::write(linked.join(person_path), PRECIOUS).expect("the person's file at the destination");
        let before: Vec<String> = worktree_state(linked.as_path());
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(refused_by_the_guard(&observed), "{arguments:?} was not refused: {}", stderr_of(&observed));
        assert!(stderr_of(&observed).contains(person_path), "{arguments:?}: {}", stderr_of(&observed));
        assert!(still_where_it_was(linked.as_path(), person_path, PRECIOUS), "{arguments:?} lost the file");
        assert_eq!(worktree_state(linked.as_path()), before, "{arguments:?} wrote something before refusing");
        let (hatch_fixture, _hatch_repo, hatch_linked): (Fixture, PathBuf, PathBuf) =
            tracked_pair(format!("{label}-hatch").as_str());
        std::fs::create_dir_all(hatch_linked.join(person_path).parent().unwrap_or(hatch_linked.as_path()))
            .expect("the directory the written path is in");
        std::fs::write(hatch_linked.join(person_path), PRECIOUS).expect("the person's file at the destination");
        let passed: Observed = run_hatched(&hatch_fixture, hatch_linked.as_path(), arguments.as_slice());
        let (direct_fixture, _direct_repo, direct_linked): (Fixture, PathBuf, PathBuf) =
            tracked_pair(format!("{label}-direct").as_str());
        std::fs::create_dir_all(direct_linked.join(person_path).parent().unwrap_or(direct_linked.as_path()))
            .expect("the directory the written path is in");
        std::fs::write(direct_linked.join(person_path), PRECIOUS).expect("the person's file at the destination");
        let direct: Observed = run_direct(&direct_fixture, direct_linked.as_path(), arguments.as_slice());
        assert_eq!(
            state_after(&hatch_fixture, hatch_linked.as_path(), &passed),
            state_after(&direct_fixture, direct_linked.as_path(), &direct),
            "{arguments:?} under the opt-out differs from real Git"
        );
        remove(&direct_fixture);
        remove(&hatch_fixture);
        remove(&fixture);
    }
}

/// A repository with two small tracked files and a linked worktree on its own branch.
fn tracked_pair(name: &str) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("tracked file");
    std::fs::write(repo.join("b.txt"), b"bee\n").expect("tracked file");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt", "b.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    return (fixture, repo, linked);
}

/// A form that reads its pathspecs from a file writes the paths that file names,
/// so the ignored file the review's session lost is refused and named,
/// and the opt-out leaves what real Git leaves.
#[test]
fn a_pathspec_file_names_the_files_the_guard_refuses() {
    for (index, arguments) in [
        (1, vec!["restore", "--source=other", "--pathspec-from-file", "list.txt"]),
        (2, vec!["restore", "--source=other", "--pathspec-from-file=list.txt"]),
        (3, vec!["checkout", "other", "--pathspec-from-file", "list.txt"]),
        (4, vec!["checkout", "other", "--pathspec-from-file=list.txt"]),
    ] {
        let label: String = format!("discard-pathspec-file-{index}");
        let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = prepared(label.as_str(), InTheWay::Ignored);
        std::fs::write(linked.join("list.txt"), b"b.txt\n").expect("the file of paths");
        let before: Vec<String> = worktree_state(linked.as_path());
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(refused_by_the_guard(&observed), "{arguments:?} was not refused: {}", stderr_of(&observed));
        assert!(stderr_of(&observed).contains("b.txt"), "{arguments:?}: {}", stderr_of(&observed));
        assert!(still_where_it_was(linked.as_path(), "b.txt", PRECIOUS), "{arguments:?} lost the file");
        assert_eq!(worktree_state(linked.as_path()), before, "{arguments:?} wrote something");
        let (hatch_fixture, _hatch_repo, hatch_linked): (Fixture, PathBuf, PathBuf) =
            prepared(format!("{label}-hatch").as_str(), InTheWay::Ignored);
        std::fs::write(hatch_linked.join("list.txt"), b"b.txt\n").expect("the file of paths");
        let passed: Observed = run_hatched(&hatch_fixture, hatch_linked.as_path(), arguments.as_slice());
        let (direct_fixture, _direct_repo, direct_linked): (Fixture, PathBuf, PathBuf) =
            prepared(format!("{label}-direct").as_str(), InTheWay::Ignored);
        std::fs::write(direct_linked.join("list.txt"), b"b.txt\n").expect("the file of paths");
        let direct: Observed = run_direct(&direct_fixture, direct_linked.as_path(), arguments.as_slice());
        assert_eq!(
            state_after(&hatch_fixture, hatch_linked.as_path(), &passed),
            state_after(&direct_fixture, direct_linked.as_path(), &direct),
            "{arguments:?} under the opt-out differs from real Git"
        );
        remove(&direct_fixture);
        remove(&hatch_fixture);
        remove(&fixture);
    }
    // A NUL-separated file names the same path, and standard input cannot be read by the guard.
    let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = prepared("discard-pathspec-file-nul", InTheWay::Ignored);
    std::fs::write(linked.join("nul-list.txt"), b"b.txt\0").expect("the NUL-separated file");
    let nul: Observed = run_wrapped(
        &fixture,
        linked.as_path(),
        &["restore", "--source=other", "--pathspec-file-nul", "--pathspec-from-file=nul-list.txt"],
    );
    assert!(refused_by_the_guard(&nul), "the NUL-separated file was not read: {}", stderr_of(&nul));
    assert!(still_where_it_was(linked.as_path(), "b.txt", PRECIOUS), "{}", stderr_of(&nul));
    let stdin: Observed = run_wrapped(
        &fixture,
        linked.as_path(),
        &["restore", "--source=other", "--pathspec-from-file=-"],
    );
    assert!(refused_by_the_guard(&stdin), "standard input was not refused: {}", stderr_of(&stdin));
    assert!(
        stderr_of(&stdin).contains("could not read the pathspecs"),
        "the refusal says why: {}",
        stderr_of(&stdin)
    );
    remove(&fixture);
}

/// A name that is not UTF-8 below an ignored directory Git reports collapsed is named exactly,
/// without a lossy conversion standing between the listing and the worktree entry.
#[cfg(unix)]
#[test]
fn a_name_that_is_not_utf8_is_found_and_refused() {
    use std::os::unix::ffi::OsStringExt;
    let fixture: Fixture = fixture("discard-non-utf8-name");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("ignore rules");
    git(&fixture, repo.as_path(), &["add", "--", ".gitignore"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ignore"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::create_dir_all(repo.join("ignored")).expect("directory on the branch");
    let tracked_name: std::ffi::OsString = std::ffi::OsString::from_vec(vec![b'b', 0xff, b'.', b't', b'x', b't']);
    std::fs::write(repo.join("ignored").join(tracked_name.as_os_str()), b"from other\n").expect("tracked file");
    git(&fixture, repo.as_path(), &[OsStr::new("add"), OsStr::new("--force"), OsStr::new("--"), OsStr::new("ignored")]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=other"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::create_dir_all(linked.join("ignored")).expect("the ignored directory");
    std::fs::write(linked.join("ignored").join(tracked_name.as_os_str()), PRECIOUS).expect("the person's file");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["reset", "--hard", "other"]);
    assert!(refused_by_the_guard(&observed), "{}", stderr_of(&observed));
    assert_eq!(
        std::fs::read(linked.join("ignored").join(tracked_name.as_os_str())).expect("the person's file"),
        PRECIOUS,
        "{}",
        stderr_of(&observed)
    );
    assert!(stderr_of(&observed).contains("b"), "the refusal names the file: {}", stderr_of(&observed));
    remove(&fixture);
}

//endregion

//region The review's two defects

/// The bound this region gives one wrapped command:
/// four times the five-second ceiling the pathspec read answers at, so a wrapper that never answers
/// fails the control rather than holding the test.
const COMMAND_BOUND_SECONDS: &str = "30";

/// What:
///  A linked worktree of a clone that has a branch only on the remote,
///  with the person's untracked file where that branch's tree writes.
///  `remote` names the remote the clone has.
/// Why:
///  `git ls-tree topic` fails for such a branch while `git checkout -f topic` creates the local
///  branch from `origin/topic` and writes its tree
///  (measured on this host with native Git 2.56.0, 2026-11-13),
///  which is the state the guard read as "nothing is written".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { fixture, repo, linked } = cloneWithARemoteOnlyBranch('d-review-remote', 'origin');
/// ```
fn clone_with_a_remote_only_branch(name: &str, remote: &str) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let source: PathBuf = repository(&fixture, OsStr::new("source"));
    std::fs::write(source.join("tracked.txt"), b"one\n").expect("tracked file");
    git(&fixture, source.as_path(), &["add", "--", "tracked.txt"]);
    git(&fixture, source.as_path(), &["commit", "--quiet", "--message=main"]);
    git(&fixture, source.as_path(), &["checkout", "--quiet", "-b", "topic"]);
    std::fs::write(source.join("b.txt"), b"from the topic tree\n").expect("the topic's file");
    git(&fixture, source.as_path(), &["add", "--", "b.txt"]);
    git(&fixture, source.as_path(), &["commit", "--quiet", "--message=topic"]);
    git(&fixture, source.as_path(), &["checkout", "--quiet", "main"]);
    let repo: PathBuf = fixture.root.join("repo");
    git(&fixture, fixture.root.as_path(), &[OsStr::new("clone"), OsStr::new("--quiet"), source.as_os_str(), repo.as_os_str()]);
    if remote != "origin" {
        git(&fixture, repo.as_path(), &["remote", "rename", "origin", remote]);
        git(&fixture, repo.as_path(), &["fetch", "--quiet", remote]);
    }
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::write(linked.join("b.txt"), PRECIOUS).expect("the person's file");
    return (fixture, repo, linked);
}

/// What:
///  A forced `checkout` or `switch` whose tree Git reaches only by guessing a remote branch is
///  refused with the file named,
///  and the opt-out runs the command as real Git runs it.
/// Why:
///  `git checkout --no-enforce-worktree-branch -f topic` was allowed and wrote over the untracked
///  `b.txt`, because `git ls-tree topic` fails and the guard read the failure as "nothing is
///  written";
///  the branch rule's opt-out is not what let it through, which the resolvable `origin/topic` row
///  shows by being refused with the same opt-out absent.
#[test]
fn a_forced_form_over_a_remote_only_branch_is_refused_and_the_opt_out_runs_it() {
    // Each row is the command as typed, and the same command as real Git runs it:
    //  the branch rule's opt-out is the wrapper's alone, so Git never sees it.
    for (index, typed, as_git_runs_it) in [
        (
            1,
            vec!["checkout", "--no-enforce-worktree-branch", "-f", "topic"],
            vec!["checkout", "-f", "topic"],
        ),
        (
            2,
            vec!["switch", "--no-enforce-worktree-branch", "--discard-changes", "topic"],
            vec!["switch", "--discard-changes", "topic"],
        ),
        (
            3,
            vec!["checkout", "--no-enforce-worktree-branch", "-f", "origin/topic"],
            vec!["checkout", "-f", "origin/topic"],
        ),
    ] {
        let label: String = format!("discard-remote-only-branch-{index}");
        let (fixture, repo, linked): (Fixture, PathBuf, PathBuf) = clone_with_a_remote_only_branch(label.as_str(), "origin");
        let before: Vec<String> = worktree_state(linked.as_path());
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), typed.as_slice());
        assert!(refused_by_the_guard(&observed), "{typed:?} was not refused: {}", stderr_of(&observed));
        assert!(stderr_of(&observed).contains("b.txt"), "{typed:?}: {}", stderr_of(&observed));
        assert!(still_where_it_was(linked.as_path(), "b.txt", PRECIOUS), "{typed:?} lost the file");
        assert_eq!(worktree_state(linked.as_path()), before, "{typed:?} wrote something before refusing");
        assert!(saved_files(repo.as_path()).is_empty(), "{typed:?} kept a copy");
        // The opt-out runs the form and leaves what real Git leaves.
        let (hatch_fixture, _hatch_repo, hatch_linked): (Fixture, PathBuf, PathBuf) =
            clone_with_a_remote_only_branch(format!("{label}-hatch").as_str(), "origin");
        let passed: Observed = run_hatched(&hatch_fixture, hatch_linked.as_path(), typed.as_slice());
        let (direct_fixture, _direct_repo, direct_linked): (Fixture, PathBuf, PathBuf) =
            clone_with_a_remote_only_branch(format!("{label}-direct").as_str(), "origin");
        let direct: Observed = run_direct(&direct_fixture, direct_linked.as_path(), as_git_runs_it.as_slice());
        assert_eq!(
            state_after(&hatch_fixture, hatch_linked.as_path(), &passed),
            state_after(&direct_fixture, direct_linked.as_path(), &direct),
            "{typed:?} under the opt-out differs from real Git: {}",
            stderr_of(&passed)
        );
        remove(&direct_fixture);
        remove(&hatch_fixture);
        remove(&fixture);
    }
}

/// What:
///  The ways a word becomes a tree, each through the built wrapper:
///  a remote whose name holds a slash and `checkout.defaultRemote` are refused (Git would write the
///  tree),
///  and a word Git itself refuses is left to Git and loses nothing.
/// Why:
///  The guess the guard makes must be the guess Git makes.
///  A repository with two remotes carrying the branch is an ambiguity Git refuses
///  (`hint: Branch name 'topic' appears in multiple remotes`),
///  `--no-guess` and `checkout.guess=false` make Git refuse the short name,
///  and an unknown word is Git's own error;
///  in each the guard must not refuse a command Git would not have run.
#[test]
fn the_guess_the_guard_makes_is_the_guess_git_makes() {
    // A remote whose own name holds a slash: Git guesses the branch from it.
    let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = clone_with_a_remote_only_branch("discard-slashed-remote", "a/b");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["checkout", "--no-enforce-worktree-branch", "-f", "topic"]);
    assert!(refused_by_the_guard(&observed), "a slashed remote was not read: {}", stderr_of(&observed));
    assert!(still_where_it_was(linked.as_path(), "b.txt", PRECIOUS));
    remove(&fixture);

    // Two remotes carrying the branch: Git refuses, and the guard leaves the command to Git.
    let (two, two_repo, two_linked): (Fixture, PathBuf, PathBuf) = clone_with_a_remote_only_branch("discard-two-remotes", "origin");
    // The source repository is the one that holds the branch, so the second remote carries it too.
    let source: PathBuf = two.root.join("source");
    git(&two, two_repo.as_path(), &[OsStr::new("remote"), OsStr::new("add"), OsStr::new("second"), source.as_os_str()]);
    git(&two, two_repo.as_path(), &["fetch", "--quiet", "second"]);
    let ambiguous: Observed = run_wrapped(&two, two_linked.as_path(), &["checkout", "--no-enforce-worktree-branch", "-f", "topic"]);
    assert!(!refused_by_the_guard(&ambiguous), "Git refuses the ambiguous name itself: {}", stderr_of(&ambiguous));
    assert!(stderr_of(&ambiguous).contains("multiple remotes"), "{}", stderr_of(&ambiguous));
    assert!(still_where_it_was(two_linked.as_path(), "b.txt", PRECIOUS));
    // `checkout.defaultRemote` names the one Git then writes, so the guard refuses over its tree.
    git(&two, two_repo.as_path(), &["config", "checkout.defaultRemote", "second"]);
    let chosen: Observed = run_wrapped(&two, two_linked.as_path(), &["checkout", "--no-enforce-worktree-branch", "-f", "topic"]);
    assert!(refused_by_the_guard(&chosen), "the default remote was not read: {}", stderr_of(&chosen));
    assert!(still_where_it_was(two_linked.as_path(), "b.txt", PRECIOUS));
    remove(&two);

    // The switches that turn the guess off, and a word no branch matches: Git refuses each.
    for (index, arguments) in [
        (1, vec!["checkout", "--no-enforce-worktree-branch", "--no-guess", "-f", "topic"]),
        (2, vec!["checkout", "--no-enforce-worktree-branch", "-f", "nosuchbranch"]),
    ] {
        let label: String = format!("discard-guess-off-{index}");
        let (off, _off_repo, off_linked): (Fixture, PathBuf, PathBuf) = clone_with_a_remote_only_branch(label.as_str(), "origin");
        if index == 1 {
            // The same word with the guess on is refused, so the row's own control is the switch.
            let on: Observed = run_wrapped(&off, off_linked.as_path(), &["checkout", "--no-enforce-worktree-branch", "-f", "topic"]);
            assert!(refused_by_the_guard(&on), "{}", stderr_of(&on));
        }
        let left_to_git: Observed = run_wrapped(&off, off_linked.as_path(), arguments.as_slice());
        assert!(
            !refused_by_the_guard(&left_to_git),
            "{arguments:?} was refused where Git refuses the name: {}",
            stderr_of(&left_to_git)
        );
        assert!(still_where_it_was(off_linked.as_path(), "b.txt", PRECIOUS));
        remove(&off);
    }
}

/// What:
///  A tree Git knows and cannot list is a refusal of the guard,
///  never a form that writes nothing.
/// Why:
///  The review's cause was one swallow: a failing probe read as "nothing to guard".
///  A transient failed read of an object answers the same way as a name that never resolved,
///  so the guard tells them apart and refuses the first.
#[test]
fn a_tree_git_cannot_list_is_refused_rather_than_read_as_nothing() {
    let (fixture, repo, linked): (Fixture, PathBuf, PathBuf) = clone_with_a_remote_only_branch("discard-unreadable-tree", "origin");
    let tree: String = String::from_utf8_lossy(&git(&fixture, repo.as_path(), &["rev-parse", "origin/topic^{tree}"]).stdout).trim().to_owned();
    std::fs::remove_file(repo.join(".git").join("objects").join(&tree[..2]).join(&tree[2..])).expect("the tree object");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["checkout", "--no-enforce-worktree-branch", "-f", "origin/topic"]);
    assert!(refused_by_the_guard(&observed), "{}", stderr_of(&observed));
    assert!(
        stderr_of(&observed).contains("could not read the tree"),
        "the refusal says what it could not read: {}",
        stderr_of(&observed)
    );
    assert!(still_where_it_was(linked.as_path(), "b.txt", PRECIOUS));
    remove(&fixture);
}

/// What:
///  A `--pathspec-from-file` that never yields bytes is refused at the ceiling,
///  where the wrapper used to wait until something killed it.
/// Why:
///  Git itself waits with no end for such a file
///  (`timeout 10 git checkout --pathspec-from-file=<fifo>` was still waiting when the probe ended
///  it, measured on this host with native Git 2.56.0, 2026-11-13),
///  and the wrapper's rule is that nothing waits forever.
///  A regular file is the control that the read itself is unchanged.
#[test]
fn a_pathspec_file_that_never_yields_bytes_is_refused_at_its_ceiling() {
    let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = tracked_pair("discard-pathspec-fifo");
    let regular: PathBuf = fixture.root.join("specs.txt");
    std::fs::write(regular.as_path(), b"a.txt\n").expect("the regular list");
    let prompt: Observed = run_wrapped(&fixture, linked.as_path(), &["checkout", "--pathspec-from-file", regular.to_str().expect("utf-8 path")]);
    assert_eq!(prompt.code, Some(0), "the control needs a regular file to be read at once: {}", stderr_of(&prompt));

    let fifo: PathBuf = fixture.root.join("specs.fifo");
    let made = std::process::Command::new("/usr/bin/mkfifo").arg(&fifo).status().expect("mkfifo runs");
    assert!(made.success(), "the fixture needs a FIFO");
    // The command's own bound is shorter than the harness backstop, so a wrapper that never answers
    //  fails this control in half a minute instead of holding the test.
    let mut command: std::process::Command = bounded_for(
        &fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(&fixture).as_os_str(),
        COMMAND_BOUND_SECONDS,
    );
    command.current_dir(linked.as_path()).args(["checkout", "--pathspec-from-file", fifo.to_str().expect("utf-8 path")]);
    let began: std::time::Instant = std::time::Instant::now();
    let observed: Observed = observe(&mut command, b"");
    let waited: std::time::Duration = began.elapsed();
    assert!(refused_by_the_guard(&observed), "{}", stderr_of(&observed));
    assert!(
        stderr_of(&observed).contains("gave no bytes within"),
        "the refusal says the file did not answer: {}",
        stderr_of(&observed)
    );
    assert!(
        waited < std::time::Duration::from_secs(120),
        "the wait ended only at the harness bound, {waited:?}, not at the read ceiling"
    );
    remove(&fixture);
}

//endregion

//region The second review's ten losses, each refused now

/// What:
///  A linked worktree holding the person's untracked file where the form's tree writes,
///  with the list Git reads from a file the control writes.
///  `&str` names the file inside the fixture.
/// Why:
///  Every one of the review's sequences starts from that shape and differs in the words the
///  command reads its paths from, so one helper keeps the rows about the words alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const list = writeTheList(fixture, 'list', Buffer.from('b.txt\n'));
/// ```
fn write_the_list(fixture: &Fixture, name: &str, contents: &[u8]) -> PathBuf {
    let list: PathBuf = fixture.root.join(name);
    std::fs::write(list.as_path(), contents).expect("the list");
    return list;
}

/// A zero-byte pathspec file is no pathspec to Git, which then switches the branch by force and
/// writes the whole tree,
/// so the guard refuses it with the file named,
/// and the opt-out runs the whole-tree form as Git runs it.
#[test]
fn an_empty_pathspec_file_is_the_whole_tree_git_writes() {
    // A local branch: `checkout -f other --pathspec-from-file=<empty>` switches to it.
    let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = prepared("discard-empty-list-local", InTheWay::File);
    let empty: PathBuf = write_the_list(&fixture, "empty-list", b"");
    let typed: Vec<String> = vec![
        String::from("checkout"),
        String::from("-f"),
        String::from("other"),
        format!("--pathspec-from-file={}", empty.display()),
    ];
    let borrowed: Vec<&str> = typed.iter().map(|word| return word.as_str()).collect::<Vec<&str>>();
    let before: Vec<String> = worktree_state(linked.as_path());
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), borrowed.as_slice());
    assert!(refused_by_the_guard(&observed), "an empty list let a forced switch run: {}", stderr_of(&observed));
    assert!(still_where_it_was(linked.as_path(), "b.txt", PRECIOUS), "the switch lost the file");
    assert_eq!(worktree_state(linked.as_path()), before, "the switch wrote before refusing");
    let (hatch_fixture, _hatch_repo, hatch_linked): (Fixture, PathBuf, PathBuf) =
        prepared("discard-empty-list-local-hatch", InTheWay::File);
    let hatch_list: PathBuf = write_the_list(&hatch_fixture, "empty-list", b"");
    let passed: Observed = run_hatched(
        &hatch_fixture,
        hatch_linked.as_path(),
        &["checkout", "-f", "other", format!("--pathspec-from-file={}", hatch_list.display()).as_str()],
    );
    let (direct_fixture, _direct_repo, direct_linked): (Fixture, PathBuf, PathBuf) =
        prepared("discard-empty-list-local-direct", InTheWay::File);
    let direct: Observed = run_direct(&direct_fixture, direct_linked.as_path(), &["checkout", "-f", "other"]);
    assert_eq!(
        state_after(&hatch_fixture, hatch_linked.as_path(), &passed),
        state_after(&direct_fixture, direct_linked.as_path(), &direct),
        "the opt-out differs from real Git: {}",
        stderr_of(&passed)
    );
    remove(&direct_fixture);
    remove(&hatch_fixture);
    remove(&fixture);

    // A remote-only branch: the same empty list leaves the whole-tree form, which guesses it.
    let (remote, _remote_repo, remote_linked): (Fixture, PathBuf, PathBuf) =
        clone_with_a_remote_only_branch("discard-empty-list-remote", "origin");
    let remote_empty: PathBuf = write_the_list(&remote, "empty-list", b"");
    let remote_typed: Vec<String> = vec![
        String::from("checkout"),
        String::from("--no-enforce-worktree-branch"),
        String::from("-f"),
        String::from("topic"),
        format!("--pathspec-from-file={}", remote_empty.display()),
    ];
    let remote_words: Vec<&str> = remote_typed.iter().map(|word| return word.as_str()).collect::<Vec<&str>>();
    let guessed: Observed = run_wrapped(&remote, remote_linked.as_path(), remote_words.as_slice());
    assert!(refused_by_the_guard(&guessed), "the guessed whole-tree write ran: {}", stderr_of(&guessed));
    assert!(still_where_it_was(remote_linked.as_path(), "b.txt", PRECIOUS), "{}", stderr_of(&guessed));
    remove(&remote);
}

/// A line that begins with a double quote is the C-quoted name Git reads,
/// so a form over a file whose name Git prints that way is refused with the name.
#[test]
fn a_quoted_pathspec_entry_names_the_file_git_writes() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let fixture: Fixture = fixture("discard-quoted-pathspec-entry");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let tabbed: OsString = OsString::from_vec(b"a\tb.txt".to_vec());
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(&fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repo.join(tabbed.as_os_str()), b"from the other tree\n").expect("the tabbed name on the branch");
    git(&fixture, repo.as_path(), &[OsStr::new("add"), OsStr::new("--"), tabbed.as_os_str()]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=tabbed"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::write(linked.join(tabbed.as_os_str()), PRECIOUS).expect("the person's untracked file");
    let list: PathBuf = write_the_list(&fixture, "quoted-list", b"\"a\\tb.txt\"\n");
    let typed: Vec<String> = vec![
        String::from("checkout"),
        String::from("other"),
        format!("--pathspec-from-file={}", list.display()),
    ];
    let borrowed: Vec<&str> = typed.iter().map(|word| return word.as_str()).collect::<Vec<&str>>();
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), borrowed.as_slice());
    assert!(refused_by_the_guard(&observed), "the quoted entry was not read: {}", stderr_of(&observed));
    assert!(
        std::fs::read(linked.join(tabbed.as_os_str())).is_ok_and(|bytes| return bytes == PRECIOUS),
        "the file Git writes by that name is still the person's"
    );
    // The control: the same path named as a word is refused too, so the row is about the quoting alone.
    let named: Observed = run_wrapped(&fixture, linked.as_path(), &["checkout", "other", "--", "a\tb.txt"]);
    assert!(refused_by_the_guard(&named), "the named path is refused as well: {}", stderr_of(&named));
    remove(&fixture);
}

/// A source written twice is the last one, which is the tree Git writes.
#[test]
fn a_source_written_twice_is_the_last_one() {
    let fixture: Fixture = fixture("discard-last-source");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(&fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repo.join("f.txt"), b"from the other tree\n").expect("the other tree's file");
    git(&fixture, repo.as_path(), &["add", "--", "f.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=other"]);
    git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::write(linked.join("f.txt"), PRECIOUS).expect("the person's untracked file");
    for (index, arguments) in [
        (1, vec!["restore", "-s", "main", "-s", "other", "--", "f.txt"]),
        (2, vec!["restore", "--source=main", "--source=other", "--", "f.txt"]),
    ] {
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(refused_by_the_guard(&observed), "{arguments:?} was let through: {}", stderr_of(&observed));
        assert!(still_where_it_was(linked.as_path(), "f.txt", PRECIOUS), "{arguments:?} lost the file");
        // The control: the first source alone holds no `f.txt`, so Git refuses the form itself and the guard stays out.
        assert!(index > 0);
    }
    let control: Observed = run_wrapped(&fixture, linked.as_path(), &["restore", "-s", "main", "--", "f.txt"]);
    assert!(!refused_by_the_guard(&control), "Git refuses a source without the path: {}", stderr_of(&control));
    assert!(still_where_it_was(linked.as_path(), "f.txt", PRECIOUS));
    // The opt-out runs the last source, as real Git does.
    let passed: Observed = run_hatched(&fixture, linked.as_path(), &["restore", "-s", "main", "-s", "other", "--", "f.txt"]);
    assert_eq!(passed.code, Some(0), "{}", stderr_of(&passed));
    assert_eq!(
        std::fs::read(linked.join("f.txt")).expect("the written file"),
        b"from the other tree\n",
        "the opt-out wrote what the last source holds"
    );
    remove(&fixture);
}

/// What:
///  The four words a command takes from bytes that are not text,
///  each written the way the review wrote it.
/// Why:
///  A prefix,
///  a source and a pathspec file's name are paths and revisions,
///  and a conversion through text names another file;
///  each of these sequences wrote over the person's file through such a conversion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // checkout-index -f --prefix=p-<0xFF>/ --all . read-tree -u --prefix=p-<0xFF>/ other .
/// // restore --source=br-<0xFF> -- f.txt . checkout -f other --pathspec-from-file=n-<0xFF>
/// ```
#[cfg(unix)]
#[test]
fn a_word_that_is_not_text_is_the_word_git_reads() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    /// `p-<0xFF>/`, the directory the review used.
    fn a_prefix() -> OsString {
        return OsString::from_vec(b"p-\xff/".to_vec());
    }
    // a. `checkout-index -f --prefix=p-<0xFF>/ --all` over an untracked file below that prefix.
    {
        let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = tracked_pair("discard-nontext-checkout-index");
        std::fs::create_dir_all(linked.join(a_prefix().as_os_str())).expect("the prefix directory");
        std::fs::write(linked.join(a_prefix().as_os_str()).join("a.txt"), PRECIOUS).expect("the person's file");
        let mut prefix: Vec<u8> = b"--prefix=".to_vec();
        prefix.extend(a_prefix().as_encoded_bytes());
        let observed: Observed = run_wrapped_os(
            &fixture,
            linked.as_path(),
            &[
                OsString::from("checkout-index"),
                OsString::from("-f"),
                OsString::from("--all"),
                OsString::from_vec(prefix),
            ],
        );
        assert!(refused_by_letter_or_guard(&observed), "checkout-index wrote over the file: {}", stderr_of(&observed));
        assert!(
            std::fs::read(linked.join(a_prefix().as_os_str()).join("a.txt")).is_ok_and(|bytes| return bytes == PRECIOUS),
            "the person's file below the prefix is still there"
        );
        remove(&fixture);
    }
    // b. `read-tree -u --prefix=p-<0xFF>/ other` over an ignored file below that prefix.
    {
        let fixture: Fixture = fixture("discard-nontext-read-tree");
        let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
        std::fs::write(repo.join(".gitignore"), b"p-*\n").expect("ignore rules");
        std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
        git(&fixture, repo.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
        git(&fixture, repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
        std::fs::write(repo.join("b.txt"), b"from the other tree\n").expect("the other tree's file");
        git(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=other"]);
        git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
        let linked: PathBuf = fixture.root.join("linked");
        git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
        std::fs::create_dir_all(linked.join(a_prefix().as_os_str())).expect("the prefix directory");
        std::fs::write(linked.join(a_prefix().as_os_str()).join("b.txt"), PRECIOUS).expect("the person's ignored file");
        let mut prefix: Vec<u8> = b"--prefix=".to_vec();
        prefix.extend(a_prefix().as_encoded_bytes());
        let observed: Observed = run_wrapped_os(
            &fixture,
            linked.as_path(),
            &[OsString::from("read-tree"), OsString::from("-u"), OsString::from_vec(prefix), OsString::from("other")],
        );
        assert!(refused_by_letter_or_guard(&observed), "read-tree wrote over the file: {}", stderr_of(&observed));
        assert!(
            std::fs::read(linked.join(a_prefix().as_os_str()).join("b.txt")).is_ok_and(|bytes| return bytes == PRECIOUS),
            "the ignored file below the prefix is still there"
        );
        remove(&fixture);
    }
    // c. `restore --source=br-<0xFF> -- f.txt` from a branch whose name is those bytes.
    {
        let fixture: Fixture = fixture("discard-nontext-source");
        let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
        let branch: OsString = OsString::from_vec(b"br-\xff".to_vec());
        std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
        git(&fixture, repo.as_path(), &["add", "--", "tracked.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
        git(&fixture, repo.as_path(), &[OsStr::new("checkout"), OsStr::new("--quiet"), OsStr::new("-b"), branch.as_os_str()]);
        std::fs::write(repo.join("f.txt"), b"from the branch\n").expect("the branch's file");
        git(&fixture, repo.as_path(), &["add", "--", "f.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=branch"]);
        git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
        let linked: PathBuf = fixture.root.join("linked");
        git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
        std::fs::write(linked.join("f.txt"), PRECIOUS).expect("the person's untracked file");
        let mut source: Vec<u8> = b"--source=".to_vec();
        source.extend_from_slice(branch.as_encoded_bytes());
        let inline: OsString = OsString::from_vec(source);
        let observed: Observed = run_wrapped_os(&fixture, linked.as_path(), &[OsString::from("restore"), inline.clone(), OsString::from("--"), OsString::from("f.txt")]);
        assert!(refused_by_letter_or_guard(&observed), "restore wrote over the file: {}", stderr_of(&observed));
        assert!(still_where_it_was(linked.as_path(), "f.txt", PRECIOUS), "the person's file is still there");
        remove(&fixture);
    }
    // d. `checkout -f other --pathspec-from-file=n-<0xFF>` with an empty decoy at the lossy name.
    {
        let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = prepared("discard-nontext-pathspec-file", InTheWay::File);
        let mut real: Vec<u8> = b"n-".to_vec();
        real.push(0xff);
        let real_path: PathBuf = fixture.root.join(std::ffi::OsString::from_vec(real.clone()));
        std::fs::write(real_path.as_path(), b"b.txt\n").expect("the real list");
        // The name a conversion through text would produce, holding no pathspec at all.
        let decoy: PathBuf = fixture.root.join("n-\u{fffd}");
        std::fs::write(decoy.as_path(), b"").expect("the decoy");
        let mut word: Vec<u8> = b"--pathspec-from-file=".to_vec();
        word.extend_from_slice(real.as_slice());
        let observed: Observed = run_wrapped_os(
            &fixture,
            linked.as_path(),
            &[OsString::from("checkout"), OsString::from("-f"), OsString::from("other"), OsString::from_vec(word)],
        );
        assert!(refused_by_letter_or_guard(&observed), "the list's name was misread: {}", stderr_of(&observed));
        assert!(still_where_it_was(linked.as_path(), "b.txt", PRECIOUS), "the person's file is still there");
        remove(&fixture);
    }
}

/// The same refusal test for a run whose words are not text.
fn refused_by_letter_or_guard(observed: &Observed) -> bool {
    return refused_by_the_guard(observed);
}

/// Run one wrapped command whose words may hold bytes that are not text.
fn run_wrapped_os(fixture: &Fixture, directory: &Path, arguments: &[std::ffi::OsString]) -> Observed {
    let mut command: std::process::Command = bounded_for(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        COMMAND_BOUND_SECONDS,
    );
    command.current_dir(directory).args(arguments);
    return observe(&mut command, b"");
}

/// What:
///  A linked worktree whose ignored directory holds a repository of the given shape,
///  with a force-and-ignore clean that would delete it.
///  `bool` says whether the repository is bare.
/// Why:
///  Git's one force keeps a directory with a `.git` entry and deletes a bare one
///  (`Skipping repository inner` against `Removing bare/`, measured on this host with native Git
///  2.56.0, 2026-11-13),
///  which is the difference the interactive cap has to know about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { fixture, linked } = ignoredRepository('bare-menu', true);
/// ```
fn ignored_repository(name: &str, bare: bool) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(".gitignore"), b"ignored/\n").expect("ignore rules");
    std::fs::write(repo.join("tracked.txt"), b"one\n").expect("tracked file");
    git(&fixture, repo.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=files"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    let inside: PathBuf = linked.join("ignored");
    std::fs::create_dir_all(inside.as_path()).expect("the ignored directory");
    let nested: PathBuf = inside.join(if bare { "bare" } else { "inner" });
    std::fs::create_dir_all(nested.as_path()).expect("the repository's directory");
    if bare {
        git(&fixture, nested.as_path(), &["init", "--quiet", "--bare"]);
    } else {
        git(&fixture, nested.as_path(), &["init", "--quiet", "--initial-branch=inner"]);
    }
    std::fs::write(nested.join("uncommitted.txt"), b"the person's work inside it\n").expect("untracked work");
    return (fixture, repo, linked);
}

/// An interactive clean whose menu would offer a repository Git's one force keeps is capped and says so;
/// one whose menu would offer a bare repository, which one force deletes, stops instead with the repository named.
#[test]
fn an_interactive_clean_that_would_offer_a_bare_repository_is_refused() {
    // The bare shape: refused, with the repository named and the two ways forward.
    let (bare_fixture, _bare_repo, bare_linked): (Fixture, PathBuf, PathBuf) = ignored_repository("discard-menu-bare", true);
    let refused: Observed = run_wrapped(&bare_fixture, bare_linked.as_path(), &["clean", "-i", "-f", "-f", "-X", "-d"]);
    let said: String = stderr_of(&refused);
    assert!(said.contains("core-incomplete"), "the clean was not stopped: {said}");
    assert!(said.contains("bare"), "the refusal names the bare repository: {said}");
    assert!(said.contains("no-enforce-worktree"), "the refusal names the way past it: {said}");
    assert!(
        bare_linked.join("ignored/bare/uncommitted.txt").exists(),
        "the work inside the bare repository is still there"
    );
    remove(&bare_fixture);

    // The `.git` shape: capped, the repository kept, and the menu answered with "clean".
    let (nested_fixture, _nested_repo, nested_linked): (Fixture, PathBuf, PathBuf) = ignored_repository("discard-menu-nested", false);
    let mut cleaning: std::process::Command = bounded_for(
        &nested_fixture,
        nested_fixture.root.join("bin/git").as_path(),
        wrapper_first_path(&nested_fixture).as_os_str(),
        COMMAND_BOUND_SECONDS,
    );
    cleaning.current_dir(nested_linked.as_path()).args(["clean", "-i", "-f", "-f", "-X", "-d"]);
    let capped: Observed = observe(&mut cleaning, b"1\n");
    assert_eq!(capped.code, Some(0), "{}", stderr_of(&capped));
    assert!(
        stderr_of(&capped).contains("runs with one force"),
        "the notice says what the cap does: {}",
        stderr_of(&capped)
    );
    assert!(
        nested_linked.join("ignored/inner/uncommitted.txt").exists(),
        "the non-bare repository is kept, as one force keeps it: {}",
        stderr_of(&capped)
    );
    // The control that the non-interactive clean still saves the bare repository whole.
    let (saving_fixture, saving_repo, saving_linked): (Fixture, PathBuf, PathBuf) = ignored_repository("discard-menu-saves-bare", true);
    let saved: Observed = run_wrapped(&saving_fixture, saving_linked.as_path(), &["clean", "-f", "-f", "-X", "-d"]);
    assert_eq!(saved.code, Some(0), "{}", stderr_of(&saved));
    assert!(!saving_linked.join("ignored/bare").exists(), "the clean deleted the repository as typed");
    assert!(!saved_files(saving_repo.as_path()).is_empty(), "the repository is below the saved copies");
    remove(&saving_fixture);
    remove(&nested_fixture);
}

/// An interactive clean a `!` shell alias runs is stopped rather than run with a menu offering a repository:
/// the cap's token has no place in an alias's own words, and Git deletes from that menu unsaved.
#[test]
fn an_interactive_clean_a_shell_alias_runs_is_refused() {
    let (fixture, repo, linked): (Fixture, PathBuf, PathBuf) = ignored_repository("discard-menu-alias", false);
    git(&fixture, repo.as_path(), &["config", "alias.wipe", "!git clean -i -f -f -X -d"]);
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["wipe"]);
    let said: String = stderr_of(&observed);
    assert!(said.contains("core-incomplete"), "the alias was not stopped: {said}");
    assert!(said.contains("ignored/inner") || said.contains("inner"), "the refusal names the repository: {said}");
    assert!(said.contains("no-enforce-worktree"), "the refusal names the way past it: {said}");
    assert!(
        linked.join("ignored/inner/uncommitted.txt").exists(),
        "the work inside the repository is still there"
    );
    // The control: the same alias without a menu saves the repository, so the alias route works everywhere else.
    let (control_fixture, control_repo, control_linked): (Fixture, PathBuf, PathBuf) = ignored_repository("discard-menu-alias-control", false);
    git(&control_fixture, control_repo.as_path(), &["config", "alias.sweep", "!git clean -f -f -X -d"]);
    let swept: Observed = run_wrapped(&control_fixture, control_linked.as_path(), &["sweep"]);
    assert_eq!(swept.code, Some(0), "{}", stderr_of(&swept));
    assert!(!control_linked.join("ignored/inner").exists(), "the clean deleted it as typed");
    assert!(!saved_files(control_repo.as_path()).is_empty(), "the repository is below the saved copies");
    remove(&control_fixture);
    remove(&fixture);
}

//endregion
//region The third review's two findings

/// What:
///  A linked worktree of a repository whose `main` tracks `a.txt`, `x.txt` and `d1/inner.txt`,
///  with `setup` laying what one move shape needs and no marking of the worktree.
/// Why:
///  A forced move's written paths turn on whether its destination is a directory,
///  which only the worktree says, so each shape needs its own fixture beside real Git's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { fixture, linked } = a_worktree_for_moving('move-shape', (linked) => mkdir(linked + '/b'));
/// ```
fn a_worktree_for_moving(name: &str, setup: fn(&Path)) -> (Fixture, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"tracked a\n").expect("a.txt");
    std::fs::write(repo.join("x.txt"), b"tracked x\n").expect("x.txt");
    std::fs::create_dir(repo.join("d1")).expect("d1");
    std::fs::write(repo.join("d1/inner.txt"), b"inside d1\n").expect("d1/inner.txt");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt", "x.txt", "d1/inner.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    setup(linked.as_path());
    return (fixture, linked);
}

/// What:
///  A forced `mv` into a directory Git only moves into is allowed and loses nothing,
///  while a move Git really writes over an untracked file is refused with that file named,
///  and the refusal's command runs the move as real Git runs it.
/// Why:
///  The third review's defect 1: the reading named the destination directory itself as written over,
///  so a safe `git mv -f a.txt b` was refused with a sentence that was untrue.
///  The control on the other side is the same form over a file at the destination, which Git replaces.
#[test]
fn a_forced_move_into_a_directory_is_allowed_and_a_move_over_a_file_is_refused() {
    fn an_untracked_directory(linked: &Path) {
        std::fs::create_dir(linked.join("b")).expect("b");
        std::fs::write(linked.join("b/keep.txt"), b"keep\n").expect("b/keep.txt");
    }
    fn nothing(linked: &Path) {
        let _ = linked;
    }
    // A source into an untracked directory: Git moves into it, and the guard lets it run.
    let (fixture, linked): (Fixture, PathBuf) = a_worktree_for_moving("discard-move-into-a-directory", an_untracked_directory);
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["mv", "-f", "a.txt", "b"]);
    assert_eq!(observed.code, Some(0), "a safe move was refused: {}", stderr_of(&observed));
    assert!(!refused_by_the_guard(&observed), "{}", stderr_of(&observed));
    assert!(linked.join("b/a.txt").exists(), "Git moved the source into the directory");
    assert_eq!(
        std::fs::read(linked.join("b/keep.txt")).expect("the file beside the moved one"),
        b"keep\n",
        "the file beside the moved one is untouched"
    );
    assert_eq!(
        std::fs::read(linked.join("x.txt")).expect("the file the move does not name"),
        b"tracked x\n",
        "a move into a directory writes no other entry"
    );
    // Several sources into a directory: Git moves both, and the guard lets it run.
    let (two_fixture, two_linked): (Fixture, PathBuf) = a_worktree_for_moving("discard-move-two-into-a-directory", an_untracked_directory);
    let two: Observed = run_wrapped(&two_fixture, two_linked.as_path(), &["mv", "-f", "a.txt", "x.txt", "b"]);
    assert_eq!(two.code, Some(0), "a safe move of two sources was refused: {}", stderr_of(&two));
    assert!(two_linked.join("b/a.txt").exists() && two_linked.join("b/x.txt").exists());
    remove(&two_fixture);
    // A source onto an untracked file: Git replaces it, and the guard refuses with the file named.
    let (on_file, on_linked): (Fixture, PathBuf) = a_worktree_for_moving("discard-move-onto-a-file", nothing);
    std::fs::write(on_linked.join("kept.txt"), PRECIOUS).expect("the person's untracked file");
    let replaced: Observed = run_wrapped(&on_file, on_linked.as_path(), &["mv", "-f", "a.txt", "kept.txt"]);
    assert!(refused_by_the_guard(&replaced), "a move that replaces a file was let through: {}", stderr_of(&replaced));
    assert!(stderr_of(&replaced).contains("kept.txt"), "{}", stderr_of(&replaced));
    assert!(still_where_it_was(on_linked.as_path(), "kept.txt", PRECIOUS));
    // The command the refusal prints runs the move as real Git runs it.
    let passed: Observed = run_hatched(&on_file, on_linked.as_path(), &["mv", "-f", "a.txt", "kept.txt"]);
    assert_eq!(passed.code, Some(0), "{}", stderr_of(&passed));
    assert!(!still_where_it_was(on_linked.as_path(), "kept.txt", PRECIOUS), "the move replaced the file");
    // A file at the source's name inside the destination: Git refuses (`Cannot overwrite`), so the guard names nothing.
    fn a_file_where_the_source_lands(linked: &Path) {
        std::fs::create_dir(linked.join("b")).expect("b");
        std::fs::write(linked.join("b/a.txt"), PRECIOUS).expect("the person's file at the landing name");
    }
    let (landing, landing_linked): (Fixture, PathBuf) = a_worktree_for_moving("discard-move-onto-a-landing-file", a_file_where_the_source_lands);
    let landing_observed: Observed = run_wrapped(&landing, landing_linked.as_path(), &["mv", "-f", "a.txt", "b"]);
    assert!(refused_by_the_guard(&landing_observed), "a move over the landing name was let through: {}", stderr_of(&landing_observed));
    assert!(still_where_it_was(landing_linked.as_path(), "b/a.txt", PRECIOUS));
    remove(&landing);
    remove(&on_file);
    remove(&fixture);
}

/// What:
///  An edit hidden by `skip-worktree` or `assume-unchanged`, at the only path a `git rm` can reach
///  it through.
///  `&str` names the mark.
/// Why:
///  `--sparse` is what makes Git reach a skipped path and delete the file with the edit in it,
///  and the third review's defect 2: the guard asked the status alone for `rm`, which shows neither
///  mark.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linked = aWorktreeWithAHiddenEdit('hidden-edit', 'skip-worktree');
/// ```
fn a_worktree_with_a_hidden_edit(name: &str, mark: &str) -> (Fixture, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("s.txt"), b"committed content\n").expect("s.txt");
    git(&fixture, repo.as_path(), &["add", "--", "s.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    git(&fixture, linked.as_path(), &[OsStr::new("update-index"), OsStr::new(&format!("--{mark}")), OsStr::new("s.txt")]);
    std::fs::write(linked.join("s.txt"), PRECIOUS).expect("the hidden edit");
    return (fixture, linked);
}

/// A `git rm` that reaches a hidden edit is refused with the file named, in both marks,
/// and the opt-out runs the command, which deletes it as real Git does;
/// each mark keeps the form that does not reach its hidden edit, with the control that shows it.
#[test]
fn a_removal_that_reaches_a_hidden_edit_is_refused_and_the_opt_out_deletes_it() {
    for (index, mark) in ["skip-worktree", "assume-unchanged"].into_iter().enumerate() {
        // The guarded form that reaches the mark: `--sparse` makes Git look past a `skip-worktree`
        //  bit, and an `assume-unchanged` mark is no obstacle at all, so both are a loss.
        let label: String = format!("discard-hidden-edit-{index}");
        let (fixture, linked): (Fixture, PathBuf) = a_worktree_with_a_hidden_edit(label.as_str(), mark);
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["rm", "-f", "--sparse", "s.txt"]);
        assert!(
            refused_by_the_guard(&observed),
            "{mark}: `rm -f --sparse s.txt` was let through: {}",
            stderr_of(&observed)
        );
        assert!(stderr_of(&observed).contains("s.txt"), "{mark}: {}", stderr_of(&observed));
        assert!(still_where_it_was(linked.as_path(), "s.txt", PRECIOUS), "{mark}: the refusal lost the edit");
        // The opt-out runs the command, and real Git deletes the file with the edit in it.
        let passed: Observed = run_hatched(&fixture, linked.as_path(), &["rm", "-f", "--sparse", "s.txt"]);
        assert_eq!(passed.code, Some(0), "{mark} under the opt-out: {}", stderr_of(&passed));
        assert!(!linked.join("s.txt").exists(), "{mark} under the opt-out kept the file Git deletes");
        remove(&fixture);

        // What each mark does to the forms that do not reach it.
        let (quiet_fixture, quiet_linked): (Fixture, PathBuf) =
            a_worktree_with_a_hidden_edit(format!("{label}-quiet").as_str(), mark);
        if mark == "skip-worktree" {
            // Git refuses a plain `rm -f` on a skipped path itself, so the guard leaves the command to Git.
            let plain: Observed = run_wrapped(&quiet_fixture, quiet_linked.as_path(), &["rm", "-f", "s.txt"]);
            assert!(!refused_by_the_guard(&plain), "{mark}: {}", stderr_of(&plain));
            assert!(still_where_it_was(quiet_linked.as_path(), "s.txt", PRECIOUS), "{mark}: the plain `rm -f` lost it");
            // `--cached` removes the index entry and keeps the worktree file, so it is left to Git too.
            let cached: Observed =
                run_wrapped(&quiet_fixture, quiet_linked.as_path(), &["rm", "-f", "--cached", "--sparse", "s.txt"]);
            assert!(!refused_by_the_guard(&cached), "{mark}: `rm --cached` keeps the file: {}", stderr_of(&cached));
            assert!(still_where_it_was(quiet_linked.as_path(), "s.txt", PRECIOUS), "{mark}: `--cached` lost it");
        } else {
            // An `assume-unchanged` edit is one the status question already reads, in every forced form.
            let plain: Observed = run_wrapped(&quiet_fixture, quiet_linked.as_path(), &["rm", "-f", "s.txt"]);
            assert!(refused_by_the_guard(&plain), "{mark}: a forced removal was let through: {}", stderr_of(&plain));
            assert!(still_where_it_was(quiet_linked.as_path(), "s.txt", PRECIOUS), "{mark}: the refusal lost it");
        }
        remove(&quiet_fixture);
    }
}

//endregion
//endregion
