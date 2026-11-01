//! Controls for installing, removing and checking the commit service's activation, against a stand-in service manager
//!  control program that records what it is asked.

use super::*;
use crate::service_messages::DENYLIST;
use crate::test_support::{executable, fixture, remove};

/// A stand-in `systemctl` that logs its arguments and answers as the flags say.
fn fake_manager(root: &Path, manager: bool, enable: bool, active: bool) -> PathBuf {
    let path: PathBuf = root.join("systemctl");
    let exit = |ok: bool| { return if ok { 0 } else { 1 }; };
    let script: String = format!(
        "#!/bin/sh\necho \"$@\" >> '{}'\ncase \"$*\" in\n*show-environment*) exit {};;\n*enable*) echo 'unit file is read-only' >&2; exit {};;\n*is-active*) exit {};;\nesac\nexit 0\n",
        root.join("calls.log").display(),
        exit(manager),
        exit(enable),
        exit(active)
    );
    executable(path.as_path(), script.as_bytes());
    return path;
}

fn host(root: &Path, manager: PathBuf) -> ServiceHost {
    let binary: PathBuf = root.join("cli-git-service");
    std::fs::write(&binary, b"binary").expect("service program");
    return ServiceHost { unit_directory: root.join("units"), systemctl: manager, service_binary: binary, git: PathBuf::from("/usr/bin/git") };
}

fn calls(root: &Path) -> Vec<String> {
    return std::fs::read_to_string(root.join("calls.log")).unwrap_or_default().lines().map(str::to_string).collect();
}

/// Text without the parts between backticks, which quote file names and commands.
fn plain(text: &str) -> String {
    return text.split('`').step_by(2).collect::<Vec<&str>>().join(" ").to_lowercase();
}

/// An install writes both units with the program and Git filled in, enables the socket, and says what it did.
#[test]
fn an_install_writes_the_units_and_enables_the_socket() {
    let root: PathBuf = fixture("service-install");
    let host: ServiceHost = host(root.as_path(), fake_manager(root.as_path(), true, true, true));
    let report: Report = install(&host);
    assert_eq!(report.code, 0, "{report:?}");
    for (name, text) in unit_texts(&host) {
        assert_eq!(std::fs::read_to_string(host.unit_directory.join(name)).expect("unit written"), text);
        assert!(!text.contains('@'), "no placeholder is left in {name}");
    }
    let service: String = std::fs::read_to_string(host.unit_directory.join(SERVICE_UNIT_NAME)).expect("service unit");
    assert!(service.contains(host.service_binary.to_string_lossy().as_ref()) && service.contains("/usr/bin/git"));
    let log: Vec<String> = calls(root.as_path());
    assert!(log.iter().any(|call| { return call == &format!("--user enable --now {SOCKET_UNIT_NAME}"); }), "{log:?}");
    assert!(!log.iter().any(|call| { return call.contains("start") && !call.contains("--now"); }), "the service is never started directly: {log:?}");
    assert!(report.stdout.contains("Installed") && report.stdout.contains(SOCKET_UNIT_NAME), "{}", report.stdout);
    remove(root.as_path());
}

/// A second install changes no file and says so.
#[test]
fn a_second_install_changes_nothing_and_says_so() {
    let root: PathBuf = fixture("service-install-twice");
    let host: ServiceHost = host(root.as_path(), fake_manager(root.as_path(), true, true, true));
    assert_eq!(install(&host).code, 0);
    let before: Vec<(&str, std::time::SystemTime)> = [SOCKET_UNIT_NAME, SERVICE_UNIT_NAME]
        .map(|name| { return (name, std::fs::metadata(host.unit_directory.join(name)).expect("unit").modified().expect("time")); })
        .to_vec();
    let again: Report = install(&host);
    assert_eq!(again.code, 0);
    assert!(again.stdout.contains("already installed"), "{}", again.stdout);
    for (name, time) in before {
        assert_eq!(std::fs::metadata(host.unit_directory.join(name)).expect("unit").modified().expect("time"), time, "{name}");
    }
    remove(root.as_path());
}

/// A changed program path rewrites only the service unit.
#[test]
fn a_moved_program_rewrites_the_service_unit() {
    let root: PathBuf = fixture("service-install-moved");
    let mut host: ServiceHost = host(root.as_path(), fake_manager(root.as_path(), true, true, true));
    assert_eq!(install(&host).code, 0);
    let moved: PathBuf = root.join("elsewhere-service");
    std::fs::write(&moved, b"binary").expect("moved");
    host.service_binary = moved.clone();
    let report: Report = install(&host);
    assert!(report.stdout.contains(SERVICE_UNIT_NAME) && !report.stdout.contains(&format!("`{}`", host.unit_directory.join(SOCKET_UNIT_NAME).display())), "{}", report.stdout);
    assert!(std::fs::read_to_string(host.unit_directory.join(SERVICE_UNIT_NAME)).expect("unit").contains(moved.to_string_lossy().as_ref()));
    remove(root.as_path());
}

