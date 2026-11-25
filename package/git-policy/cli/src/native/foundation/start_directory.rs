//! What:
//!  The directory a command was started in,
//!  read from the process where the command begins
//!  and handed to the code that needs it.
//! Why:
//!  The current directory belongs to the whole process,
//!  and a test runner runs a crate's tests as threads of one process,
//!  so a library read of it answers with whatever directory another test
//!  happens to have moved the process to,
//!  or with one that test has already deleted.
//!  One read where the command starts makes that answer an input,
//!  so a caller that starts somewhere else says so
//!  and no caller changes the directory of anyone else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const start: StartDirectory = StartDirectory.readFromTheProcess();
//! ```

/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};

/// What:
///  The directory the process was started in,
///  or the operating system's own words for why there is none.
/// Why:
///  Every reader of the current directory asks the same question but answers a missing one differently:
///  a path with no answer,
///  a fallback of its own,
///  or a refusal that names the reason.
///  Carrying both outcomes lets each reader keep the answer it always gave,
///  and lets a caller that was told where to start say so without a read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StartDirectory = { path: string } | { unreadable: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartDirectory {
    /// The directory,
    ///  or the reason there is none.
    outcome: Result<PathBuf, String>,
}

impl StartDirectory {
    /// What:
    ///  Read the directory this process was started in,
    ///  once,
    ///  where the command begins.
    /// Why:
    ///   This is the one read of the process's current directory the release executable makes;
    ///  every other reader takes its answer from here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// StartDirectory.readFromTheProcess() // { path: process.cwd() }
    /// ```
    pub fn read_from_the_process() -> StartDirectory {
        let outcome: Result<PathBuf, String> = std::env::current_dir().map_err(|error| return error.to_string());
        return StartDirectory { outcome };
    }

    /// What:
    ///  The directory a caller names as the place this command starts in.
    /// Why:
    ///   A test starts the code under test in a fixture of its own,
    ///  and a management command that was told where to work names that directory,
    ///  so neither reads nor changes the directory of the process it runs in.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// StartDirectory.at(fixture) // { path: fixture }
    /// ```
    pub fn at(directory: &Path) -> StartDirectory {
        return StartDirectory { outcome: Ok(directory.to_path_buf()) };
    }

    /// What:
    ///  No directory at all,
    ///  as when a caller knows of none.
    /// Why:
    ///   A caller that was given nothing must not fall back to the process's directory,
    ///  which is the answer this type exists to stop reading;
    ///  every reader then takes its own "no directory" ending.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// StartDirectory.unknown() // { unreadable: 'the caller named no directory' }
    /// ```
    pub fn unknown() -> StartDirectory {
        return StartDirectory { outcome: Err(String::from("the caller named no directory the command starts in")) };
    }

    /// What:
    ///  The directory,
    ///  or nothing when there is none.
    /// Why:
    ///   Most readers use an absent directory as "this answer cannot be worked out";
    ///  `Option` is that ending.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const path = 'path' in start ? start.path : undefined;
    /// ```
    pub fn path(&self) -> Option<&Path> {
        return self.outcome.as_deref().ok();
    }

    /// What:
    ///  Why there is no directory,
    ///  in the operating system's own words,
    ///  or nothing when there is one.
    /// Why:
    ///   A refusal that names the reason is the only thing a person can act on,
    ///  and readers that refuse say the reason they were given.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const reason = 'unreadable' in start ? start.unreadable : undefined;
    /// ```
    pub fn reason(&self) -> Option<&str> {
        return self.outcome.as_ref().err().map(String::as_str);
    }
}

/// The two outcomes and the two callers' answers stay out of the release executable.
#[cfg(test)]
#[path = "start_directory_tests.rs"]
mod tests;
