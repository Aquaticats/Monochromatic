//! What:
//!  The service side of the lane lifecycle, behind the `--lane <dir>` switch of `cli-git-service`: a doorbell clients ring, a startup sweep,
//!  and a loop that runs the syncer's pass when it is rung and when a pass is due.
//! Why:
//!  Plan step 3. A client that finds a service running rings it and creates nothing else; a unit that died after its execution (the manager
//!  confirms an execution, not a sweep) leaves its receipts to the next pass, which is this loop's periodic one. Passes start at most one
//!  interval apart while each takes less than an interval; that is a spacing of pass starts and no bound on the time from a receipt to its
//!  forcing (a pass that waits on the file system takes as long as the file system takes, and a pass over receipts that keep arriving
//!  sweeps one round and is followed at once by the next). A service that has died, or never started, serves nothing: the next client's
//!  ring finds nobody and launches a flusher, and the next start sweeps. The doorbell is a datagram socket in the lane directory,
//!  `service.ring`, reached through the lane directory's descriptor (a socket path is limited to a little over a hundred bytes, a lane path
//!  need not be), so that only who may write the lane directory may bind or replace it (an abstract name belongs to the whole network
//!  namespace and to anyone). A ring is sent only to a socket that is private to the account (a link or another account's socket is not
//!  this lane's service). One service owns a lane at a time, by an exclusive lock on `service.lock`, which is also what keeps two services
//!  that meet at a stale doorbell from both starting: the lock is held for the life of the service, not only through the binding. The lane
//!  directory is checked once and every later step goes through that descriptor, so a rename of the lane or of a directory above it after
//!  the check changes nothing the service touches; the directories above the lane are not checked, and a directory above that other accounts
//!  can rename into can still change what the name leads to before the check. A send never blocks: a full queue means rings are waiting that
//!  one pass will serve. The switch is the flag: without it the service touches no lane, which is how the shipped unit files run it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // ringRunningService(lane): send a datagram to lane/service.ring without blocking; refused means nobody answers
//! // startLaneService(lane, interval): lock service.lock; bind the doorbell; thread { startupSweep; loop { recv(until due) -> syncerPass } }
//! ```

use super::commit_identity::fault_point;
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

/// The interval between passes when nobody rings. A unit that died after its execution leaves its receipts to the next pass, so this is the
/// spacing at which a running service looks for such a receipt (not a bound on the time until it is forced). It is the service start ceiling (one minute; measured basis there: the slowest of 52
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

/// Checks that what the doorbell's name leads to is this account's private socket: a socket (not a link, which `symlink_metadata` does not
/// follow, and not a file), owned by the account that runs the program, that no one else can open. A send succeeds against any socket the
/// name leads to, so a link to another lane's doorbell, or a socket another account made, would be taken for this lane's service.
fn check_doorbell(found: &std::fs::Metadata) -> io::Result<()> {
    use std::os::unix::fs::MetadataExt;
    if !found.file_type().is_socket() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the doorbell name is held by something that is not a socket"));
    }
    // SAFETY: `geteuid` takes no argument and touches no memory.
    // A second name (a hard link to another lane's doorbell) is refused: it is a private socket of the account by every other test, and it
    // leads to the service of the lane it was made in. The entry could still be replaced between this check and the send by a process
    // that may rename entries of the lane; the lane directory is private to the account, and that is the premise of the lane.
    if found.uid() != unsafe { libc::geteuid() } || found.mode() & 0o077 != 0 || found.nlink() != 1 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the doorbell is not a socket private to this account with one name"));
    }
    return Ok(());
}

/// Binds the doorbell of `lane`. A socket file left by a service that died is replaced (nobody answers a send to it); one that answers is a
/// service already running, and that is an error; anything else at the name is refused.
pub fn bind_ring(lane: &Path) -> io::Result<UnixDatagram> {
    return bind_ring_in(&open_lane_directory(lane)?);
}

