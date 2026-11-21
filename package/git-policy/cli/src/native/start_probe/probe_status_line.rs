//! What:
//!  One status line a probe child writes when a step ends:
//!  the arm,
//!  the step,
//!  the answer,
//!  the errno's name and,
//!  where the step read one,
//!  the source's name.
//! Why:
//!  The mode writes every field of the record from values it read itself,
//!  and the children are the only processes that know a step's outcome
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The pipes and parent-death signals each child owns").
//!  The line is fixed keys and words from a closed vocabulary,
//!  so a parent reads an outcome rather than a sentence,
//!  and a line it cannot parse is an arm failure naming the parse
//!  rather than a guess.
//!  No value holds a space or a newline:
//!  the source name is one ZFS component chain,
//!  which the name rule already bounds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = encode({ step: 'verify-mount', answer: 'passed' }); // 'probe-step arm=verify step=verify-mount answer=passed errno=none'
//! ```

use super::probe_arms::ProbeStep;
use super::probe_errno_names::{errno_name, known_reason_word};
use super::probe_line_refusal::LineRefusal;
use super::probe_status_prefix::STATUS_PREFIX;

/// What:
///  What one step's line says happened.
/// Why:
///   Three answers are all the record needs:
///  the step passed,
///  it failed and the failure had an errno or was a comparison,
///  or it did not run and the record says why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answer = { kind: 'passed' } | { kind: 'failed'; errno?: string } | { kind: 'not-exercised'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StepAnswer {
    /// The step's call or check answered as the design expects.
    Passed,
    /// The step failed,
    /// with the errno name its call answered,
    /// or nothing where the failure was a comparison.
    Failed {
        /// The errno name,
        /// or nothing for a comparison.
        errno: Option<&'static str>,
    },
    /// The step did not run,
    /// the word says why,
    /// and the errno its call answered where one did.
    NotExercised {
        /// One word naming the cause.
        reason: &'static str,
        /// The errno name the call answered,
        /// where it answered one.
        errno: Option<&'static str>,
    },
}

impl StepAnswer {
    /// The answer word the line carries.
    pub fn word(&self) -> &'static str {
        return match self {
            StepAnswer::Passed => "passed",
            StepAnswer::Failed { .. } => "failed",
            StepAnswer::NotExercised { .. } => "not-exercised",
        };
    }
}

/// What:
///  One step's outcome as a child wrote it.
/// Why:
///   The step names its arm,
///  so the line carries no second arm field a child could disagree with,
///  and the source is present only on the step that read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StepLine = { step: Step; answer: Answer; source?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepLine {
    /// The step whose outcome this is.
    pub step: ProbeStep,
    /// What the step answered.
    pub answer: StepAnswer,
    /// The source the step read,
    /// where it read one.
    pub source: Option<String>,
}

impl StepLine {
    /// One passing step's line.
    pub fn passed(step: ProbeStep) -> Self {
        return StepLine { step, answer: StepAnswer::Passed, source: None };
    }

    /// One failed step's line,
    /// with the errno name its call answered or nothing for a comparison.
    pub fn failed(step: ProbeStep, errno: Option<&'static str>) -> Self {
        return StepLine { step, answer: StepAnswer::Failed { errno }, source: None };
    }

    /// One step that did not run,
    /// the word saying why,
    /// and the errno its call answered where one did.
    pub fn not_exercised(step: ProbeStep, reason: &'static str, errno: Option<&'static str>) -> Self {
        return StepLine { step, answer: StepAnswer::NotExercised { reason, errno }, source: None };
    }

    /// The line as the pipe carries it,
    /// its newline included.
    pub fn encode(&self) -> String {
        let errno: &str = match &self.answer {
            StepAnswer::Passed => "none",
            StepAnswer::Failed { errno: Some(name) } => name,
            StepAnswer::Failed { errno: None } => "none",
            StepAnswer::NotExercised { errno: Some(name), .. } => name,
            StepAnswer::NotExercised { errno: None, .. } => "none",
        };
        let arm: &str = self.step.arm().name();
        let source: &str = self.source.as_deref().unwrap_or("-");
        let reason: String = match &self.answer {
            StepAnswer::NotExercised { reason, .. } => format!(" reason={reason}"),
            StepAnswer::Passed | StepAnswer::Failed { .. } => String::new(),
        };
        return format!("{STATUS_PREFIX} arm={arm} step={} answer={} errno={errno} source={source}{reason}\n", self.step.name(), self.answer.word());
    }

