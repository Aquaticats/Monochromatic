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
/// The settings file name a control writes to turn the root rule down.
use crate::config_file::CONFIG_FILE_NAME;
use crate::forwarded_command::{
    MAX_ALIAS_EXPANSIONS,
    ResolvedCommand,
};
/// The rule's own finding and the renderer that prints the engine's events.
use crate::policy_events::{
    FindingEvent,
    PolicyEvent,
    render_policy_events,
};
/// The identities and severity of the policy a control turns down.
use crate::policy_registry::{
    PolicyId,
    Severity,
};
/// The lifecycle point a forwarded command's policies run at.
use crate::policy_trigger::Trigger;
use crate::real_git::ResolutionInputs;
/// The root rule's own decision and the code its finding carries.
use crate::rule_require_root::{
    NOT_AT_ROOT_CODE,
    RequireRootFacts,
    RequireRootVerdict,
    resolve_require_root,
};
use crate::test_support::{
    REAL_GIT,
    executable,
    fixture,
    isolated_overlay,
    remove,
    repository,
};
use crate::testing::fixture_zfs_binding_record::{
    CLONE_DATASET,
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
/// The move's refusal, which a control builds for the slot it means.
use crate::worktree_clone_move_refusal::refuse_move_of_a_bound_slot;
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
/// The finished child each real Git fixture run below is.
use std::process::Output;

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

/// What:
///  The `-c` words of one chain of `links` plain aliases,
///  each naming the next,
///  whose last names `command`,
///  then the first alias's name and the caller's own words after it.
/// Why:
///  A chain is read one link at a time,
///  so both ends of the count the reading keeps have to be shown:
///  a chain as long as the count reaches the command Git runs,
///  and one link more is no command at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // ['-c', 'alias.fixturechain0=fixturechain1', ..., 'fixturechain0', '-f', '-f', 'one']
/// const words = aChainOfAliases(MAX_ALIAS_EXPANSIONS, 'worktree remove', ['-f', '-f', 'one']);
/// ```
fn a_chain_of_aliases(
    links: usize,
    command: &str,
    then: &[&str],
) -> Vec<OsString> {
    let mut words: Vec<OsString> = Vec::new();
    for index in 0..links {
        let target: String = if index + 1 < links {
            format!("fixturechain{}", index + 1)
        } else {
            String::from(command)
        };
        words.push(OsString::from("-c"));
        words.push(OsString::from(format!(
            "alias.fixturechain{index}={target}"
        )));
    }
    words.push(OsString::from("fixturechain0"));
    words.extend(text(then));
    return words;
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

/// What:
///  A record folder the route made and left holding no record refuses nothing,
///  and the same command is refused once one record is filed in it.
/// Why:
///  The unknown-global refusal protects a slot that has a record,
/// and `SPEC.md`
///  says it applies "while the repository holds slot records":
///  a folder that holds none endangers no slot,
/// so the command is Git's own
///  there,
/// and the second arm proves the folder alone was not what refused it.
#[test]
fn a_record_folder_that_holds_no_record_refuses_nothing_behind_an_unread_global() {
    let unrecorded: RegisteredSlot =
        a_registered_slot("entry-slot-unread-global-empty-record-folder");
    let home: PathBuf = crate::worktree_clone_binding_record::registration_home::home_folder(
        unrecorded.common.as_path(),
    );
    std::fs::create_dir_all(home.as_path()).expect("the fixture home");
    let words: Vec<OsString> = started_in(
        unrecorded.repository.as_path(),
        &[
            A_GLOBAL_OPTION_THIS_BUILD_DOES_NOT_READ,
            "worktree",
            "remove",
            "-f",
            "-f",
            "one",
        ],
    );
    let action: Action = plan_invocation(words.as_slice(), &[], &inputs(unrecorded.root.as_path()));
    assert!(
        matches!(action, Action::Forward { .. }),
        "a record folder holding no record forwards the removal as before: {action:?}"
    );
    publish_under(
        unrecorded.common.as_path(),
        &record_naming(
            Phase::Filled,
            unrecorded.slot.as_path(),
            unrecorded.registration.as_path(),
            Some(unrecorded.pointer.clone()),
        ),
        "one",
    );
    let (code, stderr): (i32, String) = stopped(plan_invocation(
        words.as_slice(),
        &[],
        &inputs(unrecorded.root.as_path()),
    ));
    assert_eq!(
        code,
        crate::entry_zfs_slot_commands::REFUSAL_STATUS,
        "the same folder with one record refuses the removal: {stderr}"
    );
    assert!(
        stderr.contains(A_GLOBAL_OPTION_THIS_BUILD_DOES_NOT_READ),
        "the refusal names the option it could not read: {stderr}"
    );
    assert!(unrecorded.slot.is_dir(), "nothing was removed");
    remove(unrecorded.root.as_path());
}

/// What:
///  A `--` before the command word is refused in words that say what it is,
///  not as a global option this wrapper does not read.
/// Why:
///  `--` is the separator a command's own options use before their operands,
///  and Git 2.56.1 answers it with `unknown option: --` before its command
///  word (`/usr/bin/git -- worktree list`,
/// exit 129),
/// so calling it a global
///  option named something else.
#[test]
fn a_double_dash_before_the_command_word_is_refused_as_the_separator_git_refuses() {
    let bound: RegisteredSlot = a_registered_slot("entry-slot-double-dash");
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
        &["--", "worktree", "remove", "-f", "-f", "one"],
    );
    let (code, stderr): (i32, String) = stopped(plan_invocation(
        words.as_slice(),
        &[],
        &inputs(bound.root.as_path()),
    ));
    assert_eq!(
        code,
        crate::entry_zfs_slot_commands::REFUSAL_STATUS,
        "the removal is refused: {stderr}"
    );
    assert!(
        stderr.contains("not a global option"),
        "the sentence says what `--` is: {stderr}"
    );
    assert!(
        stderr.contains("unknown option: --"),
        "and what Git answers it with: {stderr}"
    );
    assert!(
        !stderr.contains("`--` is a global option this wrapper does not read"),
        "and no longer calls it an unread global: {stderr}"
    );
    assert!(bound.slot.is_dir(), "nothing was removed");
    remove(bound.root.as_path());
}

/// One fixture repository with a bound slot whose record is readable,
/// the record every shell-alias control below is refused for.
fn a_bound_slot_for_a_shell_alias(name: &str) -> RegisteredSlot {
    let bound: RegisteredSlot = a_registered_slot(name);
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
    return bound;
}

/// What:
///  A bound slot's removal is refused behind every shell-alias spelling the
///  release pass named:
///  on the command line,
/// in a configuration file,
/// with a bare `!git`,
/// and
///  through a plain alias that chains to a shell alias.
/// Why:
///  The pass's Medium 1:
///  `git -c 'alias.x=!git worktree remove' x -f -f <slot>` deleted the
///  registration in alpha.47 and alpha.48 while the plain-alias spelling was
///  refused,
/// because the planner read the alias's own name as the command word.
///  The direct spelling's refusal is beside the alias spellings as the control
///  that the refusal itself is what the fixture produces.
#[test]
fn a_bound_slots_removal_is_refused_behind_every_shell_alias() {
    let bound: RegisteredSlot = a_bound_slot_for_a_shell_alias("entry-slot-removal-shell-aliases");
    crate::test_support::git(
        bound.repository.as_path(),
        &["config", "alias.filed", "!git worktree remove"],
    );
    let spellings: Vec<(PathBuf, Vec<OsString>)> = vec![
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.cmdline=!git worktree remove",
                    "cmdline",
                    "-f",
                    "-f",
                    "one",
                ],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.bare=!git",
                    "bare",
                    "worktree",
                    "remove",
                    "-f",
                    "-f",
                    "one",
                ],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.outer=inner",
                    "-c",
                    "alias.inner=!git worktree remove",
                    "outer",
                    "-f",
                    "-f",
                    "one",
                ],
            ),
        ),
        // The alias in a configuration file, which the lookup reads with the command started in the
        //  repository; `-C` puts the policy's effective working directory at that same repository.
        (
            bound.repository.clone(),
            started_in(bound.repository.as_path(), &["filed", "-f", "-f", "one"]),
        ),
        // The direct spelling, whose refusal the alias spellings must reach.
        (
            bound.repository.clone(),
            started_in(
                bound.repository.as_path(),
                &["worktree", "remove", "-f", "-f", "one"],
            ),
        ),
        // A blank between `!` and `git` is one separator to the shell, so `! git` runs `git`.
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.spaced=! git worktree remove",
                    "spaced",
                    "-f",
                    "-f",
                    "one",
                ],
            ),
        ),
        // A TAB separates the words of a shell command as a space does.
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.tabbed=!git\tworktree remove",
                    "tabbed",
                    "-f",
                    "-f",
                    "one",
                ],
            ),
        ),
        // A value of `!git` alone leaves the command to the caller's own words, which may begin with a global.
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.bare=!git",
                    "bare",
                    "--no-pager",
                    "worktree",
                    "remove",
                    "-f",
                    "-f",
                    "one",
                ],
            ),
        ),
        // The shell alias that names another shell alias: Git starts a second shell, and that one runs the command.
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.doubleshell=!git inner",
                    "-c",
                    "alias.inner=!git worktree remove",
                    "doubleshell",
                    "-f",
                    "-f",
                    "one",
                ],
            ),
        ),
        // A chain of ordinary aliases as long as the reading follows.
        (bound.root.clone(), {
            let mut chained: Vec<OsString> = started_in(bound.repository.as_path(), &[]);
            chained.extend(a_chain_of_aliases(
                MAX_ALIAS_EXPANSIONS,
                "worktree remove",
                &["-f", "-f", "one"],
            ));
            chained
        }),
    ];
    for (start, words) in &spellings {
        let facts: CopyFacts = settled_facts(plan_invocation(
            words.as_slice(),
            &[],
            &inputs(start.as_path()),
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
        assert!(bound.slot.is_dir(), "{words:?}: nothing was removed");
    }
    remove(bound.root.as_path());
}

/// What:
///  A bound slot's move is refused behind every shell-alias spelling,
///  and the direct spelling is refused beside them.
/// Why:
///  The move is the other command the owner's answer names,
/// and a shell alias
///  that read its own name left the slot to Git,
/// which renamed it while the
///  record kept the old path.
#[test]
fn a_bound_slots_move_is_refused_behind_every_shell_alias() {
    let bound: RegisteredSlot = a_bound_slot_for_a_shell_alias("entry-slot-move-shell-aliases");
    crate::test_support::git(
        bound.repository.as_path(),
        &["config", "alias.filed", "!git worktree move"],
    );
    let spellings: Vec<(PathBuf, Vec<OsString>)> = vec![
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.cmdline=!git worktree move",
                    "cmdline",
                    "one",
                    "../moved",
                ],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.bare=!git",
                    "bare",
                    "worktree",
                    "move",
                    "one",
                    "../moved",
                ],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.outer=inner",
                    "-c",
                    "alias.inner=!git worktree move",
                    "outer",
                    "one",
                    "../moved",
                ],
            ),
        ),
        (
            bound.repository.clone(),
            started_in(bound.repository.as_path(), &["filed", "one", "../moved"]),
        ),
        (
            bound.repository.clone(),
            started_in(
                bound.repository.as_path(),
                &["worktree", "move", "one", "../moved"],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.spaced=! git worktree move",
                    "spaced",
                    "one",
                    "../moved",
                ],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.tabbed=!git\tworktree move",
                    "tabbed",
                    "one",
                    "../moved",
                ],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.bare=!git",
                    "bare",
                    "--no-pager",
                    "worktree",
                    "move",
                    "one",
                    "../moved",
                ],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.doubleshell=!git inner",
                    "-c",
                    "alias.inner=!git worktree move",
                    "doubleshell",
                    "one",
                    "../moved",
                ],
            ),
        ),
        (bound.root.clone(), {
            let mut chained: Vec<OsString> = started_in(bound.repository.as_path(), &[]);
            chained.extend(a_chain_of_aliases(
                MAX_ALIAS_EXPANSIONS,
                "worktree move",
                &["one", "../moved"],
            ));
            chained
        }),
    ];
    for (start, words) in &spellings {
        let (code, stderr): (i32, String) = stopped(plan_invocation(
            words.as_slice(),
            &[],
            &inputs(start.as_path()),
        ));
        assert_eq!(
            code,
            crate::entry_zfs_slot_commands::REFUSAL_STATUS,
            "the move is refused for {words:?}: {stderr}"
        );
        assert!(
            stderr.contains("`git worktree move` is refused"),
            "{words:?}: {stderr}"
        );
        assert!(bound.slot.is_dir(), "{words:?}: nothing was moved");
    }
    remove(bound.root.as_path());
}

