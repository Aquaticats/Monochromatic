//! The third review of the Git reader (3a):
//! symbolic-link `HEAD`,
//! remote `.`,
//! path text,
//! object validity,
//! configuration scope,
//! the other system configuration places,
//! ownership,
//! and the one Git start that answers a declined choice.
//! Git itself is the oracle,
//! run with an isolated home and the same program as the fallback.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::fs;
// Imports `std::os::unix::fs::{PermissionsExt, symlink}`.
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt, symlink};
use std::process::Command;
// Imports `std::path::{Path, PathBuf}`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
use std::path::{Path, PathBuf};
// Imports `std::time::{Duration, Instant}`.
// What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
// Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: T[] / Set / Map / number
// ```
use std::time::{Duration, Instant};

// Imports `crate::git_base::resolve_trust_base_with`.
use crate::git_base::resolve_trust_base_with;
// Imports `crate::git_differential_tests::{Fixture, TO...`.
use crate::git_differential_tests::{Fixture, TOPIC_TRACKS_BASE, assert_agrees, assert_agrees_at, assert_declines, base_branch_at_first, environment, fixture, git, oracle, write_config, write_ref};
// Imports `crate::git_refs::{GitProgram, NativeEnviron...`.
use crate::git_refs::{GitProgram, NativeEnvironment};
// Imports `crate::test_scratch::scratch`.
use crate::test_scratch::scratch;

/// The real Git,
/// so the fallback runs the program the oracle runs.
const SYSTEM_GIT: &str = "/usr/bin/git";

/// The reader's environment whose fallback runs the real Git with the isolated home,
/// like the oracle.
fn controlled(fixture: &Fixture) -> NativeEnvironment {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let mut base: NativeEnvironment = environment(fixture.root.path());
    base.git = GitProgram {
        program: PathBuf::from(SYSTEM_GIT),
        bound: Duration::from_secs(30),
        clean_environment: true,
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        environment: vec![
            // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
            // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // String(value)  // JS strings are always owned values
            // ```
            ("PATH".to_string(), "/usr/bin:/bin".to_string()),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            ("HOME".to_string(), fixture.root.path().join("home").display().to_string()),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            ("GIT_CONFIG_NOSYSTEM".to_string(), "1".to_string()),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            ("LC_ALL".to_string(), "C".to_string()),
        ],
    };
    return base;
}

/// Requires the whole choice (the reader,
/// then the fallback) to equal Git's.
fn assert_end_to_end(fixture: &Fixture, label: &str) {
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // Typed binding: `expected: Option<(String, String)>`.
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let expected: Option<(String, String)> = oracle(fixture, &fixture.work);
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    let base = resolve_trust_base_with(&fixture.work, false, None, &controlled(fixture));
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
    // Why:      The copy can be stored or moved elsewhere while the original stays in place.
    // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // structuredClone(value)  // or [...list] / {...object}
    // ```
    // What:     `.ok()` turns a `Result` into an `Option` (dropping the error); `.ok_or(e)` turns an `Option` into a `Result` with the given error.
    // Why:      The next step wants a value-or-nothing, or a value-or-error, not the other form.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return compute(); } catch { return undefined; }
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    // Typed binding: `found: Option<(String, String)>`.
    let found: Option<(String, String)> = base.commit.clone().ok().map(|commit| { return (base.spelling.clone(), commit); });
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(found, expected, "{label}: the choice is {base:?} but Git chooses {expected:?}");
}

/// Writes a configuration whose upstream lives in an included file,
/// so the reader must decline and Git answers.
fn include_with(fixture: &Fixture, included_text: &str) {
    // Typed binding: `included: PathBuf`.
    let included: PathBuf = fixture.root.path().join("included.conf");
    // Borrow `&included`: lends the value without copying or giving it away.
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    fs::write(&included, included_text).expect("included file");
    // Borrow `&format`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    write_config(fixture, &format!("[include]\n\tpath = {}\n", included.display()));
}

/// Review finding (3a 1):
/// a symbolic-link `HEAD` is a symbolic reference,
/// not a detached object.
#[test]
fn a_symbolic_link_head_is_not_a_detached_head() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    base_branch_at_first(&fixture);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    write_config(&fixture, &format!("[core]\n\tpreferSymlinkRefs = true\n{TOPIC_TRACKS_BASE}"));
    // Typed binding: `head: PathBuf`.
    let head: PathBuf = fixture.work.join(".git/HEAD");
    // Borrow `&head`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::remove_file(&head).expect("remove HEAD");
    // Borrow `&head`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    symlink("refs/heads/topic", &head).expect("a symbolic-link HEAD");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "symbolic-link HEAD");
}

/// Review finding (3a 2):
/// the remote `.` maps through its fetch refspecs like any other remote.
#[test]
fn the_remote_dot_maps_through_its_fetch_refspecs() {
    let fixture = fixture("topic");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    git(fixture.root.path(), &fixture.work, &["branch", "base", &fixture.second]);
    // Borrow `&fixture.work.join`: lends the value without copying or giving it away.
    write_ref(&fixture.work.join(".git"), "refs/remotes/local/base", &fixture.first);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    write_config(&fixture, &format!("{TOPIC_TRACKS_BASE}[remote \".\"]\n\tfetch = +refs/heads/*:refs/remotes/local/*\n"));

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "remote dot with a fetch refspec");
}

