//! Controls for the shadows the service draws out of the pool ahead of a commit:
//!  a draw hands out what a creation makes and is taken out of the pool,
//!  an empty pool answers nothing,
//!  a member that is not what a creation would make is refused and the next one is drawn,
//!  a tree nothing renewed is swept while a held one is left,
//!  a removal of a worktree leaves the tree a commit holds and takes the one nobody renewed,
//!  and a held tree is renewed until it is released.

use super::*;
use crate::capture_invocation::{InvocationCapture, capture_invocation_base, capture_invocation_layout};
use crate::commit_build_part_durations::BuildPartClock;
use crate::service_shadow_draw::DrawnShadows;
use crate::shadow_create::{ShadowTemplate, create_shadow_template, may_be_handed_a_tree};
use crate::shadow_pool::{fill, members, pool_root};
use crate::shadow_pool_member::refusal;
use crate::test_support::{fixture, git, git_context, remove, repository, wait_until};
/// The stand-in service of one control,
/// and the protocol lines it answers.
use crate::service_protocol::{ServiceMessage, encode_service};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Captures the invocation of a repository's working directory,
/// as the transaction does.
fn capture_of(repo: &Path) -> InvocationCapture {
    let context: GitContext = git_context();
    let layout = capture_invocation_layout(&context, repo, false).expect("layout");
    return capture_invocation_base(&context, repo, layout).expect("capture");
}

/// The facts one invocation's shadow is made from.
fn template_of(repo: &Path) -> ShadowTemplate {
    return ShadowTemplate::of(&git_context(), &capture_of(repo).layout).expect("template");
}

/// A repository with one commit on `main`.
fn one_commit_repository(root: &Path, name: &str) -> PathBuf {
    let repo: PathBuf = repository(root, name);
    std::fs::write(repo.join("a.txt"), "a\n").expect("a");
    git(repo.as_path(), &["add", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=a"]);
    return repo;
}

/// Everything a shadow holds,
/// by name,
/// with what a link points at,
/// sorted so two shadows compare entry for entry.
fn shape_of(directory: &Path) -> Vec<String> {
    let mut shape: Vec<String> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![PathBuf::from("")];
    while let Some(relative) = pending.pop() {
        let reading: PathBuf = directory.join(&relative);
        let Ok(entries) = std::fs::read_dir(reading.as_path()) else { continue };
        for entry in entries.flatten() {
            let name: String = entry.file_name().to_string_lossy().into_owned();
            let child: PathBuf = relative.join(&name);
            let metadata = std::fs::symlink_metadata(entry.path()).expect("metadata");
            if metadata.file_type().is_symlink() {
                let target = std::fs::read_link(entry.path()).expect("target");
                shape.push(format!("link {name}: {}", target.display()));
                continue;
            }
            if metadata.is_dir() {
                shape.push(format!("dir  {name}"));
                pending.push(child);
                continue;
            }
            let bytes = std::fs::read(entry.path()).expect("bytes");
            shape.push(format!("file {name}: {} bytes {:x}", bytes.len(), bytes.len()));
        }
    }
    shape.sort();
    return shape;
}

/// One member the fill made for a repository.
fn a_member(repo: &Path) -> (ShadowTemplate, PathBuf) {
    let template: ShadowTemplate = template_of(repo);
    let member: Option<PathBuf> = fill(&git_context(), &template).expect("fill");
    return (template, member.expect("a member"));
}

/// Set an entry's modification time to `back` before now.
fn age(path: &Path, back: Duration) {
    std::fs::File::open(path).expect("open").set_modified(SystemTime::now() - back).expect("set modification time");
}

/// What:
///  A draw hands out a tree that is what a creation makes,
///  takes it out of the pool,
///  and answers the claim's own question with nothing to refuse.
/// Why:
///  This is the whole of the option:
///  the rename a commit waited for has happened before the commit asked,
///  and the tree is still the tree every check a claim makes accepts.
#[test]
fn a_draw_hands_out_what_a_creation_makes_and_takes_it_out_of_the_pool() {
    let root: PathBuf = fixture("shadow-drawn-hand-out");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, member) = a_member(repo.as_path());
    let drawn: PathBuf = draw(&git_context(), &template).expect("a draw").expect("a drawn tree");
    assert_eq!(drawn_members(drawn_root(&template).as_path()), vec![drawn.clone()], "the drawn folder holds it");
    assert!(members(pool_root(&template).as_path()).is_empty(), "the pool gave it up to the draw");
    assert_eq!(refusal(&template, drawn.as_path()).expect("a check"), None, "the drawn tree is what a claim accepts");
    assert_ne!(drawn.as_path(), member.as_path(), "the tree is at a name of its own");
    let created: PathBuf = root.join("created");
    let mut clock: BuildPartClock = BuildPartClock::begin("test");
    create_shadow_template(&git_context(), &template, created.as_path(), &mut clock).expect("creation");
    assert_eq!(shape_of(drawn.as_path()), shape_of(created.as_path()), "a drawn tree is a creation's tree");
    remove(root.as_path());
}

/// What:
///  A pool with no member gives the draw nothing to hand out,
///  and the drawn folder stays empty.
#[test]
fn an_empty_pool_gives_the_draw_nothing() {
    let root: PathBuf = fixture("shadow-drawn-empty");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let template: ShadowTemplate = template_of(repo.as_path());
    assert_eq!(draw(&git_context(), &template).expect("a draw"), None, "nothing to draw");
    assert!(drawn_members(drawn_root(&template).as_path()).is_empty(), "and nothing is drawn");
    remove(root.as_path());
}

/// What:
///  A member that is not the tree a creation would make is refused,
///  removed,
///  and the draw answers nothing rather than handing over a tree this invocation cannot use.
/// Why:
///  A drawn tree is checked before it is handed out,
///  so a commit that asks is never handed a tree it would have to refuse,
///  and the pool is repaired on the service's time instead of a commit's.
#[test]
fn a_member_a_creation_would_not_make_is_refused_and_removed() {
    let root: PathBuf = fixture("shadow-drawn-refused");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, member) = a_member(repo.as_path());
    std::fs::write(member.join("stray").as_path(), b"left by a use no claim can trust").expect("a stray entry");
    assert_eq!(draw(&git_context(), &template).expect("a draw"), None, "the only member is refused");
    assert!(members(pool_root(&template).as_path()).is_empty(), "the refused member is removed");
    assert!(drawn_members(drawn_root(&template).as_path()).is_empty(), "and nothing is drawn");
    remove(root.as_path());
}

