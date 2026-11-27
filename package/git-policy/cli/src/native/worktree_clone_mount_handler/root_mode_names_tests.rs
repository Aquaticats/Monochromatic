//! What:
//!  Controls of the one decision every start is read as:
//!  ZED's own no-argument start,
//!  the words,
//!  each spelling of the installed path,
//!  every near-miss name,
//!  and the two argument vectors a shell cannot spell.
//! Why:
//!  Only the ZED history event's own name selects a root mode,
//!  and everything else a start can be is the wrapper,
//!  so each case is held here rather than trusted to a reading of the code
//!  (`doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`;
//!  `doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
//!  "What the answer decides,
//! case by case").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(decide(['/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets'])).toEqual({ kind: 'arguments', words: [] });
//! ```

use super::root_mode_names::{
    ROOT_GROUP_WORD,
    RootModeMisuse,
    RootModeStart,
    ZEDLET_START_NAME,
    root_mode_start,
};
use std::ffi::{
    OsStr,
    OsString,
};

/// The one start name,
/// as ZED passes it:
/// the bare file name with no directory.
const ZEDLET: &str = ZEDLET_START_NAME;

/// The installed path root's units start.
const INSTALLED: &str =
    "/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets";

/// The program's own word,
/// which the words begin with.
const PROGRAM_WORD: &str = "monochromatic-git-policy-cli";

/// One start's argument vector.
fn start(words: &[&str]) -> Vec<OsString> {
    return words
        .iter()
        .map(|word| return OsString::from(*word))
        .collect();
}

/// What one start decided,
/// as text,
/// so a case reads as one line.
fn decided(words: &[&str]) -> String {
    return match root_mode_start(start(words).as_slice()) {
        RootModeStart::Wrapper => String::from("wrapper"),
        RootModeStart::Arguments(arguments) => format!(
            "arguments {:?}",
            arguments
                .iter()
                .map(|word| return word.to_string_lossy().to_string())
                .collect::<Vec<String>>()
        ),
        RootModeStart::Refused(RootModeMisuse::NoModeWordAfterTheGroupWord) => {
            String::from("refused the group word alone")
        }
        RootModeStart::Refused(RootModeMisuse::WordUnderTheZedletName(word)) => {
            format!("refused the word `{}`", word.to_string_lossy())
        }
    };
}

/// ZED's own child step is the attaching sweep,
/// and the bare name through any other lookup is the same start.
#[test]
fn the_zedlets_own_name_with_no_argument_is_the_sweep() {
    assert_eq!(decided(&[ZEDLET]), "arguments []");
    assert_eq!(decided(&[INSTALLED]), "arguments []");
    assert_eq!(
        decided(&["/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets"]),
        "arguments []"
    );
}

/// The words select a mode on any file,
/// which is what a unit's `ExecStart`,
/// an adoption and a person use.
#[test]
fn the_words_select_a_mode_on_any_file() {
    assert_eq!(
        decided(&[INSTALLED, PROGRAM_WORD, ROOT_GROUP_WORD, "boot-reconcile"]),
        "arguments [\"boot-reconcile\"]"
    );
    assert_eq!(
        decided(&[ZEDLET, PROGRAM_WORD, ROOT_GROUP_WORD, "boot-reconcile"]),
        "arguments [\"boot-reconcile\"]"
    );
    assert_eq!(
        decided(&[
            "/home/user/.cache/monochromatic-git-policy-cli-bin/git",
            PROGRAM_WORD,
            ROOT_GROUP_WORD,
            "start-probe"
        ]),
        "arguments [\"start-probe\"]"
    );
    assert_eq!(
        decided(&[
            "/usr/bin/git",
            PROGRAM_WORD,
            ROOT_GROUP_WORD,
            "shutdown-unmount"
        ]),
        "arguments [\"shutdown-unmount\"]"
    );
}

