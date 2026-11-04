//! The unit tests of the plain-file reader of references and branch configuration,
//! over hand-made repository directories.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::fs;
// Imports `std::path::{Path, PathBuf}`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
use std::path::{Path, PathBuf};

// Imports `crate::git_config::parse_config`.
use crate::git_config::parse_config;
// Imports `crate::git_native::native_base`.
use crate::git_native::native_base;
// Imports `crate::git_refs::{GitLayout, Head, NativeEn...`.
use crate::git_refs::{GitLayout, Head, NativeEnvironment, find_layout, lookup_ref, lookup_short, read_config, read_head, tracking_ref};
// Imports `crate::test_scratch::scratch`.
use crate::test_scratch::scratch;

const FIRST: &str = "1111111111111111111111111111111111111111";
const SECOND: &str = "2222222222222222222222222222222222222222";

fn environment(root: &Path) -> NativeEnvironment {
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    return NativeEnvironment { home: None, xdg_config_home: None, system_config: Some(root.join("none")), blocking_variable: None, git: crate::git_refs::GitProgram::default(), user_id: crate::git_refs::current_user_id() };
}

/// A plain repository directory with the given HEAD text and configuration.
fn repository(head: &str, config: &str) -> (tempfile::TempDir, PathBuf) {
    let root = scratch();
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let dot_git: PathBuf = root.path().join(".git");
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    fs::create_dir_all(dot_git.join("refs/heads")).expect("refs");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(dot_git.join("objects")).expect("objects");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(dot_git.join("HEAD"), head).expect("head");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(dot_git.join("config"), config).expect("config");
    // Typed binding: `path: PathBuf`.
    let path: PathBuf = root.path().to_path_buf();
    return (root, path);
}

/// A layout over a directory that has no files;
/// the lookups of these tests read only the configuration.
fn layout_of(_config: &crate::git_config::ParsedConfig) -> GitLayout {
    return GitLayout { git_dir: PathBuf::from("/nonexistent/git"), common_dir: PathBuf::from("/nonexistent/git") };
}

fn write_ref(root: &Path, name: &str, object: &str) {
    // Typed binding: `path: PathBuf`.
    let path: PathBuf = root.join(".git").join(name);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    fs::write(path, format!("{object}\n")).expect("ref");
}

#[test]
fn head_is_a_branch_or_a_detached_object_and_anything_else_is_refused() {
    let (root, path) = repository("ref: refs/heads/main\n", "");
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `layout: GitLayout`.
    let layout: GitLayout = find_layout(&path, &environment(root.path())).expect("layout");
    // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
    // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Ok(value)  is  return value;
    // // Err(error) is  throw error;   (the caller sees it as a returned value)
    // ```
    // Borrow `&layout`: lends the value without copying or giving it away.
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(read_head(&layout), Ok(Head::Branch("refs/heads/main".to_string())));
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `format!`: builds or prints text (or a list) from a template.
    fs::write(path.join(".git/HEAD"), format!("{FIRST}\n")).expect("detach");
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&layout`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(read_head(&layout), Ok(Head::Detached(FIRST.to_string())));
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(path.join(".git/HEAD"), "garbage\n").expect("garbage");
    // Borrow `&layout`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(read_head(&layout).is_err());
}

#[test]
fn a_loose_reference_wins_over_a_packed_one_and_a_packed_one_is_found_by_name() {
    let (root, path) = repository("ref: refs/heads/main\n", "");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `layout: GitLayout`.
    let layout: GitLayout = find_layout(&path, &environment(root.path())).expect("layout");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `format!`: builds or prints text (or a list) from a template.
    fs::write(path.join(".git/packed-refs"), format!("# pack-refs with: peeled\n{FIRST} refs/heads/main\n^{SECOND}\n{SECOND} refs/heads/other\n")).expect("packed");
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&layout`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lookup_ref(&layout, "refs/heads/main"), Ok(Some(FIRST.to_string())));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&layout`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lookup_ref(&layout, "refs/heads/other"), Ok(Some(SECOND.to_string())));
    // Borrow `&path`: lends the value without copying or giving it away.
    write_ref(&path, "refs/heads/main", SECOND);
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&layout`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lookup_ref(&layout, "refs/heads/main"), Ok(Some(SECOND.to_string())));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `None`: the present or the absent value of an `Option`.
    // Borrow `&layout`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lookup_ref(&layout, "refs/heads/absent"), Ok(None));
}

