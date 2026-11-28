//! What:
//!  The names the route's clone parent has,
//!  as one value the event filter compares a delivery against.
//! Why:
//!  The filter accepts an event whose pool is the parent's and whose dataset is
//!  the parent or below it,
//!  so it needs the parent's pool and its dataset and nothing else
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  veto entry V2).
//!  Both names come from the one dataset name the route's configuration already
//!  carries,
//!  and that name's first component is its pool,
//!  which is what lets the filter decide without the pool's own facts:
//!  those are read through the ZFS device,
//!  and the filter opens nothing.
//!  A name that is no dataset this route could act on resolves to nothing,
//!  and the filter lets the event through so the handler's own checks give the
//!  person the refusal (V2:
//! "it never fails closed").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const parent = ParentScope.ofDataset('zpcachyos/nosnap/monochromatic-git-worktree-clone-datasets');
//! // { pool: 'zpcachyos', dataset: 'zpcachyos/nosnap/monochromatic-git-worktree-clone-datasets' }
//! ```

/// What:
///  The pool and dataset the route's clone parent has.
/// Why:
///   The two names are compared against a delivery's own two,
///  and they are kept together because a pool without its dataset,
/// or the
///  other way round,
/// accepts or drops nothing meaningful.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ParentScope = { pool: string; dataset: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParentScope {
    /// The pool the parent dataset lives in.
    pool: String,
    /// The pinned clone parent dataset.
    dataset: String,
}

impl ParentScope {
    /// What:
    ///  The parent's two names,
    ///  read from the dataset name the route's configuration carries.
    /// Why:
    ///   A dataset name's first component is its pool,
    ///  so one name answers both questions the filter asks,
    ///  and a name with no component after the pool names no parent this route
    ///  could enumerate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const parent = ParentScope.ofDataset('zpcachyos/nosnap/zfsroute/clones');
    /// ```
    pub fn of_dataset(dataset: &str) -> Option<ParentScope> {
        let (pool, below) = dataset.split_once('/')?;
        if pool.is_empty() || below.is_empty() {
            return None;
        }
        return Some(ParentScope {
            pool: String::from(pool),
            dataset: String::from(dataset),
        });
    }

    /// The pool the parent dataset lives in.
    pub fn pool(&self) -> &str {
        return self.pool.as_str();
    }

    /// The pinned clone parent dataset.
    pub fn dataset(&self) -> &str {
        return self.dataset.as_str();
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "parent_scope_tests.rs"]
mod tests;
