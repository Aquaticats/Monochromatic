//! Controls of the one start of a worktree's service:
//! the starter marker,
//! the greeting,
//! and the takeover of a killed starter's marker.

use super::*;

/// A folder of this test's own,
/// short enough for a socket address.
fn folder(name: &str) -> PathBuf {
    return crate::test_support::short_directory_for_sockets(format!("server-start-{name}").as_str());
}

/// What:
/// One taker of the marker wins and a second finds it held.
/// Why:
///  Two commands that both found no service must not both start one.
#[test]
fn one_taker_of_the_marker_wins() {
    let root = folder("marker");
    let marker = root.join("3f9a.starting");
    let first = StarterMarker::take(marker.as_path()).expect("the marker is created").expect("the first command takes the marker");
    assert!(StarterMarker::take(marker.as_path()).expect("the marker is there").is_none(), "the second command finds it held");
    drop(first);
    assert!(!marker.exists(), "the marker goes when the start is over");
    assert!(StarterMarker::take(marker.as_path()).expect("the marker is created").is_some(), "and the next start takes it again");
}

/// What:
/// A marker whose owner is gone is nobody's and is taken over.
/// Why:
///  A command killed between the marker and the greeting must not block every later start.
#[test]
fn a_marker_whose_owner_is_gone_is_taken_over() {
    let root = folder("takeover");
    let marker = root.join("3f9a.starting");
    std::fs::write(marker.as_path(), "1 linux:000000\n").expect("a killed command's marker");
    assert!(!holder_is_alive(marker.as_path()), "the marker names no living process");
    assert!(StarterMarker::take(marker.as_path()).expect("the marker is created").is_some(), "the next start takes it");
}

/// What:
/// Nothing at an address answers,
/// and a service that greets does.
/// Why:
///  The greeting is what tells this service from a file someone left at its address.
#[test]
fn only_a_service_that_greets_answers() {
    let root = folder("greeting");
    let socket = root.join("3f9a.sock");
    assert!(!answered(&Bind::Path(socket.clone())), "nothing at a path that is not there answers");
    let listener = std::os::unix::net::UnixListener::bind(socket.as_path()).expect("a socket");
    assert!(!answered(&Bind::Path(socket.clone())), "a connection nothing answers on is not a greeting");
    let greeter = std::thread::spawn(move || {
        use std::io::Write;
        // Every connection is greeted, because the probes before the start left connections nobody read. The loop ends after
        // a few, so the control's thread ends: a greeter that accepted for ever would hold this test at its join.
        let line: String = format!("{}\n", crate::service_protocol_encode_messages_to_lines::encode_service(&crate::service_protocol::ServiceMessage::Welcome { version: 1, release: Some(String::from("test")) }));
        let mut greeted: usize = 0;
        while greeted < 6 {
            let Ok((mut stream, _)) = listener.accept() else { return };
            let _sent = stream.write_all(line.as_bytes());
            greeted += 1;
            // The connection stays open while the greeting is read.
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    });
    assert!(waits_for_a_greeting(&Bind::Path(socket.clone()), START_WAIT), "a service that greets answers");
    // The greeter's thread ends by itself; a control that waited for it would wait on the socket it still holds.
    std::mem::forget(greeter);
}

/// A wait with no service ends at its bound and does not hang.
#[test]
fn a_wait_with_nothing_at_the_address_ends_at_its_bound() {
    let root = folder("wait");
    let began = std::time::Instant::now();
    assert!(!waits_for_a_greeting(&Bind::Path(root.join("none.sock")), Duration::from_millis(50)));
    assert!(began.elapsed() < Duration::from_secs(5), "the bound is the bound");
}

/// The environment variable that marks the child standing in for a service that is recovering.
const PREPARING_SERVICE_VARIABLE: &str = "CLI_GIT_TEST_ONLY_PREPARING_SERVICE";

/// The note that child claims.
const PREPARING_SERVICE_NOTE_VARIABLE: &str = "CLI_GIT_TEST_ONLY_PREPARING_SERVICE_NOTE";

/// What the child of the manager control is: a service that has claimed its instance and does not greet,
/// as a service recovering the batches an earlier one left open does.
///  It waits longer than `START_WAIT + MANAGER_START_EXTRA`, so only the manager's own deadline reaches it.
#[test]
fn the_preparing_service_a_manager_started() {
    let Some(note) = std::env::var_os(PREPARING_SERVICE_NOTE_VARIABLE) else { return };
    crate::service_server_pid_note_of_an_instance::claim(
        std::path::Path::new(&note),
        i32::try_from(std::process::id()).expect("a process number"),
    )
    .expect("the preparing service claims its instance");
    std::thread::sleep(Duration::from_secs(60));
}

/// What:
///  A start through the manager that sees no greeting looks for a live service that is preparing,
///  and never ends its group.
/// Why:
///  The sixth review of issue 6293 (finding 3):
///  the manager arm killed the whole process group at `START_WAIT + MANAGER_START_EXTRA` without looking at the note,
///  so a service recovering the batches an earlier service left open was killed at an arbitrary instruction
///  and the base start made a second one recover the same batch again.
/// The service here is the test binary itself, started by a stand-in manager, claiming the note and never greeting.
#[test]
fn a_manager_start_that_sees_no_greeting_keeps_a_live_service_that_is_preparing() {
    let root: PathBuf = folder("manager-preparing");
    let note: PathBuf = root.join("3f9a.pid");
    let binary: PathBuf = crate::test_support::this_test_binary().expect("the test binary");
    // The stand-in manager answers the probe and, for a start, runs the preparing service.
    let program: PathBuf = root.join("systemd-run");
    std::fs::write(
        program.as_path(),
        format!(
            "#!/bin/sh\ncase \"$*\" in *'-- /bin/true'*) exec /bin/true ;; esac\n\
             export {variable}=1 {note_variable}='{note}'\nexec '{binary}' 'service::service_server_start_by_the_wrapper::tests::the_preparing_service_a_manager_started' --exact --nocapture\n",
            variable = PREPARING_SERVICE_VARIABLE,
            note_variable = PREPARING_SERVICE_NOTE_VARIABLE,
            note = note.display(),
            binary = binary.display()
        ),
    )
    .expect("the stand-in manager");
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(program.as_path(), std::fs::Permissions::from_mode(0o700)).expect("it runs");
    }
    let server: ServerToStart = ServerToStart {
        address: Bind::Path(root.join("3f9a.sock")),
        directory: root.clone(),
        instance: String::from("3f9a"),
        journal: root.join("journal"),
        git: PathBuf::from("/usr/bin/git"),
        program: binary.clone(),
        manager: Some(crate::service_server_start_through_the_manager::ManagerStart { program: program.clone() }),
    };
    let began: std::time::Instant = std::time::Instant::now();
    let attempt: StartAttempt = start(&server);
    let waited: Duration = began.elapsed();
    match attempt {
        StartAttempt::ServiceIsPreparing { service } => {
            assert_ne!(service.pid, i32::try_from(std::process::id()).expect("a process number"), "the live service is the child that claimed the note, not this command");
            assert!(crate::service_server_pid_note_of_an_instance::is_still_the_process(&service), "and the process the note names is still alive");
        }
        other => panic!("a live service that has not greeted is waited for, not ended: {other:?} after {waited:?}"),
    }
    let held: crate::service_server_pid_note_of_an_instance::ServerProcess =
        crate::service_server_pid_note_of_an_instance::read(note.as_path()).expect("the preparing service's note is still there");
    assert!(crate::service_server_pid_note_of_an_instance::is_still_the_process(&held), "and the process it names is still alive");
    // The stand-in service is this test's own child, and it ends with this control.
    let _ended = std::process::Command::new("/usr/bin/pkill").args(["-9", "-f", PREPARING_SERVICE_VARIABLE]).status();
}

