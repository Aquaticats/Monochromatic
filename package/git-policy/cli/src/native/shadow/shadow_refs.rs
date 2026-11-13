//! What:
//!  The real refs a shadow's ref store is written from:
//!  every ref of the worktree at invocation,
//!  read from the ref files when they can be read and from one Git child otherwise.
//! Why:
//!  Hooks inside the shadow resolve tags,
//! other branches and remote-tracking refs as they stood at invocation,
//!  so the snapshot has to be every ref Git would print.
//! `git for-each-ref` answers that,
//! and it is the whole repository's answer:
//! 199 KiB for this repository's 1,982 packed and 40 loose refs,
//! and as a child of a commit it took 3.8 to 26.4 seconds on 2026-11-12 while
//!  the same command alone took 4 to 5 ms.
//! The package's own reader of Git's ref files answers the same question without
//!  starting anything,
//! and every layout it cannot prove (a reftable store,
//! an unknown object format,
//! a ref name it declines,
//! a damaged `packed-refs`) ends in the Git child,
//!  so a decline adds a start and changes no answer.
//! The writer of the store is `shadow_ref_write.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await listRealRefs({ gitPath,
//! cwd });
//! // or the ref files,
//! when they can be read
//! ```

/// One real Git start,
/// and its failure text.
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
/// The package's own reader of Git's ref files,
/// which answers without a Git start.
use crate::repo_read_refs::{RefValue, Refs};
/// A table keyed by a ref name's bytes,
/// which keeps the names in Git's own order.
use std::collections::BTreeMap;
/// The ref backend,
/// which decides whether files or tables hold the refs.
use crate::transaction_journal::RefFormat;
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

/// Which of Git's namespaces a linked worktree keeps in its own Git directory rather than in the shared one.
///  Git's own table is `parse_worktree_ref` (`refs.c`):
/// these three are per-worktree,
/// every other name is shared,
/// and `git pack-refs` never packs these three.
pub const PER_WORKTREE_NAMESPACES: [&str; 3] = ["refs/bisect", "refs/worktree", "refs/rewritten"];

/// What:
///  Where a ref snapshot is read from:
///  the two Git directories of a worktree,
///  the ref backend and the width of an object name.
/// Why:
///   The shared refs live in the common directory and the per-worktree namespaces in the worktree's
/// own,
///  so one snapshot needs both paths beside the backend that decides whether files or tables hold them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SnapshotSources = { gitDir: string; commonDir: string; refFormat: RefFormat; hexLength?: number };
/// ```
#[derive(Clone, Debug)]
pub struct SnapshotSources {
    /// Absolute Git directory of the owning worktree.
    pub git_dir: PathBuf,
    /// Absolute common Git directory.
    pub common_dir: PathBuf,
    /// Ref storage backend.
    pub ref_format: RefFormat,
    /// The number of hexadecimal digits of an object name,
    /// or `None` for an object format no reader here knows,
    ///  which sends the snapshot to Git.
    pub hex_length: Option<usize>,
}

impl SnapshotSources {
    /// What:
    ///  The sources of one worktree of a repository whose object format Git named.
    /// Why:
    ///   A caller holds the paths and the format names as Git printed them,
    /// and the width of an object name is derived once here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// SnapshotSources.of(gitDir, commonDir, refFormat, objectFormat)
    /// ```
    pub fn of(git_dir: PathBuf, common_dir: PathBuf, ref_format: RefFormat, object_format: &str) -> SnapshotSources {
        return SnapshotSources { git_dir, common_dir, ref_format, hex_length: crate::ref_files::hex_length(object_format) };
    }
}

/// What:
///  One ref of the reader as a snapshot entry,
///  with the object name of a symbolic ref filled in later.
/// Why:
///   The reader answers a value in two shapes and the snapshot keeps one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entryOf = (name: string, value: RefValue): RefSnapshotEntry
/// ```
fn entry_of(name: &str, value: RefValue) -> RefSnapshotEntry {
    // A name is bytes in a ref store, and the reader hands it as text only where it is text.
    let bytes: Vec<u8> = name.as_bytes().to_vec();
    return match value {
        RefValue::Direct(oid) => RefSnapshotEntry { name: bytes, oid, symref: Vec::new() },
        RefValue::Symbolic(target) => RefSnapshotEntry { name: bytes, oid: String::new(), symref: target.into_bytes() },
    };
}

