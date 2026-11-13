//! What:
//!  The plan for one `git worktree add` the cheap-worktree ladder may serve:
//! where the new worktree goes and where in
//!  the argument list a rung inserts `--no-checkout`,
//! or nothing when the invocation uses a form the ladder does not
//!  know.
//! Why:
//!  The ladder runs real Git once with the person's arguments,
//! so it only needs the destination path and a promise that
//!  every other word means what the table says.
//! An option the table does not list,
//! a bundled short flag,
//! a global
//!  option that changes how Git reads configuration,
//! or a destination whose parent does not exist makes the whole
//!  invocation a plain Git run,
//! which is always correct.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = parseAddPlan(['-C', 'repo', 'worktree', 'add', '-b', 'topic', '../topic'], 1, cwd); // undefined: plain Git
//! ```

use crate::worktree_add_plan_path_questions::{is_slash, is_symbolic_link, names_parent_directory, nearest_existing_ancestor};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// What:
///  The facts of a `git worktree add` the ladder serves.
///  `usize` is an unsigned size or index;
/// `PathBuf` is an owned file-system path.
/// Why:
///   The destination decides the subvolume to create and the worktree to settle;
/// the insertion index decides where
///  `--no-checkout` goes in the arguments real Git receives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddPlan = { destination: string; insertAt: number; quiet: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddPlan {
    /// The destination as Git records it:
    /// the real path of its parent joined with its last component.
    pub destination: PathBuf,
    /// The index in the full argument list where options of `add` may be inserted:
    /// directly after the word `add`.
    pub insert_at: usize,
    /// Whether the person passed `--quiet`,
    /// which silences the line Git's checkout prints.
    pub quiet: bool,
}

/// The options of `add` that take no value and keep the checkout of a new worktree the ordinary one.
const FLAG_OPTIONS: &[&[u8]] = &[b"--detach", b"-d", b"--quiet", b"-q", b"--lock", b"--checkout"];

/// The options of `add` whose value is the next argument.
const VALUE_OPTIONS: &[&[u8]] = &[b"-b", b"--reason"];

/// The global options that cannot change how the ladder's own Git runs read the repository.
const HARMLESS_GLOBALS: &[&[u8]] = &[b"--no-pager", b"-P", b"--paginate", b"-p", b"--no-advice", b"--no-lazy-fetch"];

/// What:
///  The directory a command starts in:
/// the process directory moved by every `-C <dir>` of the global prefix,
/// or nothing
///  when the prefix holds an option the ladder does not serve.
/// Why:
///   Git changes directory at each `-C` in turn,
/// and a relative destination is relative to the result.
/// A `..`
///  component in a `-C` value is declined,
/// because following it lexically differs from Git's directory changes
///  through a symbolic link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startDirectory(prefix: string[], cwd: string): string | undefined;
/// ```
pub fn start_directory(global_prefix: &[OsString], current: &Path) -> Option<PathBuf> {
    let mut start: PathBuf = current.to_path_buf();
    let mut tokens = global_prefix.iter();
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if bytes == b"-C" {
            let target: &OsString = tokens.next()?;
            if names_parent_directory(Path::new(target)) {
                return None;
            }
            if !target.is_empty() {
                start = start.join(target);
            }
        } else if !HARMLESS_GLOBALS.contains(&bytes) {
            return None;
        }
    }
    return Some(start);
}

/// What:
///  Whether a global option word is a valued global in its attached form,
/// `<name>=<value>`.
/// Why:
///   Such a word carries its value,
/// so the word after it is not a value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// COPY_VALUED_GLOBALS.some((name) => word.startsWith(`${name}=`))
/// ```
fn is_valued_global_with_equals(word: &[u8]) -> bool {
    for name in COPY_VALUED_GLOBALS {
        if let Some(rest) = word.strip_prefix(*name)
            && rest.first() == Some(&b'=')
        {
            return true;
        }
    }
    return false;
}

