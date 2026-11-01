// What:     Unit tests for `theme_roles.rs`, pulled in by `#[cfg(test)] #[path = "theme_roles_tests.rs"] mod tests;`.
// Why:      The derivation promises AA contrast and in-range colors for ANY accent, so it is swept over hues every
//           10 degrees and a range of chromas in both schemes, and the baseline root is compared to the old fixed values.

use super::*;
use crate::oklch::{Oklch, delta_e_ok, is_in_gamut, oklch_to_linear, oklch_to_srgb8, srgb_to_oklch};

/// Hex shorthand for the expected values.
fn hex(value: u32) -> Srgb8 {
    return Srgb8 { red: ((value >> 16) & 0xFF) as u8, green: ((value >> 8) & 0xFF) as u8, blue: (value & 0xFF) as u8 };
}

/// What: the fixed values `ui/theme.slint` held before D108, per role, as `(name, dark, light)`.
/// Why:  the baseline root must reproduce them closely (the old captures are the visual reference).
fn old_fixed_values() -> Vec<(&'static str, u32, u32)> {
    return vec![
        ("canvas", 0x000000, 0xFFFFFF),
        ("deck", 0x0A0A0D, 0xF4F3FA),
        ("current-row", 0x121318, 0xF4F3FA),
        ("on-surface", 0xE4E5F0, 0x30323B),
        ("on-surface-variant", 0xC5C6D0, 0x5C5E6A),
        ("outline", 0x8E9099, 0x757780),
        ("outline-variant", 0x44464F, 0xC5C6D0),
        ("primary", 0xBAC5EE, 0x4E5E8B),
        ("on-primary", 0x1F2D52, 0xFFFFFF),
        ("secondary-container", 0x353B4D, 0xDDE2F9),
        ("on-secondary-container", 0xDDE2F9, 0x161B2C),
        ("surface-high", 0x1E1F25, 0xE8E7EE),
        ("error", 0xFFB4AB, 0xBA1A1A),
        ("error-container", 0x8C1D18, 0xF9DEDC),
        ("on-error-container", 0xF9DEDC, 0x410E0B),
    ];
}

/// What: `named_roles(set)` lists every role of one scheme by its theme name.
/// Why:  lets a test walk all roles without repeating the field list.
fn named_roles(set: &RoleSet) -> Vec<(&'static str, Srgb8)> {
    return vec![
        ("canvas", set.canvas),
        ("deck", set.deck),
        ("current-row", set.current_row),
        ("surface-high", set.surface_high),
        ("on-surface", set.on_surface),
        ("on-surface-variant", set.on_surface_variant),
        ("outline", set.outline),
        ("outline-variant", set.outline_variant),
        ("primary", set.primary),
        ("on-primary", set.on_primary),
        ("secondary-container", set.secondary_container),
        ("secondary-container-hovered", set.secondary_container_hovered),
        ("secondary-container-pressed", set.secondary_container_pressed),
        ("on-secondary-container", set.on_secondary_container),
        ("match-fill", set.match_fill),
        ("error", set.error),
        ("error-container", set.error_container),
        ("on-error-container", set.on_error_container),
    ];
}

/// What: `lookup(set, name)` returns the role called `name`.
fn lookup(set: &RoleSet, name: &str) -> Srgb8 {
    for (role_name, color) in named_roles(set) {
        if role_name == name {
            return color;
        }
    }
    panic!("no role named {name}");
}

/// What: `near_gray_roots()` is every root the D118 rule must treat as chromatic although it looks gray: named
/// one-channel offsets, plus every fifth gray level with each channel raised by one step in turn, and the equal-channel
/// gray itself (a completely neutral root) at those levels.
/// Why:  the old rule gave a root with chroma below 0.004 the baseline hue and shrank the gray roles of a weak root;
///       these roots are the ones where that mattered, so the contrast sweep and the tint tests include them.
fn near_gray_roots() -> Vec<Srgb8> {
    let mut roots: Vec<Srgb8> = [0x808088, 0x7F8080, 0x8A8A8B, 0x8A8B8A, 0x808081, 0xFEFFFF, 0x010100, 0x7A7F9A]
        .into_iter()
        .map(hex)
        .collect();
    for level in (0..=255_u8).step_by(5) {
        let raised: u8 = level.saturating_add(1);
        roots.push(Srgb8 { red: level, green: level, blue: level });
        roots.push(Srgb8 { red: raised, green: level, blue: level });
        roots.push(Srgb8 { red: level, green: raised, blue: level });
        roots.push(Srgb8 { red: level, green: level, blue: raised });
    }
    return roots;
}

/// What: `is_completely_neutral(root)` is true when the three 8-bit channels are equal.
fn is_completely_neutral(root: Srgb8) -> bool {
    return root.red == root.green && root.green == root.blue;
}

/// What: `swept_roots()` is every test root: hues every 10 degrees by a spread of chromas, plus grays, the
/// near-gray roots and the extremes (saturated red, yellow, green, blue, white, black).
/// Why:  the contrast guarantee must hold for any accent, including ones whose chroma the gamut cuts hard.
fn swept_roots() -> Vec<Srgb8> {
    let mut roots: Vec<Srgb8> = Vec::new();
    let mut hue: f64 = 0.0;
    while hue < 360.0 {
        for lightness in [0.3, 0.55, 0.8, 0.95] {
            for chroma in [0.0, 0.01, 0.03, 0.06, 0.1, 0.15, 0.2, 0.3] {
                roots.push(oklch_to_srgb8(Oklch { lightness, chroma, hue }));
            }
        }
        hue += 10.0;
    }
    for value in [0xFF0000, 0xFFD000, 0xFFFF00, 0x00FF00, 0x0000FF, 0x6060C0, 0x000000, 0xFFFFFF, 0x808080, 0x7F7F80] {
        roots.push(hex(value));
    }
    roots.extend(near_gray_roots());
    return roots;
}

/// What: `surfaces_of(set)` lists the four surfaces text is drawn on.
/// Why:  Settings has no surface of its own (D109), so the canvas stands for it.
fn surfaces_of(set: &RoleSet) -> Vec<Srgb8> {
    return vec![set.canvas, set.deck, set.current_row, set.surface_high];
}

/// What: `assert_ratio(label, foreground, background, minimum)` fails with the names in the message.
fn assert_ratio(label: &str, foreground: Srgb8, background: Srgb8, minimum: f64) {
    let ratio: f64 = contrast_ratio(foreground, background);
    assert!(ratio >= minimum, "{label}: {foreground:?} on {background:?} is {ratio:.3}, below {minimum}");
}

