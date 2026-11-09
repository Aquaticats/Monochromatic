//! Controls for the number a caller publishes when its child starts:
//! nothing before the start,
//! the first number after it,
//! and nothing changed by a later start.

use super::*;

/// Nothing is known before the child starts,
/// the number published is the number read,
/// and a second start of the same caller leaves it alone.
#[test]
fn the_first_number_published_is_the_child() {
    let told: ChildNumber = ChildNumber::not_started();
    assert_eq!(told.number(), None, "a child that has not started has no number");
    told.publish(7);
    assert_eq!(told.number(), Some(7), "the number the start returned is the child's");
    told.publish(9);
    assert_eq!(told.number(), Some(7), "a later start does not take the number from the child the caller waits for");
}

/// Two callers keep their own child,
/// so one landing's number never stands for another's.
#[test]
fn two_callers_keep_two_children() {
    let one: ChildNumber = ChildNumber::not_started();
    let other: ChildNumber = ChildNumber::not_started();
    one.publish(11);
    other.publish(12);
    assert_eq!((one.number(), other.number()), (Some(11), Some(12)));
}
