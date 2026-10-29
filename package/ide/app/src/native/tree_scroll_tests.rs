//! A far reveal must place the whole target row inside the real native tree viewport.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::install_gpu_platform;
/// Exercise the shipped project component rather than a separate scrolling fixture.
use super::{AppWindow, tree_pointer_tests::sixty_entries};
/// Native model and snapshot APIs force the toolkit to materialize its displayed row window.
use slint::{ComponentHandle, ModelRc, VecModel, platform::update_timers_and_animations};
/// Owned model rows stay alive across programmatic viewport movement.
use std::rc::Rc;

/// Large forward/backward jumps and the last row must retain fractional viewport remainders.
#[test]
fn far_tree_reveal_keeps_the_entire_target_row_visible() {
    install_gpu_platform();
    let window = AppWindow::new().expect("native tree window");
    window.set_project_visible(true);
    window.set_source_available(false);
    let entries = sixty_entries("source");
    window.set_tree_entries(ModelRc::from(Rc::new(VecModel::from(entries))));
    window.show().expect("show native tree");
    window
        .window()
        .take_snapshot()
        .expect("initial row geometry");
    for index in [43, 59, 2] {
        window.invoke_reveal_tree(index);
        update_timers_and_animations();
        window
            .window()
            .take_snapshot()
            .expect("revealed row geometry");
        let top = window.get_tree_scroll_y() + index as f32 * 48.0;
        let bottom = top + 48.0;
        assert!(
            top >= -0.1,
            "row {index} starts outside the viewport: {top}"
        );
        assert!(
            bottom <= window.get_tree_viewport_height() + 0.1,
            "row {index} is only partially revealed: bottom {bottom}, viewport {}",
            window.get_tree_viewport_height()
        );
    }
    window.hide().expect("close tree window");
}