/// What:
///  A prune behind every shell-alias spelling restores the deleted pointer
///  before Git runs,
/// as the direct spelling does,
///  so Git keeps the registration.
/// Why:
///  The prune pass reads the command word to run at all,
///  and reading the alias's own name left the pointer gone,
///  after which Git dropped the registration the record is the only key to.
#[test]
fn a_prune_behind_every_shell_alias_restores_the_pointer() {
    let bound: RegisteredSlot = a_bound_slot_for_a_shell_alias("entry-slot-prune-shell-aliases");
    crate::test_support::git(
        bound.repository.as_path(),
        &["config", "alias.filed", "!git worktree prune"],
    );
    let spellings: Vec<(PathBuf, Vec<OsString>)> = vec![
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &["-c", "alias.cmdline=!git worktree prune", "cmdline"],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &["-c", "alias.bare=!git", "bare", "worktree", "prune"],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.outer=inner",
                    "-c",
                    "alias.inner=!git worktree prune",
                    "outer",
                ],
            ),
        ),
        (
            bound.repository.clone(),
            started_in(bound.repository.as_path(), &["filed"]),
        ),
        (
            bound.repository.clone(),
            started_in(bound.repository.as_path(), &["worktree", "prune"]),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &["-c", "alias.spaced=! git worktree prune", "spaced"],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &["-c", "alias.tabbed=!git\tworktree prune", "tabbed"],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.bare=!git",
                    "bare",
                    "--no-pager",
                    "worktree",
                    "prune",
                ],
            ),
        ),
        (
            bound.root.clone(),
            started_in(
                bound.repository.as_path(),
                &[
                    "-c",
                    "alias.doubleshell=!git inner",
                    "-c",
                    "alias.inner=!git worktree prune",
                    "doubleshell",
                ],
            ),
        ),
        (bound.root.clone(), {
            let mut chained: Vec<OsString> = started_in(bound.repository.as_path(), &[]);
            chained.extend(a_chain_of_aliases(
                MAX_ALIAS_EXPANSIONS,
                "worktree prune",
                &[],
            ));
            chained
        }),
    ];
    for (start, words) in &spellings {
        std::fs::remove_file(bound.slot.join(".git")).expect("the pointer goes");
        let action: Action = plan_invocation(words.as_slice(), &[], &inputs(start.as_path()));
        assert!(
            matches!(action, Action::Forward { .. }),
            "the pass lets the prune run for {words:?}: {action:?}"
        );
        assert_eq!(
            std::fs::read(bound.slot.join(".git")).expect("the pointer the pass wrote back"),
            bound.pointer,
            "{words:?}: the pointer stands again before Git runs"
        );
    }
    remove(bound.root.as_path());
}

