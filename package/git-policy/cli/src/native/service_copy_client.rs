//! What:
//!  The wrapper's side of a worktree copy: hand the written requests to the resident service, wait honestly for the
//!  answers while showing that a large copy is still running, survive a service that is killed meanwhile, and turn every
//!  failure into plain words that say what was saved and what to run.
//! Why:
//!  The copy is the service's work, so the wrapper only waits on its connection, which is not a lock. A service that is
//!  killed mid-copy is started again by the operating system and sent the same request, which is still on disk; a
//!  wrapper that is killed leaves the copy to the service. Where no service can run, the copy fails loudly and the
//!  request stays, so the next command that finds a service finishes it (`doc/worktree-copy-claim.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcomes = await requestCopies(config, commonDir, ids, stderr);
//! ```

use super::service_client::{ClientConfig, Activation, RECONNECTS, START_WAIT, reach};
use super::service_copy::{CopyOutcome, CopyReply, decode_copy_reply, withdraw_notice};
use super::service_messages::{ClientFailure, LOG_COMMAND, Platform};
use super::service_protocol::{ClientMessage, PROTOCOL_VERSION, encode_client};
use super::service_provision::{ProvisionHost, provision};
use super::service_route::{Route, RouteInputs, choose_route, remembered_own_executable};
use super::worktree_copy::CopyRequester;
use super::worktree_copy_error::WorktreeCopyError;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How long a copy runs before the client says it is still running.
pub const PROGRESS_GRACE: Duration = Duration::from_secs(1);

/// The longest the client waits for the next word from a copy that is running.
pub const PROGRESS_EVERY: Duration = Duration::from_secs(30);

/// How long the client waits for the answers of a copy before it says it cannot tell how it ended, measured from the
///  moment it asked and never reset by a notice.
pub const COPY_WAIT: Duration = Duration::from_secs(2 * 60 * 60);

/// How long a wake that waits for nothing tries to reach the service.
pub const WAKE_WAIT: Duration = Duration::from_secs(2);

/// The pause between sends after a broken connection.
const RECONNECT_PAUSE: Duration = Duration::from_millis(25);

/// What:
///  The words for a copy that could not be handed to the service.
/// Why:
///   The worktree exists and the request is saved, so the person is told what is done and what to run; no implementation
///  term reaches them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyUnavailableText(failure)
/// ```
pub fn copy_unavailable_text(failure: &ClientFailure) -> String {
    let reason: String = match failure {
        ClientFailure::DidNotStart => format!("The commit service did not start. Its log shows why: `{LOG_COMMAND}`."),
        ClientFailure::NoActivation(platform) => super::service_messages::platform_remedy(*platform),
        ClientFailure::OtherAccount => String::from(
            "Another account runs the commit service here. Use your own account, or ask the other account to stop its service.",
        ),
        ClientFailure::NoCommonVersion => String::from(
            "This git is older than the installed commit service and cannot work with it. Update the git wrapper with the tool that installed it.",
        ),
        ClientFailure::ProgramMissing(path) => format!(
            "The commit service program was not found at {path}. Reinstall the git wrapper with the tool that installed it, so that the program sits next to it.",
        ),
        ClientFailure::CouldNotStart(reason) => format!("The commit service could not be set up: {reason}."),
        ClientFailure::UnitInTheWay(path) => format!(
            "The file `{path}` has the name of a settings file of the commit service but was not written by the git wrapper. Move or delete that file.",
        ),
        _ => String::from("The commit service stopped before it confirmed the copy."),
    };
    // A connection that broke or timed out after the request was sent says nothing about whether the files were copied.
    let unknown: bool = matches!(failure, ClientFailure::Unavailable | ClientFailure::MayNotHaveLanded);
    let result: &str = if unknown {
        "the ignored files may not have been copied into the new worktree, which was created, and the result is not known."
    } else {
        "the ignored files were not copied into the new worktree, which was created."
    };
    return format!(
        "cli-git: {result} {reason} The copy is saved: it is \
         made the next time the commit service is available and any git command runs in this repository, unless it finished."
    );
}

/// How long the client keeps asking an older service to retire before it says it cannot work with it.
pub const RETIRE_WAIT: Duration = START_WAIT;

