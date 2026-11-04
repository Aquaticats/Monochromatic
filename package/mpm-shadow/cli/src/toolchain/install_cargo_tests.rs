//! Unit tests of the `cargo binstall` command for one crate:
//! a crate this repository publishes never falls back to compiling,
//! and every other crate keeps cargo-binstall's compile fallback.
//! Each runs against the fake runner inside a disposable scope;
//! nothing here installs anything.

use crate::constants::OWN_CRATE_PREFIX;

use super::cargo_installs::{is_repository_published, version_directory};
use super::fakes::Rig;
use super::binstall_no_archive::COMPILE_DISABLED_TEXT;
use super::error::ErrorKind;
use super::install::install_tool;

/// The sample lock's repository-published crate.
const PUBLISHED: &str = "cargo:monochromatic-mpm-shadow";

/// The sample lock's third-party crate.
const THIRD_PARTY: &str = "cargo:fd-find";

/// The two arguments that switch cargo-binstall's build-from-source fallback and its third-party prebuilt-binary host
/// (quick-install) off, leaving only the crate's own published binstall metadata
/// (owner decision 2026-11-04, "A: one flag, our crates").
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
/// The report wraps the sentence at the terminal width, as the real one did.
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
