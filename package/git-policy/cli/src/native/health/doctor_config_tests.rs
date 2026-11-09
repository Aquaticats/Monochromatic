//! Controls for the doctor's configuration findings against real repositories.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};

/// The findings of `cwd`,
/// read by a task of the runtime,
/// because reading them asks Git.
fn findings_of(context: &GitContext, cwd: &Path) -> ConfigFindings {
    return crate::lazy_runtime::run_on_runtime(config_findings(context, cwd)).expect("the runtime");
}

/// Configured hooks are found,
/// listed one per line,
/// and explained with the ways forward.
#[test]
fn configured_hooks_are_listed_and_explained() {
    let root: PathBuf = fixture("doctor-config-hooks");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["config", "hook.review.event", "post-commit"]);
    git(repo.as_path(), &["config", "hook.review.command", "run it"]);
    let findings: ConfigFindings = findings_of(&git_context(), repo.as_path());
    assert_eq!(findings.hooks.len(), 1);
    assert_eq!(render_hook_lines(&findings), "hook.review (events: post-commit)\n");
    let notes: String = render_notes(&findings);
    assert!(notes.contains("runs no program named by any configuration") && notes.contains("--unset-all"), "{notes}");
    assert!(!findings.retired_hooks_key);
    remove(root.as_path());
}

/// A repository with no hook and no `hooks` key has nothing to report.
#[test]
fn a_clean_repository_reports_nothing() {
    let root: PathBuf = fixture("doctor-config-clean");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let findings: ConfigFindings = findings_of(&git_context(), repo.as_path());
    assert_eq!(render_hook_lines(&findings), "");
    assert_eq!(render_notes(&findings), "");
    remove(root.as_path());
}

/// The retired `indexLock` and `landing` keys are named with the remedy,
/// each alone and both together,
/// and a file
/// without them reports nothing.
#[test]
fn the_inert_index_lock_and_landing_keys_are_named_with_the_remedy() {
    let root: PathBuf = fixture("doctor-config-inert");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let config: PathBuf = repo.join("monochromatic-git-policy-cli.config.jsonc");
    std::fs::write(&config, "{ \"policies\": {} }\n").expect("config");
    let neither: ConfigFindings = findings_of(&git_context(), repo.as_path());
    assert!(!neither.retired_index_lock_key && !neither.retired_landing_key);
    assert_eq!(render_notes(&neither), "");
    std::fs::write(&config, "{ \"indexLock\": { \"unprovenOwnerTimeoutMs\": 5000 } }\n").expect("config");
    let lock_only: ConfigFindings = findings_of(&git_context(), repo.as_path());
    assert!(lock_only.retired_index_lock_key && !lock_only.retired_landing_key && !lock_only.retired_hooks_key);
    let lock_notes: String = render_notes(&lock_only);
    assert!(lock_notes.contains("`indexLock.unprovenOwnerTimeoutMs`") && lock_notes.contains("does nothing"), "{lock_notes}");
    let service_wait_seconds: u64 = crate::wait_ceilings::LANDING_LOCK_CEILING.as_secs();
    assert!(lock_notes.contains(format!("for at most {service_wait_seconds} seconds").as_str()) && lock_notes.contains("Remove the key"), "{lock_notes}");
    assert!(!lock_notes.contains("for as long as the lock stays there"), "the service's wait has a ceiling: {lock_notes}");
    assert!(!lock_notes.contains("CLI_GIT_LOCK_WAIT_SECONDS"), "the variable no longer exists: {lock_notes}");
    assert!(!lock_notes.contains("landing.reserveAfterLostRaces"), "{lock_notes}");
    std::fs::write(&config, "{ \"landing\": { \"reserveAfterLostRaces\": 3 } }\n").expect("config");
    let landing_only: ConfigFindings = findings_of(&git_context(), repo.as_path());
    assert!(landing_only.retired_landing_key && !landing_only.retired_index_lock_key);
    let landing_notes: String = render_notes(&landing_only);
    assert!(landing_notes.contains("`landing.reserveAfterLostRaces`") && landing_notes.contains("does nothing"), "{landing_notes}");
    assert!(landing_notes.contains("Remove the key") && !landing_notes.contains("indexLock"), "{landing_notes}");
    std::fs::write(
        &config,
        "{ \"hooks\": { \"concurrentCommits\": false }, \"indexLock\": { \"unprovenOwnerTimeoutMs\": 0 }, \"landing\": { \"reserveAfterLostRaces\": 1 } }\n",
    )
    .expect("config");
    let all: ConfigFindings = findings_of(&git_context(), repo.as_path());
    assert!(all.retired_hooks_key && all.retired_index_lock_key && all.retired_landing_key);
    let all_notes: String = render_notes(&all);
    assert_eq!(all_notes.lines().count(), 3, "one note per key: {all_notes}");
    remove(root.as_path());
}

