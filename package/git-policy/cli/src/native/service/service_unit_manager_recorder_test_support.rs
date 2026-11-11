//! A manager for controls that records the stops it is asked for and can refuse one,
//! and holds no unit.

use crate::service_provision_host_and_manager::Manager;
use crate::systemd_dbus::CallError;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// A manager that records every call
/// and answers a stop of one named unit with a failure when a control asks for that.
#[derive(Clone, Default)]
pub struct Recorder {
    calls: Rc<RefCell<Vec<String>>>,
    /// The failure a stop of the named unit is answered with.
    refuse: Rc<RefCell<Option<(String, CallError)>>>,
    /// The states the named units are answered with,
    /// each queue's last answer repeated once it is empty.
    states: Rc<RefCell<std::collections::HashMap<String, VecDeque<Option<String>>>>>,
    /// The failure a state question about a named unit is answered with.
    refused_states: Rc<RefCell<Option<(String, CallError)>>>,
}

impl Recorder {
    /// Makes a stop of `unit` fail with `failure`.
    pub fn refuse_stop_of(&self, unit: &str, failure: CallError) {
        *self.refuse.borrow_mut() = Some((unit.to_owned(), failure));
    }

    /// Answers every state question about `unit` with `state`,
    /// so a control can say that the unit's processes are still there.
    pub fn report_state_of(&self, unit: &str, state: Option<&str>) {
        self.report_states_of(unit, &[state]);
    }

    /// Answers the state questions about `unit` with these answers in turn,
    /// and with the last of them from then on,
    /// so a control can watch a unit go from running to gone.
    pub fn report_states_of(&self, unit: &str, states: &[Option<&str>]) {
        let answers: VecDeque<Option<String>> = states.iter().map(|state| { return state.map(str::to_owned); }).collect();
        self.states.borrow_mut().insert(unit.to_owned(), answers);
    }

    /// Makes a state question about `unit` fail with `failure`,
    /// so a control can say that the manager does not answer.
    pub fn refuse_state_of(&self, unit: &str, failure: CallError) {
        *self.refused_states.borrow_mut() = Some((unit.to_owned(), failure));
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

    /// The state the control said the unit is in,
    /// whose queue keeps its last answer,
    /// and nothing when no control named the unit.
    fn answer_state(&self, unit: &str) -> Result<Option<String>, CallError> {
        if let Some((named, failure)) = self.refused_states.borrow().clone()
            && named == unit
        {
            return Err(failure);
        }
        let mut answers = self.states.borrow_mut();
        let Some(queued) = answers.get_mut(unit) else { return Ok(None) };
        if queued.len() > 1 {
            return Ok(queued.pop_front().flatten());
        }
        return Ok(queued.front().cloned().flatten());
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
        return self.answer_state(unit);
    }
}
