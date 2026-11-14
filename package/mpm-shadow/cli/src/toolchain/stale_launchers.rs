//! What:
//! The retirement of what `install` once made for a name the lock declares nowhere:
//! a launcher in the checkout's launcher folder,
//! the `<name>.launch` record beside it,
//! and the links in the checkout's link folders that name it.
//! Why:
//! `install` writes a launcher and its record for each bin the lock declares,
//! and touches no other name,
//! so a tool renamed at its release left its old launcher,
//! record and exposed link in the checkout for good and no install could ever rewrite them.
//! A new worktree starts as a copy of the checkout it was made from,
//! `.cache` included,
//! so every new worktree arrived with a record naming the other checkout's lock,
//! and the commit wrapper counted it as a stale record after every relink and gave up
//! (measured 2026-11-13:
//! 142 of 160 worktrees and the main checkout carried it,
//! and 352 relinks in one day failed on it).
//! What may be removed is decided from the files themselves and never from a name alone:
//! a record this tool can read and whose tool this lock still declares,
//! a launcher whose own bytes are one of `mpm-shadow`'s own programs,
//! and a link whose target is such a launcher.
//! A link is judged in every folder `mpm-shadow` keeps links in,
//! not only in the farm and the places the record of postinstall links names,
//! because a run killed between making those links and recording them leaves links nothing names.
//! A link that does not resolve is proven only when it said its name was in a launcher folder,
//! so a person's broken link that shares a name with a launcher stays.
//! A file whose own bytes are not provably `mpm-shadow`'s stays where it is,
//! and the install names it and why.
//! The removal moves the entry it judged aside in one atomic step and compares what moved with what was judged
//! (`super::removal`),
//! so a file a person puts at the path while the proof runs is put back rather than deleted.
//! An install that cannot read the lock never reaches here at all,
//! the caller holds the scope's lock over these writes so no install running beside this one is seen half done,
//! and the removal order keeps a link from ever pointing at a launcher that is gone:
//! the links go first,
//! then the launcher,
//! then its record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // stale-launchers.ts
//! export function retireUndeclaredLaunchers(ctx: Context, madeLinks: string[]): LauncherNotices
//! ```

/// Imports `BTreeSet` from `std::collections`.
use std::collections::BTreeSet;
/// Imports `fs` from `std`.
use std::fs;
/// Imports the read trait the block comparison and the record read fill their buffers with.
use std::io::Read;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports the identity every shadow carries inside its own file.
use crate::shadow_identity::identity_of_open_file;

/// Imports `is_repository_published` from `super::cargo_installs`.
use super::cargo_installs::is_repository_published;
/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `link_directory` from `super::exposure`.
use super::exposure::link_directory;
/// Imports what this module uses from `super::launcher`.
use super::launcher::{RECORD_SUFFIX, parse_record, record_path};
/// Imports what this module uses from `super::launcher_source`.
use super::launcher_source::{launcher_source, newest_installed_shadow, program_name, running_program};
/// Imports `Postinstall` from `super::model`.
use super::model::Postinstall;
/// Imports what this module uses from `super::postinstall_links`.
use super::postinstall_links::{declared_link_paths, recorded_link_paths};
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope, global_scope};
/// Imports what this module uses from `super::removal`.
use super::removal::{Judged, Removal, judged_of_metadata, remove_if_still};

//region The crate mpm-shadow places, the folders it links into, and how much of a file is read at a time

/// The crate name this tool's own launchers carry in their identity text.
/// A launcher is a copy of the `mpm-shadow` program,
/// never of one of the repository's other shadows,
/// whose identity text names them instead.
const OWN_SHADOW_CRATE: &str = "monochromatic-mpm-shadow";

/// The suffix of the directories under a checkout's `.cache` that a tool's postinstall step links into
/// (`mpm-shadow-bin`,
/// `monochromatic-git-policy-cli-bin`).
/// The same suffix the cleanup of old copies searches for links,
/// and the way to find a link that an unrecorded or damaged record does not name.
const LINK_DIRECTORY_SUFFIX: &str = "-bin";

/// The number of bytes two candidate files are compared in.
/// A measured launcher is about 5 MiB,
/// and nothing is read at all unless a name the lock declares nowhere is waiting,
/// so the whole file is compared in a handful of blocks.
const COMPARE_BLOCK: usize = 64 * 1024;

//endregion

//region One listing

/// What:
/// The entries of `directory`,
/// sorted by file name,
/// and a warning when it cannot be listed.
/// Why:
/// The order the folder holds is the file system's,
/// and a person who reads two runs compares their sentences,
/// so the pass judges in one order.
fn sorted_entries(directory: &Path, notices: &mut LauncherNotices) -> Vec<fs::DirEntry> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => {
            notices
                .warnings
                .push(format!("left everything in {} alone: the directory cannot be listed: {error}", directory.display()));
            return Vec::new();
        }
    };
    let mut found: Vec<fs::DirEntry> = entries.flatten().collect();
    found.sort_by_key(fs::DirEntry::file_name);
    return found;
}

