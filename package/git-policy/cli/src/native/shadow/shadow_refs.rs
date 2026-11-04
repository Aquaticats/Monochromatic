//! What:
//!  The shadow ref store: a snapshot of every real ref,
//!       the private target ref,
//!       symbolic refs,
//!       and the private `HEAD`.
//! Why:
//!  Hooks resolve tags, other branches and remote-tracking refs as they stood at invocation.
//!  With the files backend the snapshot is one `packed-refs` file and the private target is a loose ref,
//!  which takes precedence over a packed entry;
//!  with reftable the snapshot is one `update-ref --stdin` transaction
//!  (`src/shadow-repository/shadow-refs.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeShadowRefs({ gitPath, cwd, shadowPath, capture });
//! ```

/// One shadow Git start, one real Git start, and their failure text.
use crate::shadow_git::shadow_request;
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
/// Private file writes for the snapshot, published whole and not synced.
use crate::private_storage::write_private_file_unsynced;
/// Journal enums: the ref backend, the base and the symbolic `HEAD` at invocation.
use crate::transaction_journal::{Base, RefFormat, SymbolicHead};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Header Git writes for a sorted `packed-refs` file without peeled lines.
///  The trailing space matches Git's own writer, which older readers require.
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
    /// Full ref name.
    pub name: String,
    /// Object the ref resolves to.
    pub oid: String,
    /// Symbolic target, or empty for a direct ref.
    pub symref: String,
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
    let text: &str = std::str::from_utf8(output.stdout.as_slice())
        .map_err(|_| { return GitFailure(String::from("git for-each-ref returned non-UTF-8 output.")); })?;
    let mut refs: Vec<RefSnapshotEntry> = Vec::new();
    for line in text.split('\n').filter(|line| { return !line.is_empty(); }) {
        let fields: Vec<&str> = line.split('\0').collect();
        let [name, oid, symref] = fields.as_slice() else {
            return Err(GitFailure(format!(
                "git for-each-ref returned a malformed line: {line:?}"
            )));
        };
        refs.push(RefSnapshotEntry {
            name: String::from(*name),
            oid: String::from(*oid),
            symref: String::from(*symref),
        });
    }
    refs.sort_by(|left, right| { return left.name.as_bytes().cmp(right.name.as_bytes()); });
    return Ok(refs);
}

/// What:
///  Formats the files-backend snapshot of every direct ref except the private target.
/// Why:
///   The target is written loose, so a packed copy would only be shadowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// formatPackedRefs({ refs, targetRef });
/// ```
pub fn format_packed_refs(refs: &[RefSnapshotEntry], target_ref: &str) -> String {
    let mut text: String = String::from(PACKED_REFS_HEADER);
    for entry in refs
        .iter()
        .filter(|entry| { return entry.symref.is_empty() && entry.name != target_ref; })
    {
        text.push_str(format!("{} {}\n", entry.oid, entry.name).as_str());
    }
    return text;
}

/// What:
///  Points the shadow `HEAD` at the private target:
///  symbolic to the branch, or detached at the base.
/// Why:
///   A detached `HEAD` needs a base, and an unborn branch has no object to write.
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
///  the snapshot, the symbolic refs, the private target ref and `HEAD`.
/// Why:
///   The order matches the incumbent's:
///  symbolic refs come after their targets exist, and `HEAD` comes last.
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
                format_packed_refs(&refs, target_ref).as_bytes(),
            )
            .map_err(|error| { return GitFailure(format!("writing packed-refs failed: {error}")); })?;
        }
        RefFormat::Reftable => {
            let mut lines: String = String::new();
            for entry in refs
                .iter()
                .filter(|entry| { return entry.symref.is_empty() && entry.name != target_ref; })
            {
                lines.push_str(format!("create {} {}\n", entry.name, entry.oid).as_str());
            }
            let mut request: GitRequest = shadow_request(shadow, shadow, &["update-ref", "--stdin"]);
            request.input = Some(lines.into_bytes());
            run_git_checked(context, &request)?;
        }
    }
    for entry in refs.iter().filter(|entry| { return !entry.symref.is_empty(); }) {
        run_git_checked(
            context,
            &shadow_request(shadow, shadow, &["symbolic-ref", entry.name.as_str(), entry.symref.as_str()]),
        )?;
    }
    return write_shadow_head(context, shadow, symbolic_head, base);
}

/// Shadow ref controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_refs_tests.rs"]
mod tests;
