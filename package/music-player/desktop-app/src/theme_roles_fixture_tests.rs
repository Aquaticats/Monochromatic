// What:     Drift tests for the shared theme vector, pulled in by `#[cfg(test)] #[path = "theme_roles_fixture_tests.rs"]
//           mod fixture_tests;` in `theme_roles.rs`.
// Why:      The Android port derives the same roles in Kotlin and checks them against
//           `android-app/app/src/test/resources/theme-roles-vector.txt`. This file closes the other half: every line of
//           that fixture must equal what `derive_roles` computes now, so a change to either implementation fails a test
//           on the side that changed, and the fixture cannot silently go stale.

use super::*;

/// Path of the shared fixture, relative to the package directory (`CARGO_MANIFEST_DIR`).
const FIXTURE_PATH: &str = "../android-app/app/src/test/resources/theme-roles-vector.txt";

/// Fewest roots the fixture must hold, each in both schemes.
const MINIMUM_ROOTS: usize = 52;

/// Roots the fixture must name (D118): the baseline, nearly gray roots that must still tint every gray (one-channel
/// offsets and a weak chroma at other hues), and completely neutral roots that must give true grays.
const REQUIRED_NEAR_GRAY_ROOTS: [(&str, u32); 6] = [
    ("weak-808088", 0x808088),
    ("weak-7F8080", 0x7F8080),
    ("weak-8A8A8B", 0x8A8A8B),
    ("weak-8A8B8A", 0x8A8B8A),
    ("weak-808081", 0x808081),
    ("weak-FEFFFF", 0xFEFFFF),
];

/// Completely neutral roots the fixture must name (all three channels equal).
const REQUIRED_NEUTRAL_ROOTS: [(&str, u32); 4] =
    [("neutral-8A8A8A", 0x8A8A8A), ("gray-mid", 0x808080), ("black", 0x000000), ("white", 0xFFFFFF)];

/// What: `role_fields(set)` lists the fixture's role names (the Kotlin `ThemeRoles` field names) with their colors.
/// Why:  the fixture writes one `name=RRGGBB` pair per role; this is the single place that maps them to `RoleSet`.
fn role_fields(set: &RoleSet) -> [(&'static str, Srgb8); 18] {
    return [
        ("canvas", set.canvas),
        ("deck", set.deck),
        ("currentRow", set.current_row),
        ("surfaceHigh", set.surface_high),
        ("onSurface", set.on_surface),
        ("onSurfaceVariant", set.on_surface_variant),
        ("outline", set.outline),
        ("outlineVariant", set.outline_variant),
        ("primary", set.primary),
        ("onPrimary", set.on_primary),
        ("secondaryContainer", set.secondary_container),
        ("secondaryContainerHovered", set.secondary_container_hovered),
        ("secondaryContainerPressed", set.secondary_container_pressed),
        ("onSecondaryContainer", set.on_secondary_container),
        ("matchFill", set.match_fill),
        ("error", set.error),
        ("errorContainer", set.error_container),
        ("onErrorContainer", set.on_error_container),
    ];
}

/// What: `hex6(color)` is `RRGGBB` in upper case, the fixture's color syntax.
fn hex6(color: Srgb8) -> String {
    return format!("{:02X}{:02X}{:02X}", color.red, color.green, color.blue);
}

/// What: `parse_hex6(text)` reads `RRGGBB`, or `None` for anything else.
fn parse_hex6(text: &str) -> Option<Srgb8> {
    let all_hex: bool = text.chars().all(|digit| {
        return digit.is_ascii_hexdigit();
    });
    if text.len() != 6 || !all_hex {
        return None;
    }
    let value: u32 = u32::from_str_radix(text, 16).ok()?;
    return Some(Srgb8 { red: ((value >> 16) & 0xFF) as u8, green: ((value >> 8) & 0xFF) as u8, blue: (value & 0xFF) as u8 });
}

/// What: `FixtureReport { lines, roots, mismatches }`: what comparing a fixture text to the derivation found.
/// Why:  a function from text to a report cannot pass vacuously: the test asserts the counts as well as the list.
struct FixtureReport {
    /// Number of role lines compared.
    lines: usize,
    /// Root names seen, with the scheme names each appeared in.
    roots: Vec<(String, Vec<String>)>,
    /// One entry per role, line or token that differs from the derivation.
    mismatches: Vec<String>,
}

/// What: `compare_fixture(text)` derives every root in the fixture and reports each difference.
/// Why:  the Rust derivation is the source of the fixture, so every line must equal it exactly; a missing, extra,
///       malformed or different token is a mismatch naming the root, the scheme and the role.
fn compare_fixture(text: &str) -> FixtureReport {
    let mut report: FixtureReport = FixtureReport { lines: 0, roots: Vec::new(), mismatches: Vec::new() };
    for (index, line) in text.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        report.lines += 1;
        let tokens: Vec<&str> = line.split(' ').collect();
        if tokens.len() < 3 {
            report.mismatches.push(format!("line {}: fewer than three fields", index + 1));
            continue;
        }
        let (name, root_text, scheme_name) = (tokens[0], tokens[1], tokens[2]);
        let Some(root) = parse_hex6(root_text) else {
            report.mismatches.push(format!("line {}: the root {root_text} is not RRGGBB", index + 1));
            continue;
        };
        let palette: ThemePalette = derive_roles(Some(root));
        let set: RoleSet = match scheme_name {
            "dark" => palette.dark,
            "light" => palette.light,
            other => {
                report.mismatches.push(format!("line {}: unknown scheme {other}", index + 1));
                continue;
            }
        };
        let seen: Option<&mut (String, Vec<String>)> = report.roots.iter_mut().find(|(known, _)| {
            return known == name;
        });
        match seen {
            Some((_, schemes)) => schemes.push(scheme_name.to_string()),
            None => report.roots.push((name.to_string(), vec![scheme_name.to_string()])),
        }
        let fields: [(&str, Srgb8); 18] = role_fields(&set);
        let pairs: &[&str] = &tokens[3..];
        if pairs.len() != fields.len() {
            report.mismatches.push(format!("{name} {scheme_name}: {} role tokens, expected {}", pairs.len(), fields.len()));
        }
        for (role, derived) in fields {
            let found: Option<&str> = pairs.iter().find_map(|pair| {
                return pair.strip_prefix(role)?.strip_prefix('=');
            });
            match found {
                Some(value) if value == hex6(derived) => {}
                Some(value) => report.mismatches.push(format!(
                    "{name} {scheme_name} {role}: the fixture says {value}, the derivation gives {}",
                    hex6(derived)
                )),
                None => report.mismatches.push(format!("{name} {scheme_name} {role}: missing from the fixture")),
            }
        }
    }
    return report;
}

