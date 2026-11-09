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

/// An attributes file grown under the read takes no byte past the room,
/// and every byte it hands over is charged.
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

/// An attributes file rewritten on the same inode under the read is refused,
/// even when the pieces it handed over add up to the length the file has afterwards:
/// the accumulated bytes then mix two versions,
/// and the rule that survives may be the obsolete one.
#[test]
fn an_attributes_file_rewritten_under_the_read_is_refused() {
    let root: std::path::PathBuf = fixture_root("attributes-rewritten", b"a -text\n");
    let handed: &Cell<u64> = Box::leak(Box::new(Cell::new(0)));
    let step: &Cell<usize> = Box::leak(Box::new(Cell::new(0)));
    let rewritten: std::path::PathBuf = root.join(".gitattributes");
    let read = move |_file: &std::fs::File, buffer: &mut [u8]| -> std::io::Result<usize> {
        // What one version of the file would hand over, piece by piece, while the file itself moves on.
        let piece: &[u8] = match step.get() {
            0 => {
                // The file is rewritten on the same inode and ends up exactly as long as the two pieces together.
                std::fs::write(&rewritten, b"a ident\n# note\n").expect("rewrite");
                b"a -text\n"
            }
            1 => b"# note\n",
            _ => b"",
        };
        step.set(step.get() + 1);
        let taken: usize = piece.len().min(buffer.len());
        buffer[..taken].copy_from_slice(&piece[..taken]);
        handed.set(handed.get() + taken as u64);
        return Ok(taken);
    };
    let spent: Cell<u64> = Cell::new(0);
    let environment: ConversionEnvironment = crate::fixture_content_decision::content_environment();
    let git_dir: std::path::PathBuf = root.join(".git");
    let held: std::fs::File = crate::fs_beneath::open_root(root.as_path()).expect("the worktree opens");
    let mut judge: ContentJudge<'_> = ContentJudge::new(&held, git_dir.as_path(), git_dir.as_path(), &environment, ObjectFormat::Sha1);
    judge.charge_attributes_to(&spent);
    judge.read_attributes_with(&read);
    let outcome: Result<(), String> = judge.conversion_is_the_identity(b"a", &[]);
    assert!(outcome.is_err(), "a file rewritten under the read sets no attributes this decision may trust, and answered {outcome:?}");
    assert_eq!(spent.get(), handed.get(), "every byte the file handed over is charged");
    remove(root.as_path());
}

/// A read the operating system interrupts is tried again,
/// as the recorded-content reader already does,
/// so one interruption does not cost every remaining path of the landing its stat data.
#[test]
fn an_interrupted_attribute_read_is_tried_again() {
    let root: std::path::PathBuf = fixture_root("attributes-interrupted", b"a -text\n");
    let attempts: &Cell<usize> = Box::leak(Box::new(Cell::new(0)));
    let read = move |file: &std::fs::File, buffer: &mut [u8]| -> std::io::Result<usize> {
        attempts.set(attempts.get() + 1);
        if attempts.get() == 1 {
            return Err(std::io::Error::from(std::io::ErrorKind::Interrupted));
        }
        use std::io::Read as _;
        return (&*file).read(buffer);
    };
    let spent: Cell<u64> = Cell::new(0);
    let environment: ConversionEnvironment = crate::fixture_content_decision::content_environment();
    let git_dir: std::path::PathBuf = root.join(".git");
    let held: std::fs::File = crate::fs_beneath::open_root(root.as_path()).expect("the worktree opens");
    let mut judge: ContentJudge<'_> = ContentJudge::new(&held, git_dir.as_path(), git_dir.as_path(), &environment, ObjectFormat::Sha1);
    judge.charge_attributes_to(&spent);
    judge.read_attributes_with(&read);
    let outcome: Result<(), String> = judge.conversion_is_the_identity(b"a", &[]);
    assert!(outcome.is_ok(), "an interrupted read is tried again, so this file's attributes are read, and answered {outcome:?}");
    assert!(attempts.get() > 1, "the read was tried again, and was tried {} time(s)", attempts.get());
    remove(root.as_path());
}

/// What:
///  A judge of `root` whose attributes it reads are charged to a bound of its own,
/// with the clock the settle rule reads pinned for the length of `clock`.
/// Why:
///   The controls state the instant a change time is compared with,
/// so none of them waits for the clock to move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgeAt(root: string, clock: bigint, read?: Read): Result<void, string>;
/// ```
fn identity_at(root: &std::path::Path, clock: i128) -> Result<(), String> {
    let _pin = crate::worktree_copy_stage_leaf_copying::clock_reads(clock);
    let spent: Cell<u64> = Cell::new(0);
    let environment: ConversionEnvironment = crate::fixture_content_decision::content_environment();
    let git_dir: std::path::PathBuf = root.join(".git");
    let held: std::fs::File = crate::fs_beneath::open_root(root).expect("the worktree opens");
    let mut judge: ContentJudge<'_> = ContentJudge::new(&held, git_dir.as_path(), git_dir.as_path(), &environment, ObjectFormat::Sha1);
    judge.charge_attributes_to(&spent);
    return judge.conversion_is_the_identity(b"a", &[]);
}

/// The inode of `metadata`,
/// which a control compares with a descriptor's to know which file it is reading.
fn return_target(metadata: &std::fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt as _;
    return metadata.ino();
}

/// The change time of a path's metadata in nanoseconds since the epoch.
fn changed_nanos(metadata: &std::fs::Metadata) -> i128 {
    use std::os::unix::fs::MetadataExt as _;
    return crate::worktree_copy_stage_leaf_copying::stamp_nanos((metadata.ctime(), metadata.ctime_nsec()));
}

