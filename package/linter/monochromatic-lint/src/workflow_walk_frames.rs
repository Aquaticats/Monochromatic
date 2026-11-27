//! What:
//!  What one walk keeps while it reads a workflow:
//!  the lines,
//!  the keys and sequence items still open,
//!  the runs-on text of the job it is reading,
//!  and the steps found so far.
//! Why:
//!  The walk closes a frame when a line is no deeper than it,
//!  opens a sequence item for each dash of a line,
//!  and starts a step only where a `steps` key owns the dash;
//!  keeping that bookkeeping in one place lets the file that reads a line
//!  ask which keys are open without restating how the stack moves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Frame = { indent: number; name: string };
//! type Walker = {
//!   lines: Line[]; frames: Frame[]; found: Workflow;
//!   runsOn: [number, number] | undefined; contentSeen: boolean;
//! };
//! ```

/// Import one line of a workflow file.
use crate::workflow_lines::Line;
/// Import the records the walk fills in.
use crate::workflow_model::{
    Step,
    Workflow,
};

/// One open key or sequence item.
pub(crate) struct Frame {
    /// Column of the key or dash.
    pub(crate) indent: usize,
    /// The key,
    ///  or `#` for a sequence item.
    pub(crate) name: String,
}

/// The walk's state.
pub(crate) struct Walker<'a> {
    /// The lines of the file.
    pub(crate) lines: Vec<Line<'a>>,
    /// Open keys and sequence items.
    pub(crate) frames: Vec<Frame>,
    /// What has been found.
    pub(crate) found: Workflow,
    /// The open `runs-on` key as job index and column,
    ///  while deeper lines still belong to it.
    pub(crate) runs_on: Option<(usize, usize)>,
    /// Whether any content line has been read,
    ///  to refuse a second document.
    pub(crate) content_seen: bool,
}

/// Frame bookkeeping.
impl Walker<'_> {
    /// The names of the open keys,
    ///  outermost first.
    pub(crate) fn path(&self) -> Vec<&str> {
        let mut names: Vec<&str> = Vec::new();
        for frame in &self.frames {
            names.push(frame.name.as_str());
        }
        return names;
    }

    /// Close every frame at or right of `column`.
    pub(crate) fn close_from(
        &mut self,
        column: usize,
    ) {
        loop {
            let Some(top) = self.frames.last() else {
                return;
            };
            if top.indent < column {
                return;
            }
            self.frames.pop();
        }
    }

    /// Open the sequence items for the dashes of a line,
    ///  starting a step when the first dash sits under a `steps` key.
    pub(crate) fn open_dashes(
        &mut self,
        dashes: &[usize],
        offset: usize,
    ) {
        let Some(first) = dashes.first() else {
            return;
        };
        loop {
            let Some(top) = self.frames.last() else {
                break;
            };
            // Deeper frames close, and an earlier sequence item at the same column closes.
            if top.indent <= *first && !(top.name == "#" && top.indent == *first) {
                break;
            }
            self.frames.pop();
        }
        let owner: Option<Option<usize>> = match self.path().as_slice() {
            ["jobs", _, "steps"] => Some(Some(self.found.jobs.len().saturating_sub(1))),
            ["runs", "steps"] => Some(None),
            _ => None,
        };
        if let Some(job) = owner {
            self.found.steps.push(Step {
                anchor: offset + *first,
                job,
                body: None,
                shell: None,
            });
        }
        for dash in dashes {
            self.frames.push(Frame {
                indent: *dash,
                name: String::from("#"),
            });
        }
    }

    /// Whether an unread key form at `column` sits where the walk needs a plain key.
    pub(crate) fn odd_key_matters(
        &self,
        column: usize,
    ) -> bool {
        let mut names: Vec<&str> = Vec::new();
        for frame in &self.frames {
            if frame.indent < column {
                names.push(frame.name.as_str());
            }
        }
        return crate::workflow_shape::odd_key_matters(&names);
    }
}
