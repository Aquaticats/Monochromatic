//! Controls for narrowing an upload plan to the objects a push introduces,
//! against repositories built by real Git.

use super::*;
use crate::repo_read_layout::read_layout_in;
use crate::test_support::{fixture, git, git_output, isolated_overlay, remove};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The object name that the `n`-th fixture pointer names.
fn oid(n: u8) -> String {
    return format!("{n:02x}").repeat(32);
}

/// Commit `files` (path,
/// text) and return the commit's name.
fn commit(repo: &Path, files: &[(&str, String)], message: &str) -> String {
    for (path, text) in files {
        let target: PathBuf = repo.join(path);
        std::fs::create_dir_all(target.parent().expect("a parent")).expect("directories");
        std::fs::write(target, text).expect("file");
    }
    git(repo, &["add", "--all"]);
    git(repo, &["commit", "--quiet", "--allow-empty", &format!("--message={message}")]);
    return String::from_utf8(git_output(repo, &["rev-parse", "HEAD"]).stdout).expect("UTF-8").trim().to_owned();
}

/// The pointer file text for the `n`-th object.
fn pointer(n: u8) -> String {
    return format!("version https://git-lfs.github.com/spec/v1\noid sha256:{}\nsize 10\n", oid(n));
}

/// Put the `n`-th object into the local LFS storage,
/// as `git lfs` would have when it cleaned the file.
fn store(repo: &Path, n: u8) {
    let name: String = oid(n);
    let directory: PathBuf = repo.join(".git/lfs/objects").join(&name[..2]).join(&name[2..4]);
    std::fs::create_dir_all(&directory).expect("storage");
    std::fs::write(directory.join(&name), b"0123456789").expect("object");
}

/// A repository whose `origin/main` tracking ref is at `known` and whose `main` is at `tip`:
///  `known` holds pointer 1,
/// `tip` adds a plain file only,
/// `with_pointer` (on a branch `feature`) adds pointer 2.
fn world(root: &Path) -> (PathBuf, String, String, String) {
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    store(repo.as_path(), 1);
    let known: String = commit(repo.as_path(), &[("assets/one.bin", pointer(1))], "known");
    git(repo.as_path(), &["update-ref", "refs/remotes/origin/main", known.as_str()]);
    let tip: String = commit(repo.as_path(), &[("docs/readme.txt", String::from("text\n"))], "plain");
    git(repo.as_path(), &["checkout", "--quiet", "-b", "feature"]);
    let with_pointer: String = commit(repo.as_path(), &[("assets/two.bin", pointer(2))], "pointer");
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    return (repo, known, tip, with_pointer);
}

/// A plan for `origin` with these refs.
fn plan(refs: &[&str], full_scan: bool, everything: bool) -> UploadPlan {
    return UploadPlan {
        remotes: vec![OsString::from("origin")],
        refs: refs.iter().map(OsString::from).collect(),
        everything,
        submodules: false,
        full_scan,
        objects: None,
    };
}

/// Narrow with the layout found from the repository's files.
fn narrow(repo: &Path, plan: UploadPlan) -> Option<UploadPlan> {
    // The hermetic overlay removes the system file, which the readers would otherwise have to learn from the real Git first.
    let layout = read_layout_in(repo, isolated_overlay().as_slice());
    return narrow_plan(plan, layout.as_ref());
}

/// A push that adds no pointer starts nothing:
/// the plan narrows to no upload at all.
#[test]
fn a_push_that_adds_no_pointer_uploads_nothing() {
    let root = fixture("lfs-object-plan-none");
    let (repo, _known, _tip, _with_pointer) = world(root.as_path());
    assert_eq!(narrow(repo.as_path(), plan(&["main"], false, false)), None);
    assert_eq!(narrow(repo.as_path(), plan(&["refs/heads/main"], false, false)), None, "a full ref name reads the same");
    assert_eq!(narrow(repo.as_path(), plan(&["HEAD"], false, false)), None, "and so does HEAD");
    remove(root.as_path());
}

/// A push that adds one pointer names exactly that object and nothing the tracking ref already reaches.
#[test]
fn a_push_that_adds_one_pointer_names_that_object() {
    let root = fixture("lfs-object-plan-one");
    let (repo, _known, _tip, _with_pointer) = world(root.as_path());
    store(repo.as_path(), 2);
    let narrowed: UploadPlan = narrow(repo.as_path(), plan(&["feature"], false, false)).expect("something to upload");
    assert_eq!(narrowed.objects, Some(vec![oid(2)]));
    let runs = narrowed.runs(std::ffi::OsStr::new("origin"));
    let expected: Vec<OsString> = ["push", "--object-id", "origin", oid(2).as_str()].iter().map(OsString::from).collect();
    assert_eq!(runs, vec![expected]);
    remove(root.as_path());
}

/// When the tracking refs cannot be trusted the whole history of the pushed refs is read,
/// so the older pointer is named too.
#[test]
fn an_untrusted_destination_scans_the_whole_history() {
    let root = fixture("lfs-object-plan-full");
    let (repo, _known, _tip, _with_pointer) = world(root.as_path());
    store(repo.as_path(), 2);
    let narrowed: UploadPlan = narrow(repo.as_path(), plan(&["feature"], true, false)).expect("something to upload");
    assert_eq!(narrowed.objects, Some(vec![oid(1), oid(2)]));
    let mirrored: UploadPlan = narrow(repo.as_path(), plan(&[], true, true)).expect("something to upload");
    assert_eq!(mirrored.objects, Some(vec![oid(1), oid(2)]), "a mirror push reads every ref");
    remove(root.as_path());
}

/// A pointer whose object is not in the local storage leaves the plan as it was,
/// so `git-lfs` words the failure.
#[test]
fn a_pointer_without_its_local_object_is_left_to_git_lfs() {
    let root = fixture("lfs-object-plan-missing");
    let (repo, _known, _tip, _with_pointer) = world(root.as_path());
    let original: UploadPlan = plan(&["feature"], false, false);
    assert_eq!(narrow(repo.as_path(), original.clone()), Some(original));
    remove(root.as_path());
}

/// Anything the readers decline leaves the plan as it was:
/// no layout,
/// a ref that does not resolve,
/// a revision expression.
#[test]
fn what_the_readers_decline_leaves_the_plan_to_git_lfs() {
    let root = fixture("lfs-object-plan-decline");
    let (repo, _known, _tip, _with_pointer) = world(root.as_path());
    for refs in [vec!["nosuchref"], vec!["main~1"], vec!["main^{tree}"], vec!["feature", "nosuchref"]] {
        let original: UploadPlan = plan(&refs, false, false);
        assert_eq!(narrow(repo.as_path(), original.clone()), Some(original), "{refs:?}");
    }
    let original: UploadPlan = plan(&["feature"], false, false);
    assert_eq!(narrow_plan(original.clone(), None), Some(original), "no layout");
    remove(root.as_path());
}
