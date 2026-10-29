//! A tile the painter refuses never blanks the source:
//!  the window thread keeps the picture last painted and names the failure,
//!  and a job the worker failed is freed so the next placement asks for the tile again,
//!  and a tile the shaper had to cut carries a stamp that names the clip it was cut to.
//! Nothing here waits on the window thread:
//! every wait advances timers and lets the workers answer.

/// The reader fixture,
/// its wrapped text,
/// the pixels of the shown image,
/// and the waits.
use super::{
    AppWindow, State,
    overscan_tile_tests::scroll_to,
    render::{Paint, job, stamp_for},
    tall_window_tests::{hold_the_pass, mixed, resize_without_frame, settled_tile},
    theme_tests::switch,
    tile::{TileBounds, apply, requested},
    tile_build::{FAIL_NEXT_BUILD, build_tile},
    tile_reads::shown,
    wrap_tests::{long_text, open, settled},
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{Facts, Waiting};
/// Toolkit events.
use slint::{
    ComponentHandle, platform::update_timers_and_animations,
    private_unstable_api::re_exports::ColorScheme,
};
/// The switch is read by the next build.
use std::{cell::RefCell, rc::Rc, sync::atomic::Ordering};

/// Whether any pixel of the visible band of the tile has ink.
fn has_ink(window: &AppWindow, source: &Rc<RefCell<State>>) -> bool {
    if source.borrow().gpu.read_tile().is_none() && !source.borrow().gpu.has_pending() {
        return false;
    }
    for pixel in shown(window, source) {
        if pixel[3] > 0 {
            return true;
        }
    }
    return false;
}

/// A tile the painter refuses on the window thread leaves the text last painted on screen,
/// names the failure,
/// and the next render builds the tile again.
#[test]
fn a_refused_tile_keeps_the_text_on_screen_and_names_the_failure() {
    let (_project, opened) = open(&long_text(200));
    let window = &opened.window;
    settled(&opened);
    assert!(has_ink(window, &opened.source), "the first tile has no ink");
    assert_eq!(window.get_error_message().to_string(), "");
    // The scheme the window starts in is not known, so the first change makes sure the next one differs.
    switch(window, ColorScheme::Dark);
    settled(&opened);
    FAIL_NEXT_BUILD.store(true, Ordering::SeqCst);
    // A new color scheme repaints the tile on the window thread.
    switch(window, ColorScheme::Light);
    let message = window.get_error_message().to_string();
    assert!(
        message.contains("Cannot render source"),
        "the failure is not named: {message:?}"
    );
    assert!(
        has_ink(window, &opened.source),
        "a refused tile blanked the source"
    );
    window.hide().expect("close wrap window");
}

/// The reply of a failed worker build frees the job the window waits for and names the failure:
/// the window stops waiting for a tile that will not come,
/// and the same placement can be asked for again.
/// The reply is applied directly,
/// because a later render can ask for the same tile again in the very timer pass that took the reply,
/// and a test that polls would sometimes see that second request instead of the freed job.
#[test]
fn a_failed_worker_reply_frees_its_job_and_names_the_failure() {
    let (_project, opened) = open(&long_text(800));
    let window = &opened.window;
    settled(&opened);
    // The job is held as the window holds it while the worker builds.
    {
        let mut current = opened.source.borrow_mut();
        let wanted = requested(&current);
        current.tile_job = Some(wanted);
    }
    apply(
        window,
        &opened.source,
        Err(anyhow::anyhow!("the painter refused the tile")),
    );
    assert!(
        opened.source.borrow().tile_job.is_none(),
        "the failed job is still held {}",
        Facts {
            window,
            source: &opened.source
        }
    );
    let failures = opened.source.borrow().worker_failures.clone();
    assert!(
        failures.len() == 1 && failures[0].contains("Cannot render source"),
        "the failed worker tile was not named once: {failures:?}"
    );
    assert!(
        window
            .get_error_message()
            .to_string()
            .contains("the painter refused the tile"),
        "the window does not show the failure: {:?}",
        window.get_error_message()
    );
    window.hide().expect("close wrap window");
}

/// A tile the painter refuses on the worker leaves the displayed text on screen,
/// and the view still gets its tile afterwards:
/// the failure is reported once,
/// the next request of the same tile is built,
/// the poll stops once nothing is in flight,
/// and the text is not blank.
/// The test waits for these end states,
/// because a render that follows the failure may ask for the tile again within the same timer pass.
#[test]
fn a_view_whose_worker_tile_failed_still_gets_its_tile() {
    let (_project, opened) = open(&long_text(800));
    let window = &opened.window;
    settled(&opened);
    FAIL_NEXT_BUILD.store(true, Ordering::SeqCst);
    // Far enough to leave the tile, so the worker builds the next one.
    scroll_to(window, 12_000.0);
    {
        // The window's message is cleared by the next render, which can follow the failure within the same timer
        // pass, so the test waits for the failure the state recorded when it named it.
        let mut waiting = Waiting::slow();
        while !(!opened.source.borrow().worker_failures.is_empty()) {
            waiting.step(format_args!(
                "the failed worker tile was not reported {}",
                Facts {
                    window,
                    source: &opened.source
                }
            ));
        }
    }
    assert!(
        opened.source.borrow().worker_failures[0].contains("Cannot render source"),
        "the failed worker tile was not named: {:?}",
        opened.source.borrow().worker_failures
    );
    // One more step of the view asks for the tile again if nothing has yet, and the tile arrives.
    scroll_to(window, 12_030.0);
    {
        let mut waiting = Waiting::slow();
        loop {
            update_timers_and_animations();
            {
                let current = opened.source.borrow();
                if current.tile_job.is_none()
                    && current.scroll_wanted.is_none()
                    && current.shown.first > 50
                {
                    break;
                }
            }
            waiting.step(format_args!(
                "the tile after a failed one did not arrive {}",
                Facts {
                    window,
                    source: &opened.source
                }
            ));
        }
    }
    {
        let mut waiting = Waiting::slow();
        while !(!opened.source.borrow().tile_poll.running()) {
            update_timers_and_animations();
            waiting.step(format_args!(
                "the poll did not stop once the tile had arrived {}",
                Facts {
                    window,
                    source: &opened.source
                }
            ));
        }
    }
    assert_eq!(
        opened.source.borrow().worker_failures.len(),
        1,
        "the one refused build was reported more than once"
    );
    assert!(
        has_ink(window, &opened.source),
        "the text is blank after the retry"
    );
    window.hide().expect("close wrap window");
}

/// A tile the shaper cuts to the raster limit is stamped with the clip it was cut to:
/// the window compares that stamp to the inputs it would paint with now,
/// and a stamp of the clip that was asked for would discard every cut tile,
/// however fresh.
#[test]
fn a_tile_cut_by_the_shaper_is_stamped_with_the_clip_it_was_cut_to() {
    let (_project, opened) = open(&mixed(6000));
    let release = hold_the_pass();
    let window = &opened.window;
    resize_without_frame(&opened, 1330.0, 3000.0);
    settled_tile(&opened);
    {
        let mut current = opened.source.borrow_mut();
        let paint = Paint::new(window, &current);
        let asked = TileBounds {
            first: 0,
            count: 2000,
            clip: None,
            ..current.shown
        };
        let queued = job(&current, asked, &paint);
        let built = build_tile(&mut current.shaper, queued).expect("the cut tile is painted");
        assert_eq!(built.requested, asked, "the tile names the job it answers");
        assert!(
            built.bounds.clip.is_some(),
            "the shaper did not cut a tile of two thousand wrapped lines"
        );
        assert!(
            stamp_for(&current, built.bounds, &paint) == built.stamp,
            "the stamp of a cut tile does not name the clip it was cut to"
        );
    }
    drop(release);
    window.hide().expect("close wrap window");
}
