//! What:
//!  The state a record's proof is judged on, made by a control:
//!  a process that really ran and ended,
//!  with the identity it had while it ran,
//!  and the PID file a killed Git leaves beside its lock.
//! Why:
//!  A record naming a dead process is what a kill leaves,
//!  and no control can make a real kill produce it on demand,
//!  so a control plants the same state from a process it started and ended.
//!  The PID file's birth must be strictly later than its lock's or the proof keeps the lock,
//!  and this volume gives two files written microseconds apart one birth tick,
//!  so the file is made again after a pause until the two stamps differ.
//!  That pause orders two files of the fixture;
//!  it never places a second command inside the code under test.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [pid, identity] = aDeadProcess(); makePidFileAfterTheLock(lock, pid);
//! ```

use crate::lock_holder_records::identity_of_process;
use crate::stale_ref_lock_walk::pid_file_beside;
use std::path::Path;
use std::time::{Duration, Instant};

/// How long a control may try to give a PID file a later birth than its lock,
/// which is a file system's clock granularity and not a wait on anything.
const PID_FILE_BIRTH_DEADLINE: Duration = Duration::from_secs(5);

/// What:
///  A process number a helper really had,
///  and the identity it really had,
///  read while it ran and used after it ended.
/// Why:
///   A record of a command that ended names a process this boot had,
///  and the proof reads its identity from the record, not from the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [pid, identity] = aDeadProcess();
/// ```
pub(crate) fn a_dead_process() -> (i64, String) {
    // What: The helper is given a life long enough that this reading cannot race it, and ended here.
    // Why:  A helper that exits in the microsecond between its start and the read has no identity to read, and a
    //       machine under a power profile makes that window wider (the host ran at power-saver on 2026-11-15).
    let mut helper: std::process::Child = std::process::Command::new("sleep")
        .arg("600")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("a bounded helper starts");
    let gone: i64 = i64::from(helper.id());
    let identity: String = identity_of_process(gone).expect("its identity while it runs");
    let _killed = helper.kill();
    helper.wait().expect("the helper ends");
    return (gone, identity);
}

/// What:
///  Write the PID file beside `lock`,
///  naming `pid`,
///  and make it again until its birth is strictly after the lock's.
/// Why:
///   The proof of an abandoned lock begins with a PID file born after its lock,
///  so a control that means to see a removal must give the gate a proven order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// makePidFileAfterTheLock(indexLock, 4242)
/// ```
pub(crate) fn make_pid_file_after_the_lock(lock: &Path, pid: i64) {
    let pid_file = pid_file_beside(lock);
    let began: Instant = Instant::now();
    loop {
        let _ = std::fs::remove_file(pid_file.as_path());
        std::thread::sleep(Duration::from_millis(2));
        std::fs::write(pid_file.as_path(), format!("pid {pid}\n").as_bytes()).expect("the PID file");
        let lock_born = std::fs::symlink_metadata(lock).ok().and_then(|metadata| { return metadata.created().ok(); });
        let pid_born = std::fs::symlink_metadata(pid_file.as_path()).ok().and_then(|metadata| { return metadata.created().ok(); });
        if let (Some(lock_at), Some(pid_at)) = (lock_born, pid_born)
            && pid_at > lock_at
        {
            return;
        }
        assert!(
            began.elapsed() < PID_FILE_BIRTH_DEADLINE,
            "this file system cannot show that {} was born after {}: the lock is {:?} and the PID file {:?}",
            pid_file.display(),
            lock.display(),
            lock_born,
            pid_born
        );
    }
}
