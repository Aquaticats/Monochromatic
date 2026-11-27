//! What:
//!  Whether provisioning has anything left to do:
//!  the installed templates are the ones this wrapper would write,
//!  the worktree's socket is there,
//!  and nothing of the earlier releases is left listening,
//!  all read without touching the session bus.
//! Why:
//!   The manager starts whatever the link names,
//!  so its text alone says nothing about which program the service template names;
//!  a few small reads and two `stat` calls decide,
//!  and the bus is not touched when they agree,
//!  because a manager told about a new text re-reads every unit it holds,
//!  which stalls every other client of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! inPlace(host) // false: write, unload, start
//! ```

use crate::earlier_unix_sockets_that_listen::earlier_socket_of_instance_listens;
use crate::service_address_of_a_worktree::{
    socket_address_of,
    the_address_is_in,
};
use crate::service_instance::{
    LEGACY_SOCKET_RELATIVE,
    earlier_socket_path,
};
use crate::service_provision_host_and_manager::{
    LEGACY_SERVICE_UNIT_NAME,
    LEGACY_SOCKET_UNIT_NAME,
    ProvisionHost,
    runtime_unit_dir,
};
use crate::service_provision_loaded_unit_texts::remembers as manager_was_told;
use crate::service_provision_the_unit_texts::unit_texts;
use crate::service_provision_unit_file_ownership_and_writes::own_unit_file_in_runtime_or_user_directory;
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  Whether the installed templates are the ones this wrapper would write,
/// the worktree's socket is there,
/// and nothing of
///  the single service of the earlier releases is left,
/// and no socket of the earlier release for this worktree listens.
/// Why:
///   The socket belongs to the manager and outlives every upgrade,
/// so its presence says nothing about which program the
///  service template names;
/// the text does.
/// A few small reads and two `stat` calls decide,
/// and the bus is not touched when
///  they agree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inPlace(host) // false: write, unload, start
/// ```
pub fn in_place(host: &ProvisionHost) -> bool {
    return in_place_with(host, Path::new("/proc"));
}

/// `in_place` with the process table to read the listening sockets from.
pub fn in_place_with(
    host: &ProvisionHost,
    proc_root: &Path,
) -> bool {
    let directory: PathBuf = runtime_unit_dir(host.runtime_dir.as_path());
    let texts: [(&'static str, String); 2] = unit_texts(host);
    // Every unit file holds exactly the text the wrapper would write.
    let mut all_current: bool = true;
    for (name, text) in &texts {
        if !matches!(std::fs::read_to_string(directory.join(name)), Ok(installed) if installed == *text)
        {
            all_current = false;
            break;
        }
    }
    // What: The manager last read these very texts for this instance.
    // Why:  A file that matches says what this release would write, not what the manager loaded. A process killed between
    //       writing a template and making the manager read it leaves exactly that, and every later commit would then keep
    //       reaching the service of the release that was replaced
    //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, the interrupted upgrade). One
    //       small read of the record beside the notes decides it. The record is per instance, because the manager holds one
    //       instance at a time: another worktree's freshly read units say nothing about this one's.
    let manager_holds: bool = manager_was_told(
        host.runtime_dir.as_path(),
        host.instance.as_str(),
        texts.as_slice(),
    );
    // A file of an earlier name that something else wrote is not the wrapper's to retire, and does not keep it from being in place.
    let mut earlier_left: bool = false;
    for name in [LEGACY_SOCKET_UNIT_NAME, LEGACY_SERVICE_UNIT_NAME] {
        if own_unit_file_in_runtime_or_user_directory(
            directory.as_path(),
            host.user_unit_dir.as_deref(),
            name,
        ) {
            earlier_left = true;
            break;
        }
    }
    earlier_left = earlier_left || host.runtime_dir.join(LEGACY_SOCKET_RELATIVE).exists();
    if !all_current
        || !manager_holds
        || earlier_left
        || !the_address_is_in(
            proc_root,
            &socket_address_of(host.runtime_dir.as_path(), host.instance.as_str()),
        )
    {
        return false;
    }
    // The earlier release's socket of this worktree: a listener there starts the earlier helper on a connection, so the stop
    //  has to be tried (again). The socket's file stays after the unit stops and says nothing; the table of sockets is read
    //  only when that file is there, which a worktree the earlier release never served never has.
    let earlier_file: PathBuf =
        earlier_socket_path(host.runtime_dir.as_path(), host.instance.as_str());
    return !(earlier_file.exists()
        && earlier_socket_of_instance_listens(proc_root, host.instance.as_str()));
}
