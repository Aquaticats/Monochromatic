//! What:
//! The places where a link to an old copy of a repository tool can be:
//! `~/.local/bin`,
//! the machine-global exposure directory (the global farm),
//! the shared cargo bin directory the copies sit in (a sibling link there would dangle once the copy is gone),
//! and the `.cache/*-bin` directories of every checkout of the repository.
//! Also the scan that finds the links in them that resolve to one given file.
//! Why:
//! The owner's words for the cleanup:
//! "links in known places (~/.local/bin,
//! every checkout's .cache/*-bin,
//! the global farm)".
//! A place that cannot be listed is recorded as a reason the cleanup must not remove anything,
//! so no link that could be seen is ever left dangling.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // old-copy-places.ts
//! export function knownPlaces(scope: Scope, env: EnvMap, runner: Runner): { directories: Place[]; incomplete: string[] }
//! export function linksToCopy(places: Places, copy: string): { found: FoundLink[]; unreadable: string[] }
//! ```

/// Imports `BTreeSet` from `std::collections`.
use std::collections::BTreeSet;
/// Imports `fs` from `std`.
use std::fs;
/// Imports `ErrorKind` from `std::io`.
use std::io::ErrorKind;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `cargo_bin_dir` from `super::layout`.
use super::layout::cargo_bin_dir;
/// Imports `listed_worktrees` from `super::worktrees`.
use super::worktrees::listed_worktrees;
/// Imports `Runner` from `super::runner`.
use super::runner::Runner;
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope, ScopeKind, global_scope};

/// The suffix of the link directories a checkout keeps under `.cache`
/// (`monochromatic-git-policy-cli-bin`,
/// `mpm-shadow-bin`,
/// `search-shadow-bin`,
/// `toolchain-bin`).
/// The directories are found by this suffix,
/// not by a list,
/// because the postinstall step of a tool decides its own directory name.
const LINK_DIRECTORY_SUFFIX: &str = "-bin";

/// Which kind of place a directory is.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PlaceKind {
    /// `~/.local/bin`,
    /// where the global scope's postinstall links go.
    LocalBin,
    /// The machine-global exposure directory.
    GlobalFarm,
    /// The shared cargo bin directory itself,
    /// where a sibling link to a copy (`git` to `cli-git-native`) can sit beside the copy.
    CargoBin,
    /// A `.cache/*-bin` directory of one checkout of the repository.
    Checkout,
}

/// The methods of `PlaceKind`.
impl PlaceKind {
    /// Whether the place belongs to no checkout.
    pub fn is_outside_checkout(self) -> bool {
        return self != PlaceKind::Checkout;
    }
}

/// One directory whose symbolic links the cleanup inspects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Place {
    /// The kind of place.
    pub kind: PlaceKind,
    /// The directory.
    pub directory: PathBuf,
}

/// The places a run can see,
/// and the reasons it cannot see all of them.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Places {
    /// Every directory to inspect.
    pub directories: Vec<Place>,
    /// Why some place is unseen;
    /// while this is not empty,
    /// no old copy is removed.
    pub incomplete: Vec<String>,
}

/// One symbolic link found in a place.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FoundLink {
    /// The link itself.
    pub path: PathBuf,
    /// The kind of place it sits in.
    pub kind: PlaceKind,
}

/// The links to one file,
/// and the directories that could not be listed.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LinkScan {
    /// The links that resolve to the file,
    /// sorted.
    pub found: Vec<FoundLink>,
    /// Directories that exist but cannot be listed.
    pub unreadable: Vec<PathBuf>,
}

/// The path with symbolic links resolved when it exists,
/// otherwise as given (so two spellings of an existing directory,
/// such as `/home` and `/var/home`,
/// compare equal).
fn resolved(path: &Path) -> PathBuf {
    return match fs::canonicalize(path) {
        Ok(found) => found,
        Err(_) => path.to_path_buf(),
    };
}

/// Adds `kind` places for the `.cache/*-bin` directories of one checkout.
fn add_checkout_directories(checkout: &Path, places: &mut Places) {
    let cache: PathBuf = checkout.join(".cache");
    let entries = match fs::read_dir(&cache) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return,
        Err(error) => {
            places.incomplete.push(format!("{} cannot be listed: {error}", cache.display()));
            return;
        }
    };
    let mut directories: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(LINK_DIRECTORY_SUFFIX) && entry.path().is_dir() {
            directories.push(entry.path());
        }
    }
    directories.sort();
    for directory in directories {
        places.directories.push(Place { kind: PlaceKind::Checkout, directory });
    }
}

