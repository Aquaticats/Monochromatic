//! Controls for what a command does when it meets a managed clone's slot:
//! the move refusal,
//! the prune pass and its refusal,
//! and the repair report,
//! each read from one command line.

use super::the_unread_place::unplaced_refusal_of;
use super::*;
use crate::forwarded_command::Route;
use crate::test_support::{
    fixture,
    git,
    remove,
};
use crate::testing::fixture_zfs_binding_record::{
    RegisteredSlot,
    a_registered_slot,
    common_directory_in,
    publish_under,
    record_naming,
};
use crate::worktree_clone_binding_record::binding_words::Phase;
use std::ffi::OsString;

/// One real registration of one slot,
/// which Git writes.
fn slot(name: &str) -> RegisteredSlot {
    return a_registered_slot(name);
}

/// One command line as the wrapper resolved it.
fn command_of(values: &[&str]) -> ResolvedCommand {
    let arguments: Vec<OsString> = values
        .iter()
        .map(|value| return OsString::from(value))
        .collect();
    return ResolvedCommand {
        arguments,
        subcommand_index: 0,
        subcommand: values.first().map(|value| return OsString::from(value)),
        route: Route::Direct,
        shell_command: None,
    };
}

/// One command line as the wrapper resolved it,
///  with the global options Git reads before the command word.
fn command_after_globals(
    globals: &[&str],
    values: &[&str],
) -> ResolvedCommand {
    let mut arguments: Vec<OsString> = globals
        .iter()
        .map(|value| return OsString::from(*value))
        .collect();
    arguments.extend(values.iter().map(|value| return OsString::from(*value)));
    return ResolvedCommand {
        arguments,
        subcommand_index: globals.len(),
        subcommand: values.first().map(|value| return OsString::from(*value)),
        route: Route::Direct,
        shell_command: None,
    };
}

