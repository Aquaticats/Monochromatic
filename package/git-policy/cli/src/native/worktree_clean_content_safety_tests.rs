//! Controls for the review findings on the content decision: the system attributes file is not derived from the system
//!  configuration path, paths and patterns are bytes, attribute lines are lexed as Git lexes them, a link in a parent
//!  directory is never followed, and the bytes that are hashed are the bytes of the one file that was judged.

use super::*;
use crate::fs_beneath::open_root;
use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_index::parse_index;
use crate::subvolume_test_support::{backdate, checkout_with_worktree};
use crate::test_support::{git, remove};
use crate::worktree_clean::{check_clean_tracked, check_clean_tracked_by_content, stored_stats};
use crate::worktree_convert::inspect;
use std::os::unix::ffi::OsStrExt;

/// An environment that names no system or global file and the given system attributes.
fn environment(system_attributes: SystemAttributes) -> ConversionEnvironment {
    return ConversionEnvironment { scope: ScopeEnvironment { no_system: true, ..ScopeEnvironment::default() }, system_attributes };
}

/// Run the content check on a worktree.
fn check(worktree: &Path, environment: &ConversionEnvironment) -> Result<(), String> {
    let found = inspect(worktree).expect("registered");
    return check_clean_tracked_by_content(worktree, found.admin.as_path(), found.common.as_path(), environment, false).map(|_clean| {});
}

/// A checkout where `prepare` committed more files, with a settled detached worktree `topic` at that commit.
fn settled(name: &str, prepare: &dyn Fn(&Path)) -> (PathBuf, PathBuf, PathBuf) {
    let (parent, root) = checkout_with_worktree(name);
    prepare(root.as_path());
    git(root.as_path(), &["add", "--all", "--", "."]);
    git(root.as_path(), &["commit", "--quiet", "--allow-empty", "--message=prepared"]);
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "topic"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("topic")).expect("canonical worktree");
    backdate(worktree.as_path());
    git(worktree.as_path(), &["update-index", "--refresh"]);
    return (parent, root, worktree);
}

/// Rewrite a file with the bytes it has, so that its stat data is stale and only its content can vouch for it.
fn make_stale(path: &Path) {
    let bytes: Vec<u8> = std::fs::read(path).expect("read");
    std::fs::write(path, bytes).expect("rewrite");
}

/// A system attributes file that sets `ident` is read where the executable says it is, not beside the system configuration
///  file, and `GIT_ATTR_NOSYSTEM` or an unknown location are told apart.
#[test]
fn the_system_attributes_file_is_not_derived_from_the_system_config_path() {
    let (parent, root, worktree) = settled("content-system", &|root: &Path| { std::fs::write(root.join("target"), b"$Id: old $\n").expect("target"); });
    let etc: PathBuf = parent.join("etc");
    std::fs::create_dir_all(&etc).expect("etc");
    std::fs::write(etc.join("gitattributes"), "target ident\n").expect("system attributes");
    let elsewhere: PathBuf = parent.join("custom.gitconfig");
    std::fs::write(&elsewhere, "").expect("system config");
    make_stale(worktree.join("target").as_path());
    let moved: ConversionEnvironment = ConversionEnvironment { scope: ScopeEnvironment { system: Some(elsewhere), ..ScopeEnvironment::default() }, system_attributes: SystemAttributes::File(etc.join("gitattributes")) };
    let reason: String = check(worktree.as_path(), &moved).expect_err("the attribute applies");
    assert!(reason.contains("target") && reason.contains("ident") && reason.contains("etc/gitattributes"), "{reason}");
    check(worktree.as_path(), &environment(SystemAttributes::Disabled)).expect("the switch turns the file off");
    check(worktree.as_path(), &environment(SystemAttributes::File(etc.join("none")))).expect("a file that is not there sets nothing");
    let unknown: String = check(worktree.as_path(), &environment(SystemAttributes::Unknown)).expect_err("unknown");
    assert!(unknown.contains("system attributes file"), "{unknown}");
    drop(root);
    remove(parent.as_path());
}

