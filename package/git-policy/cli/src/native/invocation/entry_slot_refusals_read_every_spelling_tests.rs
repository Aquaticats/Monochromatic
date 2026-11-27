//! What:
//!  The slot refusals read one command line whatever global options stand
//!  before its words:
//!  `-c`,
//!  `--no-optional-locks`,
//!  `--git-dir`,
//!  `-C <the Git directory>` and
//!  `-C <the repository>`,
//!  and a removal or a move behind a global option this build does not read is
//!  refused in a repository that holds slot records.
//! Why:
//!  `SPEC.md` promises that every spelling Git accepts is read for the worktree
//!  a `git worktree remove` or `git worktree move` names,
//!  `-f -f` included,
//!  and a global option that changes nothing about where the operand lies must
//!  not hide it from the refusal.
//!  Every control here starts at a command line,
//!  goes through the planning the executable runs and the served run it ends in,
//!  and ends at the report a person reads,
//!  so no control can pass while the planner and the run disagree about which
//!  worktree the command named.
//!  A control selects its fixture repository with `-C`,
//!  because this crate's controls never change their own process's directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const line of spelling) expect(refused(planInvocation(line))).toContain('a whole managed clone is bound');
//! ```
#![cfg(unix)]

/// Import the decision under test and the fixtures the controls share.
use super::plan_invocation;
use crate::action::Action;
use crate::child_environment::invocation_overlay;
use crate::forwarded_command::ResolvedCommand;
use crate::real_git::ResolutionInputs;
use crate::test_support::{
    REAL_GIT,
    executable,
    isolated_overlay,
    remove,
};
use crate::testing::fixture_zfs_binding_record::{
    RegisteredSlot,
    a_registered_slot,
    publish_under,
    record_naming,
};
use crate::testing::mounted_dataset_table::{
    ScriptedMountTable,
    row,
};
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_mount_protocol::clone_mount_intent::IntentReading;
use crate::worktree_clone_slot_state::slot_state_live_read::{
    CloneIntentReads,
    SlotReadingSources,
};
use crate::worktree_command_facts::CopyFacts;
use crate::worktree_copy_forward::{
    ForwardedRun,
    TheRunsOwnSteps,
    run_forwarded_git,
    run_forwarded_git_with_its_own_steps,
};
use crate::worktree_removal_of_a_bound_clone::{
    RemovalReads,
    refuse_the_removal_with,
};
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The inputs selecting the image's real Git with a fixture file standing in
///  for this wrapper.
/// Why:
///  The planning asks the resolver for real Git,
///  and the fixture directory stands in for the executable the wrapper is,
///  so no control reads the runner's own process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inputs = { realGit: '/usr/bin/git', currentDirectory: root, ownExecutable: root + '/own-wrapper' };
/// ```
fn inputs(root: &Path) -> ResolutionInputs {
    let own: PathBuf = root.join("own-wrapper");
    executable(own.as_path(), b"\x7fELF fixture wrapper");
    return ResolutionInputs {
        platform: crate::real_git::Platform::Unix,
        path: OsString::from("/usr/bin"),
        path_extensions: OsString::new(),
        current_directory: root.to_path_buf(),
        common_paths: Vec::<PathBuf>::new(),
        own_executable: own,
        service_unit: None,
    };
}

/// Build owned arguments from text.
fn text(values: &[&str]) -> Vec<OsString> {
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    for value in values {
        result.push(OsString::from(value));
    }
    return result;
}

/// One command line started in a fixture repository:
/// `-C <repository>` and then the words,
/// which is how a control points a command at its own repository without
/// changing this process's directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = startedIn(repository, ['worktree', 'remove', '-f', '-f', 'one']);
/// ```
fn started_in(
    repository: &Path,
    rest: &[&str],
) -> Vec<OsString> {
    let mut words: Vec<OsString> =
        vec![OsString::from("-C"), repository.as_os_str().to_os_string()];
    words.extend(text(rest));
    return words;
}

/// The facts a settled forward carries,
/// or a failure naming what the planning answered instead.
fn settled_facts(action: Action) -> CopyFacts {
    return match action {
        Action::ForwardSettled {
            copy,
            ..
        } => *copy,
        other => panic!("expected a settled forward, got {other:?}"),
    };
}

