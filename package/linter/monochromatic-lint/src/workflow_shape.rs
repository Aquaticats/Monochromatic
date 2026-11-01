//! What:
//!  The meaning of a key at a path in a workflow or composite action.
//! Why:
//!  The walk reads a `run:` only where a step's key sits,
//!  so the places that matter are named once,
//!  apart from the indentation bookkeeping.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function shape(path: string[]): Shape;
//! ```

/// What a key at some path means to the walk.
pub(crate) enum Shape {
    /// A key that must be a block,
    ///  not an inline value:
    ///  `jobs`,
    ///  `defaults`,
    ///  `steps` and the like.
    Structure,
    /// A job's id key.
    Job,
    /// A job's `runs-on`.
    RunsOn,
    /// A `defaults.run.shell`,
    ///  at the workflow level or a job's.
    DefaultShell,
    /// A key of a step mapping.
    StepKey,
    /// Anything else.
    Other,
}

/// Classify a key by the names of the open keys above it,
///  ending with its own.
pub(crate) fn shape(path: &[&str]) -> Shape {
    match path {
        ["jobs"]
        | ["jobs", _, "steps"]
        | ["jobs", _, "defaults"]
        | ["jobs", _, "defaults", "run"]
        | ["defaults"]
        | ["defaults", "run"]
        | ["runs"]
        | ["runs", "steps"] => return Shape::Structure,
        ["jobs", _] => return Shape::Job,
        ["jobs", _, "runs-on"] => return Shape::RunsOn,
        ["jobs", _, "defaults", "run", "shell"] | ["defaults", "run", "shell"] => {
            return Shape::DefaultShell;
        }
        ["jobs", _, "steps", "#", _] | ["runs", "steps", "#", _] => return Shape::StepKey,
        _ => return Shape::Other,
    }
}

/// What:
///  Whether a key written in a form the walk does not read would sit where the walk needs a plain key:
///  a job id,
///  a key of a step,
///  or a structural key such as `steps` or `defaults`.
/// Why:
///  The same form elsewhere,
///  such as a flow mapping in a build matrix,
///  never holds a step and is ignored.
///  `path` is the open keys above the odd key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function oddKeyMatters(path: string[]): boolean;
/// ```
pub(crate) fn odd_key_matters(path: &[&str]) -> bool {
    return matches!(
        path,
        ["jobs"]
            | ["jobs", _]
            | ["jobs", _, "steps"]
            | ["jobs", _, "steps", "#"]
            | ["jobs", _, "defaults"]
            | ["jobs", _, "defaults", "run"]
            | ["defaults"]
            | ["defaults", "run"]
            | ["runs"]
            | ["runs", "steps"]
            | ["runs", "steps", "#"]
    );
}
