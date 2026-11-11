//! A manager for controls that records the stops it is asked for and can refuse one,
//! and holds no unit.

use crate::service_provision_host_and_manager::Manager;
use crate::systemd_dbus::CallError;
use std::cell::RefCell;
use std::rc::Rc;

/// A manager that records every call
/// and answers a stop of one named unit with a failure when a control asks for that.
#[derive(Clone, Default)]
pub struct Recorder {
    calls: Rc<RefCell<Vec<String>>>,
    /// The failure a stop of the named unit is answered with.
    refuse: Rc<RefCell<Option<(String, CallError)>>>,
}

impl Recorder {
    /// Makes a stop of `unit` fail with `failure`.
    pub fn refuse_stop_of(&self, unit: &str, failure: CallError) {
        *self.refuse.borrow_mut() = Some((unit.to_owned(), failure));
    }

    /// The stops asked for,
    /// in order,
    /// as `stop <unit>`.
    pub fn stops(&self) -> Vec<String> {
        return self.calls.borrow().iter().filter(|call| { return call.starts_with("stop"); }).cloned().collect();
    }

    /// Every call asked for,
    /// in order.
    pub fn calls(&self) -> Vec<String> {
        return self.calls.borrow().clone();
    }

    fn answer(&self, unit: &str) -> Result<(), CallError> {
        return match self.refuse.borrow().clone() {
            Some((named, failure)) if named == unit => Err(failure),
            _ => Ok(()),
        };
    }
}

impl Manager for Recorder {
    fn within(&mut self, wait: std::time::Duration) {
        self.calls.borrow_mut().push(format!("within {}", wait.as_secs()));
    }

    fn reload(&mut self) -> Result<(), CallError> {
        self.calls.borrow_mut().push(String::from("reload"));
        return Ok(());
    }

    fn reset_failed(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.borrow_mut().push(format!("reset-failed {unit}"));
        return Ok(());
    }

    fn start(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.borrow_mut().push(format!("start {unit}"));
        return Ok(());
    }

    fn stop(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.borrow_mut().push(format!("stop {unit}"));
        return self.answer(unit);
    }

    fn stop_alone(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.borrow_mut().push(format!("stop-alone {unit}"));
        return self.answer(unit);
    }

    fn active_state(&mut self, unit: &str) -> Result<Option<String>, CallError> {
        self.calls.borrow_mut().push(format!("state {unit}"));
        return Ok(None);
    }

    fn listen(&mut self, unit: &str) -> Result<Option<Vec<String>>, CallError> {
        // This manager holds no unit, so it can say nothing about an address it made. The question is kept apart from
        //  `calls`, which controls compare exactly, so a question that changes nothing in them stays out of them.
        let _asked: &str = unit;
        return Ok(None);
    }
}
