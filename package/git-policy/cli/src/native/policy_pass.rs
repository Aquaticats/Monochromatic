//! What:
//!  One complete policy pass over the shipped policies:
//!  the built-in policies,
//!  then
//!       the fixed argument transforms of a forwarded command,
//!  then the optional policies.
//! Why:
//!  Every lifecycle runs the same three stages in the same order,
//!  and each stage can
//!      end the pass.
//!  Keeping the sequence in one place means a wrapped command and a
//!      direct `git cli-git check` cannot drift apart in what they run or when they stop.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const result = await runPolicyEngine({ args, trigger, config, selectedPolicyIds });
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   The pass joins the stage runner,
///  the shipped checks and the fixed transforms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { runPolicyStage } from './policy_engine.ts';
/// ```
use super::policy_checks::ShippedChecks;
use super::policy_engine::{StageEnd, StageRequest, StageResult, run_policy_stage};
use super::policy_events::PolicyEvent;
use super::policy_registry::{POLICY_REGISTRY, PolicyId};
use super::policy_transforms::{TransformResult, apply_fixed_transforms};
use super::policy_trigger::Trigger;
use super::repository_facts::RepositoryFacts;
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   The pass hands back the arguments Git will receive,
///  byte for byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
///  What one pass produced.
///  A `struct` is a record with named fields;
///  `Vec<T>` is an
///       owned list.
///  `#[derive(...)]` asks the compiler to generate copying,
///  debug
///       printing and `==`.
/// Why:
///   The caller needs the arguments to forward,
///  every event in emission order,
///  and
///       how the pass ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PassResult = { args: string[]; events: PolicyEvent[]; end: StageEnd };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PassResult {
    /// The arguments after the fixed transforms;
    ///  the input arguments when the pass ended
    /// before or inside them.
    pub arguments: Vec<OsString>,
    /// Every event of the pass,
    ///  in emission order.
    pub events: Vec<PolicyEvent>,
    /// How the pass ended.
    pub end: StageEnd,
}

/// What:
///  The shipped policies of one kind,
///  in registry order.
///  `optional` selects the
///       policies that are off unless the configuration lists them;
///  `Vec<PolicyId>` is the
///       owned result.
/// Why:
///   The registry is the only list of policies;
///  the two stages are its two halves,
///  so
///       a policy added there lands in exactly one stage without a second list to update.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const policiesOfKind = (optional: boolean) => POLICY_REGISTRY.filter(p => p.offUnlessListed === optional).map(p => p.id);
/// ```
pub fn policies_of_kind(optional: bool) -> Vec<PolicyId> {
    // `Vec::<PolicyId>::new()` is an empty owned list; `mut` allows pushing.
    let mut policies: Vec<PolicyId> = Vec::<PolicyId>::new();
    // `for descriptor in POLICY_REGISTRY` borrows each compiled-in row in order.
    for descriptor in POLICY_REGISTRY {
        if descriptor.off_unless_listed == optional {
            policies.push(descriptor.id);
        }
    }
    return policies;
}

/// What:
///  Run one pass for `request` over `checks`.
///  `<F: RepositoryFacts>` says the function
///       works with any one facts provider `F`;
///  `&mut ShippedChecks<F>` lends the shipped
///       policies and their provider for writing,
///  because checks cache repository facts.
/// Why:
///   The order is the installed wrapper's.
///  Built-in policies read the command before
///       it is rewritten.
///  The fixed transforms run only for a forwarded command.
///  Optional
///       policies run last.
///  A stage that did not complete ends the pass with the
///       arguments it was given,
///  and nothing after it runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runPolicyPass(request: StageRequest, checks: ShippedChecks): Promise<PassResult>;
/// ```
pub fn run_policy_pass<F: RepositoryFacts>(
    request: &StageRequest,
    checks: &mut ShippedChecks<F>,
) -> PassResult {
    // `.as_slice()` lends an owned list as a borrowed view.
    let built_in: StageResult =
        run_policy_stage(request, policies_of_kind(false).as_slice(), checks);
    // `mut` allows the later stages to append their events.
    let mut events: Vec<PolicyEvent> = built_in.events;
    if built_in.end != StageEnd::Completed {
        return PassResult {
            // `.clone()` copies the untouched input arguments for the caller.
            arguments: checks.arguments.clone(),
            events,
            end: built_in.end,
        };
    }
    // `mut` allows the transforms to replace the arguments.
    let mut arguments: Vec<OsString> = checks.arguments.clone();
    if request.trigger == Trigger::PreForward {
        // `&mut checks.facts` lends the provider for writing: a transform may ask Git one fact.
        let transformed: TransformResult = apply_fixed_transforms(
            arguments.as_slice(),
            request.controls.commit_only_escaped,
            &mut checks.facts,
        );
        // A complete transform stage reports nothing but rejections, so any event of a
        // complete stage is one; an incomplete stage is handled first.
        let rejected: bool = !transformed.events.is_empty();
        // `.extend(list)` moves every item of `list` onto the end.
        events.extend(transformed.events);
        if !transformed.complete {
            return PassResult {
                arguments,
                events,
                end: StageEnd::Failed,
            };
        }
        if rejected && !request.controls.keep_going {
            return PassResult {
                arguments,
                events,
                end: StageEnd::Stopped,
            };
        }
        arguments = transformed.arguments;
    }
    let optional: StageResult =
        run_policy_stage(request, policies_of_kind(true).as_slice(), checks);
    events.extend(optional.events);
    return PassResult {
        arguments,
        events,
        end: optional.end,
    };
}

/// Stage order,
///  stopping and argument hand-over stay out of the release executable.
#[cfg(test)]
#[path = "policy_pass_tests.rs"]
mod tests;