/// Review finding (3a 2):
/// a merge name that exists only through Git's name expansion is Git's to resolve.
#[test]
fn a_merge_name_that_needs_name_expansion_is_left_to_git() {
    let fixture = fixture("topic");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["branch", "refs/heads/base", &fixture.first]);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, TOPIC_TRACKS_BASE);

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "merge name that needs expansion");
}

/// Review finding (3a 3):
/// the `.git` file's path keeps trailing spaces;
/// only the line ending is dropped.
#[test]
fn a_gitdir_pointer_keeps_trailing_spaces() {
    let fixture = fixture("topic");
    // Typed binding: `outer: PathBuf`.
    let outer: PathBuf = fixture.root.path().join("outer");
    // Borrow `&outer`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(&outer).expect("outer");
    // Borrow `&outer`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    git(fixture.root.path(), &outer, &["clone", "--quiet", "--bare", fixture.work.to_str().expect("utf8"), "git-a"]);
    // Borrow `&outer`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    git(fixture.root.path(), &outer, &["clone", "--quiet", "--bare", fixture.work.to_str().expect("utf8"), "git-a "]);
    // Borrow `&outer.join`: lends the value without copying or giving it away.
    git(fixture.root.path(), &outer.join("git-a "), &["update-ref", "refs/heads/topic", &fixture.first]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(outer.join(".git"), "gitdir: git-a \n").expect("pointer");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees_at(&fixture, &outer, "gitdir with a trailing space");
}

/// Review finding (3a 4):
/// a loose object with a readable prefix and a damaged rest is not a commit.
#[test]
fn a_truncated_or_malformed_loose_object_is_not_confirmed_as_a_commit() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    // Typed binding: `common: PathBuf`.
    let common: PathBuf = fixture.work.join(".git");
    // Borrow `&fixture.second`: lends the value without copying or giving it away.
    // Typed binding: `loose: PathBuf`.
    let loose: PathBuf = common.join("objects").join(&fixture.second[..2]).join(&fixture.second[2..]);
    // Borrow `&loose`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `whole: Vec<u8>`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let whole: Vec<u8> = fs::read(&loose).expect("loose commit");
    // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
    // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Ok(value)  is  return value;
    // // Err(error) is  throw error;   (the caller sees it as a returned value)
    // ```
    // Borrow `&common`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(confirm_commit(&common, &fixture.second), Ok(()), "control: the whole object is a commit");

    // Borrow `&loose`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&loose, fs::Permissions::from_mode(0o644)).expect("writable");
    // Borrow `&loose`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&loose, &whole[..whole.len() / 2]).expect("truncate");
    // Borrow `&common`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(confirm_commit(&common, &fixture.second).is_err(), "a truncated stream");

    // `let mut`: a binding that may change.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    // Borrow `&mut encoder`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    std::io::Write::write_all(&mut encoder, b"commit 5\0hello").expect("compress");
    // Borrow `&loose`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&loose, encoder.finish().expect("finish")).expect("write a malformed commit");
    // Borrow `&common`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(confirm_commit(&common, &fixture.second).is_err(), "a body that is not a commit");
}

/// Review finding (3a 4):
/// a pack entry that is cut off after its type byte is not a commit.
#[test]
fn a_truncated_pack_entry_is_not_confirmed_as_a_commit() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["gc", "--quiet"]);
    // Typed binding: `common: PathBuf`.
    let common: PathBuf = fixture.work.join(".git");
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&common`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(confirm_commit(&common, &fixture.second), Ok(()), "control: the packed commit is confirmed");
    // Closure `|entry|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `pack: PathBuf`.
    let pack: PathBuf = fs::read_dir(common.join("objects/pack")).expect("packs").flatten().map(|entry| { return entry.path(); }).find(|path| { return path.extension().is_some_and(|extension| { return extension == "pack"; }); }).expect("a pack");
    // Borrow `&pack`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&pack, fs::Permissions::from_mode(0o644)).expect("writable");
    // Borrow `&pack`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::OpenOptions::new().write(true).open(&pack).expect("open").set_len(13).expect("cut the pack after one entry byte");

    // Borrow `&common`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(confirm_commit(&common, &fixture.second).is_err());
}

/// Review finding (3a 5):
/// a corrupt lower-priority candidate must not hide an existing upstream.
#[test]
fn a_corrupt_default_branch_does_not_hide_the_upstream() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    base_branch_at_first(&fixture);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    include_with(&fixture, TOPIC_TRACKS_BASE);
    // Typed binding: `bad: &str`.
    let bad: &str = "9999999999999999999999999999999999999999";
    // Borrow `&fixture.work.join`: lends the value without copying or giving it away.
    write_ref(&fixture.work.join(".git"), "refs/remotes/origin/master", bad);
    // Typed binding: `loose: PathBuf`.
    let loose: PathBuf = fixture.work.join(".git/objects/99");
    // Borrow `&loose`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(&loose).expect("object directory");
    // Borrow `&bad`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(loose.join(&bad[2..]), b"this is not a zlib stream").expect("a corrupt loose object");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_end_to_end(&fixture, "corrupt origin/master with a valid upstream");
}