/// What:
///  The three slot commands behind a shell alias held in `GIT_CONFIG_*`
///  reach the same refusal and the same pass as the direct spellings.
/// Why:
///  The release pass's `alias-env-shell` deleted the registration while
///  `alias-env-plain` was refused;
///  `GIT_CONFIG_*` is one environment of the whole process,
///  so this control runs its body in a child whose environment holds the three
///  aliases and nothing of this run's other tests.
#[test]
fn a_bound_slots_slot_commands_behind_an_alias_in_the_environment_are_refused() {
    let control: &str =
        "a_bound_slots_slot_commands_behind_an_alias_in_the_environment_are_refused";
    let environment: [(&str, &std::ffi::OsStr); 7] = [
        ("GIT_CONFIG_COUNT", std::ffi::OsStr::new("3")),
        ("GIT_CONFIG_KEY_0", std::ffi::OsStr::new("alias.envremove")),
        (
            "GIT_CONFIG_VALUE_0",
            std::ffi::OsStr::new("!git worktree remove"),
        ),
        ("GIT_CONFIG_KEY_1", std::ffi::OsStr::new("alias.envmove")),
        (
            "GIT_CONFIG_VALUE_1",
            std::ffi::OsStr::new("!git worktree move"),
        ),
        ("GIT_CONFIG_KEY_2", std::ffi::OsStr::new("alias.envprune")),
        (
            "GIT_CONFIG_VALUE_2",
            std::ffi::OsStr::new("!git worktree prune"),
        ),
    ];
    crate::testing::process_wide_state_child_test_support::run_the_body_with_an_environment_of_its_own(
        module_path!(),
        control,
        environment.as_slice(),
        &[],
        || {
            // What: The child holds the aliases in its process environment for the Git it starts, and the
            //  planning is handed the same list so the wrapper's lock-pid overlay counts them and adds its own
            //  entry after them.
            // Why: An overlay built from an empty list sets `GIT_CONFIG_COUNT` to one, and the aliases the
            //  process environment holds are then past the count Git reads.
            let aliases: Vec<(OsString, OsString)> = environment
                .iter()
                .map(|(name, value)| {
                    return (OsString::from(name), value.to_os_string());
                })
                .collect();
            let bound: RegisteredSlot = a_bound_slot_for_a_shell_alias(
                "entry-slot-shell-alias-environment",
            );
            let removal: Vec<OsString> = started_in(
                bound.repository.as_path(),
                &["envremove", "-f", "-f", "one"],
            );
            let facts: CopyFacts = settled_facts(plan_invocation(
                removal.as_slice(),
                aliases.as_slice(),
                &inputs(bound.root.as_path()),
            ));
            let stderr: String = the_refusals_report(run_the_removal(&facts, removal.as_slice()));
            assert!(
                stderr.contains("a whole managed clone is bound to this slot"),
                "the environment alias's removal is refused: {stderr}"
            );
            let moved: Vec<OsString> = started_in(
                bound.repository.as_path(),
                &["envmove", "one", "../moved"],
            );
            let (code, move_stderr): (i32, String) = stopped(plan_invocation(
                moved.as_slice(),
                aliases.as_slice(),
                &inputs(bound.root.as_path()),
            ));
            assert_eq!(
                code,
                crate::entry_zfs_slot_commands::REFUSAL_STATUS,
                "the environment alias's move is refused: {move_stderr}"
            );
            assert!(
                move_stderr.contains("`git worktree move` is refused"),
                "{move_stderr}"
            );
            std::fs::remove_file(bound.slot.join(".git")).expect("the pointer goes");
            let pruned: Vec<OsString> =
                started_in(bound.repository.as_path(), &["envprune"]);
            let action: Action = plan_invocation(
                pruned.as_slice(),
                aliases.as_slice(),
                &inputs(bound.root.as_path()),
            );
            assert!(
                matches!(action, Action::Forward { .. }),
                "the environment alias's prune pass runs: {action:?}"
            );
            assert_eq!(
                std::fs::read(bound.slot.join(".git")).expect("the pointer the pass wrote back"),
                bound.pointer
            );
            assert!(bound.slot.is_dir(), "nothing was removed");
            remove(bound.root.as_path());
        },
    );
}

