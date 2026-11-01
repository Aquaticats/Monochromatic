//! What the service guarantees about its copy writer: one instance per socket, one instance per unit, and a client that
//!  queues behind a running copy is told so at once.

use super::*;
use crate::service_provision::{SERVICE_UNIT, SERVICE_UNIT_NAME, SOCKET_UNIT, SOCKET_UNIT_NAME};
use crate::service_protocol::encode_client;

/// A short directory under the agent scratch directory, since socket paths hold about a hundred bytes.
fn short_directory(label: &str) -> std::path::PathBuf {
    let home: std::path::PathBuf = std::path::PathBuf::from(std::env::var_os("HOME").expect("a home directory"));
    let directory: std::path::PathBuf = home.join("temp/agent").join(format!("sv-{}-{label}", std::process::id()));
    std::fs::create_dir_all(directory.as_path()).expect("directory");
    return directory;
}

/// Two services cannot listen on one address, so a service started by hand on the socket a managed one serves does not run
/// beside it: the second one's bind is refused, for a path and for an abstract name.
#[test]
fn two_services_cannot_listen_on_one_address() {
    let directory: std::path::PathBuf = short_directory("bind-twice");
    let path: Bind = Bind::Path(directory.join("s"));
    let first = bind(&path).expect("the first service binds");
    let refused = bind(&path).expect_err("the second service binds to the same path");
    assert_eq!(refused.kind(), std::io::ErrorKind::AddrInUse);
    let name: Bind = Bind::Abstract(format!("cli-git-test-{}", std::process::id()));
    let held = bind(&name).expect("the first service binds");
    let refused_name = bind(&name).expect_err("the second service binds to the same name");
    assert_eq!(refused_name.kind(), std::io::ErrorKind::AddrInUse);
    drop((first, held));
    let _removed = std::fs::remove_dir_all(directory.as_path());
}

/// The shipped units describe one service instance: the socket unit is not an accepting one and the unit names are not
/// templates, so the manager starts the service once and hands it every connection; its restarts follow its exits.
#[test]
fn the_shipped_units_run_one_service_instance() {
    for (name, text) in [(SOCKET_UNIT_NAME, SOCKET_UNIT), (SERVICE_UNIT_NAME, SERVICE_UNIT)] {
        assert!(!name.contains('@'), "{name} is a template unit");
        for line in text.lines().map(str::trim) {
            assert!(!line.eq_ignore_ascii_case("Accept=yes") && !line.eq_ignore_ascii_case("Accept=true"), "{name}: {line}");
        }
    }
    assert!(SOCKET_UNIT.contains("ListenStream="), "the socket unit has no stream to listen on");
    assert!(!SERVICE_UNIT.contains("Type=oneshot"), "the service unit runs once and exits");
}

/// The Copy message of a client, on a connected pair, and the channel the service queues it on.
fn copy_message(copying: usize, ids: Vec<String>) -> (Vec<String>, bool) {
    let (client, service) = UnixStream::pair().expect("pair");
    let mut line: String = encode_client(&ClientMessage::Copy { common_dir: std::path::PathBuf::from("/repository/.git"), ids });
    line.push('\n');
    (&client).write_all(line.as_bytes()).expect("sent");
    let (queue, _jobs) = channel::<Job>();
    let (work, _wakes) = channel::<Wake>();
    let (copies, copy_jobs) = channel::<CopyJob>();
    let counter: AtomicUsize = AtomicUsize::new(copying);
    handle_connection(service, &queue, &work, &copies, &counter, crate::service_protocol::RELEASE);
    let queued: bool = copy_jobs.try_recv().is_ok();
    // The job's stream, and with it the service's end of the pair, is dropped with the job, so the client reads to the end.
    let lines: Vec<String> = BufReader::new(client).lines().map_while(Result::ok).collect();
    return (lines, queued);
}

/// A client whose request must wait behind a copy that is running is told so at once, on its own connection; one that
/// finds the thread idle, and a wake, are not.
#[test]
fn a_client_queued_behind_a_running_copy_is_told_at_once() {
    let (busy, queued) = copy_message(1, vec![String::from("one")]);
    assert!(queued);
    assert_eq!(busy.len(), 2, "{busy:?}");
    assert!(busy[0].contains("welcome"), "{busy:?}");
    assert!(busy[1].contains("copy-notice") && busy[1].contains("busy copying"), "{busy:?}");
    let (idle, queued_idle) = copy_message(0, vec![String::from("one")]);
    assert!(queued_idle);
    assert_eq!(idle.len(), 1, "{idle:?}");
    let (wake, queued_wake) = copy_message(1, Vec::new());
    assert!(queued_wake);
    assert_eq!(wake.len(), 1, "a wake waits for nothing and is told nothing: {wake:?}");
}
