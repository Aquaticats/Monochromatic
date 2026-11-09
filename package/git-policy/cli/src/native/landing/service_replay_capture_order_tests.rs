//! Controls for the capture order of a replay's shared paths:
//!  the decisions the stamps of the landed commits support,
//! and the work one walk over the history costs for all of them.

use super::*;
/// One shared path with the entries a decision does not read.
use crate::replay_shared_paths::{SharedPath, TreeEntry};
/// The service's record of one landed capture.
use crate::service_protocol::LandedCapture;
/// The context the queries of a site run through.
use crate::transaction_git::GitContext;
/// Fixtures,
/// the real Git the fixtures run,
/// and the context the queries of a site use.
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// A capture of a commit that was never landed,
/// or of one whose stamp the case names.
fn landed(commit: &str, captured: u64) -> LandedCapture {
    return LandedCapture { commit: commit.to_string(), captured };
}

/// An entry of no consequence to a capture-order decision.
fn entry() -> Option<TreeEntry> {
    return Some(TreeEntry { mode: String::from("100644"), oid: String::from("0000000000000000000000000000000000000000") });
}

/// A shared path the decision is asked about.
fn shared(path: &[u8]) -> SharedPath {
    return SharedPath { path: path.to_vec(), base: entry(), landed: entry(), prepared: entry() };
}

/// What:
///  A replay whose history since the base changes the given paths in commits of its own.
/// Why:
///   The capture order is a question about the commits that changed a path,
///  so the fixture puts one commit per path after the base,
///  and the case says which of them the service landed and when.
struct History {
    /// The fixture directory removed at the end.
    root: PathBuf,
    /// The worktree.
    repo: PathBuf,
    /// The commit every range starts after.
    base: String,
    /// The commits after the base,
    /// in the order they were made.
    commits: Vec<String>,
}

/// Build a repository whose base holds `paths`,
/// then one commit per path that appends a line to it.
fn history(name: &str, paths: usize) -> History {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    for number in 0..paths {
        std::fs::write(repo.join(format!("path-{number:04}.txt")), format!("base {number}\n")).expect("base file");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let mut commits: Vec<String> = Vec::new();
    for number in 0..paths {
        let file: String = format!("path-{number:04}.txt");
        std::fs::write(repo.join(file.as_str()), format!("base {number}\nlanded {number}\n")).expect("landed file");
        git(repo.as_path(), &["commit", "--quiet", "--message=landed", file.as_str()]);
        commits.push(git_text(repo.as_path(), &["rev-parse", "HEAD"]));
    }
    return History { root, repo, base, commits };
}

/// The context a history's queries run through.
fn context_of() -> GitContext {
    return git_context();
}

/// The captured change of a history:
/// its base,
/// a tree nothing reads,
/// this capture's stamp,
/// and the landed commits the case names.
fn captured_of(history: &History, captured: u64, landed: Vec<LandedCapture>) -> Captured {
    return captured_of_kind(history.base.as_str(), captured, landed);
}

/// The same capture of a base the case names,
/// for a history whose base is not the fixture's first commit.
fn captured_of_kind(base: &str, captured: u64, landed: Vec<LandedCapture>) -> Captured {
    return Captured { base: base.to_string(), tree: String::from("0".repeat(40)), captured, landed, conclusion: None };
}

/// The path of the `number`th file of a history.
fn path_of(number: usize) -> Vec<u8> {
    return format!("path-{number:04}.txt").into_bytes();
}

/// What:
///  The decision follows the stamps of the commits that changed each path:
///  all earlier than this capture keeps the captured bytes,
///  all later keeps the landed ones,
///  a commit this service never landed is not decided by order,
///  and a path the history never changed is not either.
/// Why:
///  These four answers are what the replay acts on:
///  `Prepared` lands the captured entry,
///  `Landed` keeps the branch's,
///  and `Unordered` hands the path to subsumption and the merge.
///  The walk must answer each path from the commits that changed that path alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await captureOrderOfEach(site, captured, tip, [early, late, unknown, untouched]))
///   .toEqual(new Map([[early, 'prepared'], [late, 'landed'], [unknown, 'unordered'], [untouched, 'unordered']]));
/// ```
#[test]
fn the_stamps_of_the_commits_that_changed_a_path_decide_it() {
    let history: History = history("capture-order-decisions", 3);
    std::fs::create_dir_all(history.root.join("scratch")).expect("scratch");
    let context: GitContext = context_of();
    let common: PathBuf = history.repo.join(".git");
    let objects: PathBuf = common.join("objects");
    let scratch: PathBuf = history.root.join("scratch");
    let site: MergeSite<'_> = MergeSite {
        context: &context,
        cwd: history.repo.as_path(),
        common_dir: common.as_path(),
        object_directory: objects.as_path(),
        scratch: scratch.as_path(),
    };
    let tip: String = git_text(history.repo.as_path(), &["rev-parse", "HEAD"]);
    let early: &str = history.commits[0].as_str();
    let late: &str = history.commits[1].as_str();
    let captured: Captured = captured_of(&history, 100, vec![landed(early, 10), landed(late, 200)]);
    let paths: Vec<SharedPath> = vec![shared(path_of(0).as_slice()), shared(path_of(1).as_slice()), shared(path_of(2).as_slice()), shared(b"never-changed.txt")];
    let order: BTreeMap<Vec<u8>, CaptureOrderDecision> =
        capture_order_of_each(&site, &captured, tip.as_str(), paths.as_slice()).expect("the capture order");
    assert_eq!(order.get(path_of(0).as_slice()), Some(&CaptureOrderDecision::Prepared), "a commit captured before this one keeps the captured bytes");
    assert_eq!(order.get(path_of(1).as_slice()), Some(&CaptureOrderDecision::Landed), "a commit captured after this one keeps the landed bytes");
    assert_eq!(order.get(path_of(2).as_slice()), Some(&CaptureOrderDecision::Unordered), "a commit the service never landed is not decided by order");
    assert_eq!(order.get(b"never-changed.txt".as_slice()), Some(&CaptureOrderDecision::Unordered), "a path the history never changed is not decided by order");
    // A capture with no stamp of its own is not decided by order at all.
    let unsampled: Captured = captured_of(&history, 0, vec![landed(early, 10)]);
    let undecided: BTreeMap<Vec<u8>, CaptureOrderDecision> =
        capture_order_of_each(&site, &unsampled, tip.as_str(), paths.as_slice()).expect("the capture order");
    assert!(undecided.values().all(|decision| return *decision == CaptureOrderDecision::Unordered), "{undecided:?}");
    remove(history.root.as_path());
}

