//! What:
//!  The starvation reservation of the next landing slot:
//!       a transaction that lost `landing.reserveAfterLostRaces` races writes `reservation-request`,
//!       takes the reservation lock `reservation.lock` in the registry when it is granted,
//!       and every other transaction waits before it takes the landing lock.
//! Why:
//!  Without it a commit that keeps losing the race can starve behind a stream of landings.
//!       Grant order is the oldest live requester, so the transaction that waited longest goes first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await using reservation = await openLandingReservation({ registryRoot, transactionDirectory, transactionId });
//! await reservation.reserve(lostRaces);
//! ```

/// The backstop every wait on a live process is bounded by, and the line a wait for a lock prints.
use super::live_lock::{LIVE_WAIT_VARIABLE, LockWaitFacts, live_wait_limit, slow_wait_line};
/// When a wait prints its notices, the place in line it reports, and the printing itself.
use super::wait_progress::{Cadence, QueuePlace, WaitReporter, queue_place_text};
/// The sink a notice goes to.
use std::io::Write;
/// Owner-lock records and their liveness.
use super::owner_lock::{OwnerLock, OwnerLockError, try_acquire_owner_lock};
use super::owner_lock_record::{
    OwnerLockRecord, OwnerRecordError, PublishedOwner, owner_lock_holder_is_alive, read_owner_lock_record,
};
/// Registry layout and grant order.
use super::recovery_inspect::{InspectedEntry, OwnerEvidence, oldest_first};
use super::transaction_owner::OwnerLiveness;
use super::transaction_registry::{EntryKind, RESERVATION_LOCK_NAME, list_transaction_entries};
/// Private file writes that survive a crash.
use super::private_storage::{sync_directory, write_private_file};
use super::recovery_error::RecoveryError;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// Delay between reservation checks, and the clock a wait is measured on.
use std::time::{Duration, Instant};

/// Name of the request file inside a transaction directory.
pub const RESERVATION_REQUEST_FILENAME: &str = "reservation-request";

/// Delay between reservation and holder checks, as the incumbent's `RESERVATION_POLL_MS`.
pub const RESERVATION_POLL: Duration = Duration::from_millis(20);

/// What:
///  A reservation failure:
///       the registry, a lock record, or a request file could not be read or written.
/// Why:
///   Replay fails the transaction with the underlying reason instead of landing without the reservation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new CommitTransactionGitError(...);
/// ```
#[derive(Debug)]
pub enum ReservationError {
    /// The transaction registry could not be listed or inspected.
    Registry(RecoveryError),
    /// The reservation lock's record could not be read.
    Record(OwnerRecordError),
    /// The reservation lock could not be taken or released.
    Lock(OwnerLockError),
    /// The holder's process identity could not be probed.
    Identity(String),
    /// A request file or its directory could not be written.
    Write(std::io::Error),
    /// A live reservation holder or an older live requester kept the slot until the wait limit ran out.
    WaitTimedOut {
        /// How long this transaction waited.
        waited: Duration,
        /// The reservation holder's process, when one holds it and its record could be read.
        holder_pid: Option<i64>,
    },
}

/// Text of each failure:
///  the debug form for the failures that carry another error, a sentence for a refused wait.
impl std::fmt::Display for ReservationError {
    /// Writes the diagnostic text of the failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::WaitTimedOut { waited, holder_pid } => {
                let who: String = match holder_pid {
                    Some(pid) => format!("process {pid} holds the reservation"),
                    None => String::from("an older live landing request holds the next slot"),
                };
                write!(
                    formatter,
                    "gave up after waiting {} seconds for the next landing slot because {who}; \
                     nothing was taken from that process. To go on, let it finish and run the command again, \
                     check with `ps` whether it is stuck and stop it if so, or raise the limit with \
                     {LIVE_WAIT_VARIABLE} (seconds)",
                    waited.as_secs()
                )
            }
            Self::Registry(error) => write!(formatter, "{error}"),
            Self::Lock(error) => write!(formatter, "{error}"),
            other => write!(formatter, "{other:?}"),
        };
    }
}

/// What:
///  The request file bytes:
///       `{"schemaVersion":2,"state":"reservation-request","lostRaces":<n>}` and a newline.
/// Why:
///   The TypeScript writes this object, not the empty marker SPEC names;
///       both wrappers read the same file, so the bytes match the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${JSON.stringify({ schemaVersion: 2, state: 'reservation-request', lostRaces })}\n`
/// ```
pub fn reservation_request_bytes(lost_races: u64) -> Vec<u8> {
    return format!(
        "{{\"schemaVersion\":2,\"state\":\"reservation-request\",\"lostRaces\":{lost_races}}}\n"
    )
    .into_bytes();
}