#[test]
fn a_symbolic_loose_reference_and_a_name_that_escapes_are_refused() {
    let (root, path) = repository("ref: refs/heads/main\n", "");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `layout: GitLayout`.
    let layout: GitLayout = find_layout(&path, &environment(root.path())).expect("layout");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(path.join(".git/refs/heads/alias"), "ref: refs/heads/main\n").expect("symbolic");
    // Borrow `&layout`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(lookup_ref(&layout, "refs/heads/alias").is_err());
    // Borrow `&layout`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(lookup_ref(&layout, "refs/../config").is_err());
    // Borrow `&layout`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(lookup_ref(&layout, "/etc/passwd").is_err());
}

#[test]
fn a_short_name_is_the_remote_tracking_branch_unless_something_shadows_it() {
    let (root, path) = repository("ref: refs/heads/main\n", "");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `layout: GitLayout`.
    let layout: GitLayout = find_layout(&path, &environment(root.path())).expect("layout");
    // Borrow `&path`: lends the value without copying or giving it away.
    write_ref(&path, "refs/remotes/origin/main", FIRST);
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&layout`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lookup_short(&layout, "origin/main"), Ok(Some(FIRST.to_string())));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `None`: the present or the absent value of an `Option`.
    // Borrow `&layout`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lookup_short(&layout, "origin/master"), Ok(None));
    // Borrow `&path`: lends the value without copying or giving it away.
    write_ref(&path, "refs/tags/origin/main", SECOND);
    // Borrow `&layout`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(lookup_short(&layout, "origin/main").is_err());
}

#[test]
fn the_upstream_of_a_branch_comes_from_its_remote_merge_and_the_fetch_refspec() {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let config = parse_config("[remote \"origin\"]\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n[branch \"main\"]\n\tremote = origin\n\tmerge = refs/heads/trunk\n[branch \"local\"]\n\tremote = .\n\tmerge = refs/heads/main\n[branch \"lonely\"]\n\tremote = origin\n").expect("parses");

    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&config`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(tracking_ref(&config, &layout_of(&config), "main"), Ok(Some("refs/remotes/origin/trunk".to_string())));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&config`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(tracking_ref(&config, &layout_of(&config), "local"), Ok(Some("refs/heads/main".to_string())));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `None`: the present or the absent value of an `Option`.
    // Borrow `&config`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(tracking_ref(&config, &layout_of(&config), "lonely"), Ok(None), "a remote without a merge reference is no upstream");
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `None`: the present or the absent value of an `Option`.
    // Borrow `&config`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(tracking_ref(&config, &layout_of(&config), "unknown"), Ok(None));
}

#[test]
fn an_exact_refspec_maps_one_reference_and_an_uncovered_reference_has_no_tracking_branch() {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let config = parse_config("[remote \"origin\"]\n\tfetch = refs/heads/main:refs/remotes/origin/primary\n[branch \"main\"]\n\tremote = origin\n\tmerge = refs/heads/main\n[branch \"side\"]\n\tremote = origin\n\tmerge = refs/heads/side\n").expect("parses");

    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&config`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(tracking_ref(&config, &layout_of(&config), "main"), Ok(Some("refs/remotes/origin/primary".to_string())));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `None`: the present or the absent value of an `Option`.
    // Borrow `&config`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(tracking_ref(&config, &layout_of(&config), "side"), Ok(None));
}

#[test]
fn a_merge_value_that_is_not_a_full_reference_and_a_one_sided_pattern_are_refused() {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let short = parse_config("[remote \"origin\"]\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n[branch \"main\"]\n\tremote = origin\n\tmerge = main\n").expect("parses");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let lopsided = parse_config("[remote \"origin\"]\n\tfetch = refs/heads/*:refs/remotes/origin/main\n[branch \"main\"]\n\tremote = origin\n\tmerge = refs/heads/main\n").expect("parses");

    // Borrow `&short`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(tracking_ref(&short, &layout_of(&short), "main").is_err());
    // Borrow `&lopsided`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(tracking_ref(&lopsided, &layout_of(&lopsided), "main").is_err());
}

