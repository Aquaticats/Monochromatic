//! A minimap picture asks the window for a frame of its own only when no frame draws it first.
//!  A scroll draws a frame at every step of the input,
//!  and the compositor shows the frames after an extra one a step later,
//!  so the text does not move in one step of the scroll.
//!  An idle window draws no frame by itself,
//!  so it must be asked for one or the old picture stays on screen.
//! Nothing here waits on the window thread:
//! every wait advances timers and lets the workers answer.

/// The reader fixture and its window.
use super::find_tests::Reader;
/// The reader fixture,
/// the wrapped text,
/// and the window sizing.
use super::{
    minimap::PICTURE_FRAMES_ASKED,
    minimap_tests::picture_still,
    sidebar_tests::{TEXT_LEFT, minimap_beside},
    tile_reads::draw_minimap,
    wrap_tests::{long_text, open},
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Toolkit events.
use slint::{
    ComponentHandle, LogicalSize,
    platform::{WindowEvent, update_timers_and_animations},
};
/// Counters are atomic.
use std::{sync::atomic::Ordering, time::Duration};

/// A window whose first picture is on screen and drawn,
/// and the number of pictures set and frames asked for so far.
fn drawn() -> (tempfile::TempDir, Reader, u64, u64) {
    let (project, opened) = open(&long_text(400));
    picture_still(&opened);
    draw_minimap(&opened.window);
    let images = opened.source.borrow().minimap.images_set;
    let asked = PICTURE_FRAMES_ASKED.load(Ordering::SeqCst);
    return (project, opened, images, asked);
}

/// Change the window by one pixel and advance the timers until the picture of the change is handed to the GPU,
/// and stop in the step that handed it over:
/// no timer runs between that step and the next thing the caller does.
fn hand_over_a_picture(opened: &Reader, images: u64) {
    opened.window.window().dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(TEXT_LEFT + 1330.0 + minimap_beside(1330.0), 3501.0),
    });
    update_timers_and_animations();
    let mut waiting = Waiting::slow();
    while opened.source.borrow().minimap.images_set <= images {
        waiting.step("the picture of the resized window did not arrive");
    }
}

/// A picture that arrives while the window is drawing frames,
/// as it does during a scroll,
/// is drawn by the next frame,
/// and the window asks for no frame of its own.
#[test]
fn a_picture_that_a_frame_draws_asks_for_no_frame_of_its_own() {
    let (_project, opened, images, asked) = drawn();
    hand_over_a_picture(&opened, images);
    assert!(
        opened.source.borrow().gpu.minimap_waiting(),
        "the picture was not waiting for a frame"
    );
    assert_eq!(
        PICTURE_FRAMES_ASKED.load(Ordering::SeqCst),
        asked,
        "a frame was asked for at the hand-over, so a scroll gets a frame between two of its steps"
    );
    // The next step of the scroll draws a frame.
    draw_minimap(&opened.window);
    assert!(
        !opened.source.borrow().gpu.minimap_waiting(),
        "the frame did not draw the picture"
    );
    std::thread::sleep(Duration::from_millis(100));
    update_timers_and_animations();
    assert_eq!(
        PICTURE_FRAMES_ASKED.load(Ordering::SeqCst),
        asked,
        "a frame was asked for after a frame had drawn the picture"
    );
    opened.window.hide().expect("close picture window");
}

/// A picture that arrives while the window draws no frame is drawn:
/// the window asks for one once the time a frame needs to come has passed.
#[test]
fn a_picture_that_no_frame_draws_asks_for_a_frame() {
    let (_project, opened, images, asked) = drawn();
    hand_over_a_picture(&opened, images);
    let mut waiting = Waiting::eventually();
    while PICTURE_FRAMES_ASKED.load(Ordering::SeqCst) == asked {
        waiting.step("an idle window did not ask for a frame to draw the picture");
    }
    assert_eq!(
        PICTURE_FRAMES_ASKED.load(Ordering::SeqCst),
        asked + 1,
        "more than one frame was asked for one picture"
    );
    opened.window.hide().expect("close picture window");
}
