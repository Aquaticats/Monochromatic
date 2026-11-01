//! Unit tests for the focus order.

use super::promote;

/// A new entry goes first.
#[test]
fn a_new_entry_goes_first() {
    let mut order = vec![1, 2];
    promote(&mut order, 3);
    assert_eq!(order, vec![3, 1, 2]);
}

/// An entry already present moves to the front and is not duplicated.
#[test]
fn an_existing_entry_moves_to_the_front_once() {
    let mut order = vec![3, 1, 2];
    promote(&mut order, 2);
    assert_eq!(order, vec![2, 3, 1]);
    promote(&mut order, 2);
    assert_eq!(order, vec![2, 3, 1]);
}

/// Dropping the first entry leaves the previous focus first,
///  the order focus returns through.
#[test]
fn removing_the_front_exposes_the_previous_focus() {
    let mut order = vec![1];
    promote(&mut order, 2);
    order.retain(|entry| return *entry != 2);
    assert_eq!(order.first(), Some(&1));
}
