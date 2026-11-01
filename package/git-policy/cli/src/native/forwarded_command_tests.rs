//! Controls for forwarded-command resolution:
//!  alias value splitting,
//!  alias lookup through Git's
//! `GIT_CONFIG_*` overlay,
//!  expansion limits,
//!  loops,
//!  shell aliases and worktree creation.

use super::*;
use crate::test_support::isolated_overlay;
use crate::transaction_git::GitContext;

/// A context whose only aliases are the given name and value pairs,
///  read by real Git.
fn context_with_aliases(pairs: &[(&str, &str)]) -> GitContext {
    let mut overlay: Vec<(OsString, OsString)> = isolated_overlay();
    overlay.push((OsString::from("GIT_CONFIG_COUNT"), OsString::from(pairs.len().to_string())));
    for (index, (name, value)) in pairs.iter().enumerate() {
        overlay.push((
            OsString::from(format!("GIT_CONFIG_KEY_{index}")),
            OsString::from(format!("alias.{name}")),
        ));
        overlay.push((
            OsString::from(format!("GIT_CONFIG_VALUE_{index}")),
            OsString::from(*value),
        ));
    }
    return GitContext {
        real_git: std::path::PathBuf::from(crate::test_support::REAL_GIT),
        overlay,
        global_prefix: Vec::new(),
        common_dir: None,
    };
}

