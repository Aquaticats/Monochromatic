//! What:
//! A `Runner` that starts nothing:
//! it records every request and answers from rules.
//! Why:
//! A rule is a prefix of the request line (the program's base name,
//! then its arguments) and the outcome to answer,
//! so a test proves by what the fake recorded that nothing reached the real host.
//! A caller imports from `fakes` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export class FakeRunner implements Runner {
//!   calls: RunRequest[] = [];
//!   on(prefix: string, effect: (request: RunRequest) => void): void;
//!   answer(prefix: string, stdout: string, code: number): void;
//!   answerAfter(trigger: string, prefix: string, stdout: string, code: number): void;
//!   answerStreams(prefix: string, stdout: string, stderr: string, code: number): void;
//!   fail(prefix: string, stderr: string): void;
//!   lines(): string[];
//! }
//! ```

/// Imports `RefCell` from `std::cell`,
/// which is how a fake records through a shared reference.
use std::cell::RefCell;

/// Imports what this file uses from `super::super::runner`.
use super::super::runner::{
    RunOutcome,
    RunRequest,
    Runner,
};
/// What:
/// A `Runner` that starts nothing:
/// it records every request and answers from rules.
/// Why:
/// A rule is a prefix of the request line (program base name,
/// then arguments) and the outcome to answer;
///           the first matching rule wins and an unmatched request succeeds with no output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FakeRunner implements Runner { calls: RunRequest[] = []; rules: [string, RunOutcome][] = []; }
/// ```
#[derive(Default)]
pub(crate) struct FakeRunner {
    /// Every request received,
    /// in order.
    calls: RefCell<Vec<RunRequest>>,
    /// Answers by request-line prefix.
    rules: RefCell<Vec<(String, RunOutcome)>>,
    /// Answers that apply only once a request with the trigger prefix has been received:
    /// the trigger,
    /// the request-line prefix to answer and the outcome.
    after_rules: RefCell<Vec<(String, String, RunOutcome)>>,
    /// Side effects run when a request-line prefix matches.
    effects: RefCell<Vec<(String, Box<dyn Fn(&RunRequest)>)>>,
}

/// The methods of `FakeRunner`.
impl FakeRunner {
    /// An empty runner that has recorded nothing.
    pub(crate) fn new() -> FakeRunner {
        return FakeRunner::default();
    }

    /// Runs `effect` (for example,
    /// creating the files a real tool would create) whenever a request whose line starts
    /// with `prefix` arrives.
    pub(crate) fn on(
        &self,
        prefix: &str,
        effect: Box<dyn Fn(&RunRequest)>,
    ) {
        self.effects.borrow_mut().push((prefix.to_string(), effect));
    }

    /// Answers requests whose line starts with `prefix` with this stdout and exit code.
    pub(crate) fn answer(
        &self,
        prefix: &str,
        stdout: &str,
        code: i32,
    ) {
        self.rules.borrow_mut().push((
            prefix.to_string(),
            RunOutcome {
                stdout: stdout.to_string(),
                stderr: String::new(),
                code,
            },
        ));
    }

    /// Answers requests whose line starts with `prefix` with this stdout and exit code,
    /// but only after a request whose line starts with `trigger` was received
    /// (a listing that shows a package only once `add` ran).
    pub(crate) fn answer_after(
        &self,
        trigger: &str,
        prefix: &str,
        stdout: &str,
        code: i32,
    ) {
        let outcome: RunOutcome = RunOutcome {
            stdout: stdout.to_string(),
            stderr: String::new(),
            code,
        };
        self.after_rules
            .borrow_mut()
            .push((trigger.to_string(), prefix.to_string(), outcome));
    }

    /// Answers requests whose line starts with `prefix` with both streams and this exit code.
    ///
    /// Why:
    /// A listing that fails still prints something,
    /// and a test must prove which stream is read.
    pub(crate) fn answer_streams(
        &self,
        prefix: &str,
        stdout: &str,
        stderr: &str,
        code: i32,
    ) {
        let outcome: RunOutcome = RunOutcome {
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            code,
        };
        self.rules.borrow_mut().push((prefix.to_string(), outcome));
    }

    /// Answers requests whose line starts with `prefix` with a failure printing `stderr`.
    pub(crate) fn fail(
        &self,
        prefix: &str,
        stderr: &str,
    ) {
        self.rules.borrow_mut().push((
            prefix.to_string(),
            RunOutcome {
                stdout: String::new(),
                stderr: stderr.to_string(),
                code: 1,
            },
        ));
    }

    /// Every request received,
    /// in order.
    pub(crate) fn calls(&self) -> Vec<RunRequest> {
        return self.calls.borrow().clone();
    }

    /// Every request as a line with the program reduced to its base name,
    /// in order.
    pub(crate) fn lines(&self) -> Vec<String> {
        return self.calls.borrow().iter().map(matching_line).collect();
    }
}

/// The request line with the program's base name (`/x/y/uv tool list` becomes `uv tool list`).
fn matching_line(request: &RunRequest) -> String {
    let base: &str = request
        .command
        .rsplit('/')
        .next()
        .unwrap_or(&request.command);
    let mut line: String = base.to_string();
    for argument in &request.args {
        line.push(' ');
        line.push_str(argument);
    }
    return line;
}

/// Implements `Runner` for `FakeRunner`.
impl Runner for FakeRunner {
    /// Records the request,
    /// runs any matching effect,
    /// and answers from the first matching rule.
    fn run(
        &self,
        request: &RunRequest,
    ) -> RunOutcome {
        self.calls.borrow_mut().push(request.clone());
        let line: String = matching_line(request);
        for (prefix, effect) in self.effects.borrow().iter() {
            if line.starts_with(prefix.as_str()) {
                effect(request);
            }
        }
        for (trigger, prefix, outcome) in self.after_rules.borrow().iter() {
            let triggered: bool = self
                .calls
                .borrow()
                .iter()
                .any(|call| return matching_line(call).starts_with(trigger.as_str()));
            if triggered && line.starts_with(prefix.as_str()) {
                return outcome.clone();
            }
        }
        for (prefix, outcome) in self.rules.borrow().iter() {
            if line.starts_with(prefix.as_str()) {
                return outcome.clone();
            }
        }
        return RunOutcome {
            stdout: String::new(),
            stderr: String::new(),
            code: 0,
        };
    }
}
