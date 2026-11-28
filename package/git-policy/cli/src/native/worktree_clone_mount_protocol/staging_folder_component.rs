//! What:
//!  The one component a clone's staging folder begins with,
//!  and the question whether a mount point's last component is such a folder.
//! Why:
//!  A removal takes a clone out of its slot by moving it onto a staging folder
//!  under the account's cache,
//!  and the folder is named from the clone's canonical numeric GUID
//!  (`doc/decision/cli-git-zfs-staging-under-cache.md`,
//!  "Decision").
//!  Two readers ask whether a clone sits on that folder and neither may own the
//!  name:
//!  the slot state asks whether the clone's mount is the staging folder a reader
//!  may unmount and destroy,
//!  and the removal's own unmount writes the folders.
//!  The name lives here,
//!  in the folder both already read,
//!  so no second copy of it can disagree
//!  (`AGENTS.md` rule `MGN`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const name = `${STAGING_FOLDER_PREFIX}${guid}`;
//! const onTheStagingFolder = isAStagingFolderName(mountPoint.split('/').at(-1) ?? '');
//! ```

use std::path::Path;

/// The component every staging folder of this route begins with.
///
/// Why:
///  The prefix is the wrapper's own,
///  so a folder under the cache that begins with it was made by a removal and
///  not by a person,
///  and one that does not is no staging folder of this route.
pub(crate) const STAGING_FOLDER_PREFIX: &str = "monochromatic-git-policy-cli-staging-";

/// What:
///  Whether a folder name is one this route derives for a staging folder.
/// Why:
///  The name after the prefix is the clone's GUID and nothing else,
///  so the prefix alone tells a removal's folder from a person's,
///  and the caller that matched a mount of its clone against the prefix has
///  already proved which clone the folder carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const derived = isAStagingFolderName('monochromatic-git-policy-cli-staging-1234');
/// ```
pub(crate) fn is_a_staging_folder_name(name: &str) -> bool {
    return name.starts_with(STAGING_FOLDER_PREFIX);
}

/// What:
///  Whether a mount point stands on a folder whose name is a staging folder's.
/// Why:
///  A removal may meet its clone mounted where an earlier unmount put it,
///  and the mount table prints the folder as the path the clone is mounted on,
///  so its last component is the name the removal derived.
///  A mount point with no last component,
///  or one whose bytes are no text,
///  is no folder this route named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staged = isAStagingFolderMount('/home/u/.cache/monochromatic-git-policy-cli-staging-1234');
/// ```
pub(crate) fn is_a_staging_folder_mount(mount_point: &Path) -> bool {
    let Some(name) = mount_point
        .file_name()
        .and_then(|name| return name.to_str())
    else {
        return false;
    };
    return is_a_staging_folder_name(name);
}
