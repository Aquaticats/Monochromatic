//! What:
//!  Guards for the owner's rule that nothing of the wrapper starts at system startup or at login (owner,
//! 2026-11-07:
//! "Were you
//!  implying our wrapper or something in it auto starts itself on system startup or user login?
//! It should never do that!
//! Follow
//!  Bazel."
//! Then:
//! "Systemd units are fine."):
//! the unit texts carry no install line,
//! provisioning links nothing and writes only in the
//!  runtime unit directory (it may remove the earlier releases' files in the user's own unit directory),
//! and the manager is asked
//!  only to reload,
//! reset,
//! start,
//! stop and report a state.
//! Why:
//!  The service is started by the first command that needs it,
//! through the socket the manager listens on for a runtime unit,
//! and
//!  exits after three idle hours,
//! like Bazel's server.
//! A unit that `systemctl --user enable` could link into a `*.wants` folder,
//! a
//!  file in the user's own unit directory,
//! or a call that enables or links a unit would start it at login;
//! each is guarded
//!  here,
//! beside the scan of the source (`source_guard/no_start_at_login_tests.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(linesThatStartAtLogin(unitText)).toEqual([]); expect(created(afterProvisioning)).toAllBeBelow(runtimeUnitDir);
//! ```

use super::tests::{Machine, Recorder, add_socket, earlier_service_files, machine};
use super::*;
use crate::no_start_at_login_tests::lines_that_start_at_login;
use crate::test_support::remove;
use std::collections::BTreeMap;

/// The units the wrapper writes,
/// as written for a machine:
/// the two templates and every text a host would give them.
#[test]
fn the_unit_texts_the_wrapper_writes_carry_no_install_line() {
    let machine: Machine = machine("never-login-texts-lib");
    let mut texts: Vec<(String, String)> = vec![(String::from("SOCKET_UNIT"), SOCKET_UNIT.to_string()), (String::from("SERVICE_UNIT"), SERVICE_UNIT.to_string())];
    for (name, text) in unit_texts(&machine.host) {
        texts.push((name.to_string(), text));
    }
    for (name, text) in texts {
        assert_eq!(lines_that_start_at_login(text.as_str()), Vec::<String>::new(), "{name} would start at startup or login:\n{text}");
    }
    let directory: PathBuf = runtime_unit_dir(machine.host.runtime_dir.as_path());
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned");
    for name in [SOCKET_UNIT_NAME, SERVICE_UNIT_NAME] {
        let written: String = std::fs::read_to_string(directory.join(name)).expect("the written unit");
        assert_eq!(lines_that_start_at_login(written.as_str()), Vec::<String>::new(), "{name} as written");
    }
    remove(machine.root.as_path());
}

/// What a file system entry is,
/// for the comparison before and after:
/// a directory,
/// a file with its bytes,
/// or a link with its target.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Entry {
    Directory,
    File(Vec<u8>),
    Link(PathBuf),
}

/// Every entry below `root`,
/// by path,
/// found with a work stack and not following links.
fn entries_below(root: &Path) -> BTreeMap<PathBuf, Entry> {
    let mut found: BTreeMap<PathBuf, Entry> = BTreeMap::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(listing) = std::fs::read_dir(&directory) else { continue };
        for entry in listing.flatten() {
            let path: PathBuf = entry.path();
            let meta = std::fs::symlink_metadata(&path).expect("metadata");
            if meta.file_type().is_symlink() {
                found.insert(path.clone(), Entry::Link(std::fs::read_link(&path).expect("link target")));
            } else if meta.is_dir() {
                found.insert(path.clone(), Entry::Directory);
                pending.push(path);
            } else {
                found.insert(path.clone(), Entry::File(std::fs::read(&path).unwrap_or_default()));
            }
        }
    }
    return found;
}

/// Plants what the earlier releases left,
/// so the removal paths run too:
/// the single service's units in the user's own unit
///  directory with the enabling link an earlier release made,
/// and the same units in the runtime directory.
fn plant_earlier_releases(machine: &Machine) {
    let user: PathBuf = machine.host.user_unit_dir.clone().expect("a user unit directory");
    earlier_service_files(machine, user.as_path());
    earlier_service_files(machine, runtime_unit_dir(machine.host.runtime_dir.as_path()).as_path());
    let wants: PathBuf = user.join("sockets.target.wants");
    std::fs::create_dir_all(&wants).expect("wants folder");
    std::os::unix::fs::symlink(user.join(LEGACY_SOCKET_UNIT_NAME), wants.join(LEGACY_SOCKET_UNIT_NAME)).expect("the earlier enabling link");
}

