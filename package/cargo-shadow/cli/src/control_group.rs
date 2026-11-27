//! What:
//! A delegated control group (cgroup v2) as the boundary of everything a bounded run starts:
//! a transient user scope that the manager makes with `Delegate=yes`.
//! Why:
//! Round 7 (sixth review 6a R1 to R3,
//! 6b2 2,
//! 6c R2):
//! a process group,
//! and a child subreaper with a `/proc` scan,
//! failed the one proof that matters,
//! that every writer a command started has stopped.
//! A control group is inherited by every child,
//! also by those that call `setsid` or fork twice,
//! it keeps its members when this program dies,
//! `cgroup.events` says `populated 0` only when no thread is left,
//! and a recovery can ask the same question about a scope the journal recorded.
//! Round 8 (contract `doc/planning/cargo-shadow-guarantees.md`,
//! "The spawn route"):
//! no thread of this program writes a `cgroup.procs` file.
//! The program is started as `systemd-run --user --scope --unit=<name> --property=Delegate=yes --collect -- <program>`,
//! and the manager moves it into the scope before it runs.
//! Round 7's self-move (the wrapper entering the group around a `spawn`) captured other threads' children and could kill the wrapper
//! itself (7a F1,
//! F2,
//! F3) and is gone.
//! The limits:
//! `cgroup.kill` alone does not end a thread group whose leader already exited (measured),
//! so the stop also signals each listed process through a pidfd;
//! a writer that leaves the scope is outside the contract (assumption a).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control-group.ts (Linux only)
//! export function detectRoute(settings: RouteSettings): Route  // throws GroupUnavailable
//! export class ControlGroup {
//!   static plan(route: Route): ControlGroup
//!   launcher(route: Route, command: Command): Command
//!   stop(boundMs: number): boolean
//! }
//! export function query(record: ScopeRecord): 'gone' | 'alive' | { unknown: string }
//! ```

/// Imports `std::fs` for this file.
use std::fs;

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::io::Read` for this file.
use std::io::Read;

/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `std::process::Command` for this file.
use std::process::Command;

/// Imports `rustix::io::Errno` for this file.
use rustix::io::Errno;

/// The items that only the tests of the control group use.
#[cfg(test)]
pub(crate) use crate::control_group_machine_identity::{
    BOOT_ID_PATH,
    read_current_identity,
};
/// The process identity items that only the tests of the control group use.
#[cfg(test)]
pub(crate) use crate::control_group_process_identity::{
    StartOf,
    missing_entry_verdict,
    nspid_shows_own_namespace,
};
/// The route discovery item that only the tests of the control group use.
#[cfg(test)]
pub(crate) use crate::control_group_route_discovery::{
    RouteSettings,
    detect_route,
    kill_file_present,
};

/// The test-only items of the same module.
#[cfg(test)]
pub(crate) use crate::control_group_machine_identity::use_identity_for_test;
/// The boot and machine identity that a journal record of a mount view is checked against (split out of the control group module).
pub(crate) use crate::control_group_machine_identity::{
    Identity,
    current_identity,
    read_identity_text,
};

/// Process identity reads of `/proc` for the containment of a run (split out of the control group module).
pub(crate) use crate::control_group_process_identity::{
    LauncherState,
    launcher_state,
    pid_namespace_id,
    process_start,
};
/// The test-only items of the same module.
#[cfg(test)]
pub(crate) use crate::control_group_process_identity::{
    names_an_absent_process,
    process_is_running,
    procfs_matches_namespace,
};

/// Discovery of the route to a delegated control group (split out of the control group module).
pub(crate) use crate::control_group_route_discovery::{
    CGROUP_ROOT,
    GroupUnavailable,
    Route,
    TOKEN_BYTES,
    View,
    is_cgroup2,
    is_executable_file,
    namespace_id,
    unavailable,
    view_of,
};

/// The records of a scope and the membership verdict (split out of the control group module).
pub(crate) use crate::control_group_records::{
    Membership,
    Record,
    parse_record,
    query,
};

/// The reads of `cgroup.events` for the stop of a scope (split out of the control group module).
pub(crate) use crate::control_group_stop::populated_at;

/// The scope of one run.
/// The fields are `pub(crate)` because the stop of a scope lives in `control_group_stop.rs`.
#[derive(Debug)]
pub(crate) struct ControlGroup {
    /// The unit name without the `.scope` suffix.
    name: String,
    /// The directory the scope has once it runs.
    pub(crate) directory: PathBuf,
    /// The directory that holds it.
    pub(crate) parent: PathBuf,
    /// How that directory was seen when the scope was planned.
    pub(crate) parent_view: View,
    /// The inode of this process's cgroup namespace.
    pub(crate) namespace: u64,
}

/// What:
/// `fn random_token() -> io::Result<String>` is 128 random bits as hexadecimal text.
/// Why:
/// A scope name must never repeat while any journal can reference it,
/// so it carries no process id and no clock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function randomToken(): string
/// ```
fn random_token() -> io::Result<String> {
    let mut bytes: [u8; TOKEN_BYTES] = [0; TOKEN_BYTES];
    fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    let mut text: String = String::with_capacity(TOKEN_BYTES * 2);
    for byte in bytes {
        text.push_str(&format!("{byte:02x}"));
    }
    return Ok(text);
}

