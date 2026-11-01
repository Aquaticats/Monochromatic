//! What:
//!  Differential harness comparing the tokenizer with `git rev-parse --parseopt`.
//! Why:
//!  That command runs Git 2.56.0's `parse_options` over a caller-supplied table and
//!      prints the normalized result,
//!  so expectations come from the real binary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(render(parseOptions(args))).toBe(await gitParseopt(spec, args));
//! ```

/// The tokenizer under comparison,
///  its queries and the real-Git fixture helpers.
use super::command_options::{
    Arity, Boundary, OptionError, OptionErrorKind, OptionSpec, ParseMode, ParsedOptions,
    parse_options, row,
};
use super::command_options_query::{positional_tokens, value_bytes};
use super::command_test_support::{git_with_input, os_arguments};
use std::ffi::OsString;
use std::path::Path;
use std::process::Output;

/// One row per spelling,
///  arity and negation class;
///  identifiers are unique for rendering.
pub(crate) const PARSEOPT_TABLE: &[OptionSpec] = &[
    row(1, Some(b'a'), Some("all"), Arity::None, true),
    row(2, Some(b'm'), Some("message"), Arity::Required, true),
    row(
        3,
        Some(b'u'),
        Some("untracked-files"),
        Arity::Optional,
        true,
    ),
    row(4, Some(b'S'), Some("gpg-sign"), Arity::Optional, true),
    row(5, Some(b'q'), None, Arity::None, true),
    row(6, Some(b'n'), Some("no-verify"), Arity::None, true),
    row(7, None, Some("no-post-rewrite"), Arity::None, true),
    row(8, None, Some("amend"), Arity::None, true),
    row(9, None, Some("allow-empty"), Arity::None, true),
    row(10, None, Some("allow-empty-message"), Arity::None, true),
    row(11, Some(b'U'), Some("unified"), Arity::Required, false),
    row(12, None, Some("hard"), Arity::None, false),
    row(13, Some(b'i'), Some("include"), Arity::None, true),
    row(14, None, Some("interactive"), Arity::None, true),
    row(15, None, Some("inter-hunk-context"), Arity::Required, false),
    row(16, None, Some("dry-run"), Arity::None, true),
    row(17, None, Some("date"), Arity::Required, true),
    row(18, Some(b'x'), None, Arity::Required, true),
];

/// How many comparisons ended in each outcome,
///  to prove a control exercised all three.
pub(crate) struct Tally {
    pub(crate) parsed: usize,
    pub(crate) refused: usize,
    pub(crate) help: usize,
}

/// Start counting from zero.
pub(crate) fn empty_tally() -> Tally {
    return Tally {
        parsed: 0,
        refused: 0,
        help: 0,
    };
}

/// Write the table in `git rev-parse --parseopt` specification syntax.
fn specification(table: &[OptionSpec]) -> String {
    let mut text: String = String::from("fixture [<options>] [<args>...]\n--\n");
    for spec in table {
        if let Some(short) = spec.short {
            text.push(char::from(short));
            if spec.long.is_some() {
                text.push(',');
            }
        }
        if let Some(long) = spec.long {
            text.push_str(long);
        }
        if spec.arity == Arity::Required {
            text.push('=');
        }
        if spec.arity == Arity::Optional {
            text.push('?');
        }
        if !spec.negatable {
            text.push('!');
        }
        text.push_str(" help\n");
    }
    return text;
}

/// Look up the row an occurrence refers to.
fn find(table: &[OptionSpec], id: u16) -> OptionSpec {
    for spec in table {
        if spec.id == id {
            return *spec;
        }
    }
    panic!("occurrence names an identifier outside the table: {id}");
}

/// Append one value or argument in Git's `sq_quote_buf` form.
fn push_quoted(out: &mut Vec<u8>, bytes: &[u8]) {
    assert!(!bytes.contains(&b'\'') && !bytes.contains(&b'!'));
    out.push(b'\'');
    out.extend_from_slice(bytes);
    out.push(b'\'');
}

