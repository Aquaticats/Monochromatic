//! Controls for the commit argument transforms,
//! including the incumbent's quirks.

use super::*;

/// Builds owned argument lists from string literals.
fn args(items: &[&str]) -> Vec<String> {
    return items.iter().map(|item| { return String::from(*item); }).collect();
}

/// A cluster carrying a value is split into flags and the value token;
/// a pathspec tail is kept verbatim.
#[test]
fn normalise_splits_inline_values_and_keeps_the_tail() {
    assert_eq!(normalise_commit_args(&args(&["-amhello"])), args(&["-a", "-m", "hello"]));
    assert_eq!(normalise_commit_args(&args(&["-m", "x"])), args(&["-m", "x"]));
    assert_eq!(normalise_commit_args(&args(&["--message=x"])), args(&["--message=x"]));
    assert_eq!(
        normalise_commit_args(&args(&["-m", "x", "--", "-mfoo"])),
        args(&["-m", "x", "--", "-mfoo"])
    );
    assert_eq!(normalise_commit_args(&args(&["-mé"])), args(&["-m", "é"]));
}

/// An option whose value is attached (`-S<keyid>`,
/// `-u<mode>`) keeps its value in the same token,
/// even when the value spells a letter that carries a value elsewhere;
/// the tail of such a token is a value and never a cluster of flags.
#[test]
fn normalise_keeps_attached_optional_values_whole() {
    assert_eq!(normalise_commit_args(&args(&["-S6ABCDEF"])), args(&["-S6ABCDEF"]));
    assert_eq!(normalise_commit_args(&args(&["-unormal"])), args(&["-unormal"]));
    assert_eq!(normalise_commit_args(&args(&["-aS6ABCDEF"])), args(&["-aS6ABCDEF"]));
    assert_eq!(normalise_commit_args(&args(&["-Sanother", "file.txt"])), args(&["-Sanother", "file.txt"]));
    assert_eq!(without_all_flag(&args(&["-aunormal"])), args(&["-unormal"]));
    assert_eq!(without_all_flag(&args(&["-aS6ABCDEF"])), args(&["-S6ABCDEF"]));
}

/// Global options that redirect Git are dropped with their values;
/// other globals stay in order.
#[test]
fn split_drops_redirecting_globals_and_keeps_the_rest() {
    let input: Vec<String> = args(&[
        "-C", "r", "-c", "a.b=1", "--git-dir=g", "--bare", "commit", "-m", "x",
    ]);
    let (globals, commit) = split_commit_invocation(&input, 6);
    assert_eq!(globals, args(&["-c", "a.b=1"]));
    assert_eq!(commit, args(&["-m", "x"]));
}

/// `-a` leaves the region,
/// a cluster keeps its other letters,
/// and a value-taking letter stays with its value.
#[test]
fn without_all_flag_removes_only_the_all_letter() {
    assert_eq!(without_all_flag(&args(&["-am", "x"])), args(&["-m", "x"]));
    assert_eq!(without_all_flag(&args(&["-a", "-m", "x"])), args(&["-m", "x"]));
    assert_eq!(without_all_flag(&args(&["--all"])), Vec::<String>::new());
    assert_eq!(without_all_flag(&args(&["-aSkey"])), args(&["-Skey"]));
    assert_eq!(without_all_flag(&args(&["-a"])), Vec::<String>::new());
}

/// A value-taking option's value is kept even when it spells `--all`,
/// as the incumbent keeps it.
#[test]
fn without_all_flag_keeps_an_all_spelled_as_a_message() {
    assert_eq!(without_all_flag(&args(&["-m", "--all"])), args(&["-m", "--all"]));
}

/// After the pathspec separator,
/// every token stays,
/// even `-a`.
#[test]
fn without_all_flag_keeps_everything_after_the_separator() {
    assert_eq!(
        without_all_flag(&args(&["-m", "x", "--", "-a"])),
        args(&["-m", "x", "--", "-a"])
    );
}

