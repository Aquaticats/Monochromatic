//! Unit tests of the Rust parts reconciler,
//! against the fake runner and a disposable scope:
//! the tests assert that the one `rustup` listing decides what is missing,
//! which listed names count as the declared part,
//! and that a listing that fails stops the reconcile before any add.
//!
//! The listing is `rustup component list --installed`, and a declared target is held when its
//! `rust-std-<triple>` line is in it: that is the listing `rustup target list --installed` filters
//! for (`list_targets` in rustup 1.29.3 `src/cli/rustup_mode.rs`) and the component
//! `rustup target add` installs, so one question answers both.

use super::fakes::Rig;
use super::install_rust_android::reconcile_rust_parts;
use super::model::LockedTool;
use super::runner::RunRequest;

/// The sample lock's `rust` tool,
/// whose options declare the components `clippy` and `rust-src`
/// and the target `aarch64-linux-android`.
fn rust_tool(rig: &Rig) -> LockedTool {
    return rig.tool("rust");
}

/// The component listing of a toolchain holding the host's own standard library and nothing else.
const THE_HOSTS_STANDARD_LIBRARY: &str = "rust-std-x86_64-unknown-linux-gnu\n";

/// Answers the sample toolchain's component listing with this text,
/// then reconciles the tool.
fn reconcile_with(
    rig: &Rig,
    tool: &LockedTool,
    components: &str,
) -> Result<Vec<String>, super::error::ToolchainError> {
    rig.runner.answer(
        "rustup component list --installed --toolchain nightly-2026-10-30",
        components,
        0,
    );
    let mut added: Vec<String> = Vec::new();
    return reconcile_rust_parts(&rig.context(), tool, &mut added).map(|()| return added);
}

/// The adds the fake recorded, in order.
fn the_adds(rig: &Rig) -> Vec<String> {
    return rig
        .runner
        .lines()
        .into_iter()
        .filter(|line| return line.contains(" add "))
        .collect();
}

#[test]
fn a_component_listed_with_its_host_triple_counts_as_installed() {
    let rig = Rig::new();
    let added: Vec<String> = reconcile_with(
        &rig,
        &rust_tool(&rig),
        "clippy-x86_64-unknown-linux-gnu\nrust-src\nrust-std-x86_64-unknown-linux-gnu\nrust-std-aarch64-linux-android\n",
    )
    .expect("listed");
    assert!(added.is_empty(), "{added:?}");
    assert!(the_adds(&rig).is_empty(), "{:?}", rig.runner.lines());
}

/// A declared target is held when its standard library is a component line, which is the whole of the
/// claim that makes one listing enough.
#[test]
fn a_target_is_held_when_its_standard_library_is_a_component_line() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    tool.declaration.options.rust_components = None;
    let added: Vec<String> = reconcile_with(
        &rig,
        &tool,
        &format!("{THE_HOSTS_STANDARD_LIBRARY}rust-std-aarch64-linux-android\n"),
    )
    .expect("listed");
    assert!(added.is_empty(), "{added:?}");
    assert!(the_adds(&rig).is_empty(), "{:?}", rig.runner.lines());
}

#[test]
fn every_declared_part_counts_as_missing_when_rustup_lists_none() {
    let rig = Rig::new();
    let added: Vec<String> =
        reconcile_with(&rig, &rust_tool(&rig), "").expect("an empty listing is a listing");
    assert_eq!(added.len(), 3, "{added:?}");
    assert_eq!(
        rig.runner.lines(),
        vec![
            "rustup component list --installed --toolchain nightly-2026-10-30".to_string(),
            "rustup component add clippy --toolchain nightly-2026-10-30".to_string(),
            "rustup component add rust-src --toolchain nightly-2026-10-30".to_string(),
            "rustup component list --installed --toolchain nightly-2026-10-30".to_string(),
            "rustup target add aarch64-linux-android --toolchain nightly-2026-10-30".to_string(),
        ]
    );
}

#[test]
fn a_longer_installed_target_name_is_not_the_declared_target() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    tool.declaration.options.rust_targets = Some(vec!["wasm32-wasip1".to_string()]);
    tool.declaration.options.rust_components = None;
    let added: Vec<String> = reconcile_with(
        &rig,
        &tool,
        &format!("rust-src\n{THE_HOSTS_STANDARD_LIBRARY}rust-std-wasm32-wasip1-threads\n"),
    )
    .expect("listed");
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(
        added[0].contains("added the declared target wasm32-wasip1,"),
        "{added:?}"
    );
    assert!(
        rig.runner
            .lines()
            .iter()
            .any(|line| return line
                == "rustup target add wasm32-wasip1 --toolchain nightly-2026-10-30"),
        "{:?}",
        rig.runner.lines()
    );
}

