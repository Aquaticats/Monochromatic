//! What:
//!  Read the `uses/` folder of one source folder and check each entry on its own.
//! Why:
//!  The rule wants every entry to be a symbolic link to a source folder of the same package,
//!  named after the folder it points to.
//!  Whether a link is needed is a question about the code and is answered by the caller;
//!  this module answers what each entry is.
//!  Links are read with `symlink_metadata`, `read_link` and `canonicalize` on the link itself,
//!  and never followed to list files.
//!  A folder with no `uses` folder links to nothing:
//!  Git cannot store an empty folder, so a folder that uses nothing has none.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Link = { entry: Path; target: Path };
//! type LinkRead = { links: Link[]; findings: LayoutFinding[] };
//! function readLinks(folder: Path, sourceFolders: Map<Path, Path>, base: Path): LinkRead;
//! ```

/// Import the finding shape,
///  the path display helper and the relative path helper.
use crate::{
    layout_finding::{LayoutFinding, shown},
    markdown_lfs_target::lexical_normal,
    run_paths::relative_from,
};
/// Import ordered maps and native paths.
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// The name of the folder that holds the links.
pub const USES_FOLDER: &str = "uses";

/// What:
///  A good entry:
///  a link and the source folder it points to.
/// Why:
///  The caller compares these with the folders the code uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Link = { entry: Path; target: Path };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Link {
    /// The link itself.
    pub entry: PathBuf,
    /// The source folder it points to.
    pub target: PathBuf,
}

/// What:
///  The entries of one `uses` folder,
///  sorted into good links and findings.
/// Why:
///  A bad entry is reported here and is not counted as a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkRead = { links: Link[]; findings: LayoutFinding[] };
/// ```
#[derive(Clone, Debug, Default)]
pub struct LinkRead {
    /// Links that point to a source folder of the package.
    pub links: Vec<Link>,
    /// Problems with the other entries.
    pub findings: Vec<LayoutFinding>,
}

/// What:
///  Make a finding about an entry,
///  with the command that writes a correct link.
/// Why:
///  Every finding says what to do next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryFinding(entry: Path, message: string): LayoutFinding;
/// ```
fn entry_finding(entry: &Path, message: String) -> LayoutFinding {
    return LayoutFinding {
        file: entry.to_path_buf(),
        line: 0,
        message,
        help: None,
    };
}

/// What:
///  The command that creates the link a folder needs.
/// Why:
///  The target is written relative to the `uses` folder,
///  so the link keeps working when the checkout moves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linkCommand(folder: Path, target: Path): string;
/// ```
pub fn link_command(folder: &Path, target: &Path, base: &Path) -> String {
    let uses: PathBuf = folder.join(USES_FOLDER);
    let name: String = match target.file_name() {
        Some(found) => found.to_string_lossy().into_owned(),
        None => String::from("folder"),
    };
    let relative: PathBuf = relative_from(&lexical_normal(&uses), &lexical_normal(target));
    return format!(
        "mkdir --parents {uses} && ln --symbolic {relative} {entry}",
        uses = shown(&uses, base),
        relative = relative.display(),
        entry = shown(&uses.join(name), base),
    );
}

/// What:
///  Check one entry of a `uses` folder.
/// Why:
///  An entry is good only when it is a link whose target exists,
///  is a folder, is a source folder of this package, is not the folder itself,
///  and carries the target folder's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkEntry(entry: Path, folder: Path, sourceFolders: Map<Path, Path>, base: Path): Link | LayoutFinding;
/// ```
fn check_entry(
    entry: &Path,
    folder: &Path,
    source_folders: &BTreeMap<PathBuf, PathBuf>,
    base: &Path,
) -> Result<Link, LayoutFinding> {
    let shown_entry: String = shown(entry, base);
    let Ok(kind) = std::fs::symlink_metadata(entry) else {
        return Err(entry_finding(
            entry,
            format!("{shown_entry} cannot be read."),
        ));
    };
    if !kind.file_type().is_symlink() {
        return Err(entry_finding(
            entry,
            format!("{shown_entry} is not a symbolic link; a uses folder holds only links to the folders the code uses."),
        ));
    }
    let written: String = match std::fs::read_link(entry) {
        Ok(text) => text.display().to_string(),
        Err(_) => String::from("an unreadable target"),
    };
    let Ok(canonical) = std::fs::canonicalize(entry) else {
        return Err(entry_finding(
            entry,
            format!("{shown_entry} is a broken link: it points to {written}, which does not exist."),
        ));
    };
    if !canonical.is_dir() {
        return Err(entry_finding(
            entry,
            format!("{shown_entry} points to {written}, which is a file; a link must point to a folder."),
        ));
    }
    let Some(target) = source_folders.get(&canonical) else {
        return Err(entry_finding(
            entry,
            format!("{shown_entry} points to {written}, which is not a source folder of this package (a folder holding a module file of its production code)."),
        ));
    };
    if target == folder {
        return Err(entry_finding(
            entry,
            format!("{shown_entry} points to the folder it is in; a folder does not link to itself."),
        ));
    }
    let wanted: String = match target.file_name() {
        Some(found) => found.to_string_lossy().into_owned(),
        None => String::new(),
    };
    let named: String = match entry.file_name() {
        Some(found) => found.to_string_lossy().into_owned(),
        None => String::new(),
    };
    if wanted != named {
        return Err(entry_finding(
            entry,
            format!("{shown_entry} points to the folder {wanted}, so the link must be named {wanted}."),
        ));
    }
    return Ok(Link {
        entry: entry.to_path_buf(),
        target: target.clone(),
    });
}

/// What:
///  Read every entry of the `uses` folder of one source folder.
/// Why:
///  `uses` must itself be a real folder and not a link,
///  or the walk below it could leave the package.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readLinks(folder: Path, sourceFolders: Map<Path, Path>, base: Path): LinkRead;
/// ```
pub fn read_links(
    folder: &Path,
    source_folders: &BTreeMap<PathBuf, PathBuf>,
    base: &Path,
) -> LinkRead {
    let mut read: LinkRead = LinkRead::default();
    let uses: PathBuf = folder.join(USES_FOLDER);
    let kind: std::fs::Metadata = match std::fs::symlink_metadata(&uses) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return read,
        Err(error) => {
            read.findings.push(entry_finding(
                &uses,
                format!("{} cannot be read: {error}.", shown(&uses, base)),
            ));
            return read;
        }
    };
    if !kind.is_dir() {
        read.findings.push(entry_finding(
            &uses,
            format!("{} must be a folder of links, not a file or a link.", shown(&uses, base)),
        ));
        return read;
    }
    let Ok(listing) = std::fs::read_dir(&uses) else {
        read.findings.push(entry_finding(
            &uses,
            format!("{} cannot be listed.", shown(&uses, base)),
        ));
        return read;
    };
    let mut entries: Vec<PathBuf> = listing
        .flatten()
        .map(|found| return found.path())
        .collect::<Vec<PathBuf>>();
    entries.sort();
    for entry in entries {
        match check_entry(&entry, folder, source_folders, base) {
            Ok(link) => read.links.push(link),
            Err(finding) => read.findings.push(finding),
        }
    }
    return read;
}

/// Controls of link entry reading stay outside release artifacts.
#[cfg(test)]
#[path = "layout_link_entries_tests.rs"]
mod tests;