/// What:
///  The option of the table whose one-letter spelling is `letter`,
/// if any.
/// Why:
///   Bundled short options are read letter by letter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ADD_OPTIONS.find((candidate) => candidate.short === letter)
/// ```
fn option_for_short_letter(letter: u8) -> Option<&'static AddOption> {
    for candidate in ADD_OPTIONS {
        if candidate.short == Some(letter) {
            return Some(candidate);
        }
    }
    return None;
}

/// What:
///  The real path of a destination as Git records it:
/// the canonical parent joined with the last component.
/// Why:
///   Git stores `strbuf_add_real_path` of the new worktree after creating its parent directories;
/// the ladder serves
///  only a parent that exists,
/// so no directory has to be guessed.
/// A last component of `.` or `..`,
/// or one that is already
///  a symbolic link,
/// is declined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordedDestination(start: string, operand: string): string | undefined;
/// ```
fn recorded_destination(start: &Path, operand: &OsStr) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let mut trimmed: &[u8] = operand.as_bytes();
    while let Some(rest) = trimmed.strip_suffix(b"/") {
        trimmed = rest;
    }
    let last: &[u8] = trimmed.rsplit(is_slash).next()?;
    if last.is_empty() || last == b"." || last == b".." {
        return None;
    }
    let joined: PathBuf = start.join(OsStr::from_bytes(trimmed));
    let parent: PathBuf = std::fs::canonicalize(joined.parent()?).ok()?;
    let recorded: PathBuf = parent.join(OsStr::from_bytes(last));
    // Git records the resolved path of a destination that is a link, so the directory Git fills and the path the ladder
    // would swap differ: decline, and let Git run alone.
    if is_symbolic_link(recorded.as_path()) {
        return None;
    }
    return Some(recorded);
}

/// What:
///  Read the arguments after `add`:
/// the operands and whether `--quiet` was given,
/// or nothing when any option is outside
///  the table.
/// Why:
///   `--` ends the options.
/// An operand count other than one or two is a usage error that Git words itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addOperands(region: string[]): { operands: string[]; quiet: boolean } | undefined;
/// ```
fn add_operands(region: &[OsString]) -> Option<(Vec<&OsString>, bool)> {
    let mut operands: Vec<&OsString> = Vec::new();
    let mut quiet: bool = false;
    let mut tokens = region.iter();
    let mut options_ended: bool = false;
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if options_ended || !bytes.starts_with(b"-") || bytes == b"-" {
            operands.push(token);
        } else if bytes == b"--" {
            options_ended = true;
        } else if VALUE_OPTIONS.contains(&bytes) {
            tokens.next()?;
        } else if bytes == b"-q" || bytes == b"--quiet" {
            quiet = true;
        } else if !bytes.starts_with(b"--reason=") && !FLAG_OPTIONS.contains(&bytes) {
            return None;
        }
    }
    return if (1..=2).contains(&operands.len()) { Some((operands, quiet)) } else { None };
}

/// What:
///  Plan one invocation:
/// `arguments` is the whole argument list,
/// `subcommand_index` the index of `worktree` in it,
/// and
///  `start` the directory the command starts in.
/// Why:
///   The word directly after `worktree` must be `add` itself,
/// because Git reads the subcommand there and never
///  abbreviates it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseAddPlan(args: string[], subcommandIndex: number, start: string): AddPlan | undefined;
/// ```
pub fn parse_add_plan(arguments: &[OsString], subcommand_index: usize, start: &Path) -> Option<AddPlan> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree" {
        return None;
    }
    let word_index: usize = subcommand_index + 1;
    if arguments.get(word_index)?.as_encoded_bytes() != b"add" {
        return None;
    }
    let (operands, quiet): (Vec<&OsString>, bool) = add_operands(&arguments[word_index + 1..])?;
    let destination: PathBuf = recorded_destination(start, operands[0])?;
    return Some(AddPlan { destination, insert_at: word_index + 1, quiet });
}

