//! What:
//! The retirement of what `install` once made for a name the lock declares nowhere:
//! a launcher in the checkout's launcher folder,
//! the `<name>.launch` record beside it,
//! and the exposed link in the checkout's link folder.
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
//! A file whose own bytes are not provably `mpm-shadow`'s stays where it is,
//! and the install names it and why.
//! An install that cannot read the lock never reaches here at all,
//! and the removal order keeps a link from ever pointing at a launcher that is gone:
//! the exposed links go first,
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
/// Imports the read trait the block comparison fills its buffers with.
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
/// Imports `recorded_link_paths` from `super::postinstall_links`.
use super::postinstall_links::recorded_link_paths;
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope, global_scope};

//region The crate mpm-shadow places, and how much of a file is read at a time

/// The crate name this tool's own launchers carry in their identity text.
/// A launcher is a copy of the `mpm-shadow` program,
/// never of one of the repository's other shadows,
/// whose identity text names them instead.
const OWN_SHADOW_CRATE: &str = "monochromatic-mpm-shadow";

/// The number of bytes two candidate files are compared in.
/// A measured launcher is about 5 MiB,
/// and nothing is read at all unless a name the lock declares nowhere is waiting,
/// so the whole file is compared in a handful of blocks.
const COMPARE_BLOCK: usize = 64 * 1024;

//endregion

//region One listing, one removal

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

/// What:
/// Removes `path` and says whether it is gone:
/// `Ok(true)` when this call removed it,
/// `Ok(false)` when nothing was there,
/// and the reason when it could not be removed.
/// Why:
/// Two installs run in one checkout at the same time,
/// so a file the other one already retired is not an error.
fn remove_if_present(path: &Path) -> Result<bool, String> {
    return match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("it cannot be removed: {error}")),
    };
}

//endregion

//region The proofs

