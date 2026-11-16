//! Controls for the service's stock of drawn shadows:
//!  a tree leaves the stock with the answer that takes it,
//!  and the draw beside the landing loop fills the stock out of the pool.
//! The answer's own path, and that it never waits behind a landing,
//! is controlled where a connection is (`service_server_connection_handling_tests.rs`).

use super::*;
use crate::capture_invocation::{InvocationCapture, capture_invocation_base, capture_invocation_layout};
use crate::shadow_create::ShadowTemplate;
use crate::shadow_drawn::drawn_root_for;
use crate::shadow_pool::{fill, members, pool_root_for};
use crate::test_support::{fixture, git, git_context, repository, remove};
use std::path::{Path, PathBuf};

/// A tree of this test's own,
/// made where a drawn tree of the worktree would be.
fn a_tree(root: &Path, name: &str) -> PathBuf {
    let tree: PathBuf = root.join(name);
    std::fs::create_dir_all(tree.as_path()).expect("a tree to hand out");
    return tree;
}

/// What:
///  A tree leaves the stock with the answer that takes it,
///  and a second answer is told the stock holds none.
/// Why:
///   This is the whole of how a drawn tree reaches one commit and not two:
///  the list is taken from one answer at a time,
///  and a service that handed one tree to two commits would have two commits building in one repository.
#[test]
fn a_tree_leaves_the_stock_with_the_answer_that_takes_it() {
    let root: PathBuf = fixture("shadow-draw-stock");
    let first: PathBuf = a_tree(root.as_path(), "drawn-00000000000000000000000000000001");
    let stock: Arc<DrawnShadows> = DrawnShadows::none_yet();
    assert_eq!(stock.ready(), 0, "a service starts with nothing drawn");
    assert_eq!(stock.hand_out(), None, "and is asked for nothing it has not drawn");
    stock.note_ready(first.clone());
    assert_eq!(stock.ready(), 1);
    assert_eq!(stock.hand_out(), Some(first.clone()), "the tree is handed out");
    assert_eq!(stock.hand_out(), None, "and a second commit is told there is none");
    assert_eq!(stock.ready(), 0);
    remove(root.as_path());
}

/// What:
///  The draw beside the landing loop moves a pool member into the stock,
///  and the pool gives that member up.
/// Why:
///  This is the rename that used to be a commit's:
///  it happens before any commit asks,
///  and it takes a member out of the pool exactly once.
#[test]
fn a_draw_fills_the_stock_from_the_pool() {
    let root: PathBuf = fixture("shadow-draw-fills");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    let context: GitContext = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let template: ShadowTemplate = ShadowTemplate::of(&context, &capture.layout).expect("template");
    let member: PathBuf = fill(&context, &template).expect("a fill").expect("a member");
    let common_dir: PathBuf = template.common_dir.clone();
    let git_dir: PathBuf = template.git_dir.clone();
    let stock: Arc<DrawnShadows> = DrawnShadows::none_yet();
    draw_one(&context, common_dir.as_path(), git_dir.as_path(), repo.as_path(), stock.as_ref()).expect("a draw");
    assert_eq!(stock.ready(), 1, "the member became the stock's tree");
    assert!(members(pool_root_for(common_dir.as_path(), git_dir.as_path()).as_path()).is_empty(), "and the pool gave it up");
    let handed: PathBuf = stock.hand_out().expect("the tree it drew");
    assert!(handed.starts_with(drawn_root_for(common_dir.as_path(), git_dir.as_path())), "the tree is where a drawn tree lives");
    // The member is the one that moved: the pool no longer holds it, and the tree the stock held is not a new one.
    assert!(!member.exists(), "the member is no longer where the fill published it");
    remove(root.as_path());
}

/// What:
///  A second draw with the stock full moves nothing,
///  and the pool keeps the member a first draw left.
/// Why:
///  A stock of one is what one commit at a time needs,
///  and a draw that took a second member would spend the pool a commit may need when the stock is already held.
#[test]
fn a_stock_that_is_already_held_leaves_the_pool_alone() {
    let root: PathBuf = fixture("shadow-draw-full");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    let context: GitContext = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let template: ShadowTemplate = ShadowTemplate::of(&context, &capture.layout).expect("template");
    let first: PathBuf = fill(&context, &template).expect("a fill").expect("a member");
    let second: PathBuf = fill(&context, &template).expect("a second fill").expect("a second member");
    let common_dir: PathBuf = template.common_dir.clone();
    let git_dir: PathBuf = template.git_dir.clone();
    let stock: Arc<DrawnShadows> = DrawnShadows::none_yet();
    draw_one(&context, common_dir.as_path(), git_dir.as_path(), repo.as_path(), stock.as_ref()).expect("a draw");
    let held: Option<PathBuf> = stock.hand_out();
    assert!(held.is_some(), "the draw filled the stock");
    assert_eq!(members(pool_root_for(common_dir.as_path(), git_dir.as_path()).as_path()).len(), 1, "one member was drawn and one stays");
    assert!([first.as_path(), second.as_path()].iter().any(|path| { return path.exists(); }), "the member the draw left is a pool member");
    remove(root.as_path());
}
