//! Controls for `service stop-units-of-removed-worktrees` and the note `doctor` adds about it:
//! what each answer says,
//! and that a worktree that is there,
//! or an instance that maps to nothing,
//! is never stopped.

use super::*;
use crate::path_bytes_in_records::GITDIR_HEX_PREFIX;
use crate::process_wide_state_child_test_support::run_the_body_with_an_environment_of_its_own;
use crate::service_instance::{
    SOCKET_DIRECTORY,
    instance_of,
    service_unit_instance,
    socket_path,
    socket_unit_instance,
};
use crate::service_unit_manager_recorder_test_support::Recorder;
use crate::service_units_of_removed_worktrees::{
    note_path,
    record_worktree,
};
use crate::session_runtime_directory_and_bus_address::{
    BUS_ADDRESS_SWITCH,
    RUNTIME_DIRECTORY_SWITCH,
};
use crate::systemd_dbus::CallError;
use crate::test_support::{
    fixture,
    remove,
};

/// The command word the notes name.
const NAMED_COMMAND: &str =
    "git monochromatic-git-policy-cli service stop-units-of-removed-worktrees";

/// One disposable world:
/// a runtime directory with a socket and a note for a present worktree,
/// a removed worktree,
/// and an instance that maps to nothing.
struct World {
    root: PathBuf,
    runtime: PathBuf,
    gone_instance: String,
    kept_instance: String,
    unknown_instance: String,
}

fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let runtime: PathBuf = root.join("run");
    std::fs::create_dir_all(runtime.join(SOCKET_DIRECTORY)).expect("socket directory");
    let common: PathBuf = root.join("repository/.git");
    let kept: PathBuf = common.join("worktrees/kept");
    std::fs::create_dir_all(&kept).expect("kept");
    let gone: PathBuf = common.join("worktrees/gone");
    let unknown_instance: String = String::from("00000000000000000000000000000001");
    let world: World = World {
        gone_instance: instance_of(gone.as_path()),
        kept_instance: instance_of(kept.as_path()),
        unknown_instance,
        root,
        runtime,
    };
    for (instance, git_dir) in [(&world.kept_instance, &kept), (&world.gone_instance, &gone)] {
        record_worktree(
            world.runtime.as_path(),
            instance.as_str(),
            git_dir.as_path(),
        )
        .expect("note");
        std::fs::write(
            socket_path(world.runtime.as_path(), instance.as_str()),
            b"listening",
        )
        .expect("socket");
    }
    std::fs::write(
        socket_path(world.runtime.as_path(), world.unknown_instance.as_str()),
        b"listening",
    )
    .expect("socket");
    return world;
}

impl World {
    fn folders(&self) -> UnitFolders<'_> {
        return UnitFolders {
            runtime_dir: self.runtime.as_path(),
            state_dir: None,
        };
    }

    fn environment(&self) -> Vec<(OsString, OsString)> {
        return vec![(
            OsString::from("XDG_RUNTIME_DIR"),
            self.runtime.clone().into_os_string(),
        )];
    }
}

fn exit_parts(action: Action) -> (i32, String, String) {
    let Action::Exit {
        code,
        stdout,
        stderr,
    } = action
    else {
        panic!("the command exits")
    };
    return (code, stdout, stderr);
}

/// The command stops the removed worktree's two units and names the worktree,
/// and says that it left the unknown instance running.
#[test]
fn the_command_stops_the_units_of_the_removed_worktree_and_names_it() {
    let world: World = world("stop-units-command");
    let mut manager: Recorder = Recorder::default();
    let (code, stdout, stderr) = exit_parts(stop_units_action(&world.folders(), &mut manager));
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        manager.stops(),
        vec![
            format!(
                "stop {}",
                socket_unit_instance(world.gone_instance.as_str())
            ),
            format!(
                "stop {}",
                service_unit_instance(world.gone_instance.as_str())
            )
        ]
    );
    assert!(
        stdout.contains("repository/.git/worktrees/gone")
            && stdout.contains(world.gone_instance.as_str()),
        "{stdout}"
    );
    assert!(
        stderr.contains("1 commit service instance cannot be matched")
            && stderr.contains(NAMED_COMMAND),
        "{stderr}"
    );
    assert!(!stdout.contains(world.kept_instance.as_str()));
    remove(world.root.as_path());
}

