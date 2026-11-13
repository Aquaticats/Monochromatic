//! What:
//!  Which options make a `git commit` a form real Git refuses,
//!  and which near-misses Git accepts.
//! Why:
//!  A rule wider than Git's would ask Git about a command that is fine,
//!  and one narrower would let a mistyped command make a commit;
//!  each control below is one of Git's own conditions,
//!  read against the region the parser builds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(formGitRefuses(region(['--include', '--only', '--message=x', '--', 'a.txt']))).toBe('selection-conflict');
//! ```

use super::{GitRefusedForm, form_git_refuses};
use crate::command_commit::{CommitRegion, FixupKind};

/// A region in which nothing is stated,
///  so each control states only what it is about.
fn quiet() -> CommitRegion {
    return CommitRegion {
        all: false,
        only: None,
        include: false,
        interactive: false,
        patch: false,
        dry_run: false,
        amend: false,
        allow_empty: false,
        fixup: None,
        pathspec_from_file: None,
        pathspec_file_nul: false,
        pathspecs: Vec::new(),
        wrapper: crate::command_options_query::WrapperFlags { escape: Vec::new(), other: Vec::new() },
    };
}

/// A region naming one path,
///  the smallest commit that names something.
fn naming_a_path() -> CommitRegion {
    let mut region: CommitRegion = quiet();
    region.pathspecs = vec![4];
    return region;
}

/// What:
///  Two or more of `-i`, `-o`, `-a` and `--interactive`/`-p` is the form Git refuses,
///  and one of them is not.
/// Why:
///   Git's `die_for_incompatible_opt4` counts exactly these four,
///  with `--patch` counted as `--interactive`.
#[test]
fn two_or_more_selection_options_are_refused_and_one_is_not() {
    let mut include_and_only: CommitRegion = naming_a_path();
    include_and_only.include = true;
    include_and_only.only = Some(true);
    assert_eq!(form_git_refuses(&include_and_only), Some(GitRefusedForm::SelectionConflict));

    let mut all_and_patch: CommitRegion = naming_a_path();
    all_and_patch.all = true;
    all_and_patch.patch = true;
    assert_eq!(form_git_refuses(&all_and_patch), Some(GitRefusedForm::SelectionConflict));

    // `--patch` and `--interactive` are one option to Git, so both together are still one.
    let mut both_spellings: CommitRegion = quiet();
    both_spellings.patch = true;
    both_spellings.interactive = true;
    assert_eq!(form_git_refuses(&both_spellings), None);

    let mut only: CommitRegion = naming_a_path();
    only.only = Some(true);
    assert_eq!(form_git_refuses(&only), None);

    // `--no-only` states the opposite, so it counts for nothing.
    let mut negated: CommitRegion = naming_a_path();
    negated.only = Some(false);
    negated.include = true;
    assert_eq!(form_git_refuses(&negated), None);
}

/// What:
///  `--fixup=reword:` refuses a path and refuses every selection option,
///  while the plain and `amend:` forms accept both.
/// Why:
///   Git's `check_fixup_reword_options` turns on `--only` itself and refuses exactly these,
///  and `--fixup=amend:` allows an empty change, which is a different form.
#[test]
fn rewording_refuses_a_path_or_a_selection_and_the_other_fixups_do_not() {
    let mut with_path: CommitRegion = naming_a_path();
    with_path.fixup = Some(FixupKind::Reword);
    assert_eq!(form_git_refuses(&with_path), Some(GitRefusedForm::RewordWithPath));

    let mut with_include: CommitRegion = quiet();
    with_include.fixup = Some(FixupKind::Reword);
    with_include.include = true;
    assert_eq!(form_git_refuses(&with_include), Some(GitRefusedForm::RewordWithSelection));

    let mut amend_fixup: CommitRegion = naming_a_path();
    amend_fixup.fixup = Some(FixupKind::Amend);
    amend_fixup.amend = true;
    assert_eq!(form_git_refuses(&amend_fixup), None);

    let mut plain_fixup: CommitRegion = naming_a_path();
    plain_fixup.fixup = Some(FixupKind::Plain);
    plain_fixup.include = true;
    assert_eq!(form_git_refuses(&plain_fixup), None);
}