/// A file whose name is not UTF-8, or whose letters are two bytes, is matched by bytes: `?` is one byte, and the attributes
///  file of a directory with such a name is that directory's own.
#[test]
fn paths_and_patterns_are_matched_as_bytes() {
    let (parent, root, worktree) = settled("content-bytes", &|root: &Path| {
        std::fs::write(root.join(std::ffi::OsStr::from_bytes(b"\xFF.txt")), b"$Id: old $\n").expect("non-UTF-8 name");
        std::fs::write(root.join("\u{130}.txt"), b"$Id: old $\n").expect("two-byte letter");
        std::fs::create_dir(root.join(std::ffi::OsStr::from_bytes(b"\xFF"))).expect("directory");
        std::fs::write(root.join(std::ffi::OsStr::from_bytes(b"\xFF")).join(".gitattributes"), "*.dat ident\n").expect("nested attributes");
        std::fs::write(root.join(std::ffi::OsStr::from_bytes(b"\xFF")).join("a.dat"), b"$Id: old $\n").expect("nested file");
        std::fs::create_dir(root.join(std::ffi::OsStr::from_bytes(b"\xFE"))).expect("other directory");
        std::fs::write(root.join(std::ffi::OsStr::from_bytes(b"\xFE")).join("a.dat"), b"$Id: old $\n").expect("other nested file");
    });
    let off: ConversionEnvironment = environment(SystemAttributes::Disabled);
    let info: PathBuf = root.join(".git/info");
    std::fs::create_dir_all(&info).expect("info");
    // `?` against the single byte 0xFF.
    std::fs::write(info.join("attributes"), "?.txt ident\n").expect("attributes");
    make_stale(worktree.join(std::ffi::OsStr::from_bytes(b"\xFF.txt")).as_path());
    let reason: String = check(worktree.as_path(), &off).expect_err("one byte");
    assert!(reason.contains("ident"), "{reason}");
    make_stale(worktree.join(std::ffi::OsStr::from_bytes(b"\xFF.txt")).as_path());
    // `??` against the two bytes of a letter that Unicode lower-cases to three.
    std::fs::write(info.join("attributes"), "??.txt ident\n").expect("attributes");
    make_stale(worktree.join("\u{130}.txt").as_path());
    let two: String = check(worktree.as_path(), &off).expect_err("two bytes");
    assert!(two.contains("ident"), "{two}");
    // The attributes file of the directory named with a byte that is not UTF-8 is that directory's, not another one's.
    std::fs::write(info.join("attributes"), "").expect("attributes");
    make_stale(worktree.join(std::ffi::OsStr::from_bytes(b"\xFF")).join("a.dat").as_path());
    let nested: String = check(worktree.as_path(), &off).expect_err("nested");
    assert!(nested.contains("ident"), "{nested}");
    remove(parent.as_path());
}

/// Attribute lines are lexed as `attr.c` lexes them: only space, tab, carriage return and line feed separate tokens, a NUL
///  ends the text of a line, and a byte order mark on the first line is not part of the pattern.
#[test]
fn attribute_lines_are_lexed_like_git_lexes_them() {
    for (label, name, attributes) in [
        ("no-break space", "a\u{a0}b", "a\u{a0}b ident\n"),
        ("nul", "target", "target ident\0junk\n"),
        ("byte order mark", "target", "\u{feff}target ident\n"),
        ("carriage return", "target", "target\tident\r\n"),
    ] {
        let named: String = String::from(name);
        let (parent, root, worktree) = settled(format!("content-lex-{}", label.replace(' ', "-")).as_str(), &|root: &Path| { std::fs::write(root.join(&named), b"$Id: old $\n").expect("target"); });
        std::fs::create_dir_all(root.join(".git/info")).expect("info");
        std::fs::write(root.join(".git/info/attributes"), attributes.as_bytes()).expect("attributes");
        make_stale(worktree.join(name).as_path());
        let reason: String = check(worktree.as_path(), &environment(SystemAttributes::Disabled)).expect_err(label);
        assert!(reason.contains("ident"), "{label}: {reason}");
        remove(parent.as_path());
    }
}

/// A tracked directory replaced by a link to a directory that holds a file with the committed bytes is not read through.
#[test]
fn a_link_in_a_parent_directory_is_not_followed() {
    let (parent, _root, worktree) = settled("content-parent-link", &|root: &Path| {
        std::fs::create_dir(root.join("d")).expect("d");
        std::fs::write(root.join("d/f"), b"committed\n").expect("f");
    });
    let outside: PathBuf = parent.join("elsewhere");
    std::fs::create_dir(&outside).expect("outside");
    std::fs::write(outside.join("f"), b"committed\n").expect("copy of f");
    std::fs::remove_dir_all(worktree.join("d")).expect("remove d");
    std::os::unix::fs::symlink(&outside, worktree.join("d")).expect("link in its place");
    let reason: String = check(worktree.as_path(), &environment(SystemAttributes::Disabled)).expect_err("a link");
    assert!(reason.contains("d/f") && reason.contains("link"), "{reason}");
    remove(parent.as_path());
}

/// What a judge needs to look at `README.md`: the held root, its entry, its blob and the index second.
struct Subject {
    /// The held worktree root.
    root: std::fs::File,
    /// The stat entry.
    entry: crate::worktree_clean::StatEntry,
    /// The blob name the index holds.
    id: Vec<u8>,
    /// The whole seconds of the index file.
    index_seconds: u32,
    /// The worktree root path.
    path: PathBuf,
}

