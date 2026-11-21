//! What:
//!  Controls of the process layer on a machine that mounts nothing:
//!  a pipe pair carries a byte,
//!  a forked child ends in `_exit` and its status is the one it was given,
//!  and a process group of the child's own can be made and signalled.
//! Why:
//!  The mode's whole supervision rests on these calls,
//!  and a status the parent misreads would name the wrong ending in the record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pid = fork(); if (pid === 0) _exit(7); expect(waitBlocking(pid)).toEqual({ kind: 'exited', status: 7 });
//! ```

use super::{Exit, fork, pipe_pair, raw, set_process_group, wait_blocking};
use super::super::probe_child_prelude::exit_now;
use super::super::probe_pipe_lines::{read_one_blocking, write_all};

/// A child forked here ends with the status it was given,
/// and its parent reads that status.
#[test]
fn a_forked_child_ends_with_its_status() {
    match fork().expect("the fork is made") {
        None => exit_now(7),
        Some(pid) => {
            assert_eq!(wait_blocking(pid), Ok(Exit::Exited { status: 7 }));
        }
    }
}

/// A pipe pair carries one byte between two descriptors.
#[test]
fn a_pipe_pair_carries_one_byte() {
    let ends = pipe_pair().expect("a pipe is made");
    write_all(raw(&ends.write), b"P").expect("the byte is written");
    assert_eq!(read_one_blocking(raw(&ends.read)), Ok(b'P'));
}

/// A child's own process group can be made from the parent's side,
/// and the ending of one killed group is a signal,
/// never an exit status.
#[test]
fn a_process_group_is_made_and_its_kill_is_a_signal() {
    match fork().expect("the fork is made") {
        None => {
            let mut buffer: [u8; 1] = [0; 1];
            // SAFETY: the buffer is this call's own and the read waits for the parent's kill.
            let read: libc::ssize_t = unsafe { libc::read(libc::STDIN_FILENO, buffer.as_mut_ptr().cast::<libc::c_void>(), 1) };
            let _ = read;
            exit_now(0);
        }
        Some(pid) => {
            set_process_group(pid).expect("the group is made");
            assert_eq!(super::kill_process_group(pid), Ok(()));
            match wait_blocking(pid) {
                Ok(Exit::Signaled { signal }) => assert_eq!(signal, libc::SIGKILL),
                other => panic!("{other:?}"),
            }
        }
    }
}