/// What:
///  Why a `git worktree add` the ladder does not serve runs as plain Git,
/// in the clause the notice reads.
/// Why:
///   An add the wrapper cannot serve is still an add a person waits for,
/// and the one thing the wrapper
///  knows about it is which fact put it on the plain route:
/// the folder the worktree goes in does not exist yet,
///  which is what the ladder's own reading of the destination refuses,
/// or a form of the command the wrapper does not read at all.
/// The clause is built here so the words and the reading that refuses share one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whyPlain = (destination) => parentMissing(destination) ? `because the folder it goes in, ${quoted}, does not exist yet` : 'because the form of the command is not one it reads';
/// ```
pub fn why_a_plain_add(destination: Option<&Path>) -> String {
    if let Some(parent) = destination.and_then(Path::parent)
        && matches!(std::fs::symlink_metadata(parent), Err(ref error) if error.kind() == std::io::ErrorKind::NotFound)
    {
        return format!("because the folder it goes in, {}, does not exist yet", crate::json_record::quote(parent.to_string_lossy().as_ref()));
    }
    return String::from("because the command's form is not one the wrapper reads");
}

/// What:
///  Whether the person asked for `--quiet` on a `git worktree add`,
///  whatever the ladder makes of the form.
/// Why:
///   The words after `add` are read once,
/// by [`add_operands`],
/// and the plan carries the answer only for a form the ladder
///  serves:
/// an add whose destination the ladder refuses still has a person who may have asked for no
///  lines,
/// so the answer is taken from the same reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quiet = (args, subcommandIndex) => addOperands(args.slice(subcommandIndex + 2))?.[1] ?? false;
/// ```
pub fn add_is_quiet(arguments: &[OsString], subcommand_index: usize) -> bool {
    let word_is = |index: usize, wanted: &[u8]| { return arguments.get(index).is_some_and(|word| { return word.as_encoded_bytes() == wanted; }); };
    if !word_is(subcommand_index, b"worktree") {
        return false;
    }
    let word_index: usize = subcommand_index + 1;
    if !word_is(word_index, b"add") {
        return false;
    }
    return add_operands(&arguments[word_index + 1..]).is_some_and(|(_operands, quiet)| { return quiet; });
}


/// How an option of `git worktree add` takes its value,
/// as `parse-options.c` of Git 2.56.0 does (`do_get_value`,
/// line 136).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Takes {
    /// No value:
    /// `--name=x` is Git's error,
    /// and so is a value on the negated form.
    Nothing,
    /// A value in the rest of the token,
    /// after `=`,
    /// or in the next token;
    /// the negated form takes none.
    Required,
}

/// One option of `git worktree add`:
/// its short letter,
/// its long name,
/// and how it takes a value.
struct AddOption {
    /// The letter of the short form,
    /// when there is one.
    short: Option<u8>,
    /// The long name,
    /// when there is one;
    /// every long option also has a `--no-` form.
    long: Option<&'static str>,
    /// How the option takes its value.
    takes: Takes,
}

/// The options of `git worktree add` in the order of `add` in `builtin/worktree.c` (Git 2.56.0,
/// lines 821 to 862),
/// the one
/// table every reading of the command line uses.
/// `git worktree add -h` lists these,
/// and a control compares the two,
/// so the table cannot drift from Git unnoticed.
const ADD_OPTIONS: &[AddOption] = &[
    AddOption { short: Some(b'f'), long: Some("force"), takes: Takes::Nothing },
    AddOption { short: Some(b'b'), long: None, takes: Takes::Required },
    AddOption { short: Some(b'B'), long: None, takes: Takes::Required },
    AddOption { short: None, long: Some("orphan"), takes: Takes::Nothing },
    AddOption { short: Some(b'd'), long: Some("detach"), takes: Takes::Nothing },
    AddOption { short: None, long: Some("checkout"), takes: Takes::Nothing },
    AddOption { short: None, long: Some("lock"), takes: Takes::Nothing },
    AddOption { short: None, long: Some("reason"), takes: Takes::Required },
    AddOption { short: Some(b'q'), long: Some("quiet"), takes: Takes::Nothing },
    AddOption { short: None, long: Some("track"), takes: Takes::Nothing },
    AddOption { short: None, long: Some("guess-remote"), takes: Takes::Nothing },
    AddOption { short: None, long: Some("relative-paths"), takes: Takes::Nothing },
];

