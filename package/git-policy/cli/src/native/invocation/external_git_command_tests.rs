//! Controls for the executable Git runs before an alias:
//!  the search path it walks,
//!  the exec path Git reports,
//!  and the real-Git comparison of each answer.

use super::*;
///  The reading of the Git directory Git gives a `!` alias's shell,
///  which lives beside this module.
use crate::external_git_command_the_git_dir_the_shell_reads::the_git_dir_the_alias_shell_reads;
use crate::test_support::{
    executable,
    fixture,
    git_output,
    remove,
    repository,
};
use std::path::PathBuf;
use std::process::Output;

/// A repository-less fixture directory,
///  where Git resolves commands without needing one.
fn scratch(name: &str) -> PathBuf {
    return fixture(name);
}

/// An environment naming only these variables.
fn environment(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    let mut environment: Vec<(OsString, OsString)> = Vec::with_capacity(pairs.len());
    for (name, value) in pairs {
        environment.push((OsString::from(*name), OsString::from(*value)));
    }
    return environment;
}

/// Global options as the wrapper receives them.
fn globals(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// The context a search runs in:
///  real Git,
///  and no configuration of its own.
fn context() -> GitContext {
    return GitContext {
        real_git: PathBuf::from(crate::test_support::REAL_GIT),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
}

/// An executable file at `path` that prints one marker line.
fn program(path: &Path) -> PathBuf {
    executable(path, b"#!/bin/sh\necho ran\n");
    return path.to_path_buf();
}

/// What:
///  Whether real Git runs `git-<word>` for this environment,
///  read from whether the printed marker appears.
/// Why:
///  The control answers the same question the search answers,
///  so a wrong reading fails against Git and not
///  against another reading of the same rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitOutput(directory, ['word']).stdout.includes('ran')
/// ```
fn git_runs_external(
    directory: &Path,
    pairs: &[(&str, &str)],
    word: &str,
) -> bool {
    return git_runs_external_with_globals(directory, pairs, &[], word);
}

/// What:
///  Whether real Git runs `git-<word>` for this environment and these global options,
///  read from whether the printed marker appears.
/// Why:
///  A global such as `--exec-path=` decides the search as much as the environment does,
///  so a control of that reading compares
///  the same invocation Git receives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitOutput(directory, [...globals, word]).stdout.includes('ran')
/// ```
fn git_runs_external_with_globals(
    directory: &Path,
    pairs: &[(&str, &str)],
    globals: &[&str],
    word: &str,
) -> bool {
    let mut command: std::process::Command =
        std::process::Command::new(crate::test_support::REAL_GIT);
    command
        .current_dir(directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config");
    for (name, value) in pairs {
        command.env(name, value);
    }
    let output: std::process::Output = command
        .args(globals)
        .arg(word)
        .output()
        .expect("native Git fixture command");
    return String::from_utf8_lossy(&output.stdout).contains("ran");
}

/// An executable on a `PATH` entry is what Git runs for that word,
///  and the search answers it too.
#[test]
fn a_program_on_the_path_is_the_command_git_runs() {
    let root: PathBuf = scratch("external-path");
    let directory: PathBuf = root.join("bin");
    std::fs::create_dir(&directory).expect("program directory");
    program(directory.join("git-audit-external").as_path());
    let path: String = format!("{}:/usr/bin:/bin", directory.display());
    let pairs: Vec<(&str, &str)> = vec![("PATH", path.as_str())];
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        &[],
        Some(root.as_path()),
        None,
        OsStr::new("audit-external"),
    );
    assert_eq!(
        answer,
        git_runs_external(root.as_path(), pairs.as_slice(), "audit-external")
    );
    assert!(answer, "the program on PATH was not found");
    remove(root.as_path());
}

/// What:
///  The caller's `--exec-path` decides the search,
///  over a `GIT_EXEC_PATH` the environment also names.
/// Why:
///  `--exec-path=<dir>` sets Git's exec path
///  (`git.c:178` to `git.c:181`),
///  and `git_exec_path` answers with that setting before it reads the environment
///  (`exec-cmd.c:313` to `exec-cmd.c:323`),
///  so a reading that read the environment first would find a program Git never runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// dashedExternalSelected({ environment: { GIT_EXEC_PATH: old }, globals: [`--exec-path=${new}`], word: 'audit-external' })
/// ```
#[test]
fn the_exec_path_option_decides_the_search_over_the_environment() {
    let root: PathBuf = scratch("external-exec-path-option");
    let old_exec: PathBuf = root.join("old-exec");
    let new_exec: PathBuf = root.join("new-exec");
    std::fs::create_dir(&old_exec).expect("old exec directory");
    std::fs::create_dir(&new_exec).expect("new exec directory");
    program(old_exec.join("git-audit-external").as_path());
    let old_text: String = old_exec.display().to_string();
    let new_text: String = new_exec.display().to_string();
    let pairs: Vec<(&str, &str)> = vec![
        ("GIT_EXEC_PATH", old_text.as_str()),
        ("PATH", "/usr/bin:/bin"),
    ];
    let option: String = format!("--exec-path={new_text}");
    let given: [&str; 1] = [option.as_str()];
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        globals(given.as_slice()).as_slice(),
        Some(root.as_path()),
        None,
        OsStr::new("audit-external"),
    );
    assert_eq!(
        answer,
        git_runs_external_with_globals(
            root.as_path(),
            pairs.as_slice(),
            given.as_slice(),
            "audit-external"
        )
    );
    assert!(
        !answer,
        "a program only the environment's exec path names was taken as the command"
    );
    remove(root.as_path());
}

/// A word nothing on the search path names is no external command,
///  so Git reaches the alias.
#[test]
fn a_word_no_program_names_is_not_an_external_command() {
    let root: PathBuf = scratch("external-absent");
    let pairs: Vec<(&str, &str)> = vec![("PATH", "/usr/bin:/bin")];
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        &[],
        Some(root.as_path()),
        None,
        OsStr::new("audit-absent"),
    );
    assert_eq!(
        answer,
        git_runs_external(root.as_path(), pairs.as_slice(), "audit-absent")
    );
    assert!(!answer, "a program nothing provides was found");
    remove(root.as_path());
}

