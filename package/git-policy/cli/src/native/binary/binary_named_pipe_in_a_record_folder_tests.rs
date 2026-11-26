//! What:
//!  A named pipe standing where this program keeps a record:
//!  the wrapped command answers,
//!  names the file,
//!  and goes on or stops as that record's own unreadability decides.
//! Why:
//!  Opening a pipe for reading waits for a writer that may never come,
//!  so one pipe in a folder the start of every command reads held every wrapped command
//!  in that repository with no output and no end (issue 6415's shape at two places it left,
//!  `SPEC.md` "Every file this wrapper reads from its own record folders is opened as a plain file").
//!  The fixture's home and its Git directory both belong to this control,
//!  so the pipe stands where the real program looks and nothing on this machine is touched.
//!  Each control runs the built executable under a bound of its own,
//!  short enough that the pipe's wait is a failure of the control rather than a hang.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! mkfifoSync(join(repo, '.git/monochromatic-git-policy-cli-merges/x.json'), 0o600);
//! expect(runWrapped({ cwd: repo, args: ['status'] }, { bound: '20' }).code).toBe(2);
//! ```

use super::support::{
    Fixture,
    Observed,
    bounded_for,
    fixture,
    observe,
    remove,
    repository,
    wrapper_first_path,
};
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};

/// How long a command of these controls may run, in seconds.
/// Why:
///  A healthy command here answers in milliseconds,
///  and a command that waits on a pipe never answers at all,
///  so twenty seconds tells the two apart on a host other sessions load,
///  and a control that reached it fails with what it waited for instead of holding the run.
const WAIT_SECONDS: &str = "20";

/// What:
///  The fixture's `git`-named link to the built executable,
///  killed at `WAIT_SECONDS`,
///  in `directory`.
/// Why:
///  One body builds the command both controls run,
///  so no control reaches a bound of its own that the other does not share.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const command = boundedFor(fixture, join(fixture.root, 'bin/git'), wrapperFirstPath(fixture), '20'); command.cwd = directory; command.args = args;
/// ```
fn a_bounded_wrapped_command(
    fixture: &Fixture,
    directory: &Path,
    arguments: &[&str],
) -> std::process::Command {
    let mut command: std::process::Command = bounded_for(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        WAIT_SECONDS,
    );
    command.current_dir(directory).args(arguments);
    return command;
}

/// What:
///  The words a command printed on its standard error.
/// Why:
///  A control reads the line the person meets, and standard error is where this program writes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = Buffer.from(observed.stderr).toString('utf8');
/// ```
fn stderr_of(observed: &Observed) -> String {
    return String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
}

/// What:
///  A fixture repository with this program's own record folder made inside its Git directory,
///  and a named pipe standing at the name a merge record would take.
/// Why:
///  The folder is the one the start of every command lists,
///  and a name it lists that no writer ever opens is the shape that held every command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repository(fixture, 'repo'); mkdirSync(join(repo, '.git/monochromatic-git-policy-cli-merges')); mkfifoSync(join(..., 'x.json'), 0o600); return repo;
/// ```
fn a_repository_with_a_pipe_where_a_merge_record_goes(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let folder: PathBuf = repo.join(".git/monochromatic-git-policy-cli-merges");
    std::fs::create_dir_all(folder.as_path()).expect("the record folder");
    git_policy_cli::subvolume_sync_attrs::make_fifo(folder.join("x.json").as_path(), 0o600)
        .expect("a named pipe where a record goes");
    return repo;
}

/// A `status` in a repository whose merge record folder holds a named pipe stops at once,
/// naming the record it cannot read,
/// instead of holding the command (issue 6415's shape).
#[test]
fn a_command_stops_at_once_with_a_named_pipe_in_the_merge_record_folder() {
    let fixture: Fixture = fixture("named-pipe-in-the-merge-records");
    let repo: PathBuf = a_repository_with_a_pipe_where_a_merge_record_goes(&fixture);
    let mut command: std::process::Command =
        a_bounded_wrapped_command(&fixture, repo.as_path(), &["status"]);
    let observed: Observed = observe(&mut command, b"");
    let stderr: String = stderr_of(&observed);
    assert_eq!(
        observed.code,
        Some(2),
        "the command stopped saying which record it cannot read: {observed:?}"
    );
    assert!(
        stderr.contains("x.json") && stderr.contains("not a plain file"),
        "the line names the record and what stands at its name: {stderr}"
    );
    remove(&fixture);
}

/// A `status` whose home holds a named pipe at the repository link identity answers at once,
/// saying it cannot use the identity,
/// instead of holding the command.
#[test]
fn a_command_answers_at_once_with_a_named_pipe_at_the_repository_link_identity() {
    let fixture: Fixture = fixture("named-pipe-at-the-link-identity");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let identity: PathBuf = fixture
        .root
        .join("home/.config/monochromatic-dev/repository-link.json");
    std::fs::create_dir_all(identity.parent().expect("parent")).expect("config folder");
    git_policy_cli::subvolume_sync_attrs::make_fifo(identity.as_path(), 0o600)
        .expect("a named pipe where the identity file goes");
    let mut command: std::process::Command =
        a_bounded_wrapped_command(&fixture, repo.as_path(), &["status"]);
    let observed: Observed = observe(&mut command, b"");
    let stderr: String = stderr_of(&observed);
    assert_eq!(
        observed.code,
        Some(0),
        "the command went on without the identity: {observed:?}"
    );
    assert!(
        stderr.contains("the repository link identity cannot be used")
            && stderr.contains("not a plain file"),
        "the line says the identity cannot be used and what stands at its name: {stderr}"
    );
    remove(&fixture);
}