/// The declined choice agrees with Git end to end for the ordinary shapes.
#[test]
fn a_declined_choice_agrees_with_git_for_the_ordinary_shapes() {
    let upstream = fixture("topic");
    // Borrow `&upstream`: lends the value without copying or giving it away.
    base_branch_at_first(&upstream);
    // Borrow `&upstream`: lends the value without copying or giving it away.
    include_with(&upstream, TOPIC_TRACKS_BASE);
    // Borrow `&upstream`: lends the value without copying or giving it away.
    assert_end_to_end(&upstream, "upstream through an include");

    let default_branch = fixture("topic");
    // Borrow `&default_branch.work.join`: lends the value without copying or giving it away.
    write_ref(&default_branch.work.join(".git"), "refs/remotes/origin/main", &default_branch.first);
    // Borrow `&default_branch`: lends the value without copying or giving it away.
    include_with(&default_branch, "");
    // Borrow `&default_branch`: lends the value without copying or giving it away.
    assert_end_to_end(&default_branch, "origin/main through an include");

    let head_only = fixture("topic");
    // Borrow `&head_only`: lends the value without copying or giving it away.
    include_with(&head_only, "");
    // Borrow `&head_only`: lends the value without copying or giving it away.
    assert_end_to_end(&head_only, "HEAD only");
}

/// Review finding (3a 6):
/// a bare repository format version is Git's to reject.
#[test]
fn a_bare_repository_format_version_is_left_to_git() {
    let fixture = fixture("topic");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(fixture.work.join(".git/config"), "[core]\n\trepositoryformatversion\n").expect("config");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "bare format version");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_declines(&fixture, "bare format version");
}

/// Review finding (3a 6):
/// an invalid refspec of a remote that is not used still fails Git's remote configuration.
#[test]
fn an_invalid_refspec_of_an_unused_remote_is_left_to_git() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    base_branch_at_first(&fixture);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    write_config(&fixture, &format!("{TOPIC_TRACKS_BASE}[remote \"unused\"]\n\tfetch = refs/heads/**:refs/remotes/unused/**\n"));

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "invalid refspec of an unused remote");
}

/// Review finding (3a 6):
/// a required string written without a value on another branch breaks Git's remote configuration.
#[test]
fn a_bare_remote_key_on_another_branch_is_left_to_git() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    base_branch_at_first(&fixture);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    write_config(&fixture, &format!("{TOPIC_TRACKS_BASE}[branch \"other\"]\n\tremote\n"));

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "bare remote on another branch");
}

/// Review finding (3a 8):
/// a repository that another user owns is Git's to accept or refuse.
#[test]
fn a_repository_owned_by_another_user_is_left_to_git() {
    let fixture = fixture("topic");
    // `let mut`: a binding that may change.
    // Typed binding: `environment: NativeEnvironment`.
    let mut environment: NativeEnvironment = environment(fixture.root.path());
    environment.user_id = environment.user_id.wrapping_add(1);

    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    let answered = crate::git_native::native_base(&fixture.work, false, &environment);

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(answered.is_err(), "{answered:?}");
}

/// Writes an executable shell script.
fn script(directory: &Path, name: &str, body: &str) -> PathBuf {
    // Typed binding: `path: PathBuf`.
    let path: PathBuf = directory.join(name);
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `format!`: builds or prints text (or a list) from a template.
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("script");
    // Borrow `&path`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("executable");
    return path;
}

/// Review finding (3a 9) and the owner's rule of 2026-11-03:
/// a Git that outlives its bound is stopped and named.
#[test]
fn a_git_that_outlives_its_bound_is_stopped_and_named() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    include_with(&fixture, "");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // `let mut`: a binding that may change.
    // Typed binding: `environment: NativeEnvironment`.
    let mut environment: NativeEnvironment = controlled(&fixture);
    let directory = scratch();
    environment.git.program = script(directory.path(), "slow-git", "sleep 5");
    environment.git.bound = Duration::from_millis(300);
    let started = Instant::now();

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    let base = resolve_trust_base_with(&fixture.work, false, None, &environment);

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    // Typed binding: `message: String`.
    let message: String = base.commit.expect_err("a stopped git has no answer");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(message.contains("did not finish"), "{message}");
}

/// Review finding (3a 9):
/// a Git that exits without reading its input is reaped,
/// and the choice reports the failure.
#[test]
fn a_git_that_exits_without_reading_is_reaped_and_reported() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    include_with(&fixture, "");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // `let mut`: a binding that may change.
    // Typed binding: `environment: NativeEnvironment`.
    let mut environment: NativeEnvironment = controlled(&fixture);
    let directory = scratch();
    environment.git.program = script(directory.path(), "quitting-git", "exit 1");

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    let base = resolve_trust_base_with(&fixture.work, false, None, &environment);

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(base.commit.is_err(), "{base:?}");
}

/// Compresses a loose object body (header and payload) the way Git stores it.
fn deflate(bytes: &[u8]) -> Vec<u8> {
    // `let mut`: a binding that may change.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    // Borrow `&mut encoder`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    std::io::Write::write_all(&mut encoder, bytes).expect("compress");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    return encoder.finish().expect("finish");
}

