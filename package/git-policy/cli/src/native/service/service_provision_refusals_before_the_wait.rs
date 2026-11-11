//! What:
//!  The two states in which provisioning already knows that no service can answer this command,
//! and says so at once,
//! with the words helper the third one (a runtime directory that cannot be written) needs.
//! Why:
//!  The client's wait for a start ends at `START_CEILING`,
//! one minute,
//! and a start it asked the manager for once is asked again after a reload,
//! so a case that can never come right spent two minutes before the command said
//!  "The commit service did not start" and nothing about the cause.
//! Each fact below is known before anything is waited for:
//! the address the kernel must bind is longer than any address can be,
//! and the manager's answer about the address it made says whether it is the one this command looks
//!  for,
//! because the unit names its address with `%t` and `%t` is the manager's runtime directory.
//! Every refusal names the input that has to change and every way to change it,
//! and every check has a control that proves it lets a working arrangement through.
//! A session bus with no user service manager behind it is the third such state,
//! and its words live beside the mapping that chooses them (`service_provision_host_and_manager.rs`).

use crate::service_instance::{LONGEST_SOCKET_PATH, longest_runtime_directory, socket_path, socket_unit_instance};
use crate::service_messages::ClientFailure;
use crate::service_provision_host_and_manager::{Manager, ProvisionHost};
use crate::systemd_dbus::CallError;
use std::path::{Path, PathBuf};

/// What:
///  Refuses a runtime directory whose socket path is longer than one socket address can be.
/// Why:
///   The address is the runtime directory,
/// the folder of the sockets,
/// the instance name and `.sock`,
/// and the kernel refuses a name of `SOCKET_ADDRESS_BYTES` bytes or more when the manager binds it,
/// which leaves the unit failed and nothing listening under any path this command could use.
/// The count is made here because the runtime directory is the only part a person can shorten,
/// so the remedy names how long it may be.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseAPathNoSocketAddressCanCarry(host) // throws when XDG_RUNTIME_DIR is too long
/// ```
pub(crate) fn refuse_a_path_no_socket_address_can_carry(host: &ProvisionHost) -> Result<(), ClientFailure> {
    let path: PathBuf = socket_path(host.runtime_dir.as_path(), host.instance.as_str());
    let length: usize = path.as_os_str().len();
    if length <= LONGEST_SOCKET_PATH {
        return Ok(());
    }
    return Err(ClientFailure::SocketPathTooLong {
        path: path.display().to_string(),
        length,
        longest: LONGEST_SOCKET_PATH,
        longest_runtime: longest_runtime_directory(host.instance.as_str()),
    });
}

/// What:
///  Refuses a service the manager made listen somewhere this command does not look.
/// Why:
///   The socket unit names its address with `%t`,
/// which the manager reads as its own runtime directory,
/// while this command connects under the runtime directory it was given.
/// When the two differ (a commit run from a unit,
/// a container or a script with a runtime directory of its own,
/// against the login session's manager)
/// the manager starts a service this command can never reach,
/// and the wait runs out although everything is working.
/// The manager's `Listen` is the fact that decides it;
/// an answer that cannot be read,
/// or a manager that holds no such unit yet,
/// proves nothing,
/// so the wait goes on as it did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseAServiceThatListensElsewhere(host, manager) // throws with both addresses named
/// ```
pub(crate) fn refuse_a_service_that_listens_elsewhere(host: &ProvisionHost, manager: &mut dyn Manager) -> Result<(), ClientFailure> {
    let looked_for: PathBuf = socket_path(host.runtime_dir.as_path(), host.instance.as_str());
    let asked: Result<Option<Vec<String>>, CallError> = manager.listen(socket_unit_instance(host.instance.as_str()).as_str());
    let addresses: Vec<String> = match asked {
        Ok(Some(addresses)) => addresses,
        // A manager that holds no unit yet, or cannot read the property, proves nothing either way.
        _nothing_said => return Ok(()),
    };
    // One of the addresses the manager made is the one this command waits at, which is the arrangement every healthy commit
    //  has; the walk is written out because the closure the linter refuses would capture `looked_for` for nothing.
    for address in &addresses {
        if Path::new(address.as_str()) == looked_for.as_path() {
            return Ok(());
        }
    }
    let Some(made) = addresses.first() else { return Ok(()) };
    let manager_runtime: PathBuf = runtime_directory_of_an_address(Path::new(made.as_str()));
    return Err(ClientFailure::ServiceElsewhere {
        made: made.clone(),
        looked_for: looked_for.display().to_string(),
        manager_runtime: manager_runtime.display().to_string(),
        client_runtime: host.runtime_dir.display().to_string(),
    });
}

/// What:
///  The runtime directory an address stands in,
/// as the manager made it.
/// Why:
///   The unit names `<runtime directory>/<folder>/<instance>.sock`,
/// so the third part from the end is the directory the manager read `%t` as,
/// and naming it is what lets a person point `XDG_RUNTIME_DIR` at it.
/// An address of another shape falls back to the directory that holds it,
/// which is still true of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runtimeDirectoryOfAnAddress('/run/user/1000/monochromatic-git-policy-cli/3f9a.sock') // '/run/user/1000'
/// ```
pub fn runtime_directory_of_an_address(address: &Path) -> PathBuf {
    let Some(folder) = address.parent() else { return address.to_path_buf() };
    let Some(runtime) = folder.parent() else { return address.to_path_buf() };
    return runtime.to_path_buf();
}

/// What:
///  The operating system's words for a failed file operation,
/// without the error number they end with on Unix.
/// Why:
///   `std::io::Error` prints `Permission denied (os error 13)`,
/// and the number in brackets is an error number in implementation vocabulary,
/// which the wrapper's message rules keep out of what a person reads.
/// The words before it are the operating system's own and are what the person acts on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inPlainWords(new Error('Permission denied (os error 13)')) // 'Permission denied'
/// ```
pub fn in_plain_words(error: &std::io::Error) -> String {
    let said: String = error.to_string();
    return match said.find(" (os error ") {
        Some(at) => said.get(..at).unwrap_or(said.as_str()).to_string(),
        None => said,
    };
}

/// Refusals that end a provision before the wait stay out of the release executable.
#[cfg(test)]
#[path = "service_provision_refusals_before_the_wait_tests.rs"]
mod tests;