/// The runs a guard looks at:
/// a fresh machine,
/// a second run,
/// a machine the earlier releases left files on,
/// and a unit naming a
///  moved program.
/// Each gives the machine,
/// the entries before and after,
/// and the manager's calls.
fn runs() -> Vec<(Machine, BTreeMap<PathBuf, Entry>, BTreeMap<PathBuf, Entry>, Vec<String>)> {
    let mut all: Vec<(Machine, BTreeMap<PathBuf, Entry>, BTreeMap<PathBuf, Entry>, Vec<String>)> = Vec::new();
    for (name, plant, twice) in [("never-login-fresh-lib", false, false), ("never-login-second-lib", false, true), ("never-login-earlier-lib", true, false)] {
        let machine: Machine = machine(name);
        std::fs::create_dir_all(&machine.root).expect("root");
        if plant {
            plant_earlier_releases(&machine);
        }
        if twice {
            provision_with(&machine.host, &mut Recorder::default()).expect("first run");
            add_socket(&machine);
        }
        let before: BTreeMap<PathBuf, Entry> = entries_below(machine.root.as_path());
        let mut recorder: Recorder = Recorder::default();
        provision_with(&machine.host, &mut recorder).expect("provisioned");
        let after: BTreeMap<PathBuf, Entry> = entries_below(machine.root.as_path());
        let calls: Vec<String> = recorder.calls();
        all.push((machine, before, after, calls));
    }
    let mut moved: Machine = machine("never-login-moved-lib");
    provision_with(&moved.host, &mut Recorder::default()).expect("first run");
    moved.host.service_binary = moved.root.join("programs/moved");
    let before: BTreeMap<PathBuf, Entry> = entries_below(moved.root.as_path());
    let mut recorder: Recorder = Recorder::default();
    provision_with(&moved.host, &mut recorder).expect("second run");
    let after: BTreeMap<PathBuf, Entry> = entries_below(moved.root.as_path());
    let calls: Vec<String> = recorder.calls();
    all.push((moved, before, after, calls));
    return all;
}

/// Provisioning creates or changes entries only below the runtime unit directory,
/// and creates no link and no `*.wants` or
///  `*.requires` folder anywhere.
/// The one allowed exception is removal:
/// the earlier releases' files in the user's own unit
///  directory (and the earlier socket file in the runtime directory) may go.
#[test]
fn provisioning_writes_only_in_the_runtime_unit_directory_and_links_nothing() {
    for (machine, before, after, _calls) in runs() {
        let runtime_units: PathBuf = runtime_unit_dir(machine.host.runtime_dir.as_path());
        let user_units: PathBuf = machine.host.user_unit_dir.clone().expect("a user unit directory");
        for (path, entry) in &after {
            let changed: bool = before.get(path) != Some(entry);
            if !changed {
                continue;
            }
            let ancestors_of_runtime_units: bool = runtime_units.starts_with(path);
            assert!(path.starts_with(&runtime_units) || ancestors_of_runtime_units, "{} was created or changed outside {}", path.display(), runtime_units.display());
            // What: The one link provisioning makes is the program the units start, and it is named by the unit file,
            //       not an enabling link of a target.
            // Why:  A link anywhere else would make a unit start at login, which the owner forbids; the pointer is a file the
            //       manager never reads as a unit, and the service unit names it as its program.
            if matches!(entry, Entry::Link(_)) {
                assert_eq!(path, &crate::service_provision_program_pointer::program_pointer_path(machine.host.runtime_dir.as_path()), "{} is a link", path.display());
            }
            let name: String = path.file_name().map(|text| { return text.to_string_lossy().into_owned(); }).unwrap_or_default();
            assert!(!name.ends_with(".wants") && !name.ends_with(".requires"), "{} is an enabling folder", path.display());
        }
        for path in before.keys().filter(|path| { return !after.contains_key(*path); }) {
            assert!(path.starts_with(&user_units) || path.starts_with(&runtime_units) || path.starts_with(machine.host.runtime_dir.as_path()), "{} was removed from outside the unit directories", path.display());
        }
        remove(machine.root.as_path());
    }
}

/// Whether a `*.wants` link of the units is under the user's own unit directory in `entries`.
fn path_in_user_units(entries: &BTreeMap<PathBuf, Entry>, user_units: &Path) -> bool {
    return entries.keys().any(|path| { return path.starts_with(user_units) && path.to_string_lossy().contains(".wants/"); });
}

/// The earlier release's enabling link is removed with its unit,
/// not kept:
/// nothing in the user's own unit directory enables a
///  unit after provisioning.
#[test]
fn an_earlier_releases_enabling_link_is_removed() {
    let machine: Machine = machine("never-login-link-lib");
    plant_earlier_releases(&machine);
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned");
    let after: BTreeMap<PathBuf, Entry> = entries_below(machine.root.as_path());
    let user_units: PathBuf = machine.host.user_unit_dir.clone().expect("a user unit directory");
    assert!(!path_in_user_units(&after, &user_units), "{after:?}");
    remove(machine.root.as_path());
}

/// The manager is asked only to reload,
/// reset a failed unit,
/// start,
/// stop and report a state:
/// no call enables,
/// links,
/// presets,
///  masks or changes the default target,
/// whatever the run.
#[test]
fn provisioning_asks_the_manager_for_nothing_that_enables_a_unit() {
    let allowed: [&str; 6] = ["reload", "reset-failed", "start", "stop", "stop-alone", "state"];
    let mut asked: usize = 0;
    for (machine, _before, _after, calls) in runs() {
        for call in &calls {
            let verb: &str = call.split(' ').next().unwrap_or("");
            assert!(allowed.contains(&verb), "{call}: not a call that provisioning may make");
        }
        asked += calls.len();
        remove(machine.root.as_path());
    }
    assert!(asked > 0, "one of the runs asked the manager something, so the list is checked against real calls");
}
