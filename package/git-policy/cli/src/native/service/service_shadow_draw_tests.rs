//! Controls for the service's stock of drawn shadows:
//!  a tree leaves the stock with the answer that takes it,
//!  a stocked tree is kept fresh so no sweep takes it,
//!  a stock that counts a tree that is gone draws another,
//!  and the answer reads memory alone, never the volume.
//! The answer's own path, and that it never waits behind a landing,
//! is controlled where a connection is (`service_server_connection_handling_tests.rs`).

use super::*;
use crate::capture_invocation::{InvocationCapture, capture_invocation_base, capture_invocation_layout};
use crate::ref_name::RefName;
use crate::shadow_create::ShadowTemplate;
use crate::shadow_drawn::{drawn_members, drawn_root_for};
use crate::shadow_pool::{fill, members, pool_root_for};
use crate::shadow_pool_member::refusal;
use crate::test_support::{fixture, git, git_context, repository, remove, wait_until};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// A tree of this test's own,
/// made where a drawn tree of the worktree would be.
fn a_tree(root: &Path, name: &str) -> PathBuf {
    let tree: PathBuf = root.join(name);
    std::fs::create_dir_all(tree.as_path()).expect("a tree to hand out");
    return tree;
}

/// Set an entry's modification time to `back` before now.
fn age_tree(path: &Path, back: Duration) {
    std::fs::File::open(path).expect("open").set_modified(SystemTime::now() - back).expect("set modification time");
}

/// Whether a tree is younger than the age a sweep takes one at.
fn younger_than_the_sweep_takes_it(path: &Path) -> bool {
    let Ok(metadata) = std::fs::symlink_metadata(path) else { return false };
    let Ok(modified) = metadata.modified() else { return false };
    let Ok(age) = SystemTime::now().duration_since(modified) else { return false };
    return age < crate::service_scratch_abandonment::SCRATCH_TREE_ABANDONED_AFTER;
}

