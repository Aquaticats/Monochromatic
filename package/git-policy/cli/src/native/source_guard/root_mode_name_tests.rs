//! What:
//!  The pin of the one module allowed to read the name this program was started
//!  under,
//!  and the wiring assertions no text pin can see.
//! Why:
//!  Two of the owner's answers meet in one file:
//!  only the ZED history event's own name selects a root mode,
//!  and every other mode is reached through the words
//!  (`doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`;
//!  `doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
//!  "Choice 1:
//!  what selects a root mode").
//!  A list of forbidden spellings can only be as complete as the spellings
//!  someone thought of,
//!  so the module's whole code text is pinned line for line:
//!  any line added,
//!  removed,
//!  reordered,
//!  reworded or re-spelled fails here.
//!  The four assertions beside the pin hold what a text pin cannot see:
//!  which file the crate compiles as that module,
//!  what its declaration stands under,
//!  the words' first word,
//!  and the installed path's file name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(codeLines(readFileSync('worktree_clone_mount_handler/root_mode_names.rs', 'utf8'))).toEqual(NAMES_CODE);
//! ```

use super::{
    assert_whole_line_comments_only,
    pinned_code_lines,
    squeezed,
};
use crate::foundation::wrapper_names_old_and_new::COMMAND_WORD;
use std::path::Path;

/// The one module allowed to read the name this program was started under.
const NAMES_SOURCE: &str = include_str!("../worktree_clone_mount_handler/root_mode_names.rs");

/// The file that declares that module,
/// and so decides which file the crate compiles as it.
const HANDLER_MODULE_SOURCE: &str = include_str!("../worktree_clone_mount_handler/mod.rs");

/// The crate root,
/// which declares the folder the pinned module lives in.
const CRATE_ROOT_SOURCE: &str = include_str!("../lib.rs");

/// The paths module,
/// read as text:
/// its `ZEDLET_INSTALL_PATH` is one of the two spellings this
/// guard ties to the pinned start name.
const INSTALL_PATHS_SOURCE: &str =
    include_str!("../worktree_clone_mount_handler/handler_install_paths.rs");

/// `root_mode_names.rs`'s code text,
/// whole and in order:
/// comments cut,
/// blanks trimmed.
/// A change to any line here is a change to what the module decides,
/// so the pin and the code
/// change in one commit.
const NAMES_CODE: &[&str] = &[
    "use std::ffi::{",
    "OsStr,",
    "OsString,",
    "};",
    "use std::path::Path;",
    "use crate::foundation::wrapper_names_old_and_new::COMMAND_WORD;",
    "pub const ZEDLET_START_NAME: &str = \"history_event-mount_git_worktree_clone_datasets\";",
    "pub const ROOT_GROUP_WORD: &str = \"zfs-root\";",
    "pub enum RootModeMisuse<'arguments> {",
    "NoModeWordAfterTheGroupWord,",
    "WordUnderTheZedletName(&'arguments OsStr),",
    "}",
    "pub enum RootModeStart<'arguments> {",
    "Arguments(&'arguments [OsString]),",
    "Refused(RootModeMisuse<'arguments>),",
    "Wrapper,",
    "}",
    "pub fn root_mode_start(arguments: &[OsString]) -> RootModeStart<'_> {",
    "let (start_name, handed): (Option<&OsStr>, &[OsString]) = split_the_name(arguments);",
    "if starts_with_the_group_words(handed) {",
    "let mode_arguments: &[OsString] = &handed[2..];",
    "if mode_arguments.is_empty() {",
    "return RootModeStart::Refused(RootModeMisuse::NoModeWordAfterTheGroupWord);",
    "}",
    "return RootModeStart::Arguments(mode_arguments);",
    "}",
    "if !names_the_zedlet(start_name) {",
    "return RootModeStart::Wrapper;",
    "}",
    "if handed.is_empty()",
    "|| (handed.len() == 1",
    "&& (handed[0] == OsStr::new(\"--help\") || handed[0] == OsStr::new(\"--version\")))",
    "{",
    "return RootModeStart::Arguments(handed);",
    "}",
    "return RootModeStart::Refused(RootModeMisuse::WordUnderTheZedletName(",
    "word_that_cannot_be_honoured(handed),",
    "));",
    "}",
    "fn starts_with_the_group_words(handed: &[OsString]) -> bool {",
    "return handed.len() >= 2",
    "&& handed[0] == OsStr::new(COMMAND_WORD.current)",
    "&& handed[1] == OsStr::new(ROOT_GROUP_WORD);",
    "}",
    "fn names_the_zedlet(start_name: Option<&OsStr>) -> bool {",
    "return start_name.and_then(last_component) == Some(ZEDLET_START_NAME);",
    "}",
    "fn last_component(start_name: &OsStr) -> Option<&str> {",
    "return Path::new(start_name).file_name().and_then(OsStr::to_str);",
    "}",
    "fn split_the_name(arguments: &[OsString]) -> (Option<&OsStr>, &[OsString]) {",
    "let Some((start_name, handed)) = arguments.split_first() else {",
    "return (None, &[]);",
    "};",
    "return (Some(start_name.as_os_str()), handed);",
    "}",
    "fn word_that_cannot_be_honoured(handed: &[OsString]) -> &OsStr {",
    "let first: &OsStr = handed[0].as_os_str();",
    "if handed.len() > 1",
    "&& (first == OsStr::new(COMMAND_WORD.current)",
    "|| first == OsStr::new(\"--help\")",
    "|| first == OsStr::new(\"--version\"))",
    "{",
    "return handed[1].as_os_str();",
    "}",
    "return first;",
    "}",
];