/// Replaces the loose object of the second commit with the given stored bytes and returns the repository directory.
fn replace_loose_second(fixture: &Fixture, stored: &[u8]) -> PathBuf {
    // Typed binding: `common: PathBuf`.
    let common: PathBuf = fixture.work.join(".git");
    // Borrow `&fixture.second`: lends the value without copying or giving it away.
    // Typed binding: `loose: PathBuf`.
    let loose: PathBuf = common.join("objects").join(&fixture.second[..2]).join(&fixture.second[2..]);
    // Borrow `&loose`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&loose, fs::Permissions::from_mode(0o644)).expect("writable");
    // Borrow `&loose`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&loose, stored).expect("replace the loose object");
    return common;
}

fn a_malformed_parent_line_is_not_a_commit() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    let tree: String = git(fixture.root.path(), &fixture.work, &["rev-parse", "HEAD^{tree}"]);
    let good: String = format!("tree {tree}\nparent {}\nauthor A <a@b> 1 +0000\ncommitter A <a@b> 1 +0000\n\nmessage\n", fixture.first);
    let bad: String = format!("tree {tree}\nparent zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz\nauthor A <a@b> 1 +0000\ncommitter A <a@b> 1 +0000\n\nmessage\n");
    // Both bodies are stored under their own true names (round 6, 5a F3 B), so only the parent line differs between them.
    let good_name: String = hash_commit(&fixture, &good);
    let bad_name: String = hash_commit(&fixture, &bad);
    let common: PathBuf = fixture.work.join(".git");

    assert_eq!(confirm_commit(&common, &good_name), Ok(()), "control: a well-formed body under its own name is accepted");
    assert!(!git_peels(&fixture, &bad_name), "oracle: Git rejects the malformed parent line");
    assert!(confirm_commit(&common, &bad_name).is_err(), "a malformed parent line was accepted");
}

/// Review finding (4a 1):
/// the size in a loose header is canonical decimal.
#[test]
fn a_noncanonical_loose_size_is_not_a_commit() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // Typed binding: `tree: String`.
    let tree: String = git(fixture.root.path(), &fixture.work, &["rev-parse", "HEAD^{tree}"]);
    // Typed binding: `body: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let body: String = format!("tree {tree}\nauthor A <a@b> 1 +0000\ncommitter A <a@b> 1 +0000\n\nmessage\n");
    // Macro `format!`: builds or prints text (or a list) from a template.
    for size in [format!("+{}", body.len()), format!("0{}", body.len())] {
        // Borrow `&fixture`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let common = replace_loose_second(&fixture, &deflate(format!("commit {size}\0{body}").as_bytes()));
        // Borrow `&common`: lends the value without copying or giving it away.
        // Macro `assert!`: builds or prints text (or a list) from a template.
        assert!(confirm_commit(&common, &fixture.second).is_err(), "the size {size} was accepted");
    }
}

/// Review finding (4a 2):
/// a zlib stream without its checksum is not complete,
/// even though all of its output is available.
#[test]
fn a_zlib_stream_without_its_checksum_is_not_a_commit() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    // Typed binding: `common: PathBuf`.
    let common: PathBuf = fixture.work.join(".git");
    // Borrow `&fixture.second`: lends the value without copying or giving it away.
    // Typed binding: `loose: PathBuf`.
    let loose: PathBuf = common.join("objects").join(&fixture.second[..2]).join(&fixture.second[2..]);
    // Borrow `&loose`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `whole: Vec<u8>`.
    let whole: Vec<u8> = fs::read(&loose).expect("loose commit");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    replace_loose_second(&fixture, &whole[..whole.len() - 4]);

    // Borrow `&common`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(confirm_commit(&common, &fixture.second).is_err(), "a stream without its trailer was accepted");
}

/// Review finding (4a 2):
/// Git checks the pack's object count against its index when it opens the pack.
#[test]
fn a_pack_whose_object_count_disagrees_with_its_index_is_not_trusted() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["gc", "--quiet"]);
    // Typed binding: `common: PathBuf`.
    let common: PathBuf = fixture.work.join(".git");
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&common`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(confirm_commit(&common, &fixture.second), Ok(()), "control: the packed commit is confirmed");
    // Closure `|entry|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `pack: PathBuf`.
    let pack: PathBuf = fs::read_dir(common.join("objects/pack")).expect("packs").flatten().map(|entry| { return entry.path(); }).find(|path| { return path.extension().is_some_and(|extension| { return extension == "pack"; }); }).expect("a pack");
    // Borrow `&pack`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&pack, fs::Permissions::from_mode(0o644)).expect("writable");
    // Borrow `&pack`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // `let mut`: a binding that may change.
    // Typed binding: `bytes: Vec<u8>`.
    let mut bytes: Vec<u8> = fs::read(&pack).expect("pack");
    // Borrow `&[`: lends the value without copying or giving it away.
    bytes[8..12].copy_from_slice(&[0, 0, 0, 0]);
    // Borrow `&pack`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&pack, bytes).expect("patch the object count");

    // Borrow `&common`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(confirm_commit(&common, &fixture.second).is_err(), "a pack with a wrong object count was trusted");
}