/// An attributes file changed within one timestamp granularity of the clock proves nothing,
/// because a write inside that tick would leave the same change time;
/// one changed a whole granularity earlier is trusted.
#[test]
fn an_attributes_file_changed_within_the_granularity_is_refused_while_a_settled_one_is_trusted() {
    let root: std::path::PathBuf = fixture_root("attributes-settled", b"a -text\n");
    let changed: i128 = changed_nanos(&std::fs::symlink_metadata(root.join(".gitattributes")).expect("metadata"));
    let settled: i128 = changed + crate::worktree_copy_stage_leaf_copying::RECENT_CHANGE_NANOS;
    assert!(identity_at(root.as_path(), settled).is_ok(), "an attributes file changed a whole granularity ago is trusted");
    assert!(identity_at(root.as_path(), changed).is_err(), "one changed at the clock's own instant proves nothing");
    assert!(identity_at(root.as_path(), settled - 1).is_err(), "one nanosecond short of the granularity proves nothing either");
    remove(root.as_path());
}

/// A configured attributes *link* swapped under the read names another file afterwards,
/// which the descriptor's own metadata cannot show:
/// the name is looked at again after the read and must still hold the file that was read.
#[test]
fn a_configured_attributes_link_swapped_under_the_read_is_refused() {
    let root: std::path::PathBuf = fixture_root("attributes-link-swapped", b"# the worktree's own file\n");
    std::fs::write(root.join("one"), b"a -text\n").expect("first target");
    std::fs::write(root.join("two"), b"a ident\n").expect("second target");
    let configured: std::path::PathBuf = root.join("configured");
    std::os::unix::fs::symlink(root.join("one"), configured.as_path()).expect("the link");
    git(root.as_path(), &["config", "core.attributesFile", configured.to_str().expect("a path")]);
    let settled: i128 = changed_nanos(&std::fs::symlink_metadata(root.join("one")).expect("metadata"))
        + crate::worktree_copy_stage_leaf_copying::RECENT_CHANGE_NANOS;
    let _pin = crate::worktree_copy_stage_leaf_copying::clock_reads(settled);
    let swapping: std::path::PathBuf = configured.clone();
    let other: std::path::PathBuf = root.join("two");
    // The configured target, so the control swaps the link only while that file is being read.
    let target: u64 = {
        use std::os::unix::fs::MetadataExt as _;
        return_target(&std::fs::symlink_metadata(root.join("one")).expect("metadata"))
    };
    let read = move |file: &std::fs::File, buffer: &mut [u8]| -> std::io::Result<usize> {
        use std::os::unix::fs::MetadataExt as _;
        if file.metadata().map(|metadata| metadata.ino()).unwrap_or(u64::MAX) == target {
            // The link is replaced while the file it named is being read.
            std::fs::remove_file(&swapping).expect("remove the link");
            std::os::unix::fs::symlink(&other, &swapping).expect("replace the link");
        }
        use std::io::Read as _;
        return (&*file).read(buffer);
    };
    let spent: Cell<u64> = Cell::new(0);
    let environment: ConversionEnvironment = crate::fixture_content_decision::content_environment();
    let git_dir: std::path::PathBuf = root.join(".git");
    let held: std::fs::File = crate::fs_beneath::open_root(root.as_path()).expect("the worktree opens");
    let mut judge: ContentJudge<'_> = ContentJudge::new(&held, git_dir.as_path(), git_dir.as_path(), &environment, ObjectFormat::Sha1);
    judge.charge_attributes_to(&spent);
    judge.read_attributes_with(&read);
    assert!(judge.conversion_is_the_identity(b"a", &[]).is_err(), "the configured name holds another file after the read");
    remove(root.as_path());
}

/// A read the system interrupts for ever ends in the same refusal a hard read error does,
/// after a counted number of tries:
/// nothing waits for an interruption to stop.
#[test]
fn repeatedly_interrupted_attribute_reads_end_in_a_refusal() {
    let root: std::path::PathBuf = fixture_root("attributes-interrupted-forever", b"a -text\n");
    let attempts: &Cell<usize> = Box::leak(Box::new(Cell::new(0)));
    let read = move |_file: &std::fs::File, _buffer: &mut [u8]| -> std::io::Result<usize> {
        attempts.set(attempts.get() + 1);
        // The control stops the read itself, so a reader with no bound cannot hang the run.
        if attempts.get() > 10_000 {
            return Ok(0);
        }
        return Err(std::io::Error::from(std::io::ErrorKind::Interrupted));
    };
    let spent: Cell<u64> = Cell::new(0);
    let environment: ConversionEnvironment = crate::fixture_content_decision::content_environment();
    let git_dir: std::path::PathBuf = root.join(".git");
    let held: std::fs::File = crate::fs_beneath::open_root(root.as_path()).expect("the worktree opens");
    let mut judge: ContentJudge<'_> = ContentJudge::new(&held, git_dir.as_path(), git_dir.as_path(), &environment, ObjectFormat::Sha1);
    judge.charge_attributes_to(&spent);
    judge.read_attributes_with(&read);
    let outcome: Result<(), String> = judge.conversion_is_the_identity(b"a", &[]);
    assert!(outcome.is_err(), "a read that never gets through refuses the attributes, and answered {outcome:?}");
    assert!(attempts.get() <= 3, "a bounded number of tries, and the read was tried {} times", attempts.get());
    remove(root.as_path());
}