/// What:
///  A shell alias the wrapper cannot read is left to Git,
///  as the guard and `SPEC.md` leave it.
/// Why:
///  `!sh -c '...'` quotes the command,
/// so no reading of the alias value can
///  know what the shell runs;
///  the release pass measured that it deletes the slot,
/// and the proposal under
///  `doc/planning/` asks the owner whether that should change.
///  This control pins the boundary the release keeps so a later change is a
///  decision and not an accident.
#[test]
fn a_shell_alias_the_wrapper_cannot_read_is_left_to_git() {
    let bound: RegisteredSlot = a_bound_slot_for_a_shell_alias("entry-slot-shell-alias-unread");
    let words: Vec<OsString> = started_in(
        bound.repository.as_path(),
        &[
            "-c",
            "alias.quoted=!sh -c 'git worktree remove -f -f one'",
            "quoted",
        ],
    );
    let action: Action = plan_invocation(words.as_slice(), &[], &inputs(bound.root.as_path()));
    assert!(
        matches!(action, Action::Forward { .. }),
        "a shell alias the wrapper cannot read reaches Git unchanged: {action:?}"
    );
    assert!(bound.slot.is_dir(), "the planning removed nothing");
    remove(bound.root.as_path());
}

/// The `-c` value whose shell runs one `git worktree remove`,
///  which the reading follows.
const ALIAS_REMOVES: &str = "alias.rmwt=!git worktree remove";

/// The `-c` value whose shell runs one `git worktree move`,
///  which the reading follows.
const ALIAS_MOVES: &str = "alias.mvwt=!git worktree move";

/// The `-c` value whose shell runs one `git worktree prune`,
///  which the reading follows.
const ALIAS_PRUNES: &str = "alias.prwt=!git worktree prune";

/// What:
///  One `rev-parse` answer as the top of the working tree and the prefix Git ran in.
/// Why:
///  A shell alias's shell prints those two lines from where Git put it,
///  so a control reads the directory from Git rather than from a second reading of
///  Git's rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [top, prefix] = output.trimEnd().split('\n');
/// ```
fn where_the_alias_ran(output: &Output) -> (String, String) {
    let said: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    let mut lines = said.lines();
    let top: String = lines.next().unwrap_or_default().to_string();
    let prefix: String = lines.next().unwrap_or_default().to_string();
    return (top, prefix);
}

/// What:
///  A subdirectory of a fixture repository,
///  made so a control can start a command below the top of the working tree.
/// Why:
///  The directory a `!` alias's shell runs in is the top,
///  so a control needs a start whose own directory differs from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sub = mkdir(repository + '/sub');
/// ```
fn a_subdirectory_of(repository: &Path) -> PathBuf {
    let sub: PathBuf = repository.join("sub");
    std::fs::create_dir_all(sub.as_path()).expect("the repository's subdirectory");
    return sub;
}

