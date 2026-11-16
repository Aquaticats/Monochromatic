//! What:
//!  Controls for the repository a global prefix names,
//!  and for the arguments that name one repository in another process.
//! Why:
//!  A commit's Git children and the resident service read the repository only when the arguments carry it,
//!  and a value Git reads as a value (the directory of a `-C`,
//!  the setting of a `-c`) must never be read as an option.

use super::{from_global_options, of_identity, of_paths, selected_repository, without_repository_selection};
use crate::worktree_identity::WorktreeIdentity;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Own a fixture vector of arguments.
fn arguments(values: &[&str]) -> Vec<OsString> {
    return values.iter().map(|value| { return OsString::from(*value); }).collect();
}

/// The selection as readable text.
fn text(values: &[OsString]) -> Vec<String> {
    return values.iter().map(|token| { return token.to_string_lossy().into_owned(); }).collect();
}

/// What:
///  A prefix that names no repository selects nothing.
/// Why:
///  The commit path must cost no extra Git start for the ordinary layout.
#[test]
fn a_prefix_that_names_no_repository_selects_nothing() {
    assert_eq!(from_global_options(Path::new("/work"), &[]), Vec::<OsString>::new());
    assert_eq!(
        from_global_options(Path::new("/work"), arguments(&["-c", "user.name=X", "--no-pager", "-C", "sub"]).as_slice()),
        Vec::<OsString>::new(),
        "a configuration option and a directory change name no repository"
    );
}

/// What:
///  The separated and inline spellings of `--git-dir` and `--work-tree` come out as absolute paths.
/// Why:
///  The resident service runs elsewhere and has to name the repository without the caller's directory.
#[test]
fn the_options_come_out_as_absolute_paths() {
    let work: &Path = Path::new("/work");
    assert_eq!(
        text(from_global_options(work, arguments(&["--git-dir", "/abs/cfg.git"]).as_slice()).as_slice()),
        vec!["--git-dir", "/abs/cfg.git"],
        "an absolute Git directory stays as written"
    );
    assert_eq!(
        text(from_global_options(work, arguments(&["--git-dir=/abs/cfg.git", "--work-tree", "/abs/wt"]).as_slice()).as_slice()),
        vec!["--git-dir", "/abs/cfg.git", "--work-tree", "/abs/wt"],
        "both spellings are read"
    );
    assert_eq!(
        text(from_global_options(work, arguments(&["--work-tree=."]).as_slice()).as_slice()),
        vec!["--work-tree", "/work/."],
        "a relative work tree is read from the directory the command runs in, as the caller spelled it"
    );
}

/// What:
///  A relative value is read from the directory the `-C` chain ended in,
///  as Git reads it.
/// Why:
///  Git sets the variables while it reads the options and resolves a relative one at setup,
///  after its last directory change.
#[test]
fn a_relative_value_is_read_from_the_directory_the_chain_reached() {
    let work: &Path = Path::new("/work");
    let selection: Vec<OsString> = from_global_options(work, arguments(&["-C", "sub", "--git-dir=../cfg.git", "--work-tree", ".."]).as_slice());
    assert_eq!(text(selection.as_slice()), vec!["--git-dir", "/work/sub/../cfg.git", "--work-tree", "/work/sub/.."]);

    let after: Vec<OsString> = from_global_options(work, arguments(&["--git-dir=../cfg.git", "-C", "sub"]).as_slice());
    assert_eq!(
        text(after.as_slice()),
        vec!["--git-dir", "/work/sub/../cfg.git"],
        "even when the option is written before the directory it is read from"
    );
    let empty: Vec<OsString> = from_global_options(work, arguments(&["-C", "", "--git-dir=cfg.git"]).as_slice());
    assert_eq!(text(empty.as_slice()), vec!["--git-dir", "/work/cfg.git"], "an empty directory moves Git nowhere");
}

