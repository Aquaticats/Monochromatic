//! What:
//!  The repository layout of one invocation:
//!  the one `rev-parse` report that answers every pathname at once,
//!  and the questions asked one pathname at a time when that report cannot be split.
//! Why:
//!  A report whose answer count differs from the layout's pathname count holds a pathname carrying the
//!  line feed that separates two answers,
//!  so no answer's place is known and each pathname is asked for on its own.
//!  The answers are the bytes Git printed,
//!  because a pathname is a byte string and not text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answers = split(await ask(['rev-parse', ...LAYOUT_ARGUMENTS]));
//! const report = answers.length === LAYOUT_ANSWERS ? reportOf(answers) : await askedFor();
//! ```

/// The two starts a question is asked with:
///  one whose failure is an answer the caller reads,
///  and one whose failure is an error.
use crate::capture_invocation_git_questions::required;
/// The answers of one report,
///  and the pathname each of them names.
use crate::git_report_answers::{
    absolute_reported,
    answers_of,
    object_format_of,
};
/// The in-process repository layout,
///  and the report shape it answers with.
use crate::repo_read_adapter::InvocationReport;
/// The transaction Git wrapper:
///  the context a question is asked in,
///  the failure its answer is reported as,
///  and the output it prints.
use crate::transaction_git::{
    GitContext,
    GitFailure,
    GitOutput,
};
/// `PathBuf` is an owned filesystem path.
use std::path::{
    Path,
    PathBuf,
};
/// How many answers the one layout query prints:
/// six pathnames and the object format.
pub(crate) const LAYOUT_ANSWERS: usize = 6 + 1;

/// The `rev-parse` options of the one layout query,
/// in the order its answers are read.
const LAYOUT_ARGUMENTS: &[&str] = &[
    "--path-format=absolute",
    "--show-toplevel",
    "--absolute-git-dir",
    "--git-common-dir",
    "--git-path",
    "index",
    "--git-path",
    "objects",
    "--git-path",
    "cli-git-transactions",
    "--show-object-format",
];

/// How many pathname answers the layout has.
const LAYOUT_PATHS: usize = 6;

/// What:
///  The pathnames of one layout asked for one at a time,
///  in the order the shared report prints them.
/// Why:
///   One question per pathname,
/// so a line feed inside a name cannot end any other answer;
///  `landing_location.rs` reads its two answers the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GIT_DIR_QUESTION = ['rev-parse', '--path-format=absolute', '--absolute-git-dir'];
/// ```
const PATH_QUESTIONS: &[&[&str]] = &[
    &["rev-parse", "--path-format=absolute", "--show-toplevel"],
    &["rev-parse", "--path-format=absolute", "--absolute-git-dir"],
    &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    &["rev-parse", "--path-format=absolute", "--git-path", "index"],
    &[
        "rev-parse",
        "--path-format=absolute",
        "--git-path",
        "objects",
    ],
    &[
        "rev-parse",
        "--path-format=absolute",
        "--git-path",
        "cli-git-transactions",
    ],
];

/// The question that prints the object format,
///  which is text and not a pathname.
const OBJECT_FORMAT_QUESTION: &[&str] = &["rev-parse", "--show-object-format"];

/// What:
///  The layout the seven answers of one report name.
/// Why:
///  The answers are in the order `LAYOUT_ARGUMENTS` asks for them,
///  and each path is the bytes Git printed for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ topLevel: absolute(cwd, answers[0]), gitDir: absolute(cwd, answers[1]), ... })
/// ```
fn report_of_answers(
    cwd: &Path,
    answers: &[&[u8]; LAYOUT_ANSWERS],
) -> Result<InvocationReport, GitFailure> {
    return Ok(InvocationReport {
        top_level: absolute_reported(cwd, answers[0])?,
        git_dir: absolute_reported(cwd, answers[1])?,
        common_dir: absolute_reported(cwd, answers[2])?,
        index_path: absolute_reported(cwd, answers[3])?,
        object_directory: absolute_reported(cwd, answers[4])?,
        registry_root: absolute_reported(cwd, answers[5])?,
        object_format: object_format_of(answers[6])?,
    });
}

/// What:
///  Every pathname of the layout asked for on its own,
///  resolved against the invocation directory.
/// Why:
///  A report that cannot be split is the only caller of this,
/// which needs one question per pathname anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// PATH_QUESTIONS.map(async (question) => absolute(cwd, await ask(question)))
/// ```
fn absolute_answers(
    context: &GitContext,
    cwd: &Path,
) -> Result<[PathBuf; LAYOUT_PATHS], GitFailure> {
    let mut answers: Vec<PathBuf> = Vec::with_capacity(LAYOUT_PATHS);
    for &question in PATH_QUESTIONS {
        let output: GitOutput = required(context, cwd, question)?;
        answers.push(absolute_reported(cwd, output.stdout.as_slice())?);
    }
    return match <[PathBuf; LAYOUT_PATHS]>::try_from(answers) {
        Ok(exactly) => Ok(exactly),
        Err(_) => Err(GitFailure(String::from(
            "Git returned incomplete repository layout.",
        ))),
    };
}

/// What:
///  The layout of one repository,
///  with every pathname asked for on its own.
/// Why:
///  One report cannot be split once a pathname holds a line feed,
///  and a pathname that is not UTF-8 cannot be read out of text at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [topLevel, gitDir, commonDir, indexPath, objectDirectory, registryRoot] = await absoluteAnswers();
/// ```
fn layout_report_asked_for(
    context: &GitContext,
    cwd: &Path,
) -> Result<InvocationReport, GitFailure> {
    let [
        top_level,
        git_dir,
        common_dir,
        index_path,
        object_directory,
        registry_root,
    ] = absolute_answers(context, cwd)?;
    let output: GitOutput = required(context, cwd, OBJECT_FORMAT_QUESTION)?;
    return Ok(InvocationReport {
        top_level,
        git_dir,
        common_dir,
        index_path,
        object_directory,
        registry_root,
        object_format: object_format_of(output.stdout.as_slice())?,
    });
}

/// What:
///  The layout Git reports for the invocation directory,
///  read as the bytes Git printed.
/// Why:
///  One query answers a plain layout;
///  a report with more answers than the layout has pathnames holds a pathname that carries the line feed separating two answers,
///  so no answer's place is known and each pathname is asked for on its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = split(await ask(['rev-parse', ...LAYOUT_ARGUMENTS])); answers.length === 7 ? reportOf(answers) : askedFor()
/// ```
pub(crate) fn layout_report_from_git(
    context: &GitContext,
    cwd: &Path,
) -> Result<InvocationReport, GitFailure> {
    let mut arguments: Vec<&str> = vec!["rev-parse"];
    arguments.extend_from_slice(LAYOUT_ARGUMENTS);
    let output: GitOutput = required(context, cwd, arguments.as_slice())?;
    let answers: Vec<&[u8]> = answers_of(output.stdout.as_slice());
    return match <[&[u8]; LAYOUT_ANSWERS]>::try_from(answers.as_slice()) {
        Ok(exactly) => report_of_answers(cwd, &exactly),
        Err(_) => layout_report_asked_for(context, cwd),
    };
}
