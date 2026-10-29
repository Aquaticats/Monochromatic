//! Clap-backed command-line declaration for the `forbidden-strings` binary.
//!
//! This module owns flags,
//!  positionals,
//!  help text,
//!  version text,
//!  and validation
//! for argv-shaped input.
//!  Environment-variable fallback stays in `lib.rs` because
//! it is not itself command-line syntax.

/// Imports clap's parser trait and derive macro for this CLI declaration.
// What:     `use clap::Parser;` brings TWO clap tools into scope under one name:
//           the `Parser` trait, which supplies `Cli::try_parse_from(...)`, and
//           the `#[derive(Parser)]` macro, which generates the argv scanner for
//           the struct below. `::` is Rust's namespace separator, like `.` on an
//           imported object in TypeScript.
// Why:      Delegate option parsing, help, version, and validation to clap
//           instead of hand-scanning raw strings.
// Gotcha:   The derive macro writes Rust code during compilation. This file keeps
//           that code generation at the CLI boundary so the scanner logic stays
//           ordinary functions.
//
// In TS you'd write (pseudocode):
// ```ts
// import { parseArgs } from "some-cli-parser";
// ```
use clap::{Parser, Subcommand};

/// Custom clap help layout with the historical uppercase usage heading.
// What:     `const HELP_TEMPLATE: &str = "..."` declares a borrowed static string.
//           `&str` is a read-only view into UTF-8 bytes baked into the binary;
//           sibling type `String` would allocate owned bytes at runtime. Clap
//           replaces placeholders like `{usage}` and `{all-args}` when rendering
//           `--help`.
// Why:      Keep the visible `USAGE:` heading already documented by tests while
//           letting clap generate the actual flag and positional sections.
//
// In TS you'd write (pseudocode):
// ```ts
// const HELP_TEMPLATE = `${name} ${version}\n${about}\n...`;
// ```
const HELP_TEMPLATE: &str = "\
{name} {version}\n\
{about}\n\
\n\
USAGE:\n\
    {usage}\n\
\n\
{all-args}{after-help}\
";

/// Extra help text that describes runtime fallback and rule semantics.
// What:     `const AFTER_HELP: &str = "..."` is another static string slice. The
//           sibling `String` is unnecessary because the text never changes.
// Why:      Clap owns syntax help, while this block carries domain notes that are
//           not attached to one flag: environment fallback, exit codes, rule
//           grammar, the dialect's flag policy, and output format.
//
// In TS you'd write (pseudocode):
// ```ts
// const AFTER_HELP = `ENV:\n  FORBIDDEN_STRINGS_RULES ...`;
// ```
const AFTER_HELP: &str = "\
ENV:\n\
    FORBIDDEN_STRINGS_RULES    Default rules path; --rules wins if both are set.\n\
                               If unset, falls back to ./forbidden-strings.local.txt\n\
    FORBIDDEN_STRINGS_CACHE_DIR\n\
                               Absolute per-user cache-root override.\n\
\n\
RUNTIME CACHE:\n\
    Runtime rules use read-write caching by default. Cache identity is exact\n\
    rules content plus scanner version and platform. Missing or rejected cache\n\
    data emits redacted JSON, compiles authoritative text, and attempts atomic\n\
    repair. A failed repair never invalidates the in-memory scan.\n\
\n\
BUILT-IN BASELINE:\n\
    --builtin-rules appends the embedded betterleaks-ported baseline after\n\
    the resolved rules file (user rule numbering is unchanged). When the\n\
    implicit default rules file is absent, the baseline alone is used; an\n\
    explicitly named missing file (--rules or env) still errors. Without\n\
    the flag, the baseline is never read.\n\
\n\
EXIT CODES:\n\
    0    No violations.\n\
    1    One or more violations (printed to stderr, redacted).\n\
    2    Usage error or rule-file error.\n\
\n\
EXAMPLES:\n\
    # Scan a few files\n\
    forbidden-strings --rules ./rules.txt src/main.ts README.md\n\
\n\
    # Scan the whole working tree\n\
    FORBIDDEN_STRINGS_RULES=./rules.txt forbidden-strings --all\n\
\n\
    # Eagerly compile runtime rules without scanning\n\
    forbidden-strings compile-rules --rules ./rules.txt\n\
\n\
RULE FORMAT (autodetected per file, never mixed):\n\
    Tail format            -> '==> name <==' headers open one-rule sections;\n\
                              the name ([a-z0-9], then [a-z0-9.-]) is the\n\
                              rule's identity in findings. One significant\n\
                              body line classifies as below; several are one\n\
                              verbatim always-verbose pattern.\n\
    Legacy format          -> one rule per line, unnamed:\n\
    Bare line              -> case-sensitive literal; under 8 bytes it is\n\
                              word-boundary gated at word-byte ends\n\
    /PATTERN/FLAGS         -> regex in the forbidden-regex dialect\n\
    # ...                  -> comment\n\
    Empty line             -> skipped\n\
\n\
DIALECT:\n\
    Supported: literals, classes [a-z] and \\d \\w \\s, '.', (?:a|b),\n\
    bounded repetition a{3,6}, anchors ^ $ \\b, and set algebra A & B\n\
    and ~(A). Flags 'm' and 'x' are accepted no-ops (multiline and\n\
    verbose are always on); any other flag letter is a hard load error.\n\
    Rejected at compile time (fail-closed): '*', '+', unbounded {n,},\n\
    capturing '(', lookaround and inline flags, backreferences, and any\n\
    pattern matching the empty string.\n\
\n\
OUTPUT:\n\
    PATH:LINE rule=<token>         Content finding (columnless).\n\
    PATH:name:SEGMENT rule=<token>  Pathname finding (columnless); SEGMENT\n\
    counts directory names and filename from 1. Each name is matched\n\
    separately, always on, with no flag to disable pathname scanning.\n\
    Every offending pathname segment is replaced with [REDACTED] in all\n\
    findings; matched substrings are NEVER printed. The token is the\n\
    rule's section name (baseline uses betterleaks id); unnamed legacy\n\
    rules use the 0-based index. --name-path PATH pairs a logical name\n\
    with each positional content file, in order; these findings carry\n\
    input=N to identify the 0-based operand without exposing its name.\n\
\n\
See README.md for the full dialect, set-algebra examples, and CI integration.\n\
";