/// What:
///  Real Git runs the shell of a `!` alias at the top of the working tree,
///  both for a caller below the top and for a caller outside the repository who
///  named a directory inside it with `-C`.
/// Why:
///  The controls after this one say the slot refusals read a shell alias's
///  operand from that directory;
///  this control measures the directory with Git's own answer,
///  so no refusal rests on a second reading of Git's rules.
///  A repository Git found by walking up to it is the case that changes
///  directory
///  (`SPEC.md`,
///  the guard's passage),
///  and both spellings here are that case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // cwd `<root>/sub`, git -c 'alias.where=!git rev-parse --show-toplevel --show-prefix' where
/// //   => ['<root>', ''];
/// // cwd `<root>`, git -C `<root>/sub` -c ... where => ['<root>', '']
/// ```
#[test]
fn a_shell_alias_runs_at_the_top_of_the_working_tree() {
    let root: PathBuf = fixture("slot-refusals-alias-directory");
    let repository: PathBuf = repository(root.as_path(), "repository");
    let sub: PathBuf = a_subdirectory_of(repository.as_path());
    let asks_the_top: &str = "alias.where=!git rev-parse --show-toplevel --show-prefix";
    let inside: Output = crate::test_support::git(sub.as_path(), &["-c", asks_the_top, "where"]);
    assert_eq!(
        where_the_alias_ran(&inside),
        (repository.display().to_string(), String::new()),
        "a caller below the top gets the top"
    );
    let outside: Output = crate::test_support::git(
        root.as_path(),
        &[
            "-C",
            sub.to_str().expect("the fixture's path is UTF-8"),
            "-c",
            asks_the_top,
            "where",
        ],
    );
    assert_eq!(
        where_the_alias_ran(&outside),
        (repository.display().to_string(), String::new()),
        "a caller outside the repository who named a directory inside it gets the top"
    );
    remove(root.as_path());
}

/// What:
///  A bound slot's removal is refused behind a shell alias whose operand is not
///  the end of the slot's registered path,
///  read from the top of the working tree where Git runs the alias's command,
///  and the direct spelling of that same operand from that same directory is
///  refused beside it.
/// Why:
///  alpha.49's finding M1:
///  `git -C repo -c 'alias.rmwt=!git worktree remove --force' rmwt ../one`,
///  typed outside the repository,
///  removed a bound slot's registration with no refusal,
///  because the command inside the alias's shell was read with the caller's `-C`
///  dropped and its operand looked for under the directory the wrapper was
///  started in,
///  while Git looked for it under the top.
///  Every operand here is a path rather than the slot's last name,
///  so the registry's own match reads none of them and the start directory alone
///  decides which worktree is named.
///  The caller stands outside the fixture and names where it stands with `-C`,
///  because a control never moves its own process,
///  and because the wrapper's own `require-root` policy refuses a command whose
///  directory is below the top before any slot refusal runs
///  (`a_command_below_the_top_is_refused_before_the_slot_reading`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // cwd `<root>`, `-C <root>/repository` alias `rmwt ../one` => refused, as the direct spelling is
/// ```
#[test]
fn a_bound_slots_removal_reads_its_operand_where_git_runs_the_alias() {
    let bound: RegisteredSlot =
        a_bound_slot_for_a_shell_alias("entry-slot-removal-alias-directory");
    let spellings: Vec<Vec<OsString>> = vec![
        // The caller named the top with `-C`, and the alias's own command carries none of that `-C`.
        started_in(
            bound.repository.as_path(),
            &["-c", ALIAS_REMOVES, "rmwt", "../one"],
        ),
        // The same, written with a `.` step so no registered path ends with the operand.
        started_in(
            bound.repository.as_path(),
            &["-c", ALIAS_REMOVES, "rmwt", "./../one"],
        ),
        // The direct spelling of the same operand from the directory Git runs a direct command in.
        started_in(
            bound.repository.as_path(),
            &["worktree", "remove", "-f", "-f", "../one"],
        ),
    ];
    for words in &spellings {
        let facts: CopyFacts = settled_facts(plan_invocation(
            words.as_slice(),
            &[],
            &inputs(bound.root.as_path()),
        ));
        // `mut` allows the slot's canonical path to be read once beside the plan's own.
        let slot_as_read: PathBuf =
            std::fs::canonicalize(bound.slot.as_path()).expect("the slot's path reads");
        assert_eq!(
            facts.removal_target.as_deref(),
            Some(slot_as_read.as_path()),
            "{words:?} names the slot"
        );
        let stderr: String = the_refusals_report(run_the_removal(&facts, words.as_slice()));
        assert!(
            stderr.contains("a whole managed clone is bound to this slot"),
            "{words:?}: {stderr}"
        );
        assert!(bound.slot.is_dir(), "{words:?}: nothing was removed");
    }
    remove(bound.root.as_path());
}

/// What:
///  A command whose directory is below the top of its repository is refused by
///  the wrapper's own `require-root` policy,
///  before the slot refusals read anything,
///  so a control cannot show the subdirectory spelling of a shell alias's
///  operand.
/// Why:
///  This names the limit of the control above rather than leaving a reader to
///  think the subdirectory spelling was measured and refused by the slot rule:
///  it is refused earlier,
///  by a policy that says nothing about slots,
///  and a person who bypasses that policy is the one the slot refusal protects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // cwd `<root>`, `-C <root>/repository/sub` worktree remove -f -f ../../one
/// //   => require-root/not-at-root
/// ```
#[test]
fn a_command_below_the_top_is_refused_before_the_slot_reading() {
    let bound: RegisteredSlot = a_bound_slot_for_a_shell_alias("entry-slot-alias-below-the-top");
    let sub: PathBuf = a_subdirectory_of(bound.repository.as_path());
    let words: Vec<OsString> = started_in(sub.as_path(), &["-c", ALIAS_REMOVES, "rmwt", "../one"]);
    let (code, stderr): (i32, String) = stopped(plan_invocation(
        words.as_slice(),
        &[],
        &inputs(bound.root.as_path()),
    ));
    assert_eq!(code, 1, "the policy's own status: {stderr}");
    assert!(
        stderr.contains("require-root/not-at-root"),
        "the directory below the top is what stops it: {stderr}"
    );
    assert!(bound.slot.is_dir(), "nothing was removed");
    remove(bound.root.as_path());
}