/// Render a parse exactly as `parseopt_dump` prints it in `--stuck-long` form
/// (builtin/rev-parse.c:395-412,
///  554-556).
fn render(arguments: &[OsString], parsed: &ParsedOptions, keep_dashdash: bool) -> Vec<u8> {
    let mut out: Vec<u8> = b"set --".to_vec();
    for occurrence in &parsed.occurrences {
        let spec: OptionSpec = find(PARSEOPT_TABLE, occurrence.id);
        if occurrence.negated {
            out.extend_from_slice(b" --no-");
            out.extend_from_slice(spec.long.expect("negated long option").as_bytes());
        } else if let Some(long) = spec.long {
            out.extend_from_slice(b" --");
            out.extend_from_slice(long.as_bytes());
        } else {
            out.extend_from_slice(b" -");
            out.push(spec.short.expect("short-only option"));
        }
        if let Some(value) = occurrence.value {
            if spec.long.is_some() {
                out.push(b'=');
            }
            push_quoted(&mut out, value_bytes(arguments, value));
        }
    }
    out.extend_from_slice(b" --");
    let mut positionals: Vec<usize> = positional_tokens(parsed, arguments.len());
    // `if let ... && condition` (a let chain) needs both the variant and the flag.
    if let Boundary::DashDash(at) = parsed.boundary
        && keep_dashdash
    {
        positionals.push(at);
        positionals.sort_unstable();
    }
    for token in positionals {
        out.push(b' ');
        push_quoted(&mut out, arguments[token].as_encoded_bytes());
    }
    out.push(b'\n');
    return out;
}

/// Compare one argument list with the real binary and record the outcome kind.
pub(crate) fn compare(
    directory: &Path,
    mode: ParseMode,
    keep_dashdash: bool,
    values: &[&str],
    tally: &mut Tally,
) {
    let arguments: Vec<OsString> = os_arguments(values);
    let mut command: Vec<OsString> = os_arguments(&["rev-parse", "--parseopt", "--stuck-long"]);
    if keep_dashdash {
        command.push(OsString::from("--keep-dashdash"));
    }
    if mode.stop_at_non_option {
        command.push(OsString::from("--stop-at-non-option"));
    }
    command.push(OsString::from("--"));
    command.extend(arguments.clone());
    let output: Output = git_with_input(
        directory,
        command.as_slice(),
        specification(PARSEOPT_TABLE).as_bytes(),
    );
    let ours: Result<ParsedOptions, OptionError> =
        parse_options(arguments.as_slice(), PARSEOPT_TABLE, mode, &[]);
    if output.status.code() == Some(129) {
        let error: OptionError = ours.expect_err(&format!("Git refuses {values:?}"));
        assert_ne!(error.kind, OptionErrorKind::HelpRequested, "{values:?}");
        tally.refused += 1;
        return;
    }
    assert!(
        output.status.success(),
        "{values:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    if output.stdout.starts_with(b"cat <<") {
        let error: OptionError = ours.expect_err(&format!("Git prints usage for {values:?}"));
        assert_eq!(error.kind, OptionErrorKind::HelpRequested, "{values:?}");
        tally.help += 1;
        return;
    }
    let parsed: ParsedOptions = match ours {
        Ok(accepted) => accepted,
        Err(error) => panic!(
            "Git accepts {values:?} as {} but the tokenizer refused: {error}",
            String::from_utf8_lossy(&output.stdout)
        ),
    };
    assert_eq!(
        String::from_utf8_lossy(&render(arguments.as_slice(), &parsed, keep_dashdash)),
        String::from_utf8_lossy(&output.stdout),
        "{values:?}"
    );
    tally.parsed += 1;
}

/// Compare every single token of `alphabet`,
///  and every ordered pair that starts with a
/// token of `firsts`,
///  under one flag set.
pub(crate) fn compare_singles_and_pairs(
    directory: &Path,
    firsts: &[&str],
    alphabet: &[&str],
    mode: ParseMode,
    keep_dashdash: bool,
) -> Tally {
    let mut tally: Tally = empty_tally();
    for single in alphabet {
        compare(directory, mode, keep_dashdash, &[single], &mut tally);
    }
    for first in firsts {
        for second in alphabet {
            compare(directory, mode, keep_dashdash, &[first, second], &mut tally);
        }
    }
    return tally;
}