//endregion

//region The proofs

/// What:
/// Whether the `<name>.launch` record beside a launcher is this tool's own,
/// and the identity of the record the proof read.
/// Why:
/// The record names the tool it was written for,
/// so a record naming a tool the lock has dropped is not proven and the file beside it stays.
/// The identity comes from the open record,
/// because the content and the entry must be the same file for a removal to be safe.
fn record_is_own(record: &Path, declared_tools: &BTreeSet<&str>) -> Result<Judged, String> {
    let mut file: fs::File = match fs::File::open(record) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!(
                "no launcher record {} stands beside it, so mpm-shadow cannot tell it from a file someone put there",
                record.display()
            ));
        }
        Err(error) => return Err(format!("{} cannot be read: {error}", record.display())),
    };
    let judged: Judged = match file.metadata() {
        Ok(metadata) => judged_of_metadata(&metadata),
        Err(error) => return Err(format!("{} cannot be inspected: {error}", record.display())),
    };
    let mut text: String = String::new();
    if let Err(error) = file.read_to_string(&mut text) {
        return Err(format!("{} cannot be read: {error}", record.display()));
    }
    let Some(parsed) = parse_record(&text) else {
        return Err(format!("{} is not a launcher record mpm-shadow wrote", record.display()));
    };
    if !declared_tools.contains(parsed.tool_id.as_str()) {
        return Err(format!(
            "{} names {}, and this lock declares that tool nowhere, so mpm-shadow cannot say the record is its own",
            record.display(),
            parsed.tool_id
        ));
    }
    return Ok(judged);
}

/// The crate a shadow's identity text names,
/// with letter case and `_` versus `-` ignored,
/// so a build of this tool is recognized however its name was spelled.
fn owner_crate(identity: &str) -> String {
    let crate_name: &str = identity.split_once(' ').map(|(name, _version)| return name).unwrap_or(identity);
    return crate_name.to_ascii_lowercase().replace('_', "-");
}

/// What:
/// Whether two files hold the same bytes.
/// Why:
/// A launcher is byte for byte one of `mpm-shadow`'s programs,
/// so the comparison is the whole file,
/// in blocks so no file is held in memory;
/// a file that cannot be read is not one of them.
fn same_bytes(first: &Path, second: &Path) -> bool {
    let (Ok(mut one), Ok(mut two)) = (fs::File::open(first), fs::File::open(second)) else {
        return false;
    };
    let (Ok(one_metadata), Ok(two_metadata)) = (one.metadata(), two.metadata()) else {
        return false;
    };
    if one_metadata.len() != two_metadata.len() {
        return false;
    }
    let mut left: Vec<u8> = vec![0; COMPARE_BLOCK];
    let mut right: Vec<u8> = vec![0; COMPARE_BLOCK];
    loop {
        let (filled_left, filled_right) = match (read_block(&mut one, &mut left), read_block(&mut two, &mut right)) {
            (Ok(left_filled), Ok(right_filled)) => (left_filled, right_filled),
            _ => return false,
        };
        if filled_left == 0 && filled_right == 0 {
            return true;
        }
        if filled_left != filled_right || left[..filled_left] != right[..filled_right] {
            return false;
        }
    }
}