/// The end of a stop action,
/// with the report a person reads.
fn stopped(action: Action) -> (i32, String) {
    return match action {
        Action::Exit {
            code,
            stderr,
            ..
        } => (code, stderr),
        other => panic!("expected the command to stop, got {other:?}"),
    };
}

/// The pool sentence a control hands in where a refusal must read no pool:
/// its text in the report would prove the pool was read.
const NO_POOL_WAS_ASKED: &str = "this control reads no pool";

/// One mount table in which nothing stands at any slot,
/// so the refusal decides the slot's state without reading this machine's
/// mounts.
fn nothing_is_mounted() -> ScriptedMountTable {
    return ScriptedMountTable::holding(&[row(1, 0, "/", "/", "zfs", "tank/ROOT")]);
}

/// One scripted mount-intent source,
/// which answers the clone's property without this machine's ZFS device.
struct ScriptedIntent;

impl CloneIntentReads for ScriptedIntent {
    fn intent_of(
        &self,
        _clone: &str,
    ) -> Result<IntentReading, String> {
        return Ok(IntentReading::Absent);
    }
}

/// Run the command the facts and the words name,
/// with the pool a control answers instead of this machine's device,
/// and answer the run it ends in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const run = runTheRemoval(facts, ['worktree', 'remove', '-f', '-f', 'one']);
/// ```
fn run_the_removal(
    facts: &CopyFacts,
    words: &[OsString],
) -> ForwardedRun {
    let mounts: ScriptedMountTable = nothing_is_mounted();
    let intents: ScriptedIntent = ScriptedIntent;
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    return run_the_removal_with(facts, words, &sources);
}

/// The same run over the slot sources a control answers,
/// so a control can script the mount table and the clone's intent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const run = runTheRemovalWith(facts, words, { mounts: aTableWithTheMount, intents: noIntent });
/// ```
fn run_the_removal_with(
    facts: &CopyFacts,
    words: &[OsString],
    sources: &SlotReadingSources<'_>,
) -> ForwardedRun {
    let refusal = |target: &Path, common: &Path| {
        return refuse_the_removal_with(
            target,
            common,
            RemovalReads::Unreadable(NO_POOL_WAS_ASKED),
            sources,
        );
    };
    return run_forwarded_git_with_its_own_steps(
        Path::new(REAL_GIT),
        words,
        &[],
        Some(facts),
        TheRunsOwnSteps {
            refuse_a_bound_slots_removal: &refusal,
            stop_units: &|_about: &crate::worktree_removal_stops_units::WorktreeAboutToGo| {},
        },
    );
}

/// The report of a refused removal,
/// or the panic that says Git was let through.
fn the_refusals_report(run: ForwardedRun) -> String {
    return match run {
        ForwardedRun::Failed {
            stderr,
            code,
            ..
        } => {
            assert_eq!(
                code,
                crate::entry_zfs_slot_commands::REFUSAL_STATUS,
                "the refusal ends with the wrapper's own status"
            );
            stderr
        }
        other => panic!("the removal of a bound slot must be refused, got {other:?}"),
    };
}

