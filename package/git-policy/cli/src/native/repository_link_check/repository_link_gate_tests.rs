//! Controls for the repository link gate:
//! what each command costs when the identity file is absent,
//! when the command is elsewhere,
//! in phase `before`,
//! and what a dangling link prints.

use super::*;
use crate::repository_link_fixture_probe::{FixtureProbe, World};
use crate::repository_link_identity::{IDENTITY_RELATIVE_PATH, LinkPhase};
use crate::repository_link_probe::{EntryKind, RealLinkProbe};
use crate::test_support::{fixture, remove};

const LINK: &str = "/home/u/Monochromatic";

/// Writes an identity file below a home folder.
fn write_identity(home: &Path, phase: &str, mount_point: &str) {
    let path: PathBuf = home.join(IDENTITY_RELATIVE_PATH);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("config folder");
    std::fs::write(
        path,
        format!(
            "{{\"linkPath\":\"{LINK}\",\"linkText\":\"/mnt/860/repository/Monochromatic\",\"canonicalRoot\":\"/mnt/860/repository/Monochromatic\",\
             \"mountPoint\":\"{mount_point}\",\"driveUuid\":\"DRIVE\",\"nvmeUuid\":\"NVME\",\"rootCommit\":\"abc\",\"phase\":\"{phase}\"}}"
        ),
    )
    .expect("identity file");
}

/// A world in which the link path is a real folder.
fn folder_world() -> World {
    let mut world: World = World::default();
    world.entries.insert(String::from(LINK), EntryKind::Directory);
    return world;
}

fn probe_over(world: World) -> Arc<FixtureProbe> {
    return Arc::new(FixtureProbe::over(world));
}

/// No home directory,
/// and a home without the identity file,
/// cost the probe nothing and let the command go on.
#[test]
fn without_an_identity_file_the_command_goes_on_and_nothing_is_asked() {
    let home: PathBuf = fixture("link-gate-absent");
    let probe: Arc<FixtureProbe> = probe_over(folder_world());
    assert_eq!(link_gate_with_probe(probe.clone(), None, &[PathBuf::from(LINK)]), LinkGate::Clear);
    assert_eq!(link_gate_with_probe(probe.clone(), Some(home.as_path()), &[PathBuf::from(LINK)]), LinkGate::Clear);
    assert!(probe.questions().is_empty(), "{:?}", probe.questions());
    remove(home.as_path());
}

/// A command whose paths are elsewhere goes on without a question,
/// whatever the phase.
#[test]
fn a_command_elsewhere_asks_nothing() {
    let home: PathBuf = fixture("link-gate-elsewhere");
    for phase in ["before", "after"] {
        write_identity(home.as_path(), phase, "/mnt/860");
        let probe: Arc<FixtureProbe> = probe_over(folder_world());
        assert_eq!(link_gate_with_probe(probe.clone(), Some(home.as_path()), &[PathBuf::from("/usr/bin")]), LinkGate::Clear, "{phase}");
        assert!(probe.questions().is_empty(), "{phase}: {:?}", probe.questions());
    }
    remove(home.as_path());
}

/// In phase `before` a command in the repository costs one `lstat` and goes on when it says folder.
#[test]
fn phase_before_costs_one_lstat() {
    let home: PathBuf = fixture("link-gate-before");
    write_identity(home.as_path(), "before", "/mnt/860");
    let probe: Arc<FixtureProbe> = probe_over(folder_world());
    assert_eq!(link_gate_with_probe(probe.clone(), Some(home.as_path()), &[PathBuf::from(LINK).join("sub")]), LinkGate::Clear);
    assert_eq!(probe.questions(), vec!["lstat"]);
    remove(home.as_path());
}

/// In phase `before` a path that is not a folder is not the repository the identity recorded:
/// the full check refuses.
#[test]
fn phase_before_refuses_a_path_that_is_not_a_folder() {
    let home: PathBuf = fixture("link-gate-before-link");
    write_identity(home.as_path(), "before", "/mnt/860");
    let mut world: World = World::default();
    world.entries.insert(String::from(LINK), EntryKind::Link);
    let gate: LinkGate = link_gate_with_probe(probe_over(world), Some(home.as_path()), &[PathBuf::from(LINK)]);
    let LinkGate::Stop(text) = gate else { panic!("the gate should stop") };
    assert!(text.contains(LINK) && text.ends_with('\n'), "{text}");
    remove(home.as_path());
}

/// In phase `after` a real folder where the link was stops the command with the plain message,
/// and the text is one line.
#[test]
fn phase_after_stops_for_a_real_folder_with_the_plain_message() {
    let home: PathBuf = fixture("link-gate-after-folder");
    write_identity(home.as_path(), "after", "/mnt/860");
    let gate: LinkGate = link_gate_with_probe(probe_over(folder_world()), Some(home.as_path()), &[PathBuf::from(LINK)]);
    assert_eq!(gate, LinkGate::Stop(format!("fatal: {LINK} is a real folder, not the link to the 860; nothing was written\n")));
    remove(home.as_path());
}

