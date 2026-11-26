//! What:
//!  The names the ZFS route's healthy facts are built from:
//!  the checkout and its dataset,
//!  the clone container and one clone below it,
//!  the managed account and its group,
//!  this boot,
//!  the staging folder's prefix,
//!  the add nonce the healthy clone's lock names,
//!  and the origin snapshot every fixture clone names.
//! Why:
//!  Every builder beside this file spells these once,
//!  so a control that changes one field still reads the same repository,
//!  pool and boot everywhere else,
//!  and a control names the value it judges rather than a copy of it.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const CHECKOUT = '/home/user/Monochromatic';
//! const CHECKOUT_DATASET = 'zpcachyos/ROOT/cos/home/monochromatic-git-checkout';
//! ```

/// The checkout the design names.
pub(crate) const CHECKOUT: &str = "/home/user/Monochromatic";

/// The pool the checkout dataset lives in.
pub(crate) const POOL: &str = "zpcachyos";

/// The checkout dataset.
pub(crate) const CHECKOUT_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-checkout";

/// The clone container beside it.
pub(crate) const CONTAINER_DATASET: &str =
    "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets";

/// One managed clone below the container.
pub(crate) const CLONE_DATASET: &str =
    "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/one";

/// The managed account's user id.
pub(crate) const ACCOUNT: u32 = 1000;

/// The managed account's primary group.
pub(crate) const ACCOUNT_GROUP: u32 = 1000;

/// The boot identifier this run is in.
pub(crate) const LIVE_BOOT: &str = "4c4c4544-0042-3510-8051-b4c04f4e4d32";

/// The staging folder prefix,
/// which a control's names carry.
pub(crate) const STAGING_PREFIX: &str =
    crate::worktree_clone_unmount::staging_folder_name::STAGING_FOLDER_PREFIX;

/// The add nonce the healthy clone's managed Git lock names,
/// which is step 13's own fixture nonce.
pub(crate) const LOCK_NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// The origin snapshot every fixture clone names.
pub(crate) const ORIGIN_SNAPSHOT: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";