/// An adoption's options travel after the mode word untouched,
/// and so do the two modes' own.
#[test]
fn the_words_carry_the_modes_arguments() {
    assert_eq!(
        decided(&[
            INSTALLED,
            PROGRAM_WORD,
            ROOT_GROUP_WORD,
            "adopt-worktree-parent",
            "--checkout",
            "/home/user/Monochromatic",
            "--device",
            "66306",
            "--inode",
            "1234567"
        ]),
        "arguments [\"adopt-worktree-parent\", \"--checkout\", \"/home/user/Monochromatic\", \"--device\", \"66306\", \"--inode\", \"1234567\"]"
    );
    assert_eq!(
        decided(&[
            INSTALLED,
            PROGRAM_WORD,
            ROOT_GROUP_WORD,
            "record-checkout",
            "--checkout",
            "/home/user/Monochromatic"
        ]),
        "arguments [\"record-checkout\", \"--checkout\", \"/home/user/Monochromatic\"]"
    );
    assert_eq!(
        decided(&[
            INSTALLED,
            PROGRAM_WORD,
            ROOT_GROUP_WORD,
            "mount-path-descendant",
            "--dataset",
            "zpcachyos/nosnap/vm-builder-output"
        ]),
        "arguments [\"mount-path-descendant\", \"--dataset\", \"zpcachyos/nosnap/vm-builder-output\"]"
    );
}

/// The group word alone is a refusal,
/// never a sweep,
/// because no unit line and no person can mean the sweep by the words.
#[test]
fn the_group_word_alone_is_a_refusal() {
    assert_eq!(
        decided(&[INSTALLED, PROGRAM_WORD, ROOT_GROUP_WORD]),
        "refused the group word alone"
    );
    assert_eq!(
        decided(&[ZEDLET, PROGRAM_WORD, ROOT_GROUP_WORD]),
        "refused the group word alone"
    );
    assert_eq!(
        decided(&["/usr/bin/git", PROGRAM_WORD, ROOT_GROUP_WORD]),
        "refused the group word alone"
    );
}

/// Under the installed name no word but the two the modes answer reaches the
/// grammar,
/// so a stale line or a guess at that path can never be Git.
#[test]
fn every_other_word_under_the_installed_name_is_refused_before_the_grammar() {
    assert_eq!(
        decided(&[INSTALLED, "boot-reconcile"]),
        "refused the word `boot-reconcile`"
    );
    assert_eq!(
        decided(&[INSTALLED, "start-probe"]),
        "refused the word `start-probe`"
    );
    assert_eq!(decided(&[INSTALLED, "help"]), "refused the word `help`");
    assert_eq!(
        decided(&[INSTALLED, "--help", "boot-reconcile"]),
        "refused the word `boot-reconcile`"
    );
    assert_eq!(
        decided(&[INSTALLED, "--version", "x"]),
        "refused the word `x`"
    );
}

/// The word a refusal names is the word the person got wrong,
/// never the program's own word they typed correctly:
/// a misspelt group word and the wrapper's `service` words are each named,
/// and a start that is only the program's own word still names it.
#[test]
fn the_programs_own_word_is_not_the_word_a_refusal_names() {
    assert_eq!(
        decided(&[INSTALLED, PROGRAM_WORD, "zfs-roots", "boot-reconcile"]),
        "refused the word `zfs-roots`"
    );
    assert_eq!(
        decided(&[INSTALLED, PROGRAM_WORD, "service", "run"]),
        "refused the word `service`"
    );
    assert_eq!(
        decided(&[INSTALLED, PROGRAM_WORD, "boot-reconcile"]),
        "refused the word `boot-reconcile`"
    );
    assert_eq!(
        decided(&[INSTALLED, PROGRAM_WORD]),
        "refused the word `monochromatic-git-policy-cli`"
    );
}

/// The two words the modes answer at that path are answered there,
/// which is the word-free check of the installed bytes.
#[test]
fn help_and_version_under_the_installed_name_are_the_modes_own() {
    assert_eq!(decided(&[INSTALLED, "--help"]), "arguments [\"--help\"]");
    assert_eq!(
        decided(&[INSTALLED, "--version"]),
        "arguments [\"--version\"]"
    );
}