/// What:
///  Whether a lost race earns the reservation:
///       the `reserve_after`-th lost race and every later one.
/// Why:
///   `landing.reserveAfterLostRaces` counts races lost so far, from 1; the check runs after each loss.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lostRaces >= input.reserveAfterLostRaces
/// ```
pub fn reservation_earned(lost_races: u64, reserve_after: u64) -> bool {
    return lost_races >= reserve_after;
}

/// What:
///  The transactions with a live request, oldest first; the first one is the next grant.
/// Why:
///   Grant order is the oldest live requester, with the registry's creation key breaking ties.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await liveRequesters(registryRoot); // transaction IDs, oldest invocation first
/// ```
pub fn live_requesters(registry_root: &Path) -> Result<Vec<String>, ReservationError> {
    let entries = list_transaction_entries(registry_root).map_err(ReservationError::Registry)?;
    // `mut` lets the requesting entries be gathered from the registry listing.
    let mut requesting: Vec<InspectedEntry> = Vec::new();
    for entry in entries {
        if !matches!(entry.kind, EntryKind::Transaction) || !entry.path.join(RESERVATION_REQUEST_FILENAME).exists() {
            continue;
        }
        requesting.push(super::recovery_inspect::inspect_registry_entry(&entry).map_err(ReservationError::Registry)?);
    }
    return Ok(oldest_first(requesting)
        .into_iter()
        .filter(|inspected| { return matches!(inspected.owner, OwnerEvidence::Owned(_, OwnerLiveness::Alive)); })
        .map(|inspected| { return inspected.entry.transaction_id; })
        .collect());
}

/// What:
///  The live holder of the reservation lock:
///       its record when the lock is published and its owner is alive, otherwise nothing.
/// Why:
///   A dead holder's reservation no longer blocks anyone; its lock is retired by the next acquisition.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// liveHolder(lockDirectory); // OwnerLockRecord | NO_HOLDER
/// ```
pub fn live_holder(lock_directory: &Path) -> Result<Option<OwnerLockRecord>, ReservationError> {
    let published: PublishedOwner = read_owner_lock_record(lock_directory).map_err(ReservationError::Record)?;
    let record = match published {
        PublishedOwner::Owner(found) => found,
        PublishedOwner::Busy => return Ok(None),
    };
    let alive: bool = owner_lock_holder_is_alive(&record, lock_directory)
        .map_err(|error| { return ReservationError::Identity(format!("{error:?}")); })?;
    return Ok(if alive { Some(record) } else { None });
}

/// What:
///  One transaction's view of the reservation:
///       whether another transaction holds it, waiting for it, requesting it, and releasing it.
/// Why:
///   The landing loop takes the reservation after its lost races and must give it back when the loop ends,
///       landed or failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingReservation = AsyncDisposable & { awaitSlot; heldByAnother; reserve };
/// ```
pub struct LandingReservation {
    /// Transaction registry.
    registry_root: PathBuf,
    /// This transaction's directory, which holds the request file.
    transaction_directory: PathBuf,
    /// This transaction's ID.
    transaction_id: String,
    /// Whether this invocation lands under an ancestor's landing lock, which neither waits nor takes the reservation.
    nested: bool,
    /// The reservation, once granted to this transaction.
    held: Option<OwnerLock>,
    /// When a wait for the next slot prints its notices.
    cadence: Cadence,
}

impl LandingReservation {
    /// What:
    ///  Opens the reservation view for one landing loop.
    /// Why:
    ///   The caller decides `nested` from its inherited landing lease, which this module does not read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await openLandingReservation({ registryRoot, transactionDirectory, transactionId });
    /// ```
    pub fn open(registry_root: &Path, transaction_directory: &Path, transaction_id: &str, nested: bool) -> Self {
        return Self {
            registry_root: registry_root.to_path_buf(),
            transaction_directory: transaction_directory.to_path_buf(),
            transaction_id: String::from(transaction_id),
            nested,
            held: None,
            cadence: Cadence::from_environment(),
        };
    }

    /// The same view with another notice cadence;
    ///  tests choose a short one instead of waiting out the standard one.
    pub fn with_cadence(mut self, cadence: Cadence) -> Self {
        self.cadence = cadence;
        return self;
    }

    /// The reservation lock's directory.
    fn lock_directory(&self) -> PathBuf {
        return self.registry_root.join(RESERVATION_LOCK_NAME);
    }

    /// The process that holds the reservation, when it is alive and its record can be read.
    fn holder_pid(&self) -> Option<i64> {
        return live_holder(self.lock_directory().as_path()).ok().flatten().map(|record| {
            return record.owner_pid;
        });
    }

