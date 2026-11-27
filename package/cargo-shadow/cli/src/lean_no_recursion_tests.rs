//! The ban on a lean-folder tool starting a shadow or cargo:
//! no source of the lean-folder tools starts `cargo`,
//! asks for its own executable,
//! or starts a program by a path it was not given,
//! and the shadow's own start of cargo changes only the environment it already sets.
//! Owner rule (2026-11-07):
//! "We should never invoke ourselves recursively.
//! That's bad design.
//! Audit,
//! remove,
//! and ban that design aggressively."
//!
//! The guard reads every lean source wherever it sits.
//! The tool's code was split into the folders `lean_idle_trees`,
//! `lean_podman_cleanup`,
//! `lean_removal` and `lean_run`,
//! and a guard that listed `src` itself stopped reading the 18 files below them:
//! 2,174 lines on 2026-11-27,
//! finding Medium 4 of the release read.
//! So the guard walks the tree through the crate's one source walk
//! (`src/test_scratch/source_tree_files.rs`,
//! which never enters a link),
//! it states every file it read,
//! and it fails when the tree names a lean file that it did not read.

/// What:
/// `fs` is the file system's own functions.
/// Why:
/// The guard reads a source and writes the fixtures of its own control.
use std::fs;
/// What:
/// `Path` is a borrowed file-system path and `PathBuf` an owned one.
/// Why:
/// Each file the guard read is kept as an owned path,
/// which its messages name.
use std::path::{
    Path,
    PathBuf,
};

/// What:
/// `scratch` is one empty folder under the build output,
/// removed when its guard drops.
/// Why:
/// The control below makes a tree to walk,
/// and never writes under `/tmp`.
use crate::test_scratch::scratch;
/// What:
/// `source_tree_files` is the crate's one walk of its own sources,
/// which visits each file once and never enters a link.
/// Why:
/// A split moves lean code into a new folder,
/// and a folder listed by name would stop being read without anything failing.
use crate::test_scratch::source_tree_files::source_tree_files;

/// Text no lean-folder source may contain,
/// with why.
const FORBIDDEN: [(&str, &str); 5] = [
    (
        "current_exe",
        "a tool that asks for its own executable can start itself",
    ),
    (
        "Command::new(\"cargo",
        "a lean tool must not start cargo (it would start the shadow)",
    ),
    (
        "Command::new(\"cargo-shadow",
        "a lean tool must not start itself",
    ),
    (
        "find_real_cargo",
        "a lean tool must not look for a cargo to run",
    ),
    ("run_shim", "a lean tool must not enter shadow mode"),
];

/// What:
/// `LEAN_PREFIX` is the start of the name that makes a folder or a file lean code:
/// `lean_idle_trees` and `lean_policy.rs` carry it.
/// Why:
/// The guard reads the lean tool and not the whole crate,
/// so it needs the one property that says which files are the tool's.
const LEAN_PREFIX: &str = "lean_";

/// What:
/// `TEST_SUFFIX` is the end of every test file's name.
/// Why:
/// A test may name the forbidden words and start what it tests,
/// and this file itself is one.
const TEST_SUFFIX: &str = "_tests.rs";

/// What:
/// `SUPPORT_FILE` is the shared fixtures of the lean controls.
/// Why:
/// It is built for the tests and runs in a test build only,
/// so the ban on the tool's sources does not reach it.
const SUPPORT_FILE: &str = "lean_test_support.rs";

/// What:
/// `fn source_root() -> PathBuf` is the `src` folder of the crate this test binary was built from.
/// Why:
/// The guard reads the sources beside its own module,
/// and `CARGO_MANIFEST_DIR` is the folder cargo built this crate in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sourceRoot = (): string => join(import.meta.dirname, '.');
/// ```
fn source_root() -> PathBuf {
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
}

/// What:
/// `fn name_of(path: &Path) -> String` is the last part of a path,
/// which is the file's or folder's own name.
/// Why:
/// The two rules below ask whether a name starts with `lean_`,
/// and a path's own name is all they look at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nameOf = (path: string): string => basename(path);
/// ```
fn name_of(path: &Path) -> String {
    // What: `file_name()` is the last part as an `Option`, absent only for a path that ends in `..` or `/`.
    let Some(name) = path.file_name() else {
        // What: an empty `String` is the absence of a name; nothing can start with `lean_`, so no file is lean code.
        return String::new();
    };
    // What: `to_string_lossy()` reads the bytes as text, replacing any that are not valid UTF-8.
    // Why: every source of this crate has a valid UTF-8 name, and a lossy read never fails here.
    return name.to_string_lossy().into_owned();
}