/// The explicit-path removal drops the last occurrence of each pathspec that is a path,
/// so the message the person typed stays even when it spells a pathspec:
/// native Git reads `git commit a.txt -m a.txt` as the path `a.txt` and the message `a.txt`.
#[test]
fn explicit_removal_keeps_a_message_that_equals_a_pathspec() {
    assert_eq!(
        private_explicit_commit_args(&args(&["a.txt", "-m", "a.txt"]), &args(&["a.txt"])),
        args(&["-m", "a.txt"])
    );
    assert_eq!(
        private_explicit_commit_args(&args(&["-m", "a.txt", "a.txt"]), &args(&["a.txt"])),
        args(&["-m", "a.txt"])
    );
    assert_eq!(
        private_explicit_commit_args(
            &args(&["a.txt", "-m", "a.txt", "a.txt"]),
            &args(&["a.txt", "a.txt"])
        ),
        args(&["-m", "a.txt"])
    );
}

/// Git reads every unambiguous prefix of a long option,
/// so an abbreviated `--message` still takes the next token as its value and a value that spells
///  `-o` or `--only` is never removed as that flag.
#[test]
fn an_abbreviated_value_option_takes_the_next_token() {
    assert!(takes_next_token("--mess"));
    assert!(takes_next_token("--auth"));
    assert!(!takes_next_token("--mess=hello"));
    assert!(!takes_next_token("--no-message"));
    assert!(!takes_next_token("--n"));
    assert_eq!(
        private_explicit_commit_args(&args(&["--mess", "-o", "--", "a.txt"]), &args(&["a.txt"])),
        args(&["--mess", "-o"])
    );
    assert_eq!(
        private_explicit_commit_args(&args(&["--mess", "a.txt", "a.txt"]), &args(&["a.txt"])),
        args(&["--mess", "a.txt"])
    );
}

/// The pathspec-file controls,
/// the separator and `--only` never reach the real commit.
#[test]
fn explicit_removal_drops_pathspec_file_controls_and_only() {    assert_eq!(
        private_explicit_commit_args(
            &args(&["--pathspec-from-file=f", "--pathspec-file-nul", "-m", "x"]),
            &[],
        ),
        args(&["-m", "x"])
    );
    assert_eq!(
        private_explicit_commit_args(&args(&["--pathspec-from-file", "f", "-m", "x"]), &[]),
        args(&["-m", "x"])
    );
    assert_eq!(
        private_explicit_commit_args(&args(&["-m", "x", "--only", "--", "a"]), &args(&["a"])),
        args(&["-m", "x"])
    );
}

/// A message that spells a pathspec-file control stays:
/// the value of an option is the person's text,
/// never the control it spells.
#[test]
fn a_message_that_spells_a_pathspec_file_control_stays() {
    assert_eq!(
        private_explicit_commit_args(&args(&["-m", "--pathspec-from-file=x"]), &[]),
        args(&["-m", "--pathspec-from-file=x"])
    );
    assert_eq!(
        private_explicit_commit_args(&args(&["-m", "--pathspec-file-nul"]), &[]),
        args(&["-m", "--pathspec-file-nul"])
    );
}

/// Git reads every unambiguous prefix of a long option,
/// so an abbreviated pathspec-file control and the file it names leave together:
/// the private index already holds the selection the preparation read from that file.
#[test]
fn an_abbreviated_pathspec_file_control_leaves_with_its_value() {
    assert_eq!(
        private_explicit_commit_args(&args(&["--pathspec-fr", "list.txt", "--message=x"]), &[]),
        args(&["--message=x"])
    );
    assert_eq!(
        private_explicit_commit_args(&args(&["--pathspec-fr=list.txt", "--message=x"]), &[]),
        args(&["--message=x"])
    );
    assert_eq!(
        private_explicit_commit_args(&args(&["--pathspec-from-fil", "list.txt"]), &[]),
        Vec::<String>::new()
    );
    // A message that spells the abbreviation is still the person's text.
    assert_eq!(
        private_explicit_commit_args(&args(&["-m", "--pathspec-fr"]), &[]),
        args(&["-m", "--pathspec-fr"])
    );
    // A spelling Git refuses as ambiguous is no control at all: the command fails as the person's spelling asks.
    assert_eq!(
        private_explicit_commit_args(&args(&["--pathspec-f", "list.txt"]), &[]),
        args(&["--pathspec-f", "list.txt"])
    );
}

