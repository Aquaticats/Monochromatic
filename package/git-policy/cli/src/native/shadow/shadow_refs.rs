//! What:
//!  The shadow ref store:
//! a snapshot of every real ref,
//!       the private target ref,
//!       symbolic refs,
//!       and the private `HEAD`.
//! Why:
//!  Hooks resolve tags,
//! other branches and remote-tracking refs as they stood at invocation.
//!  With the files backend the snapshot is one `packed-refs` file and the private target is a loose ref,
//!  which takes precedence over a packed entry;
//!  with reftable the snapshot is one `update-ref --stdin` transaction.
//!  A reference name is bytes,
//!  so a valid name that is not text reaches the shadow unchanged and never refuses the snapshot.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeShadowRefs({ gitPath, cwd, shadowPath, capture });
//! ```

/// One shadow Git start,
/// one real Git start,
/// and their failure text.
use crate::shadow_git::shadow_request;
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
/// Private file writes for the snapshot,
/// published whole and not synced.
use crate::private_storage::write_private_file_unsynced;
/// Journal enums:
/// the ref backend,
/// the base and the symbolic `HEAD` at invocation.
use crate::transaction_journal::{Base, RefFormat, SymbolicHead};
/// Owned argument text,
/// for a reference name that is bytes.
use std::ffi::OsString;
/// The operating-system name of a name given as bytes.
use crate::path_bytes::os_from_path_bytes;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Whether one byte is the line feed that ends one listed ref.
/// `&u8` borrows the byte,
/// as `split` hands it over.
fn is_line_feed_byte(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Whether one byte is the NUL that ends one field of a listed ref.
/// `&u8` borrows the byte,
/// as `split` hands it over.
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// Header Git writes for a sorted `packed-refs` file without peeled lines.
///  The trailing space matches Git's own writer,
/// which older readers require.
pub const PACKED_REFS_HEADER: &str = "# pack-refs with: sorted \n";

/// What:
///  One real ref at invocation.
/// Why:
///   Symbolic refs and direct refs take different snapshot forms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefSnapshotEntry = { name: string; oid: string; symref: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefSnapshotEntry {
    /// Full ref name,
    /// as the bytes Git lists.
    pub name: Vec<u8>,
    /// Object the ref resolves to,
    /// as the hexadecimal text Git lists.
    pub oid: String,
    /// Symbolic target,
    /// as the bytes Git lists,
    /// or empty for a direct ref.
    pub symref: Vec<u8>,
}

/// Order two ref entries by the bytes of their names,
/// which is the order Git lists refs in.
/// `sort_by` takes this function by path.
fn compare_by_name_bytes(left: &RefSnapshotEntry, right: &RefSnapshotEntry) -> std::cmp::Ordering {
    return left.name.cmp(&right.name);
}

/// The failure for a `packed-refs` file that could not be written.
/// `map_err` takes this function by path.
fn packed_refs_failure(error: std::io::Error) -> GitFailure {
    return GitFailure(format!("writing packed-refs failed: {error}"));
}

/// What:
///  Lists every real ref of the owning worktree in byte order.
/// Why:
///   Sorting by bytes matches the incumbent's `Buffer.compare` order in the packed file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await listRealRefs({ gitPath, cwd });
/// ```
pub fn list_real_refs(context: &GitContext, cwd: &Path) -> Result<Vec<RefSnapshotEntry>, GitFailure> {
    let mut request: GitRequest = GitRequest::new(
        cwd,
        &["for-each-ref", "--format=%(refname)%00%(objectname)%00%(symref)"],
    );
    request.without_prefix = true;
    let output: GitOutput = run_git_checked(context, &request)?;
    let mut refs: Vec<RefSnapshotEntry> = Vec::new();
    for line in output.stdout.split(is_line_feed_byte) {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&[u8]> = line.split(is_nul_byte).collect();
        let [name, oid, symref] = fields.as_slice() else {
            return Err(GitFailure(format!(
                "git for-each-ref returned a malformed line: {:?}",
                String::from_utf8_lossy(line)
            )));
        };
        // An object name is hexadecimal text; only a name may be bytes that are not text.
        let object: &str = match std::str::from_utf8(oid) {
            Ok(text) => text,
            Err(_) => {
                return Err(GitFailure(format!(
                    "git for-each-ref returned a malformed object name: {:?}",
                    String::from_utf8_lossy(line)
                )))
            }
        };
        refs.push(RefSnapshotEntry {
            name: name.to_vec(),
            oid: String::from(object),
            symref: symref.to_vec(),
        });
    }
    refs.sort_by(compare_by_name_bytes);
    return Ok(refs);
}