/// What:
///  A bound slot's removal is refused behind every global option Git accepts,
///  and the slot,
///  its registration and its pointer all stand afterwards.
/// Why:
///  This is the alpha.47 pass's High finding:
///  the refusal read its operand through a reading that only served `-C` and a
///  short list of neutral options,
///  so `-c`,
///  `--no-optional-locks`,
///  `--git-dir` and `-C <the Git directory>`
///  sent the removal to Git,
///  whose `-f -f` deleted the registration and the slot's underlay pointer
///  while the record stood at `phase=filled`.
#[test]
fn a_bound_slots_removal_is_refused_behind_every_global_option() {
    let bound: RegisteredSlot = a_registered_slot("entry-slot-removal-spellings");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    let common: String = bound.common.display().to_string();
    // The attached spelling of `--git-dir`, which Git reads the same way as the separated one.
    let git_directory: String = format!("--git-dir={common}");
    let spellings: Vec<Vec<OsString>> = vec![
        started_in(
            bound.repository.as_path(),
            &["worktree", "remove", "-f", "-f", "one"],
        ),
        started_in(
            bound.repository.as_path(),
            &[
                "-c",
                "advice.x=false",
                "worktree",
                "remove",
                "-f",
                "-f",
                "one",
            ],
        ),
        started_in(
            bound.repository.as_path(),
            &[
                "--no-optional-locks",
                "worktree",
                "remove",
                "-f",
                "-f",
                "one",
            ],
        ),
        // The neutral options the ladder's own reading always served, which the release pass's controls used.
        started_in(
            bound.repository.as_path(),
            &[
                "--no-pager",
                "--no-advice",
                "worktree",
                "remove",
                "-f",
                "-f",
                "one",
            ],
        ),
        started_in(
            bound.repository.as_path(),
            &[
                "-C",
                common.as_str(),
                "worktree",
                "remove",
                "-f",
                "-f",
                "one",
            ],
        ),
        started_in(
            bound.repository.as_path(),
            &[
                "--git-dir",
                common.as_str(),
                "worktree",
                "remove",
                "-f",
                "-f",
                "one",
            ],
        ),
        // A folder outside every worktree, which is where the pass's runs s3, s4 and u1 stood.
        started_in(
            bound.root.as_path(),
            &[
                "--git-dir",
                common.as_str(),
                "worktree",
                "remove",
                "-f",
                "-f",
                "one",
            ],
        ),
        started_in(
            bound.root.as_path(),
            &[
                "-C",
                common.as_str(),
                "worktree",
                "remove",
                "-f",
                "-f",
                "one",
            ],
        ),
        text(&[
            "-C",
            common.as_str(),
            "--git-dir",
            common.as_str(),
            "worktree",
            "remove",
            "-f",
            "-f",
            "one",
        ]),
        // The attached spelling of the same option, which Git reads the same way.
        text(&[
            git_directory.as_str(),
            "worktree",
            "remove",
            "-f",
            "-f",
            "one",
        ]),
    ];
    for words in &spellings {
        let facts: CopyFacts = settled_facts(plan_invocation(
            words.as_slice(),
            &[],
            &inputs(bound.root.as_path()),
        ));
        assert_eq!(
            facts.common_dir.as_deref(),
            Some(bound.common.as_path()),
            "the refusal reads this repository's records for {words:?}"
        );
        let stderr: String = the_refusals_report(run_the_removal(&facts, words.as_slice()));
        assert!(
            stderr.contains("a whole managed clone is bound to this slot"),
            "{words:?}: {stderr}"
        );
        assert!(
            stderr.contains("state: known unmounted clone"),
            "{words:?}: {stderr}"
        );
    }
    // Nothing Git would have deleted is gone.
    assert!(bound.slot.is_dir(), "the slot stands");
    assert!(bound.registration.is_dir(), "the registration stands");
    assert_eq!(
        std::fs::read(bound.slot.join(".git")).expect("the pointer reads"),
        bound.pointer,
        "the underlay pointer Git wrote stands"
    );
    remove(bound.root.as_path());
}

/// The global option no build of this wrapper reads,
/// which is the shape of an option a newer Git adds.
const A_GLOBAL_OPTION_THIS_BUILD_DOES_NOT_READ: &str = "--no-such-global-option-yet";

/// What:
///  A bound slot's removal behind a global option this build does not read is
///  refused,
///  and the sentence names that option and the spellings the wrapper reads.
/// Why:
///  The alpha.47 pass's finding:
///  a global option a newer Git adds,
///  typed before a relative slot path,
///  left the path unread,
///  so the removal was forwarded and Git deleted the registration and the
///  underlay pointer while the record still said `phase=filled`
///  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
#[test]
fn a_bound_slots_removal_behind_a_global_option_this_build_does_not_read_is_refused() {
    let bound: RegisteredSlot = a_registered_slot("entry-slot-removal-unread-global");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    let words: Vec<OsString> = started_in(
        bound.repository.as_path(),
        &[
            A_GLOBAL_OPTION_THIS_BUILD_DOES_NOT_READ,
            "worktree",
            "remove",
            "-f",
            "-f",
            "one",
        ],
    );
    let (code, stderr): (i32, String) = stopped(plan_invocation(
        words.as_slice(),
        &[],
        &inputs(bound.root.as_path()),
    ));
    assert_eq!(
        code,
        crate::entry_zfs_slot_commands::REFUSAL_STATUS,
        "the refusal is the wrapper's own status: {stderr}"
    );
    assert!(
        stderr.contains(A_GLOBAL_OPTION_THIS_BUILD_DOES_NOT_READ),
        "the refusal names the option it could not read: {stderr}"
    );
    assert!(
        stderr.contains("git -C <repository> worktree remove"),
        "the refusal names the spelling the wrapper reads: {stderr}"
    );
    assert!(bound.slot.is_dir(), "nothing was removed");
    assert!(bound.registration.is_dir(), "the registration stands");
    assert_eq!(
        std::fs::read(bound.slot.join(".git")).expect("the pointer reads"),
        bound.pointer,
        "the underlay pointer Git wrote stands"
    );
    remove(bound.root.as_path());
}