/// Fills `buffer` from `file`,
/// and says how many bytes it holds (short only at the end of the file).
fn read_block(file: &mut fs::File, buffer: &mut [u8]) -> Result<usize, std::io::Error> {
    let mut filled: usize = 0;
    while filled < buffer.len() {
        match file.read(&mut buffer[filled..]) {
            Ok(0) => {
                return Ok(filled);
            }
            Ok(read) => {
                filled += read;
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    return Ok(filled);
}

/// What a proven launcher is:
/// the sentence naming the proof,
/// and the entry the proof was made about.
struct Proven {
    /// Why the file is `mpm-shadow`'s own.
    why: String,
    /// The entry the record and the bytes were read from.
    judged: Judged,
}

/// What:
/// The proofs a pass can bring for a launcher's bytes,
/// gathered once and only when a name the lock declares nowhere is waiting.
/// Why:
/// A launcher is a copy of an `mpm-shadow` program,
/// and the store keeps only the newest version of a repository tool,
/// so the bytes a launcher was placed from can be gone from the store.
/// Three proofs remain,
/// each a fact about the file rather than about its name:
/// the identity text every shadow carries inside its own file,
/// the bytes of the programs at hand (the running one and the newest installed one),
/// and the bytes of another launcher that stands beside a record of its own,
/// because two copies of one installer build are one file whichever folder they were placed in.
struct Proofs<'a> {
    /// The scope whose folders are cleaned.
    scope: &'a Scope,
    /// The environment the machine-global scope is read from.
    env: &'a EnvMap,
    /// The mpm-shadow programs whose bytes are this tool's own.
    images: Option<Vec<PathBuf>>,
    /// The launcher files that stand beside a record of their own, in this scope's launcher folder and the machine's.
    witnesses: Option<Vec<PathBuf>>,
}

/// The methods of `Proofs`.
impl<'a> Proofs<'a> {
    /// A pass with nothing read yet.
    fn new(ctx: &'a Context<'a>) -> Proofs<'a> {
        return Proofs { scope: ctx.scope, env: ctx.env, images: None, witnesses: None };
    }

    /// What:
    /// `Ok(why)` when the file a link names is provably a launcher of this tool's own.
    /// Why:
    /// The link names whatever it points at,
    /// which is this checkout's launcher for a tool this repository publishes,
    /// another checkout's launcher when `.cache` was copied from there,
    /// or the machine-global launcher.
    /// A link whose file another install has already removed is judged by the launcher of the same name in this checkout,
    /// but only when the link said the name was in a launcher folder at all:
    /// a person's broken link to somewhere else shares a name and nothing more.
    fn linked_launcher(&mut self, link: &Path, declared_tools: &BTreeSet<&str>) -> Result<String, String> {
        if let Ok(resolved) = fs::canonicalize(link) {
            return Ok(self.own(&resolved, declared_tools)?.why);
        }
        let stated: PathBuf = match fs::read_link(link) {
            Ok(stated) => stated,
            Err(error) => return Err(format!("the link does not resolve and cannot be read, so what it pointed at is unknown: {error}")),
        };
        let Some(name) = stated.file_name() else {
            return Err(format!("the link does not resolve and names no file name: it stated {}", stated.display()));
        };
        if !self.names_a_launcher_folder(stated.parent()) {
            return Err(format!(
                "the link does not resolve and it named {}, which is in no launcher folder, so nothing says it named a launcher",
                stated.display()
            ));
        }
        let here: PathBuf = self.scope.launcher_dir.join(name);
        if fs::symlink_metadata(&here).is_err() {
            return Err(format!(
                "the link does not resolve, it named {}, and this checkout's launcher folder holds no {} either, so what it pointed at cannot be proven",
                stated.display(),
                name.to_string_lossy()
            ));
        }
        return Ok(self.own(&here, declared_tools)?.why);
    }

    /// What:
    /// Whether `directory` is a folder this tool keeps launchers in:
    /// this checkout's launcher folder,
    /// the machine-global one,
    /// or another checkout's,
    /// which cannot be resolved here because that checkout may be gone.
    /// Why:
    /// The last name alone is not enough:
    /// a folder called `toolchain-launchers` anywhere would pass it,
    /// and then a person's broken link to a folder of that name would be taken for a link mpm-shadow made.
    /// A launcher folder is recognized by the two names it is made of
    /// (`.cache/toolchain-launchers` below a checkout, `toolchain-global/launchers` below the data directory),
    /// and a folder that resolves is compared as a folder with the two this tool knows.
    fn names_a_launcher_folder(&self, given: Option<&Path>) -> bool {
        let Some(folder) = given else {
            return false;
        };
        if same_folder(folder, &self.scope.launcher_dir) {
            return true;
        }
        let Ok(global) = global_scope(self.env) else {
            return names_the_launcher_folder_of(folder, &self.scope.launcher_dir);
        };
        if same_folder(folder, &global.launcher_dir) {
            return true;
        }
        return names_the_launcher_folder_of(folder, &self.scope.launcher_dir) || names_the_launcher_folder_of(folder, &global.launcher_dir);
    }

    /// What:
    /// `Ok(proven)` when `launcher` is provably this tool's own,
    /// and `Err(why not)` when it is not.
    /// Why:
    /// Both proofs the owner asked for are required:
    /// a record this tool can read whose tool this lock still declares,
    /// and bytes that are one of its own programs.
    /// The record comes first,
    /// because it is a few hundred bytes where the bytes are five million.
    fn own(&mut self, launcher: &Path, declared_tools: &BTreeSet<&str>) -> Result<Proven, String> {
        let metadata = match fs::symlink_metadata(launcher) {
            Ok(metadata) => metadata,
            Err(error) => return Err(format!("it cannot be inspected: {error}")),
        };
        let judged: Judged = judged_of_metadata(&metadata);
        if metadata.file_type().is_symlink() {
            return Err("it is a symbolic link, and mpm-shadow places a copy of its own program there, never a link".to_string());
        }
        if !metadata.is_file() {
            return Err("it is not a regular file".to_string());
        }
        record_is_own(&record_path(launcher), declared_tools)?;
        let why: String = self.own_bytes(launcher, declared_tools)?;
        return Ok(Proven { why, judged });
    }

    /// What:
    /// `Ok(why)` when `file`'s own bytes are one of `mpm-shadow`'s own programs.
    /// Why:
    /// The identity text is read from the file itself,
    /// so a launcher placed by any version of this tool is proven by what it holds,
    /// and never by its name,
    /// its path or the date it was written.
    fn own_bytes(&mut self, file: &Path, declared_tools: &BTreeSet<&str>) -> Result<String, String> {
        let opened: fs::File = match fs::File::open(file) {
            Ok(opened) => opened,
            Err(error) => return Err(format!("it cannot be read: {error}")),
        };
        if let Some(identity) = identity_of_open_file(&opened)
            && owner_crate(&identity) == OWN_SHADOW_CRATE
        {
            return Ok(format!("its own bytes carry mpm-shadow's identity ({identity})"));
        }
        let images: Vec<PathBuf> = self.images();
        for image in &images {
            if same_bytes(file, image) {
                return Ok(format!("its bytes are the same as {}", image.display()));
            }
        }
        let witnesses: Vec<PathBuf> = self.witnesses(declared_tools, file);
        for witness in &witnesses {
            if same_bytes(file, witness) {
                return Ok(format!("its bytes are the same as the launcher {}", witness.display()));
            }
        }
        return Err(
            "its own bytes are no mpm-shadow program this tool can name: it carries no mpm-shadow identity, it is not the running \
             program or the installed mpm-shadow, and no launcher beside a record of its own has these bytes, so it may be a file \
             someone put there"
                .to_string(),
        );
    }

    /// The `mpm-shadow` programs whose bytes a launcher may be a copy of:
    /// the newest installed one and the running one.
    fn images(&mut self) -> Vec<PathBuf> {
        if self.images.is_none() {
            let mut images: Vec<PathBuf> = Vec::new();
            if let Ok(running) = running_program() {
                if let Ok(source) = launcher_source(self.scope, running.clone(), cfg!(target_os = "linux")) {
                    images.push(source);
                }
                if !images.contains(&running) {
                    images.push(running);
                }
            }
            self.images = Some(images);
        }
        return match &self.images {
            Some(images) => images.clone(),
            None => Vec::new(),
        };
    }

    /// What:
    /// The launcher files that stand beside a record whose tool this lock still declares,
    /// in this checkout's launcher folder and in the machine-global one,
    /// leaving out `judged` itself.
    /// Why:
    /// A launcher is a file only `mpm-shadow` writes,
    /// so two byte-identical launchers are one installer build,
    /// and one whose record and tool this lock declares is proof for the other.
    /// This is what proves an old copy whose installer version the store no longer keeps:
    /// it is the same bytes as the copy the other scope placed.
    fn witnesses(&mut self, declared_tools: &BTreeSet<&str>, judged: &Path) -> Vec<PathBuf> {
        if self.witnesses.is_none() {
            let mut folders: Vec<PathBuf> = vec![self.scope.launcher_dir.clone()];
            if let Ok(global) = global_scope(self.env) {
                folders.push(global.launcher_dir);
            }
            let mut found: Vec<PathBuf> = Vec::new();
            for folder in folders {
                witness_launchers_in(&folder, declared_tools, &mut found);
            }
            self.witnesses = Some(found);
        }
        let judged_resolved: Option<PathBuf> = fs::canonicalize(judged).ok();
        let mut left: Vec<PathBuf> = Vec::new();
        for witness in self.witnesses.clone().unwrap_or_default() {
            if fs::canonicalize(&witness).ok() == judged_resolved {
                continue;
            }
            left.push(witness);
        }
        return left;
    }
}

/// What:
/// Whether two paths name the same existing folder, once their symbolic links are resolved.
/// Why:
/// A checkout reached through a symbolic link (`/home` to `/var/home` here) writes one spelling into a record and another
/// into a link, and the text comparison would call the same folder two.
fn same_folder(first: &Path, second: &Path) -> bool {
    return match (fs::canonicalize(first), fs::canonicalize(second)) {
        (Ok(one), Ok(two)) => one == two,
        _ => false,
    };
}

/// What:
/// Whether `folder` is shaped like `known`: its last two components are the same.
/// Why:
/// Another checkout's launcher folder is not on this disk to compare as a folder,
/// and the two names `.cache/toolchain-launchers` (or `toolchain-global/launchers`) are what makes one
/// (`scope::repo_scope` and `scope::global_scope` build those two).
fn names_the_launcher_folder_of(folder: &Path, known: &Path) -> bool {
    let mut folder_names: Vec<&std::ffi::OsStr> = folder.components().rev().take(2).map(|component| return component.as_os_str()).collect();
    let mut known_names: Vec<&std::ffi::OsStr> = known.components().rev().take(2).map(|component| return component.as_os_str()).collect();
    folder_names.reverse();
    known_names.reverse();
    return folder_names.len() == 2 && folder_names == known_names;
}

/// The launchers of `folder` that stand beside a record whose tool is in `declared_tools`.
fn witness_launchers_in(folder: &Path, declared_tools: &BTreeSet<&str>, witnesses: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(RECORD_SUFFIX) || name.starts_with('.') {
            continue;
        }
        let path: PathBuf = entry.path();
        if !fs::symlink_metadata(&path).is_ok_and(|metadata| return metadata.is_file()) {
            continue;
        }
        if record_is_own(&record_path(&path), declared_tools).is_ok() {
            witnesses.push(path);
        }
    }
}

