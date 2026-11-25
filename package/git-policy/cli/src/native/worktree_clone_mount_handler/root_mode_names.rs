//! What:
//!  The one module that decides which start is a root-mode start:
//!  the installed ZED history event's own file name,
//!  the words `monochromatic-git-policy-cli zfs-root`,
//!  and the mode word after them.
//! Why:
//!  ZED starts a zedlet by its file name and hands it no argument at all,
//!  so only a name can reach the ZED hook's mode,
//!  while a unit's `ExecStart`, an adoption and a person spell the words.
//!  Both exceptions to the ban on reading a start name live here and nowhere
//!  else: the name is compared as the whole last path component, against one
//!  constant, and the argument vector is read by `main` alone and handed on
//!  (`doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`;
//!  `doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
//!  "Choice 1: what selects a root mode").
//!  The words are read before the name, so a unit's line works at the installed
//!  path, and both are read before the service's words, so
//!  `service run` under the zedlet's name is refused rather than served.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const start = rootModeStart(process.argv); // { kind: 'arguments' } | { kind: 'refused' } | { kind: 'wrapper' }
//! ```

use std::ffi::{OsStr, OsString};
use std::path::Path;

use crate::foundation::wrapper_names_old_and_new::COMMAND_WORD;

/// The installed ZED history event's own file name: the one start name that
/// selects a root mode, compared as the whole last path component.
pub const ZEDLET_START_NAME: &str = "history_event-mount_git_worktree_clone_datasets";

/// The namespace word the root modes sit under, beside `service`.
pub const ROOT_GROUP_WORD: &str = "zfs-root";

/// What a start under the root modes' own spellings asked for and cannot have.
pub enum RootModeMisuse<'arguments> {
    /// The root-mode words with no mode word after them.
    NoModeWordAfterTheGroupWord,
    /// A word under the installed name that is neither ZED's none nor `--help` nor `--version`.
    WordUnderTheZedletName(&'arguments OsStr),
}

/// What one start is.
pub enum RootModeStart<'arguments> {
    /// A root-mode start, with the words the modes' grammar reads.
    Arguments(&'arguments [OsString]),
    /// A start refused before the grammar, so no mode runs and Git is never reached.
    Refused(RootModeMisuse<'arguments>),
    /// The wrapper, unchanged.
    Wrapper,
}

/// What:
///  Decide what one start is, from its whole argument vector.
/// Why:
///   Nothing here reads an element of a mode's own words beyond the two the
///  group word owns,
///  and every answer is one of three, so a start that is not a root mode
///  reaches the wrapper exactly as it did before.
///  `Path::file_name` answers the last component of both a bare name, which is
///  what ZED's `execle` passes, and a path, which is what a unit writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rootModeStart(argv) { const [name, ...handed] = argv; if (isWords(handed)) { return words(argv); } if (lastComponent(name) !== ZEDLET) { return { kind: 'wrapper' }; } return zedletStart(handed); }
/// ```
pub fn root_mode_start(arguments: &[OsString]) -> RootModeStart<'_> {
    let (start_name, handed): (Option<&OsStr>, &[OsString]) = split_the_name(arguments);
    if starts_with_the_group_words(handed) {
        let mode_arguments: &[OsString] = &handed[2..];
        if mode_arguments.is_empty() {
            return RootModeStart::Refused(RootModeMisuse::NoModeWordAfterTheGroupWord);
        }
        return RootModeStart::Arguments(mode_arguments);
    }
    if !names_the_zedlet(start_name) {
        return RootModeStart::Wrapper;
    }
    if handed.is_empty() || (handed.len() == 1 && (handed[0] == OsStr::new("--help") || handed[0] == OsStr::new("--version"))) {
        return RootModeStart::Arguments(handed);
    }
    return RootModeStart::Refused(RootModeMisuse::WordUnderTheZedletName(word_that_cannot_be_honoured(handed)));
}

/// What:
///  Whether a start's first words are the program's own word and the group word.
/// Why:
///   The words are what selects a mode on any file,
///  so a unit's line works at the installed name, at a stray copy and at the
///  wrapper's own path,
///  and the program's word is read from the constant the crate keeps it in
///  rather than written out here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isWords = (handed) => handed.length >= 2 && handed[0] === COMMAND_WORD.current && handed[1] === 'zfs-root';
/// ```
fn starts_with_the_group_words(handed: &[OsString]) -> bool {
    return handed.len() >= 2 && handed[0] == OsStr::new(COMMAND_WORD.current) && handed[1] == OsStr::new(ROOT_GROUP_WORD);
}

/// What:
///  Whether a start name is the installed ZED history event's.
/// Why:
///   A name this rule cannot read as text is not the zedlet's,
///  so a start whose name is not UTF-8 is the wrapper rather than a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isZedlet = (name) => lastComponent(name) === ZEDLET_START_NAME;
/// ```
fn names_the_zedlet(start_name: Option<&OsStr>) -> bool {
    return start_name.and_then(last_component) == Some(ZEDLET_START_NAME);
}

/// What:
///  The last component of a start name, as text.
/// Why:
///   ZED passes the bare file name and a unit writes the path as its
///  `ExecStart` spells it,
///  so one comparison has to read both:
///  a trailing slash, doubled slashes, `./` and `sub/../` all answer the same
///  last component.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lastComponent = (name) => path.basename(name) || undefined;
/// ```
fn last_component(start_name: &OsStr) -> Option<&str> {
    return Path::new(start_name).file_name().and_then(OsStr::to_str);
}

/// What:
///  One start's name and everything after it.
/// Why:
///   An empty vector is a start with no name at all, which the kernel can pass
///  and which is the wrapper:
///  the last component of nothing is none, which is not the zedlet's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const split = (argv) => argv.length === 0 ? [undefined, []] : [argv.at(0), argv.slice(1)];
/// ```
fn split_the_name(arguments: &[OsString]) -> (Option<&OsStr>, &[OsString]) {
    let Some((start_name, handed)) = arguments.split_first() else {
        return (None, &[]);
    };
    return (Some(start_name.as_os_str()), handed);
}

/// What:
///  The first word this start cannot honour.
/// Why:
///   `--help x` and `--version x` name `x`, because the first word alone is
///  accepted and the second is what the name does not allow,
///  so a person who typed the second word reads about that word and not about
///  the one the file does answer.
///  The program's own word is the same case:
///  the two group words are accepted only together,
///  so a misspelt group word or the wrapper's `service` after it is the word
///  the person got wrong,
///  while the word standing alone is the one named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wordThatCannotBeHonoured = (handed) => (handed.length > 1 && [COMMAND_WORD.current, '--help', '--version'].includes(handed[0])) ? handed[1] : handed[0];
/// ```
fn word_that_cannot_be_honoured(handed: &[OsString]) -> &OsStr {
    let first: &OsStr = handed[0].as_os_str();
    if handed.len() > 1 && (first == OsStr::new(COMMAND_WORD.current) || first == OsStr::new("--help") || first == OsStr::new("--version")) {
        return handed[1].as_os_str();
    }
    return first;
}
