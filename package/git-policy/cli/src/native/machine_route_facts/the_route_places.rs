//! What:
//!  Every path this machine's route facts are read from,
//!  named once,
//!  so the gathering code resolves no path of its own and a control reads a
//!  fixture folder instead of this machine's.
//! Why:
//!  The design fixes each place:
//!  the handler's configuration,
//!  the identity store,
//!  the two `/run` folders,
//!  the installed executable and zedlet,
//!  ZED's unit and the folders a wants link stands in,
//!  the ZFS device,
//!  the mount table and this boot's identifier
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZFS reconciliation and mount safety" and
//!  "Dataset and path layout";
//!  `crate::worktree_clone_mount_handler::handler_install_paths`).
//!  The account's cache is not one of them:
//!  it hangs under the managed account's home,
//!  which the environment names,
//!  so it is a reading rather than a constant
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The staging target and who creates it").
//!  A place named here and read elsewhere is a fixture root away from a value
//!  test,
//!  which is why every reader takes its place from this value and none writes
//!  a path of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const places = TheRoutePlaces.live(); // { configuration: '/etc/…', store: '/var/lib/…', mountTable: '/proc/self/mountinfo', … }
//! const fixture = TheRoutePlaces.under(fixtureRoot);
//! ```

/// Where the kernel lists the mounts of this process's mount namespace,
/// which is the table the route's mount parser reads.
use crate::subvolume::fs_mount_options::MOUNTINFO_PATH;
/// Where the route keeps the identity records the design compares against.
use crate::worktree_clone_mount::SLOT_IDENTITY_STORE_PATH;
/// Where the handler's configuration file lives.
use crate::worktree_clone_mount_handler::handler_install_paths::CONFIGURATION_PATH;
/// The one file root's modes run as.
use crate::worktree_clone_mount_handler::handler_install_paths::ROOT_MODE_INSTALL_PATH;
/// The ZED history event file ZED starts the sweep by.
use crate::worktree_clone_mount_handler::handler_install_paths::ZEDLET_INSTALL_PATH;
/// The `/run` folder one boot's readiness record is published in.
use crate::worktree_clone_mount_protocol::READINESS_DIRECTORY;
/// The `/run` folder the handler files its receipts in.
use crate::worktree_clone_mount_protocol::RESULTS_DIRECTORY;
/// Where the kernel reports this boot's identifier,
/// from the module that reads it for a staging call.
use crate::worktree_clone_unmount::staging_calls::BOOT_ID_PATH;
/// The ZFS device the route reads the pool through.
use monochromatic_zfs_request_to_openzfs_kernel_device::DEVICE_PATH;
/// An owned path,
/// and the borrowed one `Path` a place is built from.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Every place this machine's route facts are read from,
///  each held as a path this value owns.
/// Why:
///  One value keeps the release's constants and a control's fixture in one
///  shape,
///  so the gathering code takes no path argument of its own and a test reads
///  no path of this machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheRoutePlaces = { configuration; store; home; readiness; results; executable; zedlet; zedletsFolder; zedUnits; zedEnablements; device; mountTable; bootIdentity };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TheRoutePlaces {
    /// The handler's configuration file.
    pub configuration: PathBuf,
    /// The folder holding one identity record per slot and `.parent`.
    pub store: PathBuf,
    /// The managed account's home,
    /// whose `.cache` holds the staging folders,
    /// or nothing when the environment names none.
    pub home: Option<PathBuf>,
    /// The folder one boot's readiness record lives in.
    pub readiness: PathBuf,
    /// The folder the handler files its receipts in.
    pub results: PathBuf,
    /// The one file root's modes run as.
    pub executable: PathBuf,
    /// The ZED history event file that starts the sweep.
    pub zedlet: PathBuf,
    /// The folder the installed zedlets sit in,
    /// which is every file ZED starts.
    pub zedlets_folder: PathBuf,
    /// The places ZED's own unit may be installed at,
    /// in the order the release looks.
    pub zed_units: Vec<PathBuf>,
    /// The places a link enabling ZED's unit may stand,
    /// in the order the release looks.
    pub zed_enablements: Vec<PathBuf>,
    /// The ZFS device the pool is read through.
    pub device: PathBuf,
    /// The kernel's table of the caller's own mounts.
    pub mount_table: PathBuf,
    /// The kernel's answer naming this boot.
    pub boot_identity: PathBuf,
}

