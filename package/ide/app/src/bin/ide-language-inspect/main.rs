//! Drive the headless Language module through a scripted plan and print what it observed.
//!
//! The package task `inspect:language` builds this program in the bounded container and runs it
//! on the host against disposable projects with real language servers.
//!  It reads one JSON plan,
//! makes the plan's project root the working directory exactly as the application does at
//! startup,
//!  and prints one JSON object per observation on standard output.

/// JSON rendering of replies,
///  status,
///  diagnostics,
///  and hints.
mod render;
/// Execution of plan steps against the worker handle.
mod session;

/// Failures name the operation that failed.
use anyhow::{Context, Result, bail};
/// The production setup and the unconfined control setup.
use ide_app::language::config::LanguageSetup;
/// What:
///  `Deserialize` lets the serde library decode JSON into these records.
/// Why:
///  A typed plan rejects a malformed step instead of guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Plan = { project: string; steps: Step[] };
/// ```
use serde::Deserialize;
/// What:
///  `PathBuf` is an owned filesystem path (sibling:
///  borrowed `&Path`).
/// Why:
///  Paths come from the decoded plan and outlive it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathBuf = string;
/// ```
use std::path::PathBuf;

/// What:
///  One instruction of a plan.
///  `#[serde(tag = "do")]` selects the variant by the JSON
///       member `do`;
///  `usize` is the address-sized index type and `u64` an unsigned 64-bit count.
/// Why:
///  The same few steps express every real-server check:
///  all five feature paths,
///  the
///      reload,
///  and the stale-reply case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Step = { do: 'open'; file: string } | { do: 'ready'; seconds: number } | /* ... */;
/// ```
#[derive(Debug, Deserialize)]
#[serde(tag = "do", rename_all = "lowercase")]
pub enum Step {
    /// Display a file of the project,
    ///  read from disk.
    Open {
        /// Path relative to the project root.
        file: PathBuf,
    },
    /// Wait until some server of the displayed file is ready.
    Ready {
        /// Longest wait.
        seconds: u64,
    },
    /// Send a position request,
    ///  repeating it until the wanted outcome appears.
    Request {
        /// `definition`,
        ///  `references`,
        ///  or `hover`.
        kind: String,
        /// Character offset in the displayed text.
        at: usize,
        /// Outcome to wait for:
        ///  `hover`,
        ///  `locations`,
        ///  `empty`,
        ///  or empty text for the first answer.
        #[serde(default)]
        until: String,
        /// Longest wait.
        seconds: u64,
    },
    /// Report visible lines and wait for at least `minimum` hints.
    Hints {
        /// First visible line.
        first: usize,
        /// Number of visible lines.
        visible: usize,
        /// Fewest hints that end the wait.
        minimum: usize,
        /// Longest wait.
        seconds: u64,
    },
    /// Wait until the displayed diagnostics number between `minimum` and `maximum`.
    Diagnostics {
        /// Fewest diagnostics that end the wait.
        minimum: usize,
        /// Most diagnostics that end the wait.
        maximum: usize,
        /// Longest wait.
        seconds: u64,
    },
    /// Write new content to the displayed file,
    ///  as an external editor would,
    ///  and apply the reload.
    Reload {
        /// The new content.
        text: String,
    },
    /// Send a hover request and reload at once,
    ///  then report whether its reply was dropped.
    Stale {
        /// Character offset of the hover.
        at: usize,
        /// The new content.
        text: String,
        /// Milliseconds to keep polling afterwards.
        settle: u64,
    },
    /// Write a file of the project as another program would;
    ///  the displayed document is not touched.
    Write {
        /// Path relative to the project root.
        file: PathBuf,
        /// The new content.
        text: String,
    },
    /// Wait until at least `minimum` folders are watched for the servers.
    Folders {
        /// Fewest folders that end the wait.
        minimum: usize,
        /// Longest wait.
        seconds: u64,
    },
    /// Keep polling for a fixed time.
    Sleep {
        /// How long.
        milliseconds: u64,
    },
    /// Stop displaying the file.
    Close,
}

/// The application forwards file changes,
///  so a plan does too unless it says otherwise.
fn forward_by_default() -> bool {
    return true;
}

/// A whole plan.
#[derive(Debug, Deserialize)]
struct Plan {
    /// Root of a disposable project.
    project: PathBuf,
    /// Run servers without confinement;
    ///  only for guard controls on disposable projects.
    #[serde(default)]
    unconfined: bool,
    /// Extra definitions in Helix `languages.toml` syntax,
    ///  for example probe variables.
    #[serde(default)]
    extra_languages: Option<String>,
    /// Watch the project's folders for the servers and forward changes,
    ///  as the application does;
    ///  false
    /// is the positive control in which servers hear about no change made outside the IDE.
    #[serde(default = "forward_by_default")]
    forward_file_changes: bool,
    /// Steps in order.
    steps: Vec<Step>,
}

/// What:
///  The program entry point.
///  `Result<()>` is success without a value,
///  or an error that is
///       printed and turns into a failure exit status.
/// Why:
///  The working directory is set before anything else,
///  as the application must do,
///  because
///      Helix reads it once and roots every server from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = JSON.parse(readFileSync(process.argv[2], 'utf8')); process.chdir(plan.project); run(plan);
/// ```
fn main() -> Result<()> {
    // What: `args_os().nth(1)` takes the first argument after the program name, as an `Option`.
    // Why: The plan file is the only argument.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const planPath = process.argv[2];
    // ```
    let Some(argument) = std::env::args_os().nth(1) else {
        bail!("Usage: ide-language-inspect <plan.json>");
    };
    // `PathBuf::from` wraps the raw argument as a path without converting its bytes.
    let plan_path = PathBuf::from(argument);
    // The trailing `?` returns the error to the caller when reading or decoding fails.
    let encoded = match std::fs::read_to_string(&plan_path) {
        Ok(text) => text,
        Err(error) => {
            return Err(anyhow::Error::new(error)
                .context(format!("Cannot read plan {}", plan_path.display())));
        }
    };
    let plan: Plan = serde_json::from_str(&encoded).context("Cannot decode the plan")?;
    let project = match plan.project.canonicalize() {
        Ok(found) => found,
        Err(error) => {
            return Err(anyhow::Error::new(error)
                .context(format!("Cannot resolve project {}", plan.project.display())));
        }
    };
    ide_app::language::enter_project_directory(&project)?;
    // Log to standard error so standard output carries only observations. The worker's debug records
    // are what the inspection reads, so they are on unless `RUST_LOG` says otherwise; helix-lsp's
    // records go through the same re-labelling as in the application.
    ide_app::logging::install(
        ide_app::logging::filter("ide_app=debug"),
        std::io::stderr,
        None,
    )?;
    // The production setup confines every server; a plan may ask for the unconfined control.
    let mut setup = if plan.unconfined {
        LanguageSetup::unconfined()
    } else {
        LanguageSetup::default()
    };
    setup.extra_languages = plan.extra_languages.clone();
    let mut session = session::Session::new(&project, setup, plan.forward_file_changes)?;
    // `enumerate` pairs each step with its position, for the printed record.
    for (number, step) in plan.steps.iter().enumerate() {
        session.run(number, step)?;
    }
    session.finish();
    // `Ok(())` reports success without a value.
    return Ok(());
}
