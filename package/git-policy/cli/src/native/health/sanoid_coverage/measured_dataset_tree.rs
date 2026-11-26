//! What:
//!  The dataset tree one machine measured:
//!  each dataset's name,
//!  its mount point and its `com.sun:auto-snapshot` value,
//!  and the ancestry readers the coverage comparison needs.
//! Why:
//!  Recursive coverage follows dataset ancestry,
//!  never the paths datasets are mounted at
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Snapshots and setup reporting"),
//!  so the tree is a list of names and the readers split names on `/`.
//!  A name that merely begins with another name is not a child:
//!  `tank/homework` is not under `tank/home`,
//!  which is the one mistake a `starts_with` test makes.
//!  The facts are values the caller read from the live system,
//!  so no control here opens `/dev/zfs` or starts a program.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type MeasuredTree = { datasets: { name: string; mountpoint?: string; autoSnapshot?: string }[] };
//! tree.datasetsUnder('tank/home'); // the descendants, sorted by name
//! ```

/// What:
///  What one machine read about one dataset.
/// Why:
///  The coverage comparison needs the property that marks an exclusion and
///  the mount point that tells a mounted exclusion from an unmounted one,
///  and it must tell "the property is absent" from "the property is false",
///  because a person's unset marker is drift the correction sets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MeasuredDataset = { name: string; mountpoint?: string; autoSnapshot?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeasuredDataset {
    /// The dataset's whole name.
    pub name: String,
    /// `zfs get mountpoint`,
    /// or nothing when the read gave no value.
    pub mountpoint: Option<String>,
    /// `zfs get com.sun:auto-snapshot`,
    /// or nothing when the property is not set.
    pub auto_snapshot: Option<String>,
}

/// What:
///  Every dataset one machine measured.
/// Why:
///  The coverage comparison and the guard's report answer from one reading,
///  so both speak of the same tree even when a dataset appears after the
///  other check ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MeasuredTree = { datasets: MeasuredDataset[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeasuredTree {
    /// The datasets in the order they were read.
    pub datasets: Vec<MeasuredDataset>,
}

/// What:
///  Whether a dataset name lies under another dataset's name.
/// Why:
///  ZFS ancestry is name ancestry on `/`,
///  so the test needs the separator and not a prefix alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// child.startsWith(`${ancestor}/`);
/// ```
pub fn is_under(child: &str, ancestor: &str) -> bool {
    let Some(rest) = child.strip_prefix(ancestor) else {
        return false;
    };
    return rest.starts_with('/');
}

/// What:
///  Whether a dataset's mount point is a path.
/// Why:
///  `none`,
/// `-` and `legacy` cover no folder,
///  so an exclusion that expects to cover a folder and reads one of them
///  protects nothing while the parent dataset keeps snapshotting the folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mountpoint?.startsWith('/') ?? false;
/// ```
pub fn has_path_mountpoint(mountpoint: Option<&str>) -> bool {
    let Some(written) = mountpoint else {
        return false;
    };
    return written.starts_with('/');
}

impl MeasuredTree {
    /// One dataset's facts.
    pub fn dataset(&self, name: &str) -> Option<&MeasuredDataset> {
        return self.datasets.iter().find(|dataset| return dataset.name == name);
    }

    /// Every dataset under one name,
    /// sorted by name so two reports of one machine read the same.
    pub fn datasets_under(&self, ancestor: &str) -> Vec<&MeasuredDataset> {
        let mut found: Vec<&MeasuredDataset> = self.datasets.iter().filter(|dataset| return is_under(dataset.name.as_str(), ancestor)).collect();
        found.sort_by(|left, right| return left.name.cmp(&right.name));
        return found;
    }

    /// Every measured ancestor of one dataset,
    /// the nearest first,
    /// which is the order sanoid's section reading cares about least but a
    /// report reads best.
    pub fn ancestors(&self, name: &str) -> Vec<&MeasuredDataset> {
        let mut found: Vec<&MeasuredDataset> = self.datasets.iter().filter(|dataset| return is_under(name, dataset.name.as_str())).collect();
        found.sort_by(|left, right| return right.name.len().cmp(&left.name.len()));
        return found;
    }

    /// What:
    ///  The dataset whose path mount point is the longest prefix of one path.
    /// Why:
    ///  The store's place under recursive coverage is decided by which
    ///  dataset holds its folder,
    ///  and a dataset mounted at `none` or `legacy` holds no folder at all.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tree.holdingDataset('/fixture/home/.local/share/pnpm/store'); // 'tank/home'
    /// ```
    pub fn holding_dataset(&self, path: &str) -> Option<&str> {
        let mut best: Option<(&MeasuredDataset, usize)> = None;
        for dataset in &self.datasets {
            let Some(mounted) = dataset.mountpoint.as_deref() else {
                continue;
            };
            if !has_path_mountpoint(Some(mounted)) {
                continue;
            }
            let covers: bool = path == mounted || path.strip_prefix(mounted).is_some_and(|rest| return rest.starts_with('/'));
            if !covers {
                continue;
            }
            if best.is_none_or(|(_, held)| return held < mounted.len()) {
                best = Some((dataset, mounted.len()));
            }
        }
        return best.map(|(dataset, _)| return dataset.name.as_str());
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "measured_dataset_tree_tests.rs"]
mod tests;