/// What: `TextState` is one drawn foreground on one drawn background, with the ratio it must reach.
/// Why:  the guarantee is a list of these, so a test can show which pairs a color set fails instead of stopping at
///       the first one.
struct TextState {
    label: String,
    foreground: Srgb8,
    background: Srgb8,
    minimum: f64,
}

/// What: `push_state(states, label, foreground, background, minimum)` appends one pair to the list.
fn push_state(states: &mut Vec<TextState>, label: &str, foreground: Srgb8, background: Srgb8, minimum: f64) {
    states.push(TextState { label: label.to_string(), foreground, background, minimum });
}

/// What: `text_states(set)` is the whole contrast guarantee of one scheme: every text or outline pair the window
/// draws, at rest and under the state layers it carries (hover 8 percent, pressed and focus 12 percent).
/// Why:  one list feeds the sweep, the baseline check and the positive control. The pairs follow the `.slint` files:
///       text on the four surfaces under an on-surface layer (rows, menus, snackbar, buttons); the rail letter in
///       primary on the selected fill under an on-secondary-container layer (`folders.slint`); text on the selected
///       fill and its OKLCH hovered and pressed fills (`deck.slint`, `settings.slint`); on-primary on primary under
///       its own layer (`first_run.slint`, `deck.slint`); text on the match fill. Primary is not drawn on the OKLCH
///       hovered and pressed fills, so those two pairs are not part of the list.
fn text_states(set: &RoleSet) -> Vec<TextState> {
    let mut states: Vec<TextState> = Vec::new();
    let surface_names: [&str; 4] = ["canvas", "deck", "current-row", "surface-high"];
    for (surface_name, surface) in surface_names.into_iter().zip(surfaces_of(set)) {
        let hovered: Srgb8 = composite_over(set.on_surface, HOVER_ALPHA, surface);
        let pressed: Srgb8 = composite_over(set.on_surface, PRESSED_ALPHA, surface);
        for (text_name, text) in [("on-surface", set.on_surface), ("on-surface-variant", set.on_surface_variant), ("primary", set.primary)] {
            push_state(&mut states, &format!("{text_name} on the {surface_name}"), text, surface, 4.5);
            push_state(&mut states, &format!("{text_name} on the {surface_name}, hovered"), text, hovered, 4.5);
            push_state(&mut states, &format!("{text_name} on the {surface_name}, pressed"), text, pressed, 4.5);
        }
    }
    let fill: Srgb8 = set.secondary_container;
    let fill_hover_layer: Srgb8 = composite_over(set.on_secondary_container, HOVER_ALPHA, fill);
    let fill_pressed_layer: Srgb8 = composite_over(set.on_secondary_container, PRESSED_ALPHA, fill);
    for (fill_name, background) in [
        ("selected fill", fill),
        ("selected fill, hovered (OKLCH)", set.secondary_container_hovered),
        ("selected fill, pressed (OKLCH)", set.secondary_container_pressed),
        ("selected fill, hover layer", fill_hover_layer),
        ("selected fill, pressed layer", fill_pressed_layer),
    ] {
        push_state(&mut states, &format!("on-secondary-container on the {fill_name}"), set.on_secondary_container, background, 4.5);
    }
    for (fill_name, background) in [
        ("selected fill", fill),
        ("selected fill, hover layer", fill_hover_layer),
        ("selected fill, pressed layer", fill_pressed_layer),
    ] {
        push_state(&mut states, &format!("primary on the {fill_name}"), set.primary, background, 4.5);
    }
    push_state(&mut states, "on-primary on primary", set.on_primary, set.primary, 4.5);
    for (layer_name, alpha) in [("hovered", HOVER_ALPHA), ("pressed", PRESSED_ALPHA)] {
        let layered: Srgb8 = composite_over(set.on_primary, alpha, set.primary);
        push_state(&mut states, &format!("on-primary on primary, {layer_name}"), set.on_primary, layered, 4.5);
    }
    push_state(&mut states, "on-surface on the match fill", set.on_surface, set.match_fill, 4.5);
    push_state(&mut states, "on-surface-variant on the match fill", set.on_surface_variant, set.match_fill, 4.5);
    push_state(&mut states, "outline on the canvas", set.outline, set.canvas, 3.0);
    push_state(&mut states, "outline on the deck", set.outline, set.deck, 3.0);
    // The fixed error pairs hold too, so the guarantee covers every text pair in the window.
    push_state(&mut states, "on-error-container on the error container", set.on_error_container, set.error_container, 4.5);
    push_state(&mut states, "error on the error container", set.error, set.error_container, 3.0);
    push_state(&mut states, "error text on the canvas", set.error, set.canvas, 4.5);
    return states;
}

/// What: `failing_states(set)` names every pair of `text_states` below its minimum, with the measured ratio.
/// Why:  returns the list instead of panicking, so the positive control can compare it to the known failures.
fn failing_states(set: &RoleSet) -> Vec<String> {
    let mut failing: Vec<String> = Vec::new();
    for state in text_states(set) {
        let ratio: f64 = contrast_ratio(state.foreground, state.background);
        if ratio < state.minimum {
            failing.push(format!("{} ({ratio:.3} below {})", state.label, state.minimum));
        }
    }
    return failing;
}

/// What: `check_scheme_contrast(label, set)` asserts that no pair of one scheme falls below its minimum.
fn check_scheme_contrast(label: &str, set: &RoleSet) {
    let failing: Vec<String> = failing_states(set);
    assert!(failing.is_empty(), "{label}: {failing:?}");
}

#[test]
fn every_swept_root_keeps_aa_contrast_in_both_schemes() {
    let roots: Vec<Srgb8> = swept_roots();
    assert!(roots.len() > 1000, "the sweep is too small: {}", roots.len());
    for root in roots {
        let palette: ThemePalette = derive_roles(Some(root));
        check_scheme_contrast(&format!("dark root {root:?}"), &palette.dark);
        check_scheme_contrast(&format!("light root {root:?}"), &palette.light);
    }
}

