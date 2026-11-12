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

use super::support::{Fixture, Observed, fixture, git, porcelain, remove, repository, run_direct, run_wrapped, stderr_of};
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
    for (index, arguments, written_over) in [
        (1, vec!["checkout", "other", "--", "a.txt"], true),
        (2, vec!["restore", "--source", "other", "--", "a.txt"], true),
        (3, vec!["reset", "--hard"], false),
        (4, vec!["checkout", "--force"], false),
        (5, vec!["read-tree", "--reset", "-u", "HEAD"], false),
        (6, vec!["checkout", "--ignore-skip-worktree-bits", "--", "a.txt"], false),
    ] {
        let label: String = format!("discard-skip-worktree-{index}");
        let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = marked(label.as_str(), Some("skip-worktree"));
        let kept_before: bool = still_where_it_was(linked.as_path(), "a.txt", EDIT);
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        let kept: bool = still_where_it_was(linked.as_path(), "a.txt", EDIT)
            || still_where_it_was(linked.as_path(), "b.txt", EDIT);
        if written_over {
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
            // Git keeps the file for this form, so the guard must not refuse it.
            let (direct_fixture, _direct_repo, direct_linked): (Fixture, PathBuf, PathBuf) =
                marked(format!("{label}-direct").as_str(), Some("skip-worktree"));
            let direct: Observed = run_direct(&direct_fixture, direct_linked.as_path(), arguments.as_slice());
            let git_kept: bool = still_where_it_was(direct_linked.as_path(), "a.txt", EDIT);
            assert!(!refused_by_the_guard(&observed), "{arguments:?} was refused where real Git keeps the file (exit {:?}): {}", direct.code, stderr_of(&observed));
            assert!(git_kept, "the control needs real Git to keep the file for {arguments:?}");
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
        (3, vec!["checkout-index", "--force", "--prefix=.", "-a"], "kept.txt"),
    ] {
        let label: String = format!("discard-destination-{index}");
        let (fixture, _repo, linked): (Fixture, PathBuf, PathBuf) = tracked_pair(label.as_str());
        std::fs::write(linked.join("kept.txt"), PRECIOUS).expect("the person's file at the destination");
        let before: Vec<String> = worktree_state(linked.as_path());
        let observed: Observed = run_wrapped(&fixture, linked.as_path(), arguments.as_slice());
        assert!(refused_by_the_guard(&observed), "{arguments:?} was not refused: {}", stderr_of(&observed));
        assert!(stderr_of(&observed).contains("kept.txt"), "{arguments:?}: {}", stderr_of(&observed));
        assert!(still_where_it_was(linked.as_path(), person_path, PRECIOUS), "{arguments:?} lost the file");
        assert_eq!(worktree_state(linked.as_path()), before, "{arguments:?} wrote something before refusing");
        let (hatch_fixture, _hatch_repo, hatch_linked): (Fixture, PathBuf, PathBuf) =
            tracked_pair(format!("{label}-hatch").as_str());
        std::fs::write(hatch_linked.join("kept.txt"), PRECIOUS).expect("the person's file at the destination");
        let passed: Observed = run_hatched(&hatch_fixture, hatch_linked.as_path(), arguments.as_slice());
        let (direct_fixture, _direct_repo, direct_linked): (Fixture, PathBuf, PathBuf) =
            tracked_pair(format!("{label}-direct").as_str());
        std::fs::write(direct_linked.join("kept.txt"), PRECIOUS).expect("the person's file at the destination");
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
