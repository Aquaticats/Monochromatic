//! What:
//! What an `install` argument names:
//! a tool id the plan already holds,
//! or a new tool a request describes,
//! with the route its prefix selects and the binaries that route's own source lists.
//! Why:
//! The owner's answer of 2026-11-13 is that the argument is "a request or an existing id":
//! `mpm i -g cargo:fastmod` adds and installs a new tool,
//! `mpm i -g tokei` installs the entry the plan holds under that id,
//! and a bare name that is neither is refused rather than guessed.
//! A route whose entry needs more than a request can say (a download address,
//! a checksum,
//! an archive layout,
//! the name of the crate inside a repository) cannot be added here,
//! and the refusal names the plan file and the README section that shows the entry's shape.
//! The work is one file per step,
//! named for what it holds:
//! the routes a request can add (`the_routes_a_request_can_add`),
//! the sentences a refusal says (`the_words_a_refusal_says`),
//! the binaries a new entry exposes (`the_binaries_of_a_new_entry`),
//! what one argument names (`what_an_argument_names`),
//! and adding the entries to the plan (`adding_them_to_the_plan`).
//! A caller imports from this file and never from a part.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-argument.ts
//! export type Named = { kind: 'existing'; id: string } | { kind: 'new'; declaration: Declaration };
//! export function namedTools(plan: Plan, context: Context, arguments: string[]): Named[]
//! export * from './install_argument/the_routes_a_request_can_add.ts';
//! export * from './install_argument/the_words_a_refusal_says.ts';
//! export * from './install_argument/the_binaries_of_a_new_entry.ts';
//! export * from './install_argument/what_an_argument_names.ts';
//! export * from './install_argument/adding_them_to_the_plan.ts';
//! ```
//! The last part adds the entries it resolved,
//! under the scope's OS lock,
//! once the text is known.

/// Adding the entries the arguments described to the plan file.
mod adding_them_to_the_plan;
/// The binaries of a new entry,
/// read from the source its route owns.
mod the_binaries_of_a_new_entry;
/// The routes a request's prefix can add,
/// in the order the refusal message lists them.
mod the_routes_a_request_can_add;
/// The sentences a person reads when an argument cannot be added.
mod the_words_a_refusal_says;
/// What one argument names.
mod what_an_argument_names;

/// Re-exports the report of what adding the arguments did,
/// the fault point the real command passes,
/// and the write of the plan itself.
pub use self::adding_them_to_the_plan::{
    AddReport,
    add_named_tools,
    nothing_stops_the_run,
};
/// Re-exports the route a request's prefix selects,
/// the routes the command can add,
/// and where each one's binaries come from.
pub use self::the_routes_a_request_can_add::{
    ADDABLE_ROUTES,
    AddableRoute,
    BinsFrom,
};
/// Re-exports why a new entry cannot be added at all.
pub use self::the_words_a_refusal_says::AddRefusal;
/// Re-exports what one argument named,
/// in the step before its entry is built and once it is.
pub use self::what_an_argument_names::{
    Named,
    Target,
    named,
    target,
};
