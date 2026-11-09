//! Unit tests of the Rust component and target reconciler,
//! against the fake runner and a disposable scope:
//! the tests assert that the `rustup` listings alone decide what is missing,
//! which listed names count as the declared part,
//! and that a listing that fails stops the reconcile before any add.

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

/// Answers the sample toolchain's two listings with these texts,
/// then reconciles the tool.
fn reconcile_with(rig: &Rig, tool: &LockedTool, targets: &str, components: &str) -> Result<Vec<String>, super::error::ToolchainError> {
    rig.runner.answer("rustup target list --installed --toolchain nightly-2026-10-30", targets, 0);
    rig.runner.answer("rustup component list --installed --toolchain nightly-2026-10-30", components, 0);
    let mut added: Vec<String> = Vec::new();
    return reconcile_rust_parts(&rig.context(), tool, &mut added).map(|()| return added)
}

#[test]
fn a_component_listed_with_its_host_triple_counts_as_installed() {
    let rig = Rig::new();
    let added: Vec<String> = reconcile_with(
        &rig,
        &rust_tool(&rig),
        "x86_64-unknown-linux-gnu\naarch64-linux-android\n",
        "clippy-x86_64-unknown-linux-gnu\nrust-src\n",
    )
    .expect("listed");
    assert!(added.is_empty(), "{added:?}");
    assert!(!rig.runner.lines().iter().any(|line| return line.contains(" add ")), "{:?}", rig.runner.lines());
}

