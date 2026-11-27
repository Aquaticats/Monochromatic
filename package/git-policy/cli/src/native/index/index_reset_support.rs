//! Fixtures that compare the in-process path reset with `git reset` on copies of one repository.
//!
//! A scene is a repository whose branch has moved on by one commit made through a second index file,
//! so the main index
//! still holds the old commit's entries with real stat data,
//! as it does when the commit service lands a commit.
//!
//!  The fixtures are split across siblings because this file is over the line
//!  limit,
//!  and a reader of one control reads only the part it uses:
//!  this one re-exports them,
//!  `index_reset_support_types.rs` holds what a control asks a scene to hold,
//!  `index_reset_support_git.rs` runs real Git and the `git reset` the controls
//!  compare with,
//!  `index_reset_support_scene.rs` builds the scene and its copy,
//!  `index_reset_support_index_reads.rs` reads the index a reset wrote,
//!  and `index_reset_support_landing.rs` runs the update as the landing does.
//!

/// What one control asks a scene to hold,
/// and the repository it becomes.
#[path = "index_reset_support_types.rs"]
mod scene_asked_for;

/// Real Git in the fixture,
/// and the `git reset --no-refresh` the controls compare against.
#[path = "index_reset_support_git.rs"]
mod git_starts;

/// The scene built from one plan,
/// and the copy of it a control compares over.
#[path = "index_reset_support_scene.rs"]
mod scene_and_copy;

/// The index file,
/// its entries and the stat data a control reads after a reset.
#[path = "index_reset_support_index_reads.rs"]
mod reads_after_a_reset;

/// The update run as the landing runs it,
/// and one commit holding a raw tree entry.
#[path = "index_reset_support_landing.rs"]
mod the_landings_reset;

/// Real Git in the fixture,
/// and the `git reset --no-refresh` the controls compare against.
pub(crate) use self::git_starts::{
    git_reset,
    git_reset_status,
    git_with,
};
/// The index file,
/// its entries and the stat data a control reads after a reset.
pub(crate) use self::reads_after_a_reset::{
    entry_of,
    index_bytes,
    listing,
    named,
    stat_of_file,
};
/// The scene built from one plan,
/// and the copy of it a control compares over.
pub(crate) use self::scene_and_copy::{
    copy_of,
    finish,
    scene,
};
/// What one control asks a scene to hold,
/// and the repository it becomes.
pub(crate) use self::scene_asked_for::{
    Edit,
    Plan,
    Scene,
};
/// The update run as the landing runs it.
pub(crate) use self::the_landings_reset::{
    commit_with_raw_entry,
    landing_reset,
    landing_reset_copy,
};