#[test]
fn a_linked_worktree_finds_its_head_in_its_own_directory_and_the_rest_in_the_common_one() {
    let root = scratch();
    // Typed binding: `common: PathBuf`.
    let common: PathBuf = root.path().join("main/.git");
    // Typed binding: `own: PathBuf`.
    let own: PathBuf = common.join("worktrees/linked");
    // Typed binding: `linked: PathBuf`.
    let linked: PathBuf = root.path().join("linked");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(common.join("refs/heads")).expect("common");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(common.join("objects")).expect("objects");
    // Borrow `&own`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(&own).expect("own");
    // Borrow `&linked`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(&linked).expect("linked");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(common.join("config"), "").expect("config");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(own.join("HEAD"), "ref: refs/heads/topic\n").expect("head");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(common.join("HEAD"), "ref: refs/heads/main\n").expect("common head");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(own.join("commondir"), "../..\n").expect("commondir");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `format!`: builds or prints text (or a list) from a template.
    fs::write(linked.join(".git"), format!("gitdir: {}\n", own.display())).expect("gitfile");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `format!`: builds or prints text (or a list) from a template.
    fs::write(common.join("refs/heads/topic"), format!("{FIRST}\n")).expect("topic");

    // Borrow `&linked`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `layout: GitLayout`.
    let layout: GitLayout = find_layout(&linked, &environment(root.path())).expect("layout");

    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&layout`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(read_head(&layout), Ok(Head::Branch("refs/heads/topic".to_string())));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&layout`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(lookup_ref(&layout, "refs/heads/topic"), Ok(Some(FIRST.to_string())));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // Borrow `&linked`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    // The object names here are made up, so the base itself is declined until the object is seen to be a commit.
    assert!(native_base(&linked, true, &environment(root.path())).is_err());
}

#[test]
fn a_missing_dot_git_a_dot_git_file_without_gitdir_and_a_blocking_variable_are_refused() {
    let root = scratch();
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `blocked: NativeEnvironment`.
    let mut blocked: NativeEnvironment = environment(root.path());
    // Borrow `&blocked`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(find_layout(root.path(), &blocked).is_err());
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(root.path().join(".git"), "not a pointer\n").expect("file");
    // Borrow `&blocked`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(find_layout(root.path(), &blocked).is_err());
    // `Some`: the present or the absent value of an `Option`.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    blocked.blocking_variable = Some("GIT_DIR".to_string());
    // Borrow `&blocked`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(find_layout(root.path(), &blocked).unwrap_err().contains("GIT_DIR"));
}

#[test]
fn the_configuration_layers_override_in_gits_order_and_an_unreadable_file_is_refused() {
    let (root, path) = repository("ref: refs/heads/main\n", "[core]\n\tx = repository\n");
    // Typed binding: `home: PathBuf`.
    let home: PathBuf = root.path().join("home");
    // Borrow `&home`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(&home).expect("home");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(home.join(".gitconfig"), "[core]\n\tx = user\n\ty = user\n").expect("user");
    // `Some`: the present or the absent value of an `Option`.
    // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
    // Why:      The copy can be stored or moved elsewhere while the original stays in place.
    // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // structuredClone(value)  // or [...list] / {...object}
    // ```
    // Typed binding: `layered: NativeEnvironment`.
    let layered: NativeEnvironment = NativeEnvironment { home: Some(home.clone()), xdg_config_home: None, system_config: Some(root.path().join("none")), blocking_variable: None, git: crate::git_refs::GitProgram::default(), user_id: crate::git_refs::current_user_id() };
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `layout: GitLayout`.
    let layout: GitLayout = find_layout(&path, &layered).expect("layout");

    // Borrow `&layout`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let config = read_config(&layout, &layered, root_of(&layout), &std::cell::Cell::new(0)).expect("reads");

    // `None`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(config.last("core", None, "x"), Some("repository"));
    // `None`: the present or the absent value of an `Option`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(config.last("core", None, "y"), Some("user"));
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(home.join(".gitconfig"), "[core\n").expect("broken");
    // Borrow `&layout`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(read_config(&layout, &layered, root_of(&layout), &std::cell::Cell::new(0)).is_err());
}

#[test]
fn a_repository_that_asks_for_reftable_or_a_worktree_configuration_is_refused() {
    let (root, path) = repository("ref: refs/heads/main\n", "[extensions]\n\trefStorage = reftable\n");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `layout: GitLayout`.
    let layout: GitLayout = find_layout(&path, &environment(root.path())).expect("layout");
    // Borrow `&layout`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(read_config(&layout, &environment(root.path()), root_of(&layout), &std::cell::Cell::new(0)).unwrap_err().contains("extensions.refstorage"));
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(path.join(".git/config"), "[extensions]\n\tworktreeConfig = true\n").expect("worktree config");
    // Borrow `&layout`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(read_config(&layout, &environment(root.path()), root_of(&layout), &std::cell::Cell::new(0)).unwrap_err().contains("extensions.worktreeconfig"));
}

/// The repository root of a layout whose git directory is `<root>/.git`.
fn root_of(layout: &GitLayout) -> &Path {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    return layout.git_dir.parent().unwrap_or_else(|| { return Path::new("."); });
}