#[test]
fn the_baseline_needs_no_contrast_adjustment() {
    // With the baseline root the targets alone pass, so the fit never moved lightness: every role is the plain
    // resolution of its table row.
    let terms: RootTerms = root_terms(None);
    for (scheme, spec) in [(Scheme::Dark, DARK_SPEC), (Scheme::Light, LIGHT_SPEC)] {
        let set: RoleSet = derive_scheme(terms, scheme);
        assert_eq!(set.primary, resolve_srgb(spec.primary, terms), "{scheme:?} primary moved");
        assert_eq!(set.on_surface, resolve_srgb(spec.on_surface, terms), "{scheme:?} on-surface moved");
        assert_eq!(set.on_surface_variant, resolve_srgb(spec.on_surface_variant, terms), "{scheme:?} variant moved");
        assert_eq!(set.outline, resolve_srgb(spec.outline, terms), "{scheme:?} outline moved");
        assert_eq!(set.on_primary, resolve_srgb(spec.on_primary, terms), "{scheme:?} on-primary moved");
        assert_eq!(
            set.on_secondary_container,
            resolve_srgb(spec.on_secondary_container, terms),
            "{scheme:?} on-secondary-container moved"
        );
    }
}

#[test]
fn the_fit_moves_lightness_when_a_target_fails_a_positive_control() {
    // A start that fails 4.5 on white must come back darker and passing, proving `fit_lightness` can act.
    let white: Srgb8 = hex(0xFFFFFF);
    let start: Oklch = Oklch { lightness: 0.85, chroma: 0.05, hue: 250.0 };
    let fitted: Oklch = fit_lightness("control", start, 0.0, &|candidate| {
        return contrast_ratio(candidate, white) >= 4.6;
    });
    assert!(fitted.lightness < start.lightness);
    assert_eq!(fitted.hue, start.hue);
    assert!(contrast_ratio(oklch_to_srgb8(fitted), white) >= 4.6);
}

#[test]
fn the_fit_returns_the_endpoint_when_nothing_can_pass() {
    let start: Oklch = Oklch { lightness: 0.5, chroma: 0.0, hue: 0.0 };
    let fitted: Oklch = fit_lightness("impossible", start, 0.0, &|_candidate| {
        return false;
    });
    assert_eq!(fitted.lightness, 0.0);
}

#[test]
fn no_accent_and_the_baseline_color_derive_the_same_palette() {
    assert_eq!(derive_roles(None), derive_roles(Some(BASELINE_ROOT)));
}

/// What: the roles whose baseline was retuned for B4 (contrast in every drawn state), as `(scheme, name, old hex,
/// largest delta-E allowed)`.
/// Why:  these moved in lightness only; each bound is the measured delta-E rounded up, so a further drift fails.
fn retuned_roles() -> Vec<(&'static str, &'static str, u32, f64)> {
    return vec![
        // Primary text under the pressed and hover layers on the selected fill and on surface-high.
        ("light", "primary", 0x4E5E8B, 3.6),
        // Secondary text under the pressed layer on surface-high.
        ("light", "on-surface-variant", 0x5C5E6A, 1.6),
        // Derived from the primary, so it follows it.
        ("light", "match-fill", 0xCFD9F0, 2.2),
    ];
}

#[test]
fn the_baseline_stays_close_to_the_old_fixed_values() {
    // ΔE (OKLab times 100): about 1 is a just-noticeable difference. Every role not retuned for B4 stays under 0.5
    // (the table is fitted to the captures); each retuned role has its own explicit bound in `retuned_roles`.
    let palette: ThemePalette = derive_roles(None);
    let retuned: Vec<(&str, &str, u32, f64)> = retuned_roles();
    let mut largest: f64 = 0.0;
    let mut largest_name: String = String::new();
    for (name, dark, light) in old_fixed_values() {
        for (scheme_name, set, old) in [("dark", &palette.dark, dark), ("light", &palette.light, light)] {
            let delta: f64 = delta_e_ok(lookup(set, name), hex(old));
            let is_retuned: bool = retuned.iter().any(|(retuned_scheme, retuned_name, _, _)| {
                return *retuned_scheme == scheme_name && *retuned_name == name;
            });
            if is_retuned {
                continue;
            }
            if delta > largest {
                largest = delta;
                largest_name = format!("{scheme_name} {name}");
            }
        }
    }
    println!("largest baseline delta-E vs the old fixed values, retuned roles aside: {largest:.3} ({largest_name})");
    assert!(largest < 0.5, "baseline drifted: {largest:.3} at {largest_name}");
}

#[test]
fn the_retuned_roles_moved_in_lightness_only_and_within_their_bounds() {
    let palette: ThemePalette = derive_roles(None);
    for (scheme_name, name, old, bound) in retuned_roles() {
        let set: &RoleSet = if scheme_name == "dark" { &palette.dark } else { &palette.light };
        let delta: f64 = delta_e_ok(lookup(set, name), hex(old));
        println!("retuned {scheme_name} {name}: {old:06X} to {:?}, delta-E {delta:.3}", lookup(set, name));
        assert!(delta > 0.5, "{scheme_name} {name} did not move ({delta:.3}), so it is not a retuned role");
        assert!(delta < bound, "{scheme_name} {name} moved {delta:.3}, over its bound {bound}");
        let before: Oklch = srgb_to_oklch(hex(old));
        let after: Oklch = srgb_to_oklch(lookup(set, name));
        assert!(after.lightness < before.lightness, "{name} must get darker in the light scheme");
        // 8-bit rounding limits the hue accuracy of a low-chroma color, so the bound is loose but far below a clip.
        assert!((after.hue - before.hue).abs() < 2.0, "{name} hue moved from {} to {}", before.hue, after.hue);
    }
}

/// What: `old_light_roles()` is the light scheme as `ui/theme.slint` held it before the B4 retune.
/// Why:  the positive control of the contrast list: a color set known to fall below AA must be reported as failing.
fn old_light_roles() -> RoleSet {
    let mut set: RoleSet = derive_roles(None).light;
    for (name, _dark, light) in old_fixed_values() {
        let color: Srgb8 = hex(light);
        match name {
            "primary" => set.primary = color,
            "on-surface-variant" => set.on_surface_variant = color,
            _ => {}
        }
    }
    set.match_fill = hex(0xCFD9F0);
    set.secondary_container_hovered = hex(0xCBD0E7);
    set.secondary_container_pressed = hex(0xC2C7DE);
    return set;
}

#[test]
fn the_baseline_passes_every_drawn_state_in_both_schemes() {
    let palette: ThemePalette = derive_roles(None);
    check_scheme_contrast("baseline dark", &palette.dark);
    check_scheme_contrast("baseline light", &palette.light);
}

