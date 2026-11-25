//! Controls for what a command does when it meets a managed clone's slot:
//! the move refusal,
//! the prune pass and its refusal,
//! and the repair report,
//! each read from one command line.

use super::*;
use crate::forwarded_command::Route;
use crate::testing::fixture_zfs_binding_record::{common_directory_in, publish_under, record_naming};
use crate::test_support::{fixture, git, remove};
use crate::worktree_clone_binding_record::binding_words::Phase;
use std::ffi::OsString;

/// One fixture slot with a real registration and the record an add writes.
struct Slot {
    /// The fixture root.
    root: PathBuf,
    /// The Git common directory.
    common: PathBuf,
    /// The slot's path.
    slot: PathBuf,
    /// The bytes Git wrote into the slot.
    pointer: Vec<u8>,
}

/// One real registration of one slot.
fn slot(name: &str) -> Slot {
    let root: PathBuf = fixture(name);
    let repository: PathBuf = root.join("repository");
    std::fs::create_dir(repository.as_path()).expect("repository");
    git(repository.as_path(), &["init", "--quiet"]);
    git(repository.as_path(), &["commit", "--quiet", "--allow-empty", "--message", "base"]);
    let slot_path: PathBuf = root.join("one");
    git(repository.as_path(), &["worktree", "add", "--quiet", "--no-checkout", slot_path.to_str().expect("utf-8"), "HEAD"]);
    let common: PathBuf = repository.join(".git");
    let pointer: Vec<u8> = std::fs::read(slot_path.join(".git")).expect("pointer reads");
    return Slot { root, common, slot: slot_path, pointer };
}

/// One command line as the wrapper resolved it.
fn command_of(values: &[&str]) -> ResolvedCommand {
    let arguments: Vec<OsString> = values.iter().map(|value| return OsString::from(value)).collect();
    return ResolvedCommand {
        arguments,
        subcommand_index: 0,
        subcommand: values.first().map(|value| return OsString::from(value)),
        route: Route::Direct,
        shell_command: None,
    };
}

/// What:
///  A move of a slot a record binds is refused with the wrapper's refusal
///  status,
///  the slot stays where it is,
///  and a move of a slot no record names is left to Git.
/// Why:
///  This is the wiring plan step 12 asks for:
///  the kernel protects only a mounted clone,
///  so an unmounted slot's move is the wrapper's to refuse.
#[test]
fn a_move_of_a_bound_slot_is_refused_at_the_command() {
    let bound: Slot = slot("command-move");
    publish_under(bound.common.as_path(), &record_naming(Phase::Filled, bound.slot.as_path(), bound.common.join("worktrees").join("one").as_path(), Some(bound.pointer.clone())), "one");
    let ending: Option<Action> = move_refusal_of(&command_of(&["worktree", "move", "one", "../moved"]), Some(bound.common.as_path()));
    match ending {
        Some(Action::Exit { code, stderr: refusal, .. }) => {
            assert_eq!(code, REFUSAL_STATUS);
            assert!(refusal.contains("`git worktree move` is refused"), "{refusal}");
            assert!(refusal.contains("the slot `one` is fixed"), "{refusal}");
        }
        other => panic!("a bound slot refuses: {other:?}"),
    }
    assert!(bound.slot.is_dir(), "nothing was moved");
    // A slot no record names is Git's, and the wrapper says nothing about it.
    let other: Slot = slot("command-move-unbound");
    assert!(move_refusal_of(&command_of(&["worktree", "move", "one", "../moved"]), Some(other.common.as_path())).is_none());
    remove(bound.root.as_path());
    remove(other.root.as_path());
}