/// Review finding (4a 2):
/// the checksum at the end of the pack must be the one its index records.
#[test]
fn a_pack_whose_trailer_disagrees_with_its_index_is_not_trusted() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["gc", "--quiet"]);
    // Typed binding: `common: PathBuf`.
    let common: PathBuf = fixture.work.join(".git");
    // Closure `|entry|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `pack: PathBuf`.
    let pack: PathBuf = fs::read_dir(common.join("objects/pack")).expect("packs").flatten().map(|entry| { return entry.path(); }).find(|path| { return path.extension().is_some_and(|extension| { return extension == "pack"; }); }).expect("a pack");
    // Borrow `&pack`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&pack, fs::Permissions::from_mode(0o644)).expect("writable");
    // Borrow `&pack`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // `let mut`: a binding that may change.
    // Typed binding: `bytes: Vec<u8>`.
    let mut bytes: Vec<u8> = fs::read(&pack).expect("pack");
    // Typed binding: `last: usize`.
    let last: usize = bytes.len() - 1;
    bytes[last] ^= 0xff;
    // Borrow `&pack`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&pack, bytes).expect("patch the trailer");

    // Borrow `&common`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(confirm_commit(&common, &fixture.second).is_err(), "a pack with a wrong trailer was trusted");
}

/// Review finding (4a 7):
/// only regular files are opened as objects,
/// so a link (or a FIFO,
/// which would block) is Git's to judge.
#[test]
fn a_loose_object_that_is_a_link_is_not_read() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    // Typed binding: `common: PathBuf`.
    let common: PathBuf = fixture.work.join(".git");
    // Borrow `&fixture.second`: lends the value without copying or giving it away.
    // Typed binding: `loose: PathBuf`.
    let loose: PathBuf = common.join("objects").join(&fixture.second[..2]).join(&fixture.second[2..]);
    // Typed binding: `moved: PathBuf`.
    let moved: PathBuf = fixture.root.path().join("moved-object");
    // Borrow `&loose`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::rename(&loose, &moved).expect("move the object");
    // Borrow `&moved`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    symlink(&moved, &loose).expect("link it back");

    // Borrow `&common`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(confirm_commit(&common, &fixture.second).is_err());
}

/// Review finding (4a 5):
/// a `.git` entry that is a link is not followed for the ownership decision;
/// Git's is an `lstat`.
#[test]
fn a_dot_git_link_is_left_to_git() {
    let fixture = fixture("topic");
    // Typed binding: `outer: PathBuf`.
    let outer: PathBuf = fixture.root.path().join("outer");
    // Borrow `&outer`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(&outer).expect("outer");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    symlink(fixture.work.join(".git"), outer.join(".git")).expect("a .git link");

    // Borrow `&outer`: lends the value without copying or giving it away.
    let answered = crate::git_native::native_base(&outer, false, &environment(fixture.root.path()));

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(answered.is_err(), "{answered:?}");
}

/// Review finding (4a 8):
/// an empty `commondir` file is not an absent one;
/// Git refuses it.
#[test]
fn an_empty_commondir_is_left_to_git() {
    let fixture = fixture("topic");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(fixture.work.join(".git/commondir"), "").expect("an empty commondir");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "empty commondir");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_declines(&fixture, "empty commondir");
}

/// Review finding (4a 3):
/// a valued remote setting that Git parses as a boolean must be one.
#[test]
fn an_invalid_boolean_of_an_unused_remote_is_left_to_git() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    base_branch_at_first(&fixture);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    write_config(&fixture, &format!("{TOPIC_TRACKS_BASE}[remote \"unused\"]\n\tmirror = definitely-not-a-boolean\n"));

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "invalid boolean on an unused remote");
}

/// Review finding (4a 3):
/// Git loads push refspecs too.
#[test]
fn an_invalid_push_refspec_of_an_unused_remote_is_left_to_git() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    base_branch_at_first(&fixture);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    write_config(&fixture, &format!("{TOPIC_TRACKS_BASE}[remote \"unused\"]\n\tpush = refs/heads/**:refs/heads/**\n"));

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "invalid push refspec on an unused remote");
}

/// Review finding (4a 6):
/// when the batch stopped early,
/// a candidate whose query timed out is not read as "not a commit".
#[test]
fn a_timeout_in_the_sequential_fallback_is_reported_not_skipped() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    include_with(&fixture, "");
    let directory = scratch();
    // Macro `format!`: builds or prints text (or a list) from a template.
    let body = format!(
        "case \"$1\" in\n  cat-file) head -n 1 >/dev/null; echo '{second} commit 100'; exit 128;;\n  rev-parse) case \"$*\" in *upstream*) sleep 5;; *) echo {second};; esac;;\nesac",
        second = fixture.second
    );
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // `let mut`: a binding that may change.
    // Typed binding: `environment: NativeEnvironment`.
    let mut environment: NativeEnvironment = controlled(&fixture);
    // Borrow `&body`: lends the value without copying or giving it away.
    environment.git.program = script(directory.path(), "fake-git", &body);
    environment.git.bound = Duration::from_millis(400);

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    let base = resolve_trust_base_with(&fixture.work, false, None, &environment);

    // Typed binding: `message: String`.
    let message: String = base.commit.expect_err("the upstream query did not finish, so no lower candidate may be chosen");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(message.contains("did not finish"), "{message}");
}

