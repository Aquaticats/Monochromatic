//! Controls of the group-drop probe:
//!  the four answers its bytes can mean,
//!  the digits it writes without allocating,
//!  and the one answer this host gives,
//!  which is never a probe that could not be read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theAnswer('written')).toEqual({ kind: 'written' });
//! expect(theAnswer('open 13')).toEqual({ kind: 'skipped', errno: 13 });
//! ```

use super::{
    GroupDrop,
    ProbeCall,
    decimal_into,
    the_answer,
    the_group_drop,
};

/// What:
///  The `deny` write is the pass,
///  so this host needs no report.
#[test]
fn the_deny_written_is_a_pass() {
    assert_eq!(the_answer(b"written"), GroupDrop::Written);
}

/// What:
///  The open answering `EACCES` is the skip the owner added,
///  and no other answer of that open or of the write is.
#[test]
fn the_open_that_answered_eacces_is_the_skip_and_no_other_answer_is() {
    assert_eq!(
        the_answer(b"open 13"),
        GroupDrop::Skipped {
            errno: libc::EACCES
        }
    );
    for (written, call, errno) in [
        (b"open 1".as_slice(), ProbeCall::OpenSetgroups, libc::EPERM),
        (
            b"write 13".as_slice(),
            ProbeCall::WriteSetgroups,
            libc::EACCES,
        ),
        (
            b"write 1".as_slice(),
            ProbeCall::WriteSetgroups,
            libc::EPERM,
        ),
        (
            b"unshare 1".as_slice(),
            ProbeCall::UnshareUserNamespace,
            libc::EPERM,
        ),
    ] {
        assert_eq!(
            the_answer(written),
            GroupDrop::Answered {
                call,
                errno
            },
            "{} answering errno {errno} is that call's own answer",
            call.name()
        );
    }
}

/// What:
///  An answer this build cannot read is said as unread,
///  never as the skip and never as a pass.
#[test]
fn an_answer_this_build_cannot_read_is_said() {
    for written in [
        b"".as_slice(),
        b"who 13".as_slice(),
        b"open".as_slice(),
        b"open x".as_slice(),
        b"written 1".as_slice(),
        [0xff_u8, 0xfe].as_slice(),
    ] {
        assert!(
            matches!(the_answer(written), GroupDrop::Unread { .. }),
            "{} is no call and no number",
            String::from_utf8_lossy(written)
        );
    }
}

/// What:
///  A number is written as its own digits,
///  which is what the child can do without allocating.
#[test]
fn a_number_is_written_as_its_digits() {
    let mut text: [u8; 10] = [0; 10];
    for (value, digits) in [(13, "13"), (0, "0"), (1, "1"), (145, "145")] {
        let taken: usize = decimal_into(value, &mut text);
        assert_eq!(
            std::str::from_utf8(&text[..taken]).expect("digits are text"),
            digits,
            "{value} is written as its digits"
        );
    }
}

/// What:
///  The probe this host runs gives one of the two host answers,
///  not a probe that could not be read,
///  and it leaves no child behind.
/// Why:
///  The read is a fork,
///  a pipe and two waits,
///  and the controls above judge bytes alone;
///  this one exercises the machinery on whichever host it runs.
#[test]
fn the_probe_answers_on_this_host() {
    let answer: GroupDrop = the_group_drop();
    assert!(
        matches!(
            answer,
            GroupDrop::Written | GroupDrop::Skipped { .. } | GroupDrop::Answered { .. }
        ),
        "the probe's own machinery answers on this host, and it answered {answer:?}"
    );
}