#[test]
fn the_contrast_list_reports_the_states_the_old_light_baseline_failed_a_positive_control() {
    // Without this, a list that cannot fail would make the sweep above prove nothing. The old light literals fell
    // below AA in exactly these drawn states (the settings card and back-tonal surfaces no longer exist, D109).
    let failing: Vec<String> = failing_states(&old_light_roles());
    println!("old light baseline falls below AA in: {failing:#?}");
    for expected in [
        "on-surface-variant on the surface-high, pressed",
        "primary on the surface-high, pressed",
        "primary on the selected fill, hover layer",
        "primary on the selected fill, pressed layer",
    ] {
        let found: bool = failing.iter().any(|entry| {
            return entry.starts_with(expected);
        });
        assert!(found, "the list missed {expected}: {failing:?}");
    }
    assert_eq!(failing.len(), 4, "the old light baseline failed in an unexpected set of states: {failing:?}");
}

#[test]
fn the_contrast_list_reports_a_lightened_secondary_text_color_on_the_new_baseline() {
    // A second control on the new baseline: lightening on-surface-variant by a few steps must be reported.
    let mut set: RoleSet = derive_roles(None).light;
    set.on_surface_variant = hex(0x80828D);
    assert!(!failing_states(&set).is_empty());
}

#[test]
fn fixed_roles_do_not_follow_the_accent() {
    // Fixed roles are the pure black and pure white of D41, D34 and D45, white on-primary and the error colors. The
    // dark deck is no longer among them (D118): it is a gray and takes the root's tint.
    for root in swept_roots() {
        let palette: ThemePalette = derive_roles(Some(root));
        assert_eq!(palette.dark.canvas, hex(0x000000), "D41 black canvas under {root:?}");
        assert_eq!(palette.light.canvas, hex(0xFFFFFF), "D34 white canvas under {root:?}");
        assert_eq!(palette.light.on_primary, hex(0xFFFFFF), "white on-primary under {root:?}");
        assert_eq!(palette.dark.error, hex(0xFFB4AB));
        assert_eq!(palette.light.error_container, hex(0xF9DEDC));
    }
}

#[test]
fn accent_roles_keep_the_root_hue_instead_of_clipping() {
    // Roles with enough chroma keep root hue plus their offset within 4 degrees. 8-bit rounding limits the
    // hue accuracy of low-chroma roles, and clipping instead of chroma reduction would blow far past 4.
    let mut checked: usize = 0;
    let mut hue: f64 = 0.0;
    while hue < 360.0 {
        let root: Srgb8 = oklch_to_srgb8(Oklch { lightness: 0.6, chroma: 0.15, hue });
        let root_hue: f64 = srgb_to_oklch(root).hue;
        let palette: ThemePalette = derive_roles(Some(root));
        for (set, spec) in [(&palette.dark, DARK_SPEC), (&palette.light, LIGHT_SPEC)] {
            let terms: RootTerms = root_terms(Some(root));
            for (name, color, role) in [
                ("secondary-container", set.secondary_container, spec.secondary_container),
                ("primary", set.primary, spec.primary),
            ] {
                let lch: Oklch = srgb_to_oklch(color);
                if lch.chroma < 0.03 {
                    continue;
                }
                let spec_hue_offset: f64 = match role {
                    Role::Accent(role_spec) | Role::Neutral(role_spec) => role_spec.hue_offset,
                    Role::Fixed(_) => 0.0,
                };
                let expected: f64 = (terms.hue + spec_hue_offset).rem_euclid(360.0);
                let difference: f64 = (lch.hue - expected + 540.0).rem_euclid(360.0) - 180.0;
                assert!(difference.abs() < 4.0, "{name} hue {} vs {expected} for root hue {root_hue}", lch.hue);
                checked += 1;
            }
        }
        hue += 10.0;
    }
    assert!(checked > 40, "the hue check ran on only {checked} roles");
}

#[test]
fn derived_targets_before_quantizing_are_inside_the_gamut_after_mapping() {
    let terms: RootTerms = root_terms(Some(hex(0xFFD000)));
    for spec in [DARK_SPEC, LIGHT_SPEC] {
        for role in [spec.primary, spec.secondary_container, spec.on_primary, spec.on_secondary_container] {
            let mapped: Oklch = crate::oklch::gamut_map(resolve_oklch(role, terms));
            assert!(is_in_gamut(mapped));
            for channel in oklch_to_linear(mapped) {
                assert!((-1e-6..=1.0 + 1e-6).contains(&channel), "channel {channel}");
            }
        }
    }
}

#[test]
fn a_vivid_root_does_not_tint_the_neutral_surfaces_beyond_the_baseline() {
    let baseline: ThemePalette = derive_roles(None);
    let vivid: ThemePalette = derive_roles(Some(hex(0xFF0000)));
    // Chroma of the menu surface under a saturated red root is the baseline's, only the hue differs.
    let baseline_chroma: f64 = srgb_to_oklch(baseline.dark.surface_high).chroma;
    let vivid_chroma: f64 = srgb_to_oklch(vivid.dark.surface_high).chroma;
    assert!((vivid_chroma - baseline_chroma).abs() < 0.004, "{vivid_chroma} vs {baseline_chroma}");
}

/// Smallest chroma, measured in OKLCH from the final 8-bit colors, that every gray role keeps under a weakly chromatic
/// root. The lowest value measured over roots stepping one degree of hue in both schemes is 0.00455 (the dark deck);
/// the floor sits just under it, so a gray role that lost its tint (the old shrinking rule gave 0.001 to 0.003) fails.
const GRAY_CHROMA_FLOOR: f64 = 0.004;

/// What: `gray_specs(scheme)` lists the table rows of every gray role of one scheme with their names.
/// Why:  D118 names them: the deck, the playing row, surfaces, outlines and text variants, . Both
///       canvases are pure black or white and are not grays; the error roles are semantic colors.
fn gray_specs(scheme: Scheme) -> Vec<(&'static str, Role)> {
    let spec: SchemeSpec = if scheme == Scheme::Dark { DARK_SPEC } else { LIGHT_SPEC };
    let rows: Vec<(&'static str, Role)> = vec![
        ("deck", spec.deck),
        ("current-row", spec.current_row),
        ("surface-high", spec.surface_high),
        ("on-surface", spec.on_surface),
        ("on-surface-variant", spec.on_surface_variant),
        ("outline", spec.outline),
        ("outline-variant", spec.outline_variant),
    ];
    return rows;
}

