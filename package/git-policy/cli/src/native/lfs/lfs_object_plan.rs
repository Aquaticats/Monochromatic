//! What:
//!  Narrow an upload plan from "scan these refs with `git-lfs`" to the exact objects the push introduces,
//! decided in
//!  process,
//! or to nothing.
//! Why:
//!  Most pushes add no LFS object,
//! and then no `git-lfs` process and no request to the LFS server is needed.
//! The tips come
//!  from the plan's refs;
//! the commits already known to the destination are the remote's tracking refs when the plan
//!  trusts them
//! (they are kept only where the destination itself reports them at the same commit),
//! else none (the whole history of the pushed refs is scanned,
//! which keeps the case where the tracking
//!  refs may not describe the destination correct).
//! When the readers decline,
//! or an object the scan found is missing
//!  from `.git/lfs` (where `git-lfs push` words the failure and honours `lfs.allowincompletepush`),
//! the plan stays as it
//!  was and `git-lfs` scans.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const narrowed = narrowPlan(plan, layout); // undefined: nothing to upload; same plan: ask git-lfs
//! ```

use crate::lfs_upload_plan::UploadPlan;
use crate::repo_read_layout::RepoLayout;

/// What:
///  The plan to run:
/// the same plan when the readers decline,
/// the plan with its objects named when the scan found some,
///  nothing when it found none.
/// Why:
///   See the module text.
/// Non-Unix hosts have no readers and keep the plan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function narrowPlan(plan: UploadPlan, layout?: RepoLayout): UploadPlan | undefined;
/// ```
pub fn narrow_plan(plan: UploadPlan, layout: Option<&RepoLayout>) -> Option<UploadPlan> {
    #[cfg(unix)]
    {
        let Some(found) = layout else {
            return Some(plan);
        };
        return match unix::scan(&plan, found) {
            None => Some(plan),
            Some(objects) if objects.is_empty() => None,
            Some(objects) => Some(UploadPlan { objects: Some(objects), ..plan }),
        };
    }
    #[cfg(not(unix))]
    {
        let _unused = layout;
        return Some(plan);
    }
}

/// The readers' side of the narrowing.
#[cfg(unix)]
mod unix {
    use crate::diagnostic_log::debug;
    use crate::lfs_pointer_scan::{SCAN_BUDGET, new_pointer_objects};
    use crate::lfs_upload_plan::UploadPlan;
    use crate::repo_read_layout::RepoLayout;
    use crate::repo_read_object::{ObjectKind, from_hex, name_length, to_hex};
    use crate::repo_read_odb::ObjectDatabase;
    use crate::repo_read_refs::{Lookup, RefValue, Refs, is_listable_ref_name};

    /// The tag of the decision lines.
    const TAG: &str = "monochromatic-git-policy-cli lfs-object-plan";

    /// How many tag objects are followed to a commit.
    const PEEL_LIMIT: usize = 5;

    /// Whether one byte is the line feed that ends the first line of a tag object.
    /// `&u8` borrows the byte,
    /// as `split` hands it over.
    fn is_line_feed_byte(byte: &u8) -> bool {
        return *byte == b'\n';
    }

    /// Whether a typed ref name is made only of slash-separated,
    /// non-empty parts of letters,
    /// digits,
    /// dots,
    /// underscores and hyphens.
    fn is_plain_typed_name(typed: &str) -> bool {
        if typed.is_empty() {
            return false;
        }
        for part in typed.split('/') {
            if part.is_empty() {
                return false;
            }
            for byte in part.bytes() {
                if !(byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')) {
                    return false;
                }
            }
        }
        return true;
    }

    /// The commit a name stands for,
    /// through annotated tags;
    /// `None` for anything else.
    fn peel(database: &ObjectDatabase, layout: &RepoLayout, hex: &str) -> Option<Vec<u8>> {
        let mut name: Vec<u8> = from_hex(hex.as_bytes(), layout.object_format)?;
        for _ in 0..PEEL_LIMIT {
            let object = database.read(name.as_slice())?;
            match object.kind {
                ObjectKind::Commit => return Some(name),
                ObjectKind::Tag => {
                    let first: &[u8] = object.bytes.split(is_line_feed_byte).next()?;
                    name = from_hex(first.strip_prefix(b"object ")?, layout.object_format)?;
                }
                ObjectKind::Tree | ObjectKind::Blob => return None,
            }
        }
        return None;
    }

