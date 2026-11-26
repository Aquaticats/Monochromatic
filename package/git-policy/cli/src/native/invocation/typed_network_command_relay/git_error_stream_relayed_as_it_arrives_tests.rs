//! Controls for the relay of a child's error stream:
//! the bytes on a pipe and on a terminal,
//! what a child's own `isatty(2)` answers under each,
//! the tail's bound,
//! the end rule when a process the child started holds the stream,
//! and a refused start.

use super::{
    ErrorStreamShape,
    RelayedErrorStream,
    read_to_the_child_s_end,
    start_relaying,
};
use crate::forwarding::ChildOutcome;
use crate::test_process_owner::own_group_of_number;
// What: `CommandExt` adds `process_group` to the standard library's command type, which is the group a child leads.
// Why:  A child the relay starts is ended by the relay, and only the group it leads can carry what it left to this
//  control.
use std::os::unix::process::CommandExt as _;

/// The whole seconds a process the child starts keeps the child's standard error open,
/// which the relay must not wait for and the control must end.
/// Why:
///  A hold many times the control's own bound cannot be told from an endless one by the relay's end,
///  so the relay can only answer inside the bound by ending with the child.
const GRANDCHILD_HOLD_SECONDS: u32 = 30;

/// How long the control gives the group it killed to leave the process table.
/// Why:
///  A signal is delivered at once and the kernel withdraws a killed process in its own time,
///  so this bound judges the kernel and not the child,
///  and five seconds is what the shared waits give such work
///  (`testing/test_support_wait_bounds.rs`).
const ENDED_WITHIN: std::time::Duration = std::time::Duration::from_secs(5);

/// What:
///  A command that runs one shell line as the child the relay reads, standing in for Git.
/// Why:
///  The relay reads whatever the child writes to its standard error, and a shell writes it without a repository or a network.
///  `sh` writes to the descriptor the relay connected, which is what the control reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (text: string) => new Command('/bin/sh').arg('-c').arg(text);
/// ```
fn a_child_writing(script: &str) -> std::process::Command {
    let mut command: std::process::Command = std::process::Command::new("/bin/sh");
    command.arg("-c").arg(script);
    return command;
}

/// The bytes a child wrote reach the caller on a pipe, and its exit code is the answer.
#[test]
fn a_pipe_relays_the_child_s_bytes_and_its_code() {
    let mut command: std::process::Command = a_child_writing("printf 'one\\ntwo\\n' >&2; exit 3");
    let mut relayed_bytes: Vec<u8> = Vec::new();
    let start: super::RelayedStart =
        start_relaying(ErrorStreamShape::Pipe, None, &mut command).expect("the child starts");
    let relayed: RelayedErrorStream = read_to_the_child_s_end(start, &mut relayed_bytes)
        .expect("the relay reads to the child's end");
    assert_eq!(
        String::from_utf8_lossy(relayed.kept.as_slice()),
        "one\ntwo\n"
    );
    assert_eq!(relayed.outcome, ChildOutcome::Exited(3));
    // The same bytes went where the person reads them, which is what the relay is for.
    assert_eq!(relayed_bytes, b"one\ntwo\n".to_vec());
}

/// A terminal gives the child the answer plain Git sees, and clearing the output processing keeps the bytes plain Git's.
#[test]
fn a_terminal_is_a_terminal_and_keeps_the_line_feeds_it_got() {
    let mut asking: std::process::Command = a_child_writing(
        "if [ -t 2 ]; then printf 'terminal\\n' >&2; else printf 'pipe\\n' >&2; fi",
    );
    let mut relayed_bytes: Vec<u8> = Vec::new();
    let start: super::RelayedStart =
        start_relaying(ErrorStreamShape::Terminal, None, &mut asking).expect("the child starts");
    let relayed: RelayedErrorStream = read_to_the_child_s_end(start, &mut relayed_bytes)
        .expect("the relay reads to the child's end");
    assert_eq!(
        String::from_utf8_lossy(relayed.kept.as_slice()),
        "terminal\n"
    );
    let mut piped: std::process::Command = a_child_writing(
        "if [ -t 2 ]; then printf 'terminal\\n' >&2; else printf 'pipe\\n' >&2; fi",
    );
    let mut piped_bytes: Vec<u8> = Vec::new();
    let piped_start: super::RelayedStart =
        start_relaying(ErrorStreamShape::Pipe, None, &mut piped).expect("the child starts");
    let piped_relayed: RelayedErrorStream = read_to_the_child_s_end(piped_start, &mut piped_bytes)
        .expect("the relay reads to the child's end");
    assert_eq!(
        String::from_utf8_lossy(piped_relayed.kept.as_slice()),
        "pipe\n"
    );
}