//endregion

//region The links mpm-shadow manages

/// What:
/// Every directory in which this scope's `mpm-shadow` keeps links:
/// the farm,
/// every `-bin` directory under the checkout's `.cache`,
/// the scope's own link directory,
/// and the directories the lock's postinstall links declare.
/// Why:
/// The record of postinstall links can be missing or unreadable
/// (a run killed after it made the links and before it wrote the record,
/// a damaged file),
/// and then nothing names the links it made;
/// a launcher whose link is invisible would be removed from under it.
/// The `-bin` suffix is the convention the cleanup of old copies already reads a checkout's link folders by.
fn link_folders(ctx: &Context) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = vec![ctx.scope.exposure_dir.clone()];
    let cache: PathBuf = ctx.scope.root.join(".cache");
    if let Ok(entries) = fs::read_dir(&cache) {
        let mut found: Vec<PathBuf> = Vec::new();
        for entry in entries.flatten() {
            let name: String = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(LINK_DIRECTORY_SUFFIX) && entry.path().is_dir() {
                found.push(entry.path());
            }
        }
        found.sort();
        folders.extend(found);
    }
    folders.push(ctx.scope.link_dir.clone());
    for tool in ctx.tools {
        if let Some(Postinstall::Link(link)) = &tool.declaration.options.postinstall {
            folders.push(link_directory(ctx.scope, &link.link_dir));
        }
    }
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    let mut distinct: Vec<PathBuf> = Vec::new();
    for folder in folders {
        if seen.insert(folder.clone()) {
            distinct.push(folder);
        }
    }
    return distinct;
}