#[test]
fn a_component_whose_listed_name_only_shares_its_prefix_is_missing() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    // `rustc-dev` is a different component from `rustc`, and its listing starts with the latter's name.
    tool.declaration.options.rust_components = Some(vec!["rustc".to_string()]);
    tool.declaration.options.rust_targets = None;
    let added: Vec<String> = reconcile_with(
        &rig,
        &tool,
        &format!("rustc-dev-x86_64-unknown-linux-gnu\n{THE_HOSTS_STANDARD_LIBRARY}"),
    )
    .expect("listed");
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(
        added[0].contains("added the declared component rustc,"),
        "{added:?}"
    );
}

#[test]
fn a_component_suffixed_with_a_triple_the_toolchain_does_not_have_is_missing() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    tool.declaration.options.rust_components = Some(vec!["clippy".to_string()]);
    tool.declaration.options.rust_targets = None;
    // The listed suffix is no target of this toolchain, so it names some other build of `clippy`.
    let added: Vec<String> = reconcile_with(
        &rig,
        &tool,
        &format!("clippy-aarch64-unknown-linux-gnu\n{THE_HOSTS_STANDARD_LIBRARY}"),
    )
    .expect("listed");
    assert_eq!(added.len(), 1, "{added:?}");
}

#[test]
fn a_failed_listing_fails_the_tool_before_any_add_runs() {
    let rig = Rig::new();
    rig.runner.fail(
        "rustup component list --installed",
        "error: no installed toolchains",
    );
    let mut added: Vec<String> = Vec::new();
    let error = reconcile_rust_parts(&rig.context(), &rust_tool(&rig), &mut added)
        .expect_err("the listing failed");
    assert!(
        error.message.contains(
            "rustup component list --installed failed for rust: error: no installed toolchains"
        ),
        "{error}"
    );
    assert!(added.is_empty());
    assert_eq!(
        rig.runner.lines(),
        vec!["rustup component list --installed --toolchain nightly-2026-10-30".to_string()]
    );
}

#[test]
fn a_tool_that_declares_no_parts_asks_rustup_nothing() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    tool.declaration.options.rust_components = None;
    tool.declaration.options.rust_targets = None;
    let mut absent: Vec<String> = Vec::new();
    reconcile_rust_parts(&rig.context(), &tool, &mut absent).expect("nothing to do");
    tool.declaration.options.rust_components = Some(Vec::new());
    tool.declaration.options.rust_targets = Some(Vec::new());
    let mut empty: Vec<String> = Vec::new();
    reconcile_rust_parts(&rig.context(), &tool, &mut empty).expect("nothing to do");
    assert!(absent.is_empty() && empty.is_empty());
    assert!(
        rig.runner.calls().is_empty(),
        "no listing is needed when the lock declares no part"
    );
}

#[test]
fn a_component_add_rereads_the_listing_before_adding_a_target() {
    let rig = Rig::new();
    // The first listing lacks the declared target's standard library; adding the component brings it,
    // so the second listing has it and no target add runs.
    rig.runner.answer(
        "rustup component list --installed --toolchain nightly-2026-10-30",
        &format!("rust-src\n{THE_HOSTS_STANDARD_LIBRARY}"),
        0,
    );
    // A rule registered with `answer_after` is consulted before the plain one, so this is the second read.
    rig.runner.answer_after(
        "rustup component add clippy",
        "rustup component list --installed --toolchain nightly-2026-10-30",
        &format!("rust-src\nclippy-x86_64-unknown-linux-gnu\n{THE_HOSTS_STANDARD_LIBRARY}rust-std-aarch64-linux-android\n"),
        0,
    );
    let mut added: Vec<String> = Vec::new();
    reconcile_rust_parts(&rig.context(), &rust_tool(&rig), &mut added)
        .expect("the component was added");
    assert_eq!(added.len(), 1, "only the component was added: {added:?}");
    assert!(
        !rig.runner
            .lines()
            .iter()
            .any(|line| return line.starts_with("rustup target add")),
        "{:?}",
        rig.runner.lines()
    );
}