/// What:
///  A journal that holds a commit no branch shows is kept and named on standard error,
///  with where it is and what kept it,
///  and the command still ends with status 0 because the units it was asked about were stopped.
/// Why:
///   The person needs that line even when warnings are turned off,
/// and it must not read as a refusal:
/// nothing here failed.
#[test]
fn a_kept_journal_is_named_on_standard_error_and_is_no_refusal() {
    let world: World = world("stop-units-kept-journal");
    let state: PathBuf = world.root.join("state");
    let wrapper_state: PathBuf = state.join(crate::wrapper_names_old_and_new::SERVICE_UNIT.current);
    let journal: PathBuf = crate::service_journal_folder_of_an_instance::journal_of(
        wrapper_state.as_path(),
        world.gone_instance.as_str(),
    );
    std::fs::create_dir_all(&journal).expect("journal folder");
    std::fs::write(journal.join("7.intent"), b"seq=7\n").expect("a batch that did not land");
    let folders: UnitFolders<'_> = UnitFolders {
        runtime_dir: world.runtime.as_path(),
        state_dir: Some(state.as_path()),
    };
    let mut manager: Recorder = Recorder::default();
    let (code, _stdout, stderr) = exit_parts(stop_units_action(&folders, &mut manager));
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stderr.contains(journal.display().to_string().as_str()),
        "the line names the journal: {stderr}"
    );
    assert!(
        stderr.contains("7.intent") && stderr.contains("has not landed"),
        "the line names what kept it: {stderr}"
    );
    assert!(journal.exists(), "the journal is kept");
    remove(world.root.as_path());
}

/// What:
///  The command clears a journal of a removed worktree that no note or socket file names,
///  says where it was,
///  and asks the manager to stop no unit that does not exist.
/// Why:
///   A removal that kept a journal took the note and the socket file with it,
/// so the journals of earlier removals are reachable only through the journal itself;
/// the person ran the command to clear them and learns from its own line where each one was.
#[test]
fn the_command_clears_a_kept_journal_and_names_where_it_was() {
    let world: World = world("stop-units-journal-only");
    let state: PathBuf = world.root.join("state");
    let wrapper_state: PathBuf = state.join(crate::wrapper_names_old_and_new::SERVICE_UNIT.current);
    let journal: PathBuf = crate::service_journal_folder_of_an_instance::journal_of(
        wrapper_state.as_path(),
        world.gone_instance.as_str(),
    );
    std::fs::create_dir_all(&journal).expect("journal folder");
    let gone_admin: PathBuf = world.root.join("repository/.git/worktrees/gone");
    std::fs::write(
        journal.join("1.done"),
        format!(
            "seq=1\n{}{}\n",
            GITDIR_HEX_PREFIX,
            crate::path_bytes_in_records::hex_of_path(gone_admin.as_path())
        ),
    )
    .expect("a landed batch that names the Git directory");
    // The note and the socket file a removal that kept this journal took with it.
    std::fs::remove_file(note_path(
        world.runtime.as_path(),
        world.gone_instance.as_str(),
    ))
    .expect("note removed");
    std::fs::remove_file(socket_path(
        world.runtime.as_path(),
        world.gone_instance.as_str(),
    ))
    .expect("socket removed");
    crate::service_units_fixture_test_support::age_a_journal(journal.as_path());
    let folders: UnitFolders<'_> = UnitFolders {
        runtime_dir: world.runtime.as_path(),
        state_dir: Some(state.as_path()),
    };
    let mut manager: Recorder = Recorder::default();
    let (code, stdout, stderr) = exit_parts(stop_units_action(&folders, &mut manager));
    assert_eq!(code, 0, "{stderr}");
    assert!(
        manager.stops().is_empty(),
        "no unit of that instance exists to stop: {:?}",
        manager.stops()
    );
    assert!(
        stdout.contains("removed the commit service journal of the removed worktree")
            && stdout.contains(journal.display().to_string().as_str()),
        "the line names what was removed and where it was: {stdout}"
    );
    assert!(
        stdout.contains(world.gone_instance.as_str()),
        "the line names the instance: {stdout}"
    );
    assert!(
        !journal.exists(),
        "the journal of a removed worktree that holds no work is cleared"
    );
    remove(world.root.as_path());
}