    /// The hexadecimal name a typed ref (`main`,
    /// `refs/tags/v1`,
    /// `HEAD`,
    /// a full object name) stands for,
    /// by Git's order.
    fn tip_of(database: &ObjectDatabase, layout: &RepoLayout, refs: &Refs, typed: &str) -> Option<String> {
        if typed == "HEAD" {
            return match refs.lookup_head(layout.git_dir.as_path())? {
                RefValue::Direct(hex) => Some(hex),
                RefValue::Symbolic(target) => refs.resolve(target.as_str()),
            };
        }
        if !is_plain_typed_name(typed) {
            return None;
        }
        let candidates: [String; 6] = [
            typed.to_owned(),
            format!("refs/{typed}"),
            format!("refs/tags/{typed}"),
            format!("refs/heads/{typed}"),
            format!("refs/remotes/{typed}"),
            format!("refs/remotes/{typed}/HEAD"),
        ];
        for candidate in &candidates {
            if !is_listable_ref_name(candidate) {
                continue;
            }
            match refs.lookup(candidate) {
                Lookup::Found(_) => return refs.resolve(candidate),
                Lookup::Missing => {}
                Lookup::Unsure => return None,
            }
        }
        let as_name: Vec<u8> = from_hex(typed.as_bytes(), layout.object_format)?;
        if database.contains(as_name.as_slice()) {
            return Some(to_hex(as_name.as_slice()));
        }
        return None;
    }

    /// The names of the LFS objects the plan's push introduces;
    /// `None` when the readers decline.
    pub(super) fn scan(plan: &UploadPlan, layout: &RepoLayout) -> Option<Vec<String>> {
        let database: ObjectDatabase = ObjectDatabase::open(layout.common_dir.as_path(), None, layout.object_format)?;
        let refs: Refs = Refs::new(layout.common_dir.as_path(), layout.object_format.hex_length());
        let listing = refs.list()?;
        let mut tips: Vec<String> = Vec::new();
        if plan.everything {
            for (_, value) in &listing {
                if let RefValue::Direct(hex) = value {
                    tips.push(hex.clone());
                }
            }
        } else {
            for typed in &plan.refs {
                tips.push(tip_of(&database, layout, &refs, typed.to_str()?)?);
            }
        }
        let mut include: Vec<Vec<u8>> = Vec::new();
        for hex in &tips {
            include.push(peel(&database, layout, hex)?);
        }
        let mut stop: Vec<Vec<u8>> = Vec::new();
        if !plan.full_scan {
            let remote: &str = plan.remotes.first()?.to_str()?;
            let prefix: String = format!("refs/remotes/{remote}/");
            for (name, value) in &listing {
                if let (true, RefValue::Direct(hex)) = (name.starts_with(prefix.as_str()), value) {
                    stop.push(from_hex(hex.as_bytes(), layout.object_format)?);
                }
            }
        }
        let objects: Vec<String> = new_pointer_objects(&database, layout.object_format, include.as_slice(), stop.as_slice(), SCAN_BUDGET)?;
        for name in &objects {
            let stored = layout.common_dir.join("lfs/objects").join(&name[..2]).join(&name[2..4]).join(name);
            if !stored.is_file() {
                debug(TAG, &format!("object {name} is not in the local LFS storage; asking git-lfs"));
                return None;
            }
        }
        debug(
            TAG,
            &format!(
                "{} tips, {} known tips, full scan {}: {} objects to upload (hash length {})",
                include.len(),
                stop.len(),
                plan.full_scan,
                objects.len(),
                name_length(layout.object_format)
            ),
        );
        return Some(objects);
    }
}

/// The narrowing is controlled in the test module.
#[cfg(test)]
#[path = "lfs_object_plan_tests.rs"]
mod tests;
