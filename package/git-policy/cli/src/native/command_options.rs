//! What:
//!  Read one Git command's arguments the way Git 2.56.0 `parse-options.c` reads them.
//! Why:
//!  A policy that guesses which token is an option,
//!  a value or a path can be bypassed by a
//!      spelling Git accepts.
//!  Every per-command module declares Git's own option table and
//!      this module walks the arguments exactly once with Git's rules.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseOptions({ args, table, mode, wrapperFlags }): ParsedOptions  (throws OptionError)
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  which is where every sibling file of this crate is declared.
/// Why:
///   Long and short spellings follow different Git functions,
///  so each lives in its own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { scanLongOption } from './command_options_long.ts';
/// import { scanShortCluster } from './command_options_short.ts';
/// ```
use super::command_options_long::scan_long_option;
use super::command_options_short::scan_short_cluster;
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   File names and messages may hold bytes that are not UTF-8;
///  `String` would reject them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
///  `enum Arity` lists how an option relates to a value.
///  An `enum` is a closed set of
///       named alternatives.
///  `#[derive(...)]` asks the compiler to generate copying (`Clone`,
///       `Copy`),
///  debug printing (`Debug`) and `==` (`Eq`,
///  `PartialEq`).
/// Why:
///   Value consumption decides whether the next token is a path or an option value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Arity = 'none' | 'required' | 'optional' | 'last-argument-default';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arity {
    /// `PARSE_OPT_NOARG`:
    ///  never takes a value;
    ///  `--name=value` is an error.
    None,
    /// Takes the attached value,
    ///  otherwise the next token (`get_arg`,
    ///  parse-options.c:47-62).
    Required,
    /// `PARSE_OPT_OPTARG`:
    ///  takes an attached value only,
    ///  never the next token (203-204).
    Optional,
    /// `PARSE_OPT_LASTARG_DEFAULT`:
    ///  like `Required`,
    ///  but valid without a value when last (54-55).
    LastArgDefault,
}

/// What:
///  `struct OptionSpec` is one row of a Git option table.
///       `u16` is an unsigned 16-bit integer (siblings:
///  `u8`,
///  `u32`,
///  `usize`).
///       `Option<u8>` is "a byte or nothing";
///  `&'static str` is text baked into the program.
/// Why:
///   `id` lets a command module ask about an option by a named constant instead of by
///       table position.
///  `u16` (not `usize`) keeps rows small;
///  no table has 65 thousand rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OptionSpec = { id: number; short?: string; long?: string; arity: Arity; negatable: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OptionSpec {
    /// Identifier chosen by the command module;
    ///  `UNREAD` when no fact reads the option.
    pub id: u16,
    /// Git `short_name`:
    ///  the single letter after one dash.
    pub short: Option<u8>,
    /// Git `long_name`:
    ///  the spelling after two dashes;
    ///  it may itself start with `no-`.
    pub long: Option<&'static str>,
    /// Value relationship.
    pub arity: Arity,
    /// Whether `--no-<long>` is accepted (false for `PARSE_OPT_NONEG`).
    pub negatable: bool,
}

/// Identifier of every row no command fact reads;
///  such rows only supply arity.
pub const UNREAD: u16 = 0;

/// What:
///  `const fn` is a function the compiler can run while building a `const` table.
/// Why:
///   One short call per row keeps each Git table readable beside the C source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const row = (id, short, long, arity, negatable): OptionSpec => ({ id, short, long, arity, negatable });
/// ```
pub const fn row(
    id: u16,
    short: Option<u8>,
    long: Option<&'static str>,
    arity: Arity,
    negatable: bool,
) -> OptionSpec {
    return OptionSpec {
        id,
        short,
        long,
        arity,
        negatable,
    };
}

/// What:
///  Per-call switches matching Git's `enum parse_opt_flags`.
///  `bool` is true or false.
/// Why:
///   `git stash` keeps unknown options and stops at the first non-option;
///  most commands
///       do neither.
///  `PARSE_OPT_KEEP_DASHDASH` needs no switch:
///  the `--` index is always reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ParseMode = { keepUnknown: boolean; stopAtNonOption: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseMode {
    /// `PARSE_OPT_KEEP_UNKNOWN_OPT`:
    ///  unknown options are kept,
    ///  abbreviations are disabled.
    pub keep_unknown: bool,
    /// `PARSE_OPT_STOP_AT_NON_OPTION`:
    ///  the first non-option ends option parsing.
    pub stop_at_non_option: bool,
}

/// Flags `0`,
///  the mode of `commit`,
///  `add`,
///  `push`,
///  `clean`,
///  `branch` and `status`.
pub const DEFAULT_MODE: ParseMode = ParseMode {
    keep_unknown: false,
    stop_at_non_option: false,
};

