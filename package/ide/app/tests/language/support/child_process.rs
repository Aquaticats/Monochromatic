//! The child process a test runs in:
//!  its project root and the report file below it,
//!  the directory it gets under the shared memory file system,
//!  the log its worker writes,
//!  and the wait for the process to end.

use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
    process::Command,
    time::Duration,
};

/// Set in the child process to the project root the body works on.
const ROOT_VARIABLE: &str = "IDE_LANGUAGE_TEST_ROOT";

/// Name of the scripted server definition.
pub const SERVER: &str = "scripted-ls";

/// Longest wait for any expected state.
pub const PATIENCE: Duration = Duration::from_secs(20);

/// Request timeout of the scripted server,
///  in seconds,
///  in tests whose assertions do not depend on
/// it:
///  Helix's default,
///  which the product uses for every server without its own value
/// (`default_timeout` in `helix-core/src/syntax/config.rs`,
///  and `Languages::timeout`).
pub const PRODUCT_TIMEOUT: u64 = 20;

/// When set in the environment,
///  the child process prints the worker's debug log and helix-lsp's
/// protocol log to standard error,
///  with wall-clock times.
///  An intermittent failure is diagnosed
/// from that log together with the scripted server's report.
const LOG_VARIABLE: &str = "IDE_LANGUAGE_TEST_LOG";

/// Install the opt-in log once per process;
///  later probes of the same test find it installed.
pub fn install_log() {
    if std::env::var_os(LOG_VARIABLE).is_none() {
        return;
    }
    // helix-lsp logs every message it writes and reads at `info`, and dropped answers at `debug`.
    // The application's pipeline re-labels helix-lsp's records as in a real session. Installing
    // fails only when an earlier probe of this process already installed the log.
    let filter = tracing_subscriber::EnvFilter::builder().parse_lossy(
        ide_app::logging::directives("ide_app=debug,helix_lsp=debug", None),
    );
    let _already_installed = ide_app::logging::install(filter, std::io::stderr, None);
}

/// Where the child process works:
///  its project root,
///  its working directory,
///  and its `PWD`.
pub struct Layout {
    /// Project root handed to the worker.
    pub root: PathBuf,
    /// Working directory of the child process.
    pub cwd: PathBuf,
    /// Value of the `PWD` variable,
    ///  which Helix prefers when it names the working directory;
    /// nothing removes the variable.
    pub pwd: Option<PathBuf>,
}

/// The usual layout:
///  the project root is the working directory.
pub fn standard(base: &Path) -> Layout {
    let root = base.join("project");
    fs::create_dir(&root).expect("project directory");
    return Layout {
        root: root.clone(),
        cwd: root,
        pwd: None,
    };
}

/// In the child process:
///  the project root.
///  In the test process:
///  nothing.
pub fn child_root() -> Option<PathBuf> {
    return std::env::var_os(ROOT_VARIABLE).map(PathBuf::from);
}

/// Memory-backed directory for the files of a test,
///  when the system has one.
const MEMORY_DIRECTORY: &str = "/dev/shm";

/// A fresh directory for one test's project and report,
///  in memory when possible.
///
/// The scripted server appends a line to its report for every message it reads or writes,
///  and
/// it answers only after the line is written.
///  On a disk that the rest of the machine keeps busy,
/// one such write can block for seconds;
///  the late answers then look like a slow server,
///  and the
/// server cannot even be killed until the write returns.
///  Memory-backed files never wait for the
/// disk.
///  Elsewhere the usual temporary directory is used.
fn test_directory() -> tempfile::TempDir {
    let memory = Path::new(MEMORY_DIRECTORY);
    if memory.is_dir()
        && let Ok(directory) = tempfile::Builder::new()
            .prefix("ide-language-")
            .tempdir_in(memory)
    {
        return directory;
    }
    return tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("test directory");
}

/// Run the named test again in a child process laid out by `layout`,
///  and require it to pass.
/// `test` is the full test name,
///  including its module.
pub fn run_child(
    test: &str,
    layout: fn(&Path) -> Layout,
) {
    let directory = test_directory();
    let base = directory
        .path()
        .canonicalize()
        .expect("canonical test directory");
    fs::create_dir(base.join("scratch")).expect("scratch directory");
    let placed = layout(&base);
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command
        // An ignored test that was asked for by name must run in the child too; without this the
        // child would run nothing and report success.
        .args([
            test,
            "--exact",
            "--nocapture",
            "--test-threads=1",
            "--include-ignored",
        ])
        .current_dir(&placed.cwd)
        .env(ROOT_VARIABLE, &placed.root);
    match &placed.pwd {
        Some(pwd) => command.env("PWD", pwd),
        // The parent's value names another directory and would only mislead a reader of the log.
        None => command.env_remove("PWD"),
    };
    let status = command.status().expect("child test process");
    assert!(status.success(), "child process of {test} failed: {status}");
}

/// The scratch directory beside the project,
///  for report files and outside-project fixtures.
pub fn scratch(root: &Path) -> PathBuf {
    let mut current = root;
    loop {
        let candidate = current.join("scratch");
        if candidate.is_dir() && !candidate.starts_with(root) {
            return candidate;
        }
        current = current
            .parent()
            .expect("scratch directory beside the project");
    }
}

/// Path of the scripted server's report file.
pub fn report_path(root: &Path) -> PathBuf {
    return scratch(root).join("report.jsonl");
}
