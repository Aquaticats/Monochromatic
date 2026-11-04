//! What:
//!  Differential controls of the tokenizer against Git 2.56.0's own option parser.
//! Why:
//!  Every single token of a broad alphabet,
//!  and every pair that starts with a
//!      context-setting token,
//!  is compared with the real binary instead of with
//!      expectations written by hand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const args of cases) expect(render(parseOptions(args))).toBe(await gitParseopt(args));
//! ```

/// The tokenizer modes,
///  the comparison harness and the fixture helpers.
use super::{DEFAULT_MODE, ParseMode};
use crate::command_test_parseopt::{Tally, compare, compare_singles_and_pairs, empty_tally};
use crate::command_test_support::{fixture, remove};
use std::path::PathBuf;

/// Tokens covering clusters,
///  values,
///  abbreviations,
///  negation,
///  typos,
///  help and boundaries.
/// The empty token is added separately because it cannot be written between spaces.
const ALPHABET: &str = "-a -am -amx -ma -m -u -ua -au -qa -aq -S -Skey -x -xv -ax -U3 -all
    -amend -no-verify -nq -h -ah -z -az - -- --end-of-options file --all --al --a --am
    --allow-empty --allow --allow-empty-m --no-all --no-a --no --n --no- --message
    --message= --message=v --mess=v --no-message --no-message=v --all=1 --verify
    --no-verify --no-no-verify --no-no-post --no-post --post-rewrite --untracked-files
    --untracked-files=no --unified --unified=3 --no-unified --hard --no-hard --in --inc
    --int --inter --inter- --d --dr --da --date --help --help-all --unknown --unknown=v
    --=v --no-dry --no-d";

/// First tokens of every compared pair:
///  each changes how Git reads the token after it
/// (a pending value,
///  a flag,
///  a positional or a boundary).
const CONTEXTS: &str = "-a -am -ma -m -u -au -S -x - -- --end-of-options file --all
    --no-all --message --message=v --no-message --untracked-files --unified --date
    --no-verify --hard --am --allow-empty --dr --inc";

/// A smaller alphabet for the two flag variants,
///  which change only boundary handling.
const BOUNDARY_ALPHABET: &str =
    "-a -m -am -u - -- --end-of-options file --all --message=v --no-all --unknown -h";

/// Split a token list and add the empty token.
fn tokens(list: &str) -> Vec<&str> {
    let mut result: Vec<&str> = list.split_whitespace().collect();
    result.push("");
    return result;
}

/// Flags `0`:
///  the mode of commit,
///  add,
///  push,
///  clean,
///  branch and status.
#[test]
fn default_mode_matches_git_parse_options_on_singles_and_context_pairs() {
    let directory: PathBuf = fixture("parseopt-default");
    let alphabet: Vec<&str> = tokens(ALPHABET);
    let tally: Tally = compare_singles_and_pairs(
        directory.as_path(),
        tokens(CONTEXTS).as_slice(),
        alphabet.as_slice(),
        DEFAULT_MODE,
        false,
    );
    // Positive control: the comparison reached accepted, refused and usage outcomes.
    assert_eq!(alphabet.len(), 77);
    assert!(tally.parsed > 500, "{}", tally.parsed);
    assert!(tally.refused > 300, "{}", tally.refused);
    assert!(tally.help > 50, "{}", tally.help);
    remove(directory.as_path());
}

/// `PARSE_OPT_KEEP_DASHDASH`:
///  the mode of reset,
///  checkout and stash subcommands.
#[test]
fn keep_dashdash_mode_matches_git_parse_options() {
    let directory: PathBuf = fixture("parseopt-keep-dashdash");
    let alphabet: Vec<&str> = tokens(BOUNDARY_ALPHABET);
    let tally: Tally = compare_singles_and_pairs(
        directory.as_path(),
        alphabet.as_slice(),
        alphabet.as_slice(),
        DEFAULT_MODE,
        true,
    );
    assert!(tally.parsed > 50 && tally.refused > 20 && tally.help > 10);
    remove(directory.as_path());
}

/// `PARSE_OPT_STOP_AT_NON_OPTION`:
///  the mode of an assumed `git stash push`.
#[test]
fn stop_at_non_option_mode_matches_git_parse_options() {
    let directory: PathBuf = fixture("parseopt-stop");
    let mode: ParseMode = ParseMode {
        keep_unknown: false,
        stop_at_non_option: true,
    };
    let alphabet: Vec<&str> = tokens(BOUNDARY_ALPHABET);
    let tally: Tally = compare_singles_and_pairs(
        directory.as_path(),
        alphabet.as_slice(),
        alphabet.as_slice(),
        mode,
        true,
    );
    assert!(tally.parsed > 50 && tally.refused > 10 && tally.help > 5);
    remove(directory.as_path());
}

/// Longer sequences mixing every feature,
///  as a third layer over singles and pairs.
#[test]
fn curated_sequences_match_git_parse_options() {
    let directory: PathBuf = fixture("parseopt-curated");
    let mut tally: Tally = empty_tally();
    for values in [
        vec!["-am", "msg", "file", "--", "-a"],
        vec!["file", "-m", "-a", "--no-all", "--all", "other"],
        vec!["-qnam", "--", "x"],
        vec!["--mess", "--", "--", "y"],
        vec!["-u", "no", "-uno", "--untracked-files=", "z"],
        vec!["--inter-hunk-context", "3", "--inter-hunk=4", "-U", "5"],
        vec!["--no-date", "x", "--date", "--no-date"],
        vec!["-x", "-a", "-xa", "--end-of-options", "-a"],
        vec!["--no-verify", "--verify", "--no-no-verify", "-n"],
        vec!["a", "b", "--amend", "--no-amend", "-", "", "c"],
        vec!["-S", "key", "-Skey", "--gpg-sign", "key", "--gpg-sign=key"],
        vec!["-m", "--help", "-m", "-h", "x"],
        vec!["file", "--unknown"],
        vec!["-a", "-m"],
        vec!["x", "y", "-ah"],
    ] {
        compare(
            directory.as_path(),
            DEFAULT_MODE,
            false,
            values.as_slice(),
            &mut tally,
        );
    }
    assert_eq!((tally.parsed, tally.refused, tally.help), (12, 2, 1));
    remove(directory.as_path());
}
