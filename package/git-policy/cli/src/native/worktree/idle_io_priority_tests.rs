//! The lowering of the emptying's I/O priority,
//! and the priority a thread is left with.

use super::{class_here, with_idle_io_priority};

/// The class the work runs in while it is lowered,
/// as the kernel numbers it (`IOPRIO_CLASS_IDLE`).
const IDLE_CLASS: libc::c_int = 3;

/// A lowered stretch reports the idle class from inside,
/// and the thread has the class it had before once it returns.
#[test]
fn the_work_runs_at_the_lowest_class_and_the_thread_takes_its_own_back() {
    let before = class_here();
    let Some(lowered) = with_idle_io_priority(class_here) else { return };
    assert_eq!(lowered, IDLE_CLASS, "the emptying asks for the lowest I/O class Linux offers");
    assert_eq!(class_here(), before, "the thread is left as it was found");
}

/// A stretch that is lowered inside another one gives the outer stretch its class back,
/// so a removal nested in a removal does not leave the outer work at best effort.
#[test]
fn a_lowered_stretch_inside_another_restores_the_outer_class() {
    let before = class_here();
    let inside: Option<libc::c_int> = with_idle_io_priority(|| {
        let nested: Option<libc::c_int> = with_idle_io_priority(class_here);
        assert_eq!(nested, Some(IDLE_CLASS), "the inner stretch runs at the idle class");
        return class_here();
    });
    assert_eq!(inside, Some(IDLE_CLASS), "the outer stretch is still lowered after the inner one");
    assert_eq!(class_here(), before, "and the thread's own class is back afterwards");
}