/// Selection controls go away once the private index already holds the choice,
/// and a value that spells one is the person's text and stays.
#[test]
fn selection_removal_keeps_a_message_that_spells_a_selection_control() {
    assert_eq!(
        private_selection_commit_args(&args(&["--include", "--mess", "-i"]), &[]),
        args(&["--mess", "-i"])
    );
    assert_eq!(
        private_selection_commit_args(&args(&["-i", "-m", "--include"]), &[]),
        args(&["-m", "--include"])
    );
    assert_eq!(
        private_selection_commit_args(&args(&["-p", "--message", "-p"]), &[]),
        args(&["--message", "-p"])
    );
}

/// Selection controls leave in every spelling Git accepts and inside a short cluster,
/// so the real commit never runs a selection that already ran against the private index.
#[test]
fn selection_removal_drops_every_spelling_of_the_controls() {
    assert_eq!(
        private_selection_commit_args(&args(&["--includ", "--message=x"]), &[]),
        args(&["--message=x"])
    );
    assert_eq!(
        private_selection_commit_args(&args(&["--patc", "--message=x"]), &[]),
        args(&["--message=x"])
    );
    assert_eq!(
        private_selection_commit_args(&args(&["-im", "x"]), &[]),
        args(&["-m", "x"])
    );
    assert_eq!(
        private_selection_commit_args(&args(&["-ip", "-m", "x"]), &[]),
        args(&["-m", "x"])
    );
    // A gpg-signing key that spells a selection letter is its value and stays whole.
    assert_eq!(
        private_selection_commit_args(&args(&["-iSp", "-m", "x"]), &[]),
        args(&["-Sp", "-m", "x"])
    );
    // A message that spells an abbreviated control is still the person's text.
    assert_eq!(
        private_selection_commit_args(&args(&["-m", "--patc"]), &[]),
        args(&["-m", "--patc"])
    );
    // A value attached inside a cluster is the person's text, never a selection letter.
    assert_eq!(
        private_selection_commit_args(&args(&["--include", "-mp"]), &[]),
        args(&["-mp"])
    );
    assert_eq!(
        private_selection_commit_args(&args(&["-i", "-mip"]), &[]),
        args(&["-mip"])
    );
    assert_eq!(
        private_selection_commit_args(&args(&["-p", "-Skey-i"]), &[]),
        args(&["-Skey-i"])
    );
}

/// Selection controls go away once the private index already holds the choice.
#[test]
fn selection_removal_drops_selection_controls() {
    assert_eq!(
        private_selection_commit_args(&args(&["--patch", "-m", "x", "a"]), &args(&["a"])),
        args(&["-m", "x"])
    );
    assert_eq!(
        private_selection_commit_args(&args(&["-i", "-m", "x"]), &[]),
        args(&["-m", "x"])
    );
}

/// The real commit arguments depend on the mode and on whether selection ran privately.
#[test]
fn resolve_chooses_the_arguments_by_mode() {
    let input: Vec<String> = args(&["--patch", "-m", "x", "a"]);
    assert_eq!(
        resolve_private_commit_args(&input, &args(&["a"]), TransactionMode::ExplicitPath, false),
        args(&["--patch", "-m", "x"])
    );
    assert_eq!(
        resolve_private_commit_args(&input, &args(&["a"]), TransactionMode::Index, true),
        args(&["-m", "x"])
    );
    assert_eq!(
        resolve_private_commit_args(&input, &args(&["a"]), TransactionMode::Index, false),
        input
    );
}