/// What:
///  One way a command typed below the top of its repository gets past the
///  wrapper's own `require-root` rule.
/// Why:
///  The rule refuses such a command before any slot refusal runs,
///  so a control of the slot refusal below the top has to put the rule out of
///  the way first,
///  and each way it is out of the way is a spelling of its own:
///  the settings key at `off`,
///  the settings key at `warn`
///  (`config_parse_value_tests.rs`),
///  and the wrapper's own `--no-enforce-require-root`.
///  Both keys are read where the engine asks whether a policy applies
///  (`policy_engine::policy_applies`),
///  and the flag is the escape that same decision reads
///  (`wrapper_controls::is_escaped`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PastTheRootRule = 'off' | 'warn' | 'flag';
/// ```
#[derive(Clone, Copy, Debug)]
enum PastTheRootRule {
    /// `policies.require-root` reads `off`, so the engine never asks the rule.
    Off,
    /// `policies.require-root` reads `warn`, so the finding prints and the
    /// command goes on to the slot refusal.
    Warn,
    /// The caller writes `--no-enforce-require-root` before the command word.
    Flag,
}

impl PastTheRootRule {
    /// What:
    ///  The settings document that turns the rule down,
    ///  or nothing where the caller's own words do it.
    /// Why:
    ///  Where the rule keeps the severity that refuses, the fixture's settings
    ///  file stays as an ordinary repository's is.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const document = rule === 'flag' ? undefined : `{ "policies": { "require-root": "${rule}" } }`;
    /// ```
    fn settings_document(self) -> Option<&'static str> {
        return match self {
            Self::Off => Some(r#"{ "policies": { "require-root": "off" } }"#),
            Self::Warn => Some(r#"{ "policies": { "require-root": "warn" } }"#),
            Self::Flag => None,
        };
    }

    /// The wrapper's own words a control writes before the alias word,
    /// which the wrapper strips before Git sees them.
    fn wrapper_words(self) -> &'static [&'static str] {
        return match self {
            Self::Off | Self::Warn => &[],
            Self::Flag => &["--no-enforce-require-root"],
        };
    }
}

/// What:
///  One fixture repository whose bound slot is watched from below its top,
///  and the subdirectory a control types the command in.
/// Why:
///  The command is typed in a directory below the top rather than at it,
///  because that is the case the root rule refuses and the alias's directory
///  differs from the caller's;
///  the settings file goes at the repository's top,
///  where the loader finds it.
fn a_bound_slot_watched_from_below(
    name: &str,
    rule: PastTheRootRule,
) -> (RegisteredSlot, PathBuf) {
    let bound: RegisteredSlot = a_bound_slot_for_a_shell_alias(name);
    if let Some(document) = rule.settings_document() {
        std::fs::write(bound.repository.join(CONFIG_FILE_NAME), document)
            .expect("the fixture's settings file");
    }
    let sub: PathBuf = a_subdirectory_of(bound.repository.as_path());
    return (bound, sub);
}

/// What:
///  The words a person types below the top:
///  the directory below the top the fixture is reached by,
///  the way past the root rule,
///  and then the shell alias that names the slot command.
/// Why:
///  A control never changes this process's directory,
///  and every fact the policies read is a Git query run in this process's own
///  directory with the caller's globals replayed (`git_facts` takes no
///  directory),
///  so `-C <the directory below the top>` is the one spelling that puts the
///  whole planning in the fixture.
///  The flag is a global option, so it stands before the alias word as the
///  caller's own options do;
///  the settings ways write no word at all.
fn typed_below_the_top(
    sub: &Path,
    rule: PastTheRootRule,
    then: &[&str],
) -> Vec<OsString> {
    let mut words: Vec<OsString> = vec![OsString::from("-C"), sub.as_os_str().to_os_string()];
    words.extend(text(rule.wrapper_words()));
    words.extend(text(then));
    return words;
}

/// What:
///  The words the root rule says about a command typed in `typed_in`,
///  written by the parts that make them.
/// Why:
///  A control that asserts what another part prints reads it from that part,
///  never from a copy written by hand:
///  `resolve_require_root` makes the finding's message and
///  `render_policy_events` makes the line,
///  so this control cannot pass on a wording the rule no longer uses.
///  At `off` and behind the flag the engine never asks the rule,
///  so the rule says nothing at all.
fn the_root_rules_words(
    rule: PastTheRootRule,
    bound: &RegisteredSlot,
    typed_in: &Path,
) -> String {
    let RequireRootVerdict::NotAtRoot(violation) = resolve_require_root(&RequireRootFacts {
        effective_directory: typed_in.to_path_buf(),
        repository_root: Some(bound.repository.clone()),
    }) else {
        panic!(
            "{} is below the top, so the rule has a finding",
            typed_in.display()
        );
    };
    return match rule {
        PastTheRootRule::Warn => render_policy_events(
            0,
            &[
                PolicyEvent::Finding(FindingEvent {
                    trigger: Trigger::PreForward,
                    policy: PolicyId::RequireRoot,
                    severity: Severity::Warn,
                    code: NOT_AT_ROOT_CODE,
                    message: violation.message,
                    path: None,
                    location: None,
                    fix_available: false,
                }),
                PolicyEvent::WarnUnsafe {
                    trigger: Trigger::PreForward,
                    policy: PolicyId::RequireRoot,
                },
            ],
        ),
        PastTheRootRule::Off | PastTheRootRule::Flag => String::new(),
    };
}

