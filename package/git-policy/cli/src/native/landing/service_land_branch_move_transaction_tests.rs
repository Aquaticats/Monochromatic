//! What:
//!  Controls for the two command lines a landing writes for its branch.
//! Why:
//!  A name that is not text must reach Git byte for byte,
//! and the grammar around it must stay the grammar Git reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(referenceMoveTransaction(refName(RAW), 'b', 'a', 'delete x\n')).toEqual(Buffer.concat([...]))
//! ```

/// The lines under test.
use super::{branch_commit_question, reference_move_transaction};
/// A reference name as the bytes Git printed it.
use crate::ref_name::RefName;

/// The name of the branch one control moves,
/// with a byte that is not text in it.
const RAW_NAME: &[u8] = b"refs/heads/topic-\xff";

/// What:
///  The question names the branch by its own bytes,
/// and asks in the one spelling the in-process readers answer.
/// Why:
///   A lossy spelling would ask about another branch,
/// and a bare name would start Git instead of the reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(branchCommitQuestion(refName(RAW)).at(-1)).toEqual(RAW + '^{commit}')
/// ```
#[test]
fn the_branch_question_names_the_branch_by_its_bytes() {
    let reference: RefName = RefName::from_git_output(RAW_NAME);
    let words: [std::ffi::OsString; 4] = branch_commit_question(&reference);
    assert_eq!(crate::path_bytes::argument_bytes(words[3].as_os_str()), b"refs/heads/topic-\xff^{commit}");
    assert_eq!(crate::path_bytes::argument_bytes(words[0].as_os_str()), b"rev-parse");
}

/// What:
///  The transaction moves the branch by its own bytes,
/// and its grammar is exactly Git's.
/// Why:
///   The one write moves the branch and removes the side reference;
/// a name spelled with replacement characters would move another branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(referenceMoveTransaction(refName(RAW), 'b', 'a', 'delete s\n')).toEqual(Buffer.from('start\nupdate refs/heads/topic-\xff b a\ndelete s\nprepare\ncommit\n'))
/// ```
#[test]
fn the_transaction_moves_the_branch_by_its_bytes() {
    let reference: RefName = RefName::from_git_output(RAW_NAME);
    let text: Vec<u8> = reference_move_transaction(&reference, "b", "a", "delete s\n");
    let mut expected: Vec<u8> = b"start\nupdate ".to_vec();
    expected.extend_from_slice(RAW_NAME);
    expected.extend_from_slice(b" b a\ndelete s\nprepare\ncommit\n");
    assert_eq!(text, expected);
}