/// A Git stand-in that answers `git var GIT_CONFIG_SYSTEM` with the given path (counting the questions in `asked`) and runs the real
/// Git for everything else.
fn git_that_reports_its_system_config(directory: &Path, reported: &Path, asked: &Path) -> PathBuf {
    // Macro `format!`: builds or prints text (or a list) from a template.
    let body = format!("if [ \"$1\" = var ]; then echo x >> {asked}; echo {reported}; exit 0; fi\nexec {SYSTEM_GIT} \"$@\"", asked = asked.display(), reported = reported.display());
    // Borrow `&body`: lends the value without copying or giving it away.
    return script(directory, "reporting-git", &body);
}

/// Review finding (4a 4):
/// the system configuration is the file the Git that answers reports,
/// not an assumed `/etc/gitconfig`.
#[test]
fn the_system_configuration_is_the_file_the_answering_git_reports() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    base_branch_at_first(&fixture);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, "");
    let directory = scratch();
    // Typed binding: `reported: PathBuf`.
    let reported: PathBuf = fixture.root.path().join("elsewhere-gitconfig");
    // Borrow `&reported`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&reported, TOPIC_TRACKS_BASE).expect("the reported system configuration");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // `let mut`: a binding that may change.
    // Typed binding: `environment: NativeEnvironment`.
    let mut environment: NativeEnvironment = controlled(&fixture);
    // `None`: the present or the absent value of an `Option`.
    environment.system_config = None;
    // Borrow `&reported`: lends the value without copying or giving it away.
    environment.git.program = git_that_reports_its_system_config(directory.path(), &reported, &directory.path().join("asked"));

    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let answered = crate::git_native::native_base(&fixture.work, false, &environment).expect("answered from the reported file");

    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!((answered.spelling.as_str(), answered.commit), ("@{upstream}", Ok(fixture.first.clone())));
}

/// The answer is cached by the identity of the Git program,
/// so the question is asked once,
/// not on every call.
/// Round 6 (5a F5): a compiled Git program is asked once; its identity (file, size, times, relevant environment) is the key.
#[test]
fn the_reported_system_configuration_of_a_compiled_git_is_asked_once() {
    let fixture = fixture("topic");
    write_config(&fixture, "");
    let directory = scratch();
    let copy: PathBuf = directory.path().join("git");
    fs::copy(SYSTEM_GIT, &copy).expect("a copy of the compiled Git");
    let mut environment: NativeEnvironment = controlled(&fixture);
    environment.system_config = None;
    environment.git.program = copy;
    // The controlled environment sets GIT_CONFIG_NOSYSTEM, under which Git reports no system configuration at all.
    environment.git.environment.retain(|(name, _)| { return name != "GIT_CONFIG_NOSYSTEM"; });

    let first = crate::git_native::native_base(&fixture.work, false, &environment).expect("first");
    let second = crate::git_native::native_base(&fixture.work, false, &environment).expect("second");

    assert_eq!(first.git_starts, 1, "the question is a Git start and is counted");
    assert_eq!(second.git_starts, 0, "the cached answer costs no start");
}

/// Round 6 (5a F5): an executable that is not a compiled program may forward to any Git, so its answer is never reused.
#[test]
fn a_wrapper_script_is_asked_every_time() {
    let fixture = fixture("topic");
    write_config(&fixture, "");
    let directory = scratch();
    let asked: PathBuf = directory.path().join("asked");
    let reported: PathBuf = fixture.root.path().join("elsewhere-gitconfig");
    fs::write(&reported, "").expect("the reported system configuration");
    let mut environment: NativeEnvironment = controlled(&fixture);
    environment.system_config = None;
    environment.git.program = git_that_reports_its_system_config(directory.path(), &reported, &asked);

    let _first = crate::git_native::native_base(&fixture.work, false, &environment).expect("first");
    let _second = crate::git_native::native_base(&fixture.work, false, &environment).expect("second");

    assert_eq!(fs::read_to_string(&asked).expect("the question log").lines().count(), 2, "a wrapper's earlier answer was reused");
}

/// Round 6 (5a F5): the first executable on the search path is the one that is asked, as `execvp` would choose it.
#[test]
fn a_file_that_is_not_executable_does_not_hide_the_git_after_it_on_the_search_path() {
    let fixture = fixture("topic");
    write_config(&fixture, "");
    let directory = scratch();
    let blocked = directory.path().join("blocked");
    let usable = directory.path().join("usable");
    fs::create_dir_all(&blocked).expect("dir");
    fs::create_dir_all(&usable).expect("dir");
    fs::write(blocked.join("git"), "not executable\n").expect("a file named git that cannot run");
    let asked: PathBuf = directory.path().join("asked");
    let reported: PathBuf = fixture.root.path().join("elsewhere-gitconfig");
    fs::write(&reported, "").expect("the reported system configuration");
    let chosen = git_that_reports_its_system_config(&usable, &reported, &asked);
    fs::rename(&chosen, usable.join("git")).expect("rename");
    let mut environment: NativeEnvironment = controlled(&fixture);
    environment.system_config = None;
    environment.git.program = PathBuf::from("git");
    environment.git.environment = vec![("PATH".to_string(), format!("{}:{}", blocked.display(), usable.display()))];

    let answered = crate::git_native::native_base(&fixture.work, false, &environment);

    assert!(answered.is_ok(), "{answered:?}");
    assert_eq!(fs::read_to_string(&asked).expect("the question log").lines().count(), 1, "the usable Git was not asked");
}

