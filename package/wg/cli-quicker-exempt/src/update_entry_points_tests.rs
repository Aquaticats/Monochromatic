//! Verifies the update entry-point list with allowed and rejected command lines.
//! The list is checked against the repository's tasks,
//! verbs and launcher in the `source_tests` module.

/// Checks the list against the repository's `mise.toml` tasks,
/// the `mpm-shadow` grammar and the launcher script.
#[path = "update_entry_points_source_tests.rs"]
mod source_tests;

/// The list and the classifier.
use crate::install_roots::install_roots;
/// The list and the classifier.
use crate::update_entry_points::{
    classify,
    MatchReason,
    MISE_TASKS,
    MPM_SHADOW_VERBS,
    TOOLCHAIN_SCRIPT_TAILS,
};
/// Fixture paths.
use std::path::{Path, PathBuf};

/// Builds a raw command line from words.
fn words(items: &[&str]) -> Vec<Vec<u8>> {
    return items.iter().map(|item| return item.as_bytes().to_vec()).collect();
}

/// Roots for a fictional user whose home has two spellings.
fn roots() -> Vec<PathBuf> {
    return install_roots(Path::new("/home/tester"), Some(Path::new("/var/home/tester"))).roots;
}

/// Classifies a fixture command line run by the executable at `exe`.
fn classify_line(exe: &str, line: &[&str]) -> Option<MatchReason> {
    return classify(Path::new(exe), &words(line), &roots());
}

/// A launcher path in a fixture checkout,
/// built from a listed tail.
fn launcher_path(tail: &str) -> String {
    return format!("/repo/package{tail}");
}

/// Accepts every fetching mise task in `mise run`,
/// `mise r`
/// and bare forms.
#[test]
fn mise_runs_of_fetching_tasks_match() {
    for task in MISE_TASKS {
        assert_eq!(
            classify_line("/usr/local/bin/mise", &["mise", "run", task]),
            Some(MatchReason::MiseTask),
            "{task}"
        );
    }
    assert_eq!(classify_line("/usr/bin/mise", &["mise", "r", "toolchain:update:global"]), Some(MatchReason::MiseTask));
    assert_eq!(classify_line("/usr/bin/mise", &["mise", "toolchain:install"]), Some(MatchReason::MiseTask));
}

/// Rejects other mise commands and tasks that only mention a toolchain name inside another word,
/// and the offline toolchain tasks.
#[test]
fn other_mise_commands_do_not_match() {
    for line in [
        vec!["mise", "run", "toolchain:list"],
        vec!["mise", "run", "toolchain:list:global"],
        vec!["mise", "run", "toolchain:lock:check"],
        vec!["mise", "run", "lint"],
        vec!["mise", "ls"],
        vec!["mise", "run", "toolchain:installing"],
        vec!["mise", "run", "x:toolchain:install"],
        vec!["mise", "run", "toolchain:sync"],
    ] {
        assert_eq!(classify_line("/usr/bin/mise", &line), None, "{line:?}");
    }
    assert_eq!(classify_line("/usr/bin/other", &["other", "run", "toolchain:install"]), None);
}

/// Accepts the fetching `mpm-shadow` verbs with or without `-g`.
#[test]
fn mpm_shadow_fetching_verbs_match() {
    for verb in MPM_SHADOW_VERBS {
        assert_eq!(classify_line("/x/mpm-shadow", &["mpm-shadow", verb]), Some(MatchReason::MpmShadowVerb), "{verb}");
        assert_eq!(classify_line("/x/mpm-shadow", &["mpm-shadow", "-g", verb]), Some(MatchReason::MpmShadowVerb));
        assert_eq!(
            classify_line("/x/mpm-shadow", &["mpm-shadow", verb, "--global"]),
            Some(MatchReason::MpmShadowVerb)
        );
    }
}

/// Rejects the offline and read-only `mpm-shadow` verbs.
#[test]
fn mpm_shadow_offline_verbs_do_not_match() {
    for verb in ["list", "ls", "bin", "verify-archive", "prune"] {
        assert_eq!(classify_line("/x/mpm-shadow", &["mpm-shadow", verb]), None, "{verb}");
        assert_eq!(classify_line("/x/mpm-shadow", &["mpm-shadow", "-g", verb]), None, "{verb}");
    }
    assert_eq!(classify_line("/x/mpm-shadow", &["mpm-shadow"]), None);
    assert_eq!(classify_line("/x/other", &["other", "install"]), None);
}

/// Accepts the `mpm` shadow by its name and rejects a lookalike name.
#[test]
fn the_mpm_shadow_matches_by_name() {
    assert_eq!(classify_line("/x/mpm-shadow", &["mpm", "list"]), Some(MatchReason::MpmShim));
    assert_eq!(classify_line("/home/tester/.cargo/bin/mpm", &["/home/tester/.cargo/bin/mpm"]), Some(MatchReason::MpmShim));
    assert_eq!(classify_line("/usr/bin/mpmx", &["mpmx", "install"]), None);
}

