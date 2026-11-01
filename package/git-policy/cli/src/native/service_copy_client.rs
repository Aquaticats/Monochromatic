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
use super::service_copy::{CopyOutcome, CopyReply, decode_copy_reply};
use super::service_messages::{ClientFailure, LOG_COMMAND, Platform};
use super::service_protocol::{ClientMessage, PROTOCOL_VERSION, encode_client};
use super::service_provision::{ProvisionHost, provision};
use super::service_route::{Route, RouteInputs, choose_route, remembered_own_executable};
use super::worktree_copy::CopyRequester;
use super::worktree_copy_error::WorktreeCopyError;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How long a copy runs before the client says it is still running.
pub const PROGRESS_GRACE: Duration = Duration::from_secs(1);

/// The longest the client waits for the next word from a copy that is running.
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
    return format!(
        "cli-git: the ignored files were not copied into the new worktree, which was created. {reason} The copy is saved: it is \
         made the next time the commit service is available and any git command runs in this repository."
    );
}

/// What:
///  Hands the requests to the service and waits until each one is answered.
/// Why:
///   The same request goes again after a broken connection, and the answers already received are kept, so a service killed
///  between two requests costs only a reconnect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await requestCopies(config, commonDir, ids, notices)
/// ```
pub fn request_copies(
    config: &ClientConfig,
    common_dir: &Path,
    ids: &[String],
    notices: &mut dyn Write,
) -> Result<Vec<(String, CopyOutcome)>, ClientFailure> {
    let mut answered: Vec<(String, CopyOutcome)> = Vec::new();
    let mut announced: bool = false;
    let mut replacements: usize = 0;
    let mut breaks: usize = 0;
    loop {
        let waiting: Vec<String> =
            ids.iter().filter(|id| return !answered.iter().any(|(known, _)| return known == *id)).cloned().collect();
        if waiting.is_empty() {
            return Ok(answered);
        }
        let (mut reader, version) = reach(config, notices, &mut announced)?;
        if version != PROTOCOL_VERSION {
            // An older service is asked to finish and exit; the manager starts the current one for the next connection.
            if version > PROTOCOL_VERSION || replacements >= super::service_client::REPLACEMENTS {
                return Err(ClientFailure::NoCommonVersion);
            }
            replacements += 1;
            let mut retire: String = encode_client(&ClientMessage::Retire);
            retire.push('\n');
            let _sent = reader.get_mut().write_all(retire.as_bytes());
            std::thread::sleep(RECONNECT_PAUSE * 4);
            continue;
        }
        let mut line: String = encode_client(&ClientMessage::Copy { common_dir: common_dir.to_path_buf(), ids: waiting });
        line.push('\n');
        if reader.get_mut().write_all(line.as_bytes()).is_ok() {
            let _widened = reader.get_ref().set_read_timeout(Some(COPY_WAIT));
            let mut current: Option<String> = None;
            loop {
                let mut text: String = String::new();
                match reader.read_line(&mut text) {
                    Ok(0) => break,
                    Ok(_) => match decode_copy_reply(text.trim_end_matches('\n')) {
                        Ok(CopyReply::Started { destination, .. }) => {
                            current = Some(destination);
                            let _short = reader.get_ref().set_read_timeout(Some(PROGRESS_GRACE));
                        }
                        Ok(CopyReply::Notice(notice)) => {
                            let _written = writeln!(notices, "{notice}");
                        }
                        Ok(CopyReply::Done { id, outcome }) => {
                            current = None;
                            let _widened = reader.get_ref().set_read_timeout(Some(COPY_WAIT));
                            answered.push((id, outcome));
                        }
                        Ok(CopyReply::Finished) => return Ok(answered),
                        Err(_) => break,
                    },
                    Err(error)
                        if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) =>
                    {
                        // A copy that outlasts the grace is still running; say so once and wait for the long bound.
                        if let Some(destination) = current.take() {
                            let _written = writeln!(
                                notices,
                                "cli-git: still copying ignored files into {destination}; a large copy takes a while."
                            );
                            let _widened = reader.get_ref().set_read_timeout(Some(COPY_WAIT));
                        } else {
                            return Err(ClientFailure::Unavailable);
                        }
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
///  Wakes the service for requests or journals that are pending, and waits for nothing.
/// Why:
///   A command that creates no worktree must not wait for a copy, but a copy left by a killed wrapper or service is run
///  by the next command that finds one pending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await wakeCopies(config, commonDir)
/// ```
pub fn wake_copies(config: &ClientConfig, common_dir: &Path) {
    let mut line: String = encode_client(&ClientMessage::Copy { common_dir: common_dir.to_path_buf(), ids: Vec::new() });
    line.push('\n');
    let mut announced: bool = true;
    let Ok((mut reader, version)) = reach(config, &mut std::io::sink(), &mut announced) else { return };
    if version == PROTOCOL_VERSION {
        let _sent = reader.get_mut().write_all(line.as_bytes());
    }
}

/// The requester that goes through the resident service.
pub struct ServiceCopyRequester {
    /// How to reach the service, or why there is no way.
    reach: Result<(ClientConfig, Option<ProvisionHost>), ClientFailure>,
    /// Whether the service's socket activation was set up by this process already.
    provisioned: bool,
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
        return ServiceCopyRequester { reach, provisioned: false };
    }
}

impl CopyRequester for ServiceCopyRequester {
    fn request(
        &mut self,
        common_dir: &Path,
        ids: &[String],
        stderr: &mut dyn Write,
    ) -> Result<Vec<(String, CopyOutcome)>, WorktreeCopyError> {
        let (config, host) = match &self.reach {
            Ok((config, host)) => (config.clone(), host.clone()),
            Err(failure) => return Err(WorktreeCopyError(copy_unavailable_text(failure))),
        };
        if let Some(host) = host.as_ref().filter(|_| return !self.provisioned) {
            provision(host).map_err(|failure| return WorktreeCopyError(copy_unavailable_text(&failure)))?;
            self.provisioned = true;
        }
        return request_copies(&config, common_dir, ids, stderr)
            .map_err(|failure| return WorktreeCopyError(copy_unavailable_text(&failure)));
    }

    fn wake(&mut self, common_dir: &Path) {
        // Only a socket that is there is used: a command that creates nothing sets nothing up.
        if let Ok((config, None)) = &self.reach {
            wake_copies(&ClientConfig { start_wait: WAKE_WAIT, ..config.clone() }, common_dir);
        }
    }
}

/// Copy-client controls stay out of the release executable.
#[cfg(test)]
#[path = "service_copy_client_tests.rs"]
mod tests;