/// What:
/// The link paths `mpm-shadow` manages in this scope:
/// the entries of every folder it keeps links in,
/// and the paths the record of postinstall links names.
/// Why:
/// A hidden name is skipped:
/// a link of a tool's bin is never hidden,
/// and a hidden entry is a write of a killed run
/// (`atomic_symlink`'s `.<name>.<pid>.tmp` and this pass's `.<name>.<pid>.<n>.retired`),
/// which would otherwise keep a launcher alive for ever because it still resolved to it.
fn managed_link_paths(ctx: &Context) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for folder in link_folders(ctx) {
        if let Ok(entries) = fs::read_dir(&folder) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().starts_with('.') {
                    continue;
                }
                paths.push(entry.path());
            }
        }
    }
    paths.extend(recorded_link_paths(ctx));
    return paths;
}

/// What:
/// Every link `mpm-shadow` manages that resolves to `launcher`.
/// Why:
/// A launcher is removed only once nothing points at it,
/// and the postinstall link of a dropped name can outlive the lock that declared it.
fn links_reaching(ctx: &Context, launcher: &Path) -> Vec<PathBuf> {
    let Ok(target) = fs::canonicalize(launcher) else {
        return Vec::new();
    };
    let mut reaching: Vec<PathBuf> = Vec::new();
    for link in managed_link_paths(ctx) {
        if let Ok(resolved) = fs::canonicalize(&link)
            && resolved == target
        {
            reaching.push(link);
        }
    }
    return reaching;
}

//endregion

//region The three passes

/// What:
/// Removes one link of a name the lock declares nowhere,
/// when what it names is provably a launcher of this tool's own.
/// Why:
/// The proof is what the link points at, never the link's own name:
/// the same rule and the same sentences serve the farm and the folders a tool's postinstall step links into.
fn retire_one_link(
    reason: &str,
    path: &Path,
    judged: Judged,
    declared_tools: &BTreeSet<&str>,
    proofs: &mut Proofs<'_>,
    notices: &mut LauncherNotices,
    report_unproven: bool,
) {
    let why: String = match proofs.linked_launcher(path, declared_tools) {
        Ok(why) => why,
        Err(why) => {
            // A link nothing proves in a folder the lock does not declare is what a tool's own postinstall command keeps
            // there (the search shadow links `rg` and eight more), so the person reading the report is told only about
            // the ones they can act on.
            if report_unproven {
                notices.warnings.push(format!("left {} alone: {reason}, but {why}", path.display()));
            }
            return;
        }
    };
    match remove_if_still(path, judged) {
        Ok(Removal::Retired) => notices.retired.push(format!("retired the link {} because {reason} ({why})", path.display())),
        Ok(Removal::Gone) => {}
        Ok(Removal::Replaced) => notices.warnings.push(format!(
            "left {} alone: {reason}, but something replaced the link between deciding that and removing it, so it was put back",
            path.display()
        )),
        Err(reason_it_could_not) => notices.warnings.push(format!(
            "left {} alone: {reason}, and the link names a launcher mpm-shadow made, but {reason_it_could_not}",
            path.display()
        )),
    }
}

