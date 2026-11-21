//! What:
//!  The folders a push's children find their programs in,
//! where that list came from,
//! and the refusal that names both
//!  when `git-lfs` is not in it.
//! Why:
//!   A push typed in a shell searches that shell's `PATH`,
//! and a push the commit service runs searches the service's own,
//!  which the manager fixed at the service's start;
//! a refusal that names only `PATH` leaves the person unable to tell
//!  which list was searched or how to change it.
//! The list and the refusal sit beside `lfs_pre_push` because both files
//!  together are more than the
//! package's length limit allows one file (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const search = ProgramSearch.of(environment); const refusal = missingProgramMessage(search);
//! ```

/// The program the upload runs,
/// named in the refusal.
use crate::lfs_pre_push::LFS_PROGRAM;
/// Owned operating-system text for the arguments.
use std::ffi::OsString;

/// The variable the commit service marks the environment of a push it runs with,
/// so the refusal says whose list was searched.
/// The mark is written by the service and read here as present or absent;
/// where the service sets it,
/// the value it writes is `1` and nothing reads that value.
pub(crate) const PUSH_FROM_SERVICE: &str = "MONOCHROMATIC_PUSH_RUNS_IN_THE_COMMIT_SERVICE";

/// The variable the commit service adds to a push it runs with the list the command that asked for it gave,
/// so the refusal names that command's list rather than the service's.
/// The mark is written by the service and read here as present or absent;
/// where the service sets it,
/// the value it writes is `1` and nothing reads that value.
pub(crate) const PUSH_FROM_THE_REQUEST: &str = crate::service_push_programs_held_in_memory::FROM_THE_REQUEST;

/// What:
///  The folders a push's children find their programs in,
///  and where that list came from.
/// Why:
///   A push typed in a shell searches that shell's `PATH`,
/// and a push the commit service runs searches the service's own,
/// which the
///  user manager fixed at the service's start and which no command of the person reaches.
/// A refusal that names
///  only `PATH` leaves the person unable to tell which list was searched or how to change it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramSearch {
    /// The `PATH` the push's children receive.
    pub path: OsString,
    /// Where that list came from.
    pub source: ProgramSearchSource,
}

/// What:
///  Where a push's program search list came from.
/// Why:
///   The refusal says it,
/// so a person learns whether their own shell was searched or the service's list,
/// and which
///  one to change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgramSearchSource {
    /// The environment of the command that is pushing.
    ThisCommand,
    /// The list the command that asked for the push gave,
    /// carried in the memory of the service that runs it.
    TheCommandThatAsked,
    /// The commit service's own environment,
    /// because it runs the push of a commit and no list came with the request.
    ServiceEnvironment,
}

impl ProgramSearch {
    /// What:
    ///  The list the given environment holds,
    ///  and where it came from,
    /// read from the marker the commit service puts on a push it runs.
    /// Why:
    ///   Only the service composes a push environment,
    /// so it marks it,
    /// and every other push searches the environment of
    ///  its own command.
    /// The refusal then names the list it really searched,
    /// never one of another push.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const source = env[PUSH_FROM_THE_REQUEST] ? { kind: 'theCommandThatAsked' } : env[PUSH_FROM_SERVICE] ? { kind: 'serviceEnvironment' } : { kind: 'thisCommand' };
    /// ```
    pub fn of(environment: &[(OsString, OsString)]) -> ProgramSearch {
        let from_the_request: Option<OsString> = crate::child_environment::environment_value(environment, PUSH_FROM_THE_REQUEST);
        let from_the_service: Option<OsString> = crate::child_environment::environment_value(environment, PUSH_FROM_SERVICE);
        let source: ProgramSearchSource = match (from_the_request, from_the_service) {
            (Some(_), _) => ProgramSearchSource::TheCommandThatAsked,
            (None, Some(_)) => ProgramSearchSource::ServiceEnvironment,
            (None, None) => ProgramSearchSource::ThisCommand,
        };
        let path: OsString = crate::child_environment::environment_value(environment, "PATH").unwrap_or_default();
        return ProgramSearch { path, source };
    }