/// What:
///  One walk over the history answers every shared path,
///  so the landed commits and the changed paths are each read once.
/// Why:
///  `AGENTS.md` `RG2`:
/// both the shared paths and the history since the base grow with the repository,
///  and asking Git for one path at a time walks the whole history once per path,
///  while every commit of each of those walks is then looked up in the landed list by a walk of its
///  own.
///  The seam counts the collected elements a question looks at
///  (`foundation/collected_elements_looked_at_tests.rs` proves the count separates a set from a walk):
///  one pass over the landed commits,
///  one question per change the walk meets,
///  and one per path decide the whole set,
///  while the per-path shape counts every landed commit for every commit of every path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(lookedAt(() => captureOrderOfEach(site, captured, tip, paths))).toBeLessThanOrEqual(4 * width);
/// ```
#[test]
fn one_walk_decides_every_shared_path() {
    let width: usize = 32;
    let history: History = history("capture-order-one-walk", width);
    std::fs::create_dir_all(history.root.join("scratch")).expect("scratch");
    let context: GitContext = context_of();
    let common: PathBuf = history.repo.join(".git");
    let objects: PathBuf = common.join("objects");
    let scratch: PathBuf = history.root.join("scratch");
    let site: MergeSite<'_> = MergeSite {
        context: &context,
        cwd: history.repo.as_path(),
        common_dir: common.as_path(),
        object_directory: objects.as_path(),
        scratch: scratch.as_path(),
    };
    let tip: String = git_text(history.repo.as_path(), &["rev-parse", "HEAD"]);
    let landed: Vec<LandedCapture> = history.commits.iter().map(|commit| return landed(commit, 10)).collect();
    let captured: Captured = captured_of(&history, 100, landed);
    let paths: Vec<SharedPath> = (0..width).map(|number| return shared(path_of(number).as_slice())).collect();
    let before: u64 = crate::collected_elements_looked_at::looked_at();
    let order: BTreeMap<Vec<u8>, CaptureOrderDecision> =
        capture_order_of_each(&site, &captured, tip.as_str(), paths.as_slice()).expect("the capture order");
    let counted: u64 = crate::collected_elements_looked_at::looked_at() - before;
    assert_eq!(order.len(), width, "every shared path is decided");
    assert!(order.values().all(|decision| return *decision == CaptureOrderDecision::Prepared), "{order:?}");
    assert!(
        counted <= 4 * width as u64,
        "{counted} collected elements looked at for {width} shared paths; one walk counts the landed commits once, one \
question per change and one per path, while a walk of the history per path counts every landed commit for every commit \
of every path ({} looks here)",
        width as u64 * width as u64
    );
    remove(history.root.as_path());
}

