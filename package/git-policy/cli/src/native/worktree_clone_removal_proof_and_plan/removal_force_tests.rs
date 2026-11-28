//! Controls for the count one removal carries:
//! no force,
//! one,
//! two and more,
//! and the two questions every step asks of it.

use crate::worktree_clone_removal_proof_and_plan::removal_force::RemovalForce;

/// A command that names no force carries none,
/// and neither question is answered yes.
#[test]
fn no_force_is_not_given_and_not_twice() {
    let none: RemovalForce = RemovalForce::none();
    assert_eq!(none.count(), 0);
    assert!(!none.is_given(), "no force is given");
    assert!(!none.is_twice(), "no force is twice");
}

/// One force is given and is not two.
#[test]
fn one_force_is_given_and_not_twice() {
    let once: RemovalForce = RemovalForce::of_count(1);
    assert_eq!(once.count(), 1);
    assert!(once.is_given(), "one force is given");
    assert!(!once.is_twice(), "one force is not two");
}

/// Two forces are given and are twice,
/// and every count above two answers both questions the same way.
#[test]
fn two_forces_and_more_are_twice() {
    for count in [2, 3, 7] {
        let many: RemovalForce = RemovalForce::of_count(count);
        assert_eq!(many.count(), count);
        assert!(many.is_given(), "{count} forces are given");
        assert!(many.is_twice(), "{count} forces are twice");
    }
}