/// What:
/// Whether the `<name>.launch` record beside a launcher is this tool's own:
/// a record `mpm-shadow` can read, whose tool this lock still declares.
/// Why:
/// The record names the tool it was written for,
/// so a record naming a tool the lock has dropped is not proven and the file beside it stays.
fn record_is_own(record: &Path, declared_tools: &BTreeSet<&str>) -> Result<(), String> {
    let text: String = match fs::read_to_string(record) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!(
                "no launcher record {} stands beside it, so mpm-shadow cannot tell it from a file someone put there",
                record.display()
            ));
        }
        Err(error) => return Err(format!("{} cannot be read: {error}", record.display())),
    };
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
    return Ok(());
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
    /// `Ok(why)` when the file the exposed link at `link` names is provably a launcher of this tool's own.
    /// Why:
    /// The link names whatever it points at,
    /// which is this checkout's launcher for a tool this repository publishes,
    /// another checkout's launcher when `.cache` was copied from there,
    /// or the machine-global launcher.
    /// A link whose file another install has already removed is judged by the launcher of the same name in this checkout:
    /// that is what it pointed at before the copy, and the file itself must still prove it.
    fn linked_launcher(&mut self, link: &Path, declared_tools: &BTreeSet<&str>) -> Result<String, String> {
        if let Ok(resolved) = fs::canonicalize(link) {
            return self.own(&resolved, declared_tools);
        }
        let stated: PathBuf = match fs::read_link(link) {
            Ok(stated) => stated,
            Err(error) => return Err(format!("the link does not resolve and cannot be read, so what it pointed at is unknown: {error}")),
        };
        let Some(name) = stated.file_name() else {
            return Err(format!("the link does not resolve and names no file name: it stated {}", stated.display()));
        };
        let here: PathBuf = self.scope.launcher_dir.join(name);
        if fs::symlink_metadata(&here).is_err() {
            return Err(format!(
                "the link does not resolve and this checkout's launcher folder holds no {} either, so what it pointed at cannot be proven",
                name.to_string_lossy()
            ));
        }
        return self.own(&here, declared_tools);
    }

    /// What:
    /// `Ok(why)` when `launcher` is provably this tool's own,
    /// and `Err(why not)` when it is not.
    /// Why:
    /// Both proofs the owner asked for are required:
    /// a record this tool can read whose tool this lock still declares,
    /// and bytes that are one of its own programs.
    /// The record comes first,
    /// because it is a few hundred bytes where the bytes are five million.
    fn own(&mut self, launcher: &Path, declared_tools: &BTreeSet<&str>) -> Result<String, String> {
        let metadata = match fs::symlink_metadata(launcher) {
            Ok(metadata) => metadata,
            Err(error) => return Err(format!("it cannot be inspected: {error}")),
        };
        if metadata.file_type().is_symlink() {
            return Err("it is a symbolic link, and mpm-shadow places a copy of its own program there, never a link".to_string());
        }
        if !metadata.is_file() {
            return Err("it is not a regular file".to_string());
        }
        record_is_own(&record_path(launcher), declared_tools)?;
        return self.own_bytes(launcher, declared_tools);
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

/// The link paths `mpm-shadow` writes and remembers in this scope:
/// the entries of the farm,
/// the postinstall links the lock declares,
/// and the postinstall links the record names.
fn managed_link_paths(ctx: &Context) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&ctx.scope.exposure_dir) {
        for entry in entries.flatten() {
            paths.push(entry.path());
        }
    }
    for tool in ctx.tools {
        if let Some(Postinstall::Link(link)) = &tool.declaration.options.postinstall {
            paths.push(link_directory(ctx.scope, &link.link_dir).join(&link.link_name));
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
/// Retires every exposed link whose name the lock declares nowhere and whose target is a launcher this tool can prove its own.
/// Why:
/// The farm is `mpm-shadow`'s own folder:
/// it links a name to a real program for a tool it does not publish,
/// and to a launcher for one it does.
/// Only the second kind is provable,
/// so a link to a program is left and named
/// (a person may have made it,
/// and the lock says nothing about it).
/// A copied `.cache` makes its links name the checkout it was copied from,
/// and that checkout's own install may have removed the launcher since,
/// so a link that no longer resolves is proven by the launcher of the same name in this checkout.
fn retire_exposed_links(
    entries: &[fs::DirEntry],
    declared: &BTreeSet<String>,
    declared_tools: &BTreeSet<&str>,
    proofs: &mut Proofs<'_>,
    notices: &mut LauncherNotices,
) {
    for entry in entries {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if declared.contains(&name) || name.starts_with('.') {
            continue;
        }
        let path: PathBuf = entry.path();
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                notices.warnings.push(format!("left {} alone: it cannot be inspected: {error}", path.display()));
                continue;
            }
        };
        if !metadata.file_type().is_symlink() {
            notices.warnings.push(format!(
                "left {} alone: the lock declares no bin of that name, and this is not a symbolic link mpm-shadow made; \
                 remove it yourself if it is stale",
                path.display()
            ));
            continue;
        }
        match proofs.linked_launcher(&path, declared_tools) {
            Err(why) => notices
                .warnings
                .push(format!("left {} alone: the lock declares no bin of that name, but {why}", path.display())),
            Ok(why) => match remove_if_present(&path) {
                Ok(true) => notices.retired.push(format!(
                    "retired the exposed link {} because the lock declares no bin of that name ({why})",
                    path.display()
                )),
                Ok(false) => {}
                Err(reason) => notices.warnings.push(format!(
                    "left {} alone: the lock declares no bin of that name and the link names a launcher mpm-shadow made, but {reason}",
                    path.display()
                )),
            },
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
        let why: String = match proofs.own(&launcher, declared_tools) {
            Ok(why) => why,
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
        match remove_if_present(&launcher) {
            Ok(true) => notices.retired.push(format!(
                "retired the launcher {} because the lock declares no bin of that name ({why})",
                launcher.display()
            )),
            Ok(false) => {}
            Err(reason) => {
                notices.warnings.push(format!("left {} alone: its name is nowhere in this lock, but {reason}", launcher.display()));
                continue;
            }
        }
        let record: PathBuf = record_path(&launcher);
        match remove_if_present(&record) {
            Ok(true) => notices
                .retired
                .push(format!("retired the launcher record {} because the lock declares no bin of that name", record.display())),
            Ok(false) => {}
            Err(reason) => notices.warnings.push(format!(
                "left {} alone: the launcher it belongs to is gone and its name is nowhere in this lock, but {reason}",
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
/// a record this tool can read whose tool this lock still declares is `mpm-shadow`'s own writing.
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
        if let Err(why) = record_is_own(&record, declared_tools) {
            notices
                .warnings
                .push(format!("left {} alone: the lock declares no bin of that name, but {why}", record.display()));
            continue;
        }
        match remove_if_present(&record) {
            Ok(true) => notices
                .retired
                .push(format!("retired the launcher record {} because the lock declares no bin of that name", record.display())),
            Ok(false) => {}
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
/// Retires the launchers, records and exposed links of names this lock declares nowhere.
/// Why:
/// This is the one place `install` cleans up after a bin the plan renamed or dropped.
/// `made_links` are the exposed names this run itself linked,
/// so a name the running `mpm-shadow` owns is never taken for a leftover.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function retireUndeclaredLaunchers(ctx: Context, madeLinks: string[]): LauncherNotices
/// ```
pub fn retire_undeclared_launchers(ctx: &Context, made_links: &[String]) -> LauncherNotices {
    let mut notices: LauncherNotices = LauncherNotices::default();
    // One listing per folder: an install with nothing to do compares names against the lock and reads no file.
    let launchers: Vec<fs::DirEntry> = sorted_entries(&ctx.scope.launcher_dir, &mut notices);
    let links: Vec<fs::DirEntry> = sorted_entries(&ctx.scope.exposure_dir, &mut notices);
    if launchers.is_empty() && links.is_empty() {
        return notices;
    }
    let declared_launchers: BTreeSet<String> = declared_launcher_names(ctx);
    let declared_links: BTreeSet<String> = declared_link_names(ctx, made_links);
    let declared_tools: BTreeSet<&str> = ctx.tools.iter().map(|tool| return tool.declaration.id.as_str()).collect();
    let mut proofs: Proofs<'_> = Proofs::new(ctx);
    // The links go before the launchers they point at, so no link is ever left naming a file that is gone.
    retire_exposed_links(&links, &declared_links, &declared_tools, &mut proofs, &mut notices);
    retire_launchers(ctx, &launchers, &declared_launchers, &declared_tools, &mut proofs, &mut notices);
    retire_records_without_launchers(&launchers, &declared_launchers, &declared_tools, &mut notices);
    return notices;
}

//endregion
