//! Scrolling inside the overscan tile enters no render,
//! paints no tile,
//! shapes no row,
//! and sets no image;
//!  leaving the tile re-centers it on the tile worker,
//! the window thread only swaps the finished tile in,
//!  and the visible pixels equal a fresh render at that offset.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::{install_gpu_platform, snapshot};
/// Reuse production viewport callbacks,
/// appearance callbacks,
/// and the raster presentation.
use super::{
    AppWindow, State, bind_appearance, bind_viewport, render,
    tile_reads::{same_rows, shown},
};
/// Toolkit windows and component ownership stay on the test's event-loop thread.
use slint::{ComponentHandle, Model, platform::update_timers_and_animations};
/// UI state is shared between the window callbacks and the test.
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

/// Lines of the disposable document;
/// the scrolls in these tests stay far from its end.
const LINES: usize = 5000;
/// Offset the reader starts from,
/// well away from both ends of the document.
pub(super) const START: f32 = 40_000.0;
/// Logical pixels of one small wheel step.
const STEP: f32 = 3.0;
/// Longest wait for the tile worker to answer one re-center;
/// the host can be under heavy load.
const PATIENCE: Duration = Duration::from_secs(10);

/// Five thousand numbered lines,
/// each ending with a line feed.
pub(super) fn document() -> String {
    let mut text = String::new();
    for line in 0..LINES {
        text.push_str(&format!("line {line}\n"));
    }
    return text;
}

/// Open a shown window on `text`,
/// with production callbacks and one completed render.
pub(super) fn open(text: &str) -> (AppWindow, Rc<RefCell<State>>) {
    install_gpu_platform();
    let window = AppWindow::new().expect("headless native source window");
    let state = Rc::new(RefCell::new(State::new(text, None)));
    bind_viewport(&window, &state);
    bind_appearance(&window, &state);
    window.show().expect("show headless source window");
    render(&window, &state);
    update_timers_and_animations();
    await_worker_ready(&state);
    return (window, state);
}