/// A manager that refuses a stop makes status 1 with the instance and the reason,
/// and the command can be run again.
#[test]
fn a_refused_stop_is_status_one_with_the_instance_and_the_reason() {
    let world: World = world("stop-units-refused");
    let mut manager: Recorder = Recorder::default();
    manager.refuse_stop_of(
        socket_unit_instance(world.gone_instance.as_str()).as_str(),
        CallError::TimedOut,
    );
    let (code, _stdout, stderr) = exit_parts(stop_units_action(&world.folders(), &mut manager));
    assert_eq!(code, 1);
    assert!(
        stderr.contains(world.gone_instance.as_str())
            && stderr.contains("did not answer in time")
            && stderr.contains("run the command again"),
        "{stderr}"
    );
    remove(world.root.as_path());
}

/// With no gone worktree the command says so and asks the manager nothing.
#[test]
fn nothing_to_stop_is_said_and_asks_the_manager_nothing() {
    let world: World = world("stop-units-nothing");
    std::fs::remove_file(socket_path(
        world.runtime.as_path(),
        world.gone_instance.as_str(),
    ))
    .expect("socket removed");
    std::fs::remove_file(crate::service_units_of_removed_worktrees::note_path(
        world.runtime.as_path(),
        world.gone_instance.as_str(),
    ))
    .expect("note removed");
    let mut manager: Recorder = Recorder::default();
    let (code, stdout, _stderr) = exit_parts(stop_units_action(&world.folders(), &mut manager));
    assert_eq!(code, 0);
    assert!(
        stdout.contains("no commit service unit or journal belongs to a removed worktree"),
        "{stdout}"
    );
    assert!(manager.stops().is_empty());
    remove(world.root.as_path());
}

/// `doctor` counts the units of removed worktrees without changing a file,
/// and says nothing when none is left.
#[test]
fn doctor_counts_the_units_of_removed_worktrees_and_names_the_command() {
    let world: World = world("stop-units-note");
    let note: String = units_note(world.environment().as_slice());
    assert!(
        note.contains("1 removed worktree are still set up")
            && note.contains(NAMED_COMMAND)
            && note.contains("nothing was changed"),
        "{note}"
    );
    assert!(
        socket_path(world.runtime.as_path(), world.gone_instance.as_str()).exists(),
        "the note is read only"
    );
    std::fs::remove_file(socket_path(
        world.runtime.as_path(),
        world.gone_instance.as_str(),
    ))
    .expect("socket removed");
    std::fs::remove_file(crate::service_units_of_removed_worktrees::note_path(
        world.runtime.as_path(),
        world.gone_instance.as_str(),
    ))
    .expect("note removed");
    std::fs::remove_file(socket_path(
        world.runtime.as_path(),
        world.unknown_instance.as_str(),
    ))
    .expect("socket removed");
    assert_eq!(units_note(world.environment().as_slice()), "");
    remove(world.root.as_path());
}

/// Without a user runtime directory the note is empty and the command refuses with the reason.
#[test]
fn without_a_runtime_directory_the_note_is_empty_and_the_command_refuses() {
    assert_eq!(units_note(&[]), "");
    let (code, _stdout, stderr) = exit_parts(plan_stop_units_of_removed_worktrees(&[]));
    assert_eq!(code, 1);
    assert!(
        stderr.contains("XDG_RUNTIME_DIR is not set") && stderr.contains(NAMED_COMMAND),
        "{stderr}"
    );
}

/// What:
///  `doctor`'s note counts the journal of a removed worktree that no note and no socket file names,
///  read from the fixture's own home,
/// runtime and state directories,
///  and says nothing of this machine's state folder.
/// Why:
///   That count is what tells a person the command has something to clear,
/// and a journal that no unit names was invisible to it before (issue 6322).
/// The fixture names its own `HOME`,
/// `XDG_RUNTIME_DIR` and `XDG_STATE_HOME`,
/// which is what makes the reading touch nothing of this machine.
#[test]
fn doctor_counts_a_journal_of_a_removed_worktree_that_no_note_names() {
    let world: World = world("stop-units-note-journal-only");
    let state: PathBuf = world.root.join("state");
    let wrapper_state: PathBuf = state.join(crate::wrapper_names_old_and_new::SERVICE_UNIT.current);
    let journal: PathBuf = crate::service_journal_folder_of_an_instance::journal_of(
        wrapper_state.as_path(),
        world.gone_instance.as_str(),
    );
    std::fs::create_dir_all(&journal).expect("journal folder");
    let gone_admin: PathBuf = world.root.join("repository/.git/worktrees/gone");
    std::fs::write(
        journal.join("1.done"),
        format!(
            "seq=1\n{}{}\n",
            GITDIR_HEX_PREFIX,
            crate::path_bytes_in_records::hex_of_path(gone_admin.as_path())
        ),
    )
    .expect("a landed batch that names the Git directory");
    // The note and the socket file a removal that kept this journal took with it.
    std::fs::remove_file(note_path(
        world.runtime.as_path(),
        world.gone_instance.as_str(),
    ))
    .expect("note removed");
    std::fs::remove_file(socket_path(
        world.runtime.as_path(),
        world.gone_instance.as_str(),
    ))
    .expect("socket removed");
    let home: PathBuf = world.root.join("home");
    std::fs::create_dir_all(&home).expect("home directory");
    let environment: Vec<(OsString, OsString)> = vec![
        (OsString::from("HOME"), home.clone().into_os_string()),
        (
            OsString::from("XDG_RUNTIME_DIR"),
            world.runtime.clone().into_os_string(),
        ),
        (
            OsString::from("XDG_STATE_HOME"),
            state.clone().into_os_string(),
        ),
    ];
    let note: String = units_note(environment.as_slice());
    assert!(
        note.contains("1 removed worktree are still set up"),
        "the journal that no unit names is counted: {note}"
    );
    assert!(
        note.contains(NAMED_COMMAND),
        "the note names the command that clears it: {note}"
    );
    remove(world.root.as_path());
}