/// `bind_ring` on the lane directory that is already open.
fn bind_ring_in(directory: &File) -> io::Result<UnixDatagram> {
    let path: PathBuf = ring_path(directory);
    match std::fs::symlink_metadata(&path) {
        Ok(found) => {
            check_doorbell(&found)?;
            let probe: UnixDatagram = UnixDatagram::unbound()?;
            probe.set_nonblocking(true)?;
            match probe.send_to(b"ring", &path) {
                Ok(_) => return Err(io::Error::new(io::ErrorKind::AddrInUse, "a service already answers at the doorbell")),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Err(io::Error::new(io::ErrorKind::AddrInUse, "a service already answers at the doorbell")),
                Err(error) if error.kind() == io::ErrorKind::ConnectionRefused => {
                    // The probe was refused: the socket looks stale. Between this and the removal another service can bind a live one.
                    fault_point("service:before-stale-removal");
                    std::fs::remove_file(&path)?;
                }
                Err(error) => return Err(error),
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let ring: UnixDatagram = UnixDatagram::bind(&path)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    return Ok(ring);
}

/// Rings the doorbell of `lane`: `true` when a socket answered (or its queue is full of rings that one pass will serve), `false` when nobody
/// is bound to it. Never blocks. A doorbell that is not this account's private socket (a link, a file, another account's socket) is an
/// error, which the client reads as "nobody answered" and launches the flusher.
pub fn ring_running_service(lane: &Path) -> io::Result<bool> {
    let directory: File = match open_lane_directory(lane) {
        Ok(directory) => directory,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    // The entry is checked through the descriptor of the directory that is then used for the send, without following a link.
    match std::fs::symlink_metadata(ring_path(&directory)) {
        Ok(found) => check_doorbell(&found)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    }
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
pub fn lock_service(lane: &Path) -> io::Result<File> {
    return lock_service_in(&open_lane_directory(lane)?);
}

/// `lock_service` on the lane directory that is already open. The lock file is created private to the account (0600) and an existing one is
/// checked: another account that could open it for reading could take its exclusive lock and keep every service out, and a hard link to it
/// from another lane would couple two lanes.
fn lock_service_in(directory: &File) -> io::Result<File> {
    use std::os::unix::fs::MetadataExt;
    let path: PathBuf = PathBuf::from(format!("/proc/self/fd/{}/{}", directory.as_raw_fd(), SERVICE_LOCK));
    let file: File = OpenOptions::new().read(true).write(true).create(true).truncate(false).mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(&path)?;
    let found = file.metadata()?;
    if !found.file_type().is_file() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{SERVICE_LOCK} is not a regular file")));
    }
    // SAFETY: `geteuid` takes no argument and touches no memory.
    if found.uid() != unsafe { libc::geteuid() } || found.mode() & 0o077 != 0 || found.nlink() != 1 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{SERVICE_LOCK} is not private to this account or has another name")));
    }
    return match file.try_lock() {
        Ok(()) => Ok(file),
        Err(std::fs::TryLockError::WouldBlock) => Err(io::Error::new(io::ErrorKind::AddrInUse, "another service owns the lane")),
        Err(std::fs::TryLockError::Error(error)) => Err(error),
    };
}

/// When the next pass is due: `interval` after the start of the pass that just ended, and at once when that pass left receipts that
/// arrived while it swept.
pub fn next_due(started: Instant, interval: Duration, more: bool) -> Instant {
    return if more { Instant::now() } else { started + interval };
}

/// What:
///  Starts the lane service for `lane`: refuses a zero interval, opens and checks the lane directory once (`open_lane`), takes the lane's
///  service lock, binds the doorbell (an error when a service already holds either, so a second service on one lane fails visibly instead
///  of running without a lane), and runs the loop on a thread of its own, which ends with the process.
/// Why:
///  The caller (the service program) learns of an initialization failure before it announces readiness. The directory that was checked is the
///  one the lock, the doorbell and every pass use, through its descriptor, so a rename of the lane or of a directory above it after the
///  check changes nothing the service touches. The loop sweeps the lane once, then waits for a ring or until a pass is due and runs the
///  syncer's pass, again and again. A pass is due `interval` after the start of the previous one, so passes start at most `interval` apart
///  while they take less than `interval`; a pass that left newly arrived receipts is followed at once by the next. Errors in the loop are
///  logged and the loop goes on: a pass that fails leaves its record for the next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startLaneService(lane: string, interval: number): void; // throws when the interval is zero, the lane is not a lane, or it has a service
/// ```
pub fn start_lane_service(lane: &Path, interval: Duration) -> io::Result<()> {
    if interval.is_zero() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the interval between passes is zero"));
    }
    let directory: File = super::commit_flusher_unit::open_lane(lane).map_err(|reason| { return io::Error::new(io::ErrorKind::InvalidInput, reason); })?;
    let lock: File = lock_service_in(&directory)?;
    let ring: UnixDatagram = bind_ring_in(&directory).map_err(|error| { return io::Error::new(error.kind(), format!("the doorbell of the lane {} cannot be bound: {}", lane.display(), error)); })?;
    crate::service_log!("lane doorbell bound {}", lane.display());
    let shown: PathBuf = lane.to_path_buf();
    std::thread::spawn(move || {
        // The lock and the directory live as long as the loop.
        let _lock: File = lock;
        let pinned: PathBuf = super::commit_flusher_unit::pinned_path(&directory);
        run_lane_service(&shown, &pinned, &ring, interval);
    });
    return Ok(());
}

/// One pass of the loop: the syncer's pass, then the interrupted disposals the person authorized (a writer that was busy at an earlier try
/// is tried again here, on every pass).
/// `shown` is the lane as the person named it (for the log), `lane` the pinned path the work is done through. Returns whether receipts
/// arrived while the pass swept, so another pass is due at once.
fn pass(shown: &Path, lane: &Path, state: &Path) -> bool {
    let mut more: bool = false;
    match syncer_pass(lane) {
        Ok(done) => more = done.more,
        Err(error) => crate::service_error!("a pass over {} failed: {error:?}", shown.display()),
    }
    if let Err(error) = super::commit_disposal::resume_disposals(lane, state) {
        crate::service_error!("the interrupted disposals of {} could not be finished: {error}", shown.display());
    }
    // Said whether or not the pass succeeded: a pass that recorded an error for a receipt it could not force is a pass that ran.
    crate::service_log!("lane pass complete {}", shown.display());
    return more;
}

/// The loop of the lane service.
fn run_lane_service(shown: &Path, lane: &Path, ring: &UnixDatagram, interval: Duration) {
    let state: PathBuf = lane.join("state");
    match startup_sweep(lane, &state) {
        Ok(recovery) => {
            if let Some(reason) = &recovery.state_error {
                crate::service_error!("the state log of {} is not readable: {reason}", shown.display());
            }
        }
        Err(error) => crate::service_error!("the startup sweep of {} failed: {error:?}", shown.display()),
    }
    crate::service_log!("lane startup complete {}", shown.display());
    let mut buffer: [u8; 16] = [0; 16];
    let mut due: Instant = Instant::now() + interval;
    loop {
        // A ring and a due pass both mean "run a pass"; any other receive error is logged and the pass runs anyway. The timeout is the time
        // left until the pass is due (at least a millisecond, since a zero timeout is refused by the socket).
        let wait: Duration = due.saturating_duration_since(Instant::now()).max(Duration::from_millis(1));
        if let Err(error) = ring.set_read_timeout(Some(wait)) {
            crate::service_error!("the doorbell of {} has no timeout: {error}", shown.display());
            return;
        }
        if let Err(error) = ring.recv(&mut buffer)
            && !matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut)
        {
            crate::service_error!("the doorbell of {} failed: {error}", shown.display());
        }
        let started: Instant = Instant::now();
        let more: bool = pass(shown, lane, &state);
        due = next_due(started, interval, more);
    }
}