/// Arguments as the wrapper receives them.
fn args(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// One word as the bytes the splitter produces for it.
fn word_bytes(word: &&str) -> Vec<u8> {
    return word.as_bytes().to_vec();
}

/// Alias value words as bytes,
///  for comparing the splitter's output.
fn words(list: &[&str]) -> Option<Vec<Vec<u8>>> {
    return Some(list.iter().map(word_bytes).collect());
}

/// Whitespace separates words;
///  quotes group;
///  backslashes escape outside single quotes.
#[test]
fn alias_values_split_as_git_splits_them() {
    assert_eq!(split_alias_command(b"commit -a"), words(&["commit", "-a"]));
    assert_eq!(split_alias_command(b"a \t  b"), words(&["a", "b"]));
    // The incumbent trims before splitting; untrimmed leading whitespace yields an empty first word.
    assert_eq!(split_alias_command(b"  a"), words(&["", "a"]));
    assert_eq!(split_alias_command(b"'a b' c"), words(&["a b", "c"]));
    assert_eq!(split_alias_command(b"\"x y\""), words(&["x y"]));
    assert_eq!(split_alias_command(b"a\\ b"), words(&["a b"]));
    assert_eq!(split_alias_command(b"'a\\b'"), words(&["a\\b"]));
    assert_eq!(split_alias_command(b"\"a\\\"b\""), words(&["a\"b"]));
    assert_eq!(split_alias_command(b"''"), words(&[""]));
}

/// An empty value is one empty word,
///  as the incumbent splits it,
///  and an unfinished quote or a
/// trailing backslash yields no words at all.
#[test]
fn empty_and_unfinished_values_split_to_one_empty_word_or_fail() {
    assert_eq!(split_alias_command(b""), words(&[""]));
    assert_eq!(split_alias_command(b"   "), words(&[""]));
    assert_eq!(split_alias_command(b"'a"), None);
    assert_eq!(split_alias_command(b"\"a"), None);
    assert_eq!(split_alias_command(b"a\\"), None);
}

/// Surrounding ASCII whitespace goes;
///  invalid UTF-8 keeps its non-space bytes.
#[test]
fn alias_values_are_trimmed_before_splitting() {
    assert_eq!(trim_alias_value(b"  commit \n"), b"commit".to_vec());
    assert_eq!(trim_alias_value(b" \t "), Vec::<u8>::new());
    assert_eq!(trim_alias_value(&[0xff, b' ', b'x', b' ']), vec![0xff, b' ', b'x']);
    assert_eq!(trim_alias_value(&[b' ', 0xff]), vec![0xff]);
}

/// A built-in named directly is resolved without an alias lookup and keeps its global options.
#[test]
fn a_direct_builtin_resolves_past_its_global_options() {
    let context: GitContext = context_with_aliases(&[]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, &args(&["-C", "/x", "status"]));
    assert_eq!(resolved.route, Route::Direct);
    assert_eq!(resolved.word(), b"status");
    assert_eq!(resolved.subcommand_index, 2);
    assert!(resolved.is_builtin());
}

/// A bare `git --version` has no subcommand and is not a built-in command.
#[test]
fn a_query_has_no_subcommand() {
    let context: GitContext = context_with_aliases(&[]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, &args(&["--version"]));
    assert_eq!(resolved.subcommand, None);
    assert_eq!(resolved.word(), b"");
    assert!(!resolved.is_builtin());
}

/// An alias to a built-in expands to its words,
///  with the rest of the original arguments after them.
#[test]
fn an_alias_expands_to_its_words_then_the_rest() {
    let context: GitContext = context_with_aliases(&[("fixturecommit", "commit -v")]);
    let resolved: ResolvedCommand =
        resolve_forwarded_command(&context, &args(&["fixturecommit", "-m", "x"]));
    assert_eq!(resolved.route, Route::Alias);
    assert_eq!(resolved.word(), b"commit");
    assert_eq!(resolved.arguments, args(&["commit", "-v", "-m", "x"]));
    assert!(resolved.is_builtin());
}

/// Chained aliases expand one after another until a built-in is reached.
#[test]
fn chained_aliases_resolve_to_the_builtin_at_the_end() {
    let context: GitContext = context_with_aliases(&[("fixturex", "fixturey -a"), ("fixturey", "commit")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, &args(&["fixturex"]));
    assert_eq!(resolved.route, Route::Alias);
    assert_eq!(resolved.arguments, args(&["commit", "-a"]));
}

/// A built-in name is never replaced by an alias of the same name,
///  as Git ignores such aliases.
#[test]
fn a_builtin_name_is_not_shadowed_by_an_alias() {
    let context: GitContext = context_with_aliases(&[("commit", "status")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, &args(&["commit"]));
    assert_eq!(resolved.route, Route::Direct);
    assert_eq!(resolved.word(), b"commit");
}

/// A `!` alias runs a shell command,
///  so it is never a built-in whatever its text says.
#[test]
fn a_shell_alias_is_never_a_builtin() {
    let context: GitContext = context_with_aliases(&[("fixtureshell", "!git commit -a")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, &args(&["fixtureshell"]));
    assert_eq!(resolved.route, Route::ShellAlias);
    assert!(!resolved.is_builtin());
    assert!(!creates_or_moves_worktrees(&resolved));
}

/// An alias that names itself,
///  directly or through another,
///  is an unresolved loop.
#[test]
fn an_alias_loop_is_unresolved() {
    let context: GitContext = context_with_aliases(&[("fixtureloopa", "fixtureloopb"), ("fixtureloopb", "fixtureloopa")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, &args(&["fixtureloopa"]));
    assert_eq!(resolved.route, Route::Unresolved);
    assert!(!resolved.is_builtin());
}

/// An unknown name with no alias is unresolved and never a built-in.
#[test]
fn an_unknown_name_is_unresolved() {
    let context: GitContext = context_with_aliases(&[]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, &args(&["fixture-no-such-command"]));
    assert_eq!(resolved.route, Route::Unresolved);
    assert!(!resolved.is_builtin());
}

/// A value whose quoting cannot be split is unresolved,
///  not guessed.
#[test]
fn a_malformed_alias_value_is_unresolved() {
    let context: GitContext = context_with_aliases(&[("fixturebad", "'commit")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, &args(&["fixturebad"]));
    assert_eq!(resolved.route, Route::Unresolved);
}

/// An empty alias value has no command word and is unresolved.
#[test]
fn an_empty_alias_value_is_unresolved() {
    let context: GitContext = context_with_aliases(&[("fixtureempty", "   ")]);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, &args(&["fixtureempty"]));
    assert_eq!(resolved.route, Route::Unresolved);
}

/// A chain of `length` aliases,
///  each naming the next,
///  the last naming `commit`.
fn chain_context(length: usize) -> (GitContext, OsString) {
    let mut names: Vec<String> = Vec::new();
    for index in 0..length {
        names.push(format!("fixturechain{index}"));
    }
    let mut pairs: Vec<(String, String)> = Vec::new();
    for index in 0..length {
        let target: String = if index + 1 < length {
            names[index + 1].clone()
        } else {
            String::from("commit")
        };
        pairs.push((names[index].clone(), target));
    }
    let mut pair_refs: Vec<(&str, &str)> = Vec::new();
    for (name, target) in &pairs {
        pair_refs.push((name.as_str(), target.as_str()));
    }
    return (context_with_aliases(pair_refs.as_slice()), OsString::from(names[0].as_str()));
}

/// Sixteen chained aliases,
///  each one expansion,
///  reach the built-in;
///  the limit is 16 expansions.
#[test]
fn sixteen_chained_aliases_reach_the_builtin() {
    assert_eq!(MAX_ALIAS_EXPANSIONS, 16);
    let (context, first): (GitContext, OsString) = chain_context(16);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, &[first]);
    assert_eq!(resolved.route, Route::Alias);
    assert_eq!(resolved.word(), b"commit");
}

/// Seventeen chained aliases need one expansion more than the limit allows and stay unresolved.
#[test]
fn seventeen_chained_aliases_are_unresolved() {
    let (context, first): (GitContext, OsString) = chain_context(17);
    let resolved: ResolvedCommand = resolve_forwarded_command(&context, &[first]);
    assert_eq!(resolved.route, Route::Unresolved);
    assert!(!resolved.is_builtin());
}

/// `worktree add` and `worktree move` create or move a linked worktree;
///  other forms do not.
#[test]
fn only_worktree_add_and_move_create_or_move_worktrees() {
    let context: GitContext = context_with_aliases(&[]);
    let add: ResolvedCommand = resolve_forwarded_command(&context, &args(&["worktree", "add", "x"]));
    let moved: ResolvedCommand = resolve_forwarded_command(&context, &args(&["worktree", "move", "a", "b"]));
    let verbose_add: ResolvedCommand =
        resolve_forwarded_command(&context, &args(&["worktree", "--verbose", "add", "x"]));
    let list: ResolvedCommand = resolve_forwarded_command(&context, &args(&["worktree", "list"]));
    let bare: ResolvedCommand = resolve_forwarded_command(&context, &args(&["worktree"]));
    let other: ResolvedCommand = resolve_forwarded_command(&context, &args(&["status", "add"]));
    assert!(creates_or_moves_worktrees(&add));
    assert!(creates_or_moves_worktrees(&moved));
    assert!(creates_or_moves_worktrees(&verbose_add));
    assert!(!creates_or_moves_worktrees(&list));
    assert!(!creates_or_moves_worktrees(&bare));
    assert!(!creates_or_moves_worktrees(&other));
}