/// What:
///  A removed worktree whose note and socket file are gone and whose journal holds work
///  makes standard output name the journal it kept,
///  and the line that says no unit or journal belongs to a removed worktree is not printed beside it.
/// Why:
///   M2 of the alpha.46 commit,
/// push and LFS release pass measured one run that said on standard output
///  "no commit service unit or journal belongs to a removed worktree"
///  while standard error named the journal it had just kept for a removed worktree.
///  The two lines contradict each other,
///  and standard output is the half a person reads to learn what the command found.
#[test]
fn a_kept_journal_is_named_on_standard_output_and_the_none_line_is_not_printed() {
    let world: World = world("stop-units-kept-journal-stdout");
    let state: PathBuf = world.root.join("state");
    let wrapper_state: PathBuf = state.join(crate::wrapper_names_old_and_new::SERVICE_UNIT.current);
    let journal: PathBuf = crate::service_journal_folder_of_an_instance::journal_of(
        wrapper_state.as_path(),
        world.gone_instance.as_str(),
    );
    std::fs::create_dir_all(&journal).expect("journal folder");
    let gone_admin: PathBuf = world.root.join("repository/.git/worktrees/gone");
    std::fs::write(
        journal.join("1.done"),
        format!(
            "seq=1\n{}{}\n",
            GITDIR_HEX_PREFIX,
            crate::path_bytes_in_records::hex_of_path(gone_admin.as_path())
        ),
    )
    .expect("a landed batch that names the Git directory");
    std::fs::write(journal.join("7.intent"), b"seq=7\n").expect("a batch that did not land");
    // The note and the socket file a removal that kept this journal took with it.
    std::fs::remove_file(note_path(
        world.runtime.as_path(),
        world.gone_instance.as_str(),
    ))
    .expect("note removed");
    std::fs::remove_file(socket_path(
        world.runtime.as_path(),
        world.gone_instance.as_str(),
    ))
    .expect("socket removed");
    crate::service_units_fixture_test_support::age_a_journal(journal.as_path());
    let folders: UnitFolders<'_> = UnitFolders {
        runtime_dir: world.runtime.as_path(),
        state_dir: Some(state.as_path()),
    };
    let mut manager: Recorder = Recorder::default();
    let (code, stdout, stderr) = exit_parts(stop_units_action(&folders, &mut manager));
    assert_eq!(code, 0, "{stderr}");
    assert!(
        !stdout.contains("no commit service unit or journal belongs to a removed worktree"),
        "a journal was kept for a removed worktree, so that line would contradict standard error: {stdout}"
    );
    assert!(
        stdout.contains("kept the commit service journal of the removed worktree")
            && stdout.contains("repository/.git/worktrees/gone")
            && stdout.contains(world.gone_instance.as_str())
            && stdout.contains(journal.display().to_string().as_str()),
        "standard output names the journal it kept, whose it was and where it is: {stdout}"
    );
    assert!(
        stderr.contains("7.intent"),
        "standard error still says what kept it: {stderr}"
    );
    assert!(journal.exists(), "the journal is kept");
    remove(world.root.as_path());
}

