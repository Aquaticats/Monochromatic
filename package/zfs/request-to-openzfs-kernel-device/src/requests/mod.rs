//! What:
//!  One request to `/dev/zfs`:
//! which operation,
//! which dataset or pool,
//! the
//!  packed input list it carries,
//! the reply buffer it owns,
//! and the failure
//!  that names all of them.
//! Why:
//!  The kernel's interface is one request number and one command struct,
//! so
//!  every operation this wrapper needs becomes a constructor here rather than a
//!  call site assembling fields.
//! A constructor cannot set a field the design
//!  did not pick for that operation,
//! and the device-free tests can check every
//!  packed request without a device.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `operation.rs`:
//!   the typed request numbers,
//!   and which change state or list.
//! - `element_error.rs`:
//!   the per-name failures a write reply carries.
//! - `reply.rs`:
//!   the reply buffer,
//!   its bounded growth,
//!   and the decode of what came back.
//! - `error.rs`:
//!   the failure type carrying operation,
//!   target,
//!   error number and elements.
//! - `request.rs`:
//!   one request's owned command,
//!   input and reply buffers.
//! - `read_requests.rs`:
//!   the identity,
//!   listing,
//!   delegation and bookmark reads.
//! - `dataset_requests.rs`:
//!   set properties,
//!   destroy,
//!   promote and clone.
//! - `snapshot_requests.rs`:
//!   create snapshots and destroy an explicit snapshot set.
//! - `program_requests.rs`:
//!   the fixed root-owned program request.

/// Set properties,
/// destroy,
/// promote and clone.
mod dataset_requests;
/// The per-name failures a write reply carries.
mod element_error;
/// Why a packed list or a request is refused.
mod error;
/// The typed request numbers and their queries.
mod operation;
/// The fixed root-owned program request.
mod program_requests;
/// The identity,
/// listing,
/// delegation and bookmark reads.
mod read_requests;
/// The reply buffer,
/// its bounded growth and its decode.
mod reply;
/// One request owned command,
/// input and reply buffers.
mod request;
/// Create snapshots and destroy an explicit snapshot set.
mod snapshot_requests;

/// The four dataset writes.
pub use dataset_requests::{clone, destroy, promote, set_prop};
/// The per-name failures and their diagnostic helpers.
pub use element_error::{
    ELEMENTS_SHOWN, ElementError, MAXIMUM_ELEMENTS_KEPT, element_errors, elements_suffix,
};
/// The failure type every request returns.
pub use error::Error;
/// The typed request numbers.
pub use operation::Operation;
/// The fixed-program request and its inputs.
pub use program_requests::{ChannelProgramRequest, channel_program};
/// The five read requests.
pub use read_requests::{
    dataset_list_next, get_bookmarks, get_fsacl, objset_stats, snapshot_list_next,
};
/// The reply buffer,
/// its refusals and its error.
pub use reply::{
    INITIAL_REPLY_SIZE, MAXIMUM_REPLY_SIZE, ReplyBuffer, ReplyError, ReplyRefusal,
    ReplyRefusalReason, ReplyUse,
};
/// The request value and the two builders the packers share.
pub use request::{Request, checked_name, flag_list};
/// The two snapshot writes.
pub use snapshot_requests::{destroy_snaps, snapshot};
