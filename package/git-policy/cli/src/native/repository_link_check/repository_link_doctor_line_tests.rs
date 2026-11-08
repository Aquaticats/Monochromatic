//! Controls for the doctor's line about the repository link:
//! none without an identity file,
//! the check's own message with one.

use super::*;
use crate::repository_link_fixture_probe::{FixtureProbe, World};
use crate::repository_link_identity::IDENTITY_RELATIVE_PATH;
use crate::repository_link_probe::EntryKind;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

const LINK: &str = "/home/u/Monochromatic";

/// Records an identity in phase `after` below a home folder.
fn record(home: &Path) {
    let path: PathBuf = home.join(IDENTITY_RELATIVE_PATH);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("config folder");
    std::fs::write(
        path,
        format!(
            "{{\"linkPath\":\"{LINK}\",\"linkText\":\"/mnt/860/r/M\",\"canonicalRoot\":\"/mnt/860/r/M\",\"mountPoint\":\"/mnt/860\",\
             \"driveUuid\":\"D\",\"nvmeUuid\":\"N\",\"rootCommit\":\"abc\",\"phase\":\"after\"}}"
        ),
    )
    .expect("identity file");
}

/// No home and no identity file give no line at all,
/// and nothing is asked.
#[test]
fn without_an_identity_file_there_is_no_line() {
    let home: PathBuf = fixture("doctor-line-absent");
    let probe: Arc<FixtureProbe> = Arc::new(FixtureProbe::over(World::default()));
    assert_eq!(doctor_link_line_with_probe(probe.clone(), None), "");
    assert_eq!(doctor_link_line_with_probe(probe.clone(), Some(home.as_path())), "");
    assert!(probe.questions().is_empty());
    remove(home.as_path());
}

/// With an identity file the line is the check's message,
/// one line,
/// whatever the answer.
#[test]
fn with_an_identity_file_the_line_is_the_checks_message() {
    let home: PathBuf = fixture("doctor-line-present");
    record(home.as_path());
    let mut replaced: World = World::default();
    replaced.entries.insert(String::from(LINK), EntryKind::Directory);
    let line: String = doctor_link_line_with_probe(Arc::new(FixtureProbe::over(replaced)), Some(home.as_path()));
    assert_eq!(line, format!("repository link: {LINK} is a real folder, not the link to the 860; nothing was written\n"));
    let gone: String = doctor_link_line_with_probe(Arc::new(FixtureProbe::over(World::default())), Some(home.as_path()));
    assert!(gone.starts_with("repository link: ") && gone.ends_with('\n') && gone.matches('\n').count() == 1, "{gone}");
    remove(home.as_path());
}

/// An identity file that cannot be read is said on the line too,
/// so the missing protection is visible.
#[test]
fn an_unreadable_identity_file_is_said() {
    let home: PathBuf = fixture("doctor-line-unreadable");
    let path: PathBuf = home.join(IDENTITY_RELATIVE_PATH);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("config folder");
    std::fs::write(path, b"not json").expect("file");
    let line: String = doctor_link_line_with_probe(Arc::new(FixtureProbe::over(World::default())), Some(home.as_path()));
    assert!(line.starts_with("repository link: the identity file cannot be used"), "{line}");
    remove(home.as_path());
}