/// What: `gray_colors(set, scheme)` lists the finished color of every gray role of one scheme with its name.
fn gray_colors(set: &RoleSet, scheme: Scheme) -> Vec<(&'static str, Srgb8)> {
    return gray_specs(scheme)
        .into_iter()
        .map(|(name, _)| {
            return (name, lookup(set, name));
        })
        .collect();
}

/// What: `channel_spread(color)` is the largest channel minus the smallest, 0 for a true gray.
fn channel_spread(color: Srgb8) -> u8 {
    return color.red.max(color.green).max(color.blue) - color.red.min(color.green).min(color.blue);
}

/// What: `spec_of(role)` is the table row of a `Neutral` or `Accent` role.
fn spec_of(role: Role) -> RoleSpec {
    return match role {
        Role::Neutral(spec) | Role::Accent(spec) => spec,
        Role::Fixed(_) => panic!("a fixed role has no table row"),
    };
}

#[test]
fn the_dark_deck_is_a_table_gray_that_reproduces_the_old_literal_at_the_baseline() {
    let baseline: ThemePalette = derive_roles(None);
    assert_eq!(baseline.dark.deck, hex(0x0A0A0D), "the baseline deck must stay the captured literal");
    assert!(matches!(DARK_SPEC.deck, Role::Neutral(_)), "the dark deck is a gray, so it takes the tint (D118)");
    let old: Oklch = srgb_to_oklch(hex(0x0A0A0D));
    let deck: RoleSpec = spec_of(DARK_SPEC.deck);
    println!("dark deck: old literal L {:.5} C {:.5}, table L {} C {}", old.lightness, old.chroma, deck.lightness, deck.chroma);
    assert!((deck.lightness - old.lightness).abs() < 0.0001, "the table lightness must be the old literal's");
}

#[test]
fn completely_neutral_means_three_equal_channels_and_no_weaker_root_counts() {
    // Every equal-channel root is completely neutral with scale exactly 0.
    let mut largest_gray_chroma: f64 = 0.0;
    for level in 0..=255_u8 {
        let root: Srgb8 = Srgb8 { red: level, green: level, blue: level };
        let terms: RootTerms = root_terms(Some(root));
        assert!(terms.completely_neutral, "gray {level} must be completely neutral");
        assert_eq!(terms.scale, 0.0, "gray {level} must have scale 0");
        largest_gray_chroma = largest_gray_chroma.max(srgb_to_oklch(root).chroma);
        // Every root one step off the gray axis, on any channel and in either direction, is NOT neutral.
        for (red, green, blue) in [(1_i16, 0_i16, 0_i16), (0, 1, 0), (0, 0, 1), (-1, 0, 0), (0, -1, 0), (0, 0, -1)] {
            let moved: [i16; 3] = [i16::from(level) + red, i16::from(level) + green, i16::from(level) + blue];
            if moved.iter().any(|channel| {
                return !(0..=255).contains(channel);
            }) {
                continue;
            }
            let off: Srgb8 = Srgb8 { red: moved[0] as u8, green: moved[1] as u8, blue: moved[2] as u8 };
            let off_terms: RootTerms = root_terms(Some(off));
            assert!(!off_terms.completely_neutral, "{off:?} has chroma and must not be neutral");
            assert!(off_terms.scale > 0.0, "{off:?} must keep a positive scale");
            let off_chroma: f64 = srgb_to_oklch(off).chroma;
            assert!(off_chroma > 1.0e-3, "{off:?} measures {off_chroma}, so a 1e-3 threshold would still separate them");
            assert_eq!(off_terms.hue, srgb_to_oklch(off).hue, "{off:?} must keep its own hue");
        }
    }
    println!("largest chroma of an equal-channel root: {largest_gray_chroma:e}; smallest unequal root measured 1.06e-3");
    assert!(largest_gray_chroma < 1.0e-6, "equal-channel chroma is float noise: {largest_gray_chroma}");
}

#[test]
fn a_completely_neutral_root_gives_equal_channels_for_every_gray_role() {
    // D118: with no root chroma at all every gray is a true gray, in both schemes, for all 256 levels.
    for level in 0..=255_u8 {
        let root: Srgb8 = Srgb8 { red: level, green: level, blue: level };
        let palette: ThemePalette = derive_roles(Some(root));
        for (scheme, set) in [(Scheme::Dark, &palette.dark), (Scheme::Light, &palette.light)] {
            for (name, color) in gray_colors(set, scheme) {
                assert_eq!(channel_spread(color), 0, "{scheme:?} {name} {color:?} is not a true gray under gray {level}");
            }
            for (name, color) in named_roles(set) {
                if name.starts_with("error") || name.ends_with("error-container") {
                    continue;
                }
                assert_eq!(channel_spread(color), 0, "{scheme:?} {name} {color:?} carries a tint under gray {level}");
            }
        }
    }
}

#[test]
fn every_gray_role_of_a_weakly_chromatic_root_keeps_a_visible_tint() {
    // D118: a nearly gray root is not gray. Every gray role keeps chroma above the floor, measured from the final
    // 8-bit colors, and at least two 8-bit steps between its channels, for every weak root of the sweep.
    let mut checked: usize = 0;
    let mut lowest: f64 = f64::MAX;
    let mut lowest_label: String = String::new();
    for root in swept_roots() {
        if is_completely_neutral(root) || srgb_to_oklch(root).chroma >= 0.02 {
            continue;
        }
        let palette: ThemePalette = derive_roles(Some(root));
        for (scheme, set) in [(Scheme::Dark, &palette.dark), (Scheme::Light, &palette.light)] {
            for (name, color) in gray_colors(set, scheme) {
                let chroma: f64 = srgb_to_oklch(color).chroma;
                if chroma < lowest {
                    lowest = chroma;
                    lowest_label = format!("{scheme:?} {name} {color:?} under root {root:?}");
                }
                assert!(chroma > GRAY_CHROMA_FLOOR, "{scheme:?} {name} {color:?} has chroma {chroma:.5} under root {root:?}");
                assert!(channel_spread(color) >= 2, "{scheme:?} {name} {color:?} lost its tint to rounding under {root:?}");
                checked += 1;
            }
        }
    }
    println!("lowest gray-role chroma over the weak roots: {lowest:.5} ({lowest_label})");
    assert!(checked > 1000, "the tint check ran on only {checked} roles");
}