/// What: `fixture_text()` reads the Android fixture next to the Android unit tests.
/// Why:  the file lives in the Android package, so the path climbs out of the desktop package; a missing file fails
///       the test with the path, because skipping would hide exactly the drift this guards against. The podman
///       fallback of `mise.toml` (`containerCargo`) mounts the Android test resources read-only at
///       `/android-app/app/src/test/resources`, so the same relative path from `/work` finds the file there.
fn fixture_text() -> String {
    let path: std::path::PathBuf = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_PATH);
    return std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read the shared theme fixture at {}: {error}", path.display()));
}

/// What: `change_one_channel(text, line_start, role)` raises one channel of one role in the first line that starts
/// with `line_start` by one step, wrapping 255 to 0, and leaves every other byte alone.
/// Why:  the guard test edits the fixture by the smallest possible amount, one channel value.
fn change_one_channel(text: &str, line_start: &str, role: &str) -> String {
    let mut changed: bool = false;
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        if !changed && line.starts_with(line_start) {
            let marker: String = format!(" {role}=");
            let at: usize = line.find(&marker).unwrap_or_else(|| panic!("no {role} in the line {line}")) + marker.len();
            let color: Srgb8 = parse_hex6(&line[at..at + 6]).unwrap_or_else(|| panic!("{role} is not RRGGBB in {line}"));
            let moved: Srgb8 = Srgb8 { red: color.red.wrapping_add(1), ..color };
            lines.push(format!("{}{}{}", &line[..at], hex6(moved), &line[at + 6..]));
            changed = true;
        } else {
            lines.push(line.to_string());
        }
    }
    assert!(changed, "no line starts with {line_start}");
    return lines.join("\n") + "\n";
}

#[test]
fn every_line_of_the_android_fixture_equals_the_derivation() {
    let report: FixtureReport = compare_fixture(&fixture_text());
    assert!(report.mismatches.is_empty(), "the Android fixture drifted from the Rust derivation: {:#?}", report.mismatches);
    println!("compared {} lines of {} roots", report.lines, report.roots.len());
}

#[test]
fn the_android_fixture_holds_enough_roots_in_both_schemes_and_the_baseline() {
    let report: FixtureReport = compare_fixture(&fixture_text());
    assert!(report.roots.len() >= MINIMUM_ROOTS, "only {} roots", report.roots.len());
    assert!(report.lines >= 2 * MINIMUM_ROOTS, "only {} lines compared", report.lines);
    for (name, schemes) in &report.roots {
        assert_eq!(schemes, &vec!["dark".to_string(), "light".to_string()], "root {name} is not in both schemes, once each");
    }
    let has_baseline: bool = report.roots.iter().any(|(name, _)| {
        return name == "baseline";
    });
    assert!(has_baseline, "the baseline root is missing");
}

