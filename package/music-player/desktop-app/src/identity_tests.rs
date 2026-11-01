//! Tests that the identity constants stay in sync with the non-Rust files that
//! must carry the same strings:
//!  the macOS `Info.plist` (the bundle id) and the
//! Linux `.desktop` file (the WM class).
//!  Those files cannot share a Rust
//! constant,
//!  so these compile-time checks are the only guard against drift.

// What:     `use super::*;`. Glob-import the parent `identity` module's items
//           (its constants) into this test module. `super` names the parent.
// Why:      The tests reference `MACOS_BUNDLE_IDENTIFIER` and `APP_ID`.
use super::*;

// What:     `#[test] fn info_plist_carries_the_bundle_identifier()`. A unit test;
//           `#[test]` marks a zero-argument function the test runner executes.
// Why:      The macOS `CFBundleIdentifier` is authored in `macos/Info.plist`
//           (XML), so assert that file still contains the value identity.rs holds.
#[test]
fn info_plist_carries_the_bundle_identifier() {
    // What:     `let plist = include_str!("../macos/Info.plist");`. `include_str!`
    //           is a macro that reads the file AT COMPILE TIME and inlines its
    //           contents as a `&'static str` baked into the test binary. The path
    //           is relative to THIS source file (`src/identity_tests.rs`), so
    //           `../macos/...` reaches the package's `macos` directory.
    // Why:      Compare the committed plist text against the constant.
    let plist = include_str!("../macos/Info.plist");
    // What:     `assert!(plist.contains(MACOS_BUNDLE_IDENTIFIER), "...")`. Fail the
    //           test (printing the message) unless the plist contains the bundle
    //           id. `.contains` does a substring search.
    // Why:      Catch a plist/constant drift the moment it happens.
    assert!(
        plist.contains(MACOS_BUNDLE_IDENTIFIER),
        "macos/Info.plist must contain the bundle id {MACOS_BUNDLE_IDENTIFIER}"
    );
}

// What:     `#[test] fn desktop_file_wm_class_matches_app_id()`. A second unit test.
// Why:      KDE matches the window to the launcher entry by `StartupWMClass`, which
//           must equal `APP_ID`; the `.desktop` file is not Rust, so assert it here.
#[test]
fn desktop_file_wm_class_matches_app_id() {
    // What:     `let desktop = include_str!("../share/applications/monochromatic.music-player.desktop");`.
    //           Inline the `.desktop` file at compile time (path relative to this
    //           file). Its basename is `APP_ID` plus `.desktop`, but `include_str!`
    //           needs a string LITERAL, so the path is spelled out.
    // Why:      Read the file to check its `StartupWMClass` line.
    let desktop = include_str!("../share/applications/monochromatic.music-player.desktop");
    // What:     `let expected = format!("StartupWMClass={APP_ID}");`. Build the exact
    //           line the file must contain. `format!` allocates an owned `String`.
    // Why:      Assert the WM class EQUALS the app id, not merely that the id appears
    //           somewhere.
    let expected = format!("StartupWMClass={APP_ID}");
    // What:     `assert!(desktop.contains(&expected), "...")`. Fail unless the line
    //           is present. `&expected` lends the `String` as a `&str` to `.contains`.
    // Why:      Catch a `.desktop`/`APP_ID` drift.
    assert!(
        desktop.contains(&expected),
        "desktop file must contain {expected}"
    );
}