/// What:
///  The stand-in service of the claim-after-the-look controls:
///  a shell process that says which process it is and then waits, granting no greeting,
///  as a service in its recovery does.
/// Why:
///   It is a real second process, and the note can name it by the process number it wrote for itself.
fn silent_service_script(root: &Path, runs: &Path) -> PathBuf {
    let script: PathBuf = root.join("silent-service");
    std::fs::write(
        script.as_path(),
        format!("#!/bin/sh\nprintf '%s' \"$$\" > '{runs}/'\"$$\"\nexec /bin/sleep 120\n", runs = runs.display()),
    )
    .expect("the stand-in service");
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(script.as_path(), std::fs::Permissions::from_mode(0o700)).expect("it runs");
    }
    return script;
}

/// The number of the last silent stand-in service that said it runs,
/// once one has.
fn the_silent_service_pid(runs: &Path) -> Option<i32> {
    let mut pids: Vec<i32> = std::fs::read_dir(runs)
        .map(|listing| {
            return listing.flatten().filter_map(|entry| { return entry.file_name().to_string_lossy().parse::<i32>().ok(); }).collect::<Vec<i32>>();
        })
        .unwrap_or_default();
    pids.sort();
    return pids.last().copied();
}

/// Waits, bounded, until the silent stand-in service has said which process it is.
fn wait_for_the_silent_service(runs: &Path) -> i32 {
    let began: std::time::Instant = std::time::Instant::now();
    while began.elapsed() < Duration::from_secs(30) {
        if let Some(pid) = the_silent_service_pid(runs) {
            return pid;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("the stand-in service never said which process it is");
}

/// What:
///  Writes the note a service writes when it claims an instance,
///  for a process this control started.
/// Why:
///  A claim names the claiming process and its birth identity,
///  and a control that holds the instant between a look and an act has to produce exactly that file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claimFor(pid) // write `${pid} ${birthIdentityOf(pid)}` into the note
/// ```
fn claim_for(note: &Path, pid: i32) {
    let identity: String = crate::process_identity::process_birth_identity(i64::from(pid))
        .expect("the process's record is readable")
        .expect("the process this control started is alive");
    std::fs::write(note, format!("{pid} {identity}\n")).expect("the note a claiming service writes");
}

/// What:
///  A manager-started service that claims its note at the instant after the start looked
///  is waited for, and neither ended nor answered as a failure.
/// Why:
///  The sixth review of issue 6293 (finding 1):
///  the look and the group it permits to be ended were not one step,
/// so a service claiming between them was killed at an arbitrary instruction.
#[test]
fn a_manager_start_that_meets_a_claim_after_its_look_keeps_the_service() {
    let root: PathBuf = folder("manager-claim-after-the-look");
    let note: PathBuf = root.join("3f9a.pid");
    let runs: PathBuf = root.join("runs");
    std::fs::create_dir_all(runs.as_path()).expect("run folder");
    let service: PathBuf = silent_service_script(root.as_path(), runs.as_path());
    let manager_program: PathBuf = root.join("systemd-run");
    std::fs::write(
        manager_program.as_path(),
        format!("#!/bin/sh\ncase \"$*\" in *'-- /bin/true'*) exec /bin/true ;; esac\nexec '{service}'\n", service = service.display()),
    )
    .expect("the stand-in manager");
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(manager_program.as_path(), std::fs::Permissions::from_mode(0o700)).expect("it runs");
    }
    let server: ServerToStart = ServerToStart {
        address: Bind::Path(root.join("3f9a.sock")),
        directory: root.clone(),
        instance: String::from("3f9a"),
        journal: root.join("journal"),
        git: PathBuf::from("/usr/bin/git"),
        program: service,
        manager: Some(crate::service_server_start_through_the_manager::ManagerStart { program: manager_program }),
    };
    let awaited: PathBuf = runs.clone();
    let claimed: PathBuf = note.clone();
    crate::service_server_start_by_the_wrapper::a_control_runs_after_the_note_was_read(move || {
        if let Some(pid) = the_silent_service_pid(awaited.as_path()) {
            claim_for(claimed.as_path(), pid);
        }
    });
    let attempt: StartAttempt = start(&server);
    let pid: i32 = wait_for_the_silent_service(runs.as_path());
    let alive: bool = crate::process_identity::process_birth_identity(i64::from(pid)).expect("the process's record").is_some();
    let _killed = std::process::Command::new("/bin/kill").args(["-9", format!("{pid}").as_str()]).status();
    let preparing: bool = match &attempt {
        StartAttempt::ServiceIsPreparing { service } => service.pid == pid,
        _other => false,
    };
    assert!(preparing, "a service that claimed after the look is waited for, not ended: {attempt:?}");
    assert!(alive, "and the process the note names is still alive after the start answered");
}

/// What:
///  A base start that meets a claim at the instant before it gives up is not a failure.
/// Why:
///  The sixth review of issue 6293 (finding 2):
///  a command failed with "the service did not start" while the service it started was alive and preparing.
#[test]
fn a_base_start_that_meets_a_claim_after_its_look_is_not_a_failure() {
    let root: PathBuf = folder("base-claim-after-the-look");
    let note: PathBuf = root.join("3f9a.pid");
    let runs: PathBuf = root.join("runs");
    std::fs::create_dir_all(runs.as_path()).expect("run folder");
    let service: PathBuf = silent_service_script(root.as_path(), runs.as_path());
    let server: ServerToStart = ServerToStart {
        address: Bind::Path(root.join("3f9a.sock")),
        directory: root.clone(),
        instance: String::from("3f9a"),
        journal: root.join("journal"),
        git: PathBuf::from("/usr/bin/git"),
        program: service,
        manager: None,
    };
    let awaited: PathBuf = runs.clone();
    let claimed: PathBuf = note.clone();
    crate::service_server_start_by_the_wrapper::a_control_runs_after_the_note_was_read(move || {
        if let Some(pid) = the_silent_service_pid(awaited.as_path()) {
            claim_for(claimed.as_path(), pid);
        }
    });
    let attempt: StartAttempt = start(&server);
    let pid: i32 = wait_for_the_silent_service(runs.as_path());
    let _killed = std::process::Command::new("/bin/kill").args(["-9", format!("{pid}").as_str()]).status();
    match attempt {
        StartAttempt::ServiceIsPreparing { service } => assert_eq!(service.pid, pid, "the service that claimed is the one preparing"),
        other => panic!("a service that claimed after the look is preparing, not a failure: {other:?}"),
    }
}
