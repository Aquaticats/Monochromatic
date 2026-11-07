//! What:
//! Decides from pnpm's arguments whether the manifest warning makes sense for this run:
//! only when pnpm is about to resolve
//! or change dependencies from `package.json`,
//! and never for a help or version request.
//! Why:
//! A warning about a literal version helps at the moment pnpm would act on it (`pnpm add`,
//! `pnpm install`) and is noise,
//! plus a directory walk,
//! before `pnpm run`,
//! `pnpm list` or `pnpm --version`.
//! The user's words:
//! "only when the warning would
//! make sense".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // verb.ts
//! export function warningMakesSense(args: string[]): boolean { ... }
//! ```

/// Imports the owned operating-system text type:
/// arguments may be invalid UTF-8.
use std::ffi::OsString;

/// What:
/// `pub const INSTALL_CLASS_VERBS: [&str; 23] = [...]` is the fixed list of pnpm commands,
/// with their aliases,
/// that resolve
/// or change dependencies from `package.json`.
/// A fixed-size array (sibling:
/// `Vec<&str>`) because the list never grows at
/// run time.
/// Why:
/// Read from `pnpm --help` of pnpm 12.16.1:
/// `add`,
/// `install` (`i`),
/// `install-test` (`it`),
/// `update` (`up`,
/// `upgrade`),
/// `remove` (`rm`,
/// `uninstall`,
/// `un`,
/// `uni`),
/// `dedupe`,
/// `import`,
/// `link` (`ln`),
/// `unlink` (`dislink`),
/// and `ci`
/// (`clean-install`,
/// `ic`,
/// `install-clean`).
/// `fetch`,
/// `prune`,
/// `deploy` and the read-only commands read the lockfile or
/// the installed tree,
/// not the specifiers,
/// so a warning there would not make sense.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const INSTALL_CLASS_VERBS = ['add', 'install', 'i', ...] as const;
/// ```
pub const INSTALL_CLASS_VERBS: [&str; 23] = [
    "add",
    "install",
    "i",
    "install-test",
    "it",
    "update",
    "up",
    "upgrade",
    "remove",
    "rm",
    "uninstall",
    "un",
    "uni",
    "dedupe",
    "import",
    "link",
    "ln",
    "unlink",
    "dislink",
    "ci",
    "clean-install",
    "ic",
    "install-clean",
];

/// The commands that run another command over every project:
/// `pnpm recursive install` is an install,
/// so the word after
/// one of these is the verb.
pub const RECURSIVE_VERBS: [&str; 3] = ["recursive", "multi", "m"];

/// What:
/// `pub const VALUE_OPTIONS: [&str; 20] = [...]` lists the pnpm options that take the next argument as their value.
/// Why:
/// The verb is the first word that is not an option,
/// so the value of `--dir x` or `--filter y` must not be mistaken for it.
/// Read from `pnpm --help` of pnpm 12.16.1;
/// an option written `--name=value` is one word and needs no entry.
/// A new pnpm
/// option that takes a value must be added here (the same rule `mpm-shadow` keeps for mpm).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const VALUE_OPTIONS = ['--dir', '-C', '--filter', '-F', ...] as const;
/// ```
pub const VALUE_OPTIONS: [&str; 20] = [
    "--env-file",
    "-C",
    "--dir",
    "--store-dir",
    "--state-dir",
    "--npmrc-auth-file",
    "--userconfig",
    "--workspace-packages",
    "--registry",
    "--https-proxy",
    "--http-proxy",
    "--no-proxy",
    "--reporter",
    "--loglevel",
    "-F",
    "--filter",
    "--filter-prod",
    "--test-pattern",
    "--changed-files-ignore-pattern",
    "--workspace-concurrency",
];

/// A word anywhere before `--` that makes pnpm print help or its version and stop,
/// so nothing is installed.
pub const HELP_OR_VERSION_WORDS: [&str; 4] = ["--help", "-h", "--version", "-v"];

/// What:
/// `fn words_before_double_dash(arguments: &[OsString]) -> Vec<String>` turns the arguments into text and stops at a literal `--`.
/// Why:
/// Everything after `--` belongs to a script or a tool,
/// never to pnpm's own grammar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wordsBeforeDoubleDash(args: string[]): string[] { const end = args.indexOf('--'); return end === -1 ? args : args.slice(0, end); }
/// ```
fn words_before_double_dash(arguments: &[OsString]) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    for argument in arguments {
        // `to_string_lossy().to_string()` copies the argument as owned text, replacing invalid UTF-8 with a placeholder.
        let word: String = argument.to_string_lossy().to_string();
        if word == "--" {
            break;
        }
        words.push(word);
    }
    return words;
}

/// What:
/// `pub fn find_verb(arguments: &[OsString]) -> Option<String>` returns the command word:
/// the first word that is not an
/// option or an option's value,
/// looking through one `recursive` word.
/// Why:
/// `pnpm --filter app add zod` and `pnpm -C ../x install` have the verb after options;
/// `None` means there is none
/// (`pnpm` alone,
/// or only options).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findVerb(args: string[]): string | undefined
/// ```
pub fn find_verb(arguments: &[OsString]) -> Option<String> {
    let words: Vec<String> = words_before_double_dash(arguments);
    let mut index: usize = 0;
    let mut looked_through_recursive: bool = false;
    while index < words.len() {
        let word: &String = &words[index];
        if word.starts_with('-') {
            // An option that takes a value also consumes the next word; any other option is one word.
            if VALUE_OPTIONS.contains(&word.as_str()) {
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if !looked_through_recursive && RECURSIVE_VERBS.contains(&word.as_str()) {
            looked_through_recursive = true;
            index += 1;
            continue;
        }
        return Some(word.clone());
    }
    return None;
}

/// What:
/// `pub fn warning_makes_sense(arguments: &[OsString]) -> bool` says whether the manifest warning belongs to this run.
/// Why:
/// True only for an install-class verb with no help or version word in front of `--`:
/// `pnpm install --help` and
/// `pnpm add -h` install nothing,
/// so a warning there would not make sense.
/// An unrecognised command line says no,
/// because a
/// missed advisory costs less than a walk and a message nobody asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function warningMakesSense(args: string[]): boolean {
///   if (wordsBeforeDoubleDash(args).some((w) => HELP_OR_VERSION_WORDS.includes(w))) return false;
///   const verb = findVerb(args);
///   return verb !== undefined && INSTALL_CLASS_VERBS.includes(verb);
/// }
/// ```
pub fn warning_makes_sense(arguments: &[OsString]) -> bool {
    for word in words_before_double_dash(arguments) {
        if HELP_OR_VERSION_WORDS.contains(&word.as_str()) {
            return false;
        }
    }
    let verb: String = match find_verb(arguments) {
        Some(found) => found,
        None => return false,
    };
    return INSTALL_CLASS_VERBS.contains(&verb.as_str());
}