/// A `GIT_EXEC_PATH` entry is searched before `PATH`,
///  so a program there is what Git runs,
///  as the audit's fixture found.
#[test]
fn a_program_in_the_exec_path_is_the_command_git_runs() {
    let root: PathBuf = scratch("external-exec-path");
    let exec: PathBuf = root.join("exec");
    std::fs::create_dir(&exec).expect("exec path directory");
    program(exec.join("git-audit-external").as_path());
    let exec_path: String = exec.display().to_string();
    let pairs: Vec<(&str, &str)> = vec![
        ("GIT_EXEC_PATH", exec_path.as_str()),
        ("PATH", "/usr/bin:/bin"),
    ];
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        &[],
        Some(root.as_path()),
        None,
        OsStr::new("audit-external"),
    );
    assert_eq!(
        answer,
        git_runs_external(root.as_path(), pairs.as_slice(), "audit-external")
    );
    assert!(answer, "the program in the exec path was not found");
    remove(root.as_path());
}

/// What:
///  A relative `GIT_EXEC_PATH` is searched from the directory Git runs in,
///  so a program there is what Git runs.
/// Why:
///  Git makes a relative exec path absolute against its own directory before it adds it to `PATH`
///  (`exec-cmd.c:325` to `exec-cmd.c:340`),
///  so the search has an answer;
///  a reading that called the relative path uncertain left the alias to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// dashedExternalSelected({ environment: { GIT_EXEC_PATH: 'exec' }, currentDirectory: root, word: 'audit-external' })
/// ```
#[test]
fn a_relative_exec_path_is_searched_where_git_searches_it() {
    let root: PathBuf = scratch("external-relative-exec-path");
    let exec: PathBuf = root.join("exec");
    std::fs::create_dir(&exec).expect("exec path directory");
    program(exec.join("git-audit-external").as_path());
    let pairs: Vec<(&str, &str)> = vec![("GIT_EXEC_PATH", "exec"), ("PATH", "/usr/bin:/bin")];
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        &[],
        Some(root.as_path()),
        None,
        OsStr::new("audit-external"),
    );
    assert_eq!(
        answer,
        git_runs_external(root.as_path(), pairs.as_slice(), "audit-external")
    );
    assert!(
        answer,
        "the program in the relative exec path was not found"
    );
    remove(root.as_path());
}