/// What:
///  The last `--git-dir` wins,
///  `--bare` is a Git directory of the directory Git runs in,
///  and a value of another option is never read as one of these.
/// Why:
///  Git applies the options in order,
///  and a reading that took the first would name a repository the command never used.
#[test]
fn the_last_naming_option_wins_and_a_value_is_never_an_option() {
    let work: &Path = Path::new("/work");
    assert_eq!(
        text(from_global_options(work, arguments(&["--git-dir=first.git", "--git-dir", "second.git"]).as_slice()).as_slice()),
        vec!["--git-dir", "/work/second.git"]
    );
    assert_eq!(
        text(from_global_options(work, arguments(&["--bare"]).as_slice()).as_slice()),
        vec!["--git-dir", "/work/."],
        "a bare repository is the directory Git runs in, which is the caller's own spelling"
    );
    assert_eq!(
        text(from_global_options(work, arguments(&["-C", "bare", "--bare"]).as_slice()).as_slice()),
        vec!["--git-dir", "/work/bare/."],
        "a bare repository is the directory the chain ended in, as the caller spelled it"
    );
    assert_eq!(
        from_global_options(work, arguments(&["-c", "--git-dir=not-an-option"]).as_slice()),
        Vec::<OsString>::new(),
        "the value of -c is a configuration, not a Git directory"
    );
    assert_eq!(
        from_global_options(work, arguments(&["--git-dir"]).as_slice()),
        Vec::<OsString>::new(),
        "an option with no value is Git's own error to word"
    );
}

/// What:
///  A plain layout is named by nothing,
///  and every other layout is named by its Git directory and its work tree.
/// Why:
///  A plain layout is found from its work tree,
///  and naming it would make every in-process read of a landing start Git instead of answering.
#[test]
fn a_plain_layout_is_named_by_nothing() {
    assert_eq!(of_paths(Path::new("/repo/.git"), Path::new("/repo")), Vec::<OsString>::new());
    let bare: PathBuf = PathBuf::from("/work/cfg.git");
    assert_eq!(
        text(of_paths(bare.as_path(), Path::new("/work/wt")).as_slice()),
        vec!["--git-dir", "/work/cfg.git", "--work-tree", "/work/wt"],
        "a Git directory beside its work tree is named"
    );
    assert_eq!(
        text(of_paths(Path::new("/main/.git/worktrees/one"), Path::new("/work")).as_slice()),
        vec!["--git-dir", "/main/.git/worktrees/one", "--work-tree", "/work"],
        "a linked worktree's own Git directory is named"
    );
    assert_eq!(
        text(of_paths(Path::new("/repo/.git"), Path::new("/other")).as_slice()),
        vec!["--git-dir", "/repo/.git", "--work-tree", "/other"],
        "a Git directory that is not under this work tree is named"
    );
}

/// What:
///  The shadow's own start keeps every option but the ones that name a repository or move Git.
/// Why:
///  The shadow is a repository the wrapper names itself:
///  the caller's Git directory beside it would name the real repository,
///  and a `-C` would move Git after the wrapper placed it.
#[test]
fn the_shadows_start_keeps_every_option_but_the_repository_selection() {
    let kept = |values: &[&str]| -> Vec<String> { return text(without_repository_selection(arguments(values).as_slice()).as_slice()); };
    assert_eq!(kept(&[]), Vec::<String>::new());
    assert_eq!(
        kept(&["-C", "..", "--git-dir", "/abs/cfg.git", "--work-tree=/abs/wt", "--bare", "-c", "user.name=X", "--no-pager"]),
        vec![String::from("-c"), String::from("user.name=X"), String::from("--no-pager")]
    );
    assert_eq!(kept(&["-C", ".."]), Vec::<String>::new(), "the directory of -C goes with it");
    assert_eq!(kept(&["-c", "--git-dir=x"]), vec![String::from("-c"), String::from("--git-dir=x")], "a configuration value is not an option");
}

