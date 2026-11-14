//! Unit tests of the `cargo binstall` command for one crate:
//! a crate this repository publishes never falls back to compiling,
//! and every other crate keeps cargo-binstall's compile fallback.
//! Each runs against the fake runner inside a disposable scope;
//! nothing here installs anything.

use crate::constants::OWN_CRATE_PREFIX;

/// Imports `fs` from `std`.
use std::fs;

/// Imports `cargo_installs`' readers and the same-version removal.
use super::cargo_installs::{is_repository_published, version_directory};
/// Imports `Rig` from `super::fakes`.
use super::fakes::Rig;
/// Imports the fakes that install a crate the way cargo does.
use super::fakes_cargo::{fake_binstall, finish_install};
/// Imports `binstall_no_archive`'s refusal text.
use super::binstall_no_archive::COMPILE_DISABLED_TEXT;
/// Imports `ErrorKind` from `super::error`.
use super::error::ErrorKind;
/// Imports `install_tool` from `super::install`.
use super::install::install_tool;
/// Imports `second_tree` from the release-archive tests,
/// so a lock of another worktree is a real second tree of the same home.
use super::published_archive_tests::second_tree;
/// Imports the keeps every removal asks from `super::store_keeps`.
use super::store_keeps::InstallKeeps;
/// Imports the started program a holder test needs.
use super::store_keeps_tests::StandIn;

/// The sample lock's repository-published crate.
const PUBLISHED: &str = "cargo:monochromatic-mpm-shadow";

/// The sample lock's third-party crate.
const THIRD_PARTY: &str = "cargo:fd-find";

/// What:
/// The crate the removal controls install:
/// a repository-published name no process on any machine calls itself.
/// Why:
/// The keep rule also keeps every version of a tool whose store directory name matches the name of a process the kernel
/// hides,
/// and this machine runs such a process for the git wrapper,
/// so a fixture about the git wrapper would answer "kept" before the rule was written.
const FIXTURE_CRATE: &str = "monochromatic-store-keeps-fixture";

/// The older version another worktree's lock names.
const EARLIER: &str = "0.4.4";

/// The older version nothing names and nothing runs.
const UNNAMED: &str = "0.4.5";

/// The version this install brings,
/// newer than both.
const NEWEST: &str = "0.4.6";

/// The lock text of a worktree that pins `version` of the fixture crate.
fn lock_naming(version: &str) -> String {
    return format!(
        "{{\"tools\":{{\"cargo:{FIXTURE_CRATE}\":{{\"request\":\"cargo:{FIXTURE_CRATE}@latest\",\"route\":\"cargo-binstall\",\"version\":\"{version}\",\"bins\":[\"mpm-shadow\"],\"packageId\":\"{FIXTURE_CRATE}\"}}}}}}"
    );
}

/// A rig whose fixture crate is installed at `version`,
/// with cargo's own record inside the version directory.
fn install_finished(rig: &Rig, version: &str) {
    finish_install(&version_directory(&rig.scope.toolchain_root, FIXTURE_CRATE, version), FIXTURE_CRATE, version, &["mpm-shadow"]);
}

/// The tool of the sample lock renamed to install the fixture crate at `version`.
fn fixture_tool(rig: &Rig, version: &str) -> super::model::LockedTool {
    let mut tool = tool_installing(rig, PUBLISHED, FIXTURE_CRATE);
    tool.version = version.to_string();
    return tool;
}

/// The two arguments that switch cargo-binstall's build-from-source fallback and its third-party prebuilt-binary host
/// (quick-install) off,
/// leaving only the crate's own published binstall metadata
/// (owner decision 2026-11-04,
/// "A:
/// one flag,
/// our crates").
const DISABLE_COMPILE: [&str; 2] = ["--disable-strategies", "compile,quick-install"];