/// What:
///  A `PATH` entry without a leading separator is read from the directory Git runs in,
///  which the caller's `-C` options move.
/// Why:
///  `-C` changes Git's directory before it looks a command up
///  (`git.c:316` to `git.c:328`,
///  then `git.c:988`),
///  and `locate_in_PATH` joins each entry to the file as it stands
///  (`run-command.c:203` to `:218`),
///  so a program only the caller's own directory holds is one Git never runs,
///  and
///  a reading that resolved the entry against the caller's directory found it and left the alias to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// dashedExternalSelected({ environment: { PATH: '../programs/bin:/usr/bin:/bin' }, globals: ['-C', '../B'], currentDirectory: caller })
/// ```
#[test]
fn a_relative_path_entry_is_read_from_the_directory_git_changed_into() {
    let root: PathBuf = scratch("external-relative-path-entry");
    let caller: PathBuf = root.join("A");
    let moved: PathBuf = root.join("B");
    let programs: PathBuf = root.join("programs/bin");
    std::fs::create_dir_all(&caller).expect("caller directory");
    std::fs::create_dir(&moved).expect("moved directory");
    std::fs::create_dir_all(&programs).expect("program directory");
    program(programs.join("git-audit-relative").as_path());
    let pairs: Vec<(&str, &str)> = vec![("PATH", "../programs/bin:/usr/bin:/bin")];
    let given: Vec<OsString> = globals(&["-C", "../B"]);
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        given.as_slice(),
        Some(caller.as_path()),
        None,
        OsStr::new("audit-relative"),
    );
    assert_eq!(
        answer,
        git_runs_external_with_globals(
            caller.as_path(),
            pairs.as_slice(),
            &["-C", "../B"],
            "audit-relative"
        )
    );
    assert!(answer, "the program the moved entry names was not found");
    // A program only the directory the caller stood in holds is one Git never runs after `-C`.
    let caller_programs: PathBuf = caller.join("bin");
    std::fs::create_dir(caller_programs.as_path()).expect("caller program directory");
    program(caller_programs.join("git-audit-here").as_path());
    let here: Vec<(&str, &str)> = vec![("PATH", "bin:/usr/bin:/bin")];
    let answer_here: bool = dashed_external_selected(
        &context(),
        environment(here.as_slice()).as_slice(),
        given.as_slice(),
        Some(caller.as_path()),
        None,
        OsStr::new("audit-here"),
    );
    assert_eq!(
        answer_here,
        git_runs_external_with_globals(
            caller.as_path(),
            here.as_slice(),
            &["-C", "../B"],
            "audit-here"
        )
    );
    assert!(
        !answer_here,
        "a program only the caller's own directory holds was taken as the command"
    );
    remove(root.as_path());
}

/// A file without the owner execute bit is skipped,
///  as Git skips it,
///  and the exec path Git reports is the one its own search uses.
#[test]
fn a_file_without_the_owner_execute_bit_is_no_command() {
    let root: PathBuf = scratch("external-not-executable");
    let directory: PathBuf = root.join("bin");
    std::fs::create_dir(&directory).expect("program directory");
    let plain: PathBuf = directory.join("git-audit-plain");
    std::fs::write(&plain, b"#!/bin/sh\necho ran\n").expect("plain file");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o644))
            .expect("plain mode");
    }
    let path: String = format!("{}:/usr/bin:/bin", directory.display());
    let pairs: Vec<(&str, &str)> = vec![("PATH", path.as_str())];
    let answer: bool = dashed_external_selected(
        &context(),
        environment(pairs.as_slice()).as_slice(),
        &[],
        Some(root.as_path()),
        None,
        OsStr::new("audit-plain"),
    );
    assert_eq!(
        answer,
        git_runs_external(root.as_path(), pairs.as_slice(), "audit-plain")
    );
    assert!(!answer, "a file Git cannot execute was taken as a command");
    remove(root.as_path());
}

