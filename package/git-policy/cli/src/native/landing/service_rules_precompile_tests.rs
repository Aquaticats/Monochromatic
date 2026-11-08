//! Controls for the service's warm of the compiled forbidden-strings rules:
//! it publishes what the client would look for,
//! on the
//!  first landing of a repository and when a landing changes a rules path,
//! remembers the repository for the next start,
//! and
//!  does nothing for a landing that changes no rules path.
#![cfg(unix)]

use super::*;
use crate::scanner_test_support::{needle, run_isolated};
use std::time::Instant;

/// How long a control waits for the compile of a background warm:
/// the longest compile measured on 2026-11-07 (about 20 s on the
///  first commit of a fresh cache under load) with a margin.
const WARM_PATIENCE: Duration = Duration::from_secs(120);

/// How long a control watches for a warm that must not happen.
const QUIET_FOR: Duration = Duration::from_secs(3);

/// The rules file the default configuration names.
const RULES_FILE: &str = "forbidden-strings.local.txt";

/// The cache directory of the isolated control.
fn cache_directory() -> PathBuf {
    return PathBuf::from(std::env::var_os("FORBIDDEN_STRINGS_CACHE_DIR").expect("isolated cache directory"));
}

/// Every compiled artifact in the cache.
fn artifacts() -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![cache_directory()];
    while let Some(next) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(next) else { continue };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.file_name().is_some_and(is_artifact_name) {
                found.push(path);
            }
        }
    }
    return found;
}

/// Whether a file name is the compiled artifact's.
fn is_artifact_name(name: &std::ffi::OsStr) -> bool {
    return name == "rules.bin";
}

/// Waits until an artifact exists,
/// at most `WARM_PATIENCE`.
fn wait_for_artifact() -> bool {
    let began: Instant = Instant::now();
    while began.elapsed() < WARM_PATIENCE {
        if !artifacts().is_empty() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    return false;
}

/// A repository directory holding the planted rules file.
fn repository_in(directory: &Path) -> PathBuf {
    let repository: PathBuf = directory.join("repo");
    std::fs::create_dir_all(&repository).expect("repository directory");
    std::fs::write(repository.join(RULES_FILE), format!("{}\n", needle())).expect("rules file");
    return repository;
}

/// Body:
/// the precompile publishes the compiled rules.
fn precompile_publishes_body(directory: &Path) {
    let repository: PathBuf = repository_in(directory);
    assert!(artifacts().is_empty());
    let elapsed: Result<Duration, String> = precompile_rules(&repository);
    assert!(elapsed.is_ok(), "{elapsed:?}");
    assert_eq!(artifacts().len(), 1, "the compiled rules are in the cache");
    // A second precompile finds them and publishes nothing more.
    assert!(precompile_rules(&repository).is_ok());
    assert_eq!(artifacts().len(), 1);
}

/// The warm publishes what the client would look for.
#[test]
fn the_precompile_publishes_the_compiled_rules() {
    run_isolated("landing::service_rules_precompile::tests::the_precompile_publishes_the_compiled_rules", "rules-precompile", precompile_publishes_body);
}

/// Body:
/// the first landing of a repository warms and remembers it;
/// one that changes no rules path leaves the cache alone;
/// one that
///  changes a rules path warms again.
fn landing_triggers_body(directory: &Path) {
    let repository: PathBuf = repository_in(directory);
    let journal: PathBuf = directory.join("journal");
    std::fs::create_dir_all(&journal).expect("journal");
    after_landing(&journal, &repository, &[String::from("src/a.rs")]);
    assert!(wait_for_artifact(), "the first landing of a repository warms its rules");
    let remembered: String = std::fs::read_to_string(journal.join(REMEMBERED_FILE)).expect("the repository is remembered");
    assert_eq!(remembered.trim(), repository.to_string_lossy());
    // Another landing that changes no path of the rules warms nothing, even with the cache gone.
    std::fs::remove_dir_all(cache_directory()).expect("remove the cache");
    after_landing(&journal, &repository, &[String::from("src/b.rs")]);
    std::thread::sleep(QUIET_FOR);
    assert!(artifacts().is_empty(), "a landing that changed no rules path warmed the cache");
    // A landing that changes the rules file warms again.
    after_landing(&journal, &repository, &[String::from(RULES_FILE)]);
    assert!(wait_for_artifact(), "a landing that changed the rules file warms them");
}

/// The triggers of a landing.
#[test]
fn a_landing_warms_the_first_time_and_when_the_rules_change() {
    run_isolated("landing::service_rules_precompile::tests::a_landing_warms_the_first_time_and_when_the_rules_change", "rules-landing", landing_triggers_body);
}

/// Body:
/// a restarted service warms the repository it remembers.
fn restart_warms_body(directory: &Path) {
    let repository: PathBuf = repository_in(directory);
    let journal: PathBuf = directory.join("journal");
    std::fs::create_dir_all(&journal).expect("journal");
    std::fs::write(journal.join(REMEMBERED_FILE), format!("{}\n", repository.display())).expect("the remembered repository");
    assert!(artifacts().is_empty());
    warm_remembered(&journal);
    assert!(wait_for_artifact(), "a restarted service warms the repository it remembers");
}

/// The restart.
#[test]
fn a_restarted_service_warms_the_repository_it_remembers() {
    run_isolated("landing::service_rules_precompile::tests::a_restarted_service_warms_the_repository_it_remembers", "rules-restart", restart_warms_body);
}

/// A warm is skipped when a limit on the address space leaves less room than a compile needs,
/// and runs when no limit is set,
/// when
///  the room is exactly what it needs,
/// or when there is more.
#[test]
fn a_warm_is_skipped_only_when_the_address_space_left_is_too_small() {
    assert!(warm_fits(None), "no limit leaves the warm to run");
    assert!(warm_fits(Some(WARM_ADDRESS_SPACE)));
    assert!(warm_fits(Some(WARM_ADDRESS_SPACE + 1)));
    assert!(!warm_fits(Some(WARM_ADDRESS_SPACE - 1)));
    assert!(!warm_fits(Some(0)));
}
