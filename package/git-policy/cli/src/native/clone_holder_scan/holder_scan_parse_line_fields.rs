//! What:
//!  The fields one line of a holder scan's result carries:
//!  a mount's id,
//!  root and mount point,
//!  and a holder line's namespace,
//!  thread and every mount that namespace holds.
//! Why:
//!  The line's text is split here and its parts decoded with the writer's own escaping,
//!  so the reader of the whole result
//!  (`crate::clone_holder_scan::holder_scan_parse`) only places the lines;
//!  a line that does not split into its fields refuses the whole result.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mount = mountAfterTheKey('own-mount', value); const holder = holderFields('holder', value);
//! ```

/// One mount of the dataset as a report names it.
use crate::clone_holder_scan::held_mounts::HeldMount;
/// Why a line that is not the field the reader expected refuses the whole result.
use crate::clone_holder_scan::holder_scan_refusals::HolderScanRefusal;
/// The writer's own escaping for a path and for a sentence.
use crate::clone_holder_scan::holder_scan_text::{
    unescape_path,
    unescape_text,
};
/// One namespace and the mounts of the dataset it holds.
use crate::clone_holder_scan::namespace_holder::NamespaceHolder;
/// One mount id,
/// its root and its mount point,
/// as the last three fields of an own-mount line.
pub(crate) fn mount_after_the_key(
    key: &'static str,
    value: &str,
) -> Result<HeldMount, HolderScanRefusal> {
    let parts: Vec<&str> = value.splitn(3, ' ').collect();
    let [number, root, mount_point] = parts.as_slice() else {
        return Err(HolderScanRefusal::Value {
            key,
            shown: String::from(value),
        });
    };
    let Ok(mount_id) = number.parse::<u64>() else {
        return Err(HolderScanRefusal::Value {
            key,
            shown: String::from(value),
        });
    };
    return Ok(HeldMount {
        mount_id,
        root: unescape_path(root),
        mount_point: unescape_path(mount_point),
    });
}

/// What one holder line names:
/// its namespace,
/// its thread and every mount of the clone that namespace holds.
pub(crate) fn holder_fields(
    key: &'static str,
    value: &str,
) -> Result<NamespaceHolder, HolderScanRefusal> {
    let parts: Vec<&str> = value.split(' ').collect();
    let [namespace, task, mounts @ ..] = parts.as_slice() else {
        return Err(HolderScanRefusal::Value {
            key,
            shown: String::from(value),
        });
    };
    let mut held: Vec<HeldMount> = Vec::with_capacity(mounts.len() / 3);
    for triple in mounts.chunks(3) {
        let [number, root, mount_point] = triple else {
            return Err(HolderScanRefusal::Value {
                key,
                shown: String::from(value),
            });
        };
        let Ok(mount_id) = number.parse::<u64>() else {
            return Err(HolderScanRefusal::Value {
                key,
                shown: String::from(value),
            });
        };
        held.push(HeldMount {
            mount_id,
            root: unescape_path(root),
            mount_point: unescape_path(mount_point),
        });
    }
    return Ok(NamespaceHolder {
        namespace: unescape_text(namespace),
        task: unescape_path(task),
        mounts: held,
    });
}