    /// What:
    ///  The notice of a wait for the next landing slot.
    /// Why:
    ///   With a live holder it names the holder as every lock wait does;
    ///  with the lock free and older requests ahead it says whose turn comes first.
    ///  A commit in line is told its number.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reservationWaitLine({ holderPid, place, waitedMs, limitMs, repeat })
    /// ```
    fn wait_line(&self, waited: Duration, limit: Duration, place: Option<QueuePlace>, repeat: bool) -> String {
        let lock_directory: PathBuf = self.lock_directory();
        let holder_pid: Option<i64> = self.holder_pid();
        if holder_pid.is_some() || place.is_none() {
            return slow_wait_line(&LockWaitFacts {
                subject: "the landing reservation",
                lock_directory: lock_directory.as_path(),
                holder_pid,
                waited,
                limit,
                place,
                repeat,
            });
        }
        let position: String = place.map_or_else(String::new, |found| { return format!(" {}", queue_place_text(found)); });
        return format!(
            "cli-git: still waiting for older commits to take the landing reservation before this one \
             ({}) after {} seconds; giving up after {} seconds ({LIVE_WAIT_VARIABLE} changes this), leaving every \
             other commit's request and lock in place.{position}",
            lock_directory.display(),
            waited.as_secs(),
            limit.as_secs()
        );
    }

    /// What:
    ///  Whether another transaction holds a live reservation right now.
    /// Why:
    ///   The holder's own token never counts against itself.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await reservation.heldByAnother();
    /// ```
    pub fn held_by_another(&self) -> Result<bool, ReservationError> {
        if self.nested {
            return Ok(false);
        }
        let holder = live_holder(self.lock_directory().as_path())?;
        return Ok(match (holder, self.held.as_ref()) {
            (None, _) => false,
            (Some(record), Some(lock)) => record.token != lock.token(),
            (Some(_), None) => true,
        });
    }

    /// What:
    ///  Waits while a live reservation owned by another transaction exists.
    /// Why:
    ///   A landing may take the landing lock only after the holder has finished its landing loop.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await reservation.awaitSlot();
    /// ```
    pub fn await_slot(&self) -> Result<(), ReservationError> {
        return self.await_slot_for(live_wait_limit());
    }

    /// What:
    ///  Waits while a live reservation owned by another transaction exists, for at most `limit`,
    ///  with notices on standard error.
    /// Why:
    ///   Tests and the default wait share one loop; only the limit differs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await reservation.awaitSlot({ limitMs });
    /// ```
    pub fn await_slot_for(&self, limit: Duration) -> Result<(), ReservationError> {
        return self.await_slot_noticing(limit, &mut std::io::stderr());
    }

    /// What:
    ///  Waits while a live reservation owned by another transaction exists, for at most `limit`,
    ///  printing a notice to `notices` after the grace and at every interval.
    /// Why:
    ///   A commit that queues behind a reservation holder must say what it waits for and who holds it,
    ///  or the caller cannot tell it from a hung one; the holder is read only when a notice is due.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await reservation.awaitSlot({ limitMs, notices });
    /// ```
    pub fn await_slot_noticing(&self, limit: Duration, notices: &mut dyn Write) -> Result<(), ReservationError> {
        let started: Instant = Instant::now();
        let mut reporter: WaitReporter<'_> = WaitReporter::start(self.cadence, notices);
        while self.held_by_another()? {
            self.refuse_after(started, limit)?;
            reporter.poll(|waited, repeat| { return self.wait_line(waited, limit, None, repeat); });
            std::thread::sleep(RESERVATION_POLL);
        }
        return Ok(());
    }

    /// What:
    ///  Refuse to keep waiting once `limit` has passed since `started`.
    /// Why:
    ///   A live process that never finishes its landing would otherwise hold every other transaction for good;
    ///  the refusal names it and leaves its lock and request alone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (Date.now() - startedMs >= limitMs) throw new ReservationTimeout(holderPid);
    /// ```
    fn refuse_after(&self, started: Instant, limit: Duration) -> Result<(), ReservationError> {
        let waited: Duration = started.elapsed();
        if !limit_reached(waited, limit) {
            return Ok(());
        }
        let holder_pid: Option<i64> = live_holder(self.lock_directory().as_path())?.map(|record| {
            return record.owner_pid;
        });
        return Err(ReservationError::WaitTimedOut { waited, holder_pid });
    }

    /// What:
    ///  Writes the request, then waits until this transaction is the oldest live requester and the lock is free,
    ///       and takes the reservation.
    /// Why:
    ///   The first requester in grant order gets the next slot, so a starving commit cannot be overtaken again.
    ///       An inherited landing lease neither asks nor waits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await reservation.reserve(lostRaces); // false when already held or not applicable
    /// ```
    pub fn reserve(&mut self, lost_races: u64) -> Result<bool, ReservationError> {
        return self.reserve_for(lost_races, live_wait_limit());
    }