/// The tool of `id` renamed to install the crate `crate_name`.
fn tool_installing(rig: &Rig, id: &str, crate_name: &str) -> super::model::LockedTool {
    let mut tool = rig.tool(id);
    tool.declaration.package_id = Some(crate_name.to_string());
    return tool;
}

/// The `cargo binstall` requests the rig's runner received (the token lookup is another request).
fn binstall_calls(rig: &Rig) -> Vec<super::runner::RunRequest> {
    return rig.runner.calls().into_iter().filter(|call| return call.args.first().map(String::as_str) == Some("binstall")).collect();
}

/// The `cargo binstall` lines the rig's runner received.
fn binstall_lines(rig: &Rig) -> Vec<String> {
    return rig.runner.lines().into_iter().filter(|line| return line.starts_with("cargo binstall")).collect();
}

/// Whether the arguments of the one `cargo binstall` that installing `crate_name` started disable compiling.
fn install_disables_compile(crate_name: &str) -> bool {
    let rig = Rig::new();
    let tool = tool_installing(&rig, THIRD_PARTY, crate_name);
    install_tool(&rig.context(), &tool).expect("installed");
    let calls = binstall_calls(&rig);
    assert_eq!(calls.len(), 1, "one cargo binstall for {crate_name}");
    return calls[0].args.windows(DISABLE_COMPILE.len()).any(|window| return window == DISABLE_COMPILE);
}

#[test]
fn a_repository_crate_install_disables_the_compile_strategy() {
    let rig = Rig::new();
    install_tool(&rig.context(), &rig.tool(PUBLISHED)).expect("installed");
    let root = version_directory(&rig.scope.toolchain_root, "monochromatic-mpm-shadow", &rig.tool(PUBLISHED).version);
    assert_eq!(
        binstall_lines(&rig),
        vec![format!(
            "cargo binstall --no-confirm --root {} --version {} --disable-strategies compile,quick-install monochromatic-mpm-shadow",
            root.display(),
            rig.tool(PUBLISHED).version
        )]
    );
}

#[test]
fn a_repository_crate_install_passes_no_url_and_leaves_resolution_to_binstall() {
    let rig = Rig::new();
    install_tool(&rig.context(), &rig.tool(PUBLISHED)).expect("installed");
    let calls = binstall_calls(&rig);
    for flag in ["--pkg-url", "--pkg-fmt", "--bin-dir", "--strategies"] {
        assert!(!calls[0].args.iter().any(|argument| return argument == flag), "{flag} would point at a release by hand");
    }
}

#[test]
fn a_third_party_crate_install_keeps_binstalls_default_strategies() {
    let rig = Rig::new();
    install_tool(&rig.context(), &rig.tool(THIRD_PARTY)).expect("installed");
    let calls = binstall_calls(&rig);
    assert!(!calls[0].args.iter().any(|argument| return argument.contains("quick-install") || argument == "--disable-strategies"));
}

#[test]
fn a_third_party_crate_install_keeps_the_compile_fallback() {
    let rig = Rig::new();
    install_tool(&rig.context(), &rig.tool(THIRD_PARTY)).expect("installed");
    let root = version_directory(&rig.scope.toolchain_root, "fd-find", &rig.tool(THIRD_PARTY).version);
    assert_eq!(
        binstall_lines(&rig),
        vec![format!("cargo binstall --no-confirm --root {} --version {} fd-find", root.display(), rig.tool(THIRD_PARTY).version)]
    );
}

#[test]
fn names_that_merely_contain_the_repository_prefix_keep_the_compile_fallback() {
    let bare: &str = OWN_CRATE_PREFIX.trim_end_matches('-');
    let lookalikes: Vec<String> = vec![
        format!("not-{OWN_CRATE_PREFIX}x"),
        format!("x-{OWN_CRATE_PREFIX}y"),
        format!("{bare}x"),
        bare.to_string(),
        format!("{}-x", bare.replace('c', "k")),
    ];
    for name in lookalikes {
        assert!(!install_disables_compile(&name), "{name} is not a repository crate");
        let rig = Rig::new();
        assert!(!is_repository_published(&tool_installing(&rig, THIRD_PARTY, &name)), "{name}");
    }
}