/// What:
///  A removal behind a global option this build does not read,
///  with a `-C` after it,
///  is refused even in a repository with no slot records,
///  and the sentence says the wrapper cannot tell which repository the command
///  names.
/// Why:
///  The owner's answer of 2026-11-26:
///  `git --future-option=x -C checkouts/repo worktree remove -f -f slot-one`
///  typed outside the repository stops the wrapper's reading at the option,
///  so it never learns that `-C` names the repository,
///  and was forwarded
///  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
#[test]
fn a_removal_behind_an_unread_global_that_names_the_repository_is_refused() {
    let unrecorded: RegisteredSlot =
        a_registered_slot("entry-slot-unread-global-then-named-repository");
    let words: Vec<OsString> = text(&[
        A_GLOBAL_OPTION_THIS_BUILD_DOES_NOT_READ,
        "-C",
        unrecorded.repository.to_str().expect("utf-8"),
        "worktree",
        "remove",
        "-f",
        "-f",
        "one",
    ]);
    let action: Action = plan_invocation(words.as_slice(), &[], &inputs(unrecorded.root.as_path()));
    let (code, stderr): (i32, String) = stopped(action);
    assert_eq!(
        code,
        crate::entry_zfs_slot_commands::REFUSAL_STATUS,
        "the refusal is the wrapper's own status: {stderr}"
    );
    assert!(
        stderr.contains(A_GLOBAL_OPTION_THIS_BUILD_DOES_NOT_READ),
        "the refusal names the option it could not read: {stderr}"
    );
    assert!(
        stderr.contains("which repository the command names"),
        "the refusal says what it could not prove: {stderr}"
    );
    assert!(
        stderr.contains("git -C <repository> worktree remove"),
        "the refusal names the spelling the wrapper reads: {stderr}"
    );
    assert!(unrecorded.slot.is_dir(), "nothing was removed");
    assert!(unrecorded.registration.is_dir(), "the registration stands");
    remove(unrecorded.root.as_path());
}

/// What:
///  A move behind a global option this build does not read,
///  with a `--git-dir` after it,
///  is refused for the same reason.
/// Why:
///  Both words carry the same answer,
///  and `--git-dir` names the repository as `-C` does,
///  so a reading that only knew `-C` would leave the move to Git.
#[test]
fn a_move_behind_an_unread_global_that_names_the_repository_is_refused() {
    let unrecorded: RegisteredSlot = a_registered_slot("entry-slot-move-unread-then-git-dir");
    let words: Vec<OsString> = text(&[
        A_GLOBAL_OPTION_THIS_BUILD_DOES_NOT_READ,
        "--git-dir",
        unrecorded.common.to_str().expect("utf-8"),
        "worktree",
        "move",
        "one",
        "../moved",
    ]);
    let action: Action = plan_invocation(words.as_slice(), &[], &inputs(unrecorded.root.as_path()));
    let (code, stderr): (i32, String) = stopped(action);
    assert_eq!(
        code,
        crate::entry_zfs_slot_commands::REFUSAL_STATUS,
        "the refusal is the wrapper's own status: {stderr}"
    );
    assert!(
        stderr.contains("git -C <repository> worktree move"),
        "the refusal names the spelling the wrapper reads: {stderr}"
    );
    assert!(unrecorded.slot.is_dir(), "nothing was moved");
    remove(unrecorded.root.as_path());
}

