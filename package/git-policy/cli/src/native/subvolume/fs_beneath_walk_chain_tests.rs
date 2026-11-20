//! Controls for the walk down a chain of names and for the hop that may end on a mount:
//! a chain keeps one handle per folder,
//! a link or a boundary at any hop stops the walk there,
//! and the last hop crosses a mount that a walk refusing the crossing would stop at.

use super::*;
use crate::fs_beneath::{identity_at, identity_of, open_root};
use crate::fs_beneath_mount_facts::mount_id_of;
use crate::test_support::{fixture, remove};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The root of a walk for a fixture directory,
/// by its canonical path.
fn open_directory(path: &Path) -> std::io::Result<File> {
    return open_root(std::fs::canonicalize(path)?.as_path());
}

/// What:
///  A chain of directories is opened one component at a time,
///  each handle keeps naming its own folder,
///  and a link planted at any component stops the walk there.
/// Why:
///  The chain is what proves a parent folder's identity and holds the target folder,
///  so a walk that kept only the last handle would let a rename between two lookups pass unseen.
#[test]
fn a_chain_keeps_a_handle_per_folder_and_stops_at_a_link() {
    let root: PathBuf = fixture("beneath-chain");
    std::fs::create_dir_all(root.join("one/two/three")).expect("tree");
    std::fs::create_dir_all(root.join("elsewhere/two")).expect("elsewhere");
    let held: File = open_directory(root.as_path()).expect("root");
    let names: Vec<OsString> = [OsString::from("one"), OsString::from("two"), OsString::from("three")].to_vec();
    let chain: Vec<File> = open_chain_beneath(&held, &names).expect("chain");
    assert_eq!(chain.len(), names.len(), "one handle per component");
    let mut named_path: PathBuf = root.clone();
    for (name, handle) in ["one", "two", "three"].iter().zip(chain.iter()) {
        named_path = named_path.join(name);
        let named: std::fs::Metadata = std::fs::metadata(&named_path).expect("named");
        let walked: std::fs::Metadata = handle.metadata().expect("walked");
        assert_eq!((walked.dev(), walked.ino()), (named.dev(), named.ino()), "{}", named_path.display());
    }
    // The first handle still names the folder it was opened on after the folder moves.
    std::fs::rename(root.join("one"), root.join("moved-one")).expect("rename");
    assert_eq!(identity_of(&chain[0]).expect("identity"), identity_at(&held, OsStr::new("moved-one")).expect("moved"));
    assert!(open_chain_beneath(&held, &[OsString::from("one"), OsString::from("two")]).is_err(), "the old name is gone");
    // A link at the second component is refused, and the refusal names its position.
    std::os::unix::fs::symlink(root.join("elsewhere/two"), root.join("moved-one/two-linked")).expect("link");
    let refused: ChainRefusal = open_chain_beneath(&held, &[OsString::from("moved-one"), OsString::from("two-linked")]).expect_err("link in the way");
    assert!(matches!(refused, ChainRefusal::Link { step: 1 }), "{refused:?}");
    let empty: Vec<OsString> = Vec::new();
    assert!(open_chain_beneath(&held, &empty).expect("nothing to walk").is_empty());
    remove(root.as_path());
}

/// What:
///  The deepest-handle walk answers the same folder the chain ends on,
///  and refuses a link at any hop as the chain does.
/// Why:
///  Both answers come from one loop,
///  so a rule that changed in one of them would change the other.
#[test]
fn the_deepest_handle_is_the_chain_s_end_and_the_same_hop_stops_both() {
    let root: PathBuf = fixture("beneath-deepest");
    std::fs::create_dir_all(root.join("one/two")).expect("tree");
    let held: File = open_directory(root.as_path()).expect("root");
    let names: Vec<OsString> = [OsString::from("one"), OsString::from("two")].to_vec();
    let deepest: File = open_directory_beneath(&held, &names).expect("the deepest folder");
    let chain: Vec<File> = open_chain_beneath(&held, &names).expect("the chain");
    assert_eq!(identity_of(&deepest).expect("deepest"), identity_of(&chain[1]).expect("chain end"));
    assert_eq!(identity_of(&open_directory_beneath(&held, &[]).expect("the root itself")).expect("root"), identity_of(&held).expect("held"));
    std::os::unix::fs::symlink(root.join("one"), root.join("linked")).expect("link");
    assert!(open_directory_beneath(&held, &[OsString::from("linked")]).is_err(), "a link at the first hop");
    assert!(open_chain_beneath(&held, &[OsString::from("linked")]).is_err(), "and the chain refuses it too");
    remove(root.as_path());
}

/// What:
///  The last hop of a walk crosses a mount that the walk's own open refuses there.
/// Why:
///  Reading the mount a previous attach put at a slot needs the mounted root,
///  while every folder above the slot must still be the walked ones:
///  the walk refuses a mount anywhere else,
///  and a hop that stopped crossing the slot would refuse every attached clone.
#[test]
fn the_last_hop_crosses_a_mount_that_the_walk_refuses() {
    let root: PathBuf = fixture("beneath-mount-hop");
    std::fs::create_dir_all(root.join("real/inner")).expect("tree");
    std::fs::create_dir_all(root.join("elsewhere")).expect("elsewhere");
    std::os::unix::fs::symlink(root.join("elsewhere"), root.join("real/linked")).expect("link");
    let held: File = open_directory(root.join("real").as_path()).expect("root");
    let inner: File = open_directory_at_a_mount(&held, OsStr::new("inner")).expect("a plain folder is opened as before");
    assert_eq!(identity_of(&inner).expect("identity"), identity_at(&held, OsStr::new("inner")).expect("named"));
    assert!(open_directory_at_a_mount(&held, OsStr::new("linked")).is_err(), "a link is refused at the last hop too");
    assert!(open_directory_at_a_mount(&held, OsStr::new("../real")).is_err(), "a name with a slash is no component");
    remove(root.as_path());
}