/// What:
///  Every real ref of a worktree,
///  read from the ref files,
///  or `None` when only Git can say what it would print.
/// Why:
///   `git for-each-ref` over every ref is the whole repository's answer:
/// 199 KiB for the 1,982 packed and 40 loose refs of this repository,
/// and as a child of a commit it took 3.8 to 26.4 seconds on 2026-11-12 while real Git answered the
///  same question in 4 to 5 milliseconds
/// (`doc/troubleshooting/cli-git-one-child-of-a-commit-takes-seconds.md`).
///  The reader here is the package's own,
/// which the policy queries,
/// the automatic push and the worktree clean already use;
/// every layout it cannot prove (an unknown object format,
/// a reftable store,
/// a ref name it declines,
///  a damaged `packed-refs`) is `None`,
/// and the caller then starts Git as before,
/// so a decline adds a Git start and changes no result.
///  A symbolic ref's object name is the one its chain ends at,
/// which is what Git prints beside it,
/// and a chain that ends nowhere is printed with none,
/// as Git prints it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refsReadFromFiles = (sources) => sharedRefs.list() + perWorktreeNamespaces;
/// ```
fn chain_entry(entry: &RefSnapshotEntry) -> (Vec<u8>, (String, Vec<u8>)) {
    return (entry.name.clone(), (entry.oid.clone(), entry.symref.clone()));
}

/// What:
///  One entry of the snapshot read from the ref files,
///  or `None` when the reader cannot prove the answer.
/// Why:
///   Every refusal is `None` here and a Git start there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refsReadFromFiles = (sources) => sharedRefs.list() + perWorktreeNamespaces;
/// ```
fn refs_read_from_files(sources: &SnapshotSources) -> Option<Vec<RefSnapshotEntry>> {
    let hex_length: usize = sources.hex_length?;
    if sources.ref_format != RefFormat::Files {
        return None;
    }
    let shared: Refs = Refs::new(sources.common_dir.as_path(), hex_length);
    let mut found: BTreeMap<Vec<u8>, RefSnapshotEntry> = BTreeMap::new();
    for (name, value) in shared.list()? {
        let entry: RefSnapshotEntry = entry_of(name.as_str(), value);
        found.insert(entry.name.clone(), entry);
    }
    // A worktree that does not own its Git directory keeps these three namespaces in its own, never in `packed-refs`.
    if sources.git_dir != sources.common_dir {
        let own: Refs = Refs::new(sources.git_dir.as_path(), hex_length);
        for namespace in PER_WORKTREE_NAMESPACES {
            for (name, value) in own.loose_namespace(namespace)? {
                let entry: RefSnapshotEntry = entry_of(name.as_str(), value);
                found.insert(entry.name.clone(), entry);
            }
        }
    }
    let mut entries: Vec<RefSnapshotEntry> = found.into_values().collect();
    let chains: BTreeMap<Vec<u8>, (String, Vec<u8>)> = entries
        .iter()
        .map(chain_entry)
        .collect();
    for entry in &mut entries {
        if entry.symref.is_empty() {
            continue;
        }
        // `refs/remotes/origin/HEAD` and a branch that is a symbolic ref both end at a direct ref or nowhere.
        let mut current: Vec<u8> = entry.symref.clone();
        for _step in 0..SYMBOLIC_REF_CHAIN_LIMIT {
            let Some((oid, symref)) = chains.get(&current) else {
                break;
            };
            if symref.is_empty() {
                entry.oid = oid.clone();
                break;
            }
            current = symref.clone();
        }
    }
    return Some(entries);
}

/// What:
///  Lists every real ref of the owning worktree in byte order,
/// from the ref files when they can be read,
///  and from one Git child otherwise.
/// Why:
///   The answer is the snapshot the shadow's ref store is written from;
/// who answers it decides whether a commit starts Git over every ref of the
///  repository at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refs = refsReadFromFiles(sources) ?? listRealRefs({ gitPath, cwd });
/// ```
pub fn list_real_refs_from(context: &GitContext, cwd: &Path, sources: &SnapshotSources) -> Result<Vec<RefSnapshotEntry>, GitFailure> {
    if let Some(read) = refs_read_from_files(sources) {
        return Ok(read);
    }
    return list_real_refs(context, cwd);
}

/// What:
///  Lists every real ref of the owning worktree in byte order,
/// from one `git for-each-ref`.
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

/// How many symbolic refs the private target is followed through.
///  Git's own limit for a chain of symbolic refs is five,
/// and a chain that does not end within it is refused here as Git refuses it.
pub const SYMBOLIC_REF_CHAIN_LIMIT: usize = 5;
