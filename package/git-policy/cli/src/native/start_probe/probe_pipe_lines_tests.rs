//! What:
//!  Controls of the pipe's bytes:
//!  a whole line is written and read back,
//!  a non-blocking read says nothing-yet rather than end of file,
//!  and an ended pipe says so.
//! Why:
//!  A status pipe that reported its end on every empty read would fail a
//!  child that was still working.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeAll(pipe.write, 'line\n'); expect(readAvailable(pipe.read, buffer)).toEqual({ kind: 'bytes', count: 5 });
//! ```

use super::{
    ReadSome,
    read_available,
    read_one_blocking,
    set_nonblocking,
    write_all,
};
use std::os::fd::AsRawFd;

/// A line written to a pipe reads back whole,
/// and an empty non-blocking pipe says nothing yet.
#[test]
fn the_pipe_carries_whole_lines_and_says_nothing_yet() {
    let ends = super::super::probe_processes::pipe_pair().expect("a pipe is made");
    set_nonblocking(ends.read.as_raw_fd()).expect("the read end is non-blocking");
    let mut buffer: [u8; 16] = [0; 16];
    assert_eq!(
        read_available(ends.read.as_raw_fd(), buffer.as_mut_slice()),
        Ok(ReadSome::NothingYet)
    );
    write_all(ends.write.as_raw_fd(), b"line\n").expect("the line is written");
    assert_eq!(
        read_available(ends.read.as_raw_fd(), buffer.as_mut_slice()),
        Ok(ReadSome::Bytes(5))
    );
    assert_eq!(&buffer[..5], b"line\n");
    assert_eq!(
        read_available(ends.read.as_raw_fd(), buffer.as_mut_slice()),
        Ok(ReadSome::NothingYet)
    );
    drop(ends.write);
    assert_eq!(
        read_available(ends.read.as_raw_fd(), buffer.as_mut_slice()),
        Ok(ReadSome::End)
    );
}

/// One byte is read with a wait,
/// and a pipe that ended is an error rather than a byte.
#[test]
fn one_byte_is_read_with_a_wait() {
    let ends = super::super::probe_processes::pipe_pair().expect("a pipe is made");
    write_all(ends.write.as_raw_fd(), b"P").expect("the byte is written");
    assert_eq!(read_one_blocking(ends.read.as_raw_fd()), Ok(b'P'));
    drop(ends.write);
    assert!(read_one_blocking(ends.read.as_raw_fd()).is_err());
}