impl TheRoutePlaces {
    /// What:
    ///  The places this machine runs the route from:
    ///  the release's own constants,
    ///  ZED's unit where a source build or a package puts it,
    ///  and the cache under the account's home.
    /// Why:
    ///  Nothing here is a choice of the gatherer's:
    ///  each place is a constant of the build or the environment's answer,
    ///  so a report names the file a person opens.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const places = TheRoutePlaces.live();
    /// ```
    pub(crate) fn live() -> Self {
        return TheRoutePlaces {
            configuration: PathBuf::from(CONFIGURATION_PATH),
            store: PathBuf::from(SLOT_IDENTITY_STORE_PATH),
            home: home_named_by_the_environment(),
            readiness: PathBuf::from(READINESS_DIRECTORY),
            results: PathBuf::from(RESULTS_DIRECTORY),
            executable: PathBuf::from(ROOT_MODE_INSTALL_PATH),
            zedlet: PathBuf::from(ZEDLET_INSTALL_PATH),
            zedlets_folder: the_zedlets_folder(),
            zed_units: ZED_UNIT_PLACES
                .iter()
                .map(|place| return PathBuf::from(place))
                .collect(),
            zed_enablements: ZED_ENABLEMENT_PLACES
                .iter()
                .map(|place| return PathBuf::from(place))
                .collect(),
            device: PathBuf::from(DEVICE_PATH),
            mount_table: PathBuf::from(MOUNTINFO_PATH),
            boot_identity: PathBuf::from(BOOT_ID_PATH),
        };
    }

    /// What:
    ///  The same places under one folder a control made,
    ///  so every read lands on a file the control wrote and none on this
    ///  machine's.
    /// Why:
    ///  A value test proves what a read answers for a file that is absent,
    ///  a file of the wrong kind and a file of the right one,
    ///  and each of those needs a path no other test and no live machine
    ///  shares.
    ///  The device is the one fact a fixture cannot stand in for,
    ///  so a control that gathers hands in a scripted pool and never reaches
    ///  `/dev/zfs`
    ///  (`super::the_machines_pool_answers`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const places = TheRoutePlaces.under(fixtureRoot);
    /// // places.configuration === `${fixtureRoot}/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc`
    /// ```
    #[cfg(test)]
    pub(crate) fn under(root: &Path) -> Self {
        return TheRoutePlaces {
            configuration: below(root, Path::new(CONFIGURATION_PATH)),
            store: below(root, Path::new(SLOT_IDENTITY_STORE_PATH)),
            home: Some(below(root, Path::new(HOME_IN_A_FIXTURE))),
            readiness: below(root, Path::new(READINESS_DIRECTORY)),
            results: below(root, Path::new(RESULTS_DIRECTORY)),
            executable: below(root, Path::new(ROOT_MODE_INSTALL_PATH)),
            zedlet: below(root, Path::new(ZEDLET_INSTALL_PATH)),
            zedlets_folder: below(root, the_zedlets_folder().as_path()),
            zed_units: ZED_UNIT_PLACES
                .iter()
                .map(|place| return below(root, Path::new(place)))
                .collect(),
            zed_enablements: ZED_ENABLEMENT_PLACES
                .iter()
                .map(|place| return below(root, Path::new(place)))
                .collect(),
            device: below(root, Path::new(DEVICE_PATH)),
            mount_table: below(root, Path::new(MOUNTINFO_PATH)),
            boot_identity: below(root, Path::new(BOOT_ID_PATH)),
        };
    }

    /// What:
    ///  The account's cache folder,
    ///  which is where a removal stages a clone,
    ///  or the reason it cannot be named.
    /// Why:
    ///  Every staging fact hangs under this one name,
    ///  and a machine whose environment names no home must be told so rather
    ///  than read as if its cache were the file system's root,
    ///  so the same sentence is the reason a check answers unknown and the
    ///  reason no staging bind can be looked for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const cache = places.theCacheOrWhy(); // '/home/someone/.cache' | throws
    /// ```
    pub(crate) fn the_cache_or_why(&self) -> Result<PathBuf, String> {
        let Some(home) = self.home.as_ref() else {
            return Err(String::from(THE_CACHE_WITHOUT_A_HOME));
        };
        // What:     `Path::join` appends `.cache` to the home, so the folder hangs under it.
        // Why:      The design puts the staging folder under the account's own cache and nowhere else.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return join(home, '.cache');
        // ```
        return Ok(home.join(CACHE_FOLDER_NAME));
    }
}

/// The folder under the account's home that a removal stages a clone in.
const CACHE_FOLDER_NAME: &str = ".cache";

/// Why the account's cache cannot be named,
/// which is the one name the design would have used and the environment's own
/// silence.
const THE_CACHE_WITHOUT_A_HOME: &str = "the environment names no home, so the account's cache folder cannot be named and `.cache` could be any account's";

