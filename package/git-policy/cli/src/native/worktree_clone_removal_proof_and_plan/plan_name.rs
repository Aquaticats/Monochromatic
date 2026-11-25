//! What:
//!  The two name shapes a destruction plan's values rest on:
//!  a dataset's name and a snapshot's.
//! Why:
//!  A plan is keyed by the clone's GUID because a slot's name is used again when a worktree is
//!  added again at one path,
//!  so a plan keyed that way would be one name for two clones
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The key,
//! and why it is the clone's GUID").
//!  A ZFS name holds no newline and no `=` (`F2`),
//!  which is what lets a `key=value` line carry one verbatim,
//!  so the reader checks that and the shape a request's own name check would accept,
//!  and never interprets the name.
//!  This is not a second `entity_namecheck`:
//! it refuses the shapes a plan cannot hold,
//!  and the kernel still refuses a name it does not accept when a request carries one.
//!  The file's own name is the clone's canonical GUID,
//!  and that check is the binding record's own
//!  (`worktree_clone_binding_record::plan_folder_read::is_canonical_guid_name`),
//!  so one grammar decides it for the add and for this reader.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const isSnapshotName = (name: string) => name.split('@').length === 2 && bothPartsAreNames(name);
//! ```

/// Whether one byte may appear in a ZFS dataset or snapshot name.
///
/// Why:
///  The kernel's charset holds letters,
///  digits,
///  the space and a few punctuation bytes,
///  and neither a newline nor `=`;
///  a control byte or a second `@` would forge a line or a snapshot name.
fn is_name_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b'_' | b'-' | b'.' | b':' | b'%');
}

/// Whether one `/`-separated component of a dataset path is a name.
fn is_dataset_component(component: &str) -> bool {
    if component.is_empty() || component == "." || component == ".." {
        return false;
    }
    return component.bytes().all(is_name_byte);
}

/// What:
///  Whether text is a dataset's full name as a plan may carry one.
/// Why:
///  The clone's and the origin's dataset names decide which objects a request names,
///  so a value with a newline,
///  an `=` or an empty component is refused before it can reach a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isDatasetName = (text) => text.split('/').every(isComponent);
/// ```
pub fn is_dataset_name(text: &str) -> bool {
    if text.is_empty() || text.len() > crate::worktree_clone_removal_proof_and_plan::plan_bounds::ZFS_NAME_MAXIMUM_BYTES {
        return false;
    }
    return text.split('/').all(is_dataset_component);
}

/// What:
///  Whether text is one snapshot's full name,
///  `<dataset>@<snapshot>`.
/// Why:
///  An attempt line names the snapshot it destroys,
///  and a snapshot name is the one field the line lets hold a space,
///  so the shape is checked here rather than by splitting on the last `=`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSnapshotName = (text) => { const [dataset, snap, ...rest] = text.split('@'); return isDatasetName(dataset) && snap !== '' && rest.length === 0; };
/// ```
pub fn is_snapshot_name(text: &str) -> bool {
    let Some((dataset, snapshot)) = text.split_once('@') else {
        return false;
    };
    if snapshot.is_empty() || snapshot.contains('@') {
        return false;
    }
    if !snapshot.bytes().all(is_name_byte) {
        return false;
    }
    return is_dataset_name(dataset);
}

/// Whether a snapshot name belongs to the dataset a plan names,
/// which is the only dataset its attempt lines record.
pub fn is_a_snapshot_of(dataset: &str, snapshot_name: &str) -> bool {
    return match snapshot_name.split_once('@') {
        Some((owner, _)) => owner == dataset,
        None => false,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "plan_name_tests.rs"]
mod tests;
