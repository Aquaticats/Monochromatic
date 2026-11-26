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

/// A process the child started that holds its standard error does not hold the wrapper: the relay ends with the child.
#[test]
fn a_process_that_outlives_the_child_does_not_hold_the_relay() {
    let mut command: std::process::Command =
        a_child_writing("(sleep 3) & printf 'started\\n' >&2; exit 0");
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
    let took: std::time::Duration = began.elapsed();
    assert_eq!(
        String::from_utf8_lossy(relayed.kept.as_slice()),
        "started\n"
    );
    assert!(
        took < std::time::Duration::from_secs(2),
        "the relay waited {took:?} for a stream the child's own process holds"
    );
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