/// Distinct non-scan operations selected by first command word.
#[derive(Subcommand, Debug, PartialEq)]
pub enum CliCommand {
    /// Compiles one authoritative runtime rules file into derived user cache.
    #[command(name = "compile-rules")]
    CompileRules {
        /// Explicit authoritative rules path;
        ///  environment fallback is deliberately absent.
        #[arg(
            long = "rules",
            value_name = "PATH",
            allow_hyphen_values = true,
            help = "Path to authoritative runtime rules file"
        )]
        rules_path: String,
    },
}

/// Returns explicit compile-rules source path when selected.
impl CliCommand {
    /// Borrows rules path carried by compile operation.
    pub fn rules_path(&self) -> &str {
        let Self::CompileRules { rules_path } = self;
        return rules_path
    }
}

/// Parsed command-line options for `forbidden-strings`.
// What:     `#[derive(Parser, Debug, PartialEq)]` asks Rust to generate three
//           implementations for `Cli`: clap's parser, debug formatting, and
//           equality. `#[command(...)]` configures clap's program metadata for
//           `--help` and `--version`. `pub struct Cli { ... }` is an exported
//           record type with owned fields. Siblings a TS reader might expect:
//           a plain object type with a handwritten parser, or a builder API.
// Why:      Declaring the CLI as data lets clap validate arguments and generate
//           user-facing help while the scanner run loop receives typed options.
//
// In TS you'd write (pseudocode):
// ```ts
// // @cli({ name: "forbidden-strings", version, about: "..." })
// export type Cli = { rulesPath?: string; all: boolean; files: string[] };
// ```
#[derive(Parser, Debug, PartialEq)]
#[command(
    name = "forbidden-strings",
    version,
    about = "Linear-time deny-list scanner for Git repos.",
    help_template = HELP_TEMPLATE,
    after_help = AFTER_HELP,
    args_override_self = true,
)]
pub struct Cli {
    /// Optional non-scan operation selected before scan flags.
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    /// Optional path to the rule file passed with `--rules`.
    // What:     `#[arg(...)] pub rules_path: Option<String>` declares an optional
    //           long option named `--rules` whose value is parsed as an owned
    //           `String`. `Option<String>` means `Some(path)` when the user passed
    //           the flag and `None` when they did not; sibling `String` would need
    //           a default before env fallback runs.
    // Why:      `lib.rs` must know whether CLI syntax supplied a path so it can
    //           apply the existing precedence: flag, env var, then default file.
    //           `allow_hyphen_values` preserves the old manual parser behaviour
    //           where the token after `--rules` was always the path, even when it
    //           began with `-`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // @option("--rules <PATH>")
    // rulesPath?: string;
    // ```
    #[arg(
        long = "rules",
        value_name = "PATH",
        allow_hyphen_values = true,
        help = "Path to the rule file (tail-format sections or legacy lines)"
    )]
    pub rules_path: Option<String>,

    /// Whether to scan every Git-tracked file under the current directory.
    // What:     `pub all: bool` declares a boolean switch filled by `--all`. The
    //           sibling shape would be `Option<bool>`, but a flag is present or
    //           absent, so `false` is the correct default.
    // Why:      The run loop needs one typed branch condition for walker mode
    //           versus explicit positional files.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // @flag("--all")
    // all: boolean;
    // ```
    #[arg(long = "all", help = "Scan every git-tracked file under cwd")]
    pub all: bool,

    /// Whether to append the embedded betterleaks-ported baseline ruleset.
    // What:     `pub builtin_rules: bool` declares a boolean switch filled by
    //           `--builtin-rules`, same shape as `all`: a flag is present or
    //           absent, so `false` is the correct default (sibling
    //           `Option<bool>` would model a third state that cannot occur).
    // Why:      The baseline ships inside the binary but must stay opt-in:
    //           silently adding rules would change scan results for existing
    //           users of the published CLI.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // @flag("--builtin-rules")
    // builtinRules: boolean;
    // ```
    #[arg(
        long = "builtin-rules",
        help = "Also scan with the embedded betterleaks-ported baseline rules"
    )]
    pub builtin_rules: bool,

    /// Logical repository path for each positional file,
    ///  in matching order.
    ///
    /// Allows clients scanning historical candidate bytes from temporary files
    /// to name-scan the actual path without exposing their temporary path.
    /// Each occurrence pairs with one positional file;
    ///  no `--all` combination.
    #[arg(long = "name-path", value_name = "PATH", allow_hyphen_values = true)]
    pub name_paths: Vec<String>,

    /// Files to scan when `--all` is absent.
    // What:     `Vec<String>` is an owned, growable array of owned UTF-8 strings.
    //           Siblings: `&[String]` (borrowed slice) and `[String; N]`
    //           (fixed-size array). `num_args = 0..` means zero or more path
    //           tokens are accepted.
    // Why:      Explicit file arguments are preserved exactly for the scanner,
    //           while clap owns the positional parsing and help text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // @positional({ many: true })
    // files: string[];
    // ```
    #[arg(value_name = "FILE", num_args = 0.., help = "File path to scan")]
    pub files: Vec<String>,
}