/// What:
///  A journal that names no worktree and that no socket file and no note stand beside
///  is not counted as an instance that cannot be matched:
///  the command says a person can look at it and remove it by hand.
/// Why:
///   M1 of the alpha.46 commit,
/// push and LFS release pass measured seven such journals on a real machine,
///  every one holding nothing but `busy` and an empty `claim-watch`.
///  No unit can exist for them (a socket file or a note is what makes one possible),
///  and the line that counted them said nothing a person could do,
///  so every `doctor` named a number that no command could lower.
#[test]
fn a_journal_that_names_no_worktree_and_has_no_unit_is_not_said_to_be_unmatchable() {
    let world: World = world("stop-units-journal-without-units");
    let state: PathBuf = world.root.join("state");
    let wrapper_state: PathBuf = state.join(crate::wrapper_names_old_and_new::SERVICE_UNIT.current);
    let instance: String = String::from("0000000000000000000000000000000e");
    let journal: PathBuf = crate::service_journal_folder_of_an_instance::journal_of(
        wrapper_state.as_path(),
        instance.as_str(),
    );
    std::fs::create_dir_all(journal.join("claim-watch").as_path())
        .expect("journal with an empty claim watch");
    std::fs::write(journal.join("busy"), b"1").expect("the service's record that it is running");
    let folders: UnitFolders<'_> = UnitFolders {
        runtime_dir: world.runtime.as_path(),
        state_dir: Some(state.as_path()),
    };
    let mut manager: Recorder = Recorder::default();
    let (code, _stdout, stderr) = exit_parts(stop_units_action(&folders, &mut manager));
    assert_eq!(code, 0, "{stderr}");
    assert!(
        !stderr.contains("2 commit service instances cannot be matched"),
        "the journal with no unit beside it is not counted with the socket file's instance: {stderr}"
    );
    assert!(
        stderr.contains("1 commit service instance cannot be matched"),
        "only the instance a socket file stands beside is counted as unmatchable: {stderr}"
    );
    assert!(
        stderr.contains("1 commit service journal names no worktree"),
        "the line is about the journal that has no unit: {stderr}"
    );
    assert!(
        stderr.contains("by hand"),
        "the line says a person can remove it after looking: {stderr}"
    );
    let journals_folder: String = state
        .join(crate::wrapper_names_old_and_new::SERVICE_UNIT.current)
        .join(crate::service_journal_folder_of_an_instance::JOURNAL_FOLDER)
        .display()
        .to_string();
    assert!(
        stderr.contains(journals_folder.as_str()),
        "the line says where to look: {stderr}"
    );
    assert!(journal.exists(), "nothing of it is touched");
    // `doctor` says the same about the same journal.
    let home: PathBuf = world.root.join("home");
    std::fs::create_dir_all(home.as_path()).expect("home directory");
    let environment: Vec<(OsString, OsString)> = vec![
        (OsString::from("HOME"), home.into_os_string()),
        (
            OsString::from("XDG_RUNTIME_DIR"),
            world.runtime.clone().into_os_string(),
        ),
        (
            OsString::from("XDG_STATE_HOME"),
            state.clone().into_os_string(),
        ),
    ];
    let note: String = units_note(environment.as_slice());
    assert!(
        !note.contains("2 commit service instances cannot be matched"),
        "the note does not count it with the socket file's instance: {note}"
    );
    assert!(
        note.contains("1 commit service journal names no worktree and has no unit beside it"),
        "the note is about the journal that has no unit: {note}"
    );
    assert!(
        note.contains("by hand after looking"),
        "the note says what a person can do: {note}"
    );
    remove(world.root.as_path());
}