/// What:
/// The checkouts of the repository the run can see:
/// the scope's own worktree and every worktree git lists for it.
/// A global run sees none,
/// because the global scope has no repository.
/// Why:
/// A worktree git lists but whose directory is gone is skipped only when git marks it `prunable`;
/// otherwise its links are unseen and the reason is recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function visibleCheckouts(scope: Scope, runner: Runner, places: Places): string[]
/// ```
fn visible_checkouts(scope: &Scope, runner: &dyn Runner, places: &mut Places) -> Vec<PathBuf> {
    if scope.kind == ScopeKind::Global {
        places.incomplete.push(
            "a global install does not search the repository's checkouts; run `mpm-shadow install` inside a checkout to include them"
                .to_string(),
        );
        return Vec::new();
    }
    let mut checkouts: Vec<PathBuf> = vec![scope.root.clone()];
    match listed_worktrees(runner, &scope.root) {
        Ok(listed) => {
            for worktree in listed {
                if worktree.directory.is_dir() {
                    checkouts.push(worktree.directory);
                } else if !worktree.prunable {
                    places.incomplete.push(format!(
                        "the worktree {} is listed by git but its directory is missing and not marked prunable, so its links are unseen",
                        worktree.directory.display()
                    ));
                }
            }
        }
        Err(error) => places.incomplete.push(error.message),
    }
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    let mut distinct: Vec<PathBuf> = Vec::new();
    for checkout in checkouts {
        if seen.insert(resolved(&checkout)) {
            distinct.push(checkout);
        }
    }
    return distinct;
}

/// What:
/// Every place the run can inspect for links to an old copy.
/// Why:
/// The places are `~/.local/bin` and the global farm,
/// which are the same whichever scope runs,
/// then the `.cache/*-bin` directories of each visible checkout.
/// Asking git for the worktrees happens here,
/// so a caller that has no old copy to judge never calls this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function knownPlaces(scope: Scope, env: EnvMap, runner: Runner): Places
/// ```
pub fn known_places(scope: &Scope, env: &EnvMap, runner: &dyn Runner) -> Places {
    let mut places: Places = Places::default();
    match global_scope(env) {
        Ok(global) => {
            places.directories.push(Place { kind: PlaceKind::LocalBin, directory: global.link_dir });
            places.directories.push(Place { kind: PlaceKind::GlobalFarm, directory: global.exposure_dir });
            places.directories.push(Place { kind: PlaceKind::CargoBin, directory: cargo_bin_dir(env, &global.home) });
        }
        Err(error) => places.incomplete.push(error.message),
    }
    for checkout in visible_checkouts(scope, runner, &mut places) {
        add_checkout_directories(&checkout, &mut places);
    }
    return places;
}

/// What:
/// The symbolic links in `places` whose final target is the file `copy`,
/// directly or through other links.
/// Why:
/// The comparison is between resolved paths,
/// never between texts:
/// the same file is spelled `/home/user/.cargo/bin/x` in one link and `/var/home/user/.cargo/bin/x` in another on the
/// owner's machine (`/home` is a link to `var/home`).
/// A link that does not resolve cannot point at an existing file,
/// so it is never reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function linksToCopy(places: Places, copy: string): { found: FoundLink[]; unreadable: string[] }
/// ```
pub fn links_to_copy(places: &Places, copy: &Path) -> LinkScan {
    let mut scan: LinkScan = LinkScan::default();
    let Ok(target) = fs::canonicalize(copy) else {
        return scan;
    };
    for place in &places.directories {
        let entries = match fs::read_dir(&place.directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(_) => {
                scan.unreadable.push(place.directory.clone());
                continue;
            }
        };
        for entry in entries.flatten() {
            let is_link: bool = match entry.file_type() {
                Ok(kind) => kind.is_symlink(),
                Err(_) => false,
            };
            if !is_link {
                continue;
            }
            if let Ok(resolved_link) = fs::canonicalize(entry.path())
                && resolved_link == target
            {
                scan.found.push(FoundLink { path: entry.path(), kind: place.kind });
            }
        }
    }
    scan.found.sort();
    return scan;
}

/// What:
/// The places of this run,
/// searched the first time they are asked for and kept in `slot` after that.
/// Why:
/// Asking git for the worktrees costs a process,
/// and a run with no copy to judge must not pay it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function placesOf(slot: { places?: Places }, ctx: Context): Places
/// ```
pub fn places_of<'a>(slot: &'a mut Option<Places>, scope: &Scope, env: &EnvMap, runner: &dyn Runner) -> &'a Places {
    if slot.is_none() {
        *slot = Some(known_places(scope, env, runner));
    }
    return slot.get_or_insert_with(Places::default);
}