/// The global options that take no value and change nothing about where a relative destination lies.
const COPY_NEUTRAL_GLOBALS: &[&[u8]] = &[
    b"--no-pager", b"-P", b"--paginate", b"-p", b"--no-replace-objects", b"--no-lazy-fetch", b"--no-optional-locks",
    b"--no-advice", b"--literal-pathspecs", b"--no-literal-pathspecs", b"--glob-pathspecs", b"--noglob-pathspecs",
    b"--icase-pathspecs", b"--bare",
];

/// The global options that take a value in the next token or after `=`,
/// and change nothing about where a relative
/// destination lies (`git.c` of Git 2.56.0,
/// `handle_options`:
/// `--git-dir`,
/// `--work-tree`,
/// `--namespace`,
/// `--config-env`,
/// `--attr-source`).
/// A relative destination lies against the working directory whatever they say:
/// Git changes into the work
/// tree and prefixes the operand when the working directory is inside it,
/// and leaves it alone when it is outside
/// (`setup.c`,
/// lines 1222 to 1231).
const COPY_VALUED_GLOBALS: &[&[u8]] = &[b"--git-dir", b"--work-tree", b"--namespace", b"--config-env", b"--attr-source"];

/// The global options that take a value in the next token only.
const COPY_SEPARATE_VALUE_GLOBALS: &[&[u8]] = &[b"-c", b"--shallow-file"];

/// What:
///  The directory a command's relative destination is resolved against,
/// after the global options.
/// Why:
///   Only `-C` moves it;
/// the others change nothing about it.
/// An option this reading does not know leaves the destination
///  unknown,
/// and the copy then says it did not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyStartDirectory(globalPrefix: string[], current: string): string | undefined;
/// ```
pub fn copy_start_directory(global_prefix: &[OsString], current: &Path) -> Option<PathBuf> {
    let mut start: PathBuf = current.to_path_buf();
    let mut tokens = global_prefix.iter();
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if bytes == b"-C" {
            let target: &OsString = tokens.next()?;
            if !target.is_empty() {
                start = start.join(target);
            }
        } else if COPY_SEPARATE_VALUE_GLOBALS.contains(&bytes) || COPY_VALUED_GLOBALS.contains(&bytes) {
            tokens.next()?;
        } else if is_valued_global_with_equals(bytes) || bytes.starts_with(b"--exec-path=")
        {
            continue;
        } else if !COPY_NEUTRAL_GLOBALS.contains(&bytes) {
            return None;
        }
    }
    return Some(start);
}

/// The option a long argument names and how,
/// resolved exactly as `parse_long_opt` of Git 2.56.0 does (`parse-options.c`,
/// lines
/// 520 to 593).
struct LongOption<'a> {
    /// The option the text names,
    /// by its full name or by a unique prefix.
    option: &'static AddOption,
    /// Whether the `--no-` form was written.
    unset: bool,
    /// The text after `=`,
    /// when there is one.
    attached: Option<&'a str>,
}

