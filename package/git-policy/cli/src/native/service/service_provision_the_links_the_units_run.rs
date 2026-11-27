//! What:
//!  The links the unit files run:
//!  the one this release points at itself,
//!  the one the earlier releases named,
//!  and the one write both come to.
//! Why:
//!   The manager starts whatever the link names at the moment a start is needed,
//!  which after this commit may be hours away (the service exits after three idle hours);
//!  a link that cannot be written does not end the commit by itself:
//!  the unit files are still written,
//!  one line names the link,
//!  and the next commit points it again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! pointProgramLink(host) // the link names host.serviceBinary, or a line says what it still names
//! ```

use crate::service_instance::{
    earlier_releases_program_pointer,
    program_link,
};
use crate::service_provision_host_and_manager::ProvisionHost;
use crate::service_provision_refusals_before_the_wait::in_plain_words;
use crate::service_provision_unit_file_ownership_and_writes::link_atomically;
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  Points the per-user link the service unit runs at `host.service_binary`,
/// unless it already names it.
/// Why:
///   The manager starts whatever the link names at the moment of the start,
/// so a commit that provisions after an upgrade makes
///  the next start run its own release,
/// and it does so by writing one link,
/// not by changing the text the manager holds.
/// Every
///  commit that reaches the service points it,
/// the fast path included,
/// because a service the manager starts later
///  (after its idle exit,
/// or after a restart) must run the release of the commit that needed it.
/// A link that cannot
///  be written does not end the commit by itself:
/// the unit files are still written,
/// the commit goes on,
/// one line
///  names the link,
/// and the next commit points it again.
/// Whether the commit lands is then decided by whatever the link
///  still names,
/// because the unit starts the link and nothing else.
/// A program of this release
///  (which a link a build of the same release left at another path is)
/// greets as this release,
/// and the
///  commit lands through it.
/// A program of another release answers as that release,
/// and the client's own repair for it
///  (`service_client_reach_service.rs`) ends the commit with its message when it will not step aside.
/// A path that cannot
///  be executed at all (a directory,
/// an executable an upgrade removed) starts nothing,
/// and the commit ends with "The
///  commit service did not start".
/// Nothing smaller is safe:
/// the commit service is the only writer of a commit,
/// this release will not
///  hand one to another release's service,
/// and the manager reads the link from the text it holds,
/// which this release cannot
///  change without writing that text.
/// The remedy when it repeats is the runtime directory:
/// the link must sit where this account can
///  write,
/// which is what the line and the failure that follows both name.
/// The link the releases before this one named is pointed too
///  (`service_instance.rs`,
/// `EARLIER_RELEASES_PROGRAM_POINTER`),
/// at the same cost:
/// an instance whose loaded text is
///  theirs starts this release from it,
/// instead of failing at the EXEC step because the install removed the release that
///  program belonged to.
/// An older command that meets this release's service still asks it to retire,
/// and the service stops
///  its own socket when it is retired before it has served anything
/// (`service_server.rs`),
/// so that cannot burn the manager's
///  start limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pointProgramLink(host) // the link names host.serviceBinary, or a line says what it still names
/// ```
pub(crate) fn point_program_link(host: &ProvisionHost) {
    point_a_link(
        program_link(host.runtime_dir.as_path()).as_path(),
        host,
        true,
    );
}

/// What:
///  Points the path the releases before this release named in every unit text they wrote at this release's executable,
///  making the notes folder when it is not there yet.
/// Why:
///   The manager keeps starting such an instance from the text it loaded,
/// and the install that replaced that release removed
///  the directory the link named,
/// so without this its next start fails at the EXEC step,
/// is retried to the manager's start limit,
///  and leaves the unit failed until a command in that worktree repairs it
/// (`doc/troubleshooting/`,
/// the install of
///  `0.6.1-alpha.20`).
/// It is pointed after every refusal of this provisioning,
/// because a refused one writes nothing,
/// and
///  the folder it sits in is made here for the same reason:
/// it is the folder the worktree note and the record are written
///  into just after.
/// A link that cannot be written only warns:
/// no commit waits on the instance it serves,
/// and the
///  next commit points it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pointEarlierReleasesPointer(host) // <notes>/service-program -> host.serviceBinary
/// ```
pub(crate) fn point_earlier_releases_pointer(host: &ProvisionHost) {
    let pointer: PathBuf = earlier_releases_program_pointer(host.runtime_dir.as_path());
    let Some(folder) = pointer.parent() else {
        return;
    };
    if let Err(error) = crate::private_storage::create_private_directory_all(folder) {
        crate::service_warn!(
            "{} cannot be made ({}), so a service the manager loaded under an earlier release cannot be started until the next commit makes it; this commit goes on",
            folder.display(),
            in_plain_words(&error)
        );
        return;
    }
    point_a_link(pointer.as_path(), host, false);
}

/// What:
///  Points one link at the running executable,
/// unless it already names it,
/// and says in the log when it cannot.
/// Why:
///   Both links cost one read and,
/// when the program moved,
/// one rename;
/// neither asks the manager anything,
/// and neither
///  ends a commit,
/// which is why the failure is a line and not a `ClientFailure`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pointALink(link, host, namesThisRelease) // link -> host.serviceBinary, or one line
/// ```
fn point_a_link(
    link: &Path,
    host: &ProvisionHost,
    names_this_release: bool,
) {
    if matches!(std::fs::read_link(link), Ok(current) if current == host.service_binary) {
        return;
    }
    let Err(error) = link_atomically(link, host.service_binary.as_path()) else {
        return;
    };
    // What: The consequence named depends on which link it is.
    // Why:  The link this release's own units name decides what this commit's service runs; the one an earlier release's
    //       loaded texts name decides whether such an instance can start at all, and this release's next commit is what
    //       points it again.
    let consequence: &str = if names_this_release {
        "so this commit's service is started from whatever the link still names; the units are still written and this commit goes on, and it lands if that program is this release, and ends with that program's own message if it is another release or none at all; set XDG_RUNTIME_DIR to a runtime directory this account can write, and the next commit points the link again"
    } else {
        "so another worktree's service the manager loaded under an earlier release cannot start until the next commit points it; this commit goes on, and set XDG_RUNTIME_DIR to a runtime directory this account can write"
    };
    crate::service_warn!(
        "{} cannot be pointed at {} ({}), {consequence}",
        link.display(),
        host.service_binary.display(),
        in_plain_words(&error)
    );
}
