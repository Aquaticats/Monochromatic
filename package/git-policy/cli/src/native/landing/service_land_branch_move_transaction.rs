//! What:
//!  The two command lines a landing writes for its branch:
//! the question that reads the commit it names,
//! and the transaction that moves it.
//! Why:
//!   A reference name is bytes,
//! so both are built as bytes:
//!  a name spelled with replacement characters would read and move another ref.
//!  The grammar around the name is the wrapper's own text,
//! and the task statement of `AGENTS.md` rules `SYB` and `STB` is why the name is never
//!  spelled into it.
//!  They stand here so the landing's body states what it does and not how a line is laid out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const words = branchCommitQuestion(ref); const text = referenceMoveTransaction({ ref, new, old, removal });
//! ```

/// A reference name as the bytes Git printed it.
use crate::ref_name::RefName;
/// One command line is made of operating-system strings.
use std::ffi::OsString;

/// What:
///  The words of `git rev-parse --verify --quiet <ref>^{commit}`.
/// Why:
///   The one spelling the in-process readers answer is `<ref>^{commit}`,
/// so the question starts no Git;
///  for a branch,
/// which only ever names a commit,
/// the answer is the same as for the bare name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['rev-parse', '--verify', '--quiet', ref + '^{commit}']
/// ```
pub(crate) fn branch_commit_question(reference: &RefName) -> [OsString; 4] {
    let mut peeled: OsString = reference.as_os_str().to_os_string();
    peeled.push("^{commit}");
    return [OsString::from("rev-parse"), OsString::from("--verify"), OsString::from("--quiet"), peeled];
}

/// What:
///  The transaction one `git update-ref --stdin` reads to move the branch,
///  removing the batch's scratch reference in the same transaction.
/// Why:
///   The branch move and the removal of the side reference are one write,
/// so a kill leaves either both or neither;
///  the name is bytes and the rest of the grammar is the wrapper's own text.
///  The message of the commit never enters this input:
/// it is one argument of `-m`,
/// so no byte of it can change the grammar Git reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.concat([Buffer.from('start\nupdate '), ref, Buffer.from(` ${new} ${old}\n${removal}prepare\ncommit\n`)])
/// ```
pub(crate) fn reference_move_transaction(reference: &RefName, new: &str, old: &str, removal: &str) -> Vec<u8> {
    let mut text: Vec<u8> = b"start\nupdate ".to_vec();
    text.extend_from_slice(reference.bytes());
    text.extend_from_slice(format!(" {new} {old}\n{removal}prepare\ncommit\n").as_bytes());
    return text;
}

/// Transaction controls stay out of the release executable.
#[cfg(test)]
#[path = "service_land_branch_move_transaction_tests.rs"]
mod tests;
