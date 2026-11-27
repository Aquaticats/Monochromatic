//! What:
//!  The structural walk that finds every step's `run:` and `shell:` in a workflow or composite action.
//! Why:
//!  A regular expression on `run:` would also hit `defaults.run` and an action input named `run`,
//!  so the walk tracks the key path by indentation and reads a `run:` only where a step's key sits.
//!  Forms it cannot place are refused,
//!  so the rules never pass a step they could not read.
//!  The keys and sequence items that stay open between lines are kept in
//!  `workflow_walk_frames`,
//!  which this file asks instead of moving that stack itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function walk(text: string): Workflow; // throws WalkError
//! ```

/// Import the line reader.
use crate::workflow_lines::{
    Content,
    Entry,
    Line,
    parse_entry,
    read_lines,
    without_comment,
};
/// Import the records the walk fills in.
use crate::workflow_model::{
    Job,
    WalkError,
    Workflow,
};
/// Import the value readers.
use crate::workflow_run::{
    capture_run,
    capture_shell,
};
/// Import the key-path classification.
use crate::workflow_shape::{
    Shape,
    shape,
};
/// Import the open keys and the state that one walk keeps.
use crate::workflow_walk_frames::{
    Frame,
    Walker,
};

/// Build a refusal at a workflow byte offset.
fn refuse(
    offset: usize,
    message: &str,
) -> WalkError {
    return WalkError {
        offset,
        message: String::from(message),
    };
}