#[test]
fn a_target_declared_twice_is_added_once() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    // The row's declared components are held, and one declared target is written twice.
    tool.declaration.options.rust_targets = Some(vec![
        "aarch64-linux-android".to_string(),
        "aarch64-linux-android".to_string(),
    ]);
    let added: Vec<String> = reconcile_with(
        &rig,
        &tool,
        &format!("rust-src\nclippy-x86_64-unknown-linux-gnu\n{THE_HOSTS_STANDARD_LIBRARY}"),
    )
    .expect("listed");
    assert_eq!(added.len(), 1, "one target, added once: {added:?}");
    assert_eq!(
        the_adds(&rig),
        vec!["rustup target add aarch64-linux-android --toolchain nightly-2026-10-30".to_string()]
    );
}

#[test]
fn a_bare_declaration_added_first_covers_the_qualified_declaration_after_it() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    // The first declaration names the component bare, the second names the same part with its triple.
    tool.declaration.options.rust_components = Some(vec![
        "clippy".to_string(),
        "clippy-x86_64-unknown-linux-gnu".to_string(),
    ]);
    tool.declaration.options.rust_targets = None;
    let added: Vec<String> =
        reconcile_with(&rig, &tool, THE_HOSTS_STANDARD_LIBRARY).expect("listed");
    assert_eq!(added.len(), 1, "{added:?}");
    assert_eq!(
        the_adds(&rig),
        vec!["rustup component add clippy --toolchain nightly-2026-10-30".to_string()]
    );
}

/// The listing that decides which parts are missing must never install a toolchain:
/// `rustup 1.29.2` answers `component list --installed --toolchain 1.79.0` for an absent channel by
/// downloading and installing it (`warn: the missing active toolchain ... has been auto-installed`),
/// so without `RUSTUP_AUTO_INSTALL=0` a query would be a network install.
#[test]
fn the_installed_parts_listing_asks_rustup_not_to_install_a_missing_toolchain() {
    let rig = Rig::new();
    reconcile_with(
        &rig,
        &rust_tool(&rig),
        &format!("rust-src\n{THE_HOSTS_STANDARD_LIBRARY}"),
    )
    .expect("listed");
    let listings: Vec<RunRequest> = rig
        .runner
        .calls()
        .into_iter()
        .filter(|call| return call.args.iter().any(|argument| return argument == "list"))
        .collect();
    assert!(
        !listings.is_empty(),
        "the reconcile asks rustup what it holds: {listings:?}"
    );
    for listing in listings {
        let env = listing.env.as_ref().expect("an explicit environment");
        assert_eq!(
            env.get("RUSTUP_AUTO_INSTALL").map(String::as_str),
            Some("0"),
            "a listing runs with auto-install off: {listing:?}"
        );
    }
}

#[test]
fn the_added_sentence_names_the_part_and_the_listing_that_missed_it() {
    let rig = Rig::new();
    let added: Vec<String> = reconcile_with(
        &rig,
        &rust_tool(&rig),
        "rust-src\nrust-std-aarch64-linux-android\n",
    )
    .expect("one component added");
    assert_eq!(
        added,
        vec![
            "rust: nightly-2026-10-30 was already installed; added the declared component clippy, which \
             `rustup component list --installed` did not show"
                .to_string()
        ]
    );
}

/// The installed-parts listing asks about the same channel `packageId` names,
/// not the locked version,
/// so a real install is not read as missing and re-added forever.
#[test]
fn the_installed_parts_are_listed_for_the_package_id_channel() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    tool.declaration.package_id = Some("nightly".to_string());
    rig.runner.answer(
        "rustup component list --installed --toolchain nightly",
        &format!(
            "rust-src\nclippy-x86_64-unknown-linux-gnu\n{THE_HOSTS_STANDARD_LIBRARY}rust-std-aarch64-linux-android\n"
        ),
        0,
    );
    let mut added: Vec<String> = Vec::new();
    reconcile_rust_parts(&rig.context(), &tool, &mut added).expect("listed");
    assert!(added.is_empty(), "{added:?}");
    let lines: Vec<String> = rig.runner.lines();
    assert!(
        lines
            .iter()
            .all(|line| return line.ends_with("--toolchain nightly")),
        "every listing and add names the installed channel: {lines:?}"
    );
}