/// What:
///  Where an option's value sits.
///  `usize` is the index type of lists (siblings:
///  `u32`,
///       `u64`);
///  every standard list API indexes with `usize`.
/// Why:
///   A value is reported by position,
///  never copied,
///  so its bytes stay exactly as given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OptionValue = { token: number; offset: number } | { token: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptionValue {
    /// `-mvalue` or `--message=value`:
    ///  the value starts at byte `offset` of `token`.
    Attached { token: usize, offset: usize },
    /// `-m value`:
    ///  the whole following token is the value.
    Detached { token: usize },
}

/// What:
///  One accepted use of a declared option.
/// Why:
///   Git lets the last use win,
///  so order and negation are kept for the command module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Occurrence = { id: number; negated: boolean; value?: OptionValue; token: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Occurrence {
    /// The matched row's identifier.
    pub id: u16,
    /// True for a `--no-` form.
    pub negated: bool,
    /// Position of the value,
    ///  when one was taken.
    pub value: Option<OptionValue>,
    /// Index of the token that spelled the option.
    pub token: usize,
}

/// What:
///  One wrapper-only flag found where Git would have looked for an option.
/// Why:
///   Only such positions may be removed before forwarding;
///  values and paths stay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WrapperOccurrence = { flag: number; token: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WrapperOccurrence {
    /// Index into the caller's wrapper flag list.
    pub flag: usize,
    /// Index of the token that spelled the flag.
    pub token: usize,
}

/// What:
///  How option parsing ended;
///  each variant holds the index of the ending token.
/// Why:
///   Tokens after the boundary are never options,
///  and commands treat `--` specially.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Boundary = { kind: 'end' } | { kind: 'dash-dash' | 'end-of-options' | 'non-option'; token: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Boundary {
    /// Every token was read.
    End,
    /// `--` at this index (parse-options.c:1113-1118).
    DashDash(usize),
    /// `--end-of-options` at this index (1119-1125).
    EndOfOptions(usize),
    /// First non-option at this index,
    ///  under `stop_at_non_option` (1015-1016).
    NonOption(usize),
}

/// What:
///  Everything learned from one argument region.
///  `Vec<T>` is an owned,
///  growable list
///       (siblings:
///  `&[T]`,
///  a borrowed view,
///  and `[T; N]`,
///  a fixed-size array).
/// Why:
///   The result outlives the scan and its lengths are unknown beforehand,
///  so it owns lists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ParsedOptions = { occurrences: Occurrence[]; wrapper: WrapperOccurrence[];
///   unknown: number[]; leading: number[]; boundary: Boundary };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedOptions {
    /// Declared options in the order Git applies them.
    pub occurrences: Vec<Occurrence>,
    /// Wrapper-only flags found in option position.
    pub wrapper: Vec<WrapperOccurrence>,
    /// Tokens holding an undeclared option,
    ///  only under `keep_unknown`.
    pub unknown: Vec<usize>,
    /// Non-option tokens before the boundary.
    pub leading: Vec<usize>,
    /// How option parsing ended.
    pub boundary: Boundary,
}

/// What:
///  Why Git would refuse the region before running the command.
/// Why:
///   The caller reports or forwards such a command line;
///  it must never guess its facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OptionErrorKind = 'unknown-option' | 'ambiguous-option' | 'missing-value'
///   | 'unexpected-value' | 'single-dash-long-option' | 'help-requested';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptionErrorKind {
    /// Git prints `unknown option` or `unknown switch` and exits 129 (parse-options.c:1224-1233).
    UnknownOption,
    /// An abbreviation matches several long options (578-587).
    AmbiguousOption,
    /// A value-taking option ends the region (60).
    MissingValue,
    /// `--flag=value` or `--no-option=value` (138-143).
    UnexpectedValue,
    /// `-amend`:
    ///  letters that spell a long option;
    ///  Git asks about two dashes (622-640).
    SingleDashLongOption,
    /// `-h`,
    ///  `--help`,
    ///  `--help-all` or a lone completion helper:
    ///  Git prints and exits 0.
    HelpRequested,
}

/// What:
///  One refusal,
///  with the index of the token that caused it.
/// Why:
///   A diagnostic can name the exact argument without this module rendering text for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class OptionError extends Error { kind: OptionErrorKind; token: number }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OptionError {
    /// The refusal category.
    pub kind: OptionErrorKind,
    /// Index of the offending token within the region.
    pub token: usize,
}

/// What:
///  `impl std::fmt::Display for OptionError` supplies Rust's "print me" interface.
/// Why:
///   Callers can show the refusal with ordinary formatting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// toString(): string { return `${kind} at argument ${token}`; }
/// ```
impl std::fmt::Display for OptionError {
    /// `&self` borrows the error read-only;
    ///  `&mut` lends the formatter for writing.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `write!` is the standard formatting macro; `{:?}` prints the variant name.
        return write!(
            formatter,
            "Git 2.56.0 refuses argument {} of this command region: {:?}",
            self.token, self.kind
        );
    }
}