/// What:
///  A slot command the wrapper read and could not place is refused,
///  and the three words that read a slot are the ones refused:
///  `remove` and `move` always,
///  and `prune` only when its own words name a place of their own.
/// Why:
///  Git runs a `!` alias's shell at the top of the working tree
///  (`git.c:388` to `git.c:407`),
///  so a removal or a move reads its operand from a directory this wrapper must ask Git for;
///  a command nothing placed may act on a bound slot in a repository nothing here read
///  (the alpha.50 review's L2).
///  A prune reads no operand and no directory,
///  so the caller's own repository is the one its records belong to unless the command
///  names another with `-C`,
/// `--git-dir` or `--work-tree`.
#[test]
fn a_slot_command_the_wrapper_could_not_place_is_refused() {
    for (globals, values) in [
        (&[][..], &["worktree", "remove", "-f", "-f", "one"][..]),
        (&[][..], &["worktree", "move", "one", "../moved"][..]),
        (&["-C", "elsewhere"][..], &["worktree", "prune"][..]),
        (
            &["--git-dir", "elsewhere/.git"][..],
            &["worktree", "prune"][..],
        ),
    ] {
        let ending: Action = unplaced_refusal_of(&command_after_globals(globals, values), "")
            .unwrap_or_else(|| panic!("{values:?} reads a slot, so it is refused unplaced"));
        match ending {
            Action::Exit {
                code,
                stderr: refusal,
                ..
            } => {
                assert_eq!(code, REFUSAL_STATUS);
                assert!(
                    refusal.contains("could not be asked where that alias's shell runs"),
                    "{refusal}"
                );
                assert!(
                    refusal.contains(format!("`git worktree {}` is refused", values[1]).as_str()),
                    "{refusal}"
                );
            }
            other => panic!("a refusal ends this file's commands: {other:?}"),
        }
    }
    // A prune whose own words name no place reads the caller's own records, which the planning read,
    //  and a word that reads no slot is not this refusal's.
    assert!(
        unplaced_refusal_of(&command_after_globals(&[], &["worktree", "prune"]), "").is_none(),
        "a prune that names no place of its own is left to the pass over the caller's records"
    );
    for values in [
        &["worktree", "list"][..],
        &["status"][..],
        &["worktree", "repair", "one"][..],
    ] {
        assert!(
            unplaced_refusal_of(&command_of(values), "").is_none(),
            "{values:?} reads no slot"
        );
    }
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
    let bound: RegisteredSlot = slot("command-move");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.common.join("worktrees").join("one").as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    let ending: Option<Action> = move_refusal_of(
        &command_of(&["worktree", "move", "one", "../moved"]),
        Some(bound.common.as_path()),
        Some(std::path::Path::new("/")),
        "",
    );
    match ending {
        Some(Action::Exit {
            code,
            stderr: refusal,
            ..
        }) => {
            assert_eq!(code, REFUSAL_STATUS);
            assert!(
                refusal.contains("`git worktree move` is refused"),
                "{refusal}"
            );
            assert!(refusal.contains("the slot `one` is fixed"), "{refusal}");
        }
        other => panic!("a bound slot refuses: {other:?}"),
    }
    assert!(bound.slot.is_dir(), "nothing was moved");
    // A slot no record names is Git's, and the wrapper says nothing about it.
    let other: RegisteredSlot = slot("command-move-unbound");
    assert!(
        move_refusal_of(
            &command_of(&["worktree", "move", "one", "../moved"]),
            Some(other.common.as_path()),
            Some(std::path::Path::new("/")),
            ""
        )
        .is_none()
    );
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
    let bound: RegisteredSlot = slot("command-prune");
    let registration: PathBuf = bound.common.join("worktrees").join("one");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    std::fs::remove_file(bound.slot.join(".git")).expect("pointer removed");
    let mut stderr: String = String::new();
    assert!(
        before_a_prune(
            &command_of(&["worktree", "prune"]),
            Some(bound.common.as_path()),
            &mut stderr
        )
        .is_none()
    );
    assert!(stderr.contains("restored the pointer"), "{stderr}");
    assert_eq!(
        std::fs::read(bound.slot.join(".git")).expect("pointer reads"),
        bound.pointer
    );
    // A link planted at the pointer is never written through, so the prune stops before Git drops the registration.
    std::fs::remove_file(bound.slot.join(".git")).expect("pointer removed again");
    std::os::unix::fs::symlink(bound.root.join("elsewhere"), bound.slot.join(".git"))
        .expect("planted link");
    let mut refusal: String = String::new();
    match before_a_prune(
        &command_of(&["worktree", "prune", "--verbose"]),
        Some(bound.common.as_path()),
        &mut refusal,
    ) {
        Some(Action::Exit {
            code,
            stderr: refused_prune,
            ..
        }) => {
            assert_eq!(code, REFUSAL_STATUS);
            assert!(
                refused_prune.contains("nothing was pruned"),
                "{refused_prune}"
            );
            assert!(
                refused_prune.contains("link: cannot be proved"),
                "{refused_prune}"
            );
        }
        other => panic!("a slot nothing restores refuses: {other:?}"),
    }
    std::fs::remove_file(bound.slot.join(".git")).expect("the planted link goes");
    // A dry run removes nothing, so the pass writes nothing.
    let mut dry: String = String::new();
    assert!(
        before_a_prune(
            &command_of(&["worktree", "prune", "-n"]),
            Some(bound.common.as_path()),
            &mut dry
        )
        .is_none()
    );
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
    let bound: RegisteredSlot = slot("command-repair");
    let registration: PathBuf = bound.common.join("worktrees").join("one");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    std::fs::remove_file(bound.slot.join(".git")).expect("pointer removed");
    let mut stderr: String = String::new();
    before_a_repair(
        &command_of(&["worktree", "repair", "one"]),
        Some(bound.common.as_path()),
        &mut stderr,
        Some(std::path::Path::new("/")),
    );
    assert!(
        stderr.contains("link: damaged, and the registration stands"),
        "{stderr}"
    );
    assert!(
        stderr.contains("next safe command: `git worktree repair"),
        "{stderr}"
    );
    let mut quiet: String = String::new();
    before_a_repair(
        &command_of(&["worktree", "list"]),
        Some(bound.common.as_path()),
        &mut quiet,
        Some(std::path::Path::new("/")),
    );
    assert!(quiet.is_empty(), "{quiet}");
    let mut absent: String = String::new();
    before_a_repair(
        &command_of(&["worktree", "repair", "absent"]),
        Some(bound.common.as_path()),
        &mut absent,
        Some(std::path::Path::new("/")),
    );
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
    assert!(
        move_refusal_of(
            &command_of(&["worktree", "move", "one", "two"]),
            Some(common.as_path()),
            Some(std::path::Path::new("/")),
            ""
        )
        .is_none()
    );
    assert!(
        before_a_prune(
            &command_of(&["worktree", "prune"]),
            Some(common.as_path()),
            &mut stderr
        )
        .is_none()
    );
    before_a_repair(
        &command_of(&["worktree", "repair"]),
        Some(common.as_path()),
        &mut stderr,
        Some(std::path::Path::new("/")),
    );
    assert!(stderr.is_empty(), "{stderr}");
    remove(root.as_path());
}

