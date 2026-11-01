//! Logical size,
//!  output scale,
//!  and the physical framebuffer they produce,
//!  without a display.

/// The same geometry the compositor applies for `--size`,
///  `--scale`,
///  `resize`,
///  and `scale`.
use nested_wayland_session::screen_geometry::{OutputScale, ScreenGeometry};

/// Parse a scale the test knows is valid;
///  a parse failure stops the test with its message.
fn scale(text: &str) -> OutputScale {
    return OutputScale::parse(text).expect(text);
}

/// Build the 1100x660 screen the IDE inspection uses,
///  at one scale.
fn screen(text: &str) -> ScreenGeometry {
    return ScreenGeometry { logical_width: 1100, logical_height: 660, scale: scale(text) };
}

/// Integer and fractional KDE scales become exact 120ths,
///  the unit clients receive.
#[test]
fn scale_text_becomes_exact_120ths() {
    for (text, per_120) in [
        ("1", 120), ("1.0", 120), ("1.25", 150), ("1.5", 180), ("1.75", 210),
        ("2", 240), ("3", 360), ("0.5", 60), ("1.05", 126), ("1.3333", 160),
    ] {
        assert_eq!(scale(text).per_120(), per_120, "{text}");
    }
    assert_eq!(OutputScale::ONE, scale("1"));
}

/// Values outside KDE's range,
///  between 120ths,
///  or not numbers never reach a client.
#[test]
fn scale_text_outside_the_grammar_is_rejected() {
    for text in [
        "", " ", "0", "0.25", "0.49", "3.01", "4", "-1", "1.333", "1.01",
        "NaN", "inf", "-inf", "1,25", "125%", "1.25x", "0x2", "1.25 2", "1.25\n", "two",
        "99999999999999999999999",
    ] {
        let message = OutputScale::parse(text).expect_err(text);
        assert!(message.contains("scale"), "{text:?}: {message}");
    }
    // The refusal between steps names the two scales the caller could have meant.
    let between = OutputScale::parse("1.333").expect_err("1.333");
    assert!(between.contains("1.3250") && between.contains("1.3333"), "{between}");
}

/// What clients are told:
///  `wl_output.scale` rounds up,
///  the fractional scale stays exact.
#[test]
fn clients_are_told_the_rounded_up_integer_and_the_exact_fraction() {
    for (text, integer, fraction) in [
        ("1", 1, 1.0), ("1.25", 2, 1.25), ("1.5", 2, 1.5), ("2", 2, 2.0), ("0.5", 1, 0.5), ("3", 3, 3.0),
    ] {
        let advertised = scale(text).advertised();
        assert_eq!(advertised.integer_scale(), integer, "{text}");
        assert_eq!(advertised.fractional_scale(), fraction, "{text}");
        // Smithay sends round(fraction * 120); the stored numerator must survive that trip.
        assert_eq!((advertised.fractional_scale() * 120.0).round() as u32, scale(text).per_120(), "{text}");
    }
}

/// Screenshots are the logical size times the scale,
///  rounded half away from zero.
#[test]
fn physical_size_at_each_scale() {
    assert_eq!(screen("1").physical_size(), (1100, 660));
    assert_eq!(screen("2").physical_size(), (2200, 1320));
    assert_eq!(screen("1.25").physical_size(), (1375, 825));
    assert_eq!(screen("1.5").physical_size(), (1650, 990));
    // 853 * 1.5 = 1279.5 rounds up, as the protocol prescribes for toplevels.
    let half = ScreenGeometry { logical_width: 853, logical_height: 481, scale: scale("1.5") };
    assert_eq!(half.physical_size(), (1280, 722));
}

/// A scale switch keeps the configured logical size,
///  so the client is not resized.
#[test]
fn scale_switch_keeps_the_logical_size() {
    let before = screen("1");
    let after = ScreenGeometry { scale: scale("2"), ..before };
    assert_eq!((after.logical_width, after.logical_height), (1100, 660));
    assert_ne!(after.physical_size(), (after.logical_width, after.logical_height));
}

/// The parent window is asked for its own logical pixels,
///  rounded up,
///  never fewer pixels.
#[test]
fn parent_request_rounds_up_to_cover_the_physical_size() {
    assert_eq!(screen("1.25").parent_request(2.0), (688, 413));
    assert_eq!(screen("1.25").parent_request(3.0), (459, 275));
    assert_eq!(screen("1.25").parent_request(1.25), (1100, 660));
    assert_eq!(screen("1.25").parent_request(1.0), (1375, 825));
    assert_eq!(screen("2").parent_request(2.0), (1100, 660));
    // A parent that has not reported a scale divides by 1, never by zero.
    assert_eq!(screen("1").parent_request(0.0), (1100, 660));
    assert_eq!(screen("1").parent_request(f64::NAN), (1100, 660));
    for parent_scale in [1.0, 1.25, 1.5, 2.0, 3.0] {
        let (width, height) = screen("1.25").parent_request(parent_scale);
        // The parent rounds its logical size times its scale; that must cover 1375x825.
        assert!((f64::from(width) * parent_scale).round() >= 1375.0, "{parent_scale}");
        assert!((f64::from(height) * parent_scale).round() >= 825.0, "{parent_scale}");
    }
}

/// Parent rounding keeps the screen;
///  a real parent resize changes the logical size.
#[test]
fn parent_resize_distinguishes_rounding_from_a_new_size() {
    let current = screen("1.25");
    assert!(current.fits_framebuffer((1376, 826), 2.0));
    assert!(current.fits_framebuffer((1377, 825), 3.0));
    assert!(!current.fits_framebuffer((1378, 825), 2.0));
    assert_eq!(current.after_parent_resize((1376, 826), 2.0), current);
    let resized = current.after_parent_resize((2000, 1000), 2.0);
    assert_eq!((resized.logical_width, resized.logical_height), (1600, 800));
    assert_eq!(resized.scale, current.scale);
    // A degenerate framebuffer still yields a drawable screen.
    let tiny = current.after_parent_resize((0, 0), 1.0);
    assert_eq!((tiny.logical_width, tiny.logical_height), (1, 1));
}
