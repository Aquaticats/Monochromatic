//! Which `SLINT_BACKEND` values this program installs a backend for,
//!  and which renderer names it refuses.

/// The decision under test.
use super::{BackendChoice, choose_backend};

#[test]
fn an_unset_value_and_the_winit_spellings_are_this_programs() {
    for requested in ["", "winit", "gl", "winit-", "winit-femtovg-wgpu"] {
        assert_eq!(
            choose_backend(requested).expect("accepted spelling"),
            BackendChoice::ThisProgram,
            "{requested:?}"
        );
    }
}

#[test]
fn another_backend_stays_with_slints_selector() {
    for requested in ["qt", "linuxkms", "headless", "winitx", "femtovg-wgpu"] {
        assert_eq!(
            choose_backend(requested).expect("another backend"),
            BackendChoice::Slint,
            "{requested:?}"
        );
    }
}

#[test]
fn a_winit_renderer_this_build_lacks_is_refused_with_the_accepted_spellings() {
    // `femtovg`, `skia`, `software` and `vello` are Slint shorthands for winit with that renderer.
    for requested in [
        "winit-femtovg",
        "femtovg",
        "winit-software",
        "sw",
        "software",
        "skia",
        "winit-skia",
        "vello",
        "winit-femtovg-gl",
    ] {
        let message = choose_backend(requested)
            .expect_err("renderer this build lacks")
            .to_string();
        for needle in [requested, "does not contain", "winit-femtovg-wgpu"] {
            assert!(message.contains(needle), "missing {needle:?} in: {message}");
        }
    }
}
