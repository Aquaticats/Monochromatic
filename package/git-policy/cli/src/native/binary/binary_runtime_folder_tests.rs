//! What:
//!  The runtime folders the commit-service controls listen in:
//!  that the thread that named one removes it when it ends,
//!  that a folder another run made at the same name is left as it is,
//!  and that a folder a service still answers in is kept.
//! Why:
//!  A folder at the top of the agent scratch directory is shared by every run of every session,
//!  and it was left behind there by thousands of runs before these controls existed
//!  (`doc/handover/cli-git-rust-implementation.md`,
//! the owner's decision of 2026-11-15 on the `rn-` and `rt-` folders).
//!  Each of the three is what the removal has to tell apart:
//!  its own folder,
//!  a folder whose name another run took,
//!  and a folder whose service has not ended.

use super::support_runtime_folders::{release, runtime_folder_for};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How long a control waits for a thread that only names a folder and reports it.
/// The thread does two file-operations and a channel send,
/// so this is far past it and still bounded.
const THREAD_PATIENCE: Duration = Duration::from_secs(60);

/// A socket folder of the shape a service keeps under its runtime directory.
/// Why:
///  The service of a control binds `<runtime>/monochromatic-git-policy-cli/<instance>.sock`,
///  and a control of the folder has to put its socket where the removal looks for one.
fn socket_directory_of(runtime: &Path) -> PathBuf {
    return runtime.join(git_policy_cli::service_instance::SOCKET_DIRECTORY);
}

/// A name of this process's own for a runtime folder,
/// so two controls of one process never meet in one folder.
fn a_name(label: &str) -> String {
    return format!("runtime-folder-guard-{label}-{}", std::process::id());
}

/// What:
///  The runtime folder a control named goes when the test thread that named it ends,
///  with the stale socket file a service that ended leaves in it.
/// Why:
///   The maker is the owner:
/// a folder no service handle ever ends is removed at the end of the thread that asked for it,
/// and a socket file nothing answers on does not keep it.
#[test]
fn a_runtime_folder_goes_when_the_test_thread_that_named_it_ends() {
    let name: String = a_name("passing");
    let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();
    std::thread::spawn(move || {
        let runtime: PathBuf = runtime_folder_for(name.as_str());
        let folder: PathBuf = socket_directory_of(runtime.as_path());
        std::fs::create_dir_all(folder.as_path()).expect("the folder of the sockets");
        // A listener that ends leaves its socket file, which is what a service that was stopped leaves too.
        let socket: PathBuf = folder.join(format!("{}.sock", "0".repeat(git_policy_cli::service_instance::INSTANCE_LENGTH)));
        let listener = std::os::unix::net::UnixListener::bind(&socket).expect("a socket of the control");
        drop(listener);
        assert!(socket.exists(), "the stale socket file is there when this thread ends");
        sender.send(runtime).expect("the folder this thread named");
    })
    .join()
    .expect("the thread that named a folder ends");
    let runtime: PathBuf = receiver.recv_timeout(THREAD_PATIENCE).expect("the folder this thread named");
    assert!(
        !runtime.exists(),
        "{} stands at the top of the scratch root although the thread that named it has ended",
        runtime.display()
    );
}

/// What:
///  A folder another run made at the name a control used is left as it is.
/// Why:
///   A runtime folder is named for a process number,
/// and a process number comes round:
/// about 549 processes start a second on this machine,
///  so the folder at a name is not proof of whose it is,
/// and the device and inode the folder was made as is.
#[test]
fn a_runtime_folder_another_run_made_at_the_same_name_is_left_as_it_is() {
    let name: String = a_name("taken");
    let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();
    std::thread::spawn(move || {
        let runtime: PathBuf = runtime_folder_for(name.as_str());
        // Another run's folder, at the very name: one folder is renamed over the empty folder this thread made,
        // which is the identity of a folder another run made and never the one this thread made.
        let another: PathBuf = runtime.parent().expect("a runtime folder stands in a folder").join("runtime-folder-guard-another-run");
        let _stale = std::fs::remove_dir_all(another.as_path());
        std::fs::create_dir(another.as_path()).expect("another run's folder");
        std::fs::rename(another.as_path(), runtime.as_path()).expect("another run's folder takes the name");
        sender.send(runtime).expect("the folder another run made");
    })
    .join()
    .expect("the thread that named a folder ends");
    let taken: PathBuf = receiver.recv_timeout(THREAD_PATIENCE).expect("the folder another run made");
    assert!(
        taken.is_dir(),
        "{} was removed although another run made the folder standing there",
        taken.display()
    );
    // The folder this control made for another run is the control's to remove.
    std::fs::remove_dir(taken.as_path()).expect("the folder this control made for another run");
}

/// What:
///  A folder a service still answers in is kept,
///  and goes when the service has ended and the removal is asked again.
/// Why:
///   The socket of a live service is inside the folder:
/// removing the folder under it leaves the service listening at nothing the wrapper can find,
///  so the folder waits for the service,
/// and the control below is the whole of that wait.
#[test]
fn a_runtime_folder_a_service_still_answers_in_is_kept_until_it_ends() {
    let name: String = a_name("served");
    let (made_sender, made_receiver) = std::sync::mpsc::channel::<PathBuf>();
    let (bound_sender, bound_receiver) = std::sync::mpsc::channel::<()>();
    let naming = std::thread::spawn(move || {
        let runtime: PathBuf = runtime_folder_for(name.as_str());
        made_sender.send(runtime).expect("the folder this thread named");
        // The thread that named the folder ends only once the service of this control listens in it.
        bound_receiver.recv_timeout(THREAD_PATIENCE).expect("the service listened");
    });
    let runtime: PathBuf = made_receiver.recv_timeout(THREAD_PATIENCE).expect("the folder the control named");
    let socket_folder: PathBuf = socket_directory_of(runtime.as_path());
    std::fs::create_dir_all(socket_folder.as_path()).expect("the folder of the sockets");
    let socket: PathBuf = socket_folder.join(format!("{}.sock", "1".repeat(git_policy_cli::service_instance::INSTANCE_LENGTH)));
    let listener = std::os::unix::net::UnixListener::bind(&socket).expect("the service's socket");
    bound_sender.send(()).expect("the naming thread waits for the service");
    naming.join().expect("the thread that named the folder ends");
    assert!(
        runtime.is_dir(),
        "{} was removed under the service that still answers in it",
        runtime.display()
    );
    drop(listener);
    release(runtime.as_path());
    assert!(!runtime.exists(), "{} stands after the service ended and the removal was asked again", runtime.display());
}