/// The exec path Git reports ends in one newline that the reading removes,
///  and a Windows answer ends in a carriage return before it.
#[test]
fn the_reported_exec_path_loses_only_its_record_terminator() {
    assert_eq!(
        without_record_terminator(b"/usr/lib/git-core\n"),
        OsString::from("/usr/lib/git-core")
    );
    assert_eq!(
        without_record_terminator(b"/usr/lib/git-core\r\n"),
        OsString::from("/usr/lib/git-core")
    );
    assert_eq!(
        without_record_terminator(b"/usr/lib/git-core"),
        OsString::from("/usr/lib/git-core")
    );
    // A path whose last byte is a carriage return keeps it when no newline ended the answer.
    assert_eq!(without_record_terminator(b"/x\r"), OsString::from("/x\r"));
    assert_eq!(without_record_terminator(b""), OsString::new());
}

/// Git reports an exec path,
///  and the answer is the directory it keeps its commands in.
#[test]
fn git_reports_the_exec_path_the_search_uses() {
    let root: PathBuf = scratch("external-exec-path-query");
    let reported: Option<OsString> = printed_exec_path(&context(), &[], root.as_path());
    let expected: String =
        String::from_utf8_lossy(&git_output(root.as_path(), &["--exec-path"]).stdout)
            .trim_end_matches('\n')
            .to_owned();
    assert_eq!(reported, Some(OsString::from(expected)));
    // `match` picks by variant, so the reading names the present case instead of shadowing the option.
    match reported {
        Some(path) => assert!(!path.is_empty(), "Git reported an empty exec path"),
        None => panic!("Git reported no exec path"),
    }
    remove(root.as_path());
}

/// What:
///  What real Git answers about one place a command can be typed:
///  the working directory the shell of a `!` alias printed,
///  and the `rev-parse` questions the wrapper's reading decides from.
///  `(bool, String)` is whether Git succeeded,
/// and what it printed.
/// Why:
///  `alias_run_directory` decides the directory from those answers,
///  so a control holds the wrapper's answer against Git's own answer in the
///  same place,
///  never against a second reading of Git's rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AnswersInOnePlace = {
///   shellDirectory: string;
///   topOfTheWorkingTree: [boolean, string];
///   gitDirectory: [boolean, string];
///   isBareRepository: [boolean, string];
///   theQuestionTheReaderAsks: [boolean, string];
/// };
/// ```
struct AnswersInOnePlace {
    /// The directory the shell of the alias printed,
    ///  which is where Git put that shell.
    shell_directory: PathBuf,
    /// `rev-parse --show-toplevel`.
    top_of_the_working_tree: (bool, String),
    /// `rev-parse --git-dir`.
    git_directory: (bool, String),
    /// `rev-parse --is-bare-repository`.
    is_bare_repository: (bool, String),
    /// `rev-parse --path-format=absolute --show-toplevel --show-prefix`,
    ///  which is the one question `alias_run_directory` asks Git.
    the_question_the_reader_asks: (bool, String),
}

/// What:
///  The `-c` value whose shell prints the directory Git started it in.
/// Why:
///  Git runs an alias value through a shell only when the value holds one of the
///  bytes `prepare_shell_cmd` lists (`run-command.c:293` to `run-command.c:305`),
///  and a space is one of them,
///  so a value of `!pwd` alone would be exec'd as a program instead of run by a
///  shell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ALIAS_PRINTS_ITS_DIRECTORY = 'alias.where=!pwd -P';
/// ```
const ALIAS_PRINTS_ITS_DIRECTORY: &str = "alias.where=!pwd -P";

/// What:
///  One `rev-parse` question's answer from `directory`:
///  whether Git succeeded,
///  and what it printed without the line breaks each answer ends with.
/// Why:
///  A reader that looked at only one of the two could take a failure's empty
///  answer for an empty path,
///  which is the difference between a bare repository and the top of a worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theAnswer = (directory: string, question: string[]): [boolean, string] => {
///   const output = gitOutput(directory, question);
///   return [output.status.success, output.stdout.trimEnd()];
/// };
/// ```
fn the_answer_one_question_got(
    directory: &Path,
    question: &[&str],
) -> (bool, String) {
    let output: Output = git_output(directory, question);
    // `.into_owned()` copies the lossy text so the answer outlives the child's buffer.
    let printed: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    // `.to_owned()` copies the trimmed text into the value this function returns.
    return (
        output.status.success(),
        printed.trim_end_matches('\n').to_owned(),
    );
}