/// What:
///  A refused member beside a good one leaves the draw a tree the claim's question accepts.
/// Why:
///  A draw examines the pool's members in the order the folder lists them,
///  so it removes what it refuses and goes on to the next,
///  exactly as a claim does.
#[test]
fn a_refused_member_beside_a_good_one_leaves_a_tree_the_claim_accepts() {
    let root: PathBuf = fixture("shadow-drawn-beside");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, first) = a_member(repo.as_path());
    let second: PathBuf = fill(&git_context(), &template).expect("a second fill").expect("a second member");
    std::fs::write(first.join("stray").as_path(), b"left by a use no claim can trust").expect("a stray entry");
    let drawn: PathBuf = draw(&git_context(), &template).expect("a draw").expect("a drawn tree");
    assert_eq!(refusal(&template, drawn.as_path()).expect("a check"), None, "the drawn tree is what a claim accepts");
    assert!(
        [first.as_path(), second.as_path()].contains(&drawn.as_path()) || drawn.starts_with(drawn_root(&template)),
        "the drawn tree came out of the pool"
    );
    remove(root.as_path());
}

/// What:
///  A tree nothing renewed is swept,
///  and a tree a command still renews is left.
/// Why:
///  This is the whole of how a tree in use is told from an abandoned one:
///  a command that stopped renews nothing,
///  and the age it goes at is the age of the scratch tree it would have held instead.
#[test]
fn a_tree_nothing_renewed_is_swept_and_a_fresh_one_is_left() {
    let root: PathBuf = fixture("shadow-drawn-sweep");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, _member) = a_member(repo.as_path());
    let held: PathBuf = draw(&git_context(), &template).expect("a draw").expect("a drawn tree");
    std::fs::create_dir(drawn_root(&template).join("drawn-abandoned").as_path()).expect("a tree nothing renews");
    let abandoned: PathBuf = drawn_root(&template).join("drawn-abandoned");
    age(abandoned.as_path(), nothing_holds_after() + Duration::from_secs(60));
    let removed: Vec<PathBuf> = sweep_drawn(template.common_dir.as_path(), SystemTime::now());
    assert_eq!(removed, vec![abandoned.clone()], "the tree nothing renewed goes");
    assert!(!abandoned.exists(), "and it is not there");
    assert!(held.exists(), "the tree a command is holding stays");
    // The last tree of a worktree takes its folder with it, so a worktree that is gone leaves nothing behind.
    age(held.as_path(), nothing_holds_after() + Duration::from_secs(60));
    let removed_again: Vec<PathBuf> = sweep_drawn(template.common_dir.as_path(), SystemTime::now());
    assert_eq!(removed_again, vec![held], "the held tree goes once nothing renews it");
    assert!(std::fs::symlink_metadata(drawn_root(&template).as_path()).is_err(), "and the worktree's folder with it");
    remove(root.as_path());
}