/// Every spelling of the installed path answers the same way,
/// because the comparison reads the last component and not the text.
#[test]
fn every_spelling_of_the_installed_path_reads_the_same_last_component() {
    for spelling in [
        "/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets/",
        "/usr/libexec/monochromatic-git-policy-cli//history_event-mount_git_worktree_clone_datasets",
        "/usr/libexec/monochromatic-git-policy-cli/./history_event-mount_git_worktree_clone_datasets",
        "/usr/libexec/../libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets",
        "history_event-mount_git_worktree_clone_datasets",
    ] {
        assert_eq!(
            decided(&[spelling]),
            "arguments []",
            "{spelling} is the installed name's last component"
        );
    }
}

/// A name that is near the zedlet's but not it is the wrapper,
/// which is what a whole-component comparison buys and a contains or suffix
/// test would not.
#[test]
fn five_near_miss_names_are_the_wrapper() {
    for near_miss in [
        "history_event-mount_git_worktree_clone_dataset",
        "history_event-mount_git_worktree_clone_datasets_",
        "xhistory_event-mount_git_worktree_clone_datasets",
        "History_event-mount_git_worktree_clone_datasets",
        "mount_git_worktree_clone_datasets",
    ] {
        assert_eq!(
            decided(&[near_miss, "boot-reconcile"]),
            "wrapper",
            "{near_miss} selects nothing"
        );
        assert_eq!(
            decided(&[near_miss]),
            "wrapper",
            "{near_miss} selects nothing"
        );
    }
}

/// A stray copy under any other name is the wrapper,
/// and a copy under the zedlet's own name in another folder answers as that
/// name does:
/// the rule reads the name and not the file.
#[test]
fn a_stray_copy_answers_by_its_name_alone() {
    assert_eq!(
        decided(&["/home/user/Monochromatic/.cache/git", "boot-reconcile"]),
        "wrapper"
    );
    assert_eq!(decided(&["/home/user/Monochromatic/.cache/git"]), "wrapper");
    assert_eq!(
        decided(&["/home/user/temp/history_event-mount_git_worktree_clone_datasets"]),
        "arguments []"
    );
    assert_eq!(
        decided(&[
            "/home/user/temp/history_event-mount_git_worktree_clone_datasets",
            "boot-reconcile"
        ]),
        "refused the word `boot-reconcile`"
    );
}

/// A name the rule cannot read as text is not the zedlet's,
/// so the start is the wrapper rather than a guess.
#[test]
fn a_start_name_that_is_not_utf8_is_the_wrapper() {
    use std::os::unix::ffi::OsStringExt;
    let mut vector: Vec<OsString> = vec![OsString::from_vec(vec![b'h', b'i', 0xff, 0xfe])];
    vector.push(OsString::from("boot-reconcile"));
    assert!(matches!(
        root_mode_start(vector.as_slice()),
        RootModeStart::Wrapper
    ));
}

/// An empty vector and a vector holding one empty element are the same start,
/// and both are the wrapper:
/// the last component of `""` is none,
/// which is not the zedlet's name,
/// so the rule decides both without indexing anything.
#[test]
fn an_empty_name_and_an_empty_element_are_both_the_wrapper() {
    assert!(matches!(root_mode_start(&[]), RootModeStart::Wrapper));
    let one_empty: Vec<OsString> = vec![OsString::from("")];
    assert!(matches!(
        root_mode_start(one_empty.as_slice()),
        RootModeStart::Wrapper
    ));
    let empty_and_word: Vec<OsString> = vec![OsString::from(""), OsString::from("boot-reconcile")];
    assert!(matches!(
        root_mode_start(empty_and_word.as_slice()),
        RootModeStart::Wrapper
    ));
}

/// The word a refusal names is the one a person typed,
/// not the one the file answers:
/// `--version x` names `x`.
#[test]
fn the_word_a_refusal_names_is_the_one_that_cannot_be_honoured() {
    let vector: Vec<OsString> = start(&[INSTALLED, "--version", "extra"]);
    let RootModeStart::Refused(misuse) = root_mode_start(vector.as_slice()) else {
        panic!("`--version extra` under the installed name is refused");
    };
    let super::root_mode_names::RootModeMisuse::WordUnderTheZedletName(word) = misuse else {
        panic!("a word after `--version` is the misuse this name has");
    };
    assert_eq!(word, OsStr::new("extra"));
}