/// What:
///  Git's own answers about one place a command can be typed.
/// Why:
///  One helper keeps every control of these shapes asking Git the same questions
///  in the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answersIn = (directory: string): AnswersInOnePlace => ({ ... });
/// ```
fn answers_in(directory: &Path) -> AnswersInOnePlace {
    let alias: Output =
        crate::test_support::git(directory, &["-c", ALIAS_PRINTS_ITS_DIRECTORY, "where"]);
    // `.trim_end_matches('\n')` drops the one newline the shell's answer ends with.
    let printed: String = String::from_utf8_lossy(alias.stdout.as_slice()).into_owned();
    let shell_directory: PathBuf = PathBuf::from(printed.trim_end_matches('\n'));
    return AnswersInOnePlace {
        shell_directory,
        top_of_the_working_tree: the_answer_one_question_got(
            directory,
            &["rev-parse", "--show-toplevel"],
        ),
        git_directory: the_answer_one_question_got(directory, &["rev-parse", "--git-dir"]),
        is_bare_repository: the_answer_one_question_got(
            directory,
            &["rev-parse", "--is-bare-repository"],
        ),
        the_question_the_reader_asks: the_answer_one_question_got(
            directory,
            &[
                "rev-parse",
                "--path-format=absolute",
                "--show-toplevel",
                "--show-prefix",
            ],
        ),
    };
}

/// What:
///  The directory Git names as the repository's Git directory at `place`,
///  read from Git's own answer there.
///  `&Path` borrows the place the command would be typed in.
/// Why:
///  `--git-dir` answers `.` where the command stands in the Git directory and a
///  full path where it stands below it,
///  so a control compares the directory each answer names rather than the bytes
///  Git chose,
///  and the answer still has to name the repository the shape belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theGitDirectoryNamedAt = (place: string): string => realpath(join(place, answersIn(place).gitDirectory[1]));
/// ```
fn the_git_directory_named_at(place: &Path) -> PathBuf {
    let answers: AnswersInOnePlace = answers_in(place);
    assert!(
        answers.git_directory.0,
        "Git names the Git directory at {}",
        place.display()
    );
    // `.join(...)` reads a relative answer from the directory the command was typed in, as Git does.
    let named: PathBuf = place.join(answers.git_directory.1.as_str());
    // `.expect(...)` unwraps the canonical path or fails with this sentence.
    return std::fs::canonicalize(named.as_path()).expect("the Git directory Git named reads");
}

/// What:
///  Seven places a command can be typed,
/// and the fixture they are made in.
/// Why:
///  The two shapes issue 6625 named sit beside the shapes a control already
///  stood in,
///  so one fixture answers whichever of them a reading gets wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shapes = shapesAPlaceCanBeTyped('name'); // { root, repository, belowTheTop, gitDirectory, ... }
/// ```
struct ShapesAPlaceCanBeTyped {
    /// The fixture's own directory,
    /// which is outside every repository.
    root: PathBuf,
    /// The repository Git made.
    repository: PathBuf,
    /// A subdirectory below the top of that repository.
    below_the_top: PathBuf,
    /// The repository's Git directory.
    git_directory: PathBuf,
    /// A folder inside that Git directory.
    inside_the_git_directory: PathBuf,
    /// A bare repository.
    bare: PathBuf,
    /// A folder inside the bare repository.
    inside_the_bare: PathBuf,
}

/// What:
///  The fixture the record above describes.
/// Why:
///  Every control of these shapes starts from the same seven places,
///  and a folder inside a Git directory is a place Git answers about
///  differently from the directory itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shapes = shapesAPlaceCanBeTyped('external-alias-directory');
/// ```
fn shapes_a_place_can_be_typed(name: &str) -> ShapesAPlaceCanBeTyped {
    let root: PathBuf = scratch(name);
    let repository: PathBuf = repository(root.as_path(), "repository");
    let below_the_top: PathBuf = repository.join("sub");
    std::fs::create_dir_all(below_the_top.as_path()).expect("the subdirectory below the top");
    let git_directory: PathBuf = repository.join(".git");
    let inside_the_git_directory: PathBuf = git_directory.join("refs");
    let bare: PathBuf = root.join("bare.git");
    crate::test_support::git(
        root.as_path(),
        &[
            "init",
            "--quiet",
            "--bare",
            bare.to_str().expect("the fixture's path is UTF-8"),
        ],
    );
    let inside_the_bare: PathBuf = bare.join("refs");
    return ShapesAPlaceCanBeTyped {
        root,
        repository,
        below_the_top,
        git_directory,
        inside_the_git_directory,
        bare,
        inside_the_bare,
    };
}

