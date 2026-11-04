//! What:
//!  The service side of the lane lifecycle, behind the `--lane <dir>` switch of `cli-git-service`: a doorbell clients ring, a startup sweep,
//!  and a loop that runs the syncer's pass when it is rung and when a pass is due.
//! Why:
//!  Plan step 3. A client that finds a service running rings it and creates nothing else; a unit that died after its execution (the manager
//!  confirms an execution, not a sweep) leaves its receipts to the next pass, which is this loop's periodic one, so a receipt waits no longer
//!  than a pass interval for a service that is alive (a service that has died, or never started, serves nothing: the next client's ring finds
//!  nobody and launches a flusher, and the next start sweeps). The doorbell is a datagram socket in the lane directory, `service.ring`,
//!  reached through the lane directory's descriptor (a socket path is limited to a little over a hundred bytes, a lane path need not be),
//!  so that only who may write the lane directory may bind or replace it (an abstract name belongs to the whole network namespace and to
//!  anyone). One service owns a lane at a time, by an exclusive lock on `service.lock`. A send never blocks: a full queue means rings are
//!  waiting that one pass will serve. The switch is the flag: without it the service touches no lane, which is how the shipped unit files run it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // ringRunningService(lane): send a datagram to lane/service.ring without blocking; refused means nobody answers
//! // startLaneService(lane, interval): lock service.lock; bind the doorbell; thread { startupSweep; loop { recv(until due) -> syncerPass } }
//! ```

use super::commit_lane_lifecycle::{startup_sweep, syncer_pass};
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{FileTypeExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The doorbell's file name in the lane directory.
pub const RING_FILE: &str = "service.ring";

/// The lock file that makes one service the owner of a lane.
pub const SERVICE_LOCK: &str = "service.lock";

/// The interval between passes when nobody rings. A unit that died after its execution leaves its receipts to the next pass, so this is how
/// long such a receipt can wait for a running service. It is the service start ceiling (one minute; measured basis there: the slowest of 52
/// cold starts took 1,041 ms); no workload gives a rate of dead units, so the value is a pick with the owner's veto open, and
/// `--lane-interval-millis` overrides it for a test.
pub const SYNC_INTERVAL: Duration = crate::wait_ceilings::START_CEILING;

/// Opens the lane directory itself (not through a link) so that the doorbell is reached by a short path that cannot be redirected.
fn open_lane_directory(lane: &Path) -> io::Result<File> {
    return OpenOptions::new().read(true).custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW).open(lane);
}

/// The path of the doorbell through the descriptor of the lane directory.
fn ring_path(directory: &File) -> PathBuf {
    return PathBuf::from(format!("/proc/self/fd/{}/{}", directory.as_raw_fd(), RING_FILE));
}

/// Binds the doorbell of `lane`. A socket file left by a service that died is replaced (nobody answers a send to it); one that answers is a
/// service already running, and that is an error; anything else at the name is refused.
pub fn bind_ring(lane: &Path) -> io::Result<UnixDatagram> {
    let directory: File = open_lane_directory(lane)?;
    let path: PathBuf = ring_path(&directory);
    match std::fs::symlink_metadata(&path) {
        Ok(found) if found.file_type().is_socket() => {
            let probe: UnixDatagram = UnixDatagram::unbound()?;
            probe.set_nonblocking(true)?;
            match probe.send_to(b"ring", &path) {
                Ok(_) => return Err(io::Error::new(io::ErrorKind::AddrInUse, "a service already answers at the doorbell")),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Err(io::Error::new(io::ErrorKind::AddrInUse, "a service already answers at the doorbell")),
                Err(error) if error.kind() == io::ErrorKind::ConnectionRefused => std::fs::remove_file(&path)?,
                Err(error) => return Err(error),
            }
        }
        Ok(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "the doorbell name is held by something that is not a socket")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let ring: UnixDatagram = UnixDatagram::bind(&path)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    return Ok(ring);
}

/// Rings the doorbell of `lane`: `true` when a socket answered (or its queue is full of rings that one pass will serve), `false` when nobody
/// is bound to it. Never blocks.
pub fn ring_running_service(lane: &Path) -> io::Result<bool> {
    let directory: File = match open_lane_directory(lane) {
        Ok(directory) => directory,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    let sender: UnixDatagram = UnixDatagram::unbound()?;
    // Never blocking: a service that does not read leaves its datagram queue full, and a blocking send would wait for it for ever. A full
    // queue is itself the answer: rings are waiting that a pass will serve, and a pass serves every receipt, so one more ring adds nothing.
    sender.set_nonblocking(true)?;
    return match sender.send_to(b"ring", ring_path(&directory)) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(true),
        Err(error) if matches!(error.kind(), io::ErrorKind::ConnectionRefused | io::ErrorKind::NotFound) => Ok(false),
        Err(error) => Err(error),
    };
}

/// Takes the lock that makes this process the lane's service, without waiting; the file is kept open for the life of the service.
fn lock_service(lane: &Path) -> io::Result<File> {
    let directory: File = open_lane_directory(lane)?;
    let path: PathBuf = PathBuf::from(format!("/proc/self/fd/{}/{}", directory.as_raw_fd(), SERVICE_LOCK));
    let file: File = OpenOptions::new().read(true).write(true).create(true).truncate(false).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(&path)?;
    if !file.metadata()?.file_type().is_file() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the service lock is not a regular file"));
    }
    return match file.try_lock() {
        Ok(()) => Ok(file),
        Err(std::fs::TryLockError::WouldBlock) => Err(io::Error::new(io::ErrorKind::AddrInUse, "another service owns the lane")),
        Err(std::fs::TryLockError::Error(error)) => Err(error),
    };
}

