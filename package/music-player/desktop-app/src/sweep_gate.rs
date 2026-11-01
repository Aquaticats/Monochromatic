//! The pause gate of the background true-peak sweep (D107, D84).
//!
//! D26 keeps a Pause and Resume control for the analysis, read as a temporary pause that does not make analysis
//! optional (D84).
//! The control lives on the system indicator (`analysis_notification`), and this gate is what it pauses:
//! every sweep worker asks the gate for admission before it claims the next track, so a pause stops new claims at once,
//! a track already being decoded finishes, and a resume wakes the parked workers.
//! `close` ends the gate for good at shutdown, so parked workers leave instead of waiting for a resume that cannot come.

use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock, PoisonError};

/// What: `Admission` is the answer a worker gets before it claims a track.
/// Why: a worker either goes on or leaves; there is no third state to handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission {
    /// Claim the next track.
    Proceed,
    /// The gate is closed for good; leave the loop.
    Stop,
}

/// What: `GateState` is where the gate is.
/// Why: `Closed` outranks `Paused`, so a pause can never revive a gate that was closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GateState {
    Running,
    Paused,
    Closed,
}

/// What: `SweepGate` is the shared pause flag with the condition workers park on.
/// Why: a flag alone would make paused workers spin; the condition variable parks them until resume or close.
#[derive(Debug)]
pub struct SweepGate {
    state: Mutex<GateState>,
    wake: Condvar,
}

/// What: `SHARED` is the gate the app's sweeps use.
/// Why: the pause comes from the indicator and the sweeps start from the controller, which share no other object;
/// tests build their own gates and never touch this one.
static SHARED: OnceLock<Arc<SweepGate>> = OnceLock::new();

/// Methods of `SweepGate`.
impl SweepGate {
    /// What: `new` builds a running gate.
    /// Why: a sweep that nobody paused must not wait.
    pub fn new() -> SweepGate {
        return SweepGate {
            state: Mutex::new(GateState::Running),
            wake: Condvar::new(),
        };
    }

    /// What: `shared` returns the app-wide gate, making it on first use.
    /// Why: see `SHARED`.
    pub fn shared() -> Arc<SweepGate> {
        return Arc::clone(SHARED.get_or_init(|| return Arc::new(SweepGate::new())));
    }

    /// What: `lock` takes the state lock and ignores poisoning.
    /// Why: the state is one enum that every writer replaces whole, so a panic elsewhere cannot leave it half-written.
    fn lock(&self) -> MutexGuard<'_, GateState> {
        return self.state.lock().unwrap_or_else(PoisonError::into_inner);
    }

    /// What: `set_paused` pauses (`true`) or resumes (`false`) the sweep; a closed gate stays closed.
    /// Why: resume must wake every parked worker, and a late resume after shutdown must not reopen the gate.
    pub fn set_paused(&self, paused: bool) {
        let mut state = self.lock();
        if *state == GateState::Closed {
            tracing::debug!(paused, "sweep pause change ignored: the gate is closed");
            return;
        }
        let next = if paused { GateState::Paused } else { GateState::Running };
        if *state != next {
            tracing::info!(paused, "sweep gate changed");
            *state = next;
            self.wake.notify_all();
        }
    }

    /// What: `is_paused` is true while workers are held back.
    /// Why: callers and tests read the state without parking.
    pub fn is_paused(&self) -> bool {
        return *self.lock() == GateState::Paused;
    }

    /// What: `close` ends the gate for good and wakes every parked worker.
    /// Why: at shutdown a paused sweep would otherwise keep its coordinator waiting on workers that never wake.
    pub fn close(&self) {
        let mut state = self.lock();
        tracing::info!("sweep gate closed");
        *state = GateState::Closed;
        self.wake.notify_all();
    }

    /// What: `admit` blocks while the gate is paused, then says whether to claim the next track.
    /// Why: this is the check before each cursor claim; a track already claimed is not interrupted.
    pub fn admit(&self) -> Admission {
        let mut state = self.lock();
        while *state == GateState::Paused {
            tracing::debug!("sweep worker parked: the gate is paused");
            state = self.wake.wait(state).unwrap_or_else(PoisonError::into_inner);
        }
        return match *state {
            GateState::Closed => Admission::Stop,
            GateState::Running | GateState::Paused => Admission::Proceed,
        };
    }
}

/// Methods of the default gate.
impl Default for SweepGate {
    /// What: `default` builds a running gate.
    /// Why: clippy's `new_without_default`.
    fn default() -> Self {
        return SweepGate::new();
    }
}

/// Unit tests in `sweep_gate_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "sweep_gate_tests.rs"]
mod tests;