/// What:
///  A tree the service hands a commit is not swept for the age it carried out of the pool,
///  and the commit that was handed it builds in it and lands there.
/// Why:
///  A pool member stands in the pool until a commit takes one,
///  so a repository nobody committed in for longer than the sweep's bound holds a member already past it,
///  and the draw's one `rename` keeps that member's modification time.
///  The renewal a commit holds its tree with beats for the first time `SCRATCH_REFRESH_EVERY` after it starts,
///  and every wrapped command of the repository sweeps every worktree's drawn folder by that age
///  (`wrapped_command_notices_snapshots_and_manual_push.rs`,
///  `sweep_shadow_pools_of_gone_worktrees`),
///  so a sweep that runs between the hand-out and the commit's first write removes the tree the commit was told to build in.
///  The stock is asked here as the service asks it,
///  so the moment a control needs, the answer and then a sweep of the repository, is fixed and never raced.
#[test]
fn a_tree_handed_to_a_commit_is_not_swept_for_the_age_it_carried_out_of_the_pool() {
    let root: PathBuf = fixture("shadow-drawn-handed-then-swept");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let template: ShadowTemplate = ShadowTemplate::of(&context, &capture.layout).expect("template");
    let member: PathBuf = fill(&context, &template).expect("a fill").expect("a member");
    // The machine stood idle long enough that the member the pool holds is already past the bound.
    age(member.as_path(), nothing_holds_after() + Duration::from_secs(60));
    assert!(nothing_holds(member.as_path(), SystemTime::now()), "the member the pool holds is already past the bound");
    let drawn: PathBuf = draw(&context, &template).expect("a draw").expect("a drawn tree");
    let stock: std::sync::Arc<DrawnShadows> = DrawnShadows::none_yet();
    stock.note_ready(drawn.clone());
    let handed: PathBuf = stock.hand_out().expect("the service answered with the tree it drew");
    // A swept command of the repository, or another worktree's landing, in that moment: every worktree's drawn folder goes
    //  by its age alone.
    let swept: Vec<PathBuf> = sweep_drawn(template.common_dir.as_path(), SystemTime::now());
    assert!(swept.is_empty(), "a tree handed to a commit must not be removed by the sweep: {swept:?}");
    assert!(!nothing_holds(handed.as_path(), SystemTime::now()), "the answer marked the tree as used now, so the bound counts from it");
    let socket: PathBuf = a_stand_in_a_drawn_tree("handed-then-swept", handed.as_path());
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(scratch.as_path()).expect("the commit's scratch tree");
    let shadow: PathBuf = scratch.join("shadow");
    let mut clock: BuildPartClock = BuildPartClock::begin("test");
    let taken: crate::shadow_create::ShadowTree = crate::shadow_create::take_or_create_shadow(
        &context,
        repo.as_path(),
        &capture,
        shadow.as_path(),
        scratch.as_path(),
        false,
        &mut clock,
        &crate::service_server_socket_binding::Bind::Path(socket),
    )
    .expect("the commit's shadow, built where the service said");
    assert_eq!(taken.path, handed, "the commit built in the tree it was handed");
    assert!(taken.handed.is_some(), "and the commit holds it, so its renewal keeps it");
    assert!(handed.join("packed-refs").exists(), "the commit's own ref store is written in that tree");
    assert!(handed.join("HEAD").exists(), "and its HEAD names the branch it commits to");
    remove(root.as_path());
}

/// What:
///  A tree a command holds is marked as used from the moment it holds it,
///  not an hour later.
/// Why:
///  A tree arrives with somebody else's last mark on it:
///  a drawn tree carries the moment it was drawn and then the stock's renewals,
///  and a tree can reach a holder close to the age a sweep takes one at.
///  The first renewal is therefore made at once,
///  so a sweep that runs in the hour after a holder takes a tree cannot find it old
///  (`service_scratch_renewal.rs`).
#[test]
fn a_held_tree_is_marked_as_used_from_the_moment_it_is_held() {
    let root: PathBuf = fixture("shadow-drawn-held-marked");
    let tree: PathBuf = root.join("drawn-held-old");
    std::fs::create_dir(tree.as_path()).expect("a tree to hold");
    age(tree.as_path(), nothing_holds_after() + Duration::from_secs(60));
    let _held: HandedShadow = HandedShadow::hold(tree.clone());
    assert!(
        wait_until(&|| return !nothing_holds(tree.as_path(), SystemTime::now())),
        "a tree is marked as used from the moment a command holds it"
    );
    remove(root.as_path());
}