/// What:
///  The shell of a `!` alias runs where `alias_run_directory` answers,
///  in every shape a command can be typed in:
///  at the top of a worktree,
///  below that top,
///  inside a repository's Git directory,
///  inside a folder of that directory,
///  in a bare repository,
///  and inside a folder of that one.
/// Why:
///  Issue 6625:
///  the reading names a top only where Git's discovery walked up to a worktree,
///  and Git names no top at all in the last four shapes,
///  so the reading answers the directory the command's options left it in.
///  That sentence was never run in exactly the Git directory and the bare
///  repository,
///  and the comparison here is Git's own printed directory rather than another
///  reading of Git's rules.
///  Measured on 2026-11-27 with Git 2.56.1.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const place of theSevenPlaces)
///   expect(aliasRunDirectory(context, [], place)).toBe(answersIn(place).shellDirectory);
/// ```
#[test]
fn a_shell_alias_runs_where_the_reader_answers_in_a_git_directory_and_a_bare_repository() {
    let shapes: ShapesAPlaceCanBeTyped =
        shapes_a_place_can_be_typed("external-alias-directory-outside-a-worktree");
    let places: Vec<PathBuf> = vec![
        shapes.repository.clone(),
        shapes.below_the_top.clone(),
        shapes.git_directory.clone(),
        shapes.inside_the_git_directory.clone(),
        shapes.bare.clone(),
        shapes.inside_the_bare.clone(),
    ];
    for place in &places {
        let answers: AnswersInOnePlace = answers_in(place);
        let read: Option<PathBuf> = alias_run_directory(&context(), &[], Some(place.as_path()));
        assert_eq!(
            read.as_deref(),
            Some(answers.shell_directory.as_path()),
            "the reading names the directory Git starts an alias's shell in at {}",
            place.display()
        );
    }
    remove(shapes.root.as_path());
}

/// What:
///  What Git answers in each of those places:
///  a worktree names its top,
///  and neither a Git directory nor a bare repository names one.
///  A bare repository answers `true` to `--is-bare-repository` where a worktree
///  answers `false`,
///  which is the other difference these two shapes carry.
/// Why:
///  The reading's failure branch is the leaf the two shapes of issue 6625 take,
///  so the answers it fails on are pinned once,
/// beside the control that reads
///  them:
///  the failing question prints `fatal: this operation must be run in a work
///  tree` and the reading keeps only the status and the printed paths.
///  Measured on 2026-11-27 with Git 2.56.1.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(answersIn(gitDirectory).topOfTheWorkingTree).toEqual([false, '']);
/// expect(answersIn(bare).isBareRepository).toEqual([true, 'true']);
/// ```
#[test]
fn git_names_no_top_of_a_working_tree_in_a_git_directory_or_a_bare_repository() {
    let shapes: ShapesAPlaceCanBeTyped =
        shapes_a_place_can_be_typed("external-alias-directory-where-git-names-a-top");
    let top: String = shapes.repository.display().to_string();
    for place in [&shapes.repository, &shapes.below_the_top] {
        let answers: AnswersInOnePlace = answers_in(place);
        assert_eq!(
            answers.top_of_the_working_tree,
            (true, top.clone()),
            "the top of the worktree is named at {}",
            place.display()
        );
        assert!(
            answers.the_question_the_reader_asks.0,
            "both questions answer at {}",
            place.display()
        );
        assert_eq!(
            answers.is_bare_repository,
            (true, String::from("false")),
            "a worktree is no bare repository at {}",
            place.display()
        );
    }
    for place in [
        &shapes.git_directory,
        &shapes.inside_the_git_directory,
        &shapes.bare,
        &shapes.inside_the_bare,
    ] {
        let answers: AnswersInOnePlace = answers_in(place);
        assert_eq!(
            answers.top_of_the_working_tree,
            (false, String::new()),
            "Git names no top at {}",
            place.display()
        );
        assert!(
            !answers.the_question_the_reader_asks.0,
            "the question the reading asks fails at {}",
            place.display()
        );
    }
    for place in [&shapes.git_directory, &shapes.inside_the_git_directory] {
        assert_eq!(
            answers_in(place).is_bare_repository,
            (true, String::from("false")),
            "the Git directory is no bare repository at {}",
            place.display()
        );
    }
    for place in [&shapes.bare, &shapes.inside_the_bare] {
        assert_eq!(
            answers_in(place).is_bare_repository,
            (true, String::from("true")),
            "the bare repository answers so at {}",
            place.display()
        );
    }
    // `.expect(...)` unwraps the canonical path or fails with this sentence.
    let the_repositorys_git_directory: PathBuf =
        std::fs::canonicalize(shapes.repository.join(".git"))
            .expect("the repository's Git directory reads");
    for place in [
        &shapes.repository,
        &shapes.below_the_top,
        &shapes.git_directory,
        &shapes.inside_the_git_directory,
    ] {
        assert_eq!(
            the_git_directory_named_at(place),
            the_repositorys_git_directory,
            "Git names the repository's own Git directory at {}",
            place.display()
        );
    }
    // `.expect(...)` unwraps the canonical path or fails with this sentence.
    let the_bare_directory: PathBuf =
        std::fs::canonicalize(shapes.bare.as_path()).expect("the bare repository reads");
    for place in [&shapes.bare, &shapes.inside_the_bare] {
        assert_eq!(
            the_git_directory_named_at(place),
            the_bare_directory,
            "Git names the bare repository itself at {}",
            place.display()
        );
    }
    remove(shapes.root.as_path());
}