/// What:
/// `fn is_scannable(name: &str) -> bool` says whether a file's name makes it Rust source a test does not own.
/// Why:
/// The ban is about the tool's own code,
/// and a test or the shared fixtures may hold the words it forbids.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isScannable = (name: string): boolean => name.endsWith('.rs') && !name.endsWith('_tests.rs') && name !== 'lean_test_support.rs';
/// ```
fn is_scannable(name: &str) -> bool {
    return name.ends_with(".rs") && !name.ends_with(TEST_SUFFIX) && name != SUPPORT_FILE;
}

/// What:
/// `fn is_read_as_lean_code(relative: &Path) -> bool` says whether the guard reads a path under `src`:
/// the path's first part,
/// which is the file's own name at the top of `src` or the top folder it sits in,
/// starts with `lean_`.
/// Why:
/// This is the walk's own rule,
/// stated apart from the crate's naming so the guard can be asked whether it covers the naming.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isReadAsLeanCode = (relative: string): boolean => isScannable(basename(relative)) && firstPart(relative).startsWith('lean_');
/// ```
fn is_read_as_lean_code(relative: &Path) -> bool {
    if !is_scannable(&name_of(relative)) {
        return false;
    }
    // What: `components().next()` is the path's first part as an `Option`.
    let Some(first) = relative.components().next() else {
        return false;
    };
    // What: `starts_with(LEAN_PREFIX)` asks whether the first part opens with `lean_`.
    return first.as_os_str().to_string_lossy().starts_with(LEAN_PREFIX);
}

/// What:
/// `fn is_named_lean_code(relative: &Path) -> bool` says whether the crate's naming makes a path under `src` lean code:
/// any part of it starts with `lean_`,
/// at any depth.
/// Why:
/// The coverage check needs the naming rule alone,
/// so a `lean_` file in a folder the walk does not enter is named rather than silently skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNamedLeanCode = (relative: string): boolean => isScannable(basename(relative)) && relative.split('/').some((part) => part.startsWith('lean_'));
/// ```
fn is_named_lean_code(relative: &Path) -> bool {
    if !is_scannable(&name_of(relative)) {
        return false;
    }
    // What: `components()` is the path's parts one by one, without allocating a list of them.
    for part in relative.components() {
        if part.as_os_str().to_string_lossy().starts_with(LEAN_PREFIX) {
            return true;
        }
    }
    return false;
}

/// What:
/// `fn lean_sources_under(root: &Path) -> Vec<(PathBuf, String)>` is each lean source below one folder,
/// with the path the tree gives it and the text it holds.
/// Why:
/// The ban is read off the text,
/// and the path is what a failure names to the person who edits the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leanSourcesUnder = (root: string): Array<[string, string]> => sourceTreeFiles(root).filter(isReadAsLeanCode).map((path) => [path, readFileSync(join(root, path), 'utf8')]);
/// ```
fn lean_sources_under(root: &Path) -> Vec<(PathBuf, String)> {
    // What: a growable list of `(path, text)` pairs that this guard owns.
    let mut sources: Vec<(PathBuf, String)> = Vec::new();
    // What: `expect` states that a folder of this crate's own tree must be readable; the walk's error is a defect here.
    let held: Vec<PathBuf> = source_tree_files(root).expect("read the source tree");
    for relative in held {
        if !is_read_as_lean_code(&relative) {
            continue;
        }
        // What: the file's text, read through the path the walk answered under the root it was given.
        let text: String = fs::read_to_string(root.join(&relative)).expect("read a lean source");
        sources.push((relative, text));
    }
    return sources;
}

/// What:
/// `fn named_lean_files_under(root: &Path) -> Vec<PathBuf>` is each file below one folder that the crate's naming makes
/// lean code,
/// whether or not the guard reads it.
/// Why:
/// The guard's coverage is a comparison between this list and the one it read,
/// never a count a person keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namedLeanFilesUnder = (root: string): string[] => sourceTreeFiles(root).filter(isNamedLeanCode);
/// ```
fn named_lean_files_under(root: &Path) -> Vec<PathBuf> {
    // What: a growable list of the relative paths the naming picks out.
    let mut named: Vec<PathBuf> = Vec::new();
    for relative in source_tree_files(root).expect("read the source tree") {
        if is_named_lean_code(&relative) {
            named.push(relative);
        }
    }
    return named;
}