/// What:
/// Retires every link of the farm whose name the lock declares nowhere and whose target is a launcher this tool can prove its own.
/// Why:
/// The farm is `mpm-shadow`'s own folder:
/// it links a name to a real program for a tool it does not publish,
/// and to a launcher for one it does.
/// Only the second kind is provable,
/// so a link to a program is left and named
/// (a person may have made it,
/// and the lock says nothing about it).
fn retire_farm_links(
    entries: &[fs::DirEntry],
    declared: &BTreeSet<String>,
    declared_paths: &BTreeSet<PathBuf>,
    declared_tools: &BTreeSet<&str>,
    proofs: &mut Proofs<'_>,
    notices: &mut LauncherNotices,
) {
    for entry in entries {
        let name: String = entry.file_name().to_string_lossy().to_string();
        let path: PathBuf = entry.path();
        // A postinstall link may be declared inside the farm (`linkDir` names it), and then the path, not the bin name,
        // is what makes it the lock's own.
        if declared.contains(&name) || declared_paths.contains(&path) {
            continue;
        }
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                notices.warnings.push(format!("left {} alone: it cannot be inspected: {error}", path.display()));
                continue;
            }
        };
        if !metadata.file_type().is_symlink() {
            // A hidden name that is not a link is the record of the postinstall links or a write of a killed run, and it is
            // left silently; any other name is a check of what stands in the farm, which mpm-shadow never made.
            if name.starts_with('.') {
                continue;
            }
            notices.warnings.push(format!(
                "left {} alone: the lock declares no bin of that name, and this is not a symbolic link mpm-shadow made; \
                 remove it yourself if it is stale",
                path.display()
            ));
            continue;
        }
        retire_one_link(
            "the lock declares no bin of that name",
            &path,
            judged_of_metadata(&metadata),
            declared_tools,
            proofs,
            notices,
            true,
        );
    }
}

/// What:
/// Retires every link in the folders a tool's postinstall step links into whose path the lock declares nowhere and whose target is a launcher this tool can prove its own.
/// Why:
/// A killed run can make these links and never record them,
/// and a damaged record names none of them,
/// so the record cannot be the only answer to "which links did mpm-shadow make".
/// A link here that names a person's program is left and named, as in the farm;
/// anything but a symbolic link is a person's file and is not judged at all.
fn retire_other_links(
    ctx: &Context,
    declared_paths: &BTreeSet<PathBuf>,
    declared_tools: &BTreeSet<&str>,
    proofs: &mut Proofs<'_>,
    notices: &mut LauncherNotices,
) {
    let mut declared_folders: BTreeSet<PathBuf> = BTreeSet::new();
    for path in declared_paths {
        if let Some(parent) = path.parent() {
            declared_folders.insert(parent.to_path_buf());
        }
    }
    for folder in link_folders(ctx) {
        if folder == ctx.scope.exposure_dir {
            continue;
        }
        // Only the scope's own link directory and the folders the lock declares a link in are places a person is told
        // about; the other `-bin` folders belong to the tools' own postinstall commands.
        let report_unproven: bool = folder == ctx.scope.link_dir || declared_folders.contains(&folder);
        for entry in sorted_entries(&folder, notices) {
            let path: PathBuf = entry.path();
            if declared_paths.contains(&path) {
                continue;
            }
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => {
                    notices.warnings.push(format!("left {} alone: it cannot be inspected: {error}", path.display()));
                    continue;
                }
            };
            if !metadata.file_type().is_symlink() {
                continue;
            }
            // A hidden link here is the leftover of a killed atomic write (`atomic_symlink`'s `.<name>.<pid>.tmp` or this pass's
            // own `.<name>.<pid>.<n>.retired`); its target proves it, and leaving it would leave a link that later resolves to
            // nothing.
            retire_one_link(
                "the lock declares no postinstall link there",
                &path,
                judged_of_metadata(&metadata),
                declared_tools,
                proofs,
                notices,
                report_unproven,
            );
        }
    }
}