/// What:
///  A plain registered worktree that shares its last name with a bound slot
///  gets no report about that slot,
///  and the move of one is not the other slot's to refuse.
/// Why:
///  The record is keyed by the slot's last name,
///  so a reader that stopped at the key would print the other slot's dataset and
///  a repair command for a worktree nothing here endangers.
#[test]
fn a_plain_same_leaf_worktree_gets_no_managed_slot_report() {
    let bound: RegisteredSlot = slot("command-same-leaf");
    let registration: PathBuf = bound.common.join("worktrees").join("one");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    let other: PathBuf = bound.root.join("elsewhere").join("one");
    std::fs::create_dir_all(bound.root.join("elsewhere")).expect("elsewhere");
    git(
        bound.root.join("repository").as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            other.to_str().expect("utf-8"),
            "HEAD",
        ],
    );
    let mut stderr: String = String::new();
    before_a_repair(
        &command_of(&["worktree", "repair", other.to_str().expect("utf-8")]),
        Some(bound.common.as_path()),
        &mut stderr,
        Some(std::path::Path::new("/")),
    );
    assert!(
        stderr.is_empty(),
        "a plain worktree's repair printed the other slot's report: {stderr}"
    );
    remove(bound.root.as_path());
}

/// What:
///  A move of a plain registered worktree that shares a bound slot's last name
///  is left to Git,
///  and the bound slot's own move is still refused.
/// Why:
///  The record is keyed by the slot's last name,
///  so the slot path the record names is what says which of the two worktrees
///  it belongs to;
///  Git's own list names the other worktree at its own path,
///  so refusing that move protects a slot nothing about it endangers
///  (`doc/decision/cli-git-alpha-46-release-pass-four-answers.md`,
///  "Moving a plain worktree that shares a bound slot's last name").
#[test]
fn a_plain_same_leaf_worktree_moves_and_the_slot_stays_refused() {
    let bound: RegisteredSlot = slot("command-same-leaf-move");
    let registration: PathBuf = bound.common.join("worktrees").join("one");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    let other: PathBuf = bound.root.join("elsewhere").join("one");
    std::fs::create_dir_all(bound.root.join("elsewhere")).expect("elsewhere");
    git(
        bound.root.join("repository").as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            other.to_str().expect("utf-8"),
            "HEAD",
        ],
    );
    let plain: Option<Action> = move_refusal_of(
        &command_of(&[
            "worktree",
            "move",
            other.to_str().expect("utf-8"),
            "../moved",
        ]),
        Some(bound.common.as_path()),
        Some(std::path::Path::new("/")),
        "",
    );
    assert!(
        plain.is_none(),
        "a plain worktree with the bound slot's last name was refused: {plain:?}"
    );
    let the_slot: Option<Action> = move_refusal_of(
        &command_of(&[
            "worktree",
            "move",
            bound.slot.to_str().expect("utf-8"),
            "../moved",
        ]),
        Some(bound.common.as_path()),
        Some(std::path::Path::new("/")),
        "",
    );
    assert!(
        the_slot.is_some(),
        "the bound slot's own move must stay refused"
    );
    remove(bound.root.as_path());
}

