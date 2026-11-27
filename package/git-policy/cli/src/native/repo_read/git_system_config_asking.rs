//! What:
//!  The one question this wrapper puts to a real Git executable about a file that executable compiles in:
//!  `git var GIT_CONFIG_SYSTEM` or `git var GIT_ATTR_SYSTEM`,
//!  and the reason when it printed none.
//! Why:
//!  Two readers learn such a path from the executable itself (`git_system_config.rs`),
//!  and the reason a question gave nothing is the one fact a failing caller needs,
//!  so the ask answers that reason rather than a bare "no".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function askGitVarExplaining(realGit: string, variable: string): Buffer // throws the reason it printed nothing
//! ```

use std::path::Path;

/// What:
///  Start the executable at `real_git` with `var <variable>` and the environment's overrides removed,
/// or answer the reason it printed nothing.
/// Why:
///  A start the host refused and an exit that is not a success both leave the path unknown,
/// and the reason says which (issue 6541);
/// the caller that discards it says it in the wrapper's diagnostics (issue 6540).
pub(crate) fn ask_git_var_explaining(
    real_git: &Path,
    variable: &str,
) -> Result<Vec<u8>, String> {
    let mut command = std::process::Command::new(real_git);
    command
        .args(["var", variable])
        .env_clear()
        .env("LC_ALL", "C")
        .current_dir("/")
        .stdin(std::process::Stdio::null());
    // The environment was cleared, so the marker of this start is added after it (`started_as_git.rs`), and it is a start of
    //  the wrapper's own reading: nothing of the person's runs under it.
    crate::started_as_git::mark_real_git_start(
        &mut command,
        crate::started_as_git::StartMode::Query,
    );
    // What: `Ok` is the captured run and `Err` the refusal or the ceiling, whose text names the program and the wait.
    // Why: `Command::output` would block a thread with no end, and its failure would say nothing about why.
    let output = match crate::captured_child_run_to_its_end::output_of_within_bound(
        &mut command,
        crate::wait_ceilings::PLUMBING_STEP_CEILING,
    ) {
        Ok(output) => output,
        Err(error) => {
            return Err(format!(
                "starting {} for `git var {variable}` gave no answer: {error}",
                real_git.display()
            ));
        }
    };
    if output.status.success() {
        return Ok(output.stdout);
    }
    // What: the child's last standard-error bytes, decoded lossily; Git's own words are what a person can act on, and an empty stream leaves only the status.
    let stderr: std::borrow::Cow<'_, str> = String::from_utf8_lossy(output.stderr.as_slice());
    let said: String = stderr.trim().to_owned();
    if said.is_empty() {
        return Err(format!(
            "`git var {variable}` in {} ended with {}",
            real_git.display(),
            output.status
        ));
    }
    return Err(format!(
        "`git var {variable}` in {} ended with {} and said: {said}",
        real_git.display(),
        output.status
    ));
}