/// What:
///  A bound slot's move behind a global option this build does not read is
///  refused,
///  and the sentence names that option and the spellings the wrapper reads.
/// Why:
///  Both words are fixed by the same answer:
///  a move behind an option the wrapper does not know would rename the slot
///  while the record kept the old path,
///  exactly as the removal would delete it.
#[test]
fn a_bound_slots_move_behind_a_global_option_this_build_does_not_read_is_refused() {
    let bound: RegisteredSlot = a_registered_slot("entry-slot-move-unread-global");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    let words: Vec<OsString> = started_in(
        bound.repository.as_path(),
        &[
            A_GLOBAL_OPTION_THIS_BUILD_DOES_NOT_READ,
            "worktree",
            "move",
            "one",
            "../moved",
        ],
    );
    let (code, stderr): (i32, String) = stopped(plan_invocation(
        words.as_slice(),
        &[],
        &inputs(bound.root.as_path()),
    ));
    assert_eq!(
        code,
        crate::entry_zfs_slot_commands::REFUSAL_STATUS,
        "the refusal is the wrapper's own status: {stderr}"
    );
    assert!(
        stderr.contains(A_GLOBAL_OPTION_THIS_BUILD_DOES_NOT_READ),
        "the refusal names the option it could not read: {stderr}"
    );
    assert!(
        stderr.contains("git -C <repository> worktree move"),
        "the refusal names the spelling the wrapper reads: {stderr}"
    );
    assert!(bound.slot.is_dir(), "nothing was moved");
    remove(bound.root.as_path());
}

/// What:
///  A repository that holds no slot record forwards the same command as before.
/// Why:
///  The refusal exists for the records a slot would leave behind,
///  and an ordinary repository has none,
///  so its removal and its move must cost no refusal and reach Git.
#[test]
fn a_command_behind_a_global_option_this_build_does_not_read_is_git_s_own_without_records() {
    let unrecorded: RegisteredSlot = a_registered_slot("entry-slot-unread-global-no-records");
    for word in ["remove", "move"] {
        let words: Vec<OsString> = started_in(
            unrecorded.repository.as_path(),
            &[
                A_GLOBAL_OPTION_THIS_BUILD_DOES_NOT_READ,
                "worktree",
                word,
                "one",
                "../elsewhere",
            ],
        );
        assert!(
            matches!(
                plan_invocation(words.as_slice(), &[], &inputs(unrecorded.root.as_path())),
                Action::Forward { .. }
            ),
            "a repository with no slot records forwards the {word} as before"
        );
    }
    remove(unrecorded.root.as_path());
}

/// What:
///  A removal of a bound slot whose binding record cannot be read is refused
///  through the planner and the served run,
///  naming the record and what to repair.
/// Why:
///  The alpha.47 pass's finding 12:
///  with the record overwritten by garbage,
///  `git worktree move` refused while `git worktree remove -f -f <slot>` exited
///  0 and Git deleted the registration and the slot folder
///  (`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
#[test]
fn a_bound_slots_removal_with_a_record_that_cannot_be_read_is_refused() {
    let bound: RegisteredSlot = a_registered_slot("entry-slot-removal-unreadable-record");
    let home: PathBuf = crate::worktree_clone_binding_record::registration_home::home_folder(
        bound.common.as_path(),
    );
    std::fs::create_dir_all(home.as_path()).expect("the fixture home");
    std::fs::write(home.join("one.binding"), b"not a record\n").expect("the foreign record writes");
    let words: Vec<OsString> = started_in(
        bound.repository.as_path(),
        &["worktree", "remove", "-f", "-f", "one"],
    );
    let facts: CopyFacts = settled_facts(plan_invocation(
        words.as_slice(),
        &[],
        &inputs(bound.root.as_path()),
    ));
    let stderr: String = the_refusals_report(run_the_removal(&facts, words.as_slice()));
    assert!(
        stderr.contains("one.binding"),
        "the refusal names the record it could not read: {stderr}"
    );
    assert!(stderr.contains("cannot be read"), "{stderr}");
    assert!(
        stderr.contains("restore it from a copy"),
        "the refusal names what to repair: {stderr}"
    );
    assert!(bound.slot.is_dir(), "nothing was removed");
    assert!(bound.registration.is_dir(), "the registration stands");
    remove(bound.root.as_path());
}