#[test]
fn the_fixture_names_the_near_gray_and_the_completely_neutral_roots_with_the_expected_values() {
    let text: String = fixture_text();
    let mut seen: usize = 0;
    for (name, value) in REQUIRED_NEAR_GRAY_ROOTS.into_iter().chain(REQUIRED_NEUTRAL_ROOTS) {
        let root: Srgb8 = Srgb8 { red: ((value >> 16) & 0xFF) as u8, green: ((value >> 8) & 0xFF) as u8, blue: (value & 0xFF) as u8 };
        for scheme_name in ["dark", "light"] {
            let prefix: String = format!("{name} {} {scheme_name} ", hex6(root));
            assert!(text.lines().any(|line| {
                return line.starts_with(&prefix);
            }), "the fixture has no line starting {prefix:?}");
            seen += 1;
        }
    }
    assert_eq!(seen, 2 * (REQUIRED_NEAR_GRAY_ROOTS.len() + REQUIRED_NEUTRAL_ROOTS.len()));
    // The two groups differ in exactly the way D118 draws the line: a near-gray root has unequal channels, a neutral
    // root equal channels, and the gray roles of the first carry a tint while the second's are true grays.
    for (name, value) in REQUIRED_NEAR_GRAY_ROOTS {
        let root: Srgb8 = Srgb8 { red: ((value >> 16) & 0xFF) as u8, green: ((value >> 8) & 0xFF) as u8, blue: (value & 0xFF) as u8 };
        assert!(!(root.red == root.green && root.green == root.blue), "{name} must not be completely neutral");
        let palette: ThemePalette = derive_roles(Some(root));
        assert!(palette.light.on_surface_variant.red != palette.light.on_surface_variant.green
            || palette.light.on_surface_variant.green != palette.light.on_surface_variant.blue, "{name} lost its tint");
    }
    for (name, value) in REQUIRED_NEUTRAL_ROOTS {
        let root: Srgb8 = Srgb8 { red: ((value >> 16) & 0xFF) as u8, green: ((value >> 8) & 0xFF) as u8, blue: (value & 0xFF) as u8 };
        assert!(root.red == root.green && root.green == root.blue, "{name} must be completely neutral");
    }
}

#[test]
fn the_fixture_baseline_lines_hold_the_literals_of_the_baseline_derivation() {
    let palette: ThemePalette = derive_roles(None);
    let text: String = fixture_text();
    for (scheme_name, set) in [("dark", palette.dark), ("light", palette.light)] {
        let prefix: String = format!("baseline {} {scheme_name} ", hex6(BASELINE_ROOT));
        let line: &str = text
            .lines()
            .find(|candidate| {
                return candidate.starts_with(&prefix);
            })
            .expect("a baseline line");
        for (role, color) in role_fields(&set) {
            assert!(line.contains(&format!(" {role}={}", hex6(color))), "{scheme_name} {role} differs in {line}");
        }
    }
}

#[test]
fn a_fixture_edited_by_one_channel_value_is_reported_as_exactly_one_mismatch() {
    // The guard: the comparison must fail when the fixture is edited by one channel of one role. If it did not, the
    // equality test above would prove nothing.
    let text: String = fixture_text();
    for (line_start, role) in [("baseline 4E5E8B light ", "primary"), ("baseline 4E5E8B dark ", "onSurfaceVariant")] {
        let edited: String = change_one_channel(&text, line_start, role);
        assert_ne!(edited, text);
        let report: FixtureReport = compare_fixture(&edited);
        assert_eq!(report.mismatches.len(), 1, "expected exactly one mismatch: {:#?}", report.mismatches);
        assert!(report.mismatches[0].contains(role), "the mismatch does not name {role}: {}", report.mismatches[0]);
        assert!(report.mismatches[0].contains(line_start.split(' ').nth(2).unwrap_or("")), "{}", report.mismatches[0]);
    }
}

#[test]
fn a_fixture_with_a_missing_role_token_or_a_bad_scheme_is_reported() {
    let text: String = fixture_text();
    let without_primary: String = text.replacen(" primary=", " primaryX=", 1);
    assert!(!compare_fixture(&without_primary).mismatches.is_empty());
    let bad_scheme: String = text.replacen(" light ", " lite ", 1);
    assert!(!compare_fixture(&bad_scheme).mismatches.is_empty());
}