#[test]
fn every_spelling_crates_io_treats_as_a_repository_crate_disables_the_compile_strategy() {
    let spellings: Vec<String> = vec![
        format!("{OWN_CRATE_PREFIX}x"),
        format!("{}x", OWN_CRATE_PREFIX.to_uppercase()),
        format!("{}X", OWN_CRATE_PREFIX.replacen('m', "M", 1)),
        format!("{}x", OWN_CRATE_PREFIX.replace('-', "_")),
    ];
    for name in spellings {
        assert!(install_disables_compile(&name), "{name} is a repository crate");
    }
}

/// What cargo-binstall 1.26.2 printed for the wrapper at 0.5.34 (measured in `cargo binstall --dry-run`):
/// the archive was found but lacked a program the crate declares.
/// The report wraps the sentence at the terminal width,
/// as the real one did.
const MISSING_PROGRAM_OUTPUT: &str = "\
cargo-binstall:  INFO resolve: Resolving package: 'monochromatic-git-policy-cli@=0.5.34'
cargo-binstall: ERROR resolve: When resolving monochromatic-git-policy-cli bin cli-git-service is not found. This binary is not optional so it must be included in the archive, please contact with upstream to fix this
cargo-binstall:  WARN resolve: Error while downloading and extracting from fetcher github.com: failed to find or install binaries: bin file /x/cli-git-service not found
cargo-binstall: ERROR Fatal error:
  \u{d7} For crate monochromatic-git-policy-cli: Fallback to cargo-install is
  \u{2502} disabled
  \u{2570}\u{2500}\u{25b6} Fallback to cargo-install is disabled
";

/// The message of the error a failing install of `id` returns when `cargo binstall` prints `output`.
fn failure_message(id: &str, output: &str) -> String {
    let rig = Rig::new();
    rig.runner.fail("cargo binstall", output);
    let error = install_tool(&rig.context(), &rig.tool(id)).expect_err("the install fails");
    assert_eq!(error.kind, ErrorKind::Failure);
    return error.message;
}

#[test]
fn a_repository_crate_without_a_fitting_archive_fails_naming_the_tool_the_reason_and_every_remedy() {
    let message = failure_message(PUBLISHED, MISSING_PROGRAM_OUTPUT);
    let version = Rig::new().tool(PUBLISHED).version;
    for expected in [
        PUBLISHED,
        "crate monochromatic-mpm-shadow",
        &format!("version {version}"),
        "no prebuilt release archive",
        "never compiled from source",
        "ERROR resolve: When resolving monochromatic-git-policy-cli bin cli-git-service is not found.",
        "WARN resolve: Error while downloading and extracting from fetcher github.com",
        COMPILE_DISABLED_TEXT,
        "Publish a release of the crate",
        "Lock a version whose release archive fits",
        &format!("mpm-shadow update {PUBLISHED}"),
        "GITHUB_TOKEN",
        "gh auth login",
        "BINSTALL_MAXIMUM_RESOLUTION_TIMEOUT",
    ] {
        assert!(message.contains(expected), "missing {expected:?} in:\n{message}");
    }
}

#[test]
fn repeated_timeout_reasons_are_quoted_once() {
    let line = "cargo-binstall:  WARN resolve: Timeout reached while checking fetcher invalid url: deadline has elapsed";
    let output = format!("{line}\n{line}\n{line}\ncargo-binstall: ERROR Fatal error:\n  \u{d7} For crate c: {COMPILE_DISABLED_TEXT}\n");
    let message = failure_message(PUBLISHED, &output);
    assert_eq!(message.matches("Timeout reached while checking fetcher").count(), 1, "{message}");
}

#[test]
fn a_repository_crate_failing_for_another_reason_keeps_the_generic_failure() {
    assert_eq!(
        failure_message(PUBLISHED, "crate not found"),
        format!("cargo binstall failed for {PUBLISHED}: crate not found")
    );
}