/// What:
///  The `-c` value whose shell prints the `GIT_DIR` Git gives it.
/// Why:
///  The reading of issue 6641 answers what Git exports to the shell of a `!` alias,
///  and only Git's own answer can hold it:
///  the value prints the variable the shell really received,
///  and its space keeps Git running it through a shell
///  (`run-command.c:293` to `run-command.c:305`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ALIAS_PRINTS_ITS_GIT_DIR = 'alias.printgitdir=!printf %s "$GIT_DIR"';
/// ```
const ALIAS_PRINTS_ITS_GIT_DIR: &str = "alias.printgitdir=!printf %s \"$GIT_DIR\"";

/// What:
///  The Git directory Git gives the shell of a `!` alias typed at `place`,
///  as the shell itself printed it,
///  and nothing when Git gives it none.
///  `&Path` borrows the place the command would be typed in.
/// Why:
///  The control compares the reading with the variable a real shell received,
///  never with another reading of Git's rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theGitDirTheShellWasGiven = (place) => git(place, ['-c', ALIAS_PRINTS_ITS_GIT_DIR, 'printgitdir']).stdout.replace(/\n$/, '');
/// ```
fn the_git_dir_the_shell_was_given(place: &Path) -> String {
    let output: Output =
        crate::test_support::git(place, &["-c", ALIAS_PRINTS_ITS_GIT_DIR, "printgitdir"]);
    assert!(
        output.status.success(),
        "the fixture's alias prints at {}: {}",
        place.display(),
        String::from_utf8_lossy(output.stderr.as_slice())
    );
    let printed: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    return printed.trim_end_matches('\n').to_owned();
}