/// What:
///  `--pathspec-from-file` refuses pathspec arguments,
///  `-a` and `--interactive`,
///  and accepts none of them alone.
/// Why:
///   Git checks these three in `prepare_index` before it reads the file,
///  so the answer never depends on what the file holds.
#[test]
fn a_pathspec_file_refuses_paths_all_and_interactive_and_accepts_itself() {
    let mut with_paths: CommitRegion = naming_a_path();
    with_paths.pathspec_from_file = Some(crate::command_options::OptionValue::Detached { token: 1 });
    assert_eq!(form_git_refuses(&with_paths), Some(GitRefusedForm::PathspecFromFileWithPaths));

    let mut with_all: CommitRegion = quiet();
    with_all.pathspec_from_file = Some(crate::command_options::OptionValue::Detached { token: 1 });
    with_all.all = true;
    assert_eq!(form_git_refuses(&with_all), Some(GitRefusedForm::PathspecFromFileWithAll));

    let mut with_interactive: CommitRegion = quiet();
    with_interactive.pathspec_from_file = Some(crate::command_options::OptionValue::Detached { token: 1 });
    with_interactive.interactive = true;
    assert_eq!(form_git_refuses(&with_interactive), Some(GitRefusedForm::PathspecFromFileWithInteractive));

    let mut alone: CommitRegion = quiet();
    alone.pathspec_from_file = Some(crate::command_options::OptionValue::Detached { token: 1 });
    assert_eq!(form_git_refuses(&alone), None);
}

/// What:
///  `-a` with a path is refused,
///  and `--pathspec-file-nul` without `--pathspec-from-file` is refused.
/// Why:
///   Git dies for both before it reads the index,
///  and each has its own sentence.
#[test]
fn all_with_a_path_and_a_nul_file_alone_are_refused() {
    let mut all_with_path: CommitRegion = naming_a_path();
    all_with_path.all = true;
    assert_eq!(form_git_refuses(&all_with_path), Some(GitRefusedForm::AllWithPaths));

    let mut nul_alone: CommitRegion = quiet();
    nul_alone.pathspec_file_nul = true;
    assert_eq!(form_git_refuses(&nul_alone), Some(GitRefusedForm::PathspecFileNulWithoutFromFile));

    let mut nul_with_file: CommitRegion = quiet();
    nul_with_file.pathspec_file_nul = true;
    nul_with_file.pathspec_from_file = Some(crate::command_options::OptionValue::Detached { token: 1 });
    assert_eq!(form_git_refuses(&nul_with_file), None);
}

/// What:
///  A pathless `-i` or `-o` is refused,
///  except where Git allows an empty change:
///  `--allow-empty`,
///  `--amend`,
///  and `--fixup=amend:`.
/// Why:
///   Git's own condition ends `(only && !allow_empty && (!amend || (fixup && strcmp(fixup_prefix, "amend"))))`,
///  and a rule that ignored those words would refuse a command Git runs.
#[test]
fn a_pathless_selection_is_refused_except_where_git_allows_an_empty_change() {
    let mut include: CommitRegion = quiet();
    include.include = true;
    assert_eq!(form_git_refuses(&include), Some(GitRefusedForm::NoPathsWithSelection));

    let mut only: CommitRegion = quiet();
    only.only = Some(true);
    assert_eq!(form_git_refuses(&only), Some(GitRefusedForm::NoPathsWithSelection));

    let mut allowed_empty: CommitRegion = quiet();
    allowed_empty.only = Some(true);
    allowed_empty.allow_empty = true;
    assert_eq!(form_git_refuses(&allowed_empty), None);

    let mut amending: CommitRegion = quiet();
    amending.only = Some(true);
    amending.amend = true;
    assert_eq!(form_git_refuses(&amending), None);

    let mut rewording_amend: CommitRegion = quiet();
    rewording_amend.only = Some(true);
    rewording_amend.amend = true;
    rewording_amend.fixup = Some(FixupKind::Reword);
    assert_eq!(form_git_refuses(&rewording_amend), Some(GitRefusedForm::RewordWithSelection));

    // With a pathspec file the count of paths is Git's to read, so this file stays silent.
    let mut from_file: CommitRegion = quiet();
    from_file.only = Some(true);
    from_file.pathspec_from_file = Some(crate::command_options::OptionValue::Detached { token: 1 });
    assert_eq!(form_git_refuses(&from_file), None);
}

/// What:
///  A plain commit of staged work is no form at all,
///  and costs the check nothing but the comparison.
/// Why:
///   Most commits are this one,
///  so the rule that recognizes it is the one that keeps the check free.
#[test]
fn a_plain_commit_is_no_form() {
    assert_eq!(form_git_refuses(&quiet()), None);
    assert_eq!(form_git_refuses(&naming_a_path()), None);
    let mut amended: CommitRegion = quiet();
    amended.amend = true;
    assert_eq!(form_git_refuses(&amended), None);
    assert_eq!(GitRefusedForm::SelectionConflict.name(), "selection-conflict");
}