/// Environment variable that makes the control below run its inner half,
/// inside a mount namespace of its own.
#[cfg(target_os = "linux")]
const MOUNT_SIDE_VARIABLE: &str = "FS_BENEATH_WALK_CHAIN_MOUNT_SIDE";

/// What:
///  With a filesystem mounted on one folder below the held one,
///  the walk's own open refuses that hop with `EXDEV` while the mount-crossing open reaches the mounted root,
///  and the mount is on another device than the folder above it.
/// Why:
///  This is the positive control of the crossing:
///  the host's fixture root holds no mount point,
///  so before this control the same assertions passed with `RESOLVE_NO_XDEV` put back,
///  which would have refused every clone the attach had just mounted.
///  A mount needs a mount namespace,
///  so the inner half runs under `unshare` in a private namespace,
///  and the control skips where the host lets no unprivileged process mount.
#[cfg(target_os = "linux")]
#[test]
#[allow(clippy::disallowed_methods, reason = "starts this test binary, not the wrapper, under `unshare`; the ban on `current_exe` guards the wrapper's own start")]
fn the_last_hop_crosses_a_real_mount_in_a_namespace_of_its_own() {
    if std::env::var_os(MOUNT_SIDE_VARIABLE).is_none() {
        let probe: std::process::Output = std::process::Command::new("unshare")
            .args(["--user", "--map-root-user", "--mount", "--propagation", "private", "--", "mount", "--version"])
            .output()
            .expect("unshare runs");
        if !probe.status.success() {
            eprintln!("skipped: this host lets no unprivileged process mount in a namespace of its own, so no folder here is a mount point");
            return;
        }
        let done: std::process::Output = std::process::Command::new("unshare")
            .args(["--user", "--map-root-user", "--mount", "--propagation", "private"])
            .arg(std::env::current_exe().expect("this test binary"))
            .args(["--exact", "subvolume::fs_beneath_walk_chain::tests::the_last_hop_crosses_a_real_mount_in_a_namespace_of_its_own", "--nocapture"])
            .env(MOUNT_SIDE_VARIABLE, "1")
            .output()
            .expect("the inner half runs");
        let said: String = String::from_utf8_lossy(done.stdout.as_slice()).to_string() + String::from_utf8_lossy(done.stderr.as_slice()).as_ref();
        assert!(done.status.success(), "the inner half failed:\n{said}");
        assert!(said.contains("1 passed"), "the inner half did not run:\n{said}");
        return;
    }
    let root: PathBuf = fixture("beneath-mount-real");
    std::fs::create_dir_all(root.join("real/mounted")).expect("the folder a mount is put on");
    let mounted: MountedHere = MountedHere::at(root.join("real/mounted").as_path());
    let held: File = open_directory(root.join("real").as_path()).expect("root");
    let plain: std::io::Error = open_directory_at(&held, OsStr::new("mounted")).expect_err("the walk refuses the mount point");
    assert_eq!(plain.raw_os_error(), Some(libc::EXDEV), "the resolution refuses to cross a mount: {plain}");
    let crossed: File = open_directory_at_a_mount(&held, OsStr::new("mounted")).expect("the last hop crosses the mount");
    assert_ne!(device_of(&crossed).expect("device"), device_of(&held).expect("device"), "the crossed folder is on the mounted filesystem");
    assert_eq!(mount_id_of(&crossed).expect("mount"), mounted.id, "the crossed handle is on the mount this control made");
    // Every hop above the last is still the walked one: the chain refuses the same mount point.
    assert!(open_chain_beneath(&held, &[OsString::from("mounted")]).is_err(), "a chain never crosses a mount, whatever hop it is at");
    drop(mounted);
    remove(root.as_path());
}

/// What:
///  A filesystem this control mounted,
///  unmounted when it goes however the control ends,
///  so a failed assertion leaves no mount behind.
#[cfg(target_os = "linux")]
struct MountedHere {
    /// The directory the filesystem was mounted on.
    at: PathBuf,
    /// The mount ID the kernel gave that mount,
    /// read after it was made.
    id: u64,
}

#[cfg(target_os = "linux")]
impl MountedHere {
    /// Mount a tmpfs on `path`,
    /// which this process may only do as root in a mount namespace of its own.
    fn at(path: &Path) -> Self {
        let mounted: std::process::Output = std::process::Command::new("mount").args(["-t", "tmpfs", "tmpfs"]).arg(path).output().expect("the mount command runs");
        assert!(mounted.status.success(), "a tmpfs is mounted at {}: {}", path.display(), String::from_utf8_lossy(mounted.stderr.as_slice()));
        let handle: File = open_directory(path).expect("the mounted folder");
        let id: u64 = mount_id_of(&handle).expect("the mount's ID");
        return Self { at: path.to_path_buf(), id };
    }
}

#[cfg(target_os = "linux")]
impl Drop for MountedHere {
    fn drop(&mut self) {
        let _unmounted: std::io::Result<std::process::Output> = std::process::Command::new("umount").arg(self.at.as_path()).output();
    }
}