#[test]
fn a_weakly_chromatic_root_gives_every_gray_role_the_root_hue_not_the_baseline_hue() {
    // #7F8080 has chroma 0.0012 and hue 197 degrees, far from the baseline hue (268.6). The old rule used the baseline
    // hue below chroma 0.004; D118 uses the root's own hue. Roles with chroma of 0.012 or more keep their hue to
    // within a few degrees through 8-bit rounding, so those are measured.
    let weak: Srgb8 = hex(0x7F8080);
    let root_hue: f64 = srgb_to_oklch(weak).hue;
    let baseline_hue: f64 = srgb_to_oklch(BASELINE_ROOT).hue;
    assert!((root_hue - baseline_hue).abs() > 60.0, "the control root must be far from the baseline hue");
    let terms: RootTerms = root_terms(Some(weak));
    assert_eq!(terms.hue, root_hue);
    let palette: ThemePalette = derive_roles(Some(weak));
    let mut measured: usize = 0;
    for (scheme, set) in [(Scheme::Dark, &palette.dark), (Scheme::Light, &palette.light)] {
        for (name, role) in gray_specs(scheme) {
            let spec: RoleSpec = spec_of(role);
            if spec.chroma < 0.012 {
                continue;
            }
            let color: Oklch = srgb_to_oklch(lookup(set, name));
            let wanted: f64 = (root_hue + spec.hue_offset).rem_euclid(360.0);
            let wrong: f64 = (baseline_hue + spec.hue_offset).rem_euclid(360.0);
            let off: f64 = (color.hue - wanted + 540.0).rem_euclid(360.0) - 180.0;
            let off_wrong: f64 = (color.hue - wrong + 540.0).rem_euclid(360.0) - 180.0;
            assert!(off.abs() < 6.0, "{scheme:?} {name} hue {} vs {wanted}", color.hue);
            assert!(off_wrong.abs() > 30.0, "{scheme:?} {name} hue {} is the baseline hue {wrong}", color.hue);
            measured += 1;
        }
    }
    assert!(measured >= 8, "only {measured} roles were measured");
}

#[test]
fn the_tint_of_a_gray_is_the_roles_own_tint_whatever_the_root_chroma() {
    // D118: not shrunk by a weak root, not grown by a vivid one. The resolved chroma of every gray role equals its
    // table value for roots of every chroma, and is 0 only for a completely neutral root.
    for root in [0x808088, 0x7F8080, 0x808081, 0xFEFFFF, 0x7A7F9A, 0x4E5E8B, 0x6060C0, 0xFF0000, 0x0000FF, 0x00FF00] {
        let terms: RootTerms = root_terms(Some(hex(root)));
        for scheme in [Scheme::Dark, Scheme::Light] {
            for (name, role) in gray_specs(scheme) {
                let resolved: Oklch = resolve_oklch(role, terms);
                assert_eq!(resolved.chroma, spec_of(role).chroma, "{scheme:?} {name} under {root:06X}");
            }
        }
    }
    for level in [0x00, 0x80, 0xFF] {
        let terms: RootTerms = root_terms(Some(Srgb8 { red: level, green: level, blue: level }));
        for scheme in [Scheme::Dark, Scheme::Light] {
            for (name, role) in gray_specs(scheme) {
                assert_eq!(resolve_oklch(role, terms).chroma, 0.0, "{scheme:?} {name} under gray {level}");
            }
        }
    }
}

#[test]
fn accent_roles_still_scale_with_the_root_chroma_while_grays_do_not() {
    let weak: RootTerms = root_terms(Some(hex(0x808088)));
    let baseline: RootTerms = root_terms(None);
    assert!(weak.scale > 0.0 && weak.scale < 0.3, "the weak root keeps a small positive scale: {}", weak.scale);
    assert_eq!(baseline.scale, 1.0);
    let weak_primary: f64 = resolve_oklch(DARK_SPEC.primary, weak).chroma;
    let baseline_primary: f64 = resolve_oklch(DARK_SPEC.primary, baseline).chroma;
    assert!(weak_primary < baseline_primary * 0.3, "accent chroma must shrink with a weak root");
    assert_eq!(resolve_oklch(DARK_SPEC.on_surface, weak).chroma, resolve_oklch(DARK_SPEC.on_surface, baseline).chroma);
}

#[test]
fn the_shrinking_rule_the_floor_replaces_is_caught_by_the_floor_a_positive_control() {
    // The old rule shrank a neutral role's chroma by min(1, scale). For #808088 that puts every gray role under the
    // floor, so the floor test above would fail on it; without this, a floor that nothing can fall under proves nothing.
    let weak: RootTerms = root_terms(Some(hex(0x808088)));
    for scheme in [Scheme::Dark, Scheme::Light] {
        for (name, role) in gray_specs(scheme) {
            let old_rule_chroma: f64 = spec_of(role).chroma * weak.scale.min(1.0);
            assert!(old_rule_chroma < GRAY_CHROMA_FLOOR, "{scheme:?} {name}: the old rule gave {old_rule_chroma}");
        }
    }
}

#[test]
fn a_more_vivid_accent_gives_a_more_vivid_primary_up_to_the_cap() {
    let muted: f64 = srgb_to_oklch(derive_roles(Some(hex(0x7A7F9A))).dark.primary).chroma;
    let vivid: f64 = srgb_to_oklch(derive_roles(Some(hex(0x6060C0))).dark.primary).chroma;
    assert!(vivid > muted, "{vivid} vs {muted}");
    // Past the cap a more vivid root changes nothing: a neon blue and a saturated red share one scale.
    let cap_scale: f64 = ROOT_CHROMA_CAP / srgb_to_oklch(BASELINE_ROOT).chroma;
    assert!((root_terms(Some(hex(0x4000FF))).scale - cap_scale).abs() < 1e-12);
    assert!((root_terms(Some(hex(0xFF0000))).scale - cap_scale).abs() < 1e-12);
}

#[test]
fn the_selected_fill_layers_step_toward_the_content_color() {
    for root in [None, Some(hex(0x6060C0))] {
        let palette: ThemePalette = derive_roles(root);
        for set in [&palette.dark, &palette.light] {
            let base: Oklch = srgb_to_oklch(set.secondary_container);
            let content: Oklch = srgb_to_oklch(set.on_secondary_container);
            let hovered: Oklch = srgb_to_oklch(set.secondary_container_hovered);
            let pressed: Oklch = srgb_to_oklch(set.secondary_container_pressed);
            let toward: f64 = (content.lightness - base.lightness).signum();
            assert!((hovered.lightness - base.lightness) * toward > 0.0, "hover does not move toward the content");
            assert!((pressed.lightness - hovered.lightness) * toward > 0.0, "pressed is not past hover");
        }
    }
}