/// A dangling link on the real filesystem with the drive not mounted prints the not-mounted message and creates nothing.
#[test]
fn a_dangling_link_prints_the_not_mounted_message_and_creates_nothing() {
    let root: PathBuf = fixture("link-gate-dangling").canonicalize().expect("root");
    let home: PathBuf = root.join("home");
    let mount_point: PathBuf = root.join("not-a-mount");
    let link: PathBuf = root.join("repolink");
    std::fs::create_dir_all(mount_point.as_path()).expect("mount point stand-in");
    std::os::unix::fs::symlink(mount_point.join("repository/Monochromatic"), link.as_path()).expect("dangling link");
    let path: PathBuf = home.join(IDENTITY_RELATIVE_PATH);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("config folder");
    std::fs::write(
        path,
        format!(
            "{{\"linkPath\":\"{}\",\"linkText\":\"{}\",\"canonicalRoot\":\"{}\",\"mountPoint\":\"{}\",\"driveUuid\":\"D\",\"nvmeUuid\":\"N\",\"rootCommit\":\"abc\",\"phase\":\"after\"}}",
            link.display(),
            mount_point.join("repository/Monochromatic").display(),
            mount_point.join("repository/Monochromatic").display(),
            mount_point.display()
        ),
    )
    .expect("identity file");
    let before: Vec<String> = listing(root.as_path());
    let gate: LinkGate = link_gate_with_probe(Arc::new(RealLinkProbe::default()), Some(home.as_path()), &[link.join("sub")]);
    let LinkGate::Stop(text) = gate else { panic!("the gate should stop") };
    assert!(text.starts_with("fatal: the 860 drive is not mounted yet ("), "{text}");
    assert!(text.contains(link.to_string_lossy().as_ref()) && text.ends_with("run this again\n"), "{text}");
    assert_eq!(listing(root.as_path()), before, "nothing was created");
    remove(root.as_path());
}

/// Every path below a folder,
/// sorted,
/// links not followed.
fn listing(folder: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![folder.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).expect("read directory").flatten() {
            found.push(entry.path().to_string_lossy().into_owned());
            if entry.file_type().map(|kind| return kind.is_dir()).unwrap_or(false) {
                pending.push(entry.path());
            }
        }
    }
    found.sort();
    return found;
}

/// An identity file that is not JSON cannot name the repository,
/// so the command goes on (the gate says so on standard error).
#[test]
fn an_unreadable_identity_file_lets_the_command_go_on() {
    let home: PathBuf = fixture("link-gate-unreadable");
    let path: PathBuf = home.join(IDENTITY_RELATIVE_PATH);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("config folder");
    std::fs::write(path, b"not json").expect("file");
    let probe: Arc<FixtureProbe> = probe_over(folder_world());
    assert_eq!(link_gate_with_probe(probe.clone(), Some(home.as_path()), &[PathBuf::from(LINK)]), LinkGate::Clear);
    assert!(probe.questions().is_empty());
    let _phase: LinkPhase = LinkPhase::Before;
    remove(home.as_path());
}

/// A world in which the repository is ready on the 860 under the identity `write_identity` records in phase `after`.
fn ready_world() -> World {
    let root: &str = "/mnt/860/repository/Monochromatic";
    let mut world: World = World::default();
    world.entries.insert(String::from(LINK), EntryKind::Link);
    world.entries.insert(String::from(root), EntryKind::Directory);
    world.entries.insert(format!("{root}/.git"), EntryKind::Directory);
    world.links.insert(String::from(LINK), String::from(root));
    world.real.insert(String::from(LINK), String::from(root));
    world.real.insert(String::from("/mnt/860"), String::from("/mnt/860"));
    world.texts.insert(format!("{root}/.git/monochromatic-root-commit"), String::from("abc\n"));
    world.uuids.insert(String::from(root), String::from("DRIVE"));
    world.uuids.insert(format!("{root}/.git"), String::from("DRIVE"));
    world.mounted.insert(String::from("/mnt/860"));
    return world;
}

/// The identity of `ready_world`.
fn ready_identity(home: &Path) -> LinkIdentity {
    write_identity(home, "after", "/mnt/860");
    match crate::repository_link_identity::read_identity(home) {
        crate::repository_link_identity::IdentityRead::Found(identity) => return identity,
        other => panic!("the identity file was written: {other:?}"),
    }
}

/// A usable repository is walked once:
/// the second meeting of the same identity in one process asks nothing,
/// a refusal is walked again every time,
/// and another identity is walked on its own.
#[test]
fn a_usable_repository_is_walked_once_per_process_and_a_refusal_never_is_remembered() {
    let home: PathBuf = fixture("link-gate-memory");
    let identity: LinkIdentity = ready_identity(home.as_path());
    let paths: Vec<PathBuf> = vec![PathBuf::from(LINK).join("sub")];
    let memory: ReadyMemory = ReadyMemory::new();
    let probe: Arc<FixtureProbe> = probe_over(ready_world());
    assert_eq!(gate_remembering(probe.clone(), &identity, paths.as_slice(), &memory), LinkGate::Clear);
    let walked: usize = probe.questions().len();
    assert!(walked > 1, "the first meeting walks the link: {:?}", probe.questions());
    assert_eq!(gate_remembering(probe.clone(), &identity, paths.as_slice(), &memory), LinkGate::Clear);
    assert_eq!(probe.questions().len(), walked, "the second meeting asked nothing");
    let other: LinkIdentity = LinkIdentity { drive_uuid: String::from("OTHER"), ..identity.clone() };
    let refused: LinkGate = gate_remembering(probe.clone(), &other, paths.as_slice(), &memory);
    assert!(matches!(refused, LinkGate::Stop(_)), "another identity is walked on its own: {refused:?}");
    let before: usize = probe.questions().len();
    let again: LinkGate = gate_remembering(probe.clone(), &other, paths.as_slice(), &memory);
    assert!(matches!(again, LinkGate::Stop(_)), "{again:?}");
    assert!(probe.questions().len() > before, "a refusal is walked again");
    remove(home.as_path());
}
