//! Unit tests of the repository link cause:
//! nothing is asked without an identity file or for a path elsewhere,
//! the check's message comes through,
//! and a check that hangs is killed at the deadline.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::{CHECK_RELATIVE_PATH, IDENTITY_RELATIVE_PATH, LinkVerdict, repository_link_verdict, under_repository_link};

/// A disposable home with the repository's link path inside it.
struct Home {
    directory: tempfile::TempDir,
}

impl Home {
    fn new() -> Home {
        return Home { directory: crate::test_scratch::scratch() };
    }

    fn path(&self) -> &Path {
        return self.directory.path();
    }

    /// The repository's link path in this home.
    fn link(&self) -> PathBuf {
        return self.path().join("Monochromatic");
    }

    /// Records an identity naming this home's link path and a canonical root on a drive that is not there.
    fn record_identity(&self) {
        let file: PathBuf = self.path().join(IDENTITY_RELATIVE_PATH);
        fs::create_dir_all(file.parent().expect("parent")).expect("config folder");
        fs::write(
            file,
            format!(
                "{{\"linkPath\":\"{}\",\"canonicalRoot\":\"{}\",\"phase\":\"after\"}}",
                self.link().display(),
                self.path().join("drive/repository/Monochromatic").display()
            ),
        )
        .expect("identity file");
    }

    /// Installs a check program that runs `body` with a shell.
    fn install_check(&self, body: &str) -> PathBuf {
        let program: PathBuf = self.path().join(CHECK_RELATIVE_PATH);
        fs::create_dir_all(program.parent().expect("parent")).expect("install folder");
        fs::write(program.as_path(), format!("#!/bin/sh\n{body}\n")).expect("check program");
        fs::set_permissions(program.as_path(), fs::Permissions::from_mode(0o755)).expect("chmod");
        return program;
    }

    /// The file a check program writes when it starts.
    fn started_marker(&self) -> PathBuf {
        return self.path().join("check-started");
    }
}

const DEADLINE: Duration = Duration::from_secs(5);

/// Without an identity file nothing is asked,
/// and the check program is not started.
#[test]
fn without_an_identity_file_nothing_is_asked() {
    let home = Home::new();
    home.install_check(&format!("touch {}", home.started_marker().display()));
    assert_eq!(repository_link_verdict(home.path(), home.link().join("toolchain.lock.json").as_path(), DEADLINE), LinkVerdict::NotConfigured);
    assert!(!home.started_marker().exists(), "the check was started");
}

/// A path elsewhere is not asked about either.
#[test]
fn a_path_elsewhere_is_not_asked_about() {
    let home = Home::new();
    home.record_identity();
    home.install_check(&format!("touch {}", home.started_marker().display()));
    assert_eq!(repository_link_verdict(home.path(), home.path().join("elsewhere/toolchain.lock.json").as_path(), DEADLINE), LinkVerdict::NotConfigured);
    assert!(!home.started_marker().exists(), "the check was started");
}

/// A ready repository is ready.
#[test]
fn a_check_that_exits_zero_is_ready() {
    let home = Home::new();
    home.record_identity();
    home.install_check("echo the repository is ready");
    assert_eq!(repository_link_verdict(home.path(), home.link().join("x").as_path(), DEADLINE), LinkVerdict::Ready);
}

/// The check's own message is the verdict when it says no,
/// and it is given the home it should read.
#[test]
fn a_check_that_fails_gives_its_message() {
    let home = Home::new();
    home.record_identity();
    home.install_check("echo \"the 860 drive is not mounted yet (home $HOME)\" >&2\nexit 75");
    let verdict: LinkVerdict = repository_link_verdict(home.path(), home.link().join("x").as_path(), DEADLINE);
    let LinkVerdict::NotReady { message } = verdict else { panic!("not ready expected, got {verdict:?}") };
    assert_eq!(message, format!("the 860 drive is not mounted yet (home {})", home.path().display()));
}

/// A check program that is not installed is told apart from a check that says no.
#[test]
fn a_missing_check_program_is_unavailable() {
    let home = Home::new();
    home.record_identity();
    let verdict: LinkVerdict = repository_link_verdict(home.path(), home.link().join("x").as_path(), DEADLINE);
    assert!(matches!(verdict, LinkVerdict::CheckUnavailable { .. }), "{verdict:?}");
}

/// A check that never answers is killed at the deadline and the verdict says so.
#[test]
fn a_check_that_hangs_is_given_up_on_at_the_deadline() {
    let home = Home::new();
    home.record_identity();
    home.install_check("exec sleep 30");
    let started = Instant::now();
    let verdict: LinkVerdict = repository_link_verdict(home.path(), home.link().join("x").as_path(), Duration::from_millis(300));
    let LinkVerdict::NotReady { message } = verdict else { panic!("not ready expected, got {verdict:?}") };
    assert!(message.contains("did not answer"), "{message}");
    assert!(started.elapsed() < Duration::from_secs(5), "the wait took {:?}", started.elapsed());
}

/// Every spelling of the repository is under it,
/// and a neighbour is not.
#[test]
fn every_spelling_of_the_repository_is_under_it_and_no_neighbour_is() {
    let home = Home::new();
    home.record_identity();
    let alias: PathBuf = home.path().join("alias");
    std::os::unix::fs::symlink(home.path(), alias.as_path()).expect("alias of the home");
    for under in [
        home.link(),
        home.link().join("toolchain.lock.json"),
        home.path().join("drive/repository/Monochromatic/.cache/launchers"),
        alias.join("Monochromatic/.cache"),
    ] {
        assert!(under_repository_link(home.path(), under.as_path()), "{} is under the repository", under.display());
    }
    for outside in [home.path().join("Monochromatic2"), home.path().join("elsewhere"), PathBuf::from("/usr/bin")] {
        assert!(!under_repository_link(home.path(), outside.as_path()), "{} is not under it", outside.display());
    }
}

/// A repository the check said is usable is asked about once per process,
/// so an install that makes many launchers starts the check once.
#[test]
fn a_ready_answer_is_not_asked_for_again() {
    let home = Home::new();
    home.record_identity();
    home.install_check(&format!("echo started >> {}", home.started_marker().display()));
    for _ in 0..3 {
        assert_eq!(repository_link_verdict(home.path(), home.link().join("x").as_path(), DEADLINE), LinkVerdict::Ready);
    }
    let starts: String = fs::read_to_string(home.started_marker()).expect("the marker");
    assert_eq!(starts.lines().count(), 1, "{starts}");
}