/// What:
///  Starts the lane service for `lane`: refuses a zero interval, takes the lane's service lock, binds the doorbell (an error when a service
///  already holds either, so a second service on one lane fails visibly instead of running without a lane), and runs the loop on a thread of
///  its own, which ends with the process.
/// Why:
///  The caller (the service program) learns of an initialization failure before it announces readiness. The loop sweeps the lane once, then
///  waits for a ring or until a pass is due and runs the syncer's pass, again and again. A pass is due `interval` after the start of the
///  previous one, so passes start at most `interval` apart while they take less than `interval`; a pass that takes longer is followed at once
///  by the next. Errors in the loop are logged and the loop goes on: a pass that fails leaves its record for the next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startLaneService(lane: string, interval: number): void; // throws when the interval is zero or the lane has a service
/// ```
pub fn start_lane_service(lane: &Path, interval: Duration) -> io::Result<()> {
    if interval.is_zero() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the interval between passes is zero"));
    }
    let lock: File = lock_service(lane)?;
    let ring: UnixDatagram = bind_ring(lane).map_err(|error| { return io::Error::new(error.kind(), format!("the doorbell of the lane {} cannot be bound: {}", lane.display(), error)); })?;
    crate::service_log!("lane doorbell bound {}", lane.display());
    let owned: PathBuf = lane.to_path_buf();
    std::thread::spawn(move || {
        // The lock lives as long as the loop.
        let _lock: File = lock;
        run_lane_service(&owned, &ring, interval);
    });
    return Ok(());
}

/// One pass of the loop: the syncer's pass, then the interrupted disposals the person authorized (a writer that was busy at an earlier try
/// is tried again here, on every pass).
fn pass(lane: &Path, state: &Path) {
    match syncer_pass(lane) {
        Ok(_) => crate::service_log!("lane pass complete {}", lane.display()),
        Err(error) => crate::service_error!("a pass over {} failed: {error:?}", lane.display()),
    }
    if let Err(error) = super::commit_disposal::resume_disposals(lane, state) {
        crate::service_error!("the interrupted disposals of {} could not be finished: {error}", lane.display());
    }
}

/// The loop of the lane service.
fn run_lane_service(lane: &Path, ring: &UnixDatagram, interval: Duration) {
    let state: PathBuf = lane.join("state");
    match startup_sweep(lane, &state) {
        Ok(recovery) => {
            if let Some(reason) = &recovery.state_error {
                crate::service_error!("the state log of {} is not readable: {reason}", lane.display());
            }
        }
        Err(error) => crate::service_error!("the startup sweep of {} failed: {error:?}", lane.display()),
    }
    crate::service_log!("lane startup complete {}", lane.display());
    let mut buffer: [u8; 16] = [0; 16];
    let mut due: Instant = Instant::now() + interval;
    loop {
        // A ring and a due pass both mean "run a pass"; any other receive error is logged and the pass runs anyway. The timeout is the time
        // left until the pass is due (at least a millisecond, since a zero timeout is refused by the socket).
        let wait: Duration = due.saturating_duration_since(Instant::now()).max(Duration::from_millis(1));
        if let Err(error) = ring.set_read_timeout(Some(wait)) {
            crate::service_error!("the doorbell of {} has no timeout: {error}", lane.display());
            return;
        }
        if let Err(error) = ring.recv(&mut buffer)
            && !matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut)
        {
            crate::service_error!("the doorbell of {} failed: {error}", lane.display());
        }
        let started: Instant = Instant::now();
        pass(lane, &state);
        due = started + interval;
    }
}
