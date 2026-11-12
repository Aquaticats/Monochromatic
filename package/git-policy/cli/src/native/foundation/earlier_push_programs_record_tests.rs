//! Controls for the removal of the record an unreleased branch wrote:
//! the record and the staging files of its writes go,
//! and nothing else in the folder does.

use super::*;

/// A Git directory of this control's own,
/// with the folder that record lived in.
fn git_directory_with_the_folder(name: &str) -> PathBuf {
    let git_dir: PathBuf = crate::test_support::fixture_with_prefix("native-earlier-record", name);
    std::fs::create_dir_all(git_dir.join(FOLDER)).expect("the folder the work records use");
    return git_dir;
}

/// What:
///  The record and every staging file of one of its writes are removed.
/// Why:
///   Both hold the values that can carry a password,
/// and a command that left either behind would keep a credential on the disk for ever.
#[test]
fn the_record_and_its_staging_files_are_removed() {
    let git_dir: PathBuf = git_directory_with_the_folder("both");
    let record: PathBuf = git_dir.join(FOLDER).join(FILE);
    let staged: PathBuf = git_dir.join(FOLDER).join(format!(".{FILE}.Xk2Qa.partial"));
    let staged_again: PathBuf = git_dir.join(FOLDER).join(format!(".{FILE}.7fF0b.partial"));
    for path in [&record, &staged, &staged_again] {
        std::fs::write(path, b"{\"v\":1,\"variables\":{\"PATH\":\"/fake/bin\"}}\n").expect("a file of that branch's own shape");
    }
    remove_what_an_earlier_branch_wrote(git_dir.as_path());
    for path in [&record, &staged, &staged_again] {
        assert!(!path.exists(), "{} is gone after the first command that meets it", path.display());
    }
}

/// What:
///  The work records and the notices beside the record stay,
///  and so does a directory that carries the record's name.
/// Why:
///   The folder is this release's own;
/// a removal that emptied it would take a person's queued pushes with it,
/// and a directory of that name is not the file that branch wrote.
#[test]
fn nothing_else_in_the_folder_is_touched() {
    let git_dir: PathBuf = git_directory_with_the_folder("others");
    let folder: PathBuf = git_dir.join(FOLDER);
    let work: PathBuf = folder.join("work");
    std::fs::create_dir_all(work.as_path()).expect("the work directory");
    let queued: PathBuf = work.join("push-0123456789abcdef0123456789abcdef01234567-abc");
    std::fs::write(queued.as_path(), b"{}\n").expect("a queued record");
    let a_directory_of_the_records_name: PathBuf = folder.join(FILE);
    std::fs::create_dir_all(a_directory_of_the_records_name.as_path()).expect("not the file that branch wrote");
    let a_staging_name_that_is_not_one: PathBuf = folder.join(format!(".{FILE}.keep"));
    std::fs::write(a_staging_name_that_is_not_one.as_path(), b"mine\n").expect("a file of the person's own");
    remove_what_an_earlier_branch_wrote(git_dir.as_path());
    assert!(queued.exists(), "a queued push of this release stays");
    assert!(a_directory_of_the_records_name.is_dir(), "a directory of that name is not that branch's record");
    assert!(a_staging_name_that_is_not_one.exists(), "a name that is not one of that branch's staging files stays");
}

/// What:
///  A Git directory that never held the record is left as it is,
///  and a second command in the same worktree is quiet.
/// Why:
///   Every command that meets a worktree runs this,
/// so the ordinary case costs one listing of a folder that may not be there.
#[test]
fn a_worktree_without_the_record_is_unchanged() {
    let bare: PathBuf = crate::test_support::fixture_with_prefix("native-earlier-record", "none");
    remove_what_an_earlier_branch_wrote(bare.as_path());
    assert!(!bare.join(FOLDER).exists(), "nothing is created by looking");

    let git_dir: PathBuf = git_directory_with_the_folder("twice");
    let record: PathBuf = git_dir.join(FOLDER).join(FILE);
    std::fs::write(record.as_path(), b"{}\n").expect("the record that branch wrote");
    remove_what_an_earlier_branch_wrote(git_dir.as_path());
    remove_what_an_earlier_branch_wrote(git_dir.as_path());
    assert!(!record.exists(), "the second command finds nothing and says nothing");
}

/// What:
///  A link that carries the record's name is removed itself and never what it names.
/// Why:
///   The folder is inside a worktree,
/// which anyone who can write the worktree can fill;
/// a removal that followed a link would delete a file outside the repository.
#[test]
fn a_link_of_that_name_is_removed_and_not_what_it_names() {
    let git_dir: PathBuf = git_directory_with_the_folder("link");
    let elsewhere: PathBuf = crate::test_support::fixture_with_prefix("native-earlier-record", "elsewhere");
    let kept: PathBuf = elsewhere.join("kept.txt");
    std::fs::write(kept.as_path(), b"not the record\n").expect("a file outside the Git directory");
    let link: PathBuf = git_dir.join(FOLDER).join(FILE);
    std::os::unix::fs::symlink(kept.as_path(), link.as_path()).expect("a link with the record's name");
    remove_what_an_earlier_branch_wrote(git_dir.as_path());
    assert!(!link.exists(), "the link is gone");
    assert!(kept.exists(), "and the file it named is untouched");
}
