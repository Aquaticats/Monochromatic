//! The unit tests of `launcher_chain.rs`:
//! a chain that ends at a program is fine,
//! every other chain is refused.

use std::fs;
use std::path::{Path, PathBuf};

use super::{MAX_LAUNCHER_CHAIN, chain_problem};
use crate::test_scratch::scratch;
use crate::toolchain::launcher::{LaunchRecord, record_path, render_record};
use crate::toolchain::scope::ScopeKind;

/// A file standing for a launcher or a program,
/// with a record beside it when `real` is given.
fn make(path: &Path, real: Option<&Path>) {
    fs::create_dir_all(path.parent().expect("a parent folder")).expect("make the folder");
    fs::write(path, "x").expect("write the file");
    if let Some(target) = real {
        let record = LaunchRecord { tool_id: "t".to_string(), lock_file: PathBuf::from("/dev/null"), real: target.to_path_buf(), scope: ScopeKind::Repo };
        fs::write(record_path(path), render_record(&record)).expect("write the record");
    }
}

#[test]
fn a_launcher_that_names_a_program_has_no_problem() {
    let folder = scratch();
    let (own, program) = (folder.path().join("own"), folder.path().join("program"));
    make(&program, None);
    make(&own, Some(&program));

    assert_eq!(chain_problem(&own, &program), None);
}

#[test]
fn a_chain_of_launchers_that_ends_at_a_program_has_no_problem() {
    let folder = scratch();
    let (own, middle, program) = (folder.path().join("own"), folder.path().join("middle"), folder.path().join("program"));
    make(&program, None);
    make(&middle, Some(&program));
    make(&own, Some(&middle));

    assert_eq!(chain_problem(&own, &middle), None);
}

#[test]
fn a_record_that_names_its_own_launcher_leads_back_to_itself() {
    let folder = scratch();
    let own: PathBuf = folder.path().join("own");
    make(&own, Some(&own));

    let problem: String = chain_problem(&own, &own).expect("a launcher that starts itself is a problem");

    assert!(problem.contains("leads back to itself"), "{problem}");
}

#[test]
fn two_launchers_that_name_each_other_lead_back_to_the_first() {
    let folder = scratch();
    let (own, other) = (folder.path().join("own"), folder.path().join("other"));
    make(&own, Some(&other));
    make(&other, Some(&own));

    let problem: String = chain_problem(&own, &other).expect("launchers that start each other are a problem");

    assert!(problem.contains("leads back to itself"), "{problem}");
}

#[test]
fn launchers_that_circle_without_the_running_one_are_a_circle() {
    let folder = scratch();
    let (own, first, second) = (folder.path().join("own"), folder.path().join("first"), folder.path().join("second"));
    make(&own, Some(&first));
    make(&first, Some(&second));
    make(&second, Some(&first));

    let problem: String = chain_problem(&own, &first).expect("a circle of launchers is a problem");

    assert!(problem.contains("lead in a circle"), "{problem}");
}

#[test]
fn a_chain_longer_than_the_bound_is_refused_and_one_at_the_bound_is_not() {
    let folder = scratch();
    let program: PathBuf = folder.path().join("program");
    make(&program, None);
    let mut next: PathBuf = program.clone();
    let mut launchers: Vec<PathBuf> = Vec::new();
    for index in 0..=MAX_LAUNCHER_CHAIN + 1 {
        let launcher: PathBuf = folder.path().join(format!("launcher-{index}"));
        make(&launcher, Some(&next));
        launchers.push(launcher.clone());
        next = launcher;
    }
    let own: PathBuf = folder.path().join("own");
    make(&own, Some(&next));

    // The chain from `launchers[MAX_LAUNCHER_CHAIN - 1]` holds MAX_LAUNCHER_CHAIN launchers before the program: at the bound.
    assert_eq!(chain_problem(&own, &launchers[MAX_LAUNCHER_CHAIN - 1]), None);
    let problem: String = chain_problem(&own, &next).expect("a chain past the bound is a problem");
    assert!(problem.contains("without reaching a program"), "{problem}");
}

#[test]
fn a_missing_program_or_an_unreadable_record_ends_the_walk_with_no_problem() {
    let folder = scratch();
    let own: PathBuf = folder.path().join("own");
    make(&own, None);
    let broken: PathBuf = folder.path().join("broken");
    make(&broken, None);
    fs::write(record_path(&broken), "{not json").expect("write the record");

    assert_eq!(chain_problem(&own, &folder.path().join("missing")), None);
    assert_eq!(chain_problem(&own, &broken), None);
}