/// What:
///  A removal of a slot with no binding record and a mount at its path is
///  refused through the planner and the served run,
///  naming that mount,
///  and no pool is read.
/// Why:
///  The owner's answer of 2026-11-26 refuses the removal where no record proves
///  what a slot holds:
///  Git's own removal would delete the registration and the underlay folder while
///  the mount at the slot may hold a managed clone's files.
#[test]
fn a_removal_of_a_slot_with_a_mount_and_no_record_is_refused() {
    let unrecorded: RegisteredSlot = a_registered_slot("entry-slot-mount-with-no-record");
    // A repository the route has served holds its record folder, and this slot has no record in it.
    publish_under(
        unrecorded.common.as_path(),
        &record_naming(
            Phase::Filled,
            unrecorded.root.join("two").as_path(),
            unrecorded.registration.as_path(),
            Some(unrecorded.pointer.clone()),
        ),
        "two",
    );
    let slot: String = unrecorded.slot.display().to_string();
    let mounts: ScriptedMountTable = ScriptedMountTable::holding(&[
        row(1, 0, "/", "/", "zfs", "tank/ROOT"),
        row(50, 1, "/", slot.as_str(), "zfs", "tank/some-other-dataset"),
    ]);
    let intents: ScriptedIntent = ScriptedIntent;
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    let words: Vec<OsString> = started_in(
        unrecorded.repository.as_path(),
        &["worktree", "remove", "-f", "-f", "one"],
    );
    let facts: CopyFacts = settled_facts(plan_invocation(
        words.as_slice(),
        &[],
        &inputs(unrecorded.root.as_path()),
    ));
    let stderr: String =
        the_refusals_report(run_the_removal_with(&facts, words.as_slice(), &sources));
    assert!(
        stderr.contains("tank/some-other-dataset"),
        "the refusal names the mount no record explains: {stderr}"
    );
    assert!(
        !stderr.contains(NO_POOL_WAS_ASKED),
        "the refusal reads no pool: {stderr}"
    );
    assert!(unrecorded.slot.is_dir(), "nothing was removed");
    assert!(unrecorded.registration.is_dir(), "the registration stands");
    remove(unrecorded.root.as_path());
}

/// What:
///  A removal of a path whose slot name holds a record naming another path is
///  refused through the planner and the served run,
///  naming both paths.
/// Why:
///  The owner's answer of 2026-11-26 refuses a removal no record proves,
///  and a record keyed by this name for another path proves nothing about the
///  worktree the command names.
#[test]
fn a_removal_whose_record_names_another_path_is_refused() {
    let bound: RegisteredSlot = a_registered_slot("entry-slot-record-of-another-path");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    let elsewhere: PathBuf = bound.root.join("elsewhere").join("one");
    std::fs::create_dir_all(elsewhere.as_path()).expect("the plain folder");
    let plain: PathBuf =
        std::fs::canonicalize(elsewhere.as_path()).expect("canonical plain folder");
    let words: Vec<OsString> = started_in(
        bound.repository.as_path(),
        &[
            "worktree",
            "remove",
            "-f",
            "-f",
            plain.to_str().expect("utf-8"),
        ],
    );
    let facts: CopyFacts = settled_facts(plan_invocation(
        words.as_slice(),
        &[],
        &inputs(bound.root.as_path()),
    ));
    assert_eq!(
        facts.removal_target.as_deref(),
        Some(plain.as_path()),
        "the planner reads the operand the command names"
    );
    let stderr: String = the_refusals_report(run_the_removal(&facts, words.as_slice()));
    assert!(
        stderr.contains("not this slot's record")
            && stderr.contains(bound.slot.to_str().expect("utf-8")),
        "the refusal names the path the record holds: {stderr}"
    );
    assert!(plain.is_dir(), "nothing was removed");
    assert!(bound.slot.is_dir(), "the bound slot stands");
    remove(bound.root.as_path());
}

/// What:
///  A removal of a filled slot at a moment the mount table cannot be read is
///  refused through the planner and the served run,
///  naming the reader's reason.
/// Why:
///  The owner's answer of 2026-11-26:
///  with the clone mounted and the mount table unreadable,
///  Git would delete inside the clone
///  (the alpha.47 pass's finding 12).
#[test]
fn a_removal_at_an_unreadable_mount_table_is_refused() {
    let bound: RegisteredSlot = a_registered_slot("entry-slot-unreadable-mount-table");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    let mounts: ScriptedMountTable =
        ScriptedMountTable::unreadable("the table is too large to read");
    let intents: ScriptedIntent = ScriptedIntent;
    let sources: SlotReadingSources<'_> = SlotReadingSources {
        mounts: &mounts,
        intents: &intents,
    };
    let words: Vec<OsString> = started_in(
        bound.repository.as_path(),
        &["worktree", "remove", "-f", "-f", "one"],
    );
    let facts: CopyFacts = settled_facts(plan_invocation(
        words.as_slice(),
        &[],
        &inputs(bound.root.as_path()),
    ));
    let stderr: String =
        the_refusals_report(run_the_removal_with(&facts, words.as_slice(), &sources));
    assert!(
        stderr.contains("the mount table could not be read")
            && stderr.contains("the table is too large to read"),
        "the refusal names the reader's reason: {stderr}"
    );
    assert!(bound.slot.is_dir(), "nothing was removed");
    assert!(bound.registration.is_dir(), "the registration stands");
    remove(bound.root.as_path());
}