#[test]
fn every_declared_part_counts_as_missing_when_rustup_lists_none() {
    let rig = Rig::new();
    let added: Vec<String> = reconcile_with(&rig, &rust_tool(&rig), "", "").expect("an empty listing is a listing");
    assert_eq!(added.len(), 3, "{added:?}");
    assert_eq!(
        rig.runner.lines(),
        vec![
            "rustup target list --installed --toolchain nightly-2026-10-30".to_string(),
            "rustup component list --installed --toolchain nightly-2026-10-30".to_string(),
            "rustup component add clippy --toolchain nightly-2026-10-30".to_string(),
            "rustup component add rust-src --toolchain nightly-2026-10-30".to_string(),
            "rustup target list --installed --toolchain nightly-2026-10-30".to_string(),
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
    let added: Vec<String> = reconcile_with(&rig, &tool, "x86_64-unknown-linux-gnu\nwasm32-wasip1-threads\n", "rust-src\n")
        .expect("listed");
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(added[0].contains("added the declared target wasm32-wasip1,"), "{added:?}");
    assert!(
        rig.runner.lines().contains(&"rustup target add wasm32-wasip1 --toolchain nightly-2026-10-30".to_string()),
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
    let added: Vec<String> =
        reconcile_with(&rig, &tool, "x86_64-unknown-linux-gnu\n", "rustc-dev-x86_64-unknown-linux-gnu\n").expect("listed");
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(added[0].contains("added the declared component rustc,"), "{added:?}");
}

#[test]
fn a_component_suffixed_with_a_triple_the_toolchain_does_not_have_is_missing() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    tool.declaration.options.rust_components = Some(vec!["clippy".to_string()]);
    tool.declaration.options.rust_targets = None;
    // The listed suffix is no target of this toolchain, so it names some other build of `clippy`.
    let added: Vec<String> = reconcile_with(&rig, &tool, "x86_64-unknown-linux-gnu\n", "clippy-aarch64-unknown-linux-gnu\n")
        .expect("listed");
    assert_eq!(added.len(), 1, "{added:?}");
}

#[test]
fn a_failed_target_listing_fails_the_tool_before_any_add_runs() {
    let rig = Rig::new();
    rig.runner.fail("rustup target list --installed", "error: no installed toolchains");
    let mut added: Vec<String> = Vec::new();
    let error = reconcile_rust_parts(&rig.context(), &rust_tool(&rig), &mut added).expect_err("the listing failed");
    assert!(error.message.contains("rustup target list --installed failed for rust: error: no installed toolchains"), "{error}");
    assert!(added.is_empty());
    assert_eq!(rig.runner.lines(), vec!["rustup target list --installed --toolchain nightly-2026-10-30".to_string()]);
}

#[test]
fn a_failed_component_listing_fails_the_tool_after_the_target_listing() {
    let rig = Rig::new();
    rig.runner.answer("rustup target list --installed --toolchain nightly-2026-10-30", "x86_64-unknown-linux-gnu\n", 0);
    rig.runner.fail("rustup component list --installed", "error: metadata is corrupt");
    let mut added: Vec<String> = Vec::new();
    let error = reconcile_rust_parts(&rig.context(), &rust_tool(&rig), &mut added).expect_err("the component listing failed");
    assert!(error.message.contains("rustup component list --installed failed for rust: error: metadata is corrupt"), "{error}");
    assert_eq!(
        rig.runner.lines(),
        vec![
            "rustup target list --installed --toolchain nightly-2026-10-30".to_string(),
            "rustup component list --installed --toolchain nightly-2026-10-30".to_string(),
        ]
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
    assert!(rig.runner.calls().is_empty(), "no listing is needed when the lock declares no part");
}

#[test]
fn a_component_add_rereads_the_target_listing_before_adding_a_target() {
    let rig = Rig::new();
    // The first target listing lacks the declared triple; adding the component brings its standard library,
    // so the second listing has it and no target add runs.
    rig.runner.answer("rustup target list --installed --toolchain nightly-2026-10-30", "x86_64-unknown-linux-gnu\n", 0);
    rig.runner.answer("rustup component list --installed --toolchain nightly-2026-10-30", "rust-src\n", 0);
    rig.runner.answer_after(
        "rustup component add clippy",
        "rustup target list --installed --toolchain nightly-2026-10-30",
        "x86_64-unknown-linux-gnu\naarch64-linux-android\n",
        0,
    );
    let mut added: Vec<String> = Vec::new();
    reconcile_rust_parts(&rig.context(), &rust_tool(&rig), &mut added).expect("the component was added");
    assert_eq!(added.len(), 1, "only the component was added: {added:?}");
    assert!(!rig.runner.lines().iter().any(|line| return line.starts_with("rustup target add")), "{:?}", rig.runner.lines());
}

#[test]
fn two_declarations_that_name_one_part_add_it_once() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    // The first declaration names the component with its triple, the second names the same part bare,
    // and the third repeats a target.
    tool.declaration.options.rust_components = Some(vec![
        "rust-std-x86_64-unknown-linux-gnu".to_string(),
        "rust-std".to_string(),
    ]);
    tool.declaration.options.rust_targets = Some(vec!["aarch64-linux-android".to_string(), "aarch64-linux-android".to_string()]);
    let added: Vec<String> = reconcile_with(&rig, &tool, "x86_64-unknown-linux-gnu\n", "rust-src\n").expect("listed");
    assert_eq!(added.len(), 2, "one component and one target: {added:?}");
    let adds: Vec<String> = rig.runner.lines().into_iter().filter(|line| return line.contains(" add ")).collect();
    assert_eq!(
        adds,
        vec![
            "rustup component add rust-std-x86_64-unknown-linux-gnu --toolchain nightly-2026-10-30".to_string(),
            "rustup target add aarch64-linux-android --toolchain nightly-2026-10-30".to_string(),
        ]
    );
}

#[test]
fn a_bare_declaration_added_first_covers_the_qualified_declaration_after_it() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    // The first declaration names the component bare, the second names the same part with its triple.
    tool.declaration.options.rust_components = Some(vec!["clippy".to_string(), "clippy-x86_64-unknown-linux-gnu".to_string()]);
    tool.declaration.options.rust_targets = None;
    let added: Vec<String> = reconcile_with(&rig, &tool, "x86_64-unknown-linux-gnu\n", "").expect("listed");
    assert_eq!(added.len(), 1, "{added:?}");
    let adds: Vec<String> = rig.runner.lines().into_iter().filter(|line| return line.contains(" add ")).collect();
    assert_eq!(adds, vec!["rustup component add clippy --toolchain nightly-2026-10-30".to_string()]);
}

/// The listing that decides which parts are missing must never install a toolchain:
/// `rustup 1.29.2` answers `component list --installed --toolchain 1.79.0` for an absent channel by
/// downloading and installing it (`warn: the missing active toolchain ... has been auto-installed`),
/// so without `RUSTUP_AUTO_INSTALL=0` a query would be a network install.
#[test]
fn the_installed_parts_listings_ask_rustup_not_to_install_a_missing_toolchain() {
    let rig = Rig::new();
    reconcile_with(&rig, &rust_tool(&rig), "x86_64-unknown-linux-gnu\n", "rust-src\n").expect("listed");
    let listings: Vec<RunRequest> = rig.runner.calls().into_iter().filter(|call| return call.args.contains(&"list".to_string())).collect();
    assert!(!listings.is_empty(), "the reconcile asks rustup what it holds: {listings:?}");
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
    let added: Vec<String> = reconcile_with(&rig, &rust_tool(&rig), "aarch64-linux-android\n", "rust-src\n").expect("one component added");
    assert_eq!(
        added,
        vec![
            "rust: nightly-2026-10-30 was already installed; added the declared component clippy, which \
             `rustup component list --installed` did not show"
                .to_string()
        ]
    );
}

/// The installed-parts listings ask about the same channel `packageId` names, not the locked version,
/// so a real install is not read as missing and re-added forever.
#[test]
fn the_installed_parts_are_listed_for_the_package_id_channel() {
    let rig = Rig::new();
    let mut tool: LockedTool = rust_tool(&rig);
    tool.declaration.package_id = Some("nightly".to_string());
    reconcile_with(&rig, &tool, "x86_64-unknown-linux-gnu\n", "rust-src\n").expect("listed");
    let lines: Vec<String> = rig.runner.lines();
    assert!(
        lines.iter().all(|line| return line.ends_with("--toolchain nightly")),
        "every listing and add names the installed channel: {lines:?}"
    );
}