#[test]
fn a_git_that_cannot_report_its_system_configuration_is_not_assumed() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, "");
    let directory = scratch();
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // `let mut`: a binding that may change.
    // Typed binding: `environment: NativeEnvironment`.
    let mut environment: NativeEnvironment = controlled(&fixture);
    // `None`: the present or the absent value of an `Option`.
    environment.system_config = None;
    environment.git.program = script(directory.path(), "old-git", "exit 1");

    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    let answered = crate::git_native::native_base(&fixture.work, false, &environment);

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(answered.is_err(), "{answered:?}");
}

/// A Git that answers the batch by stopping early, and whose upstream query ends as `upstream_query` says;
/// any other query resolves the second commit.
fn git_whose_upstream_query_fails(directory: &Path, fixture: &Fixture, upstream_query: &str) -> PathBuf {
    let body = format!(
        "case \"$1\" in\n  cat-file) head -n 1 >/dev/null; echo '{second} commit 100'; exit 128;;\n  rev-parse) case \"$*\" in *upstream*) {upstream_query};; *) echo {second};; esac;;\nesac",
        second = fixture.second
    );
    return script(directory, "failing-git", &body);
}

/// Round 6 (5a F4): a Git that ends fatally is an operational failure with its message,
/// never "not a commit" that lets a lower candidate be chosen.
#[test]
fn a_fatal_git_exit_in_the_sequential_fallback_is_reported_with_its_message() {
    let fixture = fixture("topic");
    include_with(&fixture, "");
    let directory = scratch();
    let mut environment: NativeEnvironment = controlled(&fixture);
    environment.git.program = git_whose_upstream_query_fails(directory.path(), &fixture, "echo 'fatal: could not fetch the promised object' >&2; exit 128");

    let base = resolve_trust_base_with(&fixture.work, false, None, &environment);

    let message: String = base.commit.expect_err("a fatal Git must not let a lower candidate be chosen");
    assert!(message.contains("could not fetch the promised object"), "{message}");
}

/// Round 6 (5a F4): a Git killed by a signal is an operational failure.
#[test]
fn a_git_killed_by_a_signal_in_the_sequential_fallback_is_reported() {
    let fixture = fixture("topic");
    include_with(&fixture, "");
    let directory = scratch();
    let mut environment: NativeEnvironment = controlled(&fixture);
    environment.git.program = git_whose_upstream_query_fails(directory.path(), &fixture, "kill -KILL $$");

    let base = resolve_trust_base_with(&fixture.work, false, None, &environment);

    let message: String = base.commit.expect_err("a killed Git must not let a lower candidate be chosen");
    assert!(message.contains("signal"), "{message}");
}

/// Round 6 (5a F4): the quiet negative answer of Git (exit 1, nothing printed) still means "not a commit".
#[test]
fn a_quiet_negative_answer_still_lets_the_next_candidate_be_tried() {
    let fixture = fixture("topic");
    include_with(&fixture, "");
    let directory = scratch();
    let mut environment: NativeEnvironment = controlled(&fixture);
    environment.git.program = git_whose_upstream_query_fails(directory.path(), &fixture, "exit 1");

    let base = resolve_trust_base_with(&fixture.work, false, None, &environment);

    assert!(base.commit.is_ok(), "{base:?}");
}

/// A Git that logs every start and answers the system-configuration question with an empty file; all else goes to the real Git.
fn git_that_logs_every_start(directory: &Path, reported: &Path, log: &Path) -> PathBuf {
    let body = format!("echo x >> {log}\nif [ \"$1\" = var ]; then echo {reported}; exit 0; fi\nexec {SYSTEM_GIT} \"$@\"", log = log.display(), reported = reported.display());
    return script(directory, "logging-git", &body);
}

/// Round 6 (5a F6): the Git starts of an unsuccessful native attempt are counted with the starts of the fallback.
#[test]
fn a_declined_choice_counts_the_start_that_asked_for_the_system_configuration() {
    let fixture = fixture("topic");
    base_branch_at_first(&fixture);
    write_config(&fixture, &format!("{TOPIC_TRACKS_BASE}[branch \"other\"]\n\tremote\n"));
    let directory = scratch();
    let log: PathBuf = directory.path().join("starts");
    let reported: PathBuf = fixture.root.path().join("empty-system-gitconfig");
    fs::write(&reported, "").expect("the reported system configuration");
    let mut environment: NativeEnvironment = controlled(&fixture);
    environment.system_config = None;
    environment.git.program = git_that_logs_every_start(directory.path(), &reported, &log);

    let base = resolve_trust_base_with(&fixture.work, false, None, &environment);

    let started: usize = fs::read_to_string(&log).expect("the start log").lines().count();
    assert!(base.commit.is_ok(), "{base:?}");
    assert_eq!(usize::try_from(base.git_starts).expect("count"), started, "reported {} starts, Git was started {started} times", base.git_starts);
}