/// What:
///  A bound slot's move is refused behind a `-c` option.
/// Why:
///  The move refusal read its source through the same gated reading,
///  so `git -c core.x=y worktree move -f -f one ../moved` moved the slot while
///  its record kept naming the old path.
#[test]
fn a_bound_slots_move_is_refused_behind_a_config_option() {
    let bound: RegisteredSlot = a_registered_slot("entry-slot-move-spellings");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    let (code, stderr): (i32, String) = stopped(plan_invocation(
        started_in(
            bound.repository.as_path(),
            &[
                "-c", "core.x=y", "worktree", "move", "-f", "-f", "one", "../moved",
            ],
        )
        .as_slice(),
        &[],
        &inputs(bound.root.as_path()),
    ));
    assert_eq!(
        code,
        crate::entry_zfs_slot_commands::REFUSAL_STATUS,
        "the refusal is the wrapper's own status: {stderr}"
    );
    assert!(
        stderr.contains("`git worktree move` is refused"),
        "a bound slot's move is refused: {stderr}"
    );
    assert!(bound.slot.is_dir(), "nothing was moved");
    remove(bound.root.as_path());
}

/// What:
///  A prune started inside the repository's Git directory restores a deleted
///  pointer before Git runs,
///  so Git keeps the managed registration.
/// Why:
///  The alpha.47 pass's run u1:
///  `git -C <repository>/.git worktree prune` reached no pass,
///  because a command that stands in a Git directory has no worktree identity,
///  and Git dropped the registration the record is the only key to.
#[test]
fn a_prune_started_in_the_git_directory_restores_the_pointer() {
    let bound: RegisteredSlot = a_registered_slot("entry-slot-prune-spellings");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    std::fs::remove_file(bound.slot.join(".git"))
        .expect("the pointer goes, as an account's delete leaves it");
    let words: Vec<OsString> = text(&[
        "-C",
        bound.common.to_str().expect("utf-8"),
        "worktree",
        "prune",
    ]);
    let action: Action = plan_invocation(words.as_slice(), &[], &inputs(bound.root.as_path()));
    assert!(
        matches!(action, Action::Forward { .. }),
        "the pass lets the prune run: {action:?}"
    );
    assert_eq!(
        std::fs::read(bound.slot.join(".git")).expect("the pointer the pass wrote back"),
        bound.pointer
    );
    // Git itself keeps the registration once the pointer is there, which is what the pass exists for.
    let run: ForwardedRun = run_forwarded_git(
        Path::new(REAL_GIT),
        words.as_slice(),
        isolated_overlay().as_slice(),
        None,
    );
    assert_eq!(run, ForwardedRun::Ran(0), "the prune itself ends well");
    assert!(
        bound.registration.is_dir(),
        "the registration the record names stands"
    );
    remove(bound.root.as_path());
}

/// What:
///  The repair report reaches a command line started inside the repository's Git
///  directory,
///  and one naming no path prints nothing.
/// Why:
///  The slot glue a command meets reads the move,
///  the repair and the prune words in one place,
///  so a change that drops the repair report from that place would leave every
///  control of the report itself green.
#[test]
fn a_repair_report_reaches_the_command_line() {
    let bound: RegisteredSlot = a_registered_slot("entry-slot-repair-report");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    std::fs::remove_file(bound.slot.join(".git")).expect("the pointer goes");
    let action: Action = plan_invocation(
        text(&[
            "-C",
            bound.common.to_str().expect("utf-8"),
            "worktree",
            "repair",
            "one",
        ])
        .as_slice(),
        &[],
        &inputs(bound.root.as_path()),
    );
    match action {
        Action::Forward {
            stderr,
            ..
        } => assert!(
            stderr.contains("link: damaged, and the registration stands"),
            "the report a repair prints before Git runs: {stderr}"
        ),
        other => panic!("a repair is Git's own command: {other:?}"),
    }
    remove(bound.root.as_path());
}

