//! Unit tests of the output readers and the version comparison.

use serde_json::Value;

use super::parse_output::{
    global_pnpm_versions, normalize_version, parse_bin_list, parse_cargo_binstall_list, parse_uv_tool_list,
    version_matches,
};

fn parse(text: &str) -> Value {
    return serde_json::from_str(text).expect("the literal is JSON");
}

#[test]
fn versions_lose_release_tag_prefixes() {
    assert_eq!(normalize_version("v1.3.2"), "1.3.2");
    assert_eq!(normalize_version("1.3.2"), "1.3.2");
    assert_eq!(normalize_version("nightly-2026-10-21"), "nightly-2026-10-21");
    assert_eq!(normalize_version("bun-v1.3.14"), "1.3.14");
    assert_eq!(normalize_version("v"), "v");
    assert_eq!(normalize_version("vim"), "vim");
    assert_eq!(normalize_version(""), "");
}

#[test]
fn equal_and_platform_suffixed_versions_match_but_different_ones_do_not() {
    assert!(version_matches("v1.3.2", "1.3.2"));
    assert!(version_matches("nightly-2026-10-21-x86_64-unknown-linux-gnu", "nightly-2026-10-21"));
    assert!(version_matches("nightly-2026-10-21", "nightly-2026-10-21-x86_64-unknown-linux-gnu"));
    assert!(!version_matches("1.3.1", "1.3.2"));
    assert!(!version_matches("1.3.20", "1.3.2"));
}

#[test]
fn a_plain_bin_list_row_gives_the_file_name_and_the_bare_version() {
    let versions = parse_bin_list("/bin/tool  v1.2.3  https://example.test  ok");
    assert_eq!(versions.get("tool").map(String::as_str), Some("1.2.3"));
}

#[test]
fn bin_list_rows_separated_by_carriage_returns_or_unicode_line_terminators_are_all_kept() {
    let carriage = parse_bin_list("/bin/a  v1.2.3  https://a.test  ok\r/bin/b  v2.0.0  https://b.test  ok");
    assert_eq!(carriage.get("a").map(String::as_str), Some("1.2.3"));
    assert_eq!(carriage.get("b").map(String::as_str), Some("2.0.0"));
    let unicode = parse_bin_list("/bin/a  v1.2.3  https://a.test  ok\u{2028}/bin/b  v2.0.0  https://b.test  ok");
    assert_eq!(unicode.get("a").map(String::as_str), Some("1.2.3"));
    assert_eq!(unicode.get("b").map(String::as_str), Some("2.0.0"));
}

#[test]
fn bin_list_keeps_a_path_with_single_spaces_and_drops_the_star_marker() {
    let versions = parse_bin_list("/my tools/ripgrep  *v15.2.0  https://x.test  ok");
    assert_eq!(versions.get("ripgrep").map(String::as_str), Some("15.2.0"));
}

#[test]
fn bin_list_ignores_indented_short_and_empty_rows() {
    let versions = parse_bin_list("  /bin/indented  v1.0.0  https://x.test  ok\n\n/bin/short  v1.0.0\nheader only");
    assert!(versions.is_empty(), "{versions:?}");
}

#[test]
fn the_binstall_version_column_is_normalized_like_the_cargo_one() {
    let versions = parse_cargo_binstall_list("ripgrep  v14.1.0  v14.1.0  prebuilt");
    assert_eq!(versions.get("ripgrep").map(String::as_str), Some("14.1.0"));
}

#[test]
fn a_dash_in_the_binstall_column_falls_back_to_the_cargo_version() {
    let versions = parse_cargo_binstall_list("ripgrep  v14.1.0  -  compiled");
    assert_eq!(versions.get("ripgrep").map(String::as_str), Some("14.1.0"));
}

#[test]
fn the_binstall_list_header_and_footer_are_not_crates() {
    let versions = parse_cargo_binstall_list("Package  Cargo  Binstall  Source\nfd-find  v10.5.0  v10.5.0  prebuilt\nTotal: 1 crate  x  y  z");
    assert_eq!(versions.len(), 1);
    assert_eq!(versions.get("fd-find").map(String::as_str), Some("10.5.0"));
}

#[test]
fn uv_tool_rows_are_keyed_by_the_underscored_name_and_binary_rows_are_skipped() {
    let versions = parse_uv_tool_list("meta-package-manager v8.2.0\n- mpm\n- meta-package-manager\nslopo v0.9.0\n");
    assert_eq!(versions.get("meta_package_manager").map(String::as_str), Some("8.2.0"));
    assert_eq!(versions.get("slopo").map(String::as_str), Some("0.9.0"));
    assert_eq!(versions.len(), 2);
}

#[test]
fn pnpm_global_listings_in_either_shape_give_package_versions() {
    let object = parse(r#"{ "dependencies": { "wrangler": { "version": "4.156.0" }, "broken": { "from": "x" } } }"#);
    assert_eq!(global_pnpm_versions(&object).get("wrangler").map(String::as_str), Some("4.156.0"));
    assert!(!global_pnpm_versions(&object).contains_key("broken"));
    let list = parse(r#"[{ "dependencies": { "pagefind": { "version": "1.5.2" } } }, 3, { "name": "no deps" }]"#);
    assert_eq!(global_pnpm_versions(&list).get("pagefind").map(String::as_str), Some("1.5.2"));
    assert!(global_pnpm_versions(&parse("\"not an object\"")).is_empty());
}