/// What:
/// Retires every launcher in this checkout's launcher folder whose name the lock declares nowhere and whose record and bytes prove it is this tool's own,
/// once nothing `mpm-shadow` made points at it any more,
/// and then its record.
/// Why:
/// The launcher goes after the links that name it,
/// never before,
/// so a link never points at a launcher that is gone;
/// a link that could not be removed keeps its launcher and is named in a warning.
fn retire_launchers(
    ctx: &Context,
    entries: &[fs::DirEntry],
    declared: &BTreeSet<String>,
    declared_tools: &BTreeSet<&str>,
    proofs: &mut Proofs<'_>,
    notices: &mut LauncherNotices,
) {
    for entry in entries {
        let name: String = entry.file_name().to_string_lossy().to_string();
        // A record is judged with the launcher it stands beside, and a hidden name is the record of the postinstall links or a
        // write of a killed run, never a tool's launcher.
        if name.ends_with(RECORD_SUFFIX) || name.starts_with('.') || declared.contains(&name) {
            continue;
        }
        let launcher: PathBuf = entry.path();
        let proven: Proven = match proofs.own(&launcher, declared_tools) {
            Ok(proven) => proven,
            Err(why) => {
                notices
                    .warnings
                    .push(format!("left {} alone: the lock declares no bin of that name, but {why}", launcher.display()));
                continue;
            }
        };
        let reaching: Vec<PathBuf> = links_reaching(ctx, &launcher);
        if !reaching.is_empty() {
            notices.warnings.push(format!(
                "left {} alone: its name is nowhere in this lock and its bytes are mpm-shadow's own, but {} still points at it",
                launcher.display(),
                reaching.iter().map(|link| return link.display().to_string()).collect::<Vec<String>>().join(", ")
            ));
            continue;
        }
        match remove_if_still(&launcher, proven.judged) {
            Ok(Removal::Retired) => notices.retired.push(format!(
                "retired the launcher {} because the lock declares no bin of that name ({})",
                launcher.display(),
                proven.why
            )),
            Ok(Removal::Gone) => {}
            Ok(Removal::Replaced) => {
                notices.warnings.push(format!(
                    "left {} alone: something replaced it between proving it mpm-shadow's own and removing it, so it was put back",
                    launcher.display()
                ));
                continue;
            }
            Err(reason) => {
                notices
                    .warnings
                    .push(format!("left {} alone: its name is nowhere in this lock, but {reason}", launcher.display()));
                continue;
            }
        }
        let record: PathBuf = record_path(&launcher);
        let judged: Judged = match record_is_own(&record, declared_tools) {
            Ok(judged) => judged,
            Err(why) => {
                notices
                    .warnings
                    .push(format!("left {} alone: the launcher it belonged to is gone, but {why}", record.display()));
                continue;
            }
        };
        match remove_if_still(&record, judged) {
            Ok(Removal::Retired) => notices
                .retired
                .push(format!("retired the launcher record {} because the lock declares no bin of that name", record.display())),
            Ok(Removal::Gone) => {}
            Ok(Removal::Replaced) => notices.warnings.push(format!(
                "left {} alone: something replaced the record between reading it and removing it, so it was put back",
                record.display()
            )),
            Err(reason) => notices.warnings.push(format!(
                "left {} alone: the launcher it belonged to is gone and its name is nowhere in this lock, but {reason}",
                record.display()
            )),
        }
    }
}

/// What:
/// Retires the record of a name the lock declares nowhere when its launcher is already gone.
/// Why:
/// A run killed between removing a launcher and removing its record leaves that record behind,
/// and the next install reads it as a record naming another lock,
/// so that state must not survive an install.
/// Only the record itself is proven here:
/// a record this tool can read whose tool this lock still declares is `mpm-shadow`'s own writing,
/// and the caller holds the scope's lock,
/// so a record an install beside this one has just written is always accompanied by its launcher by now.
fn retire_records_without_launchers(
    entries: &[fs::DirEntry],
    declared: &BTreeSet<String>,
    declared_tools: &BTreeSet<&str>,
    notices: &mut LauncherNotices,
) {
    for entry in entries {
        let record_name: String = entry.file_name().to_string_lossy().to_string();
        let Some(launcher_name) = record_name.strip_suffix(RECORD_SUFFIX) else {
            continue;
        };
        if launcher_name.starts_with('.') || declared.contains(launcher_name) {
            continue;
        }
        let record: PathBuf = entry.path();
        if fs::symlink_metadata(&record).is_err() {
            // The pass above removed it with its launcher.
            continue;
        }
        let launcher: PathBuf = record.with_file_name(launcher_name);
        if fs::symlink_metadata(&launcher).is_ok() {
            // Its launcher stands beside it, so the pass above judged the pair.
            continue;
        }
        let judged: Judged = match record_is_own(&record, declared_tools) {
            Ok(judged) => judged,
            Err(why) => {
                notices
                    .warnings
                    .push(format!("left {} alone: the lock declares no bin of that name, but {why}", record.display()));
                continue;
            }
        };
        match remove_if_still(&record, judged) {
            Ok(Removal::Retired) => notices
                .retired
                .push(format!("retired the launcher record {} because the lock declares no bin of that name", record.display())),
            Ok(Removal::Gone) => {}
            Ok(Removal::Replaced) => notices.warnings.push(format!(
                "left {} alone: something replaced the record between reading it and removing it, so it was put back",
                record.display()
            )),
            Err(reason) => notices
                .warnings
                .push(format!("left {} alone: the lock declares no bin of that name, but {reason}", record.display())),
        }
    }
}

