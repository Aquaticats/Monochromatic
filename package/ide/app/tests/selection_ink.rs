//! Selected-text ink follows the selection background's lightness in both color schemes.

/// What:
///  Import the production contrast helpers through the library's public interface.
/// Why:
///  The native renderer calls exactly these functions;
///  the tests must not use a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { WHITE_MINIMUM, contrast, legibleInk } from 'ide-app/selection-ink';
/// ```
use ide_app::selection_ink::{WHITE_MINIMUM, contrast, legible_ink};

/// What:
///  `const` names a compile-time value;
///  `[u8; 4]` is a fixed four-byte color (red,
///  green,
///  blue,
///  alpha).
/// Why:
///  The toolkit's fluent selection background is the same in the dark and the light scheme.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FLUENT_SELECTION = [0x00, 0x78, 0xd4, 0xff] as const;
/// ```
const FLUENT_SELECTION: [u8; 4] = [0x00, 0x78, 0xD4, 0xFF];

/// The toolkit's dark-scheme selection ink.
const BLACK: [u8; 4] = [0, 0, 0, 255];

/// The toolkit's light-scheme selection ink.
const WHITE: [u8; 4] = [255, 255, 255, 255];

/// The dark scheme's black palette ink is replaced by white on the fluent selection background.
#[test]
fn dark_scheme_selection_ink_is_light_on_the_fluent_selection_background() {
    let ink = legible_ink(FLUENT_SELECTION, BLACK);
    assert_eq!(
        ink, WHITE,
        "dark-scheme selected text stayed dark on the selection background"
    );
    let ratio = contrast(ink, FLUENT_SELECTION);
    assert!(
        ratio >= 4.5,
        "selected ink has only {ratio}:1 against the selection background"
    );
    // The WCAG 2 ratio alone does not explain the choice: the rejected black ink scores slightly higher.
    let rejected = contrast(BLACK, FLUENT_SELECTION);
    assert!(
        (rejected - 4.64).abs() < 0.01 && (ratio - 4.53).abs() < 0.01,
        "measured ratios changed: black {rejected}, white {ratio}"
    );
}

/// The light scheme keeps the white ink it already had.
#[test]
fn light_scheme_selection_ink_is_unchanged() {
    assert_eq!(legible_ink(FLUENT_SELECTION, WHITE), WHITE);
}

/// A light selection background,
///  such as the fluent dark accent,
///  gets dark ink with a high ratio.
#[test]
fn light_selection_background_gets_dark_ink() {
    let background = [0x60, 0xCD, 0xFF, 0xFF];
    let ink = legible_ink(background, WHITE);
    assert_eq!(ink, BLACK, "white ink was kept on a light background");
    assert!(contrast(ink, background) >= 7.0);
}

/// Across every gray the ink switches exactly once and never falls below the documented minimum.
#[test]
fn every_gray_background_keeps_the_minimum_ratio_and_switches_once() {
    let mut switches = 0;
    let mut previous = legible_ink([0, 0, 0, 255], BLACK);
    // What: `0..=255_u8` counts every byte value including 255; `_u8` fixes the counter's type.
    // Why: The rule must hold for every lightness, not only for the palette colors above.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (let level = 0; level <= 255; level++) { ... }
    // ```
    for level in 0..=255_u8 {
        let background = [level, level, level, 255];
        let ink = legible_ink(background, BLACK);
        let ratio = contrast(ink, background);
        if ink == WHITE {
            assert!(
                ratio >= WHITE_MINIMUM,
                "white ink at gray {level} has only {ratio}:1"
            );
        } else {
            assert!(ratio >= 7.0, "black ink at gray {level} has only {ratio}:1");
        }
        if ink != previous {
            switches += 1;
        }
        previous = ink;
    }
    assert_eq!(
        switches, 1,
        "the ink must change from white to black exactly once"
    );
}

/// A translucent background has no known lightness,
///  so the toolkit's own ink is kept.
#[test]
fn translucent_selection_background_keeps_the_palette_ink() {
    assert_eq!(legible_ink([0x00, 0x78, 0xD4, 0x4D], BLACK), BLACK);
    assert_eq!(legible_ink([0x00, 0x78, 0xD4, 0x4D], WHITE), WHITE);
}
