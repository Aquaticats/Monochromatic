//! Controls for the attribute reading:
//! a file that grows under the read may not hand over a byte the caller's bound never counted.

use super::*;
use crate::worktree_clean_content_attribute_sources::ConversionEnvironment;
use crate::repo_read_layout::ObjectFormat;
use crate::test_support::{fixture, git, remove};
use std::cell::Cell;

/// What:
///  The fixture and the judge a control reads attributes through,
/// with the caller's bound `spent` and the read the control sets.
/// Why:
///   The room an attributes file has is what the bound leaves of `CONTENT_LIMIT`,
/// so a control states the bound and reads the root `.gitattributes` of a repository it wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { root, judge } = fixtureWithBound(spent);
/// ```
fn fixture_root(name: &str, attributes: &[u8]) -> std::path::PathBuf {
    let root: std::path::PathBuf = fixture(name);
    git(root.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(root.join(".gitattributes"), attributes).expect("attributes");
    return root;
}

/// An attributes file grown under the read takes no byte past the room, and every byte it hands over is charged.
#[test]
fn an_attributes_file_that_grows_under_the_read_takes_no_byte_past_the_room() {
    let root: std::path::PathBuf = fixture_root("attributes-grown", b"# eight\n");
    let handed: &Cell<u64> = Box::leak(Box::new(Cell::new(0)));
    let room: u64 = 8;
    let grown: std::path::PathBuf = root.join(".gitattributes");
    let read = move |file: &std::fs::File, buffer: &mut [u8]| -> std::io::Result<usize> {
        // The file grows past the room between the size check and this read.
        std::fs::write(&grown, b"# eight bytes and more\n").expect("grow");
        use std::io::Read as _;
        let taken: usize = (&*file).read(buffer)?;
        handed.set(handed.get() + taken as u64);
        return Ok(taken);
    };
    let spent: Cell<u64> = Cell::new(CONTENT_LIMIT - room);
    let environment: ConversionEnvironment = crate::fixture_content_decision::content_environment();
    let git_dir: std::path::PathBuf = root.join(".git");
    let held: std::fs::File = crate::fs_beneath::open_root(root.as_path()).expect("the worktree opens");
    let mut judge: ContentJudge<'_> = ContentJudge::new(&held, git_dir.as_path(), git_dir.as_path(), &environment, ObjectFormat::Sha1);
    judge.charge_attributes_to(&spent);
    judge.read_attributes_with(&read);
    let outcome: Result<(), String> = judge.conversion_is_the_identity(b"a", &[]);
    assert!(outcome.is_err(), "a file that grew under the read sets no attributes this decision may trust");
    assert!(handed.get() <= room, "the read takes at most the room, and took {}", handed.get());
    assert_eq!(spent.get() - (CONTENT_LIMIT - room), handed.get(), "and every byte it took is charged");
    remove(root.as_path());
}
