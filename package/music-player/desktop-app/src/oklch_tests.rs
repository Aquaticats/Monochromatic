// What:     Unit tests for `oklch.rs`, pulled in by `#[cfg(test)] #[path = "oklch_tests.rs"] mod tests;`.
// Why:      The theme derivation trusts this module for the color space, gamut mapping and WCAG math, so each is
//           checked against known values and invariants.

use super::*;

/// What: `hue_difference(a, b)` is the shortest angle between two hues in degrees.
/// Why:  Hue is circular, so 359 and 1 are 2 apart.
fn hue_difference(first: f64, second: f64) -> f64 {
    let difference: f64 = (first - second + 540.0).rem_euclid(360.0) - 180.0;
    return difference.abs();
}

#[test]
fn white_and_black_have_the_extreme_lightness_and_no_chroma() {
    let white: Oklch = srgb_to_oklch(Srgb8 { red: 255, green: 255, blue: 255 });
    let black: Oklch = srgb_to_oklch(Srgb8 { red: 0, green: 0, blue: 0 });
    assert!((white.lightness - 1.0).abs() < 1e-6, "white lightness {}", white.lightness);
    assert!(white.chroma < 1e-6, "white chroma {}", white.chroma);
    assert!(black.lightness.abs() < 1e-9, "black lightness {}", black.lightness);
}

#[test]
fn known_reference_colors_match_the_published_oklch() {
    // Reference values from the CSS Color 4 / Ottosson OKLab definition: sRGB red is oklch(0.628 0.2577 29.23).
    let red: Oklch = srgb_to_oklch(Srgb8 { red: 255, green: 0, blue: 0 });
    assert!((red.lightness - 0.6280).abs() < 0.001, "red lightness {}", red.lightness);
    assert!((red.chroma - 0.2577).abs() < 0.001, "red chroma {}", red.chroma);
    assert!(hue_difference(red.hue, 29.23) < 0.1, "red hue {}", red.hue);
    // sRGB blue is oklch(0.452 0.3132 264.05).
    let blue: Oklch = srgb_to_oklch(Srgb8 { red: 0, green: 0, blue: 255 });
    assert!((blue.lightness - 0.4520).abs() < 0.001, "blue lightness {}", blue.lightness);
    assert!((blue.chroma - 0.3132).abs() < 0.001, "blue chroma {}", blue.chroma);
}

#[test]
fn every_srgb_step_of_a_channel_sweep_round_trips() {
    for red in (0..=255u16).step_by(5) {
        for green in (0..=255u16).step_by(15) {
            for blue in (0..=255u16).step_by(15) {
                let color: Srgb8 = Srgb8 { red: red as u8, green: green as u8, blue: blue as u8 };
                let back: Srgb8 = oklch_to_srgb8(srgb_to_oklch(color));
                assert_eq!(back, color, "round trip of {color:?}");
            }
        }
    }
}

#[test]
fn gamut_map_keeps_lightness_and_hue_and_lands_inside_the_gamut() {
    let mut hue: f64 = 0.0;
    while hue < 360.0 {
        for lightness in [0.1, 0.3, 0.5, 0.7, 0.9] {
            for chroma in [0.05, 0.2, 0.4] {
                let wanted: Oklch = Oklch { lightness, chroma, hue };
                let mapped: Oklch = gamut_map(wanted);
                assert!(is_in_gamut(mapped), "{wanted:?} mapped to {mapped:?} is outside sRGB");
                assert_eq!(mapped.lightness, wanted.lightness);
                assert_eq!(mapped.hue, wanted.hue);
                assert!(mapped.chroma <= wanted.chroma + 1e-12);
            }
        }
        hue += 10.0;
    }
}

#[test]
fn gamut_map_reduces_chroma_to_the_boundary_and_not_further() {
    // Lightness 0.5, chroma 0.4, hue 140 is far outside sRGB, so the result is strictly smaller and maximal.
    let wanted: Oklch = Oklch { lightness: 0.5, chroma: 0.4, hue: 140.0 };
    let mapped: Oklch = gamut_map(wanted);
    assert!(mapped.chroma < 0.4);
    assert!(is_in_gamut(mapped));
    assert!(!is_in_gamut(Oklch { chroma: mapped.chroma + 1e-3, ..mapped }), "chroma could still grow");
}

#[test]
fn gamut_map_leaves_an_in_gamut_color_alone() {
    let wanted: Oklch = Oklch { lightness: 0.6, chroma: 0.05, hue: 200.0 };
    assert_eq!(gamut_map(wanted), wanted);
}

#[test]
fn gamut_map_turns_lightness_at_the_ends_into_black_and_white() {
    assert_eq!(oklch_to_srgb8(Oklch { lightness: 1.2, chroma: 0.3, hue: 90.0 }), Srgb8 { red: 255, green: 255, blue: 255 });
    assert_eq!(oklch_to_srgb8(Oklch { lightness: -0.1, chroma: 0.3, hue: 90.0 }), Srgb8 { red: 0, green: 0, blue: 0 });
}