/// What:
///  The reading of the Git directory a `!` alias's shell is given answers what Git gives it,
///  in every shape a command can be typed in:
///  at the top of a worktree,
///  below that top,
///  and inside a Git directory,
/// a folder of it and a bare repository,
///  where Git names no top at all.
///  Why:
///  Issue 6641's second class:
/// from a linked worktree Git exports `GIT_DIR`,
///  so the nested `git -C <elsewhere>` writes the caller's index over
///  another directory's files,
///  while in a plain main checkout it exports nothing and the nested Git finds the repository itself.
///  A reading that exported the directory in the main checkout too would judge a nested `-C` into a subdirectory
///  with the wrong worktree,
///  and one that exported it nowhere would judge the linked worktree's command as its files alone.
///  The seven shapes stand beside one another so both mistakes have a row.
///  Measured with Git 2.56.1 on this machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const place of theSevenPlaces)
///   expect(gitDirTheAliasShellReads(context(), [], Some(place), false) !== undefined).toBe(theGitDirTheShellWasGiven(place) !== '');
/// ```
/// What:
///  A main checkout whose own configuration names the work tree it uses,
///  which Git 2.56.1 takes the explicit path for (`setup.c` `repo_discover_implicit_gitdir`).
/// Why:
///  The alpha.52 read's finding M1:
///  with `core.worktree` set,
///  Git stores the Git directory it found and exports it to a `!` alias's shell
///  in a plain main checkout too,
///  from a subdirectory as an absolute path that a nested `-C` cannot move,
///  so the nested Git writes this checkout's index and `HEAD` over another directory's files.
///  The reading decided by comparing `<top>/.git` with `--absolute-git-dir`,
///  and with `core.worktree` set to the checkout itself those two are the same directory,
///  so the shape stood outside the seven places the control held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = aMainCheckoutNamingItsOwnWorkTree('core-worktree'); // git config core.worktree <repository>
/// ```
fn a_main_checkout_naming_its_own_work_tree(name: &str) -> PathBuf {
    let root: PathBuf = scratch(name);
    let repository: PathBuf = repository(root.as_path(), "repository");
    std::fs::create_dir_all(repository.join("sub").as_path())
        .expect("the subdirectory below the top");
    let work_tree: &str = repository.to_str().expect("the fixture's path is UTF-8");
    // The spelling a person uses, through real Git, so the file is the one Git writes.
    crate::test_support::git(
        repository.as_path(),
        &["config", "core.worktree", work_tree],
    );
    return root;
}

/// What:
///  The reading of the Git directory a `!` alias's shell is given agrees with what Git gives it
///  at one place,
///  both on whether it gives one at all and on which directory it names.
/// Why:
///  One comparison serves every shape the control holds,
///  so a shape added to either fixture is compared the same way,
///  and a reading that guessed either way fails at the place it guessed wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theGitDirTheAliasShellReads(context(), [], place, false)).toEqual(theGitDirTheShellWasGiven(place));
/// ```
fn the_reading_agrees_with_git_at(place: &Path) {
    let git_gives_it: String = the_git_dir_the_shell_was_given(place);
    let read: Option<OsString> =
        the_git_dir_the_alias_shell_reads(&context(), &globals(&[]), Some(place), false);
    assert_eq!(
        read.is_some(),
        !git_gives_it.is_empty(),
        "the reading says whether Git gives an alias's shell a Git directory at {}",
        place.display()
    );
    let Some(read_by_the_guard) = read else {
        return;
    };
    // Both answers are read from the place the command was typed in, as Git reads one.
    let read_path: PathBuf = std::fs::canonicalize(place.join(read_by_the_guard.as_os_str()))
        .expect("the Git directory the reading names reads");
    let git_path: PathBuf = std::fs::canonicalize(place.join(git_gives_it.as_str()))
        .expect("the Git directory Git named reads");
    assert_eq!(
        read_path,
        git_path,
        "the reading names the Git directory Git gives an alias's shell at {}",
        place.display()
    );
}

#[test]
fn the_git_dir_the_alias_shell_reads_is_the_one_git_gives_it() {
    let shapes: ShapesAPlaceCanBeTyped = shapes_a_place_can_be_typed("external-alias-git-dir");
    let naming_its_own_work_tree: PathBuf =
        a_main_checkout_naming_its_own_work_tree("external-alias-git-dir-with-core-worktree");
    let checkout: PathBuf = naming_its_own_work_tree.join("repository");
    let places: Vec<PathBuf> = vec![
        shapes.repository.clone(),
        shapes.below_the_top.clone(),
        shapes.git_directory.clone(),
        shapes.inside_the_git_directory.clone(),
        shapes.bare.clone(),
        shapes.inside_the_bare.clone(),
        shapes.root.clone(),
        // The eighth and ninth places: a main checkout whose configuration names its own work tree,
        // at its top and in a subdirectory, where Git exports an absolute Git directory.
        checkout.clone(),
        checkout.join("sub"),
    ];
    for place in &places {
        the_reading_agrees_with_git_at(place.as_path());
    }
    remove(shapes.root.as_path());
    remove(naming_its_own_work_tree.as_path());
}
