//! What:
//!  The service side of the lane lifecycle, behind the `--lane <dir>` switch of `cli-git-service`: a doorbell clients ring, a startup sweep,
//!  and a loop that runs the syncer's pass when it is rung and at least once per interval.
//! Why:
//!  Plan step 3. A client that finds a service running rings it and creates nothing else; a unit that died after its execution (the manager
//!  confirms an execution, not a sweep) leaves its receipts to the next pass, which is this loop's periodic one, so nothing waits for an
//!  unbounded next start. The doorbell is an abstract-namespace datagram socket named by the lane's absolute path: no file to clean up, no
//!  stale socket after a crash, and a send to a name nobody bound fails at once, which is the answer "no service". The switch is the flag:
//!  without it the service touches no lane, which is how the shipped unit files run it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // ringRunningService(lane): send a datagram to the lane's abstract name; refused means nobody answers
//! // runLaneService(lane, interval): startupSweep; loop { recv(interval) -> syncerPass }
//! ```

use super::commit_lane_lifecycle::{startup_sweep, syncer_pass};
use std::io;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixDatagram};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The prefix of the doorbell's abstract name.
const RING_PREFIX: &str = "cli-git-lane-ring-";

/// The interval between passes when nobody rings. A unit that died after its execution leaves its receipts to the next pass, so this is how
/// long such a receipt can wait for a running service. It is the service start ceiling (one minute; measured basis there: the slowest of 52
/// cold starts took 1,041 ms); no workload gives a rate of dead units, so the value is a pick with the owner's veto open, and `--lane-interval-millis`
/// overrides it for a test.
pub const SYNC_INTERVAL: Duration = crate::wait_ceilings::START_CEILING;

/// The abstract name of the doorbell of `lane` (its absolute path, hashed).
fn ring_address(lane: &Path) -> io::Result<SocketAddr> {
    let absolute: PathBuf = std::path::absolute(lane)?;
    let name: String = format!("{}{}", RING_PREFIX, super::commit_flusher_unit::unit_name(&absolute, 0));
    return SocketAddr::from_abstract_name(name.as_bytes());
}

/// Binds the doorbell of `lane`; an error when a service already holds it.
pub fn bind_ring(lane: &Path) -> io::Result<UnixDatagram> {
    return UnixDatagram::bind_addr(&ring_address(lane)?);
}

/// Rings the doorbell of `lane`: `true` when a service answered the socket, `false` when nobody is bound to it.
pub fn ring_running_service(lane: &Path) -> io::Result<bool> {
    let sender: UnixDatagram = UnixDatagram::unbound()?;
    return match sender.send_to_addr(b"ring", &ring_address(lane)?) {
        Ok(_) => Ok(true),
        Err(error) if matches!(error.kind(), io::ErrorKind::ConnectionRefused | io::ErrorKind::NotFound) => Ok(false),
        Err(error) => Err(error),
    };
}

/// What:
///  Runs the lane service for `lane` until the process ends: binds the doorbell, sweeps the lane once, then waits for a ring or `interval`
///  and runs the syncer's pass, again and again. Errors are logged and the loop goes on: a pass that fails leaves its record for the next.
/// Why:
///  The loop is the failure owner of a flusher unit that died after its execution, and the owner of every receipt a ring announced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runLaneService(lane: string, interval: number): never;
/// ```
pub fn run_lane_service(lane: &Path, interval: Duration) {
    let ring: UnixDatagram = match bind_ring(lane) {
        Ok(ring) => ring,
        Err(error) => {
            crate::service_error!("the lane {} has a service already, or its doorbell cannot be bound: {error}", lane.display());
            return;
        }
    };
    if let Err(error) = ring.set_read_timeout(Some(interval)) {
        crate::service_error!("the doorbell of {} has no timeout: {error}", lane.display());
        return;
    }
    let state: PathBuf = lane.join("state");
    match startup_sweep(lane, &state) {
        Ok(recovery) => {
            if let Some(reason) = &recovery.state_error {
                crate::service_error!("the state log of {} is not readable: {reason}", lane.display());
            }
        }
        Err(error) => crate::service_error!("the startup sweep of {} failed: {error:?}", lane.display()),
    }
    let mut buffer: [u8; 16] = [0; 16];
    loop {
        // A ring and a timeout both mean "run a pass"; any other receive error is logged and the pass runs anyway.
        if let Err(error) = ring.recv(&mut buffer)
            && !matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut)
        {
            crate::service_error!("the doorbell of {} failed: {error}", lane.display());
        }
        if let Err(error) = syncer_pass(lane) {
            crate::service_error!("a pass over {} failed: {error:?}", lane.display());
        }
    }
}
