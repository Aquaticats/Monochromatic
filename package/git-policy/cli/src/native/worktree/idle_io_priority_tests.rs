//! The lowering of the emptying's I/O priority,
//! the thread the lowered value belongs to,
//! and the priority a thread is left with.

use super::{LoweredIoPriority, class_here, with_idle_io_priority};

/// The class the work runs in while it is lowered,
/// as the kernel numbers it (`IOPRIO_CLASS_IDLE`).
const IDLE_CLASS: libc::c_int = 3;

/// The class this thread has,
///  as the work of a stretch that needs no state of its own.
fn class_without_state(_state: ()) -> Option<libc::c_int> {
    return class_here();
}

/// A stretch that runs another lowered stretch inside it,
///  and reports the class it still has once the inner one is over.
fn nested_stretch(_state: ()) -> Option<libc::c_int> {
    let nested: Option<libc::c_int> = with_idle_io_priority((), class_without_state);
    assert_eq!(nested, Some(IDLE_CLASS), "the inner stretch runs at the idle class");
    return class_here();
}

/// A stretch that unwinds on the thread it was lowered on,
/// which is what a removal that panics does to a pool's thread.
fn unwinding_stretch(_state: ()) -> Option<libc::c_int> {
    panic!("the work of one stretch unwinds");
}

/// A lowered stretch reports the idle class from inside,
/// and the thread has the class it had before once it returns.
#[test]
fn the_work_runs_at_the_lowest_class_and_the_thread_takes_its_own_back() {
    let before = class_here();
    let Some(lowered) = with_idle_io_priority((), class_without_state) else { return };
    assert_eq!(lowered, IDLE_CLASS, "the emptying asks for the lowest I/O class Linux offers");
    assert_eq!(class_here(), before, "the thread is left as it was found");
}

/// A stretch that is lowered inside another one gives the outer stretch its class back,
/// so a removal nested in a removal does not leave the outer work at best effort.
#[test]
fn a_lowered_stretch_inside_another_restores_the_outer_class() {
    let before = class_here();
    let inside: Option<libc::c_int> = with_idle_io_priority((), nested_stretch);
    assert_eq!(inside, Some(IDLE_CLASS), "the outer stretch is still lowered after the inner one");
    assert_eq!(class_here(), before, "and the thread's own class is back afterwards");
}

/// A stretch whose work unwinds still gives the thread its own class back:
/// a removal can panic,
/// and the pool reuses the thread it ran on.
#[test]
fn a_stretch_that_unwinds_gives_the_thread_its_own_class_back() {
    let before = class_here();
    let Some(lowered) = with_idle_io_priority((), class_without_state) else { return };
    assert_eq!(lowered, IDLE_CLASS, "the emptying asks for the lowest I/O class Linux offers");
    let unwound: std::thread::Result<Option<libc::c_int>> = std::panic::catch_unwind(|| {
        return with_idle_io_priority((), unwinding_stretch);
    });
    assert!(unwound.is_err(), "the work's own panic reaches the caller");
    assert_eq!(class_here(), before, "the thread is left as it was found, panic included");
}

/// What:
///  The lowering stays on the thread that made it,
/// which is what lets every restore give the right thread its priority back.
/// Why:
///   `Drop` writes the priority of the thread that drops the value,
/// so a value carried to another thread
///  would overwrite that thread's own priority and leave the lowered thread at the idle class.
/// The refusal is the compiler's:
/// the ask below has two answers exactly while the value may be carried to another thread,
/// and one answer while it may not,
///  so this file stops building the moment the value becomes movable again
/// (`error[E0283]: type annotations needed`).
#[test]
fn the_lowering_cannot_be_carried_to_another_thread() {
    /// What:
    ///  A marker only a value that may be carried to another thread has.
    struct IfCarriedToAnotherThread;

    /// What:
    ///  The one method every value offers, under each marker the value qualifies for.
    trait Offered<Marker> {
        fn offered() {}
    }

    /// What:
    ///  The marker every value qualifies for, which is the one the ask names when nothing else does.
    impl<T: ?Sized> Offered<()> for T {}

    /// What:
    ///  The marker a value that may be carried to another thread also qualifies for,
    /// which leaves the ask naming neither of the two.
    impl<T: ?Sized + Send> Offered<IfCarriedToAnotherThread> for T {}

    let _ = <LoweredIoPriority as Offered<_>>::offered;
}