/// Each failure the person can act on is the plain text of its message, and changes no file when nothing was written yet.
#[test]
fn install_failures_are_the_plain_messages() {
    let root: PathBuf = fixture("service-install-failures");
    let no_program: ServiceHost = ServiceHost { service_binary: root.join("absent"), ..host(root.as_path(), fake_manager(root.as_path(), true, true, true)) };
    let missing: Report = install(&no_program);
    assert_eq!(missing.code, 1);
    assert_eq!(missing.stderr, format!("{}\n", render(&ClientFailure::InstallProgramMissing(root.join("absent").display().to_string()))));
    assert!(!no_program.unit_directory.exists(), "nothing was written");
    let no_manager: ServiceHost = host(root.as_path(), fake_manager(root.as_path(), false, true, true));
    let unavailable: Report = install(&no_manager);
    assert_eq!(unavailable.stderr, format!("{}\n", render(&ClientFailure::NoActivation(platform_without_manager()))));
    let refusing: ServiceHost = host(root.as_path(), fake_manager(root.as_path(), true, false, true));
    let refused: Report = install(&refusing);
    assert_eq!(refused.code, 1);
    assert!(refused.stderr.contains("unit file is read-only") && refused.stderr.contains(INSTALL_COMMAND_FOR_TEST), "{}", refused.stderr);
    let absent_control: ServiceHost = ServiceHost { systemctl: root.join("no-such-program"), ..no_manager };
    assert_eq!(install(&absent_control).stderr, unavailable.stderr, "no control program is the same as no manager");
    remove(root.as_path());
}

const INSTALL_COMMAND_FOR_TEST: &str = crate::service_messages::INSTALL_COMMAND;

/// An uninstall removes both units, and one where nothing was installed says so.
#[test]
fn an_uninstall_removes_the_units_and_a_second_one_says_nothing_was_there() {
    let root: PathBuf = fixture("service-uninstall");
    let host: ServiceHost = host(root.as_path(), fake_manager(root.as_path(), true, true, true));
    assert_eq!(install(&host).code, 0);
    let removed: Report = uninstall(&host);
    assert_eq!(removed.code, 0);
    assert!(removed.stdout.contains("Removed"), "{}", removed.stdout);
    for name in [SOCKET_UNIT_NAME, SERVICE_UNIT_NAME] {
        assert!(!host.unit_directory.join(name).exists(), "{name}");
    }
    assert!(calls(root.as_path()).iter().any(|call| { return call.contains("disable --now"); }));
    let nothing: Report = uninstall(&host);
    assert_eq!(nothing.code, 0);
    assert!(nothing.stdout.contains("was not installed"), "{}", nothing.stdout);
    remove(root.as_path());
}

/// The status is zero when the socket listens, and the not-installed text otherwise.
#[test]
fn status_follows_the_socket() {
    let root: PathBuf = fixture("service-status");
    let listening: Report = status(&host(root.as_path(), fake_manager(root.as_path(), true, true, true)));
    assert_eq!(listening.code, 0);
    let silent: Report = status(&host(root.as_path(), fake_manager(root.as_path(), true, true, false)));
    assert_eq!(silent.code, 1);
    assert_eq!(silent.stderr, format!("{}\n", render(&ClientFailure::NotInstalled)));
    remove(root.as_path());
}

/// No report names an implementation term outside quoted file names and commands.
#[test]
fn reports_leak_no_implementation_term() {
    let root: PathBuf = fixture("service-install-words");
    let host: ServiceHost = host(root.as_path(), fake_manager(root.as_path(), true, true, true));
    let texts: Vec<String> = vec![install(&host).stdout, install(&host).stdout, uninstall(&host).stdout, uninstall(&host).stdout];
    for text in texts {
        let words: String = plain(text.as_str());
        for term in DENYLIST {
            let at: Option<usize> = words.find(term);
            let whole: bool = at.is_some_and(|start| {
                let before: bool = words[..start].chars().next_back().is_none_or(|c| { return !c.is_alphanumeric(); });
                let after: bool = words[start + term.len()..].chars().next().is_none_or(|c| { return !c.is_alphanumeric(); });
                return before && after;
            });
            assert!(!whole, "{term:?} in {words:?}");
        }
    }
    remove(root.as_path());
}

/// The unit directory is the configuration home's, else the home's `.config`.
#[test]
fn the_unit_directory_follows_the_configuration_home() {
    assert_eq!(unit_directory(Some(Path::new("/cfg")), Some(Path::new("/home/a"))), Some(PathBuf::from("/cfg/systemd/user")));
    assert_eq!(unit_directory(None, Some(Path::new("/home/a"))), Some(PathBuf::from("/home/a/.config/systemd/user")));
    assert_eq!(unit_directory(None, None), None);
}
