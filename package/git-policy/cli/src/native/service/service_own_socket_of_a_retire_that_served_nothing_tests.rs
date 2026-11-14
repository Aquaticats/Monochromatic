//! Controls for the socket a service stops when it was retired before it served anything:
//! that a manager's refusal is a line and not a panic,
//! that a unit the manager does not hold counts as stopped,
//! and that an account with no runtime directory or no bus is told so.

use super::*;

/// A manager that records the stops it is asked for and can refuse one,
/// and holds no unit.
#[derive(Clone, Default)]
struct Recorder {
    stops: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
    refusal: std::rc::Rc<std::cell::RefCell<Option<CallError>>>,
}

impl Manager for Recorder {
    fn within(&mut self, _wait: std::time::Duration) {}

    fn reset_failed(&mut self, _unit: &str) -> Result<(), CallError> {
        return Ok(());
    }

    fn start(&mut self, _unit: &str) -> Result<(), CallError> {
        return Ok(());
    }

    fn stop(&mut self, unit: &str) -> Result<(), CallError> {
        self.stops.borrow_mut().push(format!("stop {unit}"));
        return Ok(());
    }

    fn stop_alone(&mut self, unit: &str) -> Result<(), CallError> {
        self.stops.borrow_mut().push(format!("stop-alone {unit}"));
        return match self.refusal.borrow().clone() {
            Some(failure) => Err(failure),
            None => Ok(()),
        };
    }

    fn active_state(&mut self, _unit: &str) -> Result<Option<String>, CallError> {
        return Ok(None);
    }

    fn listen(&mut self, _unit: &str) -> Result<Option<Vec<String>>, CallError> {
        return Ok(None);
    }
}

/// What:
///  The stop this service sends is the socket's alone,
/// so the service that is already ending is not signalled by its own unit.
/// Why:
///   A plain stop would carry through the unit's `Requires=` to this very process,
/// and a process that signals itself cannot finish the
///  sentence it was writing.
#[test]
fn the_stop_is_the_sockets_and_never_the_services() {
    let mut recorder: Recorder = Recorder::default();
    stop_alone(&mut recorder, "example@abc.socket").expect("the manager took the stop");
    assert_eq!(recorder.stops.borrow().clone(), vec![String::from("stop-alone example@abc.socket")]);
}

/// What:
///  A unit the manager does not hold is stopped already,
/// and any other refusal is answered as it came.
/// Why:
///   A socket the manager collected between the question and the stop is what a repair leaves,
/// and it is not a refusal.
#[test]
fn a_unit_the_manager_does_not_hold_is_stopped_and_any_other_refusal_is_returned() {
    let mut missing: Recorder = Recorder::default();
    *missing.refusal.borrow_mut() = Some(CallError::Manager { name: String::from(NO_SUCH_UNIT), text: String::new() });
    stop_alone(&mut missing, "example@abc.socket").expect("a unit that is not there is stopped");
    let mut refused: Recorder = Recorder::default();
    *refused.refusal.borrow_mut() = Some(CallError::TimedOut);
    assert_eq!(stop_alone(&mut refused, "example@abc.socket"), Err(CallError::TimedOut));
}
