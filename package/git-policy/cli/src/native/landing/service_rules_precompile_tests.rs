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

/// How long a control waits for the service's own warm job to end:
/// a backstop that only a warm which never ends reaches,
/// never a slow host.
const WARM_PATIENCE: Duration = Duration::from_secs(120);

/// How often a control asks whether the service's warm job has ended.
const LOOK_EVERY: Duration = Duration::from_millis(50);

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

/// What:
///  The answer the service's warm jobs have given so far:
/// how many have ended,
/// and how many of those warmed the cache.
/// Why:
///   The warm runs on the blocking pool,
/// so no order of the control's own steps says whether it ran;
/// the job itself says so,
/// and a control reads that instead of sleeping and hoping.
#[derive(Clone, Copy)]
struct Warms {
    /// How many warm jobs had ended when this was read.
    ended: usize,
    /// How many of them warmed the cache.
    made: usize,
}

/// What the service's warm jobs have answered so far.
fn warms() -> Warms {
    return Warms { ended: WARM_JOBS_ENDED.load(Ordering::SeqCst), made: WARMS_MADE.load(Ordering::SeqCst) };
}

impl Warms {
    /// Waits until `count` more warm jobs have ended than at this reading,
    /// and gives the answer then;
    /// nothing when they did not end within `WARM_PATIENCE`,
    /// which only a warm that never ends reaches.
    ///  The claim a warm runs under is part of the answer:
    /// a job records its end before the runtime task that owns the claim releases it,
    /// and a landing that arrives in that window is skipped as one more warm is still running,
    /// so a control that read the news alone could make a landing nothing would judge.
    fn after(&self, count: usize) -> Option<Warms> {
        let began: Instant = Instant::now();
        loop {
            let ended: usize = WARM_JOBS_ENDED.load(Ordering::SeqCst);
            if ended >= self.ended.saturating_add(count) && !RUNNING.load(Ordering::SeqCst) {
                return Some(warms());
            }
            if began.elapsed() >= WARM_PATIENCE {
                return None;
            }
            std::thread::sleep(LOOK_EVERY);
        }
    }
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
/// Why this shape:
///  each verdict is the warm job's own answer,
/// not how long the control waited;
/// and `WARM_PATIENCE` is the backstop for a warm that never ends.
fn landing_triggers_body(directory: &Path) {
    let repository: PathBuf = repository_in(directory);
    let journal: PathBuf = directory.join("journal");
    std::fs::create_dir_all(&journal).expect("journal");
    let first: Warms = warms();
    after_landing(&journal, &repository, &[String::from("src/a.rs")]);
    let warmed: Warms = first.after(1).expect("the first landing of a repository asks the service for a warm");
    assert!(warmed.made > first.made, "the first landing of a repository warms its rules");
    assert_eq!(artifacts().len(), 1, "the compiled rules are in the cache");
    let remembered: String = std::fs::read_to_string(journal.join(REMEMBERED_FILE)).expect("the repository is remembered");
    assert_eq!(remembered.trim(), repository.to_string_lossy());
    // Another landing that changes no path of the rules warms nothing, even with the cache gone.
    let quiet: Warms = warms();
    std::fs::remove_dir_all(cache_directory()).expect("remove the cache");
    after_landing(&journal, &repository, &[String::from("src/b.rs")]);
    let answered: Warms = quiet.after(1).expect("the landing asks the service about the rules");
    assert_eq!(answered.made, quiet.made, "a landing that changed no rules path warmed the cache");
    assert!(artifacts().is_empty(), "a landing that changed no rules path warmed the cache");
    // A landing that changes the rules file warms again.
    let changed: Warms = warms();
    after_landing(&journal, &repository, &[String::from(RULES_FILE)]);
    let warmed_again: Warms = changed.after(1).expect("the landing asks the service about the rules");
    assert!(warmed_again.made > changed.made, "a landing that changed the rules file warms them");
    assert_eq!(artifacts().len(), 1, "the compiled rules are in the cache again");
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
    let before: Warms = warms();
    warm_remembered(&journal);
    let answered: Warms = before.after(1).expect("a restarted service asks for the warm it remembers");
    assert!(answered.made > before.made, "a restarted service warms the repository it remembers");
    assert_eq!(artifacts().len(), 1, "the compiled rules are in the cache");
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