    /// What:
    ///  The folders the list names,
    ///  as one line,
    ///  an empty entry standing for the current directory as `PATH` reads it.
    /// Why:
    ///   The refusal must name what was searched,
    /// and `PATH` itself is not a list of folders a person can read at a glance.
    fn folders(&self) -> String {
        let mut names: Vec<String> = Vec::new();
        for folder in std::env::split_paths(self.path.as_os_str()) {
            let shown: String = if folder.as_os_str().is_empty() { String::from(".") } else { folder.display().to_string() };
            names.push(shown);
        }
        return if names.is_empty() { String::from("none (the list is empty)") } else { names.join(", ") };
    }

    /// What:
    ///  Where the searched list came from,
    ///  in words that say what the person could change.
    /// Why:
    ///   The list is the whole question:
    /// a push that fails on the service's list would succeed on the shell's.
    fn origin(&self) -> String {
        return match &self.source {
            ProgramSearchSource::ThisCommand => String::from("The list is this command's own `PATH`"),
            ProgramSearchSource::TheCommandThatAsked => {
                String::from("The list is the `PATH` of the shell that asked for this push, which travelled with the request to the commit service that runs it")
            }
            ProgramSearchSource::ServiceEnvironment => format!(
                "No list of the shell that made the commit came with this push, so it searched the commit service's own `PATH`, which the manager that started the service fixed, and that `PATH` is not the one the shell that made the commit searches; that shell may hold `{LFS_PROGRAM}` anyway"
            ),
        };
    }

    /// What:
    ///  Where the person can put the program so the same push finds it,
    ///  with the push that follows it.
    /// Why:
    ///  A remedy that names a list the person cannot change sends them nowhere,
    /// and a remedy that promises an effect the named command does not have sends them round a circle.
    /// A push typed in that shell runs the upload in that shell's environment,
    /// and the commit service runs no part of that shell.
    fn remedy(&self) -> String {
        return match &self.source {
            ProgramSearchSource::ThisCommand => String::from("on this command's `PATH` and push again"),
            // Making the commit again is what brings that shell's list to the service, so it is the remedy that fits this list.
            ProgramSearchSource::TheCommandThatAsked => String::from("on the `PATH` of the shell that commits, then make the commit again"),
            ProgramSearchSource::ServiceEnvironment => String::from(
                "on the `PATH` of a shell and push by hand once from that shell (`git push`), which searches that shell's own `PATH`",
            ),
        };
    }
}

/// What:
///  The refusal printed when `git-lfs` is not in the folders the push searches.
/// Why:
///   Without the program the objects cannot be uploaded,
/// and pushing anyway would publish pointers the server cannot
///  serve;
/// the refusal names the program,
/// the folders that were searched and where that list came from,
/// and every valid way
///  forward,
/// including leaving LFS if the repository does not need it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: this push was stopped before it sent anything: ... not found in ${search.folders()}; ${search.origin()} ...`
/// ```
pub fn missing_program_message(search: &ProgramSearch) -> String {
    return format!(
        "monochromatic-git-policy-cli: this push was stopped before it sent anything. The Git configuration of this repository has Git LFS \
         keys (`filter.lfs.*` or `lfs.*`), so the wrapper runs `{LFS_PROGRAM} push` to upload the large \
         files the pushed commits point to before it publishes the pointer files, but `{LFS_PROGRAM}` was not found in \
         the folders this push searches: {folders}. {origin}. Pushing without that upload would publish pointers whose content the LFS server does \
         not have. Ways forward: put the folder that holds `{LFS_PROGRAM}` {remedy}; or install Git LFS so that a folder the list \
         names holds it; or, if this \
         repository does not use Git LFS, remove its configuration (`git lfs uninstall`, or delete the `filter.lfs.*` \
         and `lfs.*` keys); or accept pointers without objects for this push with `git push --no-verify` and upload \
         the objects later with `git lfs push --all <remote>` once Git LFS is installed.\n",
        folders = search.folders(),
        origin = search.origin(),
        remedy = search.remedy(),
    );
}