/// The retired `hooks` key is accepted and reported as ignored,
/// and an unreadable file is named.
#[test]
fn the_retired_key_and_a_broken_file_are_reported() {
    let root: PathBuf = fixture("doctor-config-retired");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("monochromatic-git-policy-cli.config.jsonc"), "{ \"hooks\": { \"concurrentCommits\": true } }\n").expect("config");
    let findings: ConfigFindings = findings_of(&git_context(), repo.as_path());
    assert!(findings.retired_hooks_key);
    assert!(render_notes(&findings).contains("accepted so older files still load, and ignored"));
    std::fs::write(repo.join("monochromatic-git-policy-cli.config.jsonc"), "{ not json").expect("config");
    let broken: ConfigFindings = findings_of(&git_context(), repo.as_path());
    assert!(!broken.retired_hooks_key);
    assert_eq!(render_notes(&broken), "");
    remove(root.as_path());
}

/// What:
///  A hook the person configured is still listed when the wrapper's own switch for its event is
///  in the command scope of the read.
/// Why:
///   Every Git the wrapper starts carries `-c hook.<event>.enabled=false`,
/// and a child of one passes it on through
///  `GIT_CONFIG_PARAMETERS`,
/// where it is read as command-scope configuration;
/// a doctor run inside such a child found every event
///  off and listed nothing.
#[test]
fn a_switch_the_wrapper_supplied_does_not_hide_a_configured_hook_from_doctor() {
    let root: PathBuf = fixture("doctor-config-hook-switch");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["config", "hook.control.event", "pre-commit"]);
    git(repo.as_path(), &["config", "hook.control.command", "run the linter"]);
    // The switches a wrapped commit supplies to every Git it starts, inherited here as a child inherits them.
    let mut context: GitContext = git_context();
    context.overlay.push((
        std::ffi::OsString::from("GIT_CONFIG_PARAMETERS"),
        std::ffi::OsString::from("'hook.pre-commit.enabled'='false'"),
    ));
    let findings: ConfigFindings = findings_of(&context, repo.as_path());
    assert_eq!(findings.hooks.len(), 1, "the person's hook is listed");
    assert_eq!(render_hook_lines(&findings), "hook.control (events: pre-commit)\n");
    remove(root.as_path());
}

/// What:
///  Doctor lists a hook the caller enabled with `-c`,
/// even though the repository's own file switched it off.
/// Why:
///   Command-scope values are Git's last word,
/// so a `-c hook.<name>.enabled=true` is the caller's own choice and not the wrapper's:
///  reading only the files dropped the hook from doctor and from the commit warning alike.
#[test]
fn doctor_lists_a_hook_the_caller_enabled_with_a_command_switch() {
    let root: PathBuf = fixture("doctor-config-caller-enabled");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["config", "hook.control.event", "pre-commit"]);
    git(repo.as_path(), &["config", "hook.control.command", "run the linter"]);
    git(repo.as_path(), &["config", "hook.control.enabled", "false"]);
    let mut context: GitContext = git_context();
    context.overlay.push((
        std::ffi::OsString::from("GIT_CONFIG_PARAMETERS"),
        std::ffi::OsString::from("'hook.control.enabled'='true'"),
    ));
    let findings: ConfigFindings = findings_of(&context, repo.as_path());
    assert_eq!(render_hook_lines(&findings), "hook.control (events: pre-commit)\n", "the caller's own switch enabled it");
    remove(root.as_path());
}
