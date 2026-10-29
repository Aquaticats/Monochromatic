//! Typography changes pass through the actual source-image,
//!  selection,
//!  and toolkit snapshot bindings.

/// Reuse the production window and rendering state rather than a lookalike Text widget.
use super::{
    AppWindow, State,
    gpu_snapshot::{install_gpu_platform, snapshot},
    gpu_tile, render,
    tile_reads::RENDER_NOISE,
};
/// Source positions stay independent of the rendered font instance.
use ide_app::document::ReadingPosition;
/// Construct immutable real-face requests through the public source API.
use ide_app::{shaped_text::TextShaper, source_typography::SourceTypography};
/// Flush pending property changes before taking the consumer's displayed pixels.
use slint::{ComponentHandle, platform::update_timers_and_animations};
/// What:
///  Rc owns UI-thread shared state;
///  RefCell checks temporary exclusive borrows.
/// Why:
///  Unlike Arc/Mutex,
///  these match the production single-threaded callback boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = { current: new State(source) };
/// ```
use std::{cell::RefCell, rc::Rc};

/// Whether two window captures agree byte for byte to within the GPU's render noise of one level.
fn same_frame(left: &[u8], right: &[u8]) -> bool {
    return left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(one, other)| return one.abs_diff(*other) <= RENDER_NOISE);
}

/// Install a new immutable shaper,
///  invalidate its frame,
///  and capture the real source view.
fn capture(window: &AppWindow, state: &Rc<RefCell<State>>, weight: f32, italic: bool) -> Vec<u8> {
    // What: borrow_mut lends the state exclusively until this inner block ends.
    // Why: Release the borrow before toolkit setters can invoke rendering callbacks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { const current = state.current; current.shaper = new TextShaper(settings); }
    // ```
    {
        let mut current = state.borrow_mut();
        // What: .. fills unchanged fields from the default settings; expect fails the test on error.
        // Why: Exercise weight and face changes without changing the default ligature policy.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // current.shaper = new TextShaper({ ...defaults, weight, italic });
        // ```
        current.shaper = TextShaper::with_typography(SourceTypography {
            weight,
            italic,
            ..SourceTypography::default()
        })
        .expect("valid source font instance");
        // What: None clears the optional previous frame identity rather than retaining stale pixels.
        // Why: A new immutable shaper is a new layout source even when document bytes match.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // current.frameStamp = undefined;
        // ```
        current.frame_stamp = None;
    }
    // Lend the existing production state and window rather than constructing another renderer.
    render(window, state);
    update_timers_and_animations();
    // The frame is drawn on the GPU and read back; as_bytes lends pixels, and to_vec owns them across later frames.
    return snapshot(window.window()).as_bytes().to_vec();
}

/// Rendering real italic/intermediate instances must retain the same selected source character.
#[test]
fn source_typography_repaints_without_changing_selection() {
    // The window is drawn by FemtoVG on the hardware GPU; no software renderer can stand in for it.
    println!("GPU snapshot adapter: {}", install_gpu_platform());
    // Expect makes a toolkit initialization error a test failure instead of substituting another backend.
    let window = AppWindow::new().expect("headless source window");
    // Rc/RefCell shares the same production state with each render; None selects an in-memory fixture.
    let state = Rc::new(RefCell::new(State::new("=== j affine 猫", None)));
    state.borrow_mut().document.select(ReadingPosition {
        anchor: 1,
        head: 2,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    // The source is drawn by the GPU notifier, which `bind_viewport` installs in the application.
    gpu_tile::bind(&window, &state.borrow().gpu);
    window.show().expect("show source typography fixture");
    // The first frame draws the tile's textures and hands the window its list of images, which the frame after it
    // shows: a frame of the same weight is taken first and not compared.
    capture(&window, &state, 400.0, false);
    // Lend the window/state while each independent pixel buffer remains owned by the test.
    let regular = capture(&window, &state, 400.0, false);
    let weighted = capture(&window, &state, 527.5, false);
    assert!(
        regular != weighted,
        "source weight must change displayed ink"
    );
    let italic = capture(&window, &state, 527.5, true);
    assert!(
        weighted != italic,
        "source italic request must change displayed ink"
    );
    assert_eq!(window.get_selected_text(), "=");
    assert_eq!(window.get_selection_anchor(), 1);
    assert_eq!(window.get_selection_head(), 2);
    // Two renders of one scene differ by one level in a few texels on the GPU.
    assert!(same_frame(
        &capture(&window, &state, 400.0, false),
        &regular
    ));
    window.hide().expect("close source typography fixture");
}
