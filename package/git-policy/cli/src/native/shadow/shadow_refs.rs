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
//! Three rules keep the answer Git's answer rather than a set nearly like it,
//! and each was a case a review of this reader found the code getting wrong:
//!  a linked worktree's per-worktree namespaces come only from its own Git directory,
//! so the shared directory's copies of them are left out;
//!  a symbolic ref is printed with the name its chain ends at,
//! not with the name it points at directly;
//!  and a symbolic ref whose chain ends at no ref,
//! or in a loop,
//! is not printed at all.
//! `the_reader_answers_every_kind_of_store_as_git_does_or_declines` holds all three.
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
    // A worktree that does not own its Git directory resolves the per-worktree namespaces to its own,
    // so the shared directory's copies of them are another worktree's refs and are not this snapshot's.
    let linked: bool = sources.git_dir != sources.common_dir;
    let shared: Refs = Refs::new(sources.common_dir.as_path(), hex_length);
    // The entries are built once and sorted once: a name is bytes, and the entry holds those bytes, so the list is
    //  made in place rather than through a table keyed by a copy of every name.
    let mut entries: Vec<RefSnapshotEntry> = Vec::new();
    for (name, value) in shared.list()? {
        if linked && is_a_per_worktree_name(name.as_str()) {
            continue;
        }
        entries.push(entry_of(name.as_str(), value));
    }
    // A worktree that does not own its Git directory keeps these three namespaces in its own, never in `packed-refs`.
    if linked {
        let own: Refs = Refs::new(sources.git_dir.as_path(), hex_length);
        for namespace in PER_WORKTREE_NAMESPACES {
            for (name, value) in own.loose_namespace(namespace)? {
                entries.push(entry_of(name.as_str(), value));
            }
        }
    }
    // Both halves are sorted by name already, and the worktree's three namespaces are disjoint from the shared list,
    //  so one sort of the whole list is what Git's byte order needs and no name is ever replaced.
    entries.sort_by(compare_by_name_bytes);
    let mut dropped: Vec<bool> = vec![false; entries.len()];
    for index in 0..entries.len() {
        if entries[index].symref.is_empty() {
            continue;
        }
        // Git prints a symbolic ref with the ref its chain ends at, and prints none when that chain ends nowhere.
        match end_of_a_symbolic_chain(entries.as_slice(), entries[index].symref.as_slice()) {
            Some((end, oid)) => {
                entries[index].symref = end;
                entries[index].oid = oid;
            }
            None => dropped[index] = true,
        }
    }
    let mut kept: Vec<RefSnapshotEntry> = Vec::with_capacity(entries.len());
    for (index, entry) in entries.into_iter().enumerate() {
        if !dropped[index] {
            kept.push(entry);
        }
    }
    return Some(kept);
}

/// What:
///  Whether a ref name lives in one of the namespaces Git keeps in a worktree's own Git directory.
/// Why:
///   Git resolves such a name to the worktree's own directory and never to the shared one,
/// so a shared copy of it is another worktree's ref.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isPerWorktreeName = (name) => PER_WORKTREE.some((ns) => name.startsWith(`${ns}/`))
/// ```
fn is_a_per_worktree_name(name: &str) -> bool {
    for namespace in PER_WORKTREE_NAMESPACES {
        if name.len() > namespace.len() && name.starts_with(namespace) && name.as_bytes()[namespace.len()] == b'/' {
            return true;
        }
    }
    return false;
}

/// What:
///  The direct ref a symbolic ref's chain ends at,
///  as its name and its object,
///  or `None` when the chain ends at no ref or in a loop.
/// Why:
///  `for-each-ref` prints neither a ref that points at a name that does not exist nor one that points back into its own chain,
/// and the name it prints beside a symbolic ref is the end of the chain,
///  so a two-step chain is printed with the ref it finally names.
///  The depth is Git's own `SYMREF_MAXDEPTH` (5),
/// so a chain Git refuses to follow is dropped here too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const endOfChain = (chains, target) => { let name = target; for (let step = 0; step < 5; step++) { const entry = chains.get(name); if (!entry) return undefined; if (entry.symref === '') return [name, entry.oid]; name = entry.symref } return undefined }
/// ```
/// What:
///  Where a ref name sits in the snapshot's own byte order,
///  or `None` when the snapshot holds no such name.
/// Why:
///   The entries are sorted by name once,
/// and a symbolic ref's every step is a search of that order,
///  written here as the search itself because the snapshot's names are what is compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const findByName = (entries, name) => { let low = 0, high = entries.length; while (low < high) { const middle = low + ((high - low) >> 1); const order = compare(entries[middle].name, name); if (order < 0) low = middle + 1; else if (order > 0) high = middle; else return middle } return undefined }
/// ```
fn find_by_name(entries: &[RefSnapshotEntry], name: &[u8]) -> Option<usize> {
    let mut low: usize = 0;
    let mut high: usize = entries.len();
    while low < high {
        let middle: usize = low.saturating_add(high.saturating_sub(low) / 2);
        match entries[middle].name.as_slice().cmp(name) {
            std::cmp::Ordering::Less => low = middle.saturating_add(1),
            std::cmp::Ordering::Greater => high = middle,
            std::cmp::Ordering::Equal => return Some(middle),
        }
    }
    return None;
}

fn end_of_a_symbolic_chain(entries: &[RefSnapshotEntry], target: &[u8]) -> Option<(Vec<u8>, String)> {
    let mut current: Vec<u8> = target.to_vec();
    for _step in 0..SYMBOLIC_REF_CHAIN_LIMIT {
        // The entries are in Git's byte order, so a step of the chain is one binary search and allocates nothing.
        let found: usize = find_by_name(entries, current.as_slice())?;
        if entries[found].symref.is_empty() {
            return Some((entries[found].name.clone(), entries[found].oid.clone()));
        }
        current = entries[found].symref.clone();
    }
    return None;
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