/// What:
///  A bound slot's removal typed below the top behind a shell alias is refused
///  when `rule` lets the command past the wrapper's root rule,
///  and the refusal names the slot it read.
/// Why:
///  The root rule refuses this command before any slot refusal runs
///  (`a_command_below_the_top_is_refused_before_the_slot_reading`),
///  so the slot refusal below the top has no control while the rule is at the
///  severity that refuses.
///  The operand is `../one`,
///  which names the slot only from the top of the working tree,
///  where Git runs the alias's shell;
///  read from the directory the person typed it in it names
///  `<repository>/one`, which is no worktree and holds no record,
///  so a planning handed the typed directory settles no removal at all.
fn a_removals_refusal_is_reached_below_the_top(
    name: &str,
    rule: PastTheRootRule,
) {
    let (bound, sub): (RegisteredSlot, PathBuf) = a_bound_slot_watched_from_below(name, rule);
    let words: Vec<OsString> = typed_below_the_top(
        sub.as_path(),
        rule,
        &["-c", ALIAS_REMOVES, "rmwt", "../one"],
    );
    let action: Action = plan_invocation(words.as_slice(), &[], &inputs(bound.root.as_path()));
    let (facts, said_first): (CopyFacts, String) = match action {
        Action::ForwardSettled {
            copy,
            stderr,
            ..
        } => (*copy, stderr),
        other => panic!(
            "a removal below the top must reach the slot refusal with {rule:?} letting it past \
             the wrapper's root rule, got {other:?}"
        ),
    };
    let slot_as_read: PathBuf =
        std::fs::canonicalize(bound.slot.as_path()).expect("the slot's path reads");
    assert_eq!(
        facts.removal_target.as_deref(),
        Some(slot_as_read.as_path()),
        "with {rule:?} the refusal reads the slot from the top for {words:?}"
    );
    assert_eq!(
        said_first,
        the_root_rules_words(rule, &bound, sub.as_path()),
        "what the root rule says before the refusal for {rule:?}"
    );
    let stderr: String = the_refusals_report(run_the_removal(&facts, words.as_slice()));
    assert!(
        stderr.starts_with(
            format!("the removal of `{}` is refused", slot_as_read.display()).as_str()
        ),
        "the refusal names the target it read for {rule:?}: {stderr}"
    );
    assert!(
        stderr.contains("a whole managed clone is bound to this slot"),
        "{rule:?}: {stderr}"
    );
    assert!(bound.slot.is_dir(), "{rule:?}: nothing was removed");
    remove(bound.root.as_path());
}

/// A bound slot's removal below the top is refused with the root rule off.
#[test]
fn below_the_top_behind_a_shell_alias_a_removal_is_refused_with_the_root_rule_off() {
    a_removals_refusal_is_reached_below_the_top(
        "entry-slot-below-top-removal-rule-off",
        PastTheRootRule::Off,
    );
}

/// A bound slot's removal below the top is refused with the root rule at warn,
/// and the rule's finding and its warn-unsafe line print before the refusal.
#[test]
fn below_the_top_behind_a_shell_alias_a_removal_is_refused_with_the_root_rule_at_warn() {
    a_removals_refusal_is_reached_below_the_top(
        "entry-slot-below-top-removal-rule-warn",
        PastTheRootRule::Warn,
    );
}

/// A bound slot's removal below the top is refused behind the wrapper's own
/// flag, which says nothing before the refusal.
#[test]
fn below_the_top_behind_a_shell_alias_a_removal_is_refused_behind_the_wrappers_flag() {
    a_removals_refusal_is_reached_below_the_top(
        "entry-slot-below-top-removal-rule-flag",
        PastTheRootRule::Flag,
    );
}

/// What:
///  A bound slot's move typed below the top behind a shell alias is refused
///  when `rule` lets the command past the wrapper's root rule,
///  and the refusal is the sentence the producer writes for the slot itself.
/// Why:
///  The move's source operand is resolved against the directory Git runs the
///  command in,
///  which for a shell alias is the top of the working tree;
///  the refusal sentence names the slot and the clone its record binds and
///  never the source path,
///  so the control builds the sentence the producer makes for the slot and
///  requires the planner's refusal to end with it.
///  A planning handed the typed directory resolves `<repository>/one`,
///  which is no worktree,
///  and the move is left to Git.
fn a_moves_refusal_is_reached_below_the_top(
    name: &str,
    rule: PastTheRootRule,
) {
    let (bound, sub): (RegisteredSlot, PathBuf) = a_bound_slot_watched_from_below(name, rule);
    let words: Vec<OsString> = typed_below_the_top(
        sub.as_path(),
        rule,
        &["-c", ALIAS_MOVES, "mvwt", "../one", "../moved"],
    );
    let (code, stderr): (i32, String) = stopped(plan_invocation(
        words.as_slice(),
        &[],
        &inputs(bound.root.as_path()),
    ));
    assert_eq!(
        code,
        crate::entry_zfs_slot_commands::REFUSAL_STATUS,
        "the move below the top is refused with {rule:?} letting it past the root rule: {stderr}"
    );
    let slot_as_read: PathBuf =
        std::fs::canonicalize(bound.slot.as_path()).expect("the slot's path reads");
    let Some(the_slots_own) =
        refuse_move_of_a_bound_slot(bound.common.as_path(), slot_as_read.as_path())
    else {
        panic!(
            "the slot at {} is bound, so its move is refused",
            slot_as_read.display()
        );
    };
    assert!(
        stderr.ends_with(format!("{}\n", the_slots_own.sentence).as_str()),
        "the refusal is the sentence for the slot read from the top for {rule:?}: {stderr}"
    );
    assert!(
        stderr.contains(CLONE_DATASET),
        "the refusal names the clone the slot's record binds for {rule:?}: {stderr}"
    );
    assert!(bound.slot.is_dir(), "{rule:?}: nothing was moved");
    remove(bound.root.as_path());
}

/// A bound slot's move below the top is refused with the root rule off.
#[test]
fn below_the_top_behind_a_shell_alias_a_move_is_refused_with_the_root_rule_off() {
    a_moves_refusal_is_reached_below_the_top(
        "entry-slot-below-top-move-rule-off",
        PastTheRootRule::Off,
    );
}

