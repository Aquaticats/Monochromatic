//! Controls for the layout report the invocation capture records.

use super::*;
use crate::repo_read_layout::ObjectFormat;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

/// A layout whose every path is the bytes of one repository,
///  holding a name that is not UTF-8 and a name that holds a line feed.
fn layout_of_bytes() -> RepoLayout {
    let root: &[u8] = b"/repos/na\xefve/a\nb";
    let git_dir: PathBuf = PathBuf::from(OsStr::from_bytes(root)).join(".git");
    return RepoLayout {
        git_dir: git_dir.clone(),
        common_dir: git_dir.clone(),
        worktree_root: PathBuf::from(OsStr::from_bytes(root)),
        prefix: Vec::new(),
        object_format: ObjectFormat::Sha1,
    };
}

/// What:
///  A report holds every path the layout holds,
///  byte for byte.
/// Why:
///   The capture writes its private index and its scratch directory at these paths;
///  a conversion to text replaced a byte that is not UTF-8 with U+FFFD and read the line feed as the end of one path,
///  which points those writes at directories that do not exist.
#[test]
fn a_report_keeps_every_path_the_layout_holds() {
    let layout: RepoLayout = layout_of_bytes();
    let report: InvocationReport = invocation_report(&layout).expect("the layout names its own Git paths");
    assert_eq!(report.top_level, layout.worktree_root, "the top level");
    assert_eq!(report.git_dir, layout.git_dir, "the Git directory");
    assert_eq!(report.common_dir, layout.common_dir, "the common directory");
    assert_eq!(report.index_path, layout.git_dir.join("index"), "the index");
    assert_eq!(report.object_directory, layout.git_dir.join("objects"), "the objects");
    assert_eq!(report.registry_root, layout.git_dir.join("cli-git-transactions"), "the registry");
    assert_eq!(report.object_format, "sha1");
}

/// What:
///  A linked worktree has no in-process report,
///  because Git splits its names between the two directories.
/// Why:
///   The index,
///  the objects and the registry of a linked worktree are placed by a table only Git knows,
///  so naming them by a join would put them somewhere Git never reads.
#[test]
fn a_report_declines_a_linked_worktree() {
    let common: PathBuf = PathBuf::from(OsStr::from_bytes(b"/repos/na\xefve/a\nb/.git"));
    let linked: RepoLayout = RepoLayout {
        git_dir: common.join("worktrees").join("side"),
        common_dir: common.clone(),
        worktree_root: PathBuf::from(OsStr::from_bytes(b"/repos/na\xefve/a\nb-side")),
        prefix: Vec::new(),
        object_format: ObjectFormat::Sha256,
    };
    assert!(invocation_report(&linked).is_none(), "a linked worktree's Git paths stay with Git");
}