/// The folder a control's fixture home carries,
/// so no fixture reads the person's own home.
#[cfg(test)]
const HOME_IN_A_FIXTURE: &str = "home";

/// What:
///  The places ZED's own unit file is installed at,
///  in the order the release looks:
///  a source build's prefix,
///  then the two folders a distribution's package uses.
/// Why:
///  A guest that builds OpenZFS from source and a machine that installs the
///  package put the same unit in different folders,
///  and a report that looked at one place would call a wired machine unwired.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const zedUnits = ['/lib/systemd/system/zfs-zed.service', '/usr/lib/systemd/system/zfs-zed.service', '/opt/openzfs-2.4.4/lib/systemd/system/zfs-zed.service'];
/// ```
const ZED_UNIT_PLACES: &[&str] = &[
    "/lib/systemd/system/zfs-zed.service",
    "/usr/lib/systemd/system/zfs-zed.service",
    "/opt/openzfs-2.4.4/lib/systemd/system/zfs-zed.service",
];

/// What:
///  The places a link enabling ZED's unit may stand.
/// Why:
///  Enabling a unit writes a wants link under the target that wants it,
///  and a package's unit may be wanted by the multi-user target or carry its
///  own wants folder,
///  so both shapes count as enabled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const enablements = ['/etc/systemd/system/multi-user.target.wants/zfs-zed.service', '/etc/systemd/system/zfs-zed.service.wants'];
/// ```
const ZED_ENABLEMENT_PLACES: &[&str] = &[
    "/etc/systemd/system/multi-user.target.wants/zfs-zed.service",
    "/etc/systemd/system/zfs-zed.service.wants",
];

/// What:
///  The folder the installed zedlets sit in,
///  read from the one install path the build fixes.
/// Why:
///  The event file's own folder is where ZED looks for every other event,
///  so one constant decides both and the two cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const zedletsFolder = dirname(ZEDLET_INSTALL_PATH);
/// ```
fn the_zedlets_folder() -> PathBuf {
    let installed: &Path = Path::new(ZEDLET_INSTALL_PATH);
    // What:     `Path::parent` gives the folder that holds the file, or nothing for a path with one component.
    // Why:      The install path is absolute, so it always has a folder, and the file system's root serves as a
    //           stand-in for a build whose constant stops being absolute.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return installed.parent ?? '/';
    // ```
    let folder: &Path = installed.parent().unwrap_or(Path::new("/"));
    // What:     `Path::to_path_buf` makes an owned copy of the borrowed folder, which the places value keeps.
    // Why:      The answer outlives the constant's borrowed spelling.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return folder;
    // ```
    return folder.to_path_buf();
}

/// What:
///  The home the environment names for this account,
///  or nothing when it names none or names an empty one.
/// Why:
///  The staging folder hangs under the account's own home,
///  and an empty variable would make `.cache` a folder at the file system's
///  root rather than under a home.
///  `var_os` is the read that keeps bytes a home may hold and text cannot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const home = process.env.HOME ? process.env.HOME : undefined;
/// ```
fn home_named_by_the_environment() -> Option<PathBuf> {
    let named: std::ffi::OsString = std::env::var_os("HOME")?;
    if named.is_empty() {
        return None;
    }
    // What:     `PathBuf::from` owns the text the environment gave, so the answer outlives the value it was read from.
    // Why:      The home is kept in the places value, which the gatherer holds for the whole report.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return PathBuf.from(named);
    // ```
    return Some(PathBuf::from(named));
}

/// What:
///  One absolute place moved under a fixture root,
///  keeping the place's own components.
/// Why:
///  A control's fixture must keep the shape the release reads
///  (`etc/monochromatic-git-policy-cli/…`),
///  so a reader that names the file says the same words in a report either
///  way,
///  and `Path::join` given an absolute path would replace the root instead of
///  hanging the place under it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const below = (root, place) => join(root, place.slice(1));
/// ```
#[cfg(test)]
fn below(
    root: &Path,
    place: &Path,
) -> PathBuf {
    // What:     `strip_prefix` takes the leading `/` off so `join` appends the place under the root.
    // Why:      An absolute path given to `join` replaces the root, which would read this machine's file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const relative = place.startsWith('/') ? place.slice(1) : place;
    // ```
    let relative: &Path = place.strip_prefix("/").unwrap_or(place);
    // What:     `Path::join` makes the place's path under the root, one component list after another.
    // Why:      The fixture's file is named by the same components the release names.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return join(root, relative);
    // ```
    return root.join(relative);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_route_places_tests.rs"]
mod tests;