/// What:
/// `fn unread_lean_files(read: &[PathBuf], named: &[PathBuf]) -> Vec<PathBuf>` is each path the naming picks out
/// that the guard did not read.
/// Why:
/// An empty answer is the guard's coverage,
/// and a non-empty one is what its failure names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unreadLeanFiles = (read: string[], named: string[]): string[] => named.filter((path) => !read.includes(path));
/// ```
fn unread_lean_files(
    read: &[PathBuf],
    named: &[PathBuf],
) -> Vec<PathBuf> {
    // What: a growable list of the named paths that no read answers.
    let mut unread: Vec<PathBuf> = Vec::new();
    for path in named {
        if read.contains(path) {
            continue;
        }
        // What: `clone()` copies the path, because the caller keeps the list after this one is dropped.
        unread.push(path.clone());
    }
    return unread;
}

/// The ban:
/// no lean source contains a forbidden word,
/// and the guard read every lean file the tree names.
#[test]
fn no_lean_source_starts_cargo_or_itself() {
    // What: the `src` folder the guard reads, held once so both questions ask about the same tree.
    let root: PathBuf = source_root();
    // What: each lean source the guard read, with its text.
    let sources: Vec<(PathBuf, String)> = lean_sources_under(&root);
    // What: the paths of those sources, which every message below states.
    let mut read: Vec<PathBuf> = Vec::new();
    for (path, text) in &sources {
        read.push(path.clone());
        for (needle, why) in FORBIDDEN {
            // What: the module that holds this very ban is a test file and is not scanned.
            assert!(
                !text.contains(needle),
                "{} contains `{needle}`: {why}",
                path.display()
            );
        }
    }
    // What: the files the crate's naming makes lean code, read or not.
    let named: Vec<PathBuf> = named_lean_files_under(&root);
    assert!(
        !named.is_empty(),
        "the tree names no lean source at all, so this guard checked nothing: it read {read:?}"
    );
    let unread: Vec<PathBuf> = unread_lean_files(&read, &named);
    assert!(
        unread.is_empty(),
        "this guard read {} lean sources ({read:?}) and did not read {} the tree names as lean ({unread:?}); a lean source is a `.rs` file that is not a test and whose own name or a folder above it starts `lean_`",
        read.len(),
        unread.len()
    );
}

/// The coverage itself,
/// on a tree small enough to state whole:
/// a lean file the walk does not reach is named as unread,
/// so the check above can fail instead of passing on a count.
#[test]
fn a_lean_file_the_walk_does_not_reach_is_named_as_unread() {
    let root: tempfile::TempDir = scratch();
    fs::create_dir_all(root.path().join("lean_run")).expect("make a lean folder");
    fs::create_dir_all(root.path().join("registry")).expect("make a folder that is not lean");
    fs::write(root.path().join("lean_run").join("inside.rs"), "inside").expect("write a source");
    fs::write(root.path().join("lean_top.rs"), "top").expect("write a source");
    fs::write(
        root.path().join("registry").join("lean_registry.rs"),
        "a lean name in a folder the walk does not enter",
    )
    .expect("write a source");

    let sources: Vec<(PathBuf, String)> = lean_sources_under(root.path());
    // What: the paths the walk read, in the one order the walk sorts them.
    let mut read: Vec<PathBuf> = Vec::new();
    for (path, _text) in &sources {
        read.push(path.clone());
    }
    // What: the names the walk reached, written out, so a walk that changes its reach says so here.
    let reached: Vec<PathBuf> = vec![
        Path::new("lean_run").join("inside.rs"),
        PathBuf::from("lean_top.rs"),
    ];
    assert_eq!(read, reached, "the walk read another set of lean sources");

    let named: Vec<PathBuf> = named_lean_files_under(root.path());
    let unread: Vec<PathBuf> = unread_lean_files(&read, &named);
    // What: the one file the naming picks out and the walk does not reach.
    let missed: Vec<PathBuf> = vec![Path::new("registry").join("lean_registry.rs")];
    assert_eq!(
        unread, missed,
        "a lean name the walk does not reach was not named as unread, so the coverage check cannot fail"
    );
}

#[test]
fn the_shadow_only_adds_the_build_folder_variable_for_the_lean_tools() {
    let shim: String = fs::read_to_string(source_root().join("shim.rs")).expect("read the shim");
    let uses: usize = shim.matches("lean_").count();
    assert_eq!(
        uses,
        shim.matches("lean_keyed_folder").count(),
        "the shim may reach the lean tools only through the keyed folder"
    );
    for forbidden in [
        "lean_run",
        "lean_command",
        "lean_service",
        "lean_idle",
        "lean_sweep",
    ] {
        assert!(
            !shim.contains(forbidden),
            "the shim must not start {forbidden}"
        );
    }
}
