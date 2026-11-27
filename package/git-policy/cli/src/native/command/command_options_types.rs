//! What:
//!  What one option region is read into:
//!  a row of Git's own option table,
//!  the parse switches,
//!  every accepted use and its value,
//!  and the refusal kinds.
//! Why:
//!  The option tables and the walk over the arguments both name these shapes,
//!  and while one file held them that walk named the tables
//!  and the tables named that file back,
//!  a loop between files.
//!  This file uses nothing else of the folder,
//!  so a table and the walk may each use it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // command/optionsTypes.ts: the shapes the argument walk reads into.
//! ```

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
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
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
