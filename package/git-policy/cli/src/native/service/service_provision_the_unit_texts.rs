//! What:
//!  The two unit files of the commit service,
//!  written for one host:
//!  the socket template and the service template,
//!  with every placeholder filled in.
//! Why:
//!  The service template names a link instead of the absolute path of the release that wrote it,
//!  so its text is the same in every release,
//!  in every build of a release and in a build made in a worktree;
//!  an upgrade,
//!  a downgrade and two releases used in turn write nothing,
//!  so the manager never has to read them again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! unitTexts(host) // [['…@.socket', '…ListenStream=%t/…/%i.sock…'], ['…@.service', '…']]
//! ```

use crate::service_address_of_a_worktree::listen_stream_of;
use crate::service_instance::program_link;
use crate::service_provision_host_and_manager::{
    ProvisionHost,
    STOP_TIMEOUT_SECONDS,
};
use std::path::PathBuf;
/// The socket template's text,
/// with the marker on its first line.
pub const SOCKET_UNIT: &str = include_str!("service_units/monochromatic-git-policy-cli@.socket");

/// The socket template's placeholder for the value of its `ListenStream=`,
/// which `listen_stream_of` fills in with the address form this runtime directory takes
/// (`service_address_of_a_worktree.rs`).
pub(crate) const LISTEN_STREAM_PLACEHOLDER: &str = "@LISTEN_STREAM@";

/// The service template's text,
/// with `@SERVICE_BINARY@` and `@GIT@` still to fill in.
pub const SERVICE_UNIT: &str = include_str!("service_units/monochromatic-git-policy-cli@.service");

/// File name of the socket template.
pub const SOCKET_UNIT_NAME: &str = "monochromatic-git-policy-cli@.socket";

/// File name of the service template.
pub const SERVICE_UNIT_NAME: &str = "monochromatic-git-policy-cli@.service";

pub(crate) const BINARY_PLACEHOLDER: &str = "@SERVICE_BINARY@";
pub(crate) const GIT_PLACEHOLDER: &str = "@GIT@";
pub(crate) const STOP_TIMEOUT_PLACEHOLDER: &str = "@STOP_TIMEOUT_SECONDS@";

/// What:
///  The two unit files for a host,
/// as written.
/// Why:
///   The socket unit's `ListenStream=` is the address this runtime directory's service listens at,
/// whether that is a file under the runtime directory or an abstract name,
/// and the instance is left to the unit's own `%i`.
/// The service unit's text is the same whatever the address is,
/// and its program is the per-user link and never the absolute path of the release that happens to be running,
/// so
/// the text is the same in every release (`program_link`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unitTexts(host) // [['…@.socket', '…ListenStream=%t/…/%i.sock…'], ['…@.service', '…']]
/// ```
pub fn unit_texts(host: &ProvisionHost) -> [(&'static str, String); 2] {
    let program: PathBuf = program_link(host.runtime_dir.as_path());
    let socket: String = SOCKET_UNIT.replace(
        LISTEN_STREAM_PLACEHOLDER,
        listen_stream_of(host.runtime_dir.as_path(), host.instance.as_str()).as_str(),
    );
    let service: String = SERVICE_UNIT
        .replace(BINARY_PLACEHOLDER, program.to_string_lossy().as_ref())
        .replace(GIT_PLACEHOLDER, host.git.to_string_lossy().as_ref())
        .replace(
            STOP_TIMEOUT_PLACEHOLDER,
            STOP_TIMEOUT_SECONDS.to_string().as_str(),
        );
    return [(SOCKET_UNIT_NAME, socket), (SERVICE_UNIT_NAME, service)];
}
