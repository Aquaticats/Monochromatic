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
/// The package's own reader of Git's ref files, which answers without a Git start.
use crate::repo_read_refs::{RefValue, Refs};
/// A table keyed by a ref name's bytes,
/// which keeps the names in Git's own order.
use std::collections::BTreeMap;
/// Private file writes for the snapshot,
/// published whole and not synced.
use crate::private_storage::write_private_file_unsynced;
/// Journal enums:
/// the ref backend,
/// the base and the symbolic `HEAD` at invocation.
use crate::transaction_journal::{Base, RefFormat, SymbolicHead};
/// A reference name as the bytes Git printed it.
use crate::ref_name::RefName;
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
        .map(|entry| { return (entry.name.clone(), (entry.oid.clone(), entry.symref.clone())); })
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
pub fn format_packed_refs(refs: &[RefSnapshotEntry], target_ref: &RefName) -> Vec<u8> {
    let mut text: Vec<u8> = PACKED_REFS_HEADER.as_bytes().to_vec();
    for entry in refs {
        if !(entry.symref.is_empty() && entry.name.as_slice() != target_ref.bytes()) {
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
///  The bytes of a direct ref's file:
/// the object name and a line feed.
/// Why:
///   That is the whole of a loose ref on the files backend,
///  so a ref the wrapper writes is the ref Git reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(`${oid}\n`)
/// ```
fn direct_ref_value(oid: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = oid.as_bytes().to_vec();
    bytes.push(b'\n');
    return bytes;
}

/// What:
///  The bytes of a symbolic ref's file:
/// `ref:` ,
///  the target and a line feed.
/// Why:
///   The target is bytes,
/// because a reference name is,
///  and a name that is not text reaches the shadow unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.concat([Buffer.from('ref: '), target, Buffer.from('\n')])
/// ```
fn symbolic_ref_value(target: &[u8]) -> Vec<u8> {
    let mut bytes: Vec<u8> = b"ref: ".to_vec();
    bytes.extend_from_slice(target);
    bytes.push(b'\n');
    return bytes;
}

/// What:
///  Writes one loose ref of a shadow,
///  replacing what the name already holds.
/// Why:
///   A second attempt of the same transaction finds what the first one wrote,
/// and a shadow's own `HEAD` is written by the skeleton before this,
///  so a ref write replaces.
///  The parents are made only when the write finds none:
/// a snapshot ref such as `refs/remotes/origin/HEAD` lives packed in the real repository,
///  so the shadow can hold no directory for it.
///  The shadow is thrown away when its transaction ends and recovery removes it whole,
/// so a torn ref file changes no outcome and none is synced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(join(shadow, name), bytes).catch(async (error) => { if (error.code !== 'ENOENT') throw error; await mkdir(dirname(path), { recursive: true }); await writeFile(path, bytes); });
/// ```
fn write_loose_ref(shadow: &Path, name: &OsString, value: &[u8]) -> std::io::Result<()> {
    let path: PathBuf = shadow.join(name);
    return match std::fs::write(path.as_path(), value) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let Some(parent) = path.parent() else {
                return Err(error);
            };
            std::fs::create_dir_all(parent)?;
            std::fs::write(path.as_path(), value)
        }
        other => other,
    };
}

/// What:
///  Turns the result of one ref write into the failure a commit reports.
/// Why:
///   Both kinds of loose ref report the same way,
/// and a named function stands where a closure that built
///  that text stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refFailure(error: Error): GitFailure; // new GitFailure(`writing a shadow ref failed: ${error}`)
/// ```
fn ref_failure(error: std::io::Error) -> GitFailure {
    return GitFailure(format!("writing a shadow ref failed: {error}"));
}

/// How many symbolic refs the private target is followed through.
///  Git's own limit for a chain of symbolic refs is five,
/// and a chain that does not end within it is refused here as Git refuses it.
pub const SYMBOLIC_REF_CHAIN_LIMIT: usize = 5;

/// What:
///  The target of a real symbolic ref,
///  or `None` when that name is a direct ref.
/// Why:
///   The walk over a chain asks this once per step;
/// a named function stands where a closure the linter refuses stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const symbolicTargetOf = (refs, name) => refs.find((e) => e.symref && e.name === name)?.symref;
/// ```
fn symbolic_target_of<'refs>(refs: &'refs [RefSnapshotEntry], name: &[u8]) -> Option<&'refs [u8]> {
    for entry in refs {
        if !entry.symref.is_empty() && entry.name.as_slice() == name {
            return Some(entry.symref.as_slice());
        }
    }
    return None;
}