/// Reading one line.
impl Walker<'_> {
    /// Skip the lines a value continues over,
    ///  and return the index of the next line to read.
    fn skip_value(
        &self,
        index: usize,
        column: usize,
        value: &str,
    ) -> usize {
        let visible: &str = without_comment(value);
        let mut next: usize = index + 1;
        if visible.is_empty() {
            return next;
        }
        let first: u8 = visible.as_bytes()[0];
        if first == b'[' || first == b'{' {
            let mut depth: i64 = 0;
            let mut cursor: usize = index;
            let mut text: &str = visible;
            loop {
                for byte in text.bytes() {
                    if byte == b'[' || byte == b'{' {
                        depth += 1;
                    } else if byte == b']' || byte == b'}' {
                        depth -= 1;
                    }
                }
                if depth <= 0 || cursor + 1 >= self.lines.len() {
                    break;
                }
                cursor += 1;
                text = self.lines[cursor].text;
            }
            return cursor + 1;
        }
        // Block scalars, multi-line plain scalars and multi-line quoted scalars all continue
        // over lines indented deeper than the key.
        while next < self.lines.len() {
            let line: &Line<'_> = &self.lines[next];
            if !line.is_skippable() && line.indent <= column {
                break;
            }
            next += 1;
        }
        return next;
    }

    /// Handle one `key: value` line at a classified path.
    fn on_key(
        &mut self,
        index: usize,
        entry: &Entry<'_>,
    ) -> Result<usize, WalkError> {
        let Content::Key {
            name,
            column,
            value,
            value_offset,
        } = &entry.content
        else {
            return Ok(index + 1);
        };
        let line_start: usize = self.lines[index].text_start();
        self.close_from(*column);
        self.frames.push(Frame {
            indent: *column,
            name: String::from(*name),
        });
        let kind: Shape = shape(&self.path());
        let visible: &str = without_comment(value);
        match kind {
            Shape::Structure | Shape::Job => {
                if !visible.is_empty() {
                    return Err(refuse(
                        line_start + value_offset,
                        "this key must be written as a block with its content on the following lines",
                    ));
                }
                if matches!(kind, Shape::Job) {
                    self.found.jobs.push(Job::default());
                }
                return Ok(index + 1);
            }
            Shape::RunsOn => {
                let job: usize = self.found.jobs.len().saturating_sub(1);
                self.found.jobs[job].runs_on = visible.to_ascii_lowercase();
                self.runs_on = Some((job, *column));
                return Ok(index + 1);
            }
            Shape::DefaultShell => {
                let shell = capture_shell(&self.lines[index], value, *value_offset)?;
                match self.frames.len() {
                    3 => self.found.shell = Some(shell),
                    _ => {
                        let job: usize = self.found.jobs.len().saturating_sub(1);
                        self.found.jobs[job].shell = Some(shell);
                    }
                }
                return Ok(index + 1);
            }
            Shape::StepKey if *name == "run" => {
                let (body, next) = capture_run(&self.lines, index, *column, value, *value_offset)?;
                if let Some(step) = self.found.steps.last_mut() {
                    step.body = Some(body);
                }
                return Ok(next);
            }
            Shape::StepKey if *name == "shell" => {
                let shell = capture_shell(&self.lines[index], value, *value_offset)?;
                if let Some(step) = self.found.steps.last_mut() {
                    step.shell = Some(shell);
                }
                return Ok(index + 1);
            }
            Shape::StepKey | Shape::Other => {
                return Ok(self.skip_value(index, *column, value));
            }
        }
    }

    /// Read the line at `index` and return the index of the next line to read.
    fn step(
        &mut self,
        index: usize,
    ) -> Result<usize, WalkError> {
        let (skippable, start, indent, text): (bool, usize, usize, String) = {
            let line: &Line<'_> = &self.lines[index];
            (
                line.is_skippable(),
                line.text_start(),
                line.indent,
                String::from(line.text),
            )
        };
        if skippable {
            return Ok(index + 1);
        }
        if text == "..." {
            return Err(refuse(
                start,
                "a document end marker; a workflow is one document",
            ));
        }
        if text == "---" || text.starts_with("--- ") {
            if self.content_seen {
                return Err(refuse(
                    start,
                    "a second YAML document; a workflow is one document",
                ));
            }
            return Ok(index + 1);
        }
        if text.starts_with('%') && indent == 0 && !self.content_seen {
            return Ok(index + 1);
        }
        self.content_seen = true;
        if let Some((_, column)) = self.runs_on
            && indent <= column
        {
            self.runs_on = None;
        }
        if let Some((job, _)) = self.runs_on {
            self.found.jobs[job].runs_on.push(' ');
            self.found.jobs[job]
                .runs_on
                .push_str(&text.to_ascii_lowercase());
        }
        let line_start: usize = self.lines[index].start;
        let entry: Entry<'_> = parse_entry(&self.lines[index]);
        self.open_dashes(&entry.dashes, line_start);
        match &entry.content {
            Content::Key {
                ..
            } => return self.on_key(index, &entry),
            Content::OddKey {
                column,
            } if self.odd_key_matters(*column) => {
                return Err(refuse(
                    start,
                    "a quoted, anchored, aliased, tagged or merge key; write a plain key",
                ));
            }
            Content::Scalar if !entry.dashes.is_empty() && self.in_steps_list() => {
                return Err(refuse(
                    start,
                    "a step written as something other than a block mapping",
                ));
            }
            _ => return Ok(index + 1),
        }
    }

    /// Whether the open keys end in a `steps` sequence item.
    fn in_steps_list(&self) -> bool {
        return matches!(
            self.path().as_slice(),
            ["jobs", _, "steps", "#"] | ["runs", "steps", "#"]
        );
    }
}

/// What:
///  Walk a workflow or composite action and collect its steps.
/// Why:
///  One pass over the lines;
///  the result lists every step with its script,
///  its own shell and the default shells above it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walk(text: string): Workflow; // throws WalkError
/// ```
pub(crate) fn walk(text: &str) -> Result<Workflow, WalkError> {
    let mut walker: Walker<'_> = Walker {
        lines: read_lines(text)?,
        frames: Vec::new(),
        found: Workflow::default(),
        runs_on: None,
        content_seen: false,
    };
    let mut index: usize = 0;
    while index < walker.lines.len() {
        index = walker.step(index)?;
    }
    return Ok(walker.found);
}

/// Structure walk controls stay outside release artifacts.
#[cfg(test)]
#[path = "workflow_walk_tests.rs"]
mod tests;