#[test]
fn channel_clipping_would_have_shifted_the_hue_that_gamut_mapping_keeps() {
    // Positive control: a naive clip of an out-of-gamut color changes its hue by a visible amount.
    let wanted: Oklch = Oklch { lightness: 0.8, chroma: 0.3, hue: 150.0 };
    let linear: [f64; 3] = oklch_to_linear(wanted);
    let clipped: Srgb8 = Srgb8 {
        red: srgb_encode(linear[0]),
        green: srgb_encode(linear[1]),
        blue: srgb_encode(linear[2]),
    };
    let clipped_hue: f64 = srgb_to_oklch(clipped).hue;
    assert!(hue_difference(clipped_hue, wanted.hue) > 2.0, "clipped hue {clipped_hue}");
    let mapped_hue: f64 = srgb_to_oklch(oklch_to_srgb8(wanted)).hue;
    assert!(hue_difference(mapped_hue, wanted.hue) < 2.0, "mapped hue {mapped_hue}");
}

#[test]
fn contrast_of_white_on_black_is_twenty_one_and_symmetric() {
    let white: Srgb8 = Srgb8 { red: 255, green: 255, blue: 255 };
    let black: Srgb8 = Srgb8 { red: 0, green: 0, blue: 0 };
    assert!((contrast_ratio(white, black) - 21.0).abs() < 1e-9);
    assert_eq!(contrast_ratio(white, black), contrast_ratio(black, white));
    assert!((contrast_ratio(white, white) - 1.0).abs() < 1e-12);
}

#[test]
fn contrast_matches_a_known_wcag_pair() {
    // #767676 on white is the classic 4.54:1 pair.
    let gray: Srgb8 = Srgb8 { red: 0x76, green: 0x76, blue: 0x76 };
    let white: Srgb8 = Srgb8 { red: 255, green: 255, blue: 255 };
    assert!((contrast_ratio(gray, white) - 4.54).abs() < 0.01);
}

#[test]
fn mix_takes_the_short_way_round_the_hue_circle() {
    let from: Oklch = Oklch { lightness: 0.4, chroma: 0.1, hue: 350.0 };
    let to: Oklch = Oklch { lightness: 0.8, chroma: 0.1, hue: 10.0 };
    let middle: Oklch = mix_oklch(from, to, 0.5);
    assert!((middle.lightness - 0.6).abs() < 1e-12);
    assert!(hue_difference(middle.hue, 0.0) < 1e-9, "hue {}", middle.hue);
}

#[test]
fn mix_with_an_achromatic_end_keeps_the_other_hue() {
    let from: Oklch = Oklch { lightness: 0.4, chroma: 0.1, hue: 120.0 };
    let to: Oklch = Oklch { lightness: 1.0, chroma: 0.0, hue: 0.0 };
    let middle: Oklch = mix_oklch(from, to, 0.5);
    assert!((middle.hue - 120.0).abs() < 1e-9);
    assert!((middle.chroma - 0.05).abs() < 1e-12);
}

#[test]
fn composite_over_blends_by_alpha_and_covers_the_ends() {
    let layer: Srgb8 = Srgb8 { red: 200, green: 100, blue: 0 };
    let surface: Srgb8 = Srgb8 { red: 0, green: 0, blue: 0 };
    assert_eq!(composite_over(layer, 1.0, surface), layer);
    assert_eq!(composite_over(layer, 0.0, surface), surface);
    assert_eq!(composite_over(layer, 0.5, surface), Srgb8 { red: 100, green: 50, blue: 0 });
}

#[test]
fn delta_e_is_zero_for_equal_colors_and_large_for_black_against_white() {
    let white: Srgb8 = Srgb8 { red: 255, green: 255, blue: 255 };
    let black: Srgb8 = Srgb8 { red: 0, green: 0, blue: 0 };
    assert_eq!(delta_e_ok(white, white), 0.0);
    assert!((delta_e_ok(white, black) - 100.0).abs() < 0.01);
}

#[test]
fn an_equal_channel_color_reads_as_exactly_achromatic() {
    // D118: all channels equal means no chroma at all; the ten-digit matrix would otherwise read about 4e-8.
    for level in 0..=255_u8 {
        let lch: Oklch = srgb_to_oklch(Srgb8 { red: level, green: level, blue: level });
        assert_eq!(lch.chroma, 0.0, "gray {level}");
    }
    // Positive control: one step on one channel is not achromatic.
    assert!(srgb_to_oklch(Srgb8 { red: 129, green: 128, blue: 128 }).chroma > 1.0e-3);
}

#[test]
fn zero_chroma_always_encodes_to_equal_channels_even_on_a_rounding_boundary() {
    // Walk lightness through 20000 fine steps and through the exact doubles on each side of every 8-bit boundary of
    // the gray ramp: a zero-chroma color never comes out with unequal channels.
    for step in 0..=20_000_u32 {
        let lightness: f64 = f64::from(step) / 20_000.0;
        let color: Srgb8 = oklch_to_srgb8(Oklch { lightness, chroma: 0.0, hue: 123.0 });
        assert!(color.red == color.green && color.green == color.blue, "L {lightness}: {color:?}");
    }
    for level in 0..255_u32 {
        let boundary_encoded: f64 = (f64::from(level) + 0.5) / 255.0;
        let linear: f64 = if boundary_encoded <= 0.04045 { boundary_encoded / 12.92 } else { ((boundary_encoded + 0.055) / 1.055).powf(2.4) };
        let boundary_lightness: f64 = linear.cbrt();
        for nudge in [-1.0e-9, -1.0e-12, 0.0, 1.0e-12, 1.0e-9] {
            let color: Srgb8 = oklch_to_srgb8(Oklch { lightness: boundary_lightness + nudge, chroma: 0.0, hue: 0.0 });
            assert!(color.red == color.green && color.green == color.blue, "level {level} nudge {nudge}: {color:?}");
        }
    }
}