/// The slot's pointer gone,
/// as an account that deleted it leaves it,
/// whatever an earlier arm of a control wrote.
fn leave_the_pointer_gone(pointer: &Path) {
    if pointer.exists() {
        std::fs::remove_file(pointer).expect("the pointer goes");
    }
    assert!(
        !pointer.exists(),
        "the control starts with no pointer at the slot"
    );
}

/// What:
///  Every spelling Git's own parser reads as a dry run writes nothing,
///  and the spelling that turns the dry run back off writes the pointer.
/// Why:
///  A dry run must not write,
/// and Git accepts abbreviations of `--dry-run` and
///  `--no-dry-run` (its `parse-options.c` matches any unique prefix),
/// so a pass
///  that read only the exact words wrote the pointer on a command Git pruned
///  nothing for.
#[test]
fn a_prune_git_reads_as_a_dry_run_writes_nothing() {
    let bound: RegisteredSlot = slot("command-prune-dry-spellings");
    let registration: PathBuf = bound.common.join("worktrees").join("one");
    let pointer: PathBuf = bound.slot.join(".git");
    for words in [
        &["worktree", "prune", "-n"][..],
        &["worktree", "prune", "-nv"][..],
        &["worktree", "prune", "-vn"][..],
        &["worktree", "prune", "--dry-run"][..],
        &["worktree", "prune", "--dry"][..],
        &["worktree", "prune", "--d"][..],
        &["worktree", "prune", "--dry-ru"][..],
    ] {
        publish_under(
            bound.common.as_path(),
            &record_naming(
                Phase::Filled,
                bound.slot.as_path(),
                registration.as_path(),
                Some(bound.pointer.clone()),
            ),
            "one",
        );
        leave_the_pointer_gone(pointer.as_path());
        let mut stderr: String = String::new();
        assert!(
            before_a_prune(
                &command_of(words),
                Some(bound.common.as_path()),
                &mut stderr
            )
            .is_none(),
            "{words:?}"
        );
        assert!(
            !pointer.exists(),
            "a dry run wrote the pointer back for {words:?}: {stderr}"
        );
        assert!(stderr.is_empty(), "{words:?}: {stderr}");
    }
    // The last dry-run word wins, and Git reads `--no-dry-run` and its abbreviations as a real prune.
    for words in [
        &["worktree", "prune", "--dry-run", "--no-dry-run"][..],
        &["worktree", "prune", "-n", "--no-dry"][..],
        &["worktree", "prune", "-n", "--no-d"][..],
    ] {
        publish_under(
            bound.common.as_path(),
            &record_naming(
                Phase::Filled,
                bound.slot.as_path(),
                registration.as_path(),
                Some(bound.pointer.clone()),
            ),
            "one",
        );
        leave_the_pointer_gone(pointer.as_path());
        let mut stderr: String = String::new();
        assert!(
            before_a_prune(
                &command_of(words),
                Some(bound.common.as_path()),
                &mut stderr
            )
            .is_none(),
            "{words:?}"
        );
        assert_eq!(
            std::fs::read(pointer.as_path()).expect("the pointer the pass wrote back"),
            bound.pointer,
            "a real prune has its pointer put back for {words:?}: {stderr}"
        );
    }
    remove(bound.root.as_path());
}
