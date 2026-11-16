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
use crate::shadow_create::{ShadowTemplate, create_shadow_template, may_be_handed_a_tree};
use crate::shadow_pool::{fill, members, pool_root};
use crate::shadow_pool_member::refusal;
use crate::test_support::{fixture, git, git_context, remove, repository};
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