    /// Read one line,
    /// which may keep its newline.
    ///
    /// # Errors
    /// A line of another kind,
    /// a repeated or unknown field,
    /// a missing field,
    /// an unknown step,
    /// answer or errno word,
    /// or a value that is not one word.
    pub fn parse(text: &str) -> Result<StepLine, LineRefusal> {
        let trimmed: &str = text.trim_end_matches('\n');
        let mut fields: Vec<(&str, &str)> = Vec::new();
        for (place, word) in trimmed.split(' ').enumerate() {
            if place == 0 {
                if word != STATUS_PREFIX {
                    return Err(LineRefusal::NotAStatusLine { text: String::from(trimmed) });
                }
                continue;
            }
            let Some((key, value)) = word.split_once('=') else {
                return Err(LineRefusal::UnknownKey { word: String::from(word) });
            };
            if !matches!(key, "arm" | "step" | "answer" | "errno" | "source" | "reason") {
                return Err(LineRefusal::UnknownKey { word: String::from(key) });
            }
            if value.is_empty() {
                return Err(LineRefusal::ValueNotOneWord { key: String::from(key) });
            }
            fields.push((key, value));
        }
        let mut arm: Option<&str> = None;
        let mut step_field: Option<&str> = None;
        let mut answer_field: Option<&str> = None;
        let mut errno: Option<&str> = None;
        let mut source_field: Option<&str> = None;
        let mut reason_field: Option<&str> = None;
        for (key, value) in fields {
            let slot: &mut Option<&str> = match key {
                "arm" => &mut arm,
                "step" => &mut step_field,
                "answer" => &mut answer_field,
                "errno" => &mut errno,
                "source" => &mut source_field,
                _ => &mut reason_field,
            };
            if slot.is_some() {
                return Err(LineRefusal::RepeatedKey { key: String::from(key) });
            }
            *slot = Some(value);
        }
        let Some(step_word) = step_field else {
            return Err(LineRefusal::MissingKey { key: String::from("step") });
        };
        let Some(step) = ProbeStep::from_name(step_word) else {
            return Err(LineRefusal::UnknownStep { word: String::from(step_word) });
        };
        let expected_arm: &'static str = step.arm().name();
        match arm {
            None => return Err(LineRefusal::MissingKey { key: String::from("arm") }),
            Some(word) if word != expected_arm => return Err(LineRefusal::WrongArm { expected: expected_arm, word: String::from(word) }),
            Some(_) => {}
        };
        let Some(answer_word) = answer_field else {
            return Err(LineRefusal::MissingKey { key: String::from("answer") });
        };
        let errno_name: Option<&'static str> = match errno {
            None | Some("none") => None,
            Some(word) => match errno_name(word) {
                Some(name) => Some(name),
                None => return Err(LineRefusal::UnknownErrno { word: String::from(word) }),
            },
        };
        let parsed_answer: StepAnswer = match answer_word {
            "passed" => StepAnswer::Passed,
            "failed" => StepAnswer::Failed { errno: errno_name },
            "not-exercised" => match reason_field {
                None => return Err(LineRefusal::MissingKey { key: String::from("reason") }),
                Some(word) => match known_reason_word(word) {
                    Some(reason) => StepAnswer::NotExercised { reason, errno: errno_name },
                    None => return Err(LineRefusal::UnknownReason { word: String::from(word) }),
                },
            },
            other => return Err(LineRefusal::UnknownAnswer { word: String::from(other) }),
        };
        let parsed_source: Option<String> = match source_field {
            None | Some("-") => None,
            Some(word) => Some(String::from(word)),
        };
        return Ok(StepLine { step, answer: parsed_answer, source: parsed_source });
    }
}

/// What:
///  A status pipe's bytes as whole lines.
/// Why:
///  A pipe hands the mode whatever bytes the kernel has,
///  which is no line and half a line as often as two,
///  so the reader keeps the tail until its newline arrives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// feed.push(bytes); for (const line of feed.take()) report(line);
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LineFeed {
    /// The bytes after the last newline read so far.
    pending: Vec<u8>,
}

impl LineFeed {
    /// An empty feed.
    pub fn new() -> Self {
        return LineFeed { pending: Vec::new() };
    }

    /// Keep the bytes one read answered and answer every whole line they
    ///  completed,
    ///  parsed.
    pub fn push(&mut self, bytes: &[u8]) -> Vec<Result<StepLine, LineRefusal>> {
        self.pending.extend_from_slice(bytes);
        let mut lines: Vec<Result<StepLine, LineRefusal>> = Vec::new();
        while let Some(end) = self.pending.iter().position(|byte| return *byte == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=end).collect();
            let Ok(text) = std::str::from_utf8(&line[..line.len() - 1]) else {
                lines.push(Err(LineRefusal::NotAStatusLine { text: String::from_utf8_lossy(&line).into_owned() }));
                continue;
            };
            lines.push(StepLine::parse(text));
        }
        return lines;
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_status_line_tests.rs"]
mod tests;