/// A bound slot's move below the top is refused with the root rule at warn.
#[test]
fn below_the_top_behind_a_shell_alias_a_move_is_refused_with_the_root_rule_at_warn() {
    a_moves_refusal_is_reached_below_the_top(
        "entry-slot-below-top-move-rule-warn",
        PastTheRootRule::Warn,
    );
}

/// A bound slot's move below the top is refused behind the wrapper's own flag.
#[test]
fn below_the_top_behind_a_shell_alias_a_move_is_refused_behind_the_wrappers_flag() {
    a_moves_refusal_is_reached_below_the_top(
        "entry-slot-below-top-move-rule-flag",
        PastTheRootRule::Flag,
    );
}
///  read from the top of the working tree where Git runs the alias's command.
/// Why:
///  `parse_move_source` resolves its source with the removal's own operand rule
///  (`worktree_remove_registry::path_git_picks_for_operand` first,
///  then `start.join(operand)`),
///  so the move has the removal's gap and the removal's answer:
///  an operand that is not the end of the registered path is read from the
///  directory Git runs the command in,
///  which for a `!` alias is the top of the working tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // cwd `<root>`, `-C <root>/repository` alias `mvwt ./../one ../moved` => refused
/// ```
#[test]
fn a_bound_slots_move_reads_its_operand_where_git_runs_the_alias() {
    let bound: RegisteredSlot = a_bound_slot_for_a_shell_alias("entry-slot-move-alias-directory");
    let spellings: Vec<Vec<OsString>> = vec![
        started_in(
            bound.repository.as_path(),
            &["-c", ALIAS_MOVES, "mvwt", "../one", "../moved"],
        ),
        started_in(
            bound.repository.as_path(),
            &["-c", ALIAS_MOVES, "mvwt", "./../one", "../moved"],
        ),
        // The direct spelling of the same source from the directory Git runs it in.
        started_in(
            bound.repository.as_path(),
            &["worktree", "move", "../one", "../moved"],
        ),
    ];
    for words in &spellings {
        let (code, stderr): (i32, String) = stopped(plan_invocation(
            words.as_slice(),
            &[],
            &inputs(bound.root.as_path()),
        ));
        assert_eq!(
            code,
            crate::entry_zfs_slot_commands::REFUSAL_STATUS,
            "the move is refused for {words:?}: {stderr}"
        );
        assert!(
            stderr.contains("`git worktree move` is refused"),
            "{words:?}: {stderr}"
        );
        assert!(bound.slot.is_dir(), "{words:?}: nothing was moved");
    }
    remove(bound.root.as_path());
}

/// What:
///  A prune behind a shell alias restores the pointer,
///  because the prune's own rule reads no operand and so no directory.
/// Why:
///  The removal and the move resolve an operand against where Git runs it,
///  and the prune's pass reads only the command word and the common directory
///  (`entry_zfs_slot_commands::before_a_prune`).
///  The reader the three share is the same,
///  so this control shows that fixing the two operand rules changes nothing about
///  the third.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // cwd `<root>`, `-C <root>/repository` alias `prwt` => the pointer stands again before Git runs
/// ```
#[test]
fn a_prune_behind_a_shell_alias_restores_the_pointer() {
    let bound: RegisteredSlot = a_bound_slot_for_a_shell_alias("entry-slot-prune-alias-directory");
    let words: Vec<OsString> =
        started_in(bound.repository.as_path(), &["-c", ALIAS_PRUNES, "prwt"]);
    std::fs::remove_file(bound.slot.join(".git")).expect("the pointer goes");
    let action: Action = plan_invocation(words.as_slice(), &[], &inputs(bound.root.as_path()));
    assert!(
        matches!(action, Action::Forward { .. }),
        "the pass lets the prune run: {action:?}"
    );
    assert_eq!(
        std::fs::read(bound.slot.join(".git")).expect("the pointer the pass wrote back"),
        bound.pointer,
        "the pointer stands again before Git runs"
    );
    remove(bound.root.as_path());
}

/// What:
///  A bound slot's removal behind a chain of aliases one link longer than the
///  reading follows is left to Git,
///  exactly as an alias whose value the wrapper cannot read is,
///  and no refusal is read from the words the reading stopped on.
/// Why:
///  The chain stopped one link before its end,
///  so its words are not the command Git runs;
///  the reading says so (`Route::Unread`),
///  and what the wrapper does with a command it cannot read is the open choice of
///  `doc/planning/cli-git-slot-refusals-and-shell-aliases-they-cannot-read.md`,
///  whose option C is to leave it to Git.
///  This control pins that a shape left unread reaches Git unchanged and gains no
///  refusal of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = aChainOfAliases(MAX_ALIAS_EXPANSIONS + 1, 'worktree remove', ['-f', '-f', 'one']);
/// expect(planInvocation(words)).toEqual({ kind: 'forward' });
/// ```
#[test]
fn a_removal_behind_a_chain_past_the_reading_is_left_to_git() {
    let bound: RegisteredSlot =
        a_bound_slot_for_a_shell_alias("entry-slot-removal-chain-past-the-reading");
    let mut chained: Vec<OsString> = started_in(bound.repository.as_path(), &[]);
    chained.extend(a_chain_of_aliases(
        MAX_ALIAS_EXPANSIONS + 1,
        "worktree remove",
        &["-f", "-f", "one"],
    ));
    let action: Action = plan_invocation(chained.as_slice(), &[], &inputs(bound.root.as_path()));
    assert!(
        matches!(action, Action::Forward { .. }),
        "a chain the reading cannot follow reaches Git unchanged: {action:?}"
    );
    assert!(bound.slot.is_dir(), "the planning removed nothing");
    remove(bound.root.as_path());
}