/// What:
///  Resolves the text after `--` to one option,
/// or to nothing where Git has none:
/// an exact name wins at once,
/// whatever
///  prefixes were seen before;
/// otherwise one unique prefix is the option;
/// two prefixes are Git's "ambiguous option" error.
/// Why:
///   The ignored-state copy needs the operands of an add,
/// and an abbreviated option decides which tokens are operands.
///  `--no-` negates and `--no-no-` is accepted by no option here.
/// Git also reads an argument that is a prefix of `no-` as an
///  abbreviation of every option's negation;
/// here that changes nothing,
/// because no option begins with `n`,
/// and the text
///  `no-` itself already fits every option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveLongOption(arg: string): { option; unset; attached? } | undefined;
/// ```
fn resolve_long(arg: &str) -> Option<LongOption<'_>> {
    return resolve_in(ADD_OPTIONS, arg);
}

/// What:
///  `resolve_long` over a given table of options.
/// Why:
///   The real table has no option whose name is a prefix of another's,
/// so the rule that an exact name wins can only be
///  shown with a table that has one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveIn(options: AddOption[], arg: string): { option; unset; attached? } | undefined;
/// ```
fn resolve_in<'a>(table: &'static [AddOption], arg: &'a str) -> Option<LongOption<'a>> {
    let arg_end: usize = arg.find('=').unwrap_or(arg.len());
    // `--no-x` is the negation of `x`; `--no-no-x` is accepted by no option whose name has no `no-` (line 535 to 543).
    let (arg_start, unset, double_negation): (&str, bool, bool) = match arg.strip_prefix("no-") {
        Some(rest) => match rest.strip_prefix("no-") {
            Some(inner) => (inner, false, true),
            None => (rest, true, false),
        },
        None => (arg, false, false),
    };
    if double_negation {
        return None;
    }
    // Git counts the name's length against the original text, so a text cut inside `no-` has no length and matches nothing.
    let length: usize = arg_end.checked_sub(arg.len() - arg_start.len())?;
    let mut abbreviated: Option<(&'static AddOption, bool)> = None;
    let mut ambiguous: bool = false;
    for option in table {
        let Some(name) = option.long else { continue };
        if let Some(rest) = arg_start.strip_prefix(name) {
            // An exact name returns at once, even after prefixes were seen.
            if let Some(value) = rest.strip_prefix('=') {
                return Some(LongOption { option, unset, attached: Some(value) });
            }
            if rest.is_empty() {
                return Some(LongOption { option, unset, attached: None });
            }
            continue;
        }
        if name.as_bytes().get(..length) == Some(&arg_start.as_bytes()[..length]) {
            ambiguous |= abbreviated.is_some();
            abbreviated = Some((option, unset));
        }
    }
    if ambiguous {
        return None;
    }
    let (matched, matched_unset) = abbreviated?;
    let attached: Option<&str> = if arg_end < arg.len() { Some(&arg[arg_end + 1..]) } else { None };
    return Some(LongOption { option: matched, unset: matched_unset, attached });
}

/// What:
///  Whether Git accepts the value an option was given,
/// and takes the next token when the option wants it there.
/// Why:
///   `do_get_value` (`parse-options.c`,
/// line 136) refuses a value on a negated form and on a flag,
/// and an option that needs a
///  value and has none;
/// refusing here keeps Git's error the only word on a form Git rejects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeValue(option, unset, attached, tokens): boolean;
/// ```
fn take_value<'a>(option: &AddOption, unset: bool, attached: Option<&str>, tokens: &mut impl Iterator<Item = &'a OsString>) -> Option<()> {
    if unset && attached.is_some() {
        return None;
    }
    match option.takes {
        Takes::Nothing => return if attached.is_some() { None } else { Some(()) },
        Takes::Required => {
            if !unset && attached.is_none() {
                tokens.next()?;
            }
            return Some(());
        }
    }
}