/// A first command-line word set by `exec -a` never makes another executable an entry point,
/// because only the resolved executable name is trusted.
#[test]
fn a_forged_first_word_does_not_match() {
    let launcher = launcher_path(TOOLCHAIN_SCRIPT_TAILS[0]);
    assert_eq!(classify_line("/usr/bin/curl", &["mpm", "https://example.org"]), None);
    assert_eq!(classify_line("/usr/bin/sleep", &["/usr/bin/mpm", "30"]), None);
    assert_eq!(classify_line("/usr/bin/curl", &["mise", "run", "toolchain:install"]), None);
    assert_eq!(classify_line("/usr/bin/curl", &["mpm-shadow", "install"]), None);
    assert_eq!(classify_line("/usr/bin/curl", &["node", &launcher]), None);
    assert_eq!(classify_line("/usr/bin/curl", &["tool"]), None);
}

/// Names that belong to the entry points match through the executable even when the first word is odd,
/// so the check stays usable for real launches.
#[test]
fn the_real_executables_match_whatever_the_first_word_says() {
    let launcher = launcher_path(TOOLCHAIN_SCRIPT_TAILS[0]);
    assert_eq!(classify_line("/usr/bin/mise", &["/opt/x/mise", "run", "toolchain:install"]), Some(MatchReason::MiseTask));
    assert_eq!(classify_line("/x/mpm-shadow", &["/x/mpm-shadow", "update"]), Some(MatchReason::MpmShadowVerb));
    assert_eq!(classify_line("/x/mpm-shadow", &["/home/tester/.cargo/bin/mpm", "list"]), Some(MatchReason::MpmShim));
    assert_eq!(classify_line("/usr/bin/node", &["node", &launcher, "update"]), Some(MatchReason::ToolchainScript));
}

/// Accepts the launcher run by node,
/// from the source and from the built entry,
/// and rejects other scripts and lookalike paths.
#[test]
fn toolchain_scripts_run_by_node_match() {
    for tail in TOOLCHAIN_SCRIPT_TAILS {
        let script = launcher_path(tail);
        assert_eq!(classify_line("/usr/bin/node", &["node", &script]), Some(MatchReason::ToolchainScript), "{script}");
        assert_eq!(
            classify_line("/usr/bin/node", &["node", &script, "--global"]),
            Some(MatchReason::ToolchainScript)
        );
        assert_eq!(classify_line("/usr/bin/other", &["other", &script]), None, "{script}");
        let backup = format!("{script}.bak");
        assert_eq!(classify_line("/usr/bin/node", &["node", &backup]), None, "{backup}");
        let sibling_package = script.replace("core-exec", "core-exec-evil");
        assert_eq!(classify_line("/usr/bin/node", &["node", &sibling_package]), None, "{sibling_package}");
    }
    for script in [
        "/repo/package/monorepo-task/core-exec/src/mpm-shadow.unit.test.ts",
        "/repo/package/monorepo-task/core-exec/src/mpm-shadow-lock-read.ts",
        "/repo/package/other/mpm-shadow.ts",
        "/repo/package/file-enforcer/core/src/toolchain/sync.ts",
    ] {
        assert_eq!(classify_line("/usr/bin/node", &["node", script]), None, "{script}");
    }
}

/// Accepts executables below either root in both home spellings and rejects siblings and lookalikes.
#[test]
fn executables_under_the_install_roots_match() {
    for exe in [
        "/var/home/tester/.local/share/monochromatic-dev/toolchain/node/26.13.0/bin/node",
        "/home/tester/.local/share/monochromatic-dev/toolchain-global/uv/0.13.6/uv",
        "/var/home/tester/.local/share/monochromatic-dev/toolchain/cargo/fd-find/10.4.2/bin/fd",
    ] {
        assert_eq!(classify_line(exe, &["tool"]), Some(MatchReason::ToolchainPath), "{exe}");
    }
    for exe in [
        "/usr/bin/curl",
        "/var/home/tester/.local/share/monochromatic-dev/toolchain-evil/bin/x",
        "/var/home/tester/.local/share/monochromatic-dev/tool/x",
        "/var/home/other/.local/share/monochromatic-dev/toolchain/bin/x",
        "/var/home/tester/.local/share/mise/installs/node/26.10.0/bin/node",
        "/tmp/var/home/tester/.local/share/monochromatic-dev/toolchain/bin/x",
        "/var/home/tester/.cargo/bin/rustup",
    ] {
        assert_eq!(classify_line(exe, &["tool"]), None, "{exe}");
    }
}

/// Builds the install roots without a canonical spelling when the home does not exist.
#[test]
fn install_roots_list_each_spelling_once() {
    assert_eq!(install_roots(Path::new("/home/a"), None).roots.len(), 2);
    assert_eq!(install_roots(Path::new("/home/a"), Some(Path::new("/home/a"))).roots.len(), 2);
    assert_eq!(install_roots(Path::new("/home/a"), Some(Path::new("/var/home/a"))).roots.len(), 4);
}