/// What:
///  Reaches a service that speaks this wrapper's protocol, asking an older one to retire and waiting for its replacement.
/// Why:
///   The operating system starts the replacement only after the old service has exited, so a connection that still meets
///  the old version means the old writer is still there: nothing is sent to it but the request to retire, and the wait is
///  bounded by time, not by a count of tries. Only a service of this version is ever sent a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reader = await reachCurrent(config, notices, announced)
/// ```
pub fn reach_current(
    config: &ClientConfig,
    notices: &mut dyn Write,
    announced: &mut bool,
) -> Result<BufReader<UnixStream>, ClientFailure> {
    let deadline: Instant = Instant::now() + RETIRE_WAIT;
    loop {
        let (mut reader, version) = reach(config, notices, announced)?;
        if version == PROTOCOL_VERSION {
            return Ok(reader);
        }
        if version > PROTOCOL_VERSION || Instant::now() >= deadline {
            return Err(ClientFailure::NoCommonVersion);
        }
        // An older service is asked to finish and exit; the manager starts the current one for the next connection.
        let mut retire: String = encode_client(&ClientMessage::Retire);
        retire.push('\n');
        let _sent = reader.get_mut().write_all(retire.as_bytes());
        std::thread::sleep(RECONNECT_PAUSE);
    }
}

/// What:
///  Sends the service the identities of the requests this client waits for, and returns once each is answered.
/// Why:
///   The wait is on a connection, not a lock: the service answers each request when it ends, so the client returns as soon
///  as its own requests are answered, whatever else the service goes on to copy. A copy that outlasts a second is
///  announced, and again every half minute, whether the service is copying this request or still busy with another one.
///  A service killed meanwhile is started again by the manager and asked again for what is still unanswered: the
///  requests are on disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcomes = await requestCopies(config, commonDir, ids, notices);
/// ```
pub fn request_copies(
    config: &ClientConfig,
    common_dir: &Path,
    ids: &[String],
    notices: &mut dyn Write,
) -> Result<Vec<(String, CopyOutcome)>, ClientFailure> {
    return request_copies_paced(config, common_dir, ids, notices, (PROGRESS_GRACE, PROGRESS_EVERY));
}

/// `request_copies` with its progress cadence given: the first line after `grace` of silence, then one every `every`.
pub fn request_copies_paced(
    config: &ClientConfig,
    common_dir: &Path,
    ids: &[String],
    notices: &mut dyn Write,
    (grace, every): (Duration, Duration),
) -> Result<Vec<(String, CopyOutcome)>, ClientFailure> {
    let mut answered: Vec<(String, CopyOutcome)> = Vec::new();
    let mut announced: bool = false;
    let mut breaks: usize = 0;
    let began: Instant = Instant::now();
    loop {
        let waiting: Vec<String> =
            ids.iter().filter(|id| return !answered.iter().any(|(known, _)| return known == *id)).cloned().collect();
        if waiting.is_empty() {
            return Ok(answered);
        }
        let mut reader = reach_current(config, notices, &mut announced)?;
        let mut line: String = encode_client(&ClientMessage::Copy { common_dir: common_dir.to_path_buf(), ids: waiting });
        line.push('\n');
        if reader.get_mut().write_all(line.as_bytes()).is_ok() {
            let mut current: Option<String> = None;
            let mut quiet: Duration = grace;
            loop {
                let _quiet_for = reader.get_ref().set_read_timeout(Some(quiet));
                let mut text: String = String::new();
                match reader.read_line(&mut text) {
                    Ok(0) => break,
                    Ok(_) => {
                        quiet = grace;
                        match decode_copy_reply(text.trim_end_matches('\n')) {
                            Ok(CopyReply::Started { destination, .. }) => current = Some(destination),
                            Ok(CopyReply::Notice(notice)) => {
                                let _written = writeln!(notices, "{notice}");
                            }
                            Ok(CopyReply::Done { id, outcome }) => {
                                current = None;
                                answered.push((id, outcome));
                                if ids.iter().all(|known| return answered.iter().any(|(done, _)| return done == known)) {
                                    return Ok(answered);
                                }
                            }
                            Ok(CopyReply::Finished) => return Ok(answered),
                            Err(_) => break,
                        }
                    }
                    Err(error)
                        if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) =>
                    {
                        if began.elapsed() >= COPY_WAIT {
                            return Err(ClientFailure::Unavailable);
                        }
                        // A copy that outlasts the grace is still running, or waits for the copy of another worktree.
                        let progress: String = match &current {
                            Some(destination) => {
                                format!("cli-git: still copying ignored files into {destination}; a large copy takes a while.")
                            }
                            None => String::from(
                                "cli-git: waiting for the commit service to get to this copy; it is busy with another one.",
                            ),
                        };
                        let _written = writeln!(notices, "{progress}");
                        quiet = every;
                    }
                    Err(_) => break,
                }
            }
        }
        // The connection broke before everything was answered: the service died or was replaced. The requests are still on
        //  disk, so asking again is safe.
        breaks += 1;
        if breaks > RECONNECTS {
            return Err(ClientFailure::MayNotHaveLanded);
        }
        std::thread::sleep(RECONNECT_PAUSE * u32::try_from(breaks).unwrap_or(1));
    }
}

