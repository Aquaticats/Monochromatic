//! What:
//! What one `install` argument names:
//! a tool id the plan already holds,
//! or a request the command can add,
//! together with the entry that request describes.
//! Why:
//! An id the plan holds is installed as it is,
//! so the plan is never edited for a tool that is already declared;
//! and a run that may not write refuses here,
//! before a registry is asked anything.
//! The entry a request describes is built here so the tests can read every field it fills.
//! A caller imports from `install_argument` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export type Target = { kind: 'existing'; id: string } | { kind: 'request'; request: Request };
//! export type Named = { kind: 'existing'; id: string } | { kind: 'new'; declaration: Declaration };
//! export function target(plan: Plan, argument: string, label: string): Target;
//! export function named(plan: Plan, context: Context, argument: string, label: string): Named;
//! ```

/// Imports `Context` from `super::super::context`.
use super::super::context::Context;
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports the model types an entry is built from.
use super::super::model::{
    Declaration,
    Options,
    Request,
};
/// Imports the plan the argument is looked up in.
use super::super::plan::Plan;

/// Imports the binaries a new entry exposes.
use super::the_binaries_of_a_new_entry::bins_for;
/// Imports the route a request's prefix selects.
use super::the_routes_a_request_can_add::addable_route;
/// Imports the sentences for an argument the command cannot add.
use super::the_words_a_refusal_says::{
    neither,
    unaddable,
};
/// What:
/// What one argument names,
/// in the step before its entry is built.
/// Why:
/// A run that may not write the plan refuses at this point,
/// so it never asks a registry for the binaries of a tool it will not add.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Target {
    /// An id the plan holds.
    Existing(
        /// The tool id.
        String,
    ),
    /// A request whose route the command can add.
    Request(
        /// The request as the person wrote it,
        /// with `latest` for a range they left out.
        Request,
    ),
}

/// What one argument named.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Named {
    /// An id the plan holds.
    Existing(
        /// The tool id.
        String,
    ),
    /// A tool the plan does not hold yet,
    /// described by its request.
    New(
        /// The entry to add.
        Box<Declaration>,
    ),
}

/// What:
/// The entry a request describes:
/// its id is the request's source text,
/// its route is the one its prefix selects,
/// and its binaries are read from that route's own source.
/// Why:
/// Nothing here names a person's tool for them:
/// every field comes from the request or from the registry that publishes the program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryFromRequest(context: Context, request: Request): Declaration  // throws
/// ```
pub(super) fn entry_from_request(
    context: &Context,
    request: Request,
) -> Result<Declaration, ToolchainError> {
    let source: String = request.source.clone();
    let Some(route) = addable_route(&source) else {
        return Err(ToolchainError::config(format!(
            "{source} selects no route this command can add"
        )));
    };
    let bins: Vec<String> = bins_for(context, route, &request)?;
    return Ok(Declaration {
        id: request.source.clone(),
        route: route.route,
        request,
        manager: None,
        package_id: None,
        bins,
        os: None,
        env: Vec::new(),
        options: Options::default(),
    });
}

/// What:
/// What one argument names,
/// before any entry is built or looked up:
/// an id the plan holds,
/// or a request the command can add.
/// Why:
/// An id the plan holds is installed as it is,
/// so the plan is never edited for a tool that is already declared;
/// and a run that may not write refuses here,
/// before a registry is asked anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function target(plan: Plan, argument: string, label: string): { kind: 'existing'; id: string } | { kind: 'request'; request: Request }
/// ```
pub fn target(
    plan: &Plan,
    argument: &str,
    label: &str,
) -> Result<Target, ToolchainError> {
    for declaration in &plan.tools {
        if declaration.id == argument {
            return Ok(Target::Existing(argument.to_string()));
        }
    }
    let request: Request = match Request::parse(argument) {
        Ok(parsed) => parsed,
        Err(_) => return Err(neither(argument, label)),
    };
    if addable_route(&request.source).is_none() {
        return Err(unaddable(argument, &request.source, label));
    }
    return Ok(Target::Request(request));
}

/// What:
/// What one argument named once its entry is built,
/// or why it names nothing the command can act on.
/// Why:
/// The tests read the entry this returns to prove every field comes from the request or its route's source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function named(plan: Plan, context: Context, argument: string, label: string): Named
/// ```
pub fn named(
    plan: &Plan,
    context: &Context,
    argument: &str,
    label: &str,
) -> Result<Named, ToolchainError> {
    return match target(plan, argument, label)? {
        Target::Existing(id) => Ok(Named::Existing(id)),
        Target::Request(request) => Ok(Named::New(Box::new(entry_from_request(context, request)?))),
    };
}
