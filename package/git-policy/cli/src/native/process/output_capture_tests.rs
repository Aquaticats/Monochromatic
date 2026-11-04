//! Controls for the captured output of a child.

use super::*;
use std::process::Command;

/// What a child writes to both streams is read back whole, in order, and reading twice gives the same bytes.
#[test]
fn what_a_child_writes_to_both_streams_is_read_back_whole() {
    let capture: Capture = Capture::new().expect("the files");
    let (out, err) = capture.stdio().expect("the streams");
    let status = Command::new("sh").args(["-c", "printf one; printf two >&2; printf three"]).stdout(out).stderr(err).status().expect("the child");
    assert!(status.success());
    assert_eq!(capture.read(), (b"onethree".to_vec(), b"two".to_vec()));
    assert_eq!(capture.read(), (b"onethree".to_vec(), b"two".to_vec()), "reading does not consume");
}

/// Output larger than one read buffer is collected whole.
#[test]
fn output_larger_than_a_read_buffer_is_collected_whole() {
    let capture: Capture = Capture::new().expect("the files");
    let (out, err) = capture.stdio().expect("the streams");
    let status = Command::new("sh").args(["-c", "head -c 100000 /dev/zero | tr '\\0' x"]).stdout(out).stderr(err).status().expect("the child");
    assert!(status.success());
    assert_eq!(capture.read().0.len(), 100_000);
}
