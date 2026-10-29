//! The worker handle's record of which pass is open:
//!  the finishing output of a pass that a newer post replaced must not close the newer pass.

/// The handle under test and its pass slots.
use super::TileWorker;
/// The output a pass hands to the window.
use crate::native::tile_pass::{PassKey, PassOutput};
/// The shaper a worker is started with.
use ide_app::shaped_text::TextShaper;

/// An output of the pass posted as number `generation`,
/// with no lines.
fn finished(generation: u64, done: bool) -> PassOutput {
    return PassOutput {
        key: PassKey {
            file: 1,
            revision: 0,
            scale: 1.0_f32.to_bits(),
            wrap: 673.0_f32.to_bits(),
        },
        generation,
        lines: vec![(0, 1)],
        done,
        shaped: 0,
        rebroken: 0,
    };
}

/// A pass that was replaced finishes after its replacement was posted;
/// the window must keep polling for the replacement,
/// which is still running.
#[test]
fn the_finish_of_a_replaced_pass_leaves_the_newer_pass_open() {
    let mut worker = TileWorker::new(TextShaper::new()).expect("worker starts");
    // Two posts were made, so the newest is number two and its pass is open.
    worker.pass_generation = 2;
    worker.pass_open = true;
    worker.shared.outbox.push(finished(1, true));
    let taken = worker
        .take_pass(usize::MAX)
        .expect("the stale finish is taken");
    assert!(taken.done && taken.generation == 1);
    assert!(
        worker.is_busy(),
        "the finish of the replaced pass closed the newer pass, so the window would stop polling for it"
    );
    worker.shared.outbox.push(finished(2, true));
    worker
        .take_pass(usize::MAX)
        .expect("the real finish is taken");
    assert!(
        !worker.is_busy(),
        "the finish of the newest pass left the pass open"
    );
}

/// Output that is not the last of the newest pass keeps it open.
#[test]
fn an_unfinished_output_of_the_newest_pass_leaves_it_open() {
    let mut worker = TileWorker::new(TextShaper::new()).expect("worker starts");
    worker.pass_generation = 1;
    worker.pass_open = true;
    worker.shared.outbox.push(finished(1, false));
    worker.take_pass(usize::MAX).expect("the output is taken");
    assert!(worker.is_busy(), "an unfinished pass was closed");
}