/// Round 6 (5a F7): a path that was replaced by a FIFO after it was checked cannot make the reader wait.
#[test]
fn opening_a_fifo_for_reading_returns_without_waiting() {
    let directory = scratch();
    let fifo: PathBuf = directory.path().join("object");
    let made = Command::new("mkfifo").arg(&fifo).status().expect("mkfifo");
    assert!(made.success());
    let (sender, receiver) = std::sync::mpsc::channel();
    let probe: PathBuf = fifo.clone();
    std::thread::spawn(move || { let _ = sender.send(crate::git_objects::open_regular(&probe).is_err()); });

    let answered = receiver.recv_timeout(Duration::from_secs(2));

    assert_eq!(answered, Ok(true), "the open of a FIFO waited for a writer");
    // Release a reader that may still be blocked, so no thread outlives the test.
    let _ = fs::OpenOptions::new().write(true).custom_flags(0o4000).open(&fifo);
}

/// Writes a commit object with the given body through Git and returns its name.
/// `literally` skips Git's format check, so malformed bodies can be stored under their true name.
fn hash_commit(fixture: &Fixture, body: &str) -> String {
    use std::io::Write;
    let mut git = Command::new(SYSTEM_GIT)
        .arg("-C")
        .arg(&fixture.work)
        .args(["hash-object", "-t", "commit", "-w", "--literally", "--stdin"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("git hash-object");
    git.stdin.take().expect("stdin").write_all(body.as_bytes()).expect("write the body");
    let output = git.wait_with_output().expect("git ends");
    return String::from_utf8_lossy(&output.stdout).trim().to_string();
}

/// Whether Git itself peels the name to a commit.
fn git_peels(fixture: &Fixture, name: &str) -> bool {
    let status = Command::new(SYSTEM_GIT).arg("-C").arg(&fixture.work).args(["rev-parse", "--verify", "--quiet", &format!("{name}^{{commit}}")]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().expect("git");
    return status.success();
}

/// Round 6 (5a F3 B): object files hold what their name says.
#[test]
fn a_commit_stored_under_another_name_is_not_confirmed() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    let common: PathBuf = fixture.work.join(".git");
    let other: PathBuf = common.join("objects").join(&fixture.first[..2]).join(&fixture.first[2..]);
    let bytes: Vec<u8> = fs::read(&other).expect("a valid commit");
    assert_eq!(confirm_commit(&common, &fixture.first), Ok(()), "control: the commit under its own name is confirmed");
    replace_loose_second(&fixture, &bytes);

    assert!(confirm_commit(&common, &fixture.second).is_err(), "a commit under another commit's name was confirmed");
}

/// Round 6 (5a F3 A): Git needs bytes after the tree line.
#[test]
fn a_tree_only_commit_body_is_not_a_commit() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    let tree: String = git(fixture.root.path(), &fixture.work, &["rev-parse", "HEAD^{tree}"]);
    let name: String = hash_commit(&fixture, &format!("tree {tree}\n"));
    assert!(!git_peels(&fixture, &name), "oracle: Git rejects a tree-only commit");

    let common: PathBuf = fixture.work.join(".git");
    assert!(confirm_commit(&common, &name).is_err(), "a tree-only commit was confirmed");
}

/// Round 6 (5a F3 A): Git needs bytes after the last parent line too.
#[test]
fn a_commit_body_that_ends_with_a_parent_line_is_not_a_commit() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    let tree: String = git(fixture.root.path(), &fixture.work, &["rev-parse", "HEAD^{tree}"]);
    let name: String = hash_commit(&fixture, &format!("tree {tree}\nparent {}\n", fixture.first));
    assert!(!git_peels(&fixture, &name), "oracle: Git rejects a commit that ends with its parent line");

    let common: PathBuf = fixture.work.join(".git");
    assert!(confirm_commit(&common, &name).is_err(), "a commit that ends with a parent line was confirmed");
}

/// Round 6 (5a F3 C): a loose object file ends where its zlib stream ends.
#[test]
fn bytes_after_the_stream_of_a_loose_commit_are_not_confirmed() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    let tree: String = git(fixture.root.path(), &fixture.work, &["rev-parse", "HEAD^{tree}"]);
    let name: String = hash_commit(&fixture, &format!("tree {tree}\nauthor A <a@b> 1 +0000\ncommitter A <a@b> 1 +0000\n\nmessage\n"));
    let common: PathBuf = fixture.work.join(".git");
    assert_eq!(confirm_commit(&common, &name), Ok(()), "control: the well-formed commit is confirmed");
    let loose: PathBuf = common.join("objects").join(&name[..2]).join(&name[2..]);
    fs::set_permissions(&loose, fs::Permissions::from_mode(0o644)).expect("writable");
    let mut bytes: Vec<u8> = fs::read(&loose).expect("loose");
    bytes.extend_from_slice(b"garbage");
    fs::write(&loose, bytes).expect("append garbage");
    assert!(!git_peels(&fixture, &name), "oracle: Git rejects garbage at the end of a loose object");

    assert!(confirm_commit(&common, &name).is_err(), "a loose commit with trailing bytes was confirmed");
}