#[test]
fn a_third_party_crate_keeps_the_generic_failure_even_when_compiling_was_disabled_from_outside() {
    assert_eq!(
        failure_message(THIRD_PARTY, MISSING_PROGRAM_OUTPUT),
        format!("cargo binstall failed for {THIRD_PARTY}: {}", MISSING_PROGRAM_OUTPUT.trim())
    );
}

//region what the removal of an older version keeps

/// What:
/// An install of a newer version of a repository-published crate leaves the version another worktree's lock names,
/// and still removes the version nothing names and nothing runs.
/// Why:
/// The newest-only rule took the lock-named version away,
/// so the worktree that pins it installed it again on its next sync,
/// and the next tree that installed the newer one removed it again (seen on this machine on 2026-11-13).
#[test]
fn installing_a_newer_version_keeps_the_version_another_worktree_lock_names_and_removes_the_one_nothing_names() {
    let rig = Rig::new();
    rig.runner.on("cargo binstall", fake_binstall(&["mpm-shadow"]));
    install_finished(&rig, EARLIER);
    install_finished(&rig, UNNAMED);
    let other = second_tree(&rig);
    fs::write(&other.lock_file, lock_naming(EARLIER)).expect("write the other tree's lock");
    rig.runner.answer(
        "git -C",
        &format!("worktree {}\nHEAD a\n\nworktree {}\nHEAD b\n\n", rig.scope.root.display(), other.root.display()),
        0,
    );

    install_tool(&rig.context(), &fixture_tool(&rig, NEWEST)).expect("installed");
    let root = rig.scope.toolchain_root.clone();
    assert!(version_directory(&root, FIXTURE_CRATE, EARLIER).is_dir(), "the version the other worktree's lock names stays");
    assert!(!version_directory(&root, FIXTURE_CRATE, UNNAMED).exists(), "the version nothing names and nothing runs is still removed");
    assert!(version_directory(&root, FIXTURE_CRATE, NEWEST).is_dir(), "the version this install wrote stays");
}

/// What:
/// An install of a newer version of a repository-published crate leaves the version a process of this account runs,
/// and still removes the version nothing names and nothing runs.
/// Why:
/// The newest-only rule unlinked versions 38 processes of this account were executing (measured 2026-11-13),
/// and a service whose program went away cannot start again until an install puts it back.
#[test]
fn installing_a_newer_version_keeps_the_version_a_process_of_this_account_runs() {
    let rig = Rig::new();
    rig.runner.on("cargo binstall", fake_binstall(&["mpm-shadow"]));
    install_finished(&rig, EARLIER);
    install_finished(&rig, UNNAMED);
    let root = rig.scope.toolchain_root.clone();
    let stand_in: StandIn = StandIn::started_from(&version_directory(&root, FIXTURE_CRATE, EARLIER).join("bin").join("mpm-shadow"));

    install_tool(&rig.context(), &fixture_tool(&rig, NEWEST)).expect("installed");
    assert!(version_directory(&root, FIXTURE_CRATE, EARLIER).is_dir(), "the version this account runs stays");
    assert!(!version_directory(&root, FIXTURE_CRATE, UNNAMED).exists(), "the version nothing names and nothing runs is still removed");
    // Control, asked after the fact: the rule itself names this process, so the survival above is this keep and not
    // another one. A fixture whose stand-in the scan could not see fails here rather than passing quietly.
    let keeps: InstallKeeps = InstallKeeps::of(&rig.context());
    let reason: String = keeps
        .keep_reason(&format!("cargo/{FIXTURE_CRATE}"), EARLIER, &version_directory(&root, FIXTURE_CRATE, EARLIER))
        .expect("a process of this account runs it");
    assert!(reason.contains(&format!("pid {}", stand_in.pid())), "the rule names the process: {reason}");
    stand_in.stop();
}

//endregion