/// What:
///  Formats the files-backend snapshot of every direct ref except the private target.
/// Why:
///   The target is written loose,
/// so a packed copy would only be shadowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// formatPackedRefs({ refs, targetRef });
/// ```
pub fn format_packed_refs(refs: &[RefSnapshotEntry], target_ref: &str) -> Vec<u8> {
    let mut text: Vec<u8> = PACKED_REFS_HEADER.as_bytes().to_vec();
    for entry in refs {
        if !(entry.symref.is_empty() && entry.name.as_slice() != target_ref.as_bytes()) {
            continue;
        }
        text.extend_from_slice(entry.oid.as_bytes());
        text.push(b' ');
        text.extend_from_slice(entry.name.as_slice());
        text.push(b'\n');
    }
    return text;
}

/// What:
///  Points the shadow `HEAD` at the private target:
///  symbolic to the branch,
/// or detached at the base.
/// Why:
///   A detached `HEAD` needs a base,
/// and an unborn branch has no object to write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeShadowHead({ gitPath, shadowPath, symbolicHead, base });
/// ```
fn write_shadow_head(
    context: &GitContext,
    shadow: &Path,
    symbolic_head: &SymbolicHead,
    base: &Base,
) -> Result<(), GitFailure> {
    match symbolic_head {
        SymbolicHead::Branch(reference) => {
            if let Base::Commit(oid) = base {
                run_git_checked(
                    context,
                    &shadow_request(shadow, shadow, &["update-ref", reference.as_str(), oid.as_str()]),
                )?;
            }
            run_git_checked(
                context,
                &shadow_request(shadow, shadow, &["symbolic-ref", "HEAD", reference.as_str()]),
            )?;
            return Ok(());
        }
        SymbolicHead::Detached => {
            let Base::Commit(oid) = base else {
                return Err(GitFailure(String::from(
                    "A detached HEAD has no unborn preparation base.",
                )));
            };
            run_git_checked(
                context,
                &shadow_request(shadow, shadow, &["update-ref", "--no-deref", "HEAD", oid.as_str()]),
            )?;
            return Ok(());
        }
    }
}

/// What:
///  Writes the shadow ref store:
///  the snapshot,
/// the symbolic refs,
/// the private target ref and `HEAD`.
/// Why:
///   The order matches the incumbent's:
///  symbolic refs come after their targets exist,
/// and `HEAD` comes last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeShadowRefs({ gitPath, cwd, shadowPath, capture });
/// ```
pub fn write_shadow_refs(
    context: &GitContext,
    cwd: &Path,
    shadow: &Path,
    ref_format: RefFormat,
    target_ref: &str,
    symbolic_head: &SymbolicHead,
    base: &Base,
) -> Result<(), GitFailure> {
    let refs: Vec<RefSnapshotEntry> = list_real_refs(context, cwd)?;
    match ref_format {
        RefFormat::Files => {
            // Unsynced: nothing reads the throwaway shadow's refs after a crash on the files backend, so a torn or lost
            // `packed-refs` changes no recovery outcome (`binary_recovery_tearing_shadow_tests.rs`).
            write_private_file_unsynced(
                PathBuf::from(shadow).join("packed-refs").as_path(),
                format_packed_refs(&refs, target_ref).as_slice(),
            )
            .map_err(packed_refs_failure)?;
        }
        RefFormat::Reftable => {
            let mut lines: Vec<u8> = Vec::new();
            for entry in &refs {
                if !(entry.symref.is_empty() && entry.name.as_slice() != target_ref.as_bytes()) {
                    continue;
                }
                lines.extend_from_slice(b"create ");
                lines.extend_from_slice(entry.name.as_slice());
                lines.push(b' ');
                lines.extend_from_slice(entry.oid.as_bytes());
                lines.push(b'\n');
            }
            let mut request: GitRequest = shadow_request(shadow, shadow, &["update-ref", "--stdin"]);
            request.input = Some(lines);
            run_git_checked(context, &request)?;
        }
    }
    for entry in &refs {
        if entry.symref.is_empty() {
            continue;
        }
        run_git_checked(
            context,
            &shadow_request(
                shadow,
                shadow,
                &[
                    OsString::from("symbolic-ref"),
                    os_from_path_bytes(entry.name.as_slice()),
                    os_from_path_bytes(entry.symref.as_slice()),
                ],
            ),
        )?;
    }
    return write_shadow_head(context, shadow, symbolic_head, base);
}

/// Shadow ref controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_refs_tests.rs"]
mod tests;