/// What:
///  The name a loose ref write ends at,
///  following the real repository's symbolic refs.
/// Why:
///   `git update-ref <ref> <oid>` follows a symbolic ref and writes the ref it names,
/// and a branch can itself be a symbolic ref to another branch;
/// a direct write at the first name would collapse that chain into a direct ref
///  that a hook could tell apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const targetOf = (refs, name) => { for (let step = 0; step < LIMIT; step++) { const symref = symbolicTargetOf(refs, name); if (!symref) return name; name = symref; } throw new GitFailure(`symbolic ref chain ${name} is too long`); };
/// ```
fn end_of_symbolic_chain(refs: &[RefSnapshotEntry], name: &[u8]) -> Result<Vec<u8>, GitFailure> {
    let mut current: Vec<u8> = name.to_vec();
    for _step in 0..SYMBOLIC_REF_CHAIN_LIMIT {
        let Some(target) = symbolic_target_of(refs, current.as_slice()) else {
            return Ok(current);
        };
        current = target.to_vec();
    }
    return Err(GitFailure(format!(
        "the symbolic ref {} does not end within {SYMBOLIC_REF_CHAIN_LIMIT} steps, as Git refuses one that long",
        String::from_utf8_lossy(name)
    )));
}

/// What:
///  Writes the files backend's `HEAD`,
///  and the private target ref it names.
/// Why:
///   `HEAD` and a loose ref are two small files,
/// and Git reads them directly;
/// the reference update and the symbolic ref that wrote them were two Git starts
///  per commit for a few bytes each.
///  The target is written where the chain of symbolic refs ends,
///  as the `git update-ref` this replaces wrote it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeLooseRef(shadow, endOfChain(refs, target), `${oid}\n`); await writeLooseRef(shadow, 'HEAD', `ref: ${target}\n`);
/// ```
fn write_files_shadow_head(
    shadow: &Path,
    symbolic_head: &SymbolicHead,
    base: &Base,
    refs: &[RefSnapshotEntry],
) -> Result<(), GitFailure> {
    let head: OsString = OsString::from("HEAD");
    match symbolic_head {
        SymbolicHead::Branch(reference) => {
            if let Base::Commit(oid) = base {
                let target: Vec<u8> = end_of_symbolic_chain(refs, reference.bytes())?;
                write_loose_ref(shadow, &os_from_path_bytes(target.as_slice()), direct_ref_value(oid).as_slice()).map_err(ref_failure)?;
            }
            write_loose_ref(shadow, &head, symbolic_ref_value(reference.bytes()).as_slice()).map_err(ref_failure)?;
            return Ok(());
        }
        SymbolicHead::Detached => {
            let Base::Commit(oid) = base else {
                return Err(GitFailure(String::from(
                    "A detached HEAD has no unborn preparation base.",
                )));
            };
            write_loose_ref(shadow, &head, direct_ref_value(oid).as_slice()).map_err(ref_failure)?;
            return Ok(());
        }
    }
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
                    &shadow_request(
                        shadow,
                        shadow,
                        &[OsString::from("update-ref"), reference.as_os_str().to_os_string(), OsString::from(oid)],
                    ),
                )?;
            }
            run_git_checked(
                context,
                &shadow_request(
                    shadow,
                    shadow,
                    &[OsString::from("symbolic-ref"), OsString::from("HEAD"), reference.as_os_str().to_os_string()],
                ),
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
    sources: &SnapshotSources,
    target_ref: &RefName,
    symbolic_head: &SymbolicHead,
    base: &Base,
) -> Result<(), GitFailure> {
    let refs: Vec<RefSnapshotEntry> = list_real_refs_from(context, cwd, sources)?;
    let files: bool = sources.ref_format == RefFormat::Files;
    match sources.ref_format {
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
                if !(entry.symref.is_empty() && entry.name.as_slice() != target_ref.bytes()) {
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
        // A symbolic ref is a file holding `ref: <target>`; the reftable backend keeps its refs in tables only Git writes.
        if files {
            write_loose_ref(
                shadow,
                &os_from_path_bytes(entry.name.as_slice()),
                symbolic_ref_value(entry.symref.as_slice()).as_slice(),
            )
            .map_err(ref_failure)?;
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
    if files {
        return write_files_shadow_head(shadow, symbolic_head, base, &refs);
    }
    return write_shadow_head(context, shadow, symbolic_head, base);
}

/// Shadow ref controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_refs_tests.rs"]
mod tests;