/// The module is the reviewed text,
/// whole.
#[test]
fn the_one_start_name_is_written_in_one_place() {
    assert_whole_line_comments_only("root_mode_names.rs", NAMES_SOURCE);
    let code: Vec<String> = pinned_code_lines(NAMES_SOURCE, "root_mode_names.rs")
        .into_iter()
        .filter(|line| {
            return !line.is_empty();
        })
        .collect();
    assert_eq!(
        code.len(),
        NAMES_CODE.len(),
        "root_mode_names.rs has {} code line(s), and the reviewed text has {}: {code:#?}",
        code.len(),
        NAMES_CODE.len()
    );
    for (at, (found, want)) in code.iter().zip(NAMES_CODE.iter()).enumerate() {
        assert_eq!(
            found.as_str(),
            *want,
            "root_mode_names.rs's code line {} is not the reviewed one",
            at + 1
        );
    }
}

/// What a declaration of one module in a source stands under:
/// whether the file declares it exactly once,
/// and the attribute lines immediately above that
/// declaration,
/// squeezed.
/// Why the attributes are read rather than the whole file:
///  a `#[path]` beside a declaration makes a different file the compiled module,
/// so the pin above
///  would keep passing the old one;
///  and the attribute run,
/// not the file,
/// is the thing that decides which file a `mod` names,
///  which is why `lib.rs`'s own `#[path]` on another module is allowed to stand.
fn declaration_run(
    source: &str,
    declaration: &str,
) -> (usize, String) {
    let lines: Vec<&str> = source.lines().collect();
    let declared: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_at, line)| {
            return line.trim() == declaration;
        })
        .map(|(at, _line)| {
            return at;
        })
        .collect();
    let Some(at) = declared.first().copied() else {
        return (0, String::new());
    };
    let mut run: Vec<String> = Vec::new();
    let mut cursor: Option<usize> = at.checked_sub(1);
    while let Some(line_at) = cursor {
        let trimmed: &str = lines[line_at].trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            cursor = line_at.checked_sub(1);
            continue;
        }
        if !trimmed.starts_with('#') {
            break;
        }
        run.push(String::from(trimmed));
        cursor = line_at.checked_sub(1);
    }
    run.reverse();
    let squeezed_run: String = squeezed(run.join("\n").as_str());
    return (declared.len(), squeezed_run);
}

/// The pinned file is the file the crate compiles as that module,
/// because a `#[path]`,
/// a `#[cfg_attr]` or a second declaration would make a different file the
/// module while the pin above kept passing the old one.
#[test]
fn the_pinned_module_is_the_module_the_crate_compiles() {
    let (declarations, run): (usize, String) =
        declaration_run(HANDLER_MODULE_SOURCE, "pub mod root_mode_names;");
    assert_eq!(
        declarations, 1,
        "worktree_clone_mount_handler/mod.rs must declare `pub mod root_mode_names;` exactly once, and it declares it {declarations} time(s)"
    );
    assert_eq!(
        run, "",
        "worktree_clone_mount_handler/mod.rs's declaration of root_mode_names must stand under no attribute, and it stands under {run:?}"
    );
}

/// The folder itself is declared once,
/// under the one attribute that makes it a Linux build's.
#[test]
fn the_handler_folder_is_declared_once_and_under_its_one_attribute() {
    let (declarations, run): (usize, String) =
        declaration_run(CRATE_ROOT_SOURCE, "pub mod worktree_clone_mount_handler;");
    assert_eq!(
        declarations, 1,
        "lib.rs must declare `pub mod worktree_clone_mount_handler;` exactly once, and it declares it {declarations} time(s)"
    );
    assert_eq!(
        run,
        squeezed("#[cfg(target_os = \"linux\")]"),
        "lib.rs's declaration of the handler folder must stand under `#[cfg(target_os = \"linux\")]` alone, and it stands under {run:?}"
    );
}

/// The words' first word is the program's own name,
/// taken from the one constant the crate keeps it in.
#[test]
fn the_words_first_word_is_the_programs_own_name() {
    assert_eq!(
        COMMAND_WORD.current, "monochromatic-git-policy-cli",
        "the words are `monochromatic-git-policy-cli <the group word> <mode>`, and root_mode_names.rs reads this constant for the first of them"
    );
}

/// The installed ZED path's file name is the one start name the module compares,
/// so an install and the dispatch cannot drift apart.
/// The path is read as text,
/// not imported,
/// because the folder it lives in is a Linux build's and
/// this guard runs on every developer's machine.
#[test]
fn the_installed_zedlet_path_carries_the_one_start_name() {
    // rustfmt puts a value too long for the declaration's line on the next one,
    //  so the value is the first quoted text after the declaration, on whichever line it sits.
    let installed: &str = INSTALL_PATHS_SOURCE
        .split_once("pub const ZEDLET_INSTALL_PATH: &str =")
        .and_then(|(_, rest)| {
            return rest.trim_start().strip_prefix('"');
        })
        .and_then(|rest| {
            return rest.split('"').next();
        })
        .expect("ZEDLET_INSTALL_PATH's text in handler_install_paths.rs");
    let file_name: Option<&str> = Path::new(installed).file_name().and_then(|name| {
        return name.to_str();
    });
    assert_eq!(
        file_name,
        Some("history_event-mount_git_worktree_clone_datasets"),
        "ZEDLET_INSTALL_PATH's file name must be ZEDLET_START_NAME, which root_mode_names.rs compares a start's last component against"
    );
}