/// What:
/// `fn escaped(argument: &std::ffi::OsStr) -> std::ffi::OsString` doubles every dollar sign of an argument.
/// Why:
/// `systemd-run` reads the command it is given like the command line of a unit:
/// it turns two dollar signs into one and replaces a braced variable name by the value of the variable (probed on 2026-11-04),
/// so an argument is encoded for that grammar at the point where it is handed over (STB),
/// and the program receives the bytes it was
/// given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function escaped(argument: string): string
/// ```
fn escaped(argument: &std::ffi::OsStr) -> std::ffi::OsString {
    /// The byte views of an operating-system string,
    /// so escaping never assumes the argument is UTF-8.
    use std::os::unix::ffi::{
        OsStrExt,
        OsStringExt,
    };
    let mut bytes: Vec<u8> = Vec::with_capacity(argument.len());
    for byte in argument.as_bytes() {
        bytes.push(*byte);
        if *byte == b'$' {
            bytes.push(b'$');
        }
    }
    return std::ffi::OsString::from_vec(bytes);
}

/// The methods of the type above.
impl ControlGroup {
    /// What:
    /// `pub(crate) fn plan(route: &Route) -> io::Result<ControlGroup>` names the scope of one run before anything starts.
    /// Why:
    /// The path is known before the program exists,
    /// so the record can be forced to disk first.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static plan(route: Route): ControlGroup
    /// ```
    pub(crate) fn plan(route: &Route) -> io::Result<ControlGroup> {
        let name: String = format!("cargo-shadow-{}", random_token()?);
        let directory: PathBuf = route.slice_dir.join(format!("{name}.scope"));
        return Ok(ControlGroup {
            name,
            directory,
            parent: route.slice_dir.clone(),
            parent_view: route.slice_view,
            namespace: route.namespace,
        });
    }

    /// What:
    /// `pub(crate) fn directory(&self) -> &Path` is the scope's directory.
    /// Why:
    /// Tests and the record name it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// directory(): string
    /// ```
    pub(crate) fn directory(&self) -> &Path {
        return &self.directory;
    }

    /// What:
    /// `pub(crate) fn record(&self) -> Record` is what a journal keeps of this scope.
    /// Why:
    /// The journal line is written and forced to disk before the program starts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// record(): Record
    /// ```
    pub(crate) fn record(&self) -> Record {
        return Record::Scope {
            namespace: self.namespace,
            parent_view: self.parent_view,
            path: self.directory.clone(),
        };
    }