/// A terminal whose output processing was left on would turn every line feed into a carriage return and a line feed.
#[test]
fn a_terminal_does_not_double_the_line_endings() {
    let mut command: std::process::Command = a_child_writing("printf 'a\\nb\\n' >&2; exit 0");
    let mut relayed_bytes: Vec<u8> = Vec::new();
    let start: super::RelayedStart =
        start_relaying(ErrorStreamShape::Terminal, None, &mut command).expect("the child starts");
    let relayed: RelayedErrorStream = read_to_the_child_s_end(start, &mut relayed_bytes)
        .expect("the relay reads to the child's end");
    assert_eq!(relayed.kept, b"a\nb\n".to_vec());
}

/// The kept tail is the stream's end and no more, so one command cannot hold all of what it printed.
#[test]
fn the_kept_tail_is_the_stream_s_end_and_no_more() {
    let mut command: std::process::Command = a_child_writing(
        "dd if=/dev/zero bs=1000 count=100 2>/dev/null | tr '\\0' 'A' >&2; printf 'BBBBBBBBBB' >&2",
    );
    let mut relayed_bytes: Vec<u8> = Vec::new();
    let start: super::RelayedStart =
        start_relaying(ErrorStreamShape::Pipe, None, &mut command).expect("the child starts");
    let relayed: RelayedErrorStream = read_to_the_child_s_end(start, &mut relayed_bytes)
        .expect("the relay reads to the child's end");
    assert_eq!(relayed.kept.len(), super::KEPT_TAIL_BYTES);
    assert_eq!(&relayed.kept[super::KEPT_TAIL_BYTES - 10..], b"BBBBBBBBBB");
}

/// A process the child started that holds its standard error does not hold the wrapper:
/// the relay ends with the child,
/// and what the child left is ended and waited for before the control returns.
#[test]
fn a_process_that_outlives_the_child_does_not_hold_the_relay() {
    let root: std::path::PathBuf = crate::test_support::fixture("relay-ends-with-the-child");
    let group_file: std::path::PathBuf = root.join("group");
    // What: `$$` is the shell's own process number, which is also the number of the group it leads, and the job it puts
    //  in the background stays in that group.
    // Why:  The relay answers a child's number only when a `SignalGuard` started that child, and a control of the bytes
    //  passes none, so the child writes down the number the control ends what it left by (issue 6501).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const script = `printf '%s\n' "$$" > '${groupFile}'; (exec sleep ${hold}) & printf 'started\n' >&2; exit 0`;
    // ```
    let script: String = format!(
        "printf '%s\\n' \"$$\" > '{}'; (exec /usr/bin/sleep {GRANDCHILD_HOLD_SECONDS}) & printf 'started\\n' >&2; exit 0",
        group_file.display()
    );
    let mut command: std::process::Command = a_child_writing(script.as_str());
    // What: `process_group(0)` makes the child lead a process group whose number is its own, and keeps everything it
    //  starts in that group.
    // Why:  The process holding the stream is one this control never gets a handle on,
    //  so the group is the one thing it can name and end once the child is gone (issue 6501).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // command.detached = true;
    // ```
    command.process_group(0);
    let mut relayed_bytes: Vec<u8> = Vec::new();
    let start: super::RelayedStart =
        start_relaying(ErrorStreamShape::Pipe, None, &mut command).expect("the child starts");
    // What: `Instant` is a clock reading whose difference is an elapsed time.
    // Why:  The design's end rule is a bound, and a bound is measured.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const began = Date.now();
    // ```
    let began: std::time::Instant = std::time::Instant::now();
    let relayed: RelayedErrorStream = read_to_the_child_s_end(start, &mut relayed_bytes)
        .expect("the relay reads to the child's end");
    // What: the child wrote its group's number before it ended, and that group holds the process it left.
    // Why:  Giving the number to the thread's owner ends that process however the assertions below end this test
    //  (issue 6501).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const group = writtenGroup(groupFile); ownGroupOfNumber(group);
    // ```
    let group: libc::pid_t = written_group(group_file.as_path());
    own_group_of_number(u32::try_from(group).expect("a pid fits the pid type"));
    let took: std::time::Duration = began.elapsed();
    assert_eq!(
        String::from_utf8_lossy(relayed.kept.as_slice()),
        "started\n"
    );
    assert!(
        took < std::time::Duration::from_secs(2),
        "the relay waited {took:?} for a stream the child's own process holds"
    );
    end_the_group(group);
}