/// What:
///  A merge that changed a path is one of the commits that changed it,
///  and a merge the service never landed holds the path back.
/// Why:
///  `git log -- <path>` follows history simplification,
///  so a merge whose bytes for the path are the second parent's is not a commit it reports;
///  the one walk reports a merge against each parent,
///  so the change is seen,
///  and a merge this service never landed makes the order unknown.
///  The unknown answer hands the path to subsumption and the merge,
///  which is the direction that cannot discard a landed edit;
///  a later capture that holds every change of the path still lands its bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(orderFor('p.txt', [sideAt10, mergeAt20])).toBe('prepared');
/// expect(orderFor('p.txt', [sideAt10])).toBe('unordered'); // the merge is unknown
/// ```
#[test]
fn a_merge_that_changed_a_path_counts_as_a_commit_that_changed_it() {
    let root: PathBuf = fixture("capture-order-merge");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("p.txt"), "base\n").expect("p");
    std::fs::write(repo.join("q.txt"), "base\n").expect("q");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    // The branch changes the other file, so the merge of `side` is clean and holds `side`'s bytes for p.txt.
    git(repo.as_path(), &["switch", "--quiet", "--create", "side"]);
    std::fs::write(repo.join("p.txt"), "side\n").expect("side p");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=side"]);
    let side_commit: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("q.txt"), "main\n").expect("main q");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    let main_commit: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["merge", "--quiet", "--no-ff", "--message=merge", "side"]);
    let merge_commit: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    // The merge holds p.txt as `side` left it and q.txt as `main` left it, which is what history simplification hides.
    assert_eq!(git_text(repo.as_path(), &["show", format!("{merge_commit}:p.txt").as_str()]), "side", "the merge kept the side's bytes");
    std::fs::write(repo.join("after.txt"), "after\n").expect("after");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=after"]);
    let tip: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::create_dir_all(root.join("scratch")).expect("scratch");
    let context: GitContext = context_of();
    let common: PathBuf = repo.join(".git");
    let objects: PathBuf = common.join("objects");
    let scratch: PathBuf = root.join("scratch");
    let site: MergeSite<'_> = MergeSite {
        context: &context,
        cwd: repo.as_path(),
        common_dir: common.as_path(),
        object_directory: objects.as_path(),
        scratch: scratch.as_path(),
    };
    let p: Vec<SharedPath> = vec![shared(b"p.txt")];
    // Every commit that changed the path, the merge included, was captured before this one.
    let captured: Captured = captured_of_kind(base.as_str(), 100, vec![landed(side_commit.as_str(), 10), landed(merge_commit.as_str(), 20)]);
    let order: BTreeMap<Vec<u8>, CaptureOrderDecision> =
        capture_order_of_each(&site, &captured, tip.as_str(), p.as_slice()).expect("the capture order");
    assert_eq!(order.get(b"p.txt".as_slice()), Some(&CaptureOrderDecision::Prepared), "the merge is one of the commits, and its stamp is earlier");
    // The merge is a commit this service never landed, so the order of the path is unknown.
    let unknown_merge: Captured = captured_of_kind(base.as_str(), 100, vec![landed(side_commit.as_str(), 10), landed(main_commit.as_str(), 10)]);
    let held_back: BTreeMap<Vec<u8>, CaptureOrderDecision> =
        capture_order_of_each(&site, &unknown_merge, tip.as_str(), p.as_slice()).expect("the capture order");
    assert_eq!(held_back.get(b"p.txt".as_slice()), Some(&CaptureOrderDecision::Unordered), "a merge the service never landed leaves the order unknown");
    remove(root.as_path());
}
