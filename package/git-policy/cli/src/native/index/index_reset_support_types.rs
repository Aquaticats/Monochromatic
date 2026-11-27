//! What:
//!  What one control asks a scene to hold:
//!  the edits its second commit makes,
//!  the files,
//!  links,
//!  submodule entries and configuration its first commit carries,
//!  the repository a scene is,
//!  and the two commits its index and its branch are at.
//! Why:
//!  A control names the change it compares over rather than building a
//!  repository by hand,
//!  and the commit the index holds and the commit the branch is at are what
//!  the reset's two readings are compared against.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Edit = { kind: 'put'; path; mode; content } | { kind: 'gitlink'; path; id } | { kind: 'delete'; path };
//! type Plan = { files; links; gitlinks; edits; fresh; config; untrackedCache; sha256 };
//! type Scene = { root; repo; old; new };
//! ```

/// The fixture directory a scene is made in,
/// and the repository inside it.
use std::path::PathBuf;
/// One change the second commit makes to the first commit's tree.
pub(crate) enum Edit<'a> {
    /// A file (or symbolic link,
    /// by mode) with this content.
    Put {
        path: &'a str,
        mode: &'a str,
        content: &'a [u8],
    },
    /// A submodule entry naming a commit that is not in this repository.
    Gitlink { path: &'a str, id: &'a str },
    /// The path leaves the tree.
    Delete(&'a str),
}

/// A repository,
/// the commit its index is at,
/// and the commit the branch is at.
pub(crate) struct Scene {
    /// The fixture directory,
    /// removed by the caller.
    pub root: PathBuf,
    /// The repository.
    pub repo: PathBuf,
    /// The commit the index holds.
    pub old: String,
    /// The commit the branch is at.
    pub new: String,
}

/// What a scene's first commit holds and its second commit changes.
#[derive(Default)]
pub(crate) struct Plan<'a> {
    /// Regular files of the first commit:
    /// path,
    /// content,
    /// executable.
    pub files: &'a [(&'a str, &'a [u8], bool)],
    /// Symbolic links of the first commit:
    /// path,
    /// target.
    pub links: &'a [(&'a str, &'a str)],
    /// Submodule entries of the first commit:
    /// path,
    /// commit name.
    pub gitlinks: &'a [(&'a str, &'a str)],
    /// What the second commit changes.
    pub edits: &'a [Edit<'a>],
    /// Whether the first commit's files keep the time they were written at (otherwise they are aged).
    pub fresh: bool,
    /// Configuration the repository carries before its first commit,
    /// such as `feature.manyFiles`,
    /// which Git honours on every index file it writes here.
    pub config: &'a [(&'a str, &'a str)],
    /// Whether the configuration asks for an untracked cache,
    /// so a status after the first commit writes one into the index the landing copies.
    pub untracked_cache: bool,
    /// Whether the repository uses SHA-256 object names.
    pub sha256: bool,
}
