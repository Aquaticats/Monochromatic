//! Controls of the door every ending of a commit service goes through.

use super::*;

/// A folder the controls keep their notes in.
fn folder(name: &str) -> std::path::PathBuf {
    let root: std::path::PathBuf = crate::test_support::fixture_root().join(format!("ended-{name}"));
    let _ = std::fs::remove_dir_all(root.as_path());
    std::fs::create_dir_all(root.as_path()).expect("fixture folder");
    return root;
}

/// A stand-in for a service: a process this control started, whose number and birth identity a note can name.
fn a_process_to_end() -> std::process::Child {
    return std::process::Command::new("/bin/sleep").arg("120").spawn().expect("a process to end");
}

/// The note one process's own number and birth identity make.
fn note_for(directory: &Path, instance: &str, pid: i32) -> std::path::PathBuf {
    let note: std::path::PathBuf = crate::service_server_pid_note_of_an_instance::path(directory, instance);
    let identity: String = crate::process_identity::process_birth_identity(i64::from(pid)).expect("the birth identity is readable").expect("the process is there");
    std::fs::write(note.as_path(), format!("{pid} {identity}\n")).expect("the note");
    return note;
}

/// Whether `child` is still running, read from its own state.
fn still_running(child: &mut std::process::Child) -> bool {
    return matches!(child.try_wait(), Ok(None));
}

/// What:
///  The service the note names, and that this command started, is ended through the kernel's handle for it.
/// Why:
///  The review of `567ae37198` (finding 4): a signal by number after a birth check can reach a process that took the number.
///  The door makes a descriptor for the process while the identity is read, and the signal travels through it.
#[test]
#[cfg(target_os = "linux")]
fn the_service_the_note_names_is_ended_through_the_handle() {
    let root = folder("sent");
    let mut service: std::process::Child = a_process_to_end();
    let pid: i32 = i32::try_from(service.id()).expect("a process number");
    note_for(root.as_path(), "3f9a", pid);
    assert_eq!(end_the_service(root.as_path(), "3f9a", Some(pid)), Ended::Sent, "the note's process is this command's and is signalled");
    let ended: std::process::ExitStatus = service.wait().expect("the process ends");
    assert_eq!(std::os::unix::process::ExitStatusExt::signal(&ended), Some(libc::SIGKILL), "the signal the door sends is the one that ended it");
    let _ = std::fs::remove_dir_all(root.as_path());
}

/// What:
///  A service the note names but this command did not start is left alone.
/// Why:
///  The review of `567ae37198` (finding 3): a manager's fallback ended whichever service the shared note named,
///  and a delayed scope of another command's start can be the one that bound the address.
#[test]
#[cfg(target_os = "linux")]
fn a_service_this_command_did_not_start_is_left_alone() {
    let root = folder("not-ours");
    let mut service: std::process::Child = a_process_to_end();
    let pid: i32 = i32::try_from(service.id()).expect("a process number");
    note_for(root.as_path(), "3f9a", pid);
    let other: i32 = i32::try_from(std::process::id()).expect("a process number");
    assert_eq!(end_the_service(root.as_path(), "3f9a", Some(other)), Ended::NotOurs, "another command's service is not this command's to end");
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(still_running(&mut service), "the service that another start began is still running");
    assert_eq!(end_the_service(root.as_path(), "3f9a", Some(pid)), Ended::Sent, "the control ends its own process");
    let _ = service.wait();
    let _ = std::fs::remove_dir_all(root.as_path());
}

/// What:
///  A service the kernel refuses a handle for is not signalled at all.
/// Why:
///  The review of `567ae37198` (finding 4): the proof and the signal are two operations, and the descriptor is what closes
///  the interval. A door that fell back to the number would end this process, so its survival is the control that the
///  signal travels through the descriptor.
#[test]
#[cfg(target_os = "linux")]
fn a_service_without_a_handle_is_not_signalled() {
    let root = folder("unproven");
    let mut service: std::process::Child = a_process_to_end();
    let pid: i32 = i32::try_from(service.id()).expect("a process number");
    note_for(root.as_path(), "3f9a", pid);
    crate::member_signalled_through_its_pidfd::refuse_the_next_descriptors(1);
    assert_eq!(end_the_service(root.as_path(), "3f9a", Some(pid)), Ended::Unproven, "no handle is no signal");
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(still_running(&mut service), "a service the kernel gave no handle for is still running");
    assert_eq!(end_the_service(root.as_path(), "3f9a", Some(pid)), Ended::Sent, "the handle is made once the refusal is spent");
    let _ = service.wait();
    let _ = std::fs::remove_dir_all(root.as_path());
}

/// What:
///  A note that names no living process is gone, and the process that now holds its number is not signalled.
/// Why:
///  The note outlives a killed service, and a number it names can belong to another process.
///  The kernel's answer for a member that is not there is `Gone`, and the successor is untouched.
#[test]
#[cfg(target_os = "linux")]
fn a_note_that_names_no_living_process_is_gone() {
    let root = folder("gone");
    let mut successor: std::process::Child = a_process_to_end();
    let pid: i32 = i32::try_from(successor.id()).expect("a process number");
    let note: std::path::PathBuf = crate::service_server_pid_note_of_an_instance::path(root.as_path(), "3f9a");
    std::fs::write(note.as_path(), format!("{pid} linux:000000\n")).expect("a note of a birth that is not this process's");
    assert_eq!(end_the_service(root.as_path(), "3f9a", Some(pid)), Ended::Gone, "the note names no process that is there");
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(still_running(&mut successor), "the process that holds the number is not the one the note named, and is not signalled");
    let _killed = successor.kill();
    let _ = successor.wait();
    let _ = std::fs::remove_dir_all(root.as_path());
}
