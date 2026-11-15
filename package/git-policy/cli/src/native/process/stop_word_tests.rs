//! Controls for a stop's signal and sender as one word.
//! Why:
//!   The record of a stopped commit states both as one fact,
//!   and a word that reads back wrong names a process that did not send what ended the command.

use super::*;

/// What:
///  A packed stop gives back the signal and the sender it was written with,
///  and a word that holds no sender is read as none.
/// Why:
///  The two live in one word so that a delivery or a kill between two writes cannot pair one stop's signal with
///  another stop's sender;
///  the packing is therefore the thing that has to be right,
///  and each shape a stop can have is read back here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(unpacked(packed(SIGTERM, 4242))).toEqual([SIGTERM, 4242]);
/// expect(unpacked(packed(SIGINT, undefined))).toEqual([SIGINT, undefined]);
/// expect(unpacked(0)).toBeUndefined();
/// ```
#[test]
fn a_stop_is_read_back_with_the_sender_it_was_written_with() {
    assert_eq!(unpacked(packed(libc::SIGTERM, Some(4_242))), Some((libc::SIGTERM, Some(4_242))), "a process's stop keeps both halves");
    assert_eq!(unpacked(packed(libc::SIGINT, None)), Some((libc::SIGINT, None)), "a terminal's stop names no process");
    assert_eq!(unpacked(0), None, "a word no stop was written to holds none");
    // A sender alone is no stop: the signal half is what says there is one.
    assert_eq!(unpacked(packed(0, Some(4_242))), None, "a word with a sender and no signal holds no stop");
}
