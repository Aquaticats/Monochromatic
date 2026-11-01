//! What:
//!  Controls for atomic replacement on real disposable files.
//! Why:
//!  The contents,
//!  the permission bits,
//!  symbolic links and the directory's other entries must
//! all be exactly as specified after a success and after a failure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe(writeAtomically.name, () => { /* contents, mode, symlink, no leftovers, failures */ });
//! ```

/// Import the operation under test and fixture helpers.
use super::write_atomically;
use crate::run_test_support::{read, write};
use crate::test_fs::Fixture;
use std::path::{Path, PathBuf};

/// The sorted names of a directory's entries.
fn entries(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::<String>::new();
    for entry in std::fs::read_dir(directory).expect("readable directory") {
        names.push(
            entry
                .expect("directory entry")
                .file_name()
                .to_string_lossy()
                .into_owned(),
        );
    }
    names.sort();
    return names;
}

/// Read a file's permission bits.
#[cfg(unix)]
fn mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    return std::fs::metadata(path)
        .expect("metadata")
        .permissions()
        .mode()
        & 0o7777;
}

/// Set a file's permission bits.
#[cfg(unix)]
fn set_mode(path: &Path, bits: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(bits)).expect("chmod");
}

/// New contents replace old ones exactly,
///  including when shorter,
///  empty or not text,
///  with no leftover files.
#[test]
fn contents_are_replaced_exactly_without_leftovers() {
    let fixture: Fixture = Fixture::new();
    let path: PathBuf = write(&fixture.path, "a.md", "old contents that are longer\n");
    write(&fixture.path, "sibling.md", "untouched\n");
    write_atomically(&path, "new\n".as_bytes()).expect("replace");
    assert_eq!(read(&fixture.path, "a.md"), "new\n");
    write_atomically(&path, &[0xff, 0x00, b'x']).expect("replace with bytes");
    assert_eq!(std::fs::read(&path).expect("bytes"), [0xff, 0x00, b'x']);
    write_atomically(&path, &[]).expect("replace with nothing");
    assert_eq!(std::fs::read(&path).expect("bytes"), Vec::<u8>::new());
    assert_eq!(read(&fixture.path, "sibling.md"), "untouched\n");
    assert_eq!(entries(&fixture.path), ["a.md", "sibling.md"]);
}

/// The replacement carries the original permission bits exactly,
///  whatever the process umask.
#[cfg(unix)]
#[test]
fn permission_bits_are_kept_exactly() {
    let fixture: Fixture = Fixture::new();
    for bits in [0o600, 0o640, 0o644, 0o664, 0o666, 0o755, 0o400] {
        let path: PathBuf = write(&fixture.path, "a.md", "old\n");
        set_mode(&path, bits);
        write_atomically(&path, "new\n".as_bytes()).expect("replace");
        assert_eq!(mode(&path), bits, "{bits:o}");
        // Restore write permission so the next round's fixture write succeeds.
        set_mode(&path, 0o600);
        assert_eq!(read(&fixture.path, "a.md"), "new\n");
    }
}

/// A symbolic link stays a link;
///  the file it names is the one replaced.
#[cfg(unix)]
#[test]
fn symbolic_links_are_followed_not_replaced() {
    let fixture: Fixture = Fixture::new();
    let target: PathBuf = write(&fixture.path, "real/target.md", "old\n");
    let link: PathBuf = fixture.path.join("link.md");
    std::os::unix::fs::symlink(&target, &link).expect("symlink");
    write_atomically(&link, "new\n".as_bytes()).expect("replace through link");
    assert!(
        std::fs::symlink_metadata(&link)
            .expect("link metadata")
            .file_type()
            .is_symlink()
    );
    assert_eq!(read(&fixture.path, "real/target.md"), "new\n");
    assert_eq!(entries(&fixture.path.join("real")), ["target.md"]);
    assert_eq!(entries(&fixture.path), ["link.md", "real"]);
}

/// A missing target is an error naming the path,
///  and nothing is created.
#[test]
fn a_missing_target_is_an_error_and_creates_nothing() {
    let fixture: Fixture = Fixture::new();
    let path: PathBuf = fixture.path.join("absent.md");
    let error = write_atomically(&path, "new\n".as_bytes()).expect_err("missing target");
    assert!(error.message.contains("absent.md"), "{}", error.message);
    assert_eq!(error.to_string(), error.message);
    assert!(entries(&fixture.path).is_empty());
}

/// When the directory refuses new files,
///  the original bytes and mode stay and no temporary file remains.
#[cfg(unix)]
#[test]
fn a_failed_write_leaves_the_original_untouched() {
    let fixture: Fixture = Fixture::new();
    let directory: PathBuf = fixture.path.join("locked");
    let path: PathBuf = write(&fixture.path, "locked/a.md", "old\n");
    set_mode(&path, 0o640);
    set_mode(&directory, 0o500);
    let outcome = write_atomically(&path, "new\n".as_bytes());
    set_mode(&directory, 0o700);
    // A privileged user can create files despite the directory mode; then the write simply succeeds.
    if let Err(error) = outcome {
        assert!(
            error.message.contains("create temporary file"),
            "{}",
            error.message
        );
        assert_eq!(read(&fixture.path, "locked/a.md"), "old\n");
    } else {
        assert_eq!(read(&fixture.path, "locked/a.md"), "new\n");
    }
    assert_eq!(mode(&path), 0o640);
    assert_eq!(entries(&directory), ["a.md"]);
}

/// A rename that cannot succeed removes the temporary file and leaves the target as it was.
#[test]
fn a_failed_rename_removes_the_temporary_file() {
    let fixture: Fixture = Fixture::new();
    write(&fixture.path, "target/inside.md", "kept\n");
    // A regular file cannot be renamed over a directory, whatever the user's privileges.
    let error = write_atomically(&fixture.path.join("target"), "new\n".as_bytes())
        .expect_err("a directory cannot be replaced by a file");
    assert!(
        error.message.contains("rename temporary file over"),
        "{}",
        error.message
    );
    assert!(error.message.contains("target"), "{}", error.message);
    assert_eq!(entries(&fixture.path), ["target"]);
    assert_eq!(read(&fixture.path, "target/inside.md"), "kept\n");
}

/// Rewrite one file several times;
///  the last round's contents must be the final contents.
fn rewrite_rounds(path: &Path) {
    for round in 0..8 {
        write_atomically(path, format!("round {round}\n").as_bytes()).expect("replace");
    }
}

/// Concurrent replacements of different files in one directory do not collide on temporary names.
#[test]
fn concurrent_replacements_do_not_collide() {
    let fixture: Fixture = Fixture::new();
    let mut paths: Vec<PathBuf> = Vec::<PathBuf>::new();
    for index in 0..16 {
        paths.push(write(
            &fixture.path,
            format!("f{index}.md").as_str(),
            "old\n",
        ));
    }
    // The thread API needs callables; each closure only forwards to the named `rewrite_rounds`.
    std::thread::scope(|scope| {
        for path in &paths {
            scope.spawn(move || return rewrite_rounds(path));
        }
    });
    for path in &paths {
        assert_eq!(std::fs::read_to_string(path).expect("text"), "round 7\n");
    }
    assert_eq!(entries(&fixture.path).len(), 16);
}