/// What:
///  A shadow that cannot be written for a reason of its own is the commit's to report:
///  the old tree or the pool is not tried again in its place.
/// Why:
///  The commit falls back only when the tree it was handed went,
///  because that is a loss of the wrapper's own cache and nothing the person did;
///  a tree that is still there and cannot be written is a failure of the volume or of the repository,
///  and a commit that hid it would build a tree it cannot trust either.
#[test]
fn a_handed_tree_that_cannot_be_written_is_the_commits_to_report() {
    let root: PathBuf = fixture("shadow-drawn-cannot-write");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let template: ShadowTemplate = ShadowTemplate::of(&context, &capture.layout).expect("template");
    // The pool's only member is the tree the draw makes, so no member stands in for the one that cannot be written.
    let _member: PathBuf = fill(&context, &template).expect("a fill").expect("a member");
    let drawn: PathBuf = draw(&context, &template).expect("a draw").expect("a drawn tree");
    // A tree every read of the claim's question may take and no write: the ref store of this commit cannot be written into it.
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(drawn.as_path(), std::fs::Permissions::from_mode(0o555)).expect("a tree no write reaches");
    }
    assert_eq!(refusal(&template, drawn.as_path()).expect("a check"), None, "the tree is one the commit's question accepts");
    let socket: PathBuf = a_stand_in_a_drawn_tree("cannot-write", drawn.as_path());
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(scratch.as_path()).expect("the commit's scratch tree");
    let shadow: PathBuf = scratch.join("shadow");
    let mut clock: BuildPartClock = BuildPartClock::begin("test");
    let taken: Result<crate::shadow_create::ShadowTree, crate::transaction_git::GitFailure> = crate::shadow_create::take_or_create_shadow(
        &context,
        repo.as_path(),
        &capture,
        shadow.as_path(),
        scratch.as_path(),
        false,
        &mut clock,
        &crate::service_server_socket_binding::Bind::Path(socket),
    );
    let Err(reported) = taken else { panic!("a tree that cannot be written must be reported, not replaced") };
    assert!(reported.0.contains("packed-refs"), "the failure names the write that failed: {}", reported.0);
    assert!(std::fs::symlink_metadata(drawn.as_path()).is_ok(), "the tree that could not be written is still there for the person to look at");
    remove(root.as_path());
}

/// What:
///  A drawn tree nobody took is removed once its bound has passed,
///  and the bound that removes it counts from the draw.
/// Why:
///  The draw marks the tree it drew as used,
///  and that mark is the only thing that keeps a tree in the stock,
///  so a control that a much older member still leaves a tree the sweep collects proves the mark did not turn the bound into never.
#[test]
fn a_drawn_tree_nobody_took_is_removed_once_its_bound_has_passed() {
    let root: PathBuf = fixture("shadow-drawn-bound");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, member) = a_member(repo.as_path());
    // The member stood in the pool longer than a bound, so a draw that marked nothing would leave a tree that is already old.
    age(member.as_path(), nothing_holds_after() + Duration::from_secs(60));
    let drawn: PathBuf = draw(&git_context(), &template).expect("a draw").expect("a drawn tree");
    assert!(!nothing_holds(drawn.as_path(), SystemTime::now()), "the draw marked the tree as drawn now");
    // Nobody takes it, and the bound passes with it standing there.
    age(drawn.as_path(), nothing_holds_after() + Duration::from_secs(1));
    let removed: Vec<PathBuf> = sweep_drawn(template.common_dir.as_path(), SystemTime::now());
    assert_eq!(removed, vec![drawn.clone()], "a drawn tree nobody took goes once its bound has passed");
    assert!(std::fs::symlink_metadata(drawn.as_path()).is_err(), "and it is not there");
    remove(root.as_path());
}

