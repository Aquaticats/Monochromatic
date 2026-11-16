//! What:
//!  One path a test thread made,
//!  with the identity the entry at it had when the thread made it.
//! Why:
//!  A path is a name,
//! and the names a control makes come round:
//! a fixture root carries the process number,
//! and a runtime directory carries a token drawn from sixteen million.
//!  Two runs can therefore stand at one path in turn,
//! and a cleanup at the end of the first thread that removed "the entry at this path" would take
//! the entry the second run made,
//! which is neither its to remove nor harmless to that run.
//!  So the identity is read once,
//! when the thread makes the entry,
//! and the thread's cleanup asks it before it removes anything
//! (`folder_identity`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const made = madeByThisThread(root); // { path: root, identity: fs.lstatSync(root) }
//! made.standing(); // 'stillTheOne' | 'anotherEntryNow' | 'gone'
//! ```

/// The path type an entry is named with.
use std::path::{Path, PathBuf};

/// The device and inode an entry had when a control made it,
/// and what stands at that path now.
use crate::testing::folder_identity::{self, FolderIdentity, FolderStanding};

/// What:
///  One entry a test thread made:
///  the path,
///  and the identity that path had when the thread took it.
/// Why:
///   The identity is nothing where nothing stood at the path when it was taken,
/// which no caller here does:
/// every one of them makes its entry first.
pub(crate) struct PathThisThreadMade {
    /// The path this thread made.
    pub(crate) path: PathBuf,
    /// The device and the inode it had then,
    /// or nothing where nothing stood there.
    identity: Option<FolderIdentity>,
}

/// What:
///  Take `path` as one entry this thread made,
///  with the identity the entry at it has now.
/// Why:
///   The identity has to be read while the entry is still the thread's own,
/// which is here and never at the thread's end.
pub(crate) fn made_by_this_thread(path: &Path) -> PathThisThreadMade {
    let identity: Option<FolderIdentity> = folder_identity::identity_of(path);
    return PathThisThreadMade { path: path.to_path_buf(), identity };
}

impl PathThisThreadMade {
    /// What:
    ///  What stands at this path now,
    ///  read against the identity the entry had when this thread made it.
    /// Why:
    ///   A cleanup that removes only `StillTheOne` never takes an entry another run made at a name this one used.
    pub(crate) fn standing(&self) -> Option<FolderStanding> {
        return self.identity.map(|identity| { return identity.standing(self.path.as_path()); });
    }
}