    /// What:
    /// `pub(crate) fn launcher(&self, route: &Route, original: &Command, clean_environment: bool) -> Command` is the command that
    /// starts `original` in this scope.
    /// Why:
    /// `systemd-run --scope` asks the manager to move its own process into the scope and then becomes the program,
    /// so the program runs in the scope from its first instruction and no thread of this program touches a `cgroup.procs`.
    /// The working directory and the variables of the original are given to the launcher;
    /// a command that clears the environment runs through `env --ignore-environment`,
    /// so the manager is still reachable by the launcher and the program sees only the variables it was given.
    /// The scope carries the limits of the caller's control group that the route read (`inherited_limits.rs`):
    /// the manager makes the
    /// scope under its own slice,
    /// so without them a capped caller would get an uncapped run.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// launcher(route: Route, original: Command, cleanEnvironment: boolean): Command
    /// ```
    pub(crate) fn launcher(
        &self,
        route: &Route,
        original: &Command,
        clean_environment: bool,
    ) -> Command {
        let mut launch: Command = Command::new(&route.systemd_run);
        launch.args([
            "--user",
            "--scope",
            "--quiet",
            "--collect",
            "--property=Delegate=yes",
        ]);
        // The scope sits under the manager's slice, so the caller's own limits would not reach it: it carries them (absent stay absent).
        for property in route.limits.properties() {
            launch.arg(format!("--property={property}"));
        }
        launch.arg(format!("--unit={}", self.name));
        launch.arg(format!("--slice={}", route.slice));
        launch.arg("--");
        if let Some(directory) = original.get_current_dir() {
            launch.current_dir(directory);
        }
        if clean_environment {
            launch.arg("env").arg("--ignore-environment");
            for (name, value) in original.get_envs() {
                if let Some(set) = value {
                    let mut assignment: std::ffi::OsString = name.to_os_string();
                    assignment.push("=");
                    assignment.push(set);
                    launch.arg(escaped(&assignment));
                }
            }
        } else {
            for (name, value) in original.get_envs() {
                match value {
                    Some(set) => launch.env(name, set),
                    None => launch.env_remove(name),
                };
            }
        }
        launch.arg(escaped(original.get_program()));
        for argument in original.get_args() {
            launch.arg(escaped(argument));
        }
        return launch;
    }

    /// What:
    /// `pub(crate) fn members(&self) -> io::Result<Vec<i32>>` lists the processes in the scope (the leader of each thread group).
    /// Why:
    /// A thread group whose leader exited is still listed while another of its threads lives.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// members(): number[]
    /// ```
    pub(crate) fn members(&self) -> io::Result<Vec<i32>> {
        #[cfg(test)]
        if crate::failpoints::fires(crate::failpoints::Point::ContainmentInspection) {
            return Err(io::Error::other("injected failure to list the members"));
        }
        let text: String = match fs::read_to_string(self.directory.join("cgroup.procs")) {
            Ok(found) => found,
            // A scope that was removed has no members; `ENODEV` is what a read of a removed group's file says.
            Err(error)
                if error.kind() == io::ErrorKind::NotFound
                    || error.raw_os_error() == Some(Errno::NODEV.raw_os_error()) =>
            {
                return Ok(Vec::new());
            }
            Err(error) => return Err(error),
        };
        let mut found: Vec<i32> = Vec::new();
        for line in text.lines() {
            match line.trim().parse::<i32>() {
                Ok(number) => found.push(number),
                Err(error) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("cgroup.procs holds {line:?} ({error})"),
                    ));
                }
            }
        }
        return Ok(found);
    }

    /// What:
    /// `pub(crate) fn populated(&self) -> io::Result<bool>` says whether any thread of the scope lives.
    /// Why:
    /// This is the kernel's own statement of quiescence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// populated(): boolean
    /// ```
    #[cfg(test)]
    pub(crate) fn populated(&self) -> io::Result<bool> {
        return populated_at(&self.directory);
    }
}
