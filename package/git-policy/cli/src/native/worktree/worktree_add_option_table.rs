//! What:
//!  The option table of `git worktree add`,
//!  and the rules that read it:
//! which option a long word names,
//! how a bundled short word is read,
//!  and whether the value an option carries is one Git accepts.
//! Why:
//!   The table is the one list of the command's options,
//! read against `git worktree add -h` by a control so it cannot drift from Git;
//! an exact name,
//! a unique prefix and the `--no-` form resolve as `parse-options.c` of Git 2.56.0 resolves them,
//! and one place that reads the rules keeps every reading of the command line agreeing about which words are operands.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const table: Option[] = [{ short: 'b', long: undefined, takes: 'value' }];
//! ```

use std::ffi::OsString;

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
pub(crate) fn option_for_short_letter(letter: u8) -> Option<&'static AddOption> {
    for candidate in ADD_OPTIONS {
        if candidate.short == Some(letter) {
            return Some(candidate);
        }
    }
    return None;
}

/// How an option of `git worktree add` takes its value,
/// as `parse-options.c` of Git 2.56.0 does (`do_get_value`,
/// line 136).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Takes {
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
pub(crate) struct AddOption {
    /// The letter of the short form,
    /// when there is one.
    pub(crate) short: Option<u8>,
    /// The long name,
    /// when there is one;
    /// every long option also has a `--no-` form.
    pub(crate) long: Option<&'static str>,
    /// How the option takes its value.
    pub(crate) takes: Takes,
}

/// The options of `git worktree add` in the order of `add` in `builtin/worktree.c` (Git 2.56.0,
/// lines 821 to 862),
/// the one
/// table every reading of the command line uses.
/// `git worktree add -h` lists these,
/// and a control compares the two,
/// so the table cannot drift from Git unnoticed.
pub(crate) const ADD_OPTIONS: &[AddOption] = &[
    AddOption {
        short: Some(b'f'),
        long: Some("force"),
        takes: Takes::Nothing,
    },
    AddOption {
        short: Some(b'b'),
        long: None,
        takes: Takes::Required,
    },
    AddOption {
        short: Some(b'B'),
        long: None,
        takes: Takes::Required,
    },
    AddOption {
        short: None,
        long: Some("orphan"),
        takes: Takes::Nothing,
    },
    AddOption {
        short: Some(b'd'),
        long: Some("detach"),
        takes: Takes::Nothing,
    },
    AddOption {
        short: None,
        long: Some("checkout"),
        takes: Takes::Nothing,
    },
    AddOption {
        short: None,
        long: Some("lock"),
        takes: Takes::Nothing,
    },
    AddOption {
        short: None,
        long: Some("reason"),
        takes: Takes::Required,
    },
    AddOption {
        short: Some(b'q'),
        long: Some("quiet"),
        takes: Takes::Nothing,
    },
    AddOption {
        short: None,
        long: Some("track"),
        takes: Takes::Nothing,
    },
    AddOption {
        short: None,
        long: Some("guess-remote"),
        takes: Takes::Nothing,
    },
    AddOption {
        short: None,
        long: Some("relative-paths"),
        takes: Takes::Nothing,
    },
];

/// The option a long argument names and how,
/// resolved exactly as `parse_long_opt` of Git 2.56.0 does (`parse-options.c`,
/// lines
/// 520 to 593).
pub(crate) struct LongOption<'a> {
    /// The option the text names,
    /// by its full name or by a unique prefix.
    pub(crate) option: &'static AddOption,
    /// Whether the `--no-` form was written.
    pub(crate) unset: bool,
    /// The text after `=`,
    /// when there is one.
    pub(crate) attached: Option<&'a str>,
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
pub(crate) fn resolve_long(arg: &str) -> Option<LongOption<'_>> {
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
pub(crate) fn resolve_in<'a>(
    table: &'static [AddOption],
    arg: &'a str,
) -> Option<LongOption<'a>> {
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
                return Some(LongOption {
                    option,
                    unset,
                    attached: Some(value),
                });
            }
            if rest.is_empty() {
                return Some(LongOption {
                    option,
                    unset,
                    attached: None,
                });
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
    let attached: Option<&str> = if arg_end < arg.len() {
        Some(&arg[arg_end + 1..])
    } else {
        None
    };
    return Some(LongOption {
        option: matched,
        unset: matched_unset,
        attached,
    });
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
pub(crate) fn take_value<'a>(
    option: &AddOption,
    unset: bool,
    attached: Option<&str>,
    tokens: &mut impl Iterator<Item = &'a OsString>,
) -> Option<()> {
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

/// Option-table controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_add_option_table_tests.rs"]
mod tests;