/// A request naming the worktree a template was made from,
/// which is what the landing beside a draw names.
fn a_request_naming(template: &ShadowTemplate) -> crate::service_protocol::CommitRequest {
    return crate::service_protocol::CommitRequest {
        id: String::from("a-request"),
        repository: template.common_dir.parent().unwrap_or(Path::new("/nowhere")).to_path_buf(),
        common_dir: template.common_dir.clone(),
        git_dir: template.git_dir.clone(),
        reference: RefName::from_text("refs/heads/main"),
        base: String::new(),
        changes: Vec::new(),
        message: String::from("a commit\n"),
        author: None,
        committer: String::from("Fixture <fixture@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: template.git_dir.join("index"),
        scratch: template.git_dir.join("monochromatic-git-policy-cli-service-a-0"),
        shadow: None,
        captured: 0,
        holder: String::new(),
    };
}

/// What:
///  A tree that stands in the stock while the repository is idle is marked as used while it waits,
///  so no sweep takes it and the commit it was drawn for is handed a tree that is there.
/// Why:
///  A drawn tree is standing state until a commit takes it, as a pool member is until one claims it;
///  the stock is memory only, so its entry has to keep the tree fresh in the only language the sweep speaks.
///  Without that, a repository idle longer than the bound leaves a tree the next sweep removes,
///  and the stock still counts it: the landing draws nothing in its place and the commit is answered a path that is gone.
#[test]
fn a_stocked_tree_is_kept_fresh_so_no_sweep_takes_it() {
    let root: PathBuf = fixture("shadow-draw-stocked-fresh");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    let context: GitContext = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let template: ShadowTemplate = ShadowTemplate::of(&context, &capture.layout).expect("template");
    let _member: PathBuf = fill(&context, &template).expect("a fill").expect("a member");
    let drawn: PathBuf = crate::shadow_drawn::draw(&context, &template).expect("a draw").expect("a drawn tree");
    let stock: Arc<DrawnShadows> = DrawnShadows::none_yet();
    stock.note_ready(drawn.clone());
    // The repository stood idle longer than a sweep takes a tree at, with the tree waiting in the stock.
    age_tree(drawn.as_path(), crate::service_scratch_abandonment::SCRATCH_TREE_ABANDONED_AFTER + Duration::from_secs(60));
    assert!(wait_until(&|| return younger_than_the_sweep_takes_it(drawn.as_path())), "a stocked tree is marked as used while it waits");
    let swept: Vec<PathBuf> = crate::shadow_drawn::sweep_drawn(template.common_dir.as_path(), SystemTime::now());
    assert!(swept.is_empty(), "a stocked tree must not be removed by the sweep: {swept:?}");
    let handed: PathBuf = stock.hand_out().expect("the stock answers with the tree it kept");
    assert_eq!(handed, drawn, "and the tree it answers with is the one that was kept");
    assert!(handed.exists(), "which is still there for the commit");
    assert_eq!(refusal(&template, handed.as_path()).expect("a check"), None, "and is still what a claim accepts");
    remove(root.as_path());
}

/// What:
///  A step of the wall clock forward does not sweep a tree the stock holds,
///  and the answer still names a tree that is there.
/// Why:
///  The stock's entry is kept fresh by its renewal, and that keeps a sweep at a steady clock away;
///  a clock that steps forward makes the same tree look hours old,
///  and the service's holding is the one thing a step cannot take (`tree_held_by_a_live_process.rs`).
///  A stock whose tree went would answer the next commit with a path that is gone.
#[test]
fn a_clock_step_forward_never_sweeps_a_tree_the_stock_holds() {
    let root: PathBuf = fixture("shadow-draw-stock-clock-step");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    let context: GitContext = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let template: ShadowTemplate = ShadowTemplate::of(&context, &capture.layout).expect("template");
    let _member: PathBuf = fill(&context, &template).expect("a fill").expect("a member");
    let drawn: PathBuf = crate::shadow_drawn::draw(&context, &template).expect("a draw").expect("a drawn tree");
    let stock: Arc<DrawnShadows> = DrawnShadows::none_yet();
    stock.note_ready(drawn.clone());
    // The host clock steps forward three hours, well past the bound and its renewal margin, and a sweep reads it.
    let stepped: SystemTime = SystemTime::now() + Duration::from_secs(3 * 60 * 60);
    let swept: Vec<PathBuf> = crate::shadow_drawn::sweep_drawn(template.common_dir.as_path(), stepped);
    assert!(swept.is_empty(), "a tree the stock holds must not be swept however far the clock stepped: {swept:?}");
    let handed: PathBuf = stock.hand_out().expect("the stock answers with the tree it kept");
    assert!(handed.exists(), "and the tree it names is there for the commit");
    remove(root.as_path());
}

/// What:
///  A stock that counts a tree the drawn folder no longer holds draws another one,
///  so the commit the landing drew for is not answered a path that is gone.
/// Why:
///  The stock is memory only and the drawn folder is on the volume, where another program can take a tree;
///  a landing that counted a gone tree would leave the stock as it is and answer the next commit with nothing there.
#[test]
fn a_stock_that_counts_a_tree_that_is_gone_draws_another() {
    let root: PathBuf = fixture("shadow-draw-stock-gone");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    let context: GitContext = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let template: ShadowTemplate = ShadowTemplate::of(&context, &capture.layout).expect("template");
    // The draw this control waits for moves a pool member, so the fill makes one.
    let _member: PathBuf = fill(&context, &template).expect("a fill").expect("a member");
    let folder: PathBuf = drawn_root_for(template.common_dir.as_path(), template.git_dir.as_path());
    let stock: Arc<DrawnShadows> = DrawnShadows::none_yet();
    // A tree the service counted and the drawn folder no longer holds, as another program's removal leaves it.
    let gone: PathBuf = a_tree(folder.as_path(), "drawn-00000000000000000000000000000007");
    stock.note_ready(gone.clone());
    std::fs::remove_dir_all(gone.as_path()).expect("the counted tree goes");
    // The landing that follows asks for the next commit's tree.
    start(&context, &a_request_naming(&template), &stock);
    assert!(
        wait_until(&|| return !drawn_members(folder.as_path()).is_empty()),
        "a stock that counts a tree that is gone must draw another one"
    );
    remove(root.as_path());
}

/// What:
///  A commit's question is answered from memory alone,
///  even when the tree the stock holds answers no open at all.
/// Why:
///  The answer is written on the connection's own task,
///  and a call on the volume there is the wait the draw-ahead exists to take off a commit
///  (`service_server_connection_handling.rs`,
///  `answer_the_shadow_question`).
///  A named pipe is the one entry whose plain open waits for a writer that may never come,
///  so an answer that touched the tree would never be written.
#[test]
fn the_answer_reads_memory_and_never_touches_the_tree() {
    let root: PathBuf = fixture("shadow-draw-answer-memory");
    let tree: PathBuf = root.join("drawn-00000000000000000000000000000003");
    {
        // The bytes of a pathname, for the C interface.
        use std::os::unix::ffi::OsStrExt as _;
        let named: std::ffi::CString = std::ffi::CString::new(tree.as_os_str().as_bytes()).expect("a fixture path holds no NUL byte");
        // SAFETY: the string is NUL-terminated and outlives the call, and the name is a fresh path in the fixture.
        let made: libc::c_int = unsafe { libc::mkfifo(named.as_ptr(), 0o600) };
        assert_eq!(made, 0, "make a named pipe at {}: {}", tree.display(), std::io::Error::last_os_error());
    }
    let stock: Arc<DrawnShadows> = DrawnShadows::none_yet();
    stock.note_ready(tree.clone());
    // The answer is taken on a thread of its own, so an answer that never comes fails this control instead of holding it.
    let (sent, heard): (std::sync::mpsc::Sender<Option<PathBuf>>, std::sync::mpsc::Receiver<Option<PathBuf>>) = std::sync::mpsc::channel();
    let asked: Arc<DrawnShadows> = Arc::clone(&stock);
    let _answering = std::thread::spawn(move || {
        let _sent: std::result::Result<(), std::sync::mpsc::SendError<Option<PathBuf>>> = sent.send(asked.hand_out());
    });
    let answered: Option<PathBuf> = heard.recv_timeout(Duration::from_secs(30)).expect("the answer to a commit's question must not wait on the volume");
    assert_eq!(answered, Some(tree.clone()), "the stock answers with the tree it holds, from memory");
    // Release the renewal that waited in its own open, so the tree of this control outlives nothing.
    let _released = std::fs::OpenOptions::new().write(true).open(tree.as_path());
    remove(root.as_path());
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
