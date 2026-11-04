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

/// How long a step that must not block may take in a control.
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(20);

/// The mode of a file only its owner may use.
const OWNER_ONLY: libc::mode_t = 0o600;

/// How many macros the reverse chain of the closure control defines.
const MACRO_CHAIN_LENGTH: usize = 4_000;

/// Run `work` on a thread and fail when it does not finish within the patience.
fn finishes<T: Send + 'static>(what: &str, work: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _sent = sender.send(work());
    });
    return receiver.recv_timeout(PATIENCE).unwrap_or_else(|_error| { panic!("{what} did not finish within {PATIENCE:?}") });
}

/// A named pipe at `path`.
fn make_pipe(path: &Path) {
    let name = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("path text");
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), OWNER_ONLY) };
    assert_eq!(result, 0, "mkfifo {}", path.display());
}

/// An environment that names no system or global file and the given system attributes.
fn environment(system_attributes: SystemAttributes) -> ConversionEnvironment {
    return ConversionEnvironment { scope: ScopeEnvironment { no_system: true, ..ScopeEnvironment::default() }, system_attributes, alternate_attribute_source: None, variables: Vec::new() };
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
    let moved: ConversionEnvironment = ConversionEnvironment { scope: ScopeEnvironment { system: Some(elsewhere), ..ScopeEnvironment::default() }, system_attributes: SystemAttributes::File(etc.join("gitattributes")), alternate_attribute_source: None, variables: Vec::new() };
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

/// Every quoted line refuses the paths below its directory, whatever it sets and wherever its tokens end: an attribute value that
///  holds the macro marker, a token glued to the closing quote, and a line that sets nothing the content cares about.
#[test]
fn every_quoted_line_refuses_at_every_source_level() {
    for (label, in_directory, text) in [
        ("macro marker in a value", true, "\"target\" ident note=[attr]x\n"),
        ("token glued to the closing quote", false, "\"target\"ident\n"),
        ("a quoted line that sets nothing", false, "\"other\" -text\n"),
    ] {
        let (parent, root, worktree) = settled(format!("content-quoted-{}", label.replace(' ', "-")).as_str(), &|root: &Path| {
            std::fs::create_dir(root.join("d")).expect("d");
            std::fs::write(root.join("d/target"), b"$Id: old $\n").expect("target");
        });
        if in_directory {
            std::fs::write(worktree.join("d/.gitattributes"), text.as_bytes()).expect("directory attributes");
        } else {
            std::fs::create_dir_all(root.join(".git/info")).expect("info");
            std::fs::write(root.join(".git/info/attributes"), text.as_bytes()).expect("info attributes");
        }
        make_stale(worktree.join("d/target").as_path());
        let reason: String = check(worktree.as_path(), &environment(SystemAttributes::Disabled)).expect_err(label);
        assert!(reason.contains("quoted"), "{label}: {reason}");
        remove(parent.as_path());
    }
}

/// A named pipe where an attributes file should be ends the check with a reason instead of blocking it.
#[test]
fn a_named_pipe_as_an_attributes_file_does_not_block() {
    let (parent, root, worktree) = settled("content-pipe", &|root: &Path| {
        std::fs::create_dir(root.join("d")).expect("d");
        std::fs::write(root.join("d/target"), b"$Id: old $\n").expect("target");
    });
    std::fs::create_dir_all(root.join(".git/info")).expect("info");
    make_pipe(root.join(".git/info/attributes").as_path());
    make_stale(worktree.join("d/target").as_path());
    let at: PathBuf = worktree.clone();
    let outside = finishes("the check with a pipe in info/attributes", move || { return check(at.as_path(), &environment(SystemAttributes::Disabled)); });
    assert!(outside.expect_err("pipe").contains("not a plain file"), "info/attributes");
    std::fs::remove_file(root.join(".git/info/attributes")).expect("remove the pipe");
    make_pipe(worktree.join("d/.gitattributes").as_path());
    let there: PathBuf = worktree.clone();
    let inside = finishes("the check with a pipe as a directory's attributes", move || { return check(there.as_path(), &environment(SystemAttributes::Disabled)); });
    assert!(inside.is_err(), "d/.gitattributes");
    remove(parent.as_path());
}

/// A long reverse chain of macros is followed in work that grows with its length, not with the cube of it.
#[test]
fn a_long_chain_of_macros_is_followed_in_linear_work() {
    let (parent, root, worktree) = settled("content-macros", &|root: &Path| { std::fs::write(root.join("target"), b"$Id: old $\n").expect("target"); });
    let mut text: String = String::new();
    for index in (1..=MACRO_CHAIN_LENGTH).rev() {
        text.push_str(format!("[attr]chain{index} chain{}\n", index - 1).as_str());
    }
    text.push_str("[attr]chain0 ident\n");
    text.push_str(format!("target chain{MACRO_CHAIN_LENGTH}\n").as_str());
    std::fs::create_dir_all(root.join(".git/info")).expect("info");
    std::fs::write(root.join(".git/info/attributes"), text).expect("attributes");
    make_stale(worktree.join("target").as_path());
    let at: PathBuf = worktree.clone();
    let outcome = finishes("the macro closure", move || { return check(at.as_path(), &environment(SystemAttributes::Disabled)); });
    assert!(outcome.expect_err("the chain reaches ident").contains("chain"), "the reason names the macro");
    remove(parent.as_path());
}

/// The stat-only proof reaches files below a held root too: a directory moved away and replaced by a link is not read through.
#[test]
fn the_stat_only_proof_does_not_follow_a_link_in_a_parent() {
    let (parent, _root, worktree) = settled("content-stat-link", &|_root: &Path| {});
    let found = inspect(worktree.as_path()).expect("registered");
    check_clean_tracked(worktree.as_path(), found.admin.as_path(), found.common.as_path()).expect("positive control");
    let away: PathBuf = parent.join("src-away");
    std::fs::rename(worktree.join("src"), &away).expect("move the directory away");
    std::os::unix::fs::symlink(&away, worktree.join("src")).expect("link in its place");
    let reason: String = check_clean_tracked(worktree.as_path(), found.admin.as_path(), found.common.as_path()).expect_err("a link");
    assert!(reason.contains("src/") && reason.contains("link"), "{reason}");
    remove(parent.as_path());
}

/// A tree chosen as the source of attributes (`GIT_ATTR_SOURCE`, `attr.tree`, the command line) is not the worktree files the
///  judge reads, so the content shortcut is refused; an entry the stat data vouches for is unaffected.
#[test]
fn an_alternate_attribute_source_refuses_the_content_shortcut() {
    let (parent, root, worktree) = settled("content-source", &|root: &Path| { std::fs::write(root.join("target"), b"$Id: old $\n").expect("target"); });
    let selected: ConversionEnvironment = ConversionEnvironment { alternate_attribute_source: Some(String::from("GIT_ATTR_SOURCE is set")), ..environment(SystemAttributes::Disabled) };
    check(worktree.as_path(), &selected).expect("every entry is vouched for by its stat data, so no content is read");
    make_stale(worktree.join("target").as_path());
    let reason: String = check(worktree.as_path(), &selected).expect_err("selected by the environment");
    assert!(reason.contains("GIT_ATTR_SOURCE") && reason.contains("cannot be ruled out"), "{reason}");
    check(worktree.as_path(), &environment(SystemAttributes::Disabled)).expect("positive control: without a source the content is decided");
    git(root.as_path(), &["config", "attr.tree", "HEAD"]);
    let configured: String = check(worktree.as_path(), &environment(SystemAttributes::Disabled)).expect_err("selected by the configuration");
    assert!(configured.contains("attr.tree"), "{configured}");
    remove(parent.as_path());
}

/// The environment of a process names an attribute source when `GIT_ATTR_SOURCE` is there, whatever its value.
#[test]
fn the_process_environment_names_an_attribute_source() {
    let with = |value: &str| -> Vec<(OsString, OsString)> { return vec![(OsString::from("GIT_ATTR_SOURCE"), OsString::from(value))]; };
    assert!(ConversionEnvironment::from_variables(with("HEAD").as_slice()).alternate_attribute_source.is_some());
    assert!(ConversionEnvironment::from_variables(with("").as_slice()).alternate_attribute_source.is_some());
    assert!(ConversionEnvironment::from_variables(&[]).alternate_attribute_source.is_none());
}