/// What:
///  The removal of a worktree removes the trees of its own folder that nothing renewed,
///  and leaves the tree a commit is running in.
/// Why:
///  A removal that took a live commit's tree would take the ref store that commit had just written there,
///  and the commit would report a failure real Git did not have.
#[test]
fn a_removal_leaves_the_tree_a_commit_holds_and_takes_the_one_nobody_renewed() {
    let root: PathBuf = fixture("shadow-drawn-removal");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let (template, _member) = a_member(repo.as_path());
    let held: PathBuf = draw(&git_context(), &template).expect("a draw").expect("a drawn tree");
    let other: PathBuf = drawn_root(&template).join("drawn-by-a-command-that-stopped");
    std::fs::create_dir(other.as_path()).expect("a tree nothing renews");
    age(other.as_path(), nothing_holds_after() + Duration::from_secs(60));
    let removed: Vec<PathBuf> = sweep_drawn_of(template.common_dir.as_path(), template.git_dir.as_path(), SystemTime::now());
    assert_eq!(removed, vec![other], "only the tree nothing renewed goes");
    assert!(held.exists(), "the tree a commit is running in is still there");
    remove(root.as_path());
}

/// What:
///  A stand-in service at a socket of its own that hands out one drawn tree,
///  and the socket's path.
/// Why:
///  The hand-out is one exchange,
///  and a control of the tree a commit was handed needs a service that hands out exactly one.
///  The socket sits in a short folder because one socket address holds 107 bytes and a fixture root does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const socket = aStandInADrawnTree('refused', tree);
/// ```
fn a_stand_in_a_drawn_tree(name: &str, tree: &Path) -> PathBuf {
    let folder: PathBuf = std::env::var_os("HOME")
        .map(PathBuf::from)
        .expect("a home directory")
        .join("temp/agent")
        .join(format!("drawn{}", std::process::id()))
        .join(name);
    std::fs::create_dir_all(folder.as_path()).expect("the short folder");
    crate::test_support::track_for_this_thread(folder.parent().expect("the folder's parent").to_path_buf().as_path());
    let socket: PathBuf = folder.join("service.sock");
    let listener: UnixListener = UnixListener::bind(socket.as_path()).expect("a stand-in socket");
    let handed: PathBuf = tree.to_path_buf();
    std::thread::spawn(move || {
        let Ok((stream, _address)) = listener.accept() else { return };
        let mut writer = stream.try_clone().expect("the second half of the connection");
        let welcome: String = format!("{}\n", encode_service(&ServiceMessage::Welcome { version: crate::service_protocol::PROTOCOL_VERSION, release: Some(String::from("test")) }));
        if writer.write_all(welcome.as_bytes()).is_err() {
            return;
        }
        let mut reader: BufReader<std::os::unix::net::UnixStream> = BufReader::new(stream);
        let mut line: String = String::new();
        if reader.read_line(&mut line).is_err() {
            return;
        }
        let answer: String = encode_service(&ServiceMessage::Drawn { tree: handed });
        let _sent = writer.write_all(format!("{answer}\n").as_bytes());
    });
    return socket;
}

/// What:
///  A tree a commit holds is renewed while the value lives,
///  so the sweep finds it fresh however long the commit runs.
/// Why:
///  The renewal is the whole of the evidence that a command still runs,
///  so a control that watches one renewal happen proves the sweep's question is answerable
///  without a lock and without asking any process.
#[test]
fn a_held_tree_is_renewed_while_its_command_lives() {
    let root: PathBuf = fixture("shadow-drawn-renewal");
    let tree: PathBuf = root.join("drawn-renewed");
    std::fs::create_dir(tree.as_path()).expect("a tree to hold");
    let _held: HandedShadow = HandedShadow::hold_renewed_every(tree.clone(), Duration::from_millis(10));
    age(tree.as_path(), Duration::from_secs(60));
    let aged: SystemTime = std::fs::symlink_metadata(tree.as_path()).expect("metadata").modified().expect("a time");
    std::thread::sleep(Duration::from_millis(120));
    let renewed: SystemTime = std::fs::symlink_metadata(tree.as_path()).expect("metadata").modified().expect("a time");
    assert!(renewed > aged, "a held tree is renewed while its command lives");
}

