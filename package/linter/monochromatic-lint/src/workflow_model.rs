//! What:
//!  The records the workflow walker returns.
//! Why:
//!  The walker reads structure and offsets only,
//!  and the processor turns these records into virtual shell files,
//!  so neither needs the other's internals.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Workflow = { shell?: Shell; jobs: Job[]; steps: Step[] };
//! ```

/// What:
///  A refusal to read the workflow with the byte where the unsupported form starts.
/// Why:
///  Inability to read a step must surface as a processing failure,
///  never as a step that silently passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WalkError = { offset: number; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WalkError {
    /// Workflow byte offset of the unsupported form.
    pub offset: usize,
    /// What is unsupported and what to write instead.
    pub message: String,
}

/// One physical line of a block-literal `run:` body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BodyLine {
    /// First byte of the physical line,
    ///  including the indentation that is removed.
    pub envelope: usize,
    /// First byte of the script text after the removed indentation.
    pub payload: usize,
    /// One past the line's newline,
    ///  or the end of the workflow.
    pub end: usize,
}

/// The script text of one step's `run:`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RunBody {
    /// `run: command` on one line;
    ///  the range excludes a trailing comment.
    Plain {
        /// First byte of the command.
        start: usize,
        /// One past the last byte of the command.
        end: usize,
    },
    /// `run: |` and its indented lines.
    Block {
        /// The body lines in order,
        ///  without trailing blank lines.
        lines: Vec<BodyLine>,
    },
}

/// A `shell:` value and where it was written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Shell {
    /// The value without quotes or a trailing comment.
    pub value: String,
    /// Workflow byte offset of the value.
    pub offset: usize,
}

/// One step with its optional script and shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Step {
    /// Workflow byte offset of the step's first dash.
    pub anchor: usize,
    /// The index in the workflow's job list,
    ///  or `None` for a composite action's step.
    pub job: Option<usize>,
    /// The script,
    ///  or `None` for a step without `run:`.
    pub body: Option<RunBody>,
    /// The step's own `shell:`.
    pub shell: Option<Shell>,
}

/// One job's facts that decide the default shell.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Job {
    /// Everything written under `runs-on`,
    ///  lowercase,
    ///  searched for a Windows runner.
    pub runs_on: String,
    /// The job's `defaults.run.shell`.
    pub shell: Option<Shell>,
}

/// Everything the walker found.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Workflow {
    /// The workflow's `defaults.run.shell`.
    pub shell: Option<Shell>,
    /// The jobs in file order.
    pub jobs: Vec<Job>,
    /// Every step in file order.
    pub steps: Vec<Step>,
}