/// An empty `impl` marks the type as a standard error value for generic handling.
impl std::error::Error for OptionError {}

/// What:
///  Find a wrapper-only flag by exact byte spelling.
///  `&[u8]` borrows bytes;
///  `&[&[u8]]`
///       borrows a list of byte spellings.
///  The result is its list index or nothing.
/// Why:
///   Wrapper flags are never abbreviated and never carry `=value`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const flag = wrapperFlags.indexOf(token); // -1 becomes "nothing"
/// ```
fn wrapper_flag_index(token: &[u8], wrapper_flags: &[&[u8]]) -> Option<usize> {
    let mut index: usize = 0;
    while index < wrapper_flags.len() {
        if wrapper_flags[index] == token {
            // `Some(x)` is the "present" case of `Option`.
            return Some(index);
        }
        index += 1;
    }
    // `None` is the "absent" case of `Option`.
    return None;
}

/// What:
///  Tokenize one region in a single forward pass (`parse_options_step`,
///       parse-options.c:995-1169).
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   One faithful scan replaces per-command guesses about option arity.
/// Gotcha:
///  Indexes in the result are relative to `arguments`,
///  the region after the subcommand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseOptions({ args, table, mode, wrapperFlags }): ParsedOptions; // throws OptionError
/// ```
pub fn parse_options(
    arguments: &[OsString],
    table: &[OptionSpec],
    mode: ParseMode,
    wrapper_flags: &[&[u8]],
) -> Result<ParsedOptions, OptionError> {
    // `mut` allows the lists to grow; `Vec::new()` is an empty list.
    let mut parsed: ParsedOptions = ParsedOptions {
        occurrences: Vec::<Occurrence>::new(),
        wrapper: Vec::<WrapperOccurrence>::new(),
        unknown: Vec::<usize>::new(),
        leading: Vec::<usize>::new(),
        boundary: Boundary::End,
    };
    // True while the token being visited is the value of the option before it.
    let mut is_value: bool = false;
    // `.iter().enumerate()` visits every token once with its position, so the scan always
    // ends: an option only says whether the next token is its value.
    for (index, argument) in arguments.iter().enumerate() {
        if is_value {
            is_value = false;
            continue;
        }
        // `.as_encoded_bytes()` lends the raw bytes; nothing is decoded or copied.
        let token: &[u8] = argument.as_encoded_bytes();
        // `if let Some(flag) = ...` runs the block only when a value is present.
        if let Some(flag) = wrapper_flag_index(token, wrapper_flags) {
            parsed
                .wrapper
                .push(WrapperOccurrence { flag, token: index });
            continue;
        }
        // A token not starting with `-`, the empty token and a lone `-` are not options (1011).
        if token.first() != Some(&b'-') || token.len() == 1 {
            if mode.stop_at_non_option {
                parsed.boundary = Boundary::NonOption(index);
                break;
            }
            parsed.leading.push(index);
            continue;
        }
        // A lone `-h` asks for usage (1050); a lone completion request prints the table
        // (1057-1060). Git exits 0 for both without running the command.
        if arguments.len() == 1
            && (token == b"-h"
                || token == b"--git-completion-helper"
                || token == b"--git-completion-helper-all")
        {
            // `Err(x)` is the failure case of `Result`.
            return Err(OptionError {
                kind: OptionErrorKind::HelpRequested,
                token: index,
            });
        }
        if token[1] != b'-' {
            // `?` returns the failure to our caller, or unwraps the success value.
            is_value = scan_short_cluster(arguments, index, table, mode, &mut parsed)?;
            continue;
        }
        if token.len() == 2 {
            parsed.boundary = Boundary::DashDash(index);
            break;
        }
        // `&token[2..]` borrows the bytes after the two dashes.
        let name: &[u8] = &token[2..];
        if name == b"end-of-options" {
            parsed.boundary = Boundary::EndOfOptions(index);
            break;
        }
        if name == b"help" || name == b"help-all" {
            return Err(OptionError {
                kind: OptionErrorKind::HelpRequested,
                token: index,
            });
        }
        is_value = scan_long_option(arguments, index, table, mode, &mut parsed)?;
    }
    // `Ok(x)` is the success case of `Result`.
    return Ok(parsed);
}

/// Synthetic-table cases,
///  ported incumbent cases and typed refusals.
#[cfg(test)]
#[path = "command_options_tests.rs"]
mod tests;

/// Differential controls against `git rev-parse --parseopt` of the real Git 2.56.0.
#[cfg(test)]
#[path = "command_options_git_tests.rs"]
mod git_tests;