/// What:
///  A command started inside a Git directory whose repository holds no record
///  is forwarded unchanged,
///  and one whose records name no slot it removes is carried out by Git.
/// Why:
///  Every forwarded command meets the slot glue,
///  and the ordinary removal in such a directory must still be Git's own,
///  with no refusal and no report;
///  the run that carries a named removal out forwards it once the refusal
///  answered nothing,
///  which is what the second half shows by taking the worktree away.
#[test]
fn a_removal_that_names_no_bound_slot_is_git_s_own() {
    let unrecorded: RegisteredSlot = a_registered_slot("entry-slot-plain-in-git-directory");
    let words: Vec<OsString> = text(&[
        "-C",
        unrecorded.common.to_str().expect("utf-8"),
        "worktree",
        "remove",
        "-f",
        "one",
    ]);
    assert_eq!(
        plan_invocation(words.as_slice(), &[], &inputs(unrecorded.root.as_path())),
        Action::Forward {
            real_git: PathBuf::from(REAL_GIT),
            arguments: words.clone(),
            overlay: invocation_overlay(&[], Path::new(REAL_GIT)),
            stderr: String::new(),
        },
        "a repository with no record settles nothing"
    );
    remove(unrecorded.root.as_path());
    // A repository whose record binds another slot: the removal names this worktree, so the run it ends in is settled,
    // and Git takes the worktree once the refusal has nothing to say about it.
    let bound: RegisteredSlot = a_registered_slot("entry-slot-plain-beside-a-record");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    let plain: PathBuf = bound.root.join("elsewhere").join("two");
    std::fs::create_dir_all(bound.root.join("elsewhere")).expect("elsewhere");
    crate::test_support::git(
        bound.repository.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            plain.to_str().expect("utf-8"),
            "HEAD",
        ],
    );
    let removal: Vec<OsString> = text(&[
        "-C",
        bound.common.to_str().expect("utf-8"),
        "worktree",
        "remove",
        "-f",
        "two",
    ]);
    let facts: CopyFacts = settled_facts(plan_invocation(
        removal.as_slice(),
        &[],
        &inputs(bound.root.as_path()),
    ));
    assert_eq!(
        run_the_removal(&facts, removal.as_slice()),
        ForwardedRun::Ran(0),
        "the removal Git was asked for ends well"
    );
    assert!(!plain.exists(), "Git removed the worktree it was told to");
    assert!(bound.slot.is_dir(), "the bound slot stands");
    remove(bound.root.as_path());
}

/// The command line as the wrapper resolved it,
/// with `subcommand_index` at the `worktree` word the global options stand in
/// front of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const command = { arguments: words, subcommandIndex: words.indexOf('worktree') };
/// ```
fn move_command(words: &[&str]) -> ResolvedCommand {
    let arguments: Vec<OsString> = text(words);
    let index: usize = arguments
        .iter()
        .position(|word| return word.as_encoded_bytes() == b"worktree")
        .expect("the move's own word stands in the line");
    return ResolvedCommand {
        arguments,
        subcommand_index: index,
        subcommand: Some(OsString::from("worktree")),
        route: crate::forwarded_command::Route::Direct,
        shell_command: None,
    };
}

/// What:
///  The move refusal reads the slot behind a global option that changes nothing
///  about where the operand lies.
/// Why:
///  A `-c` or `--no-optional-locks` option must not hide the slot from the
///  refusal,
///  and the move glue is where that reading lives.
#[test]
fn the_move_refusal_reads_the_slot_behind_a_global_option() {
    let bound: RegisteredSlot = a_registered_slot("entry-slot-move-glue");
    publish_under(
        bound.common.as_path(),
        &record_naming(
            Phase::Filled,
            bound.slot.as_path(),
            bound.registration.as_path(),
            Some(bound.pointer.clone()),
        ),
        "one",
    );
    for words in [
        &["worktree", "move", "one", "../moved"][..],
        &["-c", "core.x=y", "worktree", "move", "one", "../moved"][..],
        &[
            "--no-optional-locks",
            "worktree",
            "move",
            "-f",
            "one",
            "../moved",
        ][..],
    ] {
        let ending = crate::entry_zfs_slot_commands::move_refusal_of(
            &move_command(words),
            Some(bound.common.as_path()),
            Some(bound.repository.as_path()),
        );
        assert!(
            ending.is_some(),
            "the slot's own move is refused for {words:?}"
        );
    }
    remove(bound.root.as_path());
}
