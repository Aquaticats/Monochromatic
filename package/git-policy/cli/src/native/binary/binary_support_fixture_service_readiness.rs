//! What:
//!  What tells a fixture that its service is at work and that it is ready:
//! the counters that move while a service starts,
//! and the socket that answers only once a service serves.
//! Why:
//!  A service reads its configuration,
//! writes its log and binds its socket before it listens,
//! so the work it does separates starting from stopped,
//! and a socket that still answers belongs to the service that is going,
//! never to the one that starts beside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const signs = SignsOfWork.of(pid); signs.changed(pid); // true when the service moved since the last call
//! const serving = (socket: string) => connect(socket).then(readLine).then((line) => line.length > 0).catch(() => false)
//! ```

use std::path::Path;

/// How long the fixture's service may show no sign of work before the fixture fails the control,
/// so a service that never listens still ends the control instead of holding it.
/// Why not a bound on the whole wait:
///   A service that starts on a loaded host is working,
/// and how long that takes is the host's;
/// what says it is over is the
///  service showing nothing,
/// which is the evidence every other wait in this crate uses (`run_progress`).
pub(crate) const SERVICE_QUIET_WINDOW: std::time::Duration = std::time::Duration::from_secs(120);

/// What:
///  The counters that show a starting service is at work:
///  its processor ticks and the bytes it has read and written.
/// Why:
///   A service reads its configuration,
/// writes its log and binds its socket before it listens,
/// so the fixture tells a service
///  that is starting from one that has stopped,
/// and ends the wait on the second alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const signs = SignsOfWork.of(pid); signs.changed(pid); // true when the service moved since the last call
/// ```
pub(crate) struct SignsOfWork {
    /// The processor ticks read last.
    ticks: u64,
    /// The bytes read and written read last.
    bytes: u64,
}

impl SignsOfWork {
    /// The signs of the service `pid` now.
    pub(crate) fn of(pid: u32) -> SignsOfWork {
        let (ticks, bytes): (u64, u64) = SignsOfWork::read(pid);
        return SignsOfWork { ticks, bytes };
    }

    /// The counters of the service `pid`,
    /// zero where they cannot be read.
    fn read(pid: u32) -> (u64, u64) {
        let stat: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
        // The fields after the process name: state, then the user and system times, which only grow.
        let ticks: u64 = stat
            .rfind(')')
            .and_then(|close| { return stat.get(close + 2..); })
            .and_then(|after_name| {
                let fields: Vec<&str> = after_name.split(' ').collect();
                let user: u64 = fields.get(11)?.parse().ok()?;
                let system: u64 = fields.get(12)?.parse().ok()?;
                return Some(user.saturating_add(system));
            })
            .unwrap_or(0);
        let io: String = std::fs::read_to_string(format!("/proc/{pid}/io")).unwrap_or_default();
        let mut bytes: u64 = 0;
        for line in io.lines() {
            let value: &str = match line.strip_prefix("rchar: ").or_else(|| { return line.strip_prefix("wchar: "); }) {
                Some(count) => count,
                None => continue,
            };
            bytes = bytes.saturating_add(value.trim().parse::<u64>().unwrap_or(0));
        }
        return (ticks, bytes);
    }

    /// Whether the service moved since the last call,
    /// and takes the signs read now.
    pub(crate) fn changed(&mut self, pid: u32) -> bool {
        let (ticks, bytes): (u64, u64) = SignsOfWork::read(pid);
        let moved: bool = ticks != self.ticks || bytes != self.bytes;
        self.ticks = ticks;
        self.bytes = bytes;
        return moved;
    }
}

/// What:
///  Whether the service that listens at `socket` is serving:
/// it answers a connection with its greeting.
/// Why:
///   The kernel accepts a connection as soon as the service listens,
/// before the service has recovered its journal and started
///  the threads that answer,
/// so an accepted connection says nothing about the service's state.
/// A control that changes the
///  service right after the fixture started it (a lowered process limit) met a service that had not started its work thread and
///  ended it at once with status 4.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const serving = (socket: string) => connect(socket).then(readLine).then((line) => line.length > 0).catch(() => false)
/// ```
pub fn service_is_serving(socket: &Path) -> bool {
    use std::io::BufRead;
    let Ok(stream) = std::os::unix::net::UnixStream::connect(socket) else { return false };
    // A service greets as soon as it has started its threads; a read that gets nothing within this bound is asked again by the
    //  caller's loop, which has its own bound.
    let _timeout = stream.set_read_timeout(Some(GREETING_READ));
    let mut line: String = String::new();
    // An empty read is a connection that ended without a greeting: the service went away as it was starting.
    return std::io::BufReader::new(stream).read_line(&mut line).is_ok_and(|read| { return read > 0; });
}

/// How long one readiness check waits for the greeting.
/// A service answers within milliseconds once its threads run,
/// so a second
///  is far past it;
/// a longer start (a loaded host) is covered by the repeated checks of the caller,
/// not by a longer read.
const GREETING_READ: std::time::Duration = std::time::Duration::from_secs(1);

/// How long a fixture waits for an ended service's socket to stop answering before it says the service is still there.
pub const SOCKET_RELEASE_PATIENCE: std::time::Duration = std::time::Duration::from_secs(60);

/// What:
///  Waits until nothing answers at `socket`:
/// the file is absent,
/// or it is a stale socket whose listener is gone.
/// Why:
///   Ending a service that runs under a tracer reaps the tracer,
/// and the traced service dies a moment later,
/// so its socket can
///  still answer after the fixture counts the service as ended.
/// A new service cannot take such a socket,
/// and a readiness poll
///  that connects to it reports a service that is not the new one.
///
/// # Panics
/// When something still answers after `SOCKET_RELEASE_PATIENCE`,
/// which is a service the fixture did not end.
pub fn wait_until_nothing_listens(socket: &Path) {
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        match std::os::unix::net::UnixStream::connect(socket) {
            Err(error) if matches!(error.kind(), std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound) => return,
            Ok(_) | Err(_) => {}
        }
        assert!(began.elapsed() < SOCKET_RELEASE_PATIENCE, "something still answers at {} although the fixture ended its service", socket.display());
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
