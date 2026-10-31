//! Unit tests of the request text:
//! `<origin>@<range>`.

use super::request::{Origin, Request};

#[test]
fn the_range_defaults_to_latest_and_the_text_round_trips() {
    let request = Request::parse("github:FiloSottile/age").expect("parses");
    assert_eq!(request.source, "github:FiloSottile/age");
    assert_eq!(request.range, "latest");
    assert_eq!(request.text(), "github:FiloSottile/age@latest");
    assert_eq!(Request::parse(&request.text()).expect("parses again"), request);
}

#[test]
fn the_separator_is_the_last_at_sign_that_does_not_follow_a_colon_or_a_slash() {
    let scoped = Request::parse("npm:@actions/languageserver").expect("parses");
    assert_eq!(scoped.source, "npm:@actions/languageserver");
    assert_eq!(scoped.range, "latest");
    let scoped_pinned = Request::parse("npm:@actions/languageserver@0.3.61").expect("parses");
    assert_eq!(scoped_pinned.source, "npm:@actions/languageserver");
    assert_eq!(scoped_pinned.range, "0.3.61");
    let git = Request::parse("git:https://git.example/o/r@ref:abc123").expect("parses");
    assert_eq!(git.source, "git:https://git.example/o/r");
    assert_eq!(git.range, "ref:abc123");
}

#[test]
fn every_origin_kind_is_recognized() {
    assert_eq!(Request::parse("github:o/r@1").expect("parses").origin(), Origin::Github("o/r".to_string()));
    assert_eq!(Request::parse("npm:wrangler@1").expect("parses").origin(), Origin::Npm("wrangler".to_string()));
    assert_eq!(Request::parse("cargo:fd-find@1").expect("parses").origin(), Origin::Cargo("fd-find".to_string()));
    assert_eq!(Request::parse("pipx:slopo@1").expect("parses").origin(), Origin::Pipx("slopo".to_string()));
    assert_eq!(Request::parse("conda:bubblewrap@1").expect("parses").origin(), Origin::Conda("bubblewrap".to_string()));
    assert_eq!(Request::parse("go:github.com/charmbracelet/crush@1").expect("parses").origin(), Origin::GoModule("github.com/charmbracelet/crush".to_string()));
    assert_eq!(Request::parse("zig@0.15.2").expect("parses").origin(), Origin::Named("zig".to_string()));
    assert_eq!(Request::parse("java:temurin@21").expect("parses").origin(), Origin::Named("java:temurin".to_string()));
}

#[test]
fn an_unknown_origin_an_empty_part_or_a_space_is_refused_with_the_valid_origins_named() {
    let unknown = Request::parse("frobnicate@1").expect_err("unknown origin");
    assert!(unknown.contains("zig") && unknown.contains("github:"), "{unknown}");
    for bad in ["", "@1", "zig@", "github:", "zig @1", "zig@ 1"] {
        assert!(Request::parse(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn exact_ranges_never_move_and_latest_channels_and_major_lines_do() {
    let exact = |text: &str| return Request::parse(text).expect("parses").is_exact();
    assert!(exact("zig@0.15.2"));
    assert!(exact("github:ggml-org/llama.cpp@b9829"));
    assert!(exact("rust@nightly-2026-10-29"));
    assert!(exact("npm:wrangler@4.156.0"));
    assert!(!exact("zig@latest"));
    assert!(!exact("zig"));
    assert!(!exact("rust@nightly"));
    assert!(!exact("java:temurin@21"));
    assert!(exact("java:temurin@21.0.13+9"));
    assert!(!exact("npm:wrangler@next"));
}
