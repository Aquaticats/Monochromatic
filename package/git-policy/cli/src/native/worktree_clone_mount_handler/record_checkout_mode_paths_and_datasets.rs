//! What:
//!  What a checkout's path and a dataset's name have to be:
//!  the checkout is the root of the mount it is on,
//!  a container sits beside the checkout and neither inside it nor above it,
//!  and the pool a dataset stands in is the first component of its name.
//! Why:
//!  A record written for a folder inside a dataset would make every later proof
//!  compare against a folder,
//!  and a container inside the checkout would put managed clones inside a
//!  served tree,
//!  so each of those facts is proved here rather than assumed by the caller.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const isRoot = (checkout, mount) => canonical(checkout) === canonical(mount.mountPoint);
//! const pool = (dataset) => dataset.split('/')[0];
//! proveTheContainerBeside(dataset, container); // throws ContainerNotBesideTheCheckout
//! ```

use crate::worktree_clone_mount_handler::record_checkout_mode_record::RecordCheckoutFailure;
use crate::worktree_clone_mount_handler::sweep_checkout_mount::CheckoutMount;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Whether the checkout is the root of the mount it is on.
/// Why:
///  A folder inside a dataset's mount is not the dataset's own mountpoint,
///  and a record written for it would make every later proof compare against a
///  folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isRoot = (checkout, mount) => canonical(checkout) === canonical(mount.mountPoint);
/// ```
pub(super) fn the_checkout_is_the_mounts_root(
    checkout: &Path,
    mount: &CheckoutMount,
) -> bool {
    return canonical(checkout) == canonical(mount.mount_point.as_path());
}

/// A path with its `.` and `..` components resolved and its repeated slashes
/// folded,
/// as text,
/// and the path as written when it cannot be read.
fn canonical(path: &Path) -> PathBuf {
    return std::fs::canonicalize(path).unwrap_or_else(|_error| return lexical(path));
}

/// What:
///  A path's text with its `.` and `..` components resolved and its repeated
///  slashes folded,
///  without asking the system.
/// Why:
///  A mountpoint whose own directory does not exist yet is compared as the text
///  it is,
///  so the proof never creates a directory to compare one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folded = lexical('/home/user/./Monochromatic//package/../');
/// ```
fn lexical(path: &Path) -> PathBuf {
    let mut folded: PathBuf = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                let _dropped = folded.pop();
            }
            other => folded.push(other.as_os_str()),
        }
    }
    return folded;
}

/// What:
///  The pool a dataset's name stands in.
/// Why:
///  A pool is the name's first component,
///  and the container and the checkout must be in one pool or a foreign pool's
///  subtree would sit inside a served checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = (dataset) => dataset.split('/')[0];
/// ```
pub(super) fn pool_of(dataset: &str) -> String {
    return dataset.split('/').next().unwrap_or_default().to_string();
}

/// What:
///  Refuse a container that is not beside the checkout.
/// Why:
///  The route places the container beside the checkout,
///  and a container inside the checkout would put managed clones inside a served
///  tree,
///  while a container holding the checkout would put a clone store above it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proveTheContainerBeside('zpcachyos/nosnap/monochromatic-git-checkout', 'zpcachyos/nosnap/clone-datasets');
/// ```
///
/// # Errors
/// `ContainerNotBesideTheCheckout` naming the direction the proof found.
pub(super) fn prove_the_container_beside(
    dataset: &str,
    container: &str,
) -> Result<(), RecordCheckoutFailure> {
    if container == dataset {
        return Err(RecordCheckoutFailure::ContainerNotBesideTheCheckout {
            dataset: String::from(dataset),
            container: String::from(container),
            because: String::from("they are the same dataset"),
        });
    }
    if is_inside(container, dataset) {
        return Err(RecordCheckoutFailure::ContainerNotBesideTheCheckout {
            dataset: String::from(dataset),
            container: String::from(container),
            because: String::from("the container is inside the checkout"),
        });
    }
    if is_inside(dataset, container) {
        return Err(RecordCheckoutFailure::ContainerNotBesideTheCheckout {
            dataset: String::from(dataset),
            container: String::from(container),
            because: String::from("the container holds the checkout"),
        });
    }
    return Ok(());
}

/// Whether one dataset's name stands under another's,
/// compared on `/` boundaries so `a/bc` is not inside `a/b`.
fn is_inside(
    inner: &str,
    outer: &str,
) -> bool {
    return inner.len() > outer.len()
        && inner.starts_with(outer)
        && inner.as_bytes().get(outer.len()) == Some(&b'/');
}