/// Advance timers until the tile worker has started and reported ready,
/// failing after `PATIENCE`.
/// The first shown tile starts the worker,
/// so every scroll in these tests finds it warm.
fn await_worker_ready(state: &Rc<RefCell<State>>) {
    let start = Instant::now();
    loop {
        update_timers_and_animations();
        let ready = match state.borrow().tile_worker.as_ref() {
            Some(worker) => worker.is_ready(),
            None => false,
        };
        if ready {
            return;
        }
        assert!(
            start.elapsed() < PATIENCE,
            "the source tile worker did not report ready within {PATIENCE:?}, after the first tile was shown"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Scroll to `offset` and let the toolkit deliver the viewport-changed callback.
pub(super) fn scroll_to(window: &AppWindow, offset: f32) {
    window.set_scroll_y(-offset);
    update_timers_and_animations();
}

/// Advance timers until no re-center is queued or shown,
/// failing after `PATIENCE`.
/// The displayed tile stays on screen while this waits,
/// so the wait is never a blank frame.
pub(super) fn settle_tile(state: &Rc<RefCell<State>>) {
    let start = Instant::now();
    loop {
        update_timers_and_animations();
        {
            let current = state.borrow();
            if current.tile_job.is_none() && current.scroll_wanted.is_none() {
                return;
            }
        }
        assert!(
            start.elapsed() < PATIENCE,
            "the source tile worker did not finish the re-center within {PATIENCE:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Advance timers until the worker has answered one more request than `before`,
/// failing after `PATIENCE`.
fn until_worker_reply(state: &Rc<RefCell<State>>, before: u64) {
    let start = Instant::now();
    loop {
        update_timers_and_animations();
        if state.borrow().worker_tiles > before {
            return;
        }
        assert!(
            start.elapsed() < PATIENCE,
            "the source tile worker did not answer within {PATIENCE:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Zero the test counters so the assertions count only what follows.
pub(super) fn reset(state: &Rc<RefCell<State>>) {
    let mut current = state.borrow_mut();
    current.renders = 0;
    current.tiles_painted = 0;
    current.rows_shaped = 0;
    current.images_set = 0;
    current.worker_tiles = 0;
    current.worker_rows_shaped = 0;
    current.tiles_applied = 0;
    current.tiles_discarded = 0;
}

/// Renders,
/// painted tiles,
/// shaped rows,
/// and images set on the window thread since the last reset,
/// in that order.
pub(super) fn counts(state: &Rc<RefCell<State>>) -> [u64; 4] {
    let current = state.borrow();
    return [
        current.renders,
        current.tiles_painted,
        current.rows_shaped,
        current.images_set,
    ];
}

/// Tiles the worker answered,
/// rows it shaped,
/// tiles swapped in,
/// and tiles discarded,
/// since the last reset.
pub(super) fn worker_counts(state: &Rc<RefCell<State>>) -> [u64; 4] {
    let current = state.borrow();
    return [
        current.worker_tiles,
        current.worker_rows_shaped,
        current.tiles_applied,
        current.tiles_discarded,
    ];
}

/// Small wheel steps inside the tile enter no render,
/// paint no tile,
/// shape no row,
/// and set no image.
#[test]
fn scrolling_inside_the_overscan_tile_does_no_source_work() {
    let (window, state) = open(&document());
    let height = window.get_viewport_height();
    scroll_to(&window, START);
    settle_tile(&state);
    reset(&state);
    // A frame draws the scene the settled tile handed over, so the GPU counts below start from a drawn tile.
    let _first = snapshot(window.window());
    let gpu = state.borrow().gpu.clone();
    let drawn = gpu.tiles_rendered();
    assert!(drawn > 0, "the settled tile was drawn by the GPU");
    // Half a view height each way stays inside the tile, which holds the view until about 0.75 view heights.
    let steps = (height * 0.5 / STEP) as usize;
    for step in 1..=steps {
        scroll_to(&window, START + step as f32 * STEP);
        // The frame the toolkit draws for this offset samples the same texture; no scene is built or drawn for it.
        let _frame = snapshot(window.window());
    }
    for step in (0..steps).rev() {
        scroll_to(&window, START + step as f32 * STEP);
        let _frame = snapshot(window.window());
    }
    assert_eq!(
        gpu.tiles_rendered(),
        drawn,
        "scrolling inside the overscan tile must not render a scene on the GPU"
    );
    assert!(
        !gpu.has_pending(),
        "scrolling inside the overscan tile must not leave a scene waiting"
    );
    let [renders, tiles, rows, images] = counts(&state);
    assert_eq!(
        renders, 0,
        "scrolling inside the overscan tile must not enter render"
    );
    assert_eq!(
        tiles, 0,
        "scrolling inside the overscan tile must not paint a tile"
    );
    assert_eq!(
        rows, 0,
        "scrolling inside the overscan tile must not shape a row"
    );
    assert_eq!(
        images, 0,
        "scrolling inside the overscan tile must not set an image"
    );
    window.hide().expect("close headless source window");
}

/// Leaving the tile queues one re-center for the worker:
/// the window thread shapes,
/// paints,
/// and builds no image
/// during the scroll,
/// the worker does the work,
/// and the swapped-in image,
/// its offset,
/// and its gutter belong to one tile.
#[test]
fn leaving_the_overscan_tile_re_centers_on_the_worker_thread() {
    let (window, state) = open(&document());
    let height = window.get_viewport_height();
    scroll_to(&window, START);
    settle_tile(&state);
    reset(&state);
    let _first = snapshot(window.window());
    let drawn = state.borrow().gpu.tiles_rendered();
    let target = START + height;
    scroll_to(&window, target);
    let [renders, tiles, rows, images] = counts(&state);
    assert_eq!(renders, 1, "leaving the tile enters render once");
    assert_eq!(
        tiles, 0,
        "the window thread paints no tile for a scroll re-center"
    );
    assert_eq!(
        rows, 0,
        "the window thread shapes no row for a scroll re-center"
    );
    assert_eq!(
        images, 0,
        "the window thread builds no image for a scroll re-center"
    );
    settle_tile(&state);
    let _after = snapshot(window.window());
    assert_eq!(
        state.borrow().gpu.tiles_rendered(),
        drawn + 1,
        "the re-center is drawn by the GPU exactly once"
    );
    let [worker_tiles, worker_rows, applied, discarded] = worker_counts(&state);
    assert_eq!(worker_tiles, 1, "the worker answers exactly one re-center");
    assert_eq!(applied, 1, "the finished tile is swapped in once");
    assert_eq!(discarded, 0, "no fresh tile is discarded");
    let current = state.borrow();
    let count = current.count as u64;
    // The measurement pass keeps every line it shaped, so the worker may find all of the tile's lines in its cache;
    // it must never shape more lines than the tile holds.
    assert!(
        worker_rows < count,
        "the worker shaped {worker_rows} rows of {count}; lines it shares with the previous tile must come from its cache"
    );
    let view = current.shaped.as_ref().expect("swapped-in source view");
    assert_eq!(
        (current.shown.first, current.shown.count),
        (view.viewport.first, view.viewport.count),
        "the gutter and line numbers describe the tile whose rows are shown"
    );
    assert_eq!(
        window.get_row_numbers().row_data(0),
        Some(view.viewport.first as i32 + 1),
        "the first line number belongs to the swapped-in tile"
    );
    let mut visual_rows = 0usize;
    for row in view.rows.iter() {
        visual_rows += row.visual_rows();
    }
    assert_eq!(
        window.get_row_tops().row_count(),
        visual_rows,
        "one gutter entry per visual row of the swapped-in tile"
    );
    assert_eq!(
        window.get_image_y(),
        view.origin,
        "the image offset is the origin of the swapped-in tile"
    );
    drop(current);
    window.hide().expect("close headless source window");
}

/// The re-centered pixels equal a fresh render at the same offset.
#[test]
fn leaving_the_overscan_tile_re_centers_once_with_fresh_pixels() {
    let text = document();
    let (window, state) = open(&text);
    let height = window.get_viewport_height();
    scroll_to(&window, START);
    settle_tile(&state);
    let target = START + height;
    scroll_to(&window, target);
    settle_tile(&state);
    let (fresh_window, fresh) = open(&text);
    scroll_to(&fresh_window, target);
    settle_tile(&fresh);
    assert!(
        same_rows(&shown(&window, &state), &shown(&fresh_window, &fresh)),
        "the visible pixels after scrolling differ from a fresh render at offset {target}"
    );
    window.hide().expect("close headless source window");
    fresh_window
        .hide()
        .expect("close fresh headless source window");
}

/// A tile queued for the worker is discarded when an edit renders the tile synchronously first,
/// and the image then matches a fresh render of the edited text at the same offset.
#[test]
fn an_edit_during_a_re_center_discards_the_worker_result() {
    let text = document();
    let (window, state) = open(&text);
    let height = window.get_viewport_height();
    scroll_to(&window, START);
    settle_tile(&state);
    reset(&state);
    let target = START + height;
    scroll_to(&window, target);
    let edited = format!("edited first line\n{text}");
    let reload = state.borrow().document.prepare_reload(&edited);
    assert!(
        state.borrow_mut().document.apply_reload(reload),
        "the edit is accepted at the current revision"
    );
    render(&window, &state);
    let worker_before = state.borrow().worker_tiles;
    until_worker_reply(&state, worker_before);
    let [_, _, applied, discarded] = worker_counts(&state);
    assert_eq!(
        discarded, 1,
        "the worker result built before the edit is discarded"
    );
    assert_eq!(applied, 0, "no stale result is swapped in");
    let (fresh_window, fresh) = open(&edited);
    scroll_to(&fresh_window, target);
    settle_tile(&fresh);
    assert!(
        same_rows(&shown(&window, &state), &shown(&fresh_window, &fresh)),
        "after an edit during a re-center, the visible pixels differ from a fresh render of the edited text at offset {target}"
    );
    window.hide().expect("close headless source window");
    fresh_window
        .hide()
        .expect("close fresh headless source window");
}

/// A tile worker that stops is reported once,
/// the poll timer goes quiet,
/// and the next scroll re-center starts a fresh worker.
#[test]
fn a_stopped_tile_worker_is_reported_once_and_replaced_on_the_next_re_center() {
    let (window, state) = open(&document());
    let height = window.get_viewport_height();
    scroll_to(&window, START);
    settle_tile(&state);
    // Closing the request channel of the idle worker ends its thread, so the next poll finds it disconnected.
    state
        .borrow_mut()
        .tile_worker
        .as_mut()
        .expect("the first re-center started the worker")
        .close_for_test();
    let start = Instant::now();
    loop {
        // Poll directly: an idle poll timer stops, and the dead worker is then seen by the next poll or submission.
        super::tile::poll(&window, &state);
        if state.borrow().tile_worker.is_none() {
            break;
        }
        assert!(
            start.elapsed() < PATIENCE,
            "the stopped tile worker was not released within {PATIENCE:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(
        window
            .get_error_message()
            .to_string()
            .contains("stopped unexpectedly"),
        "the stopped worker is named in the diagnostics"
    );
    assert!(
        !state.borrow().tile_poll.running(),
        "the poll timer stops when no re-center is queued"
    );
    for _ in 0..5 {
        update_timers_and_animations();
    }
    assert!(
        state.borrow().tile_worker.is_none(),
        "no worker is restarted without a re-center"
    );
    reset(&state);
    scroll_to(&window, START + height);
    settle_tile(&state);
    let [worker_tiles, _, applied, discarded] = worker_counts(&state);
    assert_eq!(worker_tiles, 1, "a fresh worker builds the next re-center");
    assert_eq!(applied, 1, "the fresh worker's tile is swapped in");
    assert_eq!(discarded, 0, "no fresh tile is discarded");
    assert!(
        state.borrow().tile_worker.is_some(),
        "the re-center started a new worker"
    );
    window.hide().expect("close headless source window");
}

/// Measures what a cold tile worker costs,
/// printing one line per process for the measurement log.
/// Each run is a new process,
/// so the first worker and its first font discovery are cold.
#[test]
fn measure_worker_cold_start() {
    let shaper_start = Instant::now();
    let shaper = ide_app::shaped_text::TextShaper::new();
    let shaper_ms = shaper_start.elapsed().as_secs_f64() * 1000.0;
    let fork_start = Instant::now();
    let forked = shaper.fork();
    let fork_ms = fork_start.elapsed().as_secs_f64() * 1000.0;
    drop(forked);
    drop(shaper);
    let open_start = Instant::now();
    let (window, state) = open(&document());
    let ready_ms = open_start.elapsed().as_secs_f64() * 1000.0;
    let height = window.get_viewport_height();
    let first_start = Instant::now();
    scroll_to(&window, START);
    settle_tile(&state);
    let first_ms = first_start.elapsed().as_secs_f64() * 1000.0;
    let warm_start = Instant::now();
    scroll_to(&window, START + height);
    settle_tile(&state);
    let warm_ms = warm_start.elapsed().as_secs_f64() * 1000.0;
    eprintln!(
        "COLD-START shaper_new_ms={shaper_ms:.1} fork_ms={fork_ms:.1} open_ready_ms={ready_ms:.1} first_reply_ms={first_ms:.1} warm_reply_ms={warm_ms:.1}"
    );
    window.hide().expect("close headless source window");
}

/// Dropping a worker whose build is in flight returns at once;
/// the build is held on a channel the test controls,
/// so the bound measures the drop alone and not the build.
#[test]
fn dropping_a_worker_with_a_build_in_flight_does_not_wait_for_it() {
    let (window, state) = open(&document());
    let height = window.get_viewport_height();
    scroll_to(&window, START);
    settle_tile(&state);
    let (release, gate) = std::sync::mpsc::channel();
    *super::tile_build::BUILD_GATE
        .lock()
        .expect("the test build gate is free") = Some(gate);
    scroll_to(&window, START + height);
    // The build starts once the worker takes the job from the gate slot; wait until it has.
    let start = Instant::now();
    while super::tile_build::BUILD_GATE
        .lock()
        .expect("the test build gate is free")
        .is_some()
    {
        update_timers_and_animations();
        assert!(
            start.elapsed() < PATIENCE,
            "the tile worker did not start the held build within {PATIENCE:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    let worker = state
        .borrow_mut()
        .tile_worker
        .take()
        .expect("the worker of the held build");
    // The drop runs on its own thread, so a drop that waits fails the bound below instead of hanging the test.
    let (dropped_sender, dropped_receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        drop(worker);
        let _ = dropped_sender.send(());
    });
    let dropped = dropped_receiver.recv_timeout(Duration::from_millis(250));
    // Release the held build so the worker thread ends its loop whatever the outcome.
    release
        .send(())
        .expect("the held build is still waiting for its release");
    assert!(
        dropped.is_ok(),
        "dropping the worker waited for a build that is still in flight"
    );
    window.hide().expect("close headless source window");
}
