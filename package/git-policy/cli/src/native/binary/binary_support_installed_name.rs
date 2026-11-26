//! What:
//!  The installed ZED history event's name in a guest control's folder,
//!  and the words every other root mode is reached through.
//! Why:
//!  The release holds one executable,
//!  and a root mode is reached either by the installed name,
//!  for ZED's no-argument sweep,
//!  or by the words,
//!  which select a mode on any file
//!  (`doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`;
//!  `doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
//!  choice 1).
//!  A guest control that spelled the name into its own command line would copy
//!  the rule into every arm,
//!  so the link is made here once and each arm names it.
//!  The name and the words are read from the library's constants,
//!  which the source guards pin,
//!  so a control cannot name a spelling the dispatch does not accept.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const link = installedName(); spawnSync(link, []); // the sweep
//! spawnSync(link, [...rootModeWords(), 'start-probe']);
//! ```

use git_policy_cli::foundation::wrapper_names_old_and_new::COMMAND_WORD;
use git_policy_cli::worktree_clone_mount_handler::root_mode_names::{
    ROOT_GROUP_WORD,
    ZEDLET_START_NAME,
};
use std::path::PathBuf;

/// The wrapper this control's guest built,
/// which the link names.
pub const BUILT_WRAPPER: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// What:
///  The installed name's link to the built wrapper,
///  made fresh for this start.
/// Why:
///  The name is the whole last path component and nothing else,
///  so a link under the fixture root answers exactly as the installed file
///  does,
///  on this host and inside a guest,
///  and a control never needs root's own install folder to exist.
///  Each caller names its own folder, so two arms of one test never share a
///  link and a target,
///  and the link is removed and made again so a re-run starts from nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const link = installedName('sweep');
/// ```
pub fn installed_name(what: &str) -> PathBuf {
    let folder: PathBuf = crate::support_fixture_directories::fixture_root()
        .join("installed-name")
        .join(what);
    std::fs::create_dir_all(folder.as_path()).expect("the installed name's folder");
    let link: PathBuf = folder.join(ZEDLET_START_NAME);
    let _removed: std::io::Result<()> = std::fs::remove_file(link.as_path());
    std::os::unix::fs::symlink(BUILT_WRAPPER, link.as_path()).expect("the installed name's link");
    return link;
}

/// The words a unit, an adoption and a person spell before a mode word.
pub fn root_mode_words() -> [&'static str; 2] {
    return [COMMAND_WORD.current, ROOT_GROUP_WORD];
}