#[test]
fn the_match_fill_is_darker_than_primary_in_dark_and_lighter_in_light() {
    let palette: ThemePalette = derive_roles(Some(hex(0x6060C0)));
    assert!(srgb_to_oklch(palette.dark.match_fill).lightness < srgb_to_oklch(palette.dark.primary).lightness);
    assert!(srgb_to_oklch(palette.light.match_fill).lightness > srgb_to_oklch(palette.light.primary).lightness);
}

#[test]
fn the_new_derived_roles_stay_near_what_the_old_srgb_layering_showed() {
    // Old look: the selected segment's hover and pressed fills were the container blended in sRGB with its content
    // color at 8 and 12 percent, and the match fill was the primary at 30 percent alpha over the row. These roles are
    // now OKLCH-derived (D108); the distance to the old rendering is measured here and bounded, and printed for the
    // report. The match fill is a deliberate change that mirrors Android's `searchMatchFill` (A3, D59).
    let palette: ThemePalette = derive_roles(None);
    let mut segment_largest: f64 = 0.0;
    let mut match_largest: f64 = 0.0;
    for (label, set) in [("dark", &palette.dark), ("light", &palette.light)] {
        let hovered_old: Srgb8 = composite_over(set.on_secondary_container, HOVER_ALPHA, set.secondary_container);
        let pressed_old: Srgb8 = composite_over(set.on_secondary_container, PRESSED_ALPHA, set.secondary_container);
        let hovered_delta: f64 = delta_e_ok(set.secondary_container_hovered, hovered_old);
        let pressed_delta: f64 = delta_e_ok(set.secondary_container_pressed, pressed_old);
        let over_canvas: Srgb8 = composite_over(set.primary, 0.3, set.canvas);
        let over_row: Srgb8 = composite_over(set.primary, 0.3, set.current_row);
        let canvas_delta: f64 = delta_e_ok(set.match_fill, over_canvas);
        let row_delta: f64 = delta_e_ok(set.match_fill, over_row);
        println!(
            "{label}: segment hover {hovered_delta:.2}, pressed {pressed_delta:.2}; match fill vs old over canvas {canvas_delta:.2}, over row {row_delta:.2}"
        );
        segment_largest = segment_largest.max(hovered_delta).max(pressed_delta);
        match_largest = match_largest.max(canvas_delta).max(row_delta);
    }
    assert!(segment_largest < 1.5, "segment layers drifted from the old look by {segment_largest:.2}");
    assert!(match_largest < 12.0, "match fill drifted from the old look by {match_largest:.2}");
}

