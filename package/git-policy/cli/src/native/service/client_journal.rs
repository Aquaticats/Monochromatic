//! What:
//!  One entry from the wrapper a person ran to the journal, as a datagram on the native socket, sent without waiting and
//!  never failing the command.
//! Why:
//!  The wrapper is no service: its standard error is the person's terminal or an agent's capture, which is gone when the
//!  command ends. A decision the person was told about once (an add that took the slow way and why) is then recoverable
//!  afterwards only from the journal (`journalctl --identifier=monochromatic-git-policy-cli`). The socket is unconnected and non-blocking, so a
//!  journal that is absent, restarting or full costs the command nothing; the entry is then only in the debug log.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! try { socket.send(`PRIORITY=4\nSYSLOG_IDENTIFIER=monochromatic-git-policy-cli\nMESSAGE=${message}\n`) } catch { /* nothing waits for the journal */ }
//! ```

use crate::diagnostic_log::debug;
use crate::journald_native::{CLIENT_IDENTIFIER, Priority, TEST_SOCKET_VARIABLE, Target, datagram_as};
use std::os::unix::net::UnixDatagram;

/// Tag used for diagnostics from this module.
const TAG: &str = "clientJournal";

/// What:
///  Where the entry goes: the journal's socket, or the stand-in a test names; a unit test that names none reaches nothing.
/// Why:
///   A control must never write into the host's journal, and the variable is the only way a test says where its entry may go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const target = process.env.CLI_GIT_TEST_ONLY_JOURNAL_SOCKET ?? (isUnitTest ? undefined : '/run/systemd/journal/socket');
/// ```
fn target() -> Option<Target> {
    return match std::env::var(TEST_SOCKET_VARIABLE) {
        Ok(text) => Some(Target::parse(text.as_str())),
        Err(_) if cfg!(test) => None,
        Err(_) => Some(Target::journal()),
    };
}

/// What:
///  Send one entry to the journal; whether the socket took it.
/// Why:
///   The command that sends it goes on whatever the journal does; the answer is for a test and for the debug log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sent = recordEntry({ priority: 'warning', message, fields: [['CLI_GIT_WORKTREE', path]] });
/// ```
pub fn record_entry(priority: Priority, message: &str, fields: &[(&str, String)]) -> bool {
    return record_entry_as(CLIENT_IDENTIFIER, priority, message, fields);
}

/// What:
///  Send one entry under the identifier of the sender: the wrapper's, or the service's when the code runs inside the service.
/// Why:
///   An entry belongs next to the others of the program that sent it (`journalctl --identifier=cli-git-service`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sent = recordEntryAs('cli-git-service', { priority: 'warning', message, fields });
/// ```
pub fn record_entry_as(identifier: &str, priority: Priority, message: &str, fields: &[(&str, String)]) -> bool {
    let Some(found) = target() else { return false };
    return send_entry_as(&found, identifier, priority, message, fields);
}

/// What:
///  Send one entry to `target`; whether the socket took it.
/// Why:
///   Split from the choice of target so a control names its own receiver without touching the process environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sent = sendEntry(target, 'warning', message, fields);
/// ```
pub fn send_entry(target: &Target, priority: Priority, message: &str, fields: &[(&str, String)]) -> bool {
    return send_entry_as(target, CLIENT_IDENTIFIER, priority, message, fields);
}

/// What:
///  Send one entry to `target` under `identifier`; whether the socket took it.
/// Why:
///   The same send for the wrapper and for code that runs inside the service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sent = sendEntryAs(target, 'cli-git-service', 'warning', message, fields);
/// ```
pub fn send_entry_as(target: &Target, identifier: &str, priority: Priority, message: &str, fields: &[(&str, String)]) -> bool {
    let Some(address) = target.address() else {
        debug(TAG, format!("the journal address {target:?} does not fit a socket name; no entry is sent").as_str());
        return false;
    };
    let socket: UnixDatagram = match UnixDatagram::unbound() {
        Ok(opened) => opened,
        Err(error) => {
            debug(TAG, format!("no socket for the journal entry ({error})").as_str());
            return false;
        }
    };
    if let Err(error) = socket.set_nonblocking(true) {
        debug(TAG, format!("the journal socket cannot be made non-blocking ({error}); no entry is sent").as_str());
        return false;
    }
    return match socket.send_to_addr(datagram_as(identifier, priority, message, fields).as_slice(), &address) {
        Ok(_) => true,
        Err(error) => {
            debug(TAG, format!("the journal took no entry ({error})").as_str());
            false
        }
    };
}

/// Journal entry controls stay out of the release executable.
#[cfg(test)]
#[path = "client_journal_tests.rs"]
mod tests;