/// The subject for `README.md` of a settled worktree.
fn subject_of(worktree: &Path) -> Subject {
    let found = inspect(worktree).expect("registered");
    let clean = check_clean_tracked(worktree, found.admin.as_path(), found.common.as_path()).expect("clean before the change");
    let entry = stored_stats(clean.index_bytes.as_slice(), clean.hash_length).expect("entries").into_iter().find(|candidate| { return candidate.path == b"README.md"; }).expect("README.md");
    let id: Vec<u8> = parse_index(clean.index_bytes.as_slice(), ObjectFormat::Sha1).expect("index").into_iter().find(|candidate| { return candidate.path == b"README.md"; }).expect("README.md").id;
    return Subject { root: open_root(worktree).expect("held root"), entry, id, index_seconds: clean.index_seconds, path: worktree.to_path_buf() };
}

/// Judge `README.md` with the given actions at the three moments.
fn judge_with(subject: &Subject, seams: &mut ReadSeams<'_>) -> Result<(), String> {
    let found = inspect(subject.path.as_path()).expect("registered");
    let environment: ConversionEnvironment = environment(SystemAttributes::Disabled);
    let mut judge: ContentJudge<'_> = ContentJudge::new(&subject.root, found.admin.as_path(), found.common.as_path(), &environment, ObjectFormat::Sha1);
    let place: Place<'_> = Place { directory: &subject.root, parents: &[], name: std::ffi::OsStr::new("README.md") };
    return judge.agrees_seamed(&place, &subject.entry, subject.id.as_slice(), subject.index_seconds, seams);
}

/// The file is one file from the look at its state to the end of the read: a `chmod`, a swap to a link and a write each refuse,
///  and an unchanged file is accepted.
#[test]
fn the_hashed_bytes_are_the_bytes_of_the_one_file_that_was_judged() {
    let (parent, _root, worktree) = settled("content-stable", &|_root: &Path| {});
    let subject: Subject = subject_of(worktree.as_path());
    let readme: PathBuf = worktree.join("README.md");
    make_stale(readme.as_path());
    judge_with(&subject, &mut ReadSeams { before_open: &mut || {}, before_read: &mut || {}, after_read: &mut || {} }).expect("positive control");
    // A chmod between the look and the open.
    let chmod: String = judge_with(&subject, &mut ReadSeams {
        before_open: &mut || { std::fs::set_permissions(&readme, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("chmod"); },
        before_read: &mut || {},
        after_read: &mut || {},
    })
    .expect_err("executable");
    assert!(chmod.contains("README.md") && chmod.contains("executable bit"), "{chmod}");
    std::fs::set_permissions(&readme, std::os::unix::fs::PermissionsExt::from_mode(0o644)).expect("chmod back");
    make_stale(readme.as_path());
    // A swap for a link whose target text equals the committed bytes.
    let swapped: String = judge_with(&subject, &mut ReadSeams {
        before_open: &mut || {
            std::fs::remove_file(&readme).expect("remove");
            std::os::unix::fs::symlink("readme\n", &readme).expect("link");
        },
        before_read: &mut || {},
        after_read: &mut || {},
    })
    .expect_err("link");
    assert!(swapped.contains("README.md"), "{swapped}");
    std::fs::remove_file(&readme).expect("remove link");
    std::fs::write(&readme, b"readme\n").expect("restore");
    // A write after the bytes were read and before the state is read again.
    let written: String = judge_with(&subject, &mut ReadSeams {
        before_open: &mut || {},
        before_read: &mut || {},
        after_read: &mut || { std::fs::write(&readme, b"readme\n").expect("write"); },
    })
    .expect_err("written");
    assert!(written.contains("changed while it was examined"), "{written}");
    remove(parent.as_path());
}

/// A file that grows past the limit after its size was looked at is refused, and a read never takes more than the bound.
#[test]
fn a_file_that_grows_during_the_look_is_refused_and_the_read_is_bounded() {
    let (parent, _root, worktree) = settled("content-grows", &|_root: &Path| {});
    let subject: Subject = subject_of(worktree.as_path());
    let readme: PathBuf = worktree.join("README.md");
    make_stale(readme.as_path());
    let grown: String = judge_with(&subject, &mut ReadSeams {
        before_open: &mut || {},
        before_read: &mut || { std::fs::OpenOptions::new().write(true).open(&readme).expect("open").set_len(CONTENT_LIMIT + 10).expect("grow"); },
        after_read: &mut || {},
    })
    .expect_err("grown");
    assert!(grown.contains("larger than"), "{grown}");
    // A reader that never ends is asked for one byte more than the limit and no more.
    struct Endless {
        /// How many bytes were handed out.
        served: std::rc::Rc<std::cell::Cell<u64>>,
    }
    impl Read for Endless {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            buffer.fill(b'x');
            self.served.set(self.served.get() + buffer.len() as u64);
            return Ok(buffer.len());
        }
    }
    let served: std::rc::Rc<std::cell::Cell<u64>> = std::rc::Rc::new(std::cell::Cell::new(0));
    assert_eq!(read_at_most(Endless { served: std::rc::Rc::clone(&served) }, 1_000).expect("read"), None);
    assert_eq!(served.get(), 1_001, "never more than the limit and one byte");
    assert_eq!(read_at_most(&b"abc"[..], 3).expect("read"), Some(b"abc".to_vec()));
    remove(parent.as_path());
}
