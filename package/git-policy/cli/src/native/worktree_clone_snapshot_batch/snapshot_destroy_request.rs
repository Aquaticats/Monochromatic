//! What:
//!  The one `ZFS_IOC_DESTROY_SNAPS` request a verified snapshot set becomes,
//! its exact encoded input size,
//! and the proof that it names no `defer` key.
//! Why:
//!  The kernel reads a snapshot's deferral from the presence of the `defer`
//!  key,
//! never from its value:
//! the accepted pair is a bare boolean,
//! the handler asks
//!  only whether the key exists,
//! and a deferred destroy would mark a snapshot with
//!  dependent clones instead of refusing it
//! (`module/zfs/zfs_ioctl.c:4112` to `:4132` and
//!  `lib/libzfs_core/libzfs_core.c:475` to `:476`,
//! OpenZFS `zfs-2.4.4`;
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery",
//! step 3).
//!  The binding's constructor omits the key,
//! and this file builds every request
//!  through it so the bytes the planning measures are the bytes the device
//!  sends.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const request = destroyRequest(pool, names); // snaps list, no defer key
//! const bytes = encodedRequestBytes(pool, names);
//! const safe = carriesNoDefer(request);
//! ```

/// The request value and the binding's own snapshot-destroy constructor.
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    NvList,
    Request,
    destroy_snaps,
};

/// The refusal a set this route will not submit reports.
use super::request_refusal::BatchRefusal;

/// The one key whose presence marks a snapshot for deferred destruction.
pub const DEFER_KEY: &str = "defer";

/// What:
///  The request that destroys `names` in `pool` now,
///  as the binding packs it.
/// Why:
///  Building through the binding's constructor is what makes the `defer`
///  absence structural rather than a rule a caller could forget,
/// and the planning,
///  the guard and the device all build through this one function.
///
/// # Throws
///
/// [`BatchRefusal::Unbuildable`] when the pool name does not fit the command
/// field or a snapshot name cannot be packed.
pub fn destroy_request(
    pool: &str,
    names: &[String],
) -> Result<Request, BatchRefusal> {
    let borrowed: Vec<&str> = names
        .iter()
        .map(|name| {
            return name.as_str();
        })
        .collect();
    return destroy_snaps(pool, borrowed.as_slice()).map_err(|source| {
        return BatchRefusal::Unbuildable {
            pool: pool.to_string(),
            source,
        };
    });
}

/// What:
///  The bytes the request for `pool` and `names` would carry.
/// Why:
///  The budget is compared with this exact length,
///  which holds the list header,
///  every pair's header,
///  name,
///  padding and the terminator,
///  so the split moves only
///  when the kernel-bound bytes move.
///
/// # Throws
///
/// [`BatchRefusal::Unbuildable`] as [`destroy_request`] does.
pub fn encoded_request_bytes(
    pool: &str,
    names: &[String],
) -> Result<usize, BatchRefusal> {
    let request: Request = destroy_request(pool, names)?;
    return Ok(request.input_bytes().len());
}

/// What:
///  Whether the request's input names no `defer` key,
///  so nothing it carries can be
///  marked for deferred destruction.
/// Why:
///  The kernel reads the key at the input list's top level,
///  so this decodes the
///  request's own bytes and compares the pair names it would read.
///  A request whose input
///  cannot be decoded answers `false`,
///  because a guard on the absence of a key must
///  fail closed.
pub fn carries_no_defer(request: &Request) -> bool {
    let Ok(input) = NvList::unpack(request.input_bytes()) else {
        return false;
    };
    return input.get(DEFER_KEY).is_none();
}

/// The controls of the request and its `defer` absence.
#[cfg(test)]
#[path = "snapshot_destroy_request_tests.rs"]
mod tests;