//endregion

//region The declared names and the pass

/// The names this lock declares a launcher for,
/// as `rebuild_exposure` and the postinstall links ask for one:
/// every bin of a repository-published tool,
/// and the source bin of every postinstall link such a tool declares.
fn declared_launcher_names(ctx: &Context) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for tool in ctx.tools {
        if !is_repository_published(tool) {
            continue;
        }
        for bin in &tool.declaration.bins {
            names.insert(bin.clone());
        }
        if let Some(Postinstall::Link(link)) = &tool.declaration.options.postinstall {
            names.insert(link.source_bin.clone());
        }
    }
    return names;
}

/// What:
/// The names this lock declares an exposed link for:
/// every declared bin,
/// the names this run just linked,
/// and the running `mpm-shadow` under its own name when an installed copy of it exists.
/// Why:
/// The farm links the running shadow too when no plan row declares that bin,
/// and that name is as much the lock's arrangement as a declared bin is.
fn declared_link_names(ctx: &Context, made_links: &[String]) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for tool in ctx.tools {
        for bin in &tool.declaration.bins {
            names.insert(bin.clone());
        }
    }
    for name in made_links {
        names.insert(name.clone());
    }
    if let Ok(running) = running_program()
        && let Some(installed) = newest_installed_shadow(ctx.scope, &running)
        && let Some(name) = program_name(&installed)
    {
        names.insert(name.to_string_lossy().to_string());
    }
    return names;
}

/// What:
/// What a pass of the retirement did:
/// one sentence per file removed,
/// and one warning per file left where it was.
/// Why:
/// The installer prints both,
/// so a person whose file was kept sees which file and why,
/// and a pass that removed nothing says nothing.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LauncherNotices {
    /// One sentence per link, launcher or record removed.
    pub retired: Vec<String>,
    /// One sentence per file left in place, naming what could not be proven.
    pub warnings: Vec<String>,
}

/// What:
/// Retires the launchers, records and links of names this lock declares nowhere.
/// Why:
/// This is the one place `install` cleans up after a bin the plan renamed or dropped.
/// `made_links` are the exposed names this run itself linked,
/// so a name the running `mpm-shadow` owns is never taken for a leftover.
/// The caller holds the scope's lock over this pass,
/// which is what makes "a record with no launcher is a leftover" true while another install runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function retireUndeclaredLaunchers(ctx: Context, madeLinks: string[]): LauncherNotices
/// ```
pub fn retire_undeclared_launchers(ctx: &Context, made_links: &[String]) -> LauncherNotices {
    let mut notices: LauncherNotices = LauncherNotices::default();
    // One listing per folder: an install with nothing to do compares names against the lock and reads no file.
    let launchers: Vec<fs::DirEntry> = sorted_entries(&ctx.scope.launcher_dir, &mut notices);
    let farm: Vec<fs::DirEntry> = sorted_entries(&ctx.scope.exposure_dir, &mut notices);
    if launchers.is_empty() && farm.is_empty() {
        return notices;
    }
    let declared_launchers: BTreeSet<String> = declared_launcher_names(ctx);
    let declared_links: BTreeSet<String> = declared_link_names(ctx, made_links);
    let declared_paths: BTreeSet<PathBuf> = declared_link_paths(ctx);
    let declared_tools: BTreeSet<&str> = ctx.tools.iter().map(|tool| return tool.declaration.id.as_str()).collect();
    let mut proofs: Proofs<'_> = Proofs::new(ctx);
    // The links go before the launchers they point at, so no link is ever left naming a file that is gone.
    retire_farm_links(&farm, &declared_links, &declared_paths, &declared_tools, &mut proofs, &mut notices);
    retire_other_links(ctx, &declared_paths, &declared_tools, &mut proofs, &mut notices);
    retire_launchers(ctx, &launchers, &declared_launchers, &declared_tools, &mut proofs, &mut notices);
    retire_records_without_launchers(&launchers, &declared_launchers, &declared_tools, &mut notices);
    return notices;
}

//endregion
