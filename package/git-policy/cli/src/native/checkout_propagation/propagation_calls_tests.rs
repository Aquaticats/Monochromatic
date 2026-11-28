//! What:
//!  The seam the step's kernel acts answer,
//! as a control exercises it:
//!  a double that answers each act,
//!  the number each refusal carries,
//!  and the sentence a person reads.
//! Why:
//!  Every value control of the step drives this double,
//!  so the double must answer the same shape the kernel does and must say
//!  which act it was asked for,
//!  or a control would pass on an act the step never made
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "The tests a build must have").
//!  The number each refusal carries is what the record's `errno` field and the
//!  summary line print.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const calls = new ScriptedCalls(); expect(calls.opened()).toBe('/home/user/Monochromatic');
//! ```

use super::*;

#[test]
fn a_refused_open_carries_the_number_and_says_which_call_refused() {
    let failure: CallFailure = CallFailure::Open {
        errno: libc::ENOENT,
    };
    assert_eq!(failure.errno(), Some(libc::ENOENT));
    assert!(
        failure.message().contains("opened") && failure.message().contains("2"),
        "the sentence names the act and the number: {}",
        failure.message()
    );
}

#[test]
fn a_refused_set_and_table_carry_their_own_numbers() {
    let set: CallFailure = CallFailure::Set {
        errno: libc::EPERM,
    };
    let table: CallFailure = CallFailure::Table {
        errno: libc::E2BIG,
    };
    assert_eq!(set.errno(), Some(libc::EPERM));
    assert_eq!(table.errno(), Some(libc::E2BIG));
    assert!(set.message().contains("refused the set"));
    assert!(table.message().contains("mount table"));
}

#[test]
fn a_read_refusal_keeps_the_number_a_kernel_call_answered() {
    let read: CallFailure = CallFailure::Read(MountReadFailure::Refused {
        call: "statmount",
        errno: libc::EINVAL,
    });
    let unsupported: CallFailure = CallFailure::Read(MountReadFailure::Unsupported {
        why: String::from("this kernel reports no source"),
    });
    assert_eq!(read.errno(), Some(libc::EINVAL));
    assert_eq!(
        unsupported.errno(),
        None,
        "a kernel that cannot answer gives the record no number to print"
    );
    assert!(
        unsupported.message().contains("no source"),
        "the sentence carries what the kernel lacks: {}",
        unsupported.message()
    );
}