#[test]
fn every_swept_root_keeps_accent_roles_in_gamut_with_the_hue_kept_not_clipped() {
    // Chroma reduction keeps the hue (a clip would shift it), so wherever a target keeps enough chroma its hue must
    // equal root hue plus the role's offset. Run over the whole sweep, both schemes, every table role.
    let mut checked: usize = 0;
    for root in swept_roots() {
        let terms: RootTerms = root_terms(Some(root));
        for spec in [DARK_SPEC, LIGHT_SPEC] {
            for role in [
                spec.current_row, spec.surface_high, spec.on_surface,
                spec.on_surface_variant, spec.outline, spec.outline_variant, spec.primary, spec.on_primary,
                spec.secondary_container, spec.on_secondary_container, spec.deck,
            ] {
                let wanted: Oklch = resolve_oklch(role, terms);
                let mapped: Oklch = crate::oklch::gamut_map(wanted);
                assert!(is_in_gamut(mapped), "{wanted:?} for root {root:?} maps outside sRGB");
                assert_eq!(mapped.lightness, wanted.lightness);
                assert_eq!(mapped.hue, wanted.hue);
                let quantized: Oklch = srgb_to_oklch(oklch_to_srgb8(wanted));
                if mapped.chroma >= 0.03 && wanted.lightness > 0.1 && wanted.lightness < 0.97 {
                    let difference: f64 = (quantized.hue - wanted.hue + 540.0).rem_euclid(360.0) - 180.0;
                    assert!(difference.abs() < 4.0, "hue {} vs {} for {wanted:?}, root {root:?}", quantized.hue, wanted.hue);
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 1000, "the hue check ran on only {checked} roles");
}

#[test]
fn settings_has_no_role_of_its_own_and_is_drawn_on_the_canvas_alone() {
    // D109: the page, the header and the rows share the canvas, which is pure white in light and pure black in dark.
    let baseline: ThemePalette = derive_roles(None);
    assert_eq!(baseline.light.canvas, hex(0xFFFFFF));
    assert_eq!(baseline.dark.canvas, hex(0x000000));
    for (name, _) in named_roles(&baseline.dark) {
        assert!(
            !name.starts_with("settings") && name != "back-tonal",
            "the role {name} belongs to the removed Settings tints"
        );
    }
}

#[test]
fn the_settings_row_pairs_hold_aa_under_every_swept_root() {
    // The Settings rows draw on the canvas: title in on-surface, line in on-surface-variant, both under the hover and
    // pressed layers; the selected row draws on the secondary container with on-secondary-container, and its icon
    // circle is that same container with the same content color.
    for root in swept_roots() {
        let palette: ThemePalette = derive_roles(Some(root));
        for (label, set) in [("dark", &palette.dark), ("light", &palette.light)] {
            let hovered: Srgb8 = composite_over(set.on_surface, HOVER_ALPHA, set.canvas);
            let pressed: Srgb8 = composite_over(set.on_surface, PRESSED_ALPHA, set.canvas);
            assert_ratio(&format!("{label} title on the canvas {root:?}"), set.on_surface, set.canvas, 4.5);
            assert_ratio(&format!("{label} title pressed {root:?}"), set.on_surface, pressed, 4.5);
            assert_ratio(&format!("{label} line on the canvas {root:?}"), set.on_surface_variant, set.canvas, 4.5);
            assert_ratio(&format!("{label} line hovered {root:?}"), set.on_surface_variant, hovered, 4.5);
            for fill in [set.secondary_container, set.secondary_container_hovered, set.secondary_container_pressed] {
                assert_ratio(&format!("{label} selected row {root:?}"), set.on_secondary_container, fill, 4.5);
            }
        }
    }
}

/// What: `accent_colors(set, scheme)` lists the finished color of every accent role of one scheme with its name.
/// Why:  the light on-primary is pure white (lightness 1 cannot carry chroma), so it is not among the light accents.
fn accent_colors(set: &RoleSet, scheme: Scheme) -> Vec<(&'static str, Srgb8)> {
    let mut colors: Vec<(&'static str, Srgb8)> = vec![
        ("primary", set.primary),
        ("secondary-container", set.secondary_container),
        ("secondary-container-hovered", set.secondary_container_hovered),
        ("secondary-container-pressed", set.secondary_container_pressed),
        ("on-secondary-container", set.on_secondary_container),
        ("match-fill", set.match_fill),
    ];
    if scheme == Scheme::Dark {
        colors.push(("on-primary", set.on_primary));
    }
    return colors;
}

/// Smallest chroma, measured in OKLCH from the final 8-bit colors, that every accent role keeps under a weakly
/// chromatic root. The lowest value measured over the weak roots of the sweep is 0.01313 (the dark pressed selected
/// fill), against a table floor of 0.0153 (dark) or 0.0193 (light) before rounding; the test floor sits just under it.
const ACCENT_CHROMA_FLOOR: f64 = 0.012;

#[test]
fn the_accent_floor_is_the_largest_gray_tint_of_each_scheme() {
    // Pins the two numbers the reading of D118 hangs on, so a changed gray row shows up here and in the fixture.
    assert!((gray_tint_ceiling(&DARK_SPEC) - 0.0153).abs() < 1e-12, "{}", gray_tint_ceiling(&DARK_SPEC));
    assert!((gray_tint_ceiling(&LIGHT_SPEC) - 0.0193).abs() < 1e-12, "{}", gray_tint_ceiling(&LIGHT_SPEC));
    for scheme in [Scheme::Dark, Scheme::Light] {
        let spec: SchemeSpec = if scheme == Scheme::Dark { DARK_SPEC } else { LIGHT_SPEC };
        let largest: f64 = gray_specs(scheme)
            .iter()
            .map(|(_, role)| {
                return spec_of(*role).chroma;
            })
            .fold(0.0, f64::max);
        assert_eq!(gray_tint_ceiling(&spec), largest, "{scheme:?}");
    }
}

#[test]
fn the_accent_floor_stays_under_every_baseline_accent_so_the_baseline_does_not_change() {
    // At the baseline scale is exactly 1, so an accent keeps its table chroma only when that is above the floor. The
    // one accent row with chroma 0 is the light on-primary, which is pure white (lightness 1) and stays white.
    for (scheme, spec) in [(Scheme::Dark, DARK_SPEC), (Scheme::Light, LIGHT_SPEC)] {
        let ceiling: f64 = gray_tint_ceiling(&spec);
        for (name, role) in [
            ("primary", spec.primary),
            ("on-primary", spec.on_primary),
            ("secondary-container", spec.secondary_container),
            ("on-secondary-container", spec.on_secondary_container),
        ] {
            let chroma: f64 = spec_of(role).chroma;
            let is_white: bool = spec_of(role).lightness >= 1.0;
            assert!(chroma >= ceiling || is_white, "{scheme:?} {name} chroma {chroma} is under the floor {ceiling}");
        }
    }
    assert_eq!(root_terms(None).scale, 1.0);
    assert_eq!(derive_roles(None).light.on_primary, hex(0xFFFFFF));
}

#[test]
fn an_accent_role_of_a_weakly_chromatic_root_is_never_grayer_than_the_grays() {
    // D118 reading: for a root that is not completely neutral every accent role keeps chroma of at least the
    // strongest gray tint of its scheme (before 8-bit rounding and gamut mapping), and the measured chroma stays
    // above a floor just under the gray tint.
    let mut checked: usize = 0;
    let mut lowest: f64 = f64::MAX;
    let mut lowest_label: String = String::new();
    for root in swept_roots() {
        if is_completely_neutral(root) || srgb_to_oklch(root).chroma >= 0.02 {
            continue;
        }
        let palette: ThemePalette = derive_roles(Some(root));
        for (scheme, set) in [(Scheme::Dark, &palette.dark), (Scheme::Light, &palette.light)] {
            for (name, color) in accent_colors(set, scheme) {
                let chroma: f64 = srgb_to_oklch(color).chroma;
                if chroma < lowest {
                    lowest = chroma;
                    lowest_label = format!("{scheme:?} {name} {color:?} under root {root:?}");
                }
                assert!(chroma > ACCENT_CHROMA_FLOOR, "{scheme:?} {name} {color:?} has chroma {chroma:.5} under {root:?}");
                checked += 1;
            }
        }
    }
    println!("lowest accent-role chroma over the weak roots: {lowest:.5} ({lowest_label})");
    assert!(checked > 1000, "the accent check ran on only {checked} roles");
}

#[test]
fn a_weak_roots_accent_resolves_to_the_scheme_floor_while_a_vivid_roots_keeps_its_scaled_chroma() {
    for (scheme, spec) in [(Scheme::Dark, DARK_SPEC), (Scheme::Light, LIGHT_SPEC)] {
        let ceiling: f64 = gray_tint_ceiling(&spec);
        let weak: RootTerms =
            RootTerms { accent_chroma_floor: ceiling, ..root_terms(Some(hex(0x7F8080))) };
        let vivid: RootTerms =
            RootTerms { accent_chroma_floor: ceiling, ..root_terms(Some(hex(0xFF0000))) };
        assert_eq!(resolve_oklch(spec.primary, weak).chroma, ceiling, "{scheme:?} weak primary");
        assert_eq!(
            resolve_oklch(spec.primary, vivid).chroma,
            spec_of(spec.primary).chroma * vivid.scale,
            "{scheme:?} vivid primary"
        );
    }
}

#[test]
fn a_completely_neutral_root_keeps_every_accent_role_at_chroma_zero() {
    for level in [0x00, 0x40, 0x80, 0xC0, 0xFF] {
        let palette: ThemePalette = derive_roles(Some(Srgb8 { red: level, green: level, blue: level }));
        for (scheme, set) in [(Scheme::Dark, &palette.dark), (Scheme::Light, &palette.light)] {
            for (name, color) in accent_colors(set, scheme) {
                assert_eq!(channel_spread(color), 0, "{scheme:?} {name} {color:?} under gray {level}");
            }
        }
    }
}