/// What:
///  The pass before a prune restores a deleted pointer,
///  prints the line,
///  and leaves the prune to Git;
///  a slot nothing restores ends the command as a refusal.
/// Why:
///  An explicit prune drops the registration at once,
///  so the restore has to happen before Git starts,
///  and the refusal has to happen instead of Git when nothing can be restored.
#[test]
fn the_prune_pass_restores_or_refuses() {
    let bound: Slot = slot("command-prune");
    let registration: PathBuf = bound.common.join("worktrees").join("one");
    publish_under(bound.common.as_path(), &record_naming(Phase::Filled, bound.slot.as_path(), registration.as_path(), Some(bound.pointer.clone())), "one");
    std::fs::remove_file(bound.slot.join(".git")).expect("pointer removed");
    let mut stderr: String = String::new();
    assert!(before_a_prune(&command_of(&["worktree", "prune"]), Some(bound.common.as_path()), &mut stderr).is_none());
    assert!(stderr.contains("restored the pointer"), "{stderr}");
    assert_eq!(std::fs::read(bound.slot.join(".git")).expect("pointer reads"), bound.pointer);
    // A link planted at the pointer is never written through, so the prune stops before Git drops the registration.
    std::fs::remove_file(bound.slot.join(".git")).expect("pointer removed again");
    std::os::unix::fs::symlink(bound.root.join("elsewhere"), bound.slot.join(".git")).expect("planted link");
    let mut refusal: String = String::new();
    match before_a_prune(&command_of(&["worktree", "prune", "--verbose"]), Some(bound.common.as_path()), &mut refusal) {
        Some(Action::Exit { code, stderr: refused_prune, .. }) => {
            assert_eq!(code, REFUSAL_STATUS);
            assert!(refused_prune.contains("nothing was pruned"), "{refused_prune}");
            assert!(refused_prune.contains("link: cannot be proved"), "{refused_prune}");
        }
        other => panic!("a slot nothing restores refuses: {other:?}"),
    }
    std::fs::remove_file(bound.slot.join(".git")).expect("the planted link goes");
    // A dry run removes nothing, so the pass writes nothing.
    let mut dry: String = String::new();
    assert!(before_a_prune(&command_of(&["worktree", "prune", "-n"]), Some(bound.common.as_path()), &mut dry).is_none());
    assert!(dry.is_empty(), "{dry}");
    assert!(!bound.slot.join(".git").exists(), "a dry run wrote nothing");
    remove(bound.root.as_path());
}

/// What:
///  A repair command prints the report of what it can and cannot restore
///  before Git runs,
///  and no other command prints it.
/// Why:
///  Git 2.56.1 rebuilds a missing pointer and exits 1,
///  and a registration already pruned is not rebuilt at all,
///  so a person reads the difference before they act.
#[test]
fn a_repair_prints_the_report_before_git() {
    let bound: Slot = slot("command-repair");
    let registration: PathBuf = bound.common.join("worktrees").join("one");
    publish_under(bound.common.as_path(), &record_naming(Phase::Filled, bound.slot.as_path(), registration.as_path(), Some(bound.pointer.clone())), "one");
    std::fs::remove_file(bound.slot.join(".git")).expect("pointer removed");
    let mut stderr: String = String::new();
    before_a_repair(&command_of(&["worktree", "repair", "one"]), Some(bound.common.as_path()), &mut stderr);
    assert!(stderr.contains("link: damaged, and the registration stands"), "{stderr}");
    assert!(stderr.contains("next safe command: `git worktree repair"), "{stderr}");
    let mut quiet: String = String::new();
    before_a_repair(&command_of(&["worktree", "list"]), Some(bound.common.as_path()), &mut quiet);
    assert!(quiet.is_empty(), "{quiet}");
    let mut absent: String = String::new();
    before_a_repair(&command_of(&["worktree", "repair", "absent"]), Some(bound.common.as_path()), &mut absent);
    assert!(absent.is_empty(), "{absent}");
    remove(bound.root.as_path());
}

/// What:
///  A common directory with no record home answers nothing for every command.
/// Why:
///  Every repository's forwarded commands run this glue,
///  and the ordinary case must print nothing and change nothing.
#[test]
fn a_repository_without_records_gets_nothing() {
    let root: PathBuf = fixture("command-no-records");
    let common: PathBuf = common_directory_in(root.as_path());
    let mut stderr: String = String::new();
    assert!(move_refusal_of(&command_of(&["worktree", "move", "one", "two"]), Some(common.as_path())).is_none());
    assert!(before_a_prune(&command_of(&["worktree", "prune"]), Some(common.as_path()), &mut stderr).is_none());
    before_a_repair(&command_of(&["worktree", "repair"]), Some(common.as_path()), &mut stderr);
    assert!(stderr.is_empty(), "{stderr}");
    remove(root.as_path());
}
