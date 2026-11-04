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
    let first: Captured = capture.snapshot(1024);
    assert_eq!((first.stdout.as_slice(), first.stderr.as_slice()), (b"onethree".as_slice(), b"two".as_slice()));
    assert!(first.notes.is_empty(), "{:?}", first.notes);
    assert_eq!(capture.snapshot(1024), first, "reading does not consume");
}

/// Output larger than one read buffer is collected whole.
#[test]
fn output_larger_than_a_read_buffer_is_collected_whole() {
    let capture: Capture = Capture::new().expect("the files");
    let (out, err) = capture.stdio().expect("the streams");
    let status = Command::new("sh").args(["-c", "head -c 100000 /dev/zero | tr '\\0' x"]).stdout(out).stderr(err).status().expect("the child");
    assert!(status.success());
    let all: Captured = capture.snapshot(1_000_000);
    assert_eq!(all.stdout.len(), 100_000);
    assert!(all.notes.is_empty());
}

/// Output larger than the limit keeps its tail and says what was left out (a tool prints its reason last).
#[test]
fn output_past_the_limit_keeps_the_tail_and_says_what_was_left_out() {
    let capture: Capture = Capture::new().expect("the files");
    let (out, err) = capture.stdio().expect("the streams");
    let status = Command::new("sh").args(["-c", "i=0; while [ $i -lt 2000 ]; do printf 'line %05d\\n' $i; i=$((i + 1)); done"]).stdout(out).stderr(err).status().expect("the child");
    assert!(status.success());
    let shown: Captured = capture.snapshot(100);
    assert_eq!(shown.stdout.len(), 100);
    assert!(String::from_utf8_lossy(shown.stdout.as_slice()).ends_with("line 01999\n"), "the tail is kept");
    assert_eq!(shown.notes.len(), 1, "{:?}", shown.notes);
    let written: usize = capture.snapshot(usize::MAX).stdout.len();
    let left_out: String = format!("{} bytes at the start of the standard output", written - shown.stdout.len());
    assert!(shown.notes[0].contains(left_out.as_str()) && shown.notes[0].contains("the last 100 bytes"), "{}", shown.notes[0]);
}

/// What is appended after the snapshot began is not chased (sol-1 round 3: the earlier control passed under a reader that reads to the
/// end of the file). The control appends to the file after every chunk the reader takes, so a reader that follows the end never gets
/// there; the snapshot has the length it found and the control has a count to stop a chasing reader with a failure.
#[test]
fn the_snapshot_reads_the_length_it_found_and_no_more() {
    let capture: Capture = Capture::new().expect("the files");
    let (out, err) = capture.stdio().expect("the streams");
    let found: usize = 100_000;
    let status = Command::new("sh").args(["-c", format!("head --bytes={found} /dev/zero | tr '\\0' 'a'").as_str()]).stdout(out).stderr(err).status().expect("the child");
    assert!(status.success());
    let mut appender = capture.out.try_clone().expect("a second handle on the same file");
    let mut chunks: usize = 0;
    let (shown, notes) = capture.snapshot_stdout_with(usize::MAX, &mut || {
        use std::io::{Seek, SeekFrom, Write};
        chunks += 1;
        assert!(chunks < 10_000, "the reader chased the growing end of the file");
        appender.seek(SeekFrom::End(0)).expect("end");
        appender.write_all(&[b'b'; 8192]).expect("append");
    });
    assert_eq!(shown.len(), found, "exactly the length found when the read began");
    assert!(shown.iter().all(|byte| { return *byte == b'a'; }), "none of what was appended");
    assert!(notes.is_empty(), "{notes:?}");
}

/// A read that fails is not the end of the output: the note says so, and the bytes read so far are shown as what they are.
#[test]
fn a_read_that_fails_is_reported_and_not_taken_for_the_end() {
    let directory: std::path::PathBuf = crate::test_support::fixture("output-capture-unreadable");
    let path: std::path::PathBuf = directory.join("only-writable");
    std::fs::write(&path, b"some output").expect("a file");
    let write_only: File = std::fs::OpenOptions::new().write(true).open(&path).expect("open for writing only");
    let capture: Capture = Capture { out: write_only.try_clone().expect("clone"), err: write_only };
    let shown: Captured = capture.snapshot(1024);
    assert!(shown.stdout.is_empty());
    assert!(shown.notes.iter().any(|note| { return note.contains("could not be read whole"); }), "{:?}", shown.notes);
    crate::test_support::remove(directory.as_path());
}