/// What:
///  Tells the service that requests or journals are pending, and waits for nothing.
/// Why:
///   Reaches the service the same way a request does, an older service retired and a newer one refused, so a pending copy
///  is finished by whichever service is current.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wakeCopies(config, commonDir)
/// ```
pub fn wake_copies(config: &ClientConfig, common_dir: &Path) {
    let mut line: String = encode_client(&ClientMessage::Copy { common_dir: common_dir.to_path_buf(), ids: Vec::new() });
    line.push('\n');
    let mut announced: bool = true;
    let Ok(mut reader) = reach_current(config, &mut std::io::sink(), &mut announced) else { return };
    let _sent = reader.get_mut().write_all(line.as_bytes());
}

/// The requester that goes through the resident service.
pub struct ServiceCopyRequester {
    /// How to reach the service, or why there is no way.
    reach: Result<(ClientConfig, Option<ProvisionHost>), ClientFailure>,
    /// Whether the service's socket activation was set up by this process already.
    provisioned: bool,
    /// The requests whose failure was returned and not yet shown.
    failed: Vec<String>,
    /// What sets the service up through the session bus; a control replaces it.
    provisioner: fn(&ProvisionHost) -> Result<(), ClientFailure>,
}

impl ServiceCopyRequester {
    /// The requester for the environment this process runs in: the route follows the machine, as a commit's does.
    pub fn from_environment(real_git: &Path) -> ServiceCopyRequester {
        let path_variable = |name: &str| -> Option<PathBuf> {
            return std::env::var_os(name).filter(|value| return !value.is_empty()).map(PathBuf::from);
        };
        let runtime: Option<PathBuf> = path_variable("XDG_RUNTIME_DIR");
        let config_home: Option<PathBuf> = path_variable("XDG_CONFIG_HOME");
        let home: Option<PathBuf> = path_variable("HOME");
        let bus_address: Option<String> = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
        let route: Route = choose_route(&RouteInputs {
            runtime_dir: runtime.as_deref(),
            bus_address: bus_address.as_deref(),
            config_home: config_home.as_deref(),
            home: home.as_deref(),
            own_executable: remembered_own_executable(),
            real_git,
        });
        let reach: Result<(ClientConfig, Option<ProvisionHost>), ClientFailure> = match route {
            Route::Service { address, provision: host } => Ok((
                ClientConfig {
                    address,
                    activation: Activation::Managed,
                    start_wait: START_WAIT,
                    commit_wait: COPY_WAIT,
                },
                host,
            )),
            Route::Earlier => Err(ClientFailure::NoActivation(if cfg!(target_os = "linux") {
                Platform::LinuxWithoutUserServices
            } else {
                Platform::Other
            })),
        };
        return ServiceCopyRequester { reach, provisioned: false, failed: Vec::new(), provisioner: provision };
    }
}

impl CopyRequester for ServiceCopyRequester {
    fn request(
        &mut self,
        common_dir: &Path,
        ids: &[String],
        stderr: &mut dyn Write,
    ) -> Result<Vec<(String, CopyOutcome)>, WorktreeCopyError> {
        let (config, setup) = match &self.reach {
            Ok((config, setup)) => (config.clone(), setup.clone()),
            Err(failure) => return Err(WorktreeCopyError(copy_unavailable_text(failure))),
        };
        if let Some(host) = setup.as_ref().filter(|_| return !self.provisioned) {
            (self.provisioner)(host).map_err(|failure| return WorktreeCopyError(copy_unavailable_text(&failure)))?;
            self.provisioned = true;
        }
        let answers = request_copies(&config, common_dir, ids, stderr)
            .map_err(|failure| return WorktreeCopyError(copy_unavailable_text(&failure)))?;
        // The service left a notice for each failure before it told this client; it is withdrawn once the failure is shown.
        self.failed.extend(
            answers
                .iter()
                .filter(|(_, outcome)| return matches!(outcome, CopyOutcome::Failed(_)))
                .map(|(id, _)| return id.clone()),
        );
        return Ok(answers);
    }

    fn wake(&mut self, common_dir: &Path) {
        // A pending copy is the person's earlier command still unfinished, so the wake sets the service up the same way
        // that command did; a wake that cannot reach a service says nothing, because that command already failed loudly.
        let Ok((config, setup)) = &self.reach else { return };
        if let Some(host) = setup.as_ref().filter(|_| return !self.provisioned) {
            if (self.provisioner)(host).is_err() {
                return;
            }
            self.provisioned = true;
        }
        wake_copies(&ClientConfig { start_wait: WAKE_WAIT.max(config.start_wait.min(START_WAIT)), ..config.clone() }, common_dir);
    }

    fn failures_shown(&mut self, common_dir: &Path) {
        for id in self.failed.drain(..) {
            withdraw_notice(common_dir, id.as_str());
        }
    }
}

/// Copy-client controls stay out of the release executable.
#[cfg(test)]
#[path = "service_copy_client_tests.rs"]
mod tests;
