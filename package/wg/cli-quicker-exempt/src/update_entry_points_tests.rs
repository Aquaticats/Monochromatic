//! Verifies the update entry-point list with allowed and rejected command lines,
//! and against the toolchain's current task names and `mpm-shadow` verbs.

/// The list and the classifier.
use crate::update_entry_points::{
    classify,
    install_roots,
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
    return install_roots(Path::new("/home/tester"), Some(Path::new("/var/home/tester")));
}

/// Classifies a fixture command line run by the executable at `exe`.
fn classify_line(exe: &str, line: &[&str]) -> Option<MatchReason> {
    return classify(Path::new(exe), &words(line), &roots());
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

/// Rejects other mise commands and tasks that only mention a toolchain name inside another word.
#[test]
fn other_mise_commands_do_not_match() {
    for line in [
        vec!["mise", "run", "toolchain:status"],
        vec!["mise", "run", "lint"],
        vec!["mise", "ls"],
        vec!["mise", "run", "toolchain:syncing"],
        vec!["mise", "run", "x:toolchain:sync"],
    ] {
        assert_eq!(classify_line("/usr/bin/mise", &line), None, "{line:?}");
    }
    assert_eq!(classify_line("/usr/bin/other", &["other", "run", "toolchain:sync"]), None);
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
    for verb in ["list", "bin", "verify-archive", "prune"] {
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
    assert_eq!(classify_line("/usr/bin/curl", &["mpm", "https://example.org"]), None);
    assert_eq!(classify_line("/usr/bin/sleep", &["/usr/bin/mpm", "30"]), None);
    assert_eq!(classify_line("/usr/bin/curl", &["mise", "run", "toolchain:sync"]), None);
    assert_eq!(classify_line("/usr/bin/curl", &["mpm-shadow", "install"]), None);
    assert_eq!(classify_line("/usr/bin/curl", &["node", "/repo/package/file-enforcer/core/src/toolchain/sync.ts"]), None);
    assert_eq!(classify_line("/usr/bin/curl", &["tool"]), None);
}

/// Names that belong to the entry points match through the executable even when the first word is odd,
/// so the check stays usable for real launches.
#[test]
fn the_real_executables_match_whatever_the_first_word_says() {
    assert_eq!(classify_line("/usr/bin/mise", &["/opt/x/mise", "run", "toolchain:sync"]), Some(MatchReason::MiseTask));
    assert_eq!(classify_line("/x/mpm-shadow", &["/x/mpm-shadow", "update"]), Some(MatchReason::MpmShadowVerb));
    assert_eq!(classify_line("/x/mpm-shadow", &["/home/tester/.cargo/bin/mpm", "list"]), Some(MatchReason::MpmShim));
    assert_eq!(classify_line("/usr/bin/node", &["node", "/repo/package/file-enforcer/core/src/toolchain/update.ts"]), Some(MatchReason::ToolchainScript));
}

/// Accepts the TypeScript toolchain scripts run by node and rejects other scripts.
#[test]
fn toolchain_scripts_run_by_node_match() {
    for tail in TOOLCHAIN_SCRIPT_TAILS {
        let script = format!("/repo/package{tail}");
        assert_eq!(classify_line("/usr/bin/node", &["node", &script]), Some(MatchReason::ToolchainScript), "{script}");
        assert_eq!(
            classify_line("/usr/bin/node", &["node", &script, "--global"]),
            Some(MatchReason::ToolchainScript)
        );
    }
    assert_eq!(
        classify_line("/usr/bin/node", &["node", "/repo/package/file-enforcer/core/src/toolchain/status.ts"]),
        None
    );
    assert_eq!(classify_line("/usr/bin/node", &["node", "/repo/package/other/sync.ts"]), None);
    assert_eq!(classify_line("/usr/bin/other", &["other", "/repo/package/file-enforcer/core/src/toolchain/sync.ts"]), None);
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
    assert_eq!(install_roots(Path::new("/home/a"), None).len(), 2);
    assert_eq!(install_roots(Path::new("/home/a"), Some(Path::new("/home/a"))).len(), 2);
    assert_eq!(install_roots(Path::new("/home/a"), Some(Path::new("/var/home/a"))).len(), 4);
}

/// The repository root,
/// two levels above this package's `package/wg/cli-quicker-exempt` directory.
fn repository_root() -> PathBuf {
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
}

/// Reads one repository file.
fn read_repository_file(relative: &str) -> String {
    return std::fs::read_to_string(repository_root().join(relative)).expect(relative);
}

/// Splits `mise.toml` into task blocks named by their `[tasks."name"]` or `[tasks.name]` header.
fn mise_task_blocks(text: &str) -> Vec<(String, String)> {
    let mut blocks: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        if let Some(header) = line.strip_prefix("[tasks.") {
            let name = header.trim_end_matches(']').trim_matches('"').to_string();
            blocks.push((name, String::new()));
        } else if line.starts_with('[') {
            blocks.push((String::new(), String::new()));
        } else if let Some(last) = blocks.last_mut() {
            last.1.push_str(line);
            last.1.push('\n');
        }
    }
    return blocks;
}

/// Reports whether a task body fetches from the network:
/// it runs the TypeScript sync or update script,
/// or runs a toolchain task that does.
fn task_fetches(body: &str) -> bool {
    return body.contains("toolchain/sync.ts")
        || body.contains("toolchain/update.ts")
        || body.contains("mise run toolchain:sync")
        || body.contains("mise run toolchain:update");
}

/// Every task in the generated `mise.toml` that fetches must be on the list.
#[test]
fn every_fetching_mise_task_is_listed() {
    let blocks = mise_task_blocks(&read_repository_file("mise.toml"));
    let mut fetching: Vec<String> = Vec::new();
    for (name, body) in &blocks {
        if !name.is_empty() && task_fetches(body) {
            fetching.push(name.clone());
        }
    }
    assert!(fetching.contains(&"toolchain:sync".to_string()), "{fetching:?}");
    assert!(fetching.contains(&"toolchain:update:global".to_string()), "{fetching:?}");
    for name in &fetching {
        assert!(MISE_TASKS.contains(&name.as_str()), "mise task {name} fetches but is not on the wg list");
    }
}

/// Task names the toolchain cutover adds under new spellings of `sync` and `status`.
/// They are on the list before `mise.toml` has them,
/// so a run started the day the cutover lands is already covered.
const CUTOVER_PENDING: &[&str] = &["toolchain:install", "toolchain:install:global", "toolchain:outdated", "toolchain:outdated:global"];

/// Every listed name is a real task,
/// except the cutover names that do not exist yet;
/// the test fails the day one of them appears,
/// so the pending list shrinks with the cutover.
#[test]
fn every_listed_mise_task_exists_unless_the_cutover_is_pending() {
    let blocks = mise_task_blocks(&read_repository_file("mise.toml"));
    let names: Vec<&str> = blocks.iter().map(|(name, _body)| return name.as_str()).collect();
    for listed in MISE_TASKS {
        let exists = names.contains(listed);
        let pending = CUTOVER_PENDING.contains(listed);
        assert!(exists || pending, "{listed} is on the wg list but no mise task has that name");
        assert!(!(exists && pending), "{listed} now exists in mise.toml: remove it from CUTOVER_PENDING");
    }
    for pending in CUTOVER_PENDING {
        assert!(MISE_TASKS.contains(pending), "{pending} is pending but not on the wg list");
    }
}

/// The offline status tasks must stay off the list.
#[test]
fn offline_mise_tasks_are_not_listed() {
    for name in ["toolchain:status", "toolchain:status:global", "toolchain:list", "toolchain:bin"] {
        assert!(!MISE_TASKS.contains(&name), "{name}");
    }
}

/// The names of the `mpm-shadow` commands declared in its grammar.
fn mpm_shadow_command_names() -> Vec<String> {
    let text = read_repository_file("package/mpm-shadow/cli/src/toolchain/commands.rs");
    let mut names: Vec<String> = Vec::new();
    for part in text.split("Command::new(\"").skip(1) {
        if let Some(end) = part.find('"') {
            names.push(part[..end].to_string());
        }
    }
    return names;
}

/// Every listed `mpm-shadow` verb must still exist,
/// and the three fetching verbs the toolchain README names must be listed.
#[test]
fn listed_mpm_shadow_verbs_match_the_current_grammar() {
    let names = mpm_shadow_command_names();
    for verb in MPM_SHADOW_VERBS {
        if *verb == "i" {
            continue;
        }
        assert!(names.contains(&(*verb).to_string()), "mpm-shadow has no verb {verb}: {names:?}");
    }
    for verb in ["install", "update", "outdated"] {
        assert!(MPM_SHADOW_VERBS.contains(&verb), "{verb}");
    }
    let grammar = read_repository_file("package/mpm-shadow/cli/src/toolchain/commands.rs");
    assert!(grammar.contains("\"i\""), "the short alias of install changed");
}

/// The listed TypeScript scripts must exist in the repository.
#[test]
fn listed_toolchain_scripts_exist() {
    for tail in TOOLCHAIN_SCRIPT_TAILS {
        let relative = format!("package{tail}");
        assert!(repository_root().join(&relative).is_file(), "{relative}");
    }
}
