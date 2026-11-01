//! Command-line argument parsing for the binary's launch path.
//!  The parser (which
//! file/folder paths to enqueue,
//!  and whether to auto-play) is declared here in the
//! library,
//!  not in `main.rs`,
//!  so it can be unit-tested with `clap`'s
//! `try_parse_from` without building the Slint window or starting an audio backend.

/// What:
///      `use std::path::PathBuf;`.
///  The OWNED filesystem path type:
///  a heap-
///           allocated,
///  growable path buffer.
///  Sibling:
///  `&Path`,
///  a BORROWED view
///           that does not own its bytes (the `String` vs `&str` distinction,
///  but
///           for paths).
/// Why:
///       Each positional argument is parsed straight into an owned `PathBuf`
///           that the caller can hand to the engine,
///  outliving this scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no import needed; a path is just a string
/// ```
use std::path::PathBuf;

/// What:
///      `use clap::Parser;`.
///  Brings TWO things into scope under one name:
///  the
///           `Parser` TRAIT (whose `parse()` / `try_parse_from()` methods read the
///           arguments) and the `#[derive(Parser)]` MACRO that generates that
///           trait's implementation for our struct.
/// Why:
///       The derive macro reads the struct below and writes the whole argv
///           scanner for us,
///  and the trait gives `Cli::parse()` to the binary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseArgs } from "some-cli-parser";
/// ```
use clap::Parser;

/// What:
///      `#[derive(Parser, Debug, PartialEq)]` asks the compiler to auto-generate
///           three trait implementations for the struct on the next line:
///           - `Parser`:
///  the clap-generated argv -> `Cli` parser (this is the macro
///             that replaces a hand-written argument loop).
///           - `Debug`:
///  lets `{:?}` format a `Cli` (used by test failure messages).
///           - `PartialEq`:
///  lets `==` / `assert_eq!` compare two `Cli` values.
///           `#[command(...)]` configures the program-level metadata clap prints in
///           `--help`:
///  the program `name`,
///  `version` (pulled from `CARGO_PKG_VERSION`
///           at build time),
///  and the one-line `about`.
/// Why:
///       Declaring the CLI as data on a struct,
///  then deriving the parser,
///  is the
///           whole point of adopting clap:
///  it replaces hand-rolled argv scanning with
///           a single source of truth that also generates `--help` / `--version`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // @cli({ name: "music-player", version, about: "..." })
/// // class Cli { ... }  // decorator generates the parser from the fields
/// ```
#[derive(Parser, Debug, PartialEq)]
#[command(
    name = "music-player",
    version,
    about = "Minimal native music player. Opens one folder (scanned recursively) as the source root, or one file (its parent folder becomes the source root and the file is preselected), loaded PAUSED unless --start-playing is passed."
)]
pub struct Cli {
    /// What:
    ///      `#[arg(long = "start-playing")] pub start_playing: bool`.
    ///  The
    ///           `#[arg(long = "...")]` attribute tells clap this field is an OPTIONAL
    ///           `--start-playing` flag (a `bool` field with no value becomes a
    ///           presence flag:
    ///  absent -> `false`,
    ///  present -> `true`).
    ///  `pub` exposes
    ///           the field to the binary and the tests.
    /// Why:
    ///       This flag is the ONLY way to ask a command-line launch to begin
    ///           playing immediately;
    ///  without it,
    ///  opened paths load paused.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // @option("--start-playing") start_playing: boolean = false;
    /// ```
    #[arg(
        long = "start-playing",
        help = "Begin playback immediately on launch instead of loading the queue paused"
    )]
    pub start_playing: bool,

    /// What:
    ///      `#[arg(long = "licenses")] pub licenses: bool`.
    ///  A presence flag like `--start-playing`.
    /// Why:
    ///       Prints the license notices the binary carries (`src/licenses.rs`) and exits without opening a window;
    ///  the single Windows executable has no other place for them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // @option("--licenses") licenses: boolean = false;
    /// ```
    #[arg(
        long = "licenses",
        help = "Print the license notices of the program and of every component built into it, then exit"
    )]
    pub licenses: bool,

    /// What:
    ///      `pub path: Option<PathBuf>`.
    ///  A field with NO `#[arg(...)]` flag attribute
    ///           is a POSITIONAL argument;
    ///  typed as `Option<PathBuf>`,
    ///  clap accepts ZERO or
    ///           ONE positional token (a second one is a parse error),
    ///  parsing it into a
    ///           `PathBuf`.
    ///  `Option<T>` is `Some(value)` or `None` (no path given).
    /// Why:
    ///       Exactly one Source Root is loaded,
    ///  so the CLI takes at most one path:
    ///  a
    ///           folder,
    ///  or a single file whose parent folder becomes the root.
    ///  The old
    ///           multi-path form is deliberately removed;
    ///  passing two paths now errors.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // @positional() path?: string;
    /// ```
    #[arg(help = "One folder (scanned recursively) or one file (its parent folder becomes the source root)")]
    pub path: Option<PathBuf>,
}

/// What:
///      `pub fn command_with_color(color: clap::ColorChoice) -> clap::Command`.
///  The clap command `Cli` derives, with its color setting replaced by `color`.
/// Why:
///       clap reads the setting when it prints help and errors, so the parser below builds from this command.
pub fn command_with_color(color: clap::ColorChoice) -> clap::Command {
    return <Cli as clap::CommandFactory>::command().color(color);
}

/// What:
///      `pub fn try_parse_from_with_color<I, T>(arguments: I, color: clap::ColorChoice) -> Result<Cli, clap::Error>`.
///  Parses `arguments` (program name first) with clap's color setting replaced by `color`.
/// Why:
///       The Windows build turns clap's colors off when a console refused ANSI escape sequences
///           (`console_color::cli_color`); the setting has to reach the help and error text clap prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tryParseWithColor(argv: string[], color: "auto" | "never"): Cli { ... } // throws on bad input
/// ```
pub fn try_parse_from_with_color<I, T>(arguments: I, color: clap::ColorChoice) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let matches = command_with_color(color).try_get_matches_from(arguments)?;
    return <Cli as clap::FromArgMatches>::from_arg_matches(&matches);
}

/// What:
///      `pub fn parse_with_color(color: clap::ColorChoice) -> Cli`.
///  `Cli::parse()` with clap's color setting replaced by `color`: reads the process arguments, and on `--help`,
///  `--version` or a bad argument prints and exits the process as `Cli::parse()` does.
/// Why:
///       Same contract as `Cli::parse()`, so `main` swaps one call for the other on Windows only.
pub fn parse_with_color(color: clap::ColorChoice) -> Cli {
    return try_parse_from_with_color(std::env::args_os(), color).unwrap_or_else(|error| error.exit());
}

/// What:
///      `#[cfg(test)] #[path = "cli_tests.rs"] mod tests;`.
///  Pull the unit tests
///           in from the flat sibling file `cli_tests.rs` as a child module named
///           `tests`.
///  `#[cfg(test)]` compiles it ONLY under `cargo test` / `cargo
///           nextest run`;
///  `#[path = "..."]` overrides the default `cli/tests.rs`
///           lookup so the file can sit beside this one.
/// Why:
///       Keep the tests next to the code without inflating this file or its
///           max-lines budget (sibling `*_tests.rs` files are linter-exempt),
///  and let
///           them reach private items via `use super::*`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // import "./cli_tests"; // test runner picks it up automatically
/// ```
#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
