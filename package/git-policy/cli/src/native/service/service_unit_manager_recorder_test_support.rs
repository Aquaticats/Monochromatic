//! A manager for controls that records the stops it is asked for and can refuse one,
//! and holds no unit.

use crate::service_provision_host_and_manager::Manager;
use crate::systemd_dbus::CallError;
use std::sync::{Arc, Mutex};

/// A manager that records every call and answers a stop of one named unit with a failure when a control asks for that.
#[derive(Clone, Default)]
pub struct Recorder {
    calls: Arc<Mutex<Vec<String>>>,
    /// The failure a stop of the named unit is answered with.
    refuse: Arc<Mutex<Option<(String, CallError)>>>,
}

impl Recorder {
    /// Makes a stop of `unit` fail with `failure`.
    pub fn refuse_stop_of(&self, unit: &str, failure: CallError) {
        *self.refuse.lock().expect("lock") = Some((unit.to_owned(), failure));
    }

    /// The stops asked for,
    /// in order,
    /// as `stop <unit>`.
    pub fn stops(&self) -> Vec<String> {
        return self.calls.lock().expect("lock").iter().filter(|call| { return call.starts_with("stop"); }).cloned().collect();
    }

    /// Every call asked for,
    /// in order.
    pub fn calls(&self) -> Vec<String> {
        return self.calls.lock().expect("lock").clone();
    }

    fn answer(&self, unit: &str) -> Result<(), CallError> {
        return match self.refuse.lock().expect("lock").clone() {
            Some((named, failure)) if named == unit => Err(failure),
            _ => Ok(()),
        };
    }
}

impl Manager for Recorder {
    fn within(&mut self, wait: std::time::Duration) {
        self.calls.lock().expect("lock").push(format!("within {}", wait.as_secs()));
    }

    fn reload(&mut self) -> Result<(), CallError> {
        self.calls.lock().expect("lock").push(String::from("reload"));
        return Ok(());
    }

    fn reset_failed(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.lock().expect("lock").push(format!("reset-failed {unit}"));
        return Ok(());
    }

    fn start(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.lock().expect("lock").push(format!("start {unit}"));
        return Ok(());
    }

    fn stop(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.lock().expect("lock").push(format!("stop {unit}"));
        return self.answer(unit);
    }

    fn stop_alone(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.lock().expect("lock").push(format!("stop-alone {unit}"));
        return self.answer(unit);
    }

    fn active_state(&mut self, unit: &str) -> Result<Option<String>, CallError> {
        self.calls.lock().expect("lock").push(format!("state {unit}"));
        return Ok(None);
    }
}