/// What:
///  A build made for tests takes no route to this account's manager from the environment the entry hands in:
///  the command is handed this process's whole environment,
///  with a runtime directory and a bus address a login session would name,
///  and refuses because it has no session place of its own.
/// Why:
///  The two places this command acts under come through
///  `foundation/session_runtime_directory_and_bus_address.rs`,
///  which a build made for tests answers only from what a control handed in (issue 6586),
///  so even the environment a real session hands in names no manager here
///  (the alpha.49 behaviour read's Low 3 asked what such a control would reach).
///  The body runs in a child with both switches cleared,
///  because the switches belong to the whole process,
///  and it adds the two places to the inherited list itself:
///  a control that read only the run's own environment would prove nothing in a run that has no login session.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const environment = [...process.env, ['XDG_RUNTIME_DIR', fixture], ['DBUS_SESSION_BUS_ADDRESS', `unix:path=${fixture}/bus`]];
/// expect(planStopUnitsOfRemovedWorktrees(environment)).toEqual({ code: 1, stderr: /XDG_RUNTIME_DIR is not set/ });
/// ```
#[test]
fn the_environment_the_entry_hands_in_names_no_manager() {
    run_the_body_with_an_environment_of_its_own(
        module_path!(),
        "the_environment_the_entry_hands_in_names_no_manager",
        &[],
        &[RUNTIME_DIRECTORY_SWITCH, BUS_ADDRESS_SWITCH],
        || {
            // What: the places a login session names, which this body names itself so the arm holds in a run that has none.
            // Why:  the command must refuse them because they came through the environment and not through the one file.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const runtime = fixture('stop-units-handed-in-session'); writeFileSync(join(runtime, 'bus'), '');
            // ```
            let runtime: PathBuf = fixture("stop-units-handed-in-session");
            std::fs::create_dir_all(runtime.as_path()).expect("the runtime directory");
            std::fs::write(runtime.join("bus"), b"").expect("the stand-in bus");
            // What: this process's whole environment, without the two switches, and with the two places a session names.
            // Why:  the switches are what a build made for tests reads, so an arm that left them set would prove nothing.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const environment = [...process.env].filter(([name]) => !name.startsWith('CLI_GIT_TEST_ONLY_SESSION_'));
            // environment.push(['XDG_RUNTIME_DIR', runtime], ['DBUS_SESSION_BUS_ADDRESS', `unix:path=${runtime}/bus`]);
            // ```
            let mut environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
            environment.retain(|(name, _value)| {
                return name != RUNTIME_DIRECTORY_SWITCH
                    && name != BUS_ADDRESS_SWITCH
                    && name != "XDG_RUNTIME_DIR"
                    && name != "DBUS_SESSION_BUS_ADDRESS";
            });
            environment.push((
                OsString::from("XDG_RUNTIME_DIR"),
                runtime.clone().into_os_string(),
            ));
            environment.push((
                OsString::from("DBUS_SESSION_BUS_ADDRESS"),
                OsString::from(format!("unix:path={}", runtime.join("bus").display())),
            ));
            let (code, _stdout, stderr) =
                exit_parts(plan_stop_units_of_removed_worktrees(environment.as_slice()));
            assert_eq!(code, 1, "{stderr}");
            assert!(
                stderr.contains("XDG_RUNTIME_DIR is not set") && stderr.contains(NAMED_COMMAND),
                "the environment the entry hands in named a runtime directory and a bus, and this build found \
                 neither of them, because the two places come through the one file: {stderr}"
            );
            remove(runtime.as_path());
        },
    );
}

/// What:
///  The same command takes both of its places from what a control hands in through the one file:
///  with the two switches naming a runtime directory and a bus of the fixture,
///  it reaches the bus and refuses because nothing answers there.
/// Why:
///  A control that only saw the refusal above could not tell a command reading the one file from a command that
///  read no place at all,
///  so this arm shows the probe able to see a route.
///  The bus path names a plain file,
///  which is no socket:
///  the command asks to connect and learns that nothing answers,
///  and no manager of this account is reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// planStopUnitsOfRemovedWorktrees([]) // with the two switches set: { code: 1, stderr: /could not be reached/ }
/// ```
#[test]
fn the_command_takes_its_places_from_the_one_file() {
    let runtime: PathBuf = fixture("stop-units-switched-session");
    std::fs::create_dir_all(runtime.as_path()).expect("the runtime directory");
    let bus: PathBuf = runtime.join("bus");
    std::fs::write(bus.as_path(), b"").expect("the stand-in bus");
    let address: OsString = OsString::from(format!("unix:path={}", bus.display()));
    let named: [(&str, &std::ffi::OsStr); 2] = [
        (RUNTIME_DIRECTORY_SWITCH, runtime.as_os_str()),
        (BUS_ADDRESS_SWITCH, address.as_os_str()),
    ];
    run_the_body_with_an_environment_of_its_own(
        module_path!(),
        "the_command_takes_its_places_from_the_one_file",
        named.as_slice(),
        &["XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS"],
        || {
            let (code, _stdout, stderr) = exit_parts(plan_stop_units_of_removed_worktrees(&[]));
            assert_eq!(code, 1, "{stderr}");
            assert!(
                stderr.contains("could not be reached") && stderr.contains(NAMED_COMMAND),
                "the runtime directory the control handed in holds a bus path and the command dialled it, \
                 because the one file is what answers the two places: {stderr}"
            );
        },
    );
    remove(runtime.as_path());
}
