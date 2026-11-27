//! What:
//!  The ref storage backend,
//!  the commit a ref names at invocation,
//!  and the empty tree an unborn ref starts from.
//! Why:
//!  The preparation base is the target's commit read after the layout,
//!  so it is never older than the sequence the transaction records;
//!  an unborn target starts from the empty tree.
//!  The ref storage and the empty tree are read without a Git process when a fact already proves them:
//!  a configuration snapshot without `extensions.refStorage` proves `files`,
//!  and the empty tree is a constant per object format.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const base = await resolveRefCommit({ ref });
//! const format = snapshot?.provesFilesRefStorage() ? 'files' : await resolveRefFormat();
//! ```

/// The questions of this capture:
///  the decoded answer,
///  the answer a nonzero exit does not refuse,
///  and the answer whose failure is an error.
use crate::capture_invocation_git_questions::{
    decoded_trimmed,
    probe,
    required,
};
/// The one configuration read the invocation's consumers share.
use crate::config_snapshot::ConfigSnapshot;
/// The empty tree's name per object format,
/// known without a Git process.
use crate::empty_tree::empty_tree_name;
/// A reference name as the bytes Git printed it.
use crate::ref_name::RefName;
/// The transaction Git wrapper:
///  the context a question is asked in,
///  the failure its answer is reported as,
///  and the output it prints.
use crate::transaction_git::{
    GitContext,
    GitFailure,
    GitOutput,
};
/// Journal enums shared with the transaction record:
///  the base and the ref backend.
use crate::transaction_journal::{
    Base,
    RefFormat,
};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// What:
///  Resolves a ref to its commit,
///  or to an unborn base when it names nothing.
/// Why:
///   The preparation base is the target's commit at invocation,
///  and an unborn target starts from the empty tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resolveRefCommit({ gitPath, cwd, ref });
/// ```
pub fn resolve_ref_commit(
    context: &GitContext,
    cwd: &Path,
    reference: &RefName,
) -> Result<Base, GitFailure> {
    // The question is the one spelling the in-process readers answer, `<ref>^{commit}`, and both halves are bytes.
    let mut peeled: OsString = reference.as_os_str().to_os_string();
    peeled.push("^{commit}");
    let output: GitOutput = probe(
        context,
        cwd,
        &[
            OsString::from("rev-parse"),
            OsString::from("--verify"),
            OsString::from("--quiet"),
            peeled,
        ],
    )?;
    if output.code != Some(0) {
        return Ok(Base::Unborn);
    }
    return Ok(Base::Commit(decoded_trimmed(&output)?));
}

/// What:
///  Reads the ref storage backend,
/// from the configuration snapshot when it proves `files`.
/// Why:
///   `rev-parse --show-ref-format` prints `reftable` only for a repository whose configuration names
///  `extensions.refStorage`,
/// so a snapshot read without global options that has no such key proves `files`
///  without starting Git;
/// any other snapshot,
/// and a missing one,
/// asks Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot?.provesFilesRefStorage() ? 'files' : await resolveRefFormat({ gitPath, cwd });
/// ```
pub fn resolve_ref_format_with(
    context: &GitContext,
    cwd: &Path,
    snapshot: Option<&ConfigSnapshot>,
) -> Result<RefFormat, GitFailure> {
    if snapshot.is_some_and(ConfigSnapshot::proves_files_ref_storage) {
        return Ok(RefFormat::Files);
    }
    let output: GitOutput = probe(context, cwd, &["rev-parse", "--show-ref-format"])?;
    let reftable: bool = output.code == Some(0) && decoded_trimmed(&output)? == "reftable";
    return Ok(if reftable {
        RefFormat::Reftable
    } else {
        RefFormat::Files
    });
}

/// What:
///  The empty tree's object name under the repository's object format.
/// Why:
///   The name is a constant per object format,
/// so a known format needs no Git process;
///  a format the table does not know is hashed by Git,
/// which stays the authority.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// emptyTreeName(objectFormat) ?? (await runTransactionGit({ args: ['hash-object', '-t', 'tree', '--stdin'] })).trim()
/// ```
pub(crate) fn resolve_empty_tree(
    context: &GitContext,
    cwd: &Path,
    object_format: &str,
) -> Result<String, GitFailure> {
    if let Some(known) = empty_tree_name(object_format) {
        return Ok(String::from(known));
    }
    let hashed: GitOutput = required(context, cwd, &["hash-object", "-t", "tree", "--stdin"])?;
    return decoded_trimmed(&hashed);
}