/// Only `--` and `--end-of-options` in option position end the options:
/// as the value of `-m`,
/// `--message` or a cluster ending in `m` they are a
///  message,
/// and a value attached to its option or a letter that takes it attached leaves the next token in option position.
#[test]
fn the_end_of_the_options_is_read_in_option_position_only() {
    assert_eq!(boundary_index(&args(&["-m", "x", "--end-of-options", "-a"])), Some(2));
    assert_eq!(boundary_index(&args(&["-m", "--", "--", "a.txt"])), Some(2));
    assert_eq!(boundary_index(&args(&["--message", "--", "--", "a.txt"])), Some(2));
    assert_eq!(boundary_index(&args(&["--mess", "--", "--", "a.txt"])), Some(2));
    assert_eq!(boundary_index(&args(&["-qm", "--", "--"])), Some(2));
    assert_eq!(boundary_index(&args(&["-m", "--"])), None);
    assert_eq!(boundary_index(&args(&["-m--", "--", "a.txt"])), Some(1));
    assert_eq!(boundary_index(&args(&["--message=--", "--", "a.txt"])), Some(1));
    assert_eq!(boundary_index(&args(&["-S", "--", "a.txt"])), Some(1));
    assert_eq!(boundary_index(&args(&["-u", "--", "a.txt"])), Some(1));
    assert_eq!(boundary_index(&args(&["-a", "x", "--"])), Some(2));
    assert_eq!(boundary_index(&args(&[])), None);
}

/// The value of an option is never read as a cluster of options,
/// and never ends the options.
#[test]
fn a_value_that_looks_like_a_cluster_or_a_separator_is_kept_whole() {
    assert_eq!(normalise_commit_args(&args(&["-m", "-cut", "--", "-mfoo"])), args(&["-m", "-cut", "--", "-mfoo"]));
    // The message `--` is a value; the cluster after it is in option position, and the tail after the real separator is verbatim.
    assert_eq!(normalise_commit_args(&args(&["-m", "--", "-amx", "--", "-amx"])), args(&["-m", "--", "-a", "-m", "x", "--", "-amx"]));
}

/// A message that is `--` stays,
/// while the separator,
/// `-o` and the pathspec the index already holds go;
/// text equal to `-o` after
///  the separator or as a value is not the option.
#[test]
fn the_private_commit_arguments_drop_the_typed_separator_and_nothing_else() {
    assert_eq!(
        private_explicit_commit_args(&args(&["-o", "-m", "--", "--", "a.txt"]), &args(&["a.txt"])),
        args(&["-m", "--"])
    );
    assert_eq!(private_explicit_commit_args(&args(&["-m", "-o", "--", "a.txt"]), &args(&["a.txt"])), args(&["-m", "-o"]));
    assert_eq!(
        private_explicit_commit_args(&args(&["--only", "--message=x", "--", "-o"]), &args(&["-o"])),
        args(&["--message=x"])
    );
    assert_eq!(private_explicit_commit_args(&args(&["--message=x"]), &[]), args(&["--message=x"]));
}

/// What:
///  Git ends a commit region's options at `--end-of-options` exactly as at `--`,
///  so every token after it is a path,
///  and a path that spells an option's name is committed as the person's path.
/// Why:
///  Without it the region's reading marked a path as the value of a path before it,
///  left it out of the pathspec removal and handed the real commit a shorter list:
///  `git commit -m msg --end-of-options -m x` committed `x` alone and left the edit of `-m` uncommitted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(separatorIndex(['-m', 'x', '--end-of-options', '-m', 'x'])).toBe(2);
/// ```
#[test]
fn an_end_of_options_ends_the_option_region_as_the_separator_does() {
    assert_eq!(boundary_index(&args(&["-m", "x", "--end-of-options", "-m", "x"])), Some(2));
    assert_eq!(boundary_index(&args(&["--end-of-options"])), Some(0));
    // A `--end-of-options` that is the value of `-m` is a message and no boundary.
    assert_eq!(boundary_index(&args(&["-m", "--end-of-options"])), None);
    // The separator can come first, and then `--end-of-options` is one more path.
    assert_eq!(boundary_index(&args(&["-m", "x", "--", "--end-of-options"])), Some(2));
    // The region after either boundary is kept verbatim, so a cluster-shaped path is never split into flags.
    assert_eq!(
        normalise_commit_args(&args(&["-m", "x", "--end-of-options", "-amhello"])),
        args(&["-m", "x", "--end-of-options", "-amhello"])
    );
    // Every path after the boundary goes, and the real commit is left with the message alone.
    assert_eq!(
        private_explicit_commit_args(&args(&["-m", "msg", "--end-of-options", "-m", "x"]), &args(&["-m", "x"])),
        args(&["-m", "msg"])
    );
    assert_eq!(
        private_explicit_commit_args(&args(&["-m", "msg", "--end-of-options", "-amhello", "x"]), &args(&["-amhello", "x"])),
        args(&["-m", "msg"])
    );
    // `-a` and `--all` after the boundary are paths too, never the all flag.
    assert_eq!(
        without_all_flag(&args(&["-m", "x", "--end-of-options", "-a"])),
        args(&["-m", "x", "--end-of-options", "-a"])
    );
    assert_eq!(
        without_all_flag(&args(&["--end-of-options", "--all"])),
        args(&["--end-of-options", "--all"])
    );
}

