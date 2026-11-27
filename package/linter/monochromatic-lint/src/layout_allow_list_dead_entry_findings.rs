//! What:
//!  The findings for the entries of a list that tolerated nothing.
//! Why:
//!  A dead line is the sign the list has not shrunk after the work that made it dead,
//!  and the run must say so on the line itself.
//!  A `uses` or `file-uses` line that names a folder no source folder of the package is called
//!  says which folder it read and that no source folder has that name:
//!  that is what a hand-aligned or mistyped line looks like,
//!  and it is the only thing that tells its reader to re-space the line rather than delete it.
//!  The sentence speaks of source folders rather than of folders,
//!  because the set it is judged against is the module tree's:
//!  a folder on disk that holds no module file (a `docs` folder holding a `README.md`)
//!  is no source folder, and telling its reader that the package holds no such folder would be false.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function deadEntryFindings(list: AllowList, used: Set<number>, folders: Set<Path>, base: Path): LayoutFinding[];
//! ```

/// Import the list and the entry shapes it holds,
///  the finding shape and the spelling of the names in it.
use crate::{
    layout_allow_list::{
        AllowKind,
        AllowList,
    },
    layout_finding::{
        LayoutFinding,
        key_path,
        shown,
        spelled_whitespace,
    },
};
/// Import ordered collections and native paths.
use std::{
    collections::BTreeSet,
    path::{
        Path,
        PathBuf,
    },
};

/// What:
///  The findings for the entries that tolerated nothing.
/// Why:
///  A dead line is the sign the list has not shrunk after the work that made it dead,
///  and the run must say so on the line itself.
///  A `uses` or `file-uses` line that names a folder no source folder of the package is called
///  says which folder it read and that no source folder has that name:
///  that is what a hand-aligned or mistyped line looks like,
///  and it is the only thing that tells its reader to re-space the line rather than delete it.
///  The sentence speaks of source folders rather than of folders,
///  because the set it is judged against is the module tree's:
///  a folder on disk that holds no module file is no source folder,
///  and telling its reader that the package holds no such folder would be false.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deadEntryFindings(list: AllowList, used: Set<number>, folders: Set<Path>, base: Path): LayoutFinding[];
/// ```
pub fn dead_entry_findings(
    list: &AllowList,
    used: &BTreeSet<usize>,
    folders: &BTreeSet<PathBuf>,
    base: &Path,
) -> Vec<LayoutFinding> {
    // What: the folders of the package in the spelling a list line writes them.
    // Why: an entry names its folder that way, so the two spellings must meet before a name can be called absent.
    let mut known: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for folder in folders {
        known.insert(PathBuf::from(key_path(folder, base)));
    }
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    for (index, entry) in list.entries().iter().enumerate() {
        if used.contains(&index) {
            continue;
        }
        // What: the folders this entry names that no folder of the package is called.
        // Why: a name that is not there is a misspelling or a hand-aligned line, and naming it is what tells its reader apart from a use that is simply gone.
        let mut absent: Vec<String> = Vec::<String>::new();
        match &entry.kind {
            AllowKind::FolderUse {
                from,
                to,
            } => {
                for path in [from, to] {
                    if known.contains(path) {
                        continue;
                    }
                    // What: the folder as the message spells it, with every character a reader cannot see written out.
                    // Why: a tab or a no-break space beside a name is invisible, so `src/a` would be shown for a folder the package does hold.
                    absent.push(format!(
                        "`{}`",
                        spelled_whitespace(path.display().to_string().as_str())
                    ));
                }
            }
            AllowKind::FileUse {
                folder,
                ..
            } => {
                if !known.contains(folder) {
                    absent.push(format!(
                        "`{}`",
                        spelled_whitespace(folder.display().to_string().as_str())
                    ));
                }
            }
            AllowKind::Key(_) => {}
        }
        let (message, help): (String, String) = if absent.is_empty() {
            (
                format!(
                    "The allow-list entry `{}` in {} tolerates nothing any more. Delete the line so the list keeps shrinking.",
                    entry.text,
                    shown(list.source(), base)
                ),
                String::from(
                    "Delete this line. The list holds today's findings only, and a line that matches none of them must not wait for a later round.",
                ),
            )
        } else {
            // What: the word that makes the sentence agree with how many folders are absent.
            // Why: most dead lines name one absent folder, and only a line never read names two.
            let names: &str = if absent.len() == 1 {
                "the folder"
            } else {
                "the folders"
            };
            (
                format!(
                    "The allow-list entry `{}` in {} names {} {}. No source folder of this package has that name, so the entry tolerates nothing any more. Check the line's spelling and the spacing between its words.",
                    entry.text,
                    shown(list.source(), base),
                    names,
                    absent.join(" and ")
                ),
                String::from(
                    "Write the folder name the rule prints, or delete the line if the use is gone: a doubled space, or a tab beside a name, is read as part of that name.",
                ),
            )
        };
        findings.push(LayoutFinding {
            file: list.source().to_path_buf(),
            line: entry.line,
            message,
            help: Some(help),
            key: String::new(),
        });
    }
    return findings;
}