/// What:
///  The operands of a `git worktree add`:
/// the path and the optional commit-ish,
/// with every option skipped the way Git's
///  `parse_options` skips it.
/// Why:
///   The destination of a copy must be known from the command line.
/// Long options are resolved by `resolve_long`,
/// short
///  options bundle (`-fq`,
/// `-bfeature`,
/// `-b feature`),
/// `--` and `--end-of-options` end the options,
/// and everything Git
///  rejects makes the reading refuse,
/// so the copy never guesses an operand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyAddOperands(region: string[]): string[] | undefined;
/// ```
fn copy_add_operands(region: &[OsString]) -> Option<Vec<&OsString>> {
    let mut operands: Vec<&OsString> = Vec::new();
    let mut tokens = region.iter();
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if !bytes.starts_with(b"-") || bytes == b"-" {
            operands.push(token);
        } else if bytes == b"--" || bytes == b"--end-of-options" {
            operands.extend(tokens.by_ref());
        } else if let Some(long) = bytes.strip_prefix(b"--") {
            let resolved: LongOption<'_> = resolve_long(std::str::from_utf8(long).ok()?)?;
            take_value(resolved.option, resolved.unset, resolved.attached, &mut tokens)?;
        } else {
            // Bundled short options: flags stand alone, and `b` or `B` takes the rest of the token or the next token.
            let letters: &[u8] = &bytes[1..];
            for (position, letter) in letters.iter().enumerate() {
                let option: &AddOption = option_for_short_letter(*letter)?;
                if option.takes == Takes::Required {
                    if position + 1 == letters.len() {
                        tokens.next()?;
                    }
                    break;
                }
            }
        }
    }
    return if (1..=2).contains(&operands.len()) { Some(operands) } else { None };
}

/// What:
///  The destination of a path operand in the form Git records it,
/// whether or not its parent directories exist yet.
/// Why:
///   Git creates the leading directories of the destination,
/// so `.claude/worktrees/topic` names a destination before
///  `.claude/worktrees` does;
/// the nearest existing ancestor is resolved to its real path and the rest is appended as written.
///  A destination that is a link is declined,
/// as in the ladder's reading,
/// and a `..` in the part that does not exist yet is
///  declined because it cannot be resolved without the directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function creatingDestination(start: string, operand: string): string | undefined;
/// ```
fn creating_destination(start: &Path, operand: &OsStr) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let mut trimmed: &[u8] = operand.as_bytes();
    while let Some(rest) = trimmed.strip_suffix(b"/") {
        trimmed = rest;
    }
    let last: &[u8] = trimmed.rsplit(is_slash).next()?;
    if last.is_empty() || last == b"." || last == b".." {
        return None;
    }
    let joined: PathBuf = start.join(OsStr::from_bytes(trimmed));
    if is_symbolic_link(joined.as_path()) {
        return None;
    }
    let existing: &Path = nearest_existing_ancestor(joined.as_path())?;
    let mut resolved: PathBuf = std::fs::canonicalize(existing).ok()?;
    for part in joined.strip_prefix(existing).ok()?.components() {
        match part {
            std::path::Component::Normal(name) => resolved.push(name),
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    return Some(resolved);
}

/// What:
///  The destination a `git worktree add` names,
/// in the form Git records it,
/// for every form Git's option parser accepts.
/// Why:
///   The ignored-state copy goes to the worktree this command created,
/// found from the command line instead of by comparing
///  the repository's registrations before and after,
/// which another add running at the same time would disturb.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCopyDestination(args: string[], subcommandIndex: number, start: string): string | undefined;
/// ```
pub fn parse_copy_destination(arguments: &[OsString], subcommand_index: usize, start: &Path) -> Option<PathBuf> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree" {
        return None;
    }
    let word_index: usize = subcommand_index + 1;
    if arguments.get(word_index)?.as_encoded_bytes() != b"add" {
        return None;
    }
    let operands: Vec<&OsString> = copy_add_operands(&arguments[word_index + 1..])?;
    return creating_destination(start, operands[0]);
}

/// Plan controls,
/// with a real Git control,
/// stay out of the release executable.
#[cfg(test)]
#[path = "worktree_add_plan_tests.rs"]
mod tests;