/// What:
///  A selection control leaves in every spelling Git accepts,
///  in a short cluster and beside a value,
///  and a spelling Git refuses is left for Git to refuse.
/// Why:
///  The removal compared the exact spelling,
///  so `git commit --on a.txt -m msg` handed the real commit `--on` with no paths and it answered
///  "No paths with --include/--only does not make sense" with exit 128,
///  while every other reader of the same region already read Git's abbreviations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(privateExplicitCommitArgs({ args: ['--on', 'a.txt', '-m', 'msg'], pathspecs: ['a.txt'] })).toEqual(['-m', 'msg']);
/// ```
#[test]
fn an_abbreviated_only_leaves_where_git_reads_it() {
    // `--o` is `--only`: no other long option of `git commit` starts with `o`.
    for spelling in ["--on", "--o", "--onl", "--only"] {
        assert_eq!(
            private_explicit_commit_args(&args(&[spelling, "a.txt", "-m", "msg"]), &args(&["a.txt"])),
            args(&["-m", "msg"]),
            "{spelling}"
        );
    }
    // A short cluster carries it beside the letters that stay, and `-m` keeps its value.
    assert_eq!(
        private_explicit_commit_args(&args(&["-om", "msg", "a.txt"]), &args(&["a.txt"])),
        args(&["-m", "msg"])
    );
    assert_eq!(
        private_explicit_commit_args(&args(&["-qo", "-m", "msg", "a.txt"]), &args(&["a.txt"])),
        args(&["-q", "-m", "msg"])
    );
    // The `no-` form is another instruction: it selects the whole index and stays.
    assert_eq!(
        private_explicit_commit_args(&args(&["--no-only", "-m", "msg", "a.txt"]), &args(&["a.txt"])),
        args(&["--no-only", "-m", "msg"])
    );
    // A value on a flag is a spelling Git refuses, and the refusal stays Git's.
    assert_eq!(private_explicit_commit_args(&args(&["--only=x", "-m", "msg"]), &[]), args(&["--only=x", "-m", "msg"]));
    assert_eq!(private_explicit_commit_args(&args(&["--pathspec-file-nul=x"]), &[]), args(&["--pathspec-file-nul=x"]));
    assert_eq!(without_all_flag(&args(&["--all=x"])), args(&["--all=x"]));
    // A value that spells the control is the person's text.
    assert_eq!(private_explicit_commit_args(&args(&["-m", "--on"]), &[]), args(&["-m", "--on"]));
    assert_eq!(private_explicit_commit_args(&args(&["-m", "-o"]), &[]), args(&["-m", "-o"]));
    // `--pat` is ambiguous, so Git refuses it and no token of it is removed as a control.
    assert_eq!(
        private_explicit_commit_args(&args(&["--pat", "a.txt", "-m", "msg"]), &args(&["a.txt"])),
        args(&["--pat", "-m", "msg"])
    );
}