/// What:
///  A tree the service took is left for the service to remove,
///  and one the service did not take is removed beside the command.
/// Why:
///  The service removes the trees a request named once that landing is over,
///  and a removal on both sides at once would report a directory that is already gone;
///  a request the service never took leaves the tree nothing else knows about,
///  so the command that asked must remove it.
#[test]
fn a_tree_the_service_took_is_left_to_the_service_and_one_it_did_not_is_removed() {
    let root: PathBuf = fixture("shadow-drawn-release");
    let taken: PathBuf = root.join("drawn-taken");
    std::fs::create_dir(taken.as_path()).expect("a tree the service takes");
    let mut held: HandedShadow = HandedShadow::hold(taken.clone());
    held.the_service_removes_it();
    drop(held);
    let removed: PathBuf = root.join("drawn-released");
    std::fs::create_dir(removed.as_path()).expect("a tree the command releases");
    drop(HandedShadow::hold(removed.clone()));
    assert!(
        crate::test_support::wait_until(&|| return std::fs::symlink_metadata(removed.as_path()).is_err()),
        "a tree the service did not take is removed beside the command"
    );
    assert!(taken.exists(), "a tree the service took is still there for the service to remove");
    remove(root.as_path());
}

/// What:
///  A commit that concludes a merge, a cherry-pick or a revert is never handed a drawn tree,
///  and a reftable repository or a worktree that keeps its own `info` is never handed one either.
/// Why:
///   The landing of a conclusion compares the state Git left in the shadow with the copy it kept,
///  reading the shadow at the path the commit's own scratch tree holds,
///  so a tree used where it stands is not one that comparison can name yet.
///  The other two cases are shadows no draw makes
///  (`shadow_pool_member.rs`,
///  `shadow_drawn.rs`).
#[test]
fn a_conclusion_commit_or_a_shadow_no_draw_makes_is_never_handed_a_tree() {
    let root: PathBuf = fixture("shadow-drawn-gates");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let template: ShadowTemplate = template_of(repo.as_path());
    assert!(may_be_handed_a_tree(&template, false), "an ordinary commit of a files-backend repository is one the service may serve");
    assert!(!may_be_handed_a_tree(&template, true), "a commit that concludes something is not");
    let reftable: ShadowTemplate = ShadowTemplate { ref_format: crate::transaction_journal::RefFormat::Reftable, ..template_of(repo.as_path()) };
    assert!(!may_be_handed_a_tree(&reftable, false), "a reftable repository's shadow is Git's own tables");
    remove(root.as_path());
}

/// What:
///  A drawn tree this invocation's own check refuses is released and the commit takes the pool's member.
/// Why:
///   The service's check of a member is the fill's, and this commit's facts are what decide:
///  a repository that changed between the draw and the claim is refused here and never used,
///  the tree is handed back in the background,
///  and the commit goes on to the paths it knows instead of failing.
#[test]
fn a_drawn_tree_this_invocation_refuses_is_released_and_the_pool_serves_the_commit() {
    let root: PathBuf = fixture("shadow-drawn-refused-at-the-hand-out");
    let repo: PathBuf = one_commit_repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let layout = capture_invocation_layout(&context, repo.as_path(), false).expect("layout");
    let capture: InvocationCapture = capture_invocation_base(&context, repo.as_path(), layout).expect("capture");
    let template: ShadowTemplate = ShadowTemplate::of(&context, &capture.layout).expect("template");
    let member: PathBuf = fill(&context, &template).expect("a fill").expect("a member");
    let refused: PathBuf = drawn_root(&template).join("drawn-00000000000000000000000000000004");
    std::fs::create_dir_all(refused.as_path()).expect("a drawn tree");
    std::fs::write(refused.join("stray").as_path(), b"left by a use no claim can trust").expect("a stray entry");
    let socket: PathBuf = a_stand_in_a_drawn_tree("refused", refused.as_path());
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(scratch.as_path()).expect("the commit's scratch tree");
    let shadow: PathBuf = scratch.join("shadow");
    let mut clock: BuildPartClock = BuildPartClock::begin("test");
    let taken: crate::shadow_create::ShadowTree = crate::shadow_create::take_or_create_shadow(
        &context,
        repo.as_path(),
        &capture,
        shadow.as_path(),
        scratch.as_path(),
        false,
        &mut clock,
        &crate::service_server_socket_binding::Bind::Path(socket),
    )
    .expect("a shadow");
    assert_eq!(taken.path, shadow, "the pool's member is the tree the commit builds in");
    assert!(taken.handed.is_none(), "and nothing is handed back by the commit's own end");
    assert!(members(pool_root(&template).as_path()).is_empty(), "the member this commit took is the one it used");
    assert!(!member.exists(), "and it is no longer a member");
    assert!(
        crate::test_support::wait_until(&|| return std::fs::symlink_metadata(refused.as_path()).is_err()),
        "the tree this invocation refused is released beside the command"
    );
    remove(root.as_path());
}