/// What:
///  The process group number a child of this control wrote down before it ended.
/// Why:
///   The relay answers a child's number only when a `SignalGuard` started that child,
///  and this control passes none,
///  so the child's shell writes the number of the group it leads,
///  which is the group the control ends (issue 6501).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writtenGroup(path: string): number; // parseInt(readFileSync(path, 'utf8'), 10)
/// ```
fn written_group(path: &std::path::Path) -> libc::pid_t {
    let text: String = std::fs::read_to_string(path)
        .expect("the child wrote the number of the group it leads before it ended");
    return text.trim().parse().expect("a group number the child wrote");
}

/// What:
///  Kill the process group the child led and wait for it,
///  bounded by `ENDED_WITHIN`,
///  until no process of it is left.
/// Why:
///   The control's claim is that the relay ends with the child,
///  not that the process holding the stream is gone:
///  what the child left is still the control's to end,
///  and a test that returned with it running holds the run's own output past its end,
///  which the container gate reports as a leak (issue 6501).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endTheGroup(group: number): void; // killpg(group, 'SIGKILL'); waitUntil(() => !groupHasAMember(group), 5_000)
/// ```
fn end_the_group(group: libc::pid_t) {
    // SAFETY: `killpg` takes a group number and a signal number and touches no memory of this process.
    unsafe {
        libc::killpg(group, libc::SIGKILL);
    }
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        // What: `killpg` with signal `0` asks whether one process of the group is still there, sending nothing.
        // Why:  A group with no member left is the state this waits for, and the bound fails the control rather than
        //  letting the run wait for a process the kernel never withdraws.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (killpg(group, 0) !== 0) return;
        // ```
        // SAFETY: as above; signal `0` sends nothing.
        let present: libc::c_int = unsafe { libc::killpg(group, 0) };
        if present != 0 {
            return;
        }
        assert!(
            began.elapsed() < ENDED_WITHIN,
            "the group {group} still holds a process {:?} after the child ended",
            began.elapsed()
        );
        std::thread::sleep(crate::test_support::WAIT_LOOK_EVERY);
    }
}

/// A program the system will not start is a refused start, not an empty transcript.
#[test]
fn a_child_that_cannot_start_is_refused() {
    let mut command: std::process::Command =
        std::process::Command::new("/nonexistent/typed-network-command-relay-control");
    // What: `match` reads the result without needing the success value to be printable.
    // Why:  The started child owns descriptors and a process, so it is not a value a control prints.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const answered = startRelaying(...); assert(answered.error?.kind === 'NotFound');
    // ```
    match start_relaying(ErrorStreamShape::Pipe, None, &mut command) {
        Ok(_started) => panic!("a missing program is not started"),
        Err(refused) => assert_eq!(refused.kind(), std::io::ErrorKind::NotFound),
    }
}
