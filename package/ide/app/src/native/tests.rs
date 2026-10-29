//! Real Slint window events must invalidate source pixels even when source and logical size are unchanged.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::install_gpu_platform;
/// Reuse production initialization,
///  viewport callbacks,
///  and raster presentation.
use super::{AppWindow, State, bind_appearance, bind_viewport, render, tile_reads::tile_size};
/// Native toolkit events and component ownership are exercised without a host window.
use slint::{
    ComponentHandle,
    platform::{WindowEvent, update_timers_and_animations},
};
/// UI state remains on the test's event-loop thread.
use std::{cell::RefCell, rc::Rc};

/// Moving to a different display scale must not leave an old-resolution bitmap until the next keypress.
#[test]
fn source_raster_tracks_scale_without_input_or_resize() {
    install_gpu_platform();
    let window = AppWindow::new().expect("headless native source window");
    let state = Rc::new(RefCell::new(State::new("=== and 猫", None)));
    bind_viewport(&window, &state);
    bind_appearance(&window, &state);
    window.show().expect("show headless source window");
    render(&window, &state);
    update_timers_and_animations();
    let before = tile_size(&window, &state);
    assert!(before.0 > 0 && before.1 > 0);
    let original_scale = window.window().scale_factor();
    for factor in [2.0, 1.25, 1.0] {
        window
            .window()
            .dispatch_event(WindowEvent::ScaleFactorChanged {
                scale_factor: original_scale * factor,
            });
        update_timers_and_animations();
        let after = tile_size(&window, &state);
        assert_eq!(
            after.0,
            (before.0 as f32 * factor).ceil() as u32,
            "source texture must follow scale without a resize or reading-state change"
        );
        assert_eq!(after.1, (before.1 as f32 * factor).ceil() as u32);
    }
    window.hide().expect("close headless source window");
}