    /// What:
    ///  Writes the request and waits for the grant, for at most `limit`.
    /// Why:
    ///   Tests and the default wait share one loop; only the limit differs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await reservation.reserve(lostRaces, { limitMs });
    /// ```
    pub fn reserve_for(&mut self, lost_races: u64, limit: Duration) -> Result<bool, ReservationError> {
        return self.reserve_noticing(lost_races, limit, &mut std::io::stderr());
    }

    /// What:
    ///  Writes the request and waits for the grant, for at most `limit`,
    ///  printing a notice to `notices` after the grace and at every interval.
    /// Why:
    ///   The live requesters are listed on every check, so the notice names this commit's place among them
    ///  without reading anything more.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await reservation.reserve(lostRaces, { limitMs, notices });
    /// ```
    pub fn reserve_noticing(
        &mut self,
        lost_races: u64,
        limit: Duration,
        notices: &mut dyn Write,
    ) -> Result<bool, ReservationError> {
        if self.nested || self.held.is_some() {
            return Ok(false);
        }
        write_private_file(
            self.transaction_directory.join(RESERVATION_REQUEST_FILENAME).as_path(),
            &reservation_request_bytes(lost_races),
        )
        .map_err(ReservationError::Write)?;
        sync_directory(self.transaction_directory.as_path()).map_err(ReservationError::Write)?;
        let started: Instant = Instant::now();
        let mut reporter: WaitReporter<'_> = WaitReporter::start(self.cadence, notices);
        loop {
            let lock_directory: PathBuf = self.lock_directory();
            let free: bool = live_holder(lock_directory.as_path())?.is_none();
            let requesters: Vec<String> = live_requesters(self.registry_root.as_path())?;
            let first: bool = requesters.first() == Some(&self.transaction_id);
            let place: Option<QueuePlace> = requesters
                .iter()
                .position(|requester| { return *requester == self.transaction_id; })
                .map(|index| { return QueuePlace { position: index + 1, waiting: requesters.len() }; });
            let granted: Option<OwnerLock> = if free && first {
                try_acquire_owner_lock(lock_directory.as_path(), Some(self.transaction_id.as_str()))
                    .map_err(ReservationError::Lock)?
            } else {
                None
            };
            if let Some(lock) = granted {
                self.held = Some(lock);
                return Ok(true);
            }
            self.refuse_after(started, limit)?;
            reporter.poll(|waited, repeat| { return self.wait_line(waited, limit, place, repeat); });
            std::thread::sleep(RESERVATION_POLL);
        }
    }

    /// What:
    ///  Releases the reservation if held and removes the request file.
    /// Why:
    ///   The request goes even when the reservation was never granted, so no stale request blocks the next grant.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await reservation[Symbol.asyncDispose]();
    /// ```
    pub fn release(mut self) -> Result<(), ReservationError> {
        let request: PathBuf = self.transaction_directory.join(RESERVATION_REQUEST_FILENAME);
        match std::fs::remove_file(&request) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(ReservationError::Write(error)),
        }
        if let Some(lock) = self.held.take() {
            lock.release().map_err(ReservationError::Lock)?;
        }
        return Ok(());
    }
}

/// What:
///  Whether a wait has used up its limit; a wait of exactly the limit has.
/// Why:
///   The clock cannot be set to land on the limit, so the comparison is a function a test can call with chosen values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const limitReached = (waitedMs: number, limitMs: number): boolean => waitedMs >= limitMs;
/// ```
fn limit_reached(waited: Duration, limit: Duration) -> bool {
    return waited >= limit;
}

/// What:
///  Takes the landing lock once no other transaction's reservation stands in the way,
///       checking again under the lock, so a reservation granted while this landing waited is honoured.
/// Why:
///   The landing lock is acquired by the caller's closure because its module belongs to the landing lane.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireReservedLandingLock({ gitPath, cwd, registryRoot, reservation });
/// ```
pub fn acquire_reserved_landing_lock<F>(
    reservation: &LandingReservation,
    mut acquire_landing: F,
) -> Result<OwnerLock, ReservationError>
where
    F: FnMut() -> Result<OwnerLock, ReservationError>,
{
    loop {
        reservation.await_slot()?;
        let lock: OwnerLock = acquire_landing()?;
        if !reservation.held_by_another()? {
            return Ok(lock);
        }
        lock.release().map_err(ReservationError::Lock)?;
    }
}

#[cfg(test)]
#[path = "replay_reservation_tests.rs"]
mod tests;