/// What:
///  The reader of the selected repository reports the directory the `-C` chain ends in,
///  the Git directory the options or `GIT_DIR` name,
///  and which of the two named it.
/// Why:
///  The pass that clears a killed Git's locks searches the repository this answers,
///  and it must not search a bare repository or a directory the command never reaches,
///  which is what the third field tells it apart by.
#[test]
fn the_selected_repository_names_the_git_directory_and_who_named_it() {
    let work: &Path = Path::new("/work");
    let quiet = selected_repository(work, arguments(&["-C", "sub", "-c", "user.name=X"]).as_slice(), &[]);
    assert_eq!(quiet.directory, PathBuf::from("/work/sub"), "the directory Git would discover from");
    assert_eq!(quiet.git_directory, None, "no option and no variable name a Git directory");
    assert!(!quiet.named_by_option);

    let option = selected_repository(work, arguments(&["--git-dir=../cfg.git"]).as_slice(), &[(OsString::from("GIT_DIR"), OsString::from("/elsewhere"))]);
    assert_eq!(option.git_directory, Some(PathBuf::from("/work/../cfg.git")), "the option wins over the variable, read from the chain's directory");
    assert!(option.named_by_option, "the command's own option named it");

    let bare = selected_repository(work, arguments(&["-C", "bare", "--bare"]).as_slice(), &[]);
    assert_eq!(bare.directory, PathBuf::from("/work/bare"));
    assert_eq!(bare.git_directory, Some(PathBuf::from("/work/bare/.")), "a bare repository is the directory Git runs in");
    assert!(bare.named_by_option);

    let variable = selected_repository(work, arguments(&[]).as_slice(), &[(OsString::from("GIT_DIR"), OsString::from(".git"))]);
    assert_eq!(variable.git_directory, Some(PathBuf::from("/work/.git")), "a relative GIT_DIR is read from the directory the command runs in");
    assert!(!variable.named_by_option, "the environment named it, not an option");

    let twice = selected_repository(
        work,
        arguments(&[]).as_slice(),
        &[(OsString::from("GIT_DIR"), OsString::from("/first")), (OsString::from("GIT_DIR"), OsString::from("/second"))],
    );
    assert_eq!(twice.git_directory, Some(PathBuf::from("/second")), "the last GIT_DIR of the environment is the one Git reads");
}

/// What:
///  A located invocation's identity names the repository Git resolved,
///  and names nothing for a layout Git finds by itself or for a bare repository.
/// Why:
///  A rule that starts Git somewhere other than the caller's own directory cannot read the caller's options again:
///  a relative `--git-dir` is resolved after the last `-C`,
///  so reading it again from a discovered directory moves Git twice and names another repository.
#[test]
fn a_location_names_the_repository_git_resolved() {
    let plain: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: PathBuf::from("/repo/.git"),
        git_dir: PathBuf::from("/repo/.git"),
        worktree_root: PathBuf::from("/repo"),
    };
    assert_eq!(of_identity(&plain), Vec::<OsString>::new(), "a plain layout is found from its work tree");
    let beside: WorktreeIdentity = WorktreeIdentity::MainWorktree {
        common_dir: PathBuf::from("/holder/cfg.git"),
        git_dir: PathBuf::from("/holder/cfg.git"),
        worktree_root: PathBuf::from("/holder/wt"),
    };
    assert_eq!(
        text(of_identity(&beside).as_slice()),
        vec!["--git-dir", "/holder/cfg.git", "--work-tree", "/holder/wt"],
        "a Git directory beside its work tree is named"
    );
    let linked: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: PathBuf::from("/main/.git"),
        git_dir: PathBuf::from("/main/.git/worktrees/linked"),
        worktree_root: PathBuf::from("/linked"),
    };
    assert_eq!(
        text(of_identity(&linked).as_slice()),
        vec!["--git-dir", "/main/.git/worktrees/linked", "--work-tree", "/linked"],
        "a linked worktree's own Git directory is named"
    );
    let bare: WorktreeIdentity = WorktreeIdentity::BareRepository {
        common_dir: PathBuf::from("/bare.git"),
        git_dir: PathBuf::from("/bare.git"),
    };
    assert_eq!(of_identity(&bare), Vec::<OsString>::new(), "a bare repository has no work tree to name");
    assert_eq!(of_identity(&WorktreeIdentity::OutsideWorktree), Vec::<OsString>::new());
}
