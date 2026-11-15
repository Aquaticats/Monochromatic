//! Controls for the stop this process relayed to a child and its sender:
//! the one word the handler writes and the wait reads.
//! Why:
//!   The two live in one word so that a process killed between two stores cannot pair one stop's signal with
//!   another stop's sender,
//!   and the packing is therefore what has to be right.

use super::*;

/// What:
///  A packed stop gives back the signal and the sender it was written with,
///  and a word that holds no sender is read as none.
/// Why:
///  The two live in one word so that a process killed between two stores cannot pair one stop's signal with
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
fn a_relayed_stop_is_read_back_with_the_sender_it_was_written_with() {
    assert_eq!(unpacked(packed(libc::SIGTERM, Some(4_242))), Some((libc::SIGTERM, Some(4_242))), "a process's stop keeps both halves");
    assert_eq!(unpacked(packed(libc::SIGINT, None)), Some((libc::SIGINT, None)), "a terminal's stop names no process");
    assert_eq!(unpacked(0), None, "a word no stop was written to holds none");
    // A sender alone is no stop: the signal half is what says there is one.
    assert_eq!(unpacked(packed(0, Some(4_242))), None, "a word with a sender and no signal holds no stop");
}
