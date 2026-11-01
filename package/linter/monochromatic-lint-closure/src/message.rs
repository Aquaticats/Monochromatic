//! What:
//!  The text of the lint message and help,
//!  built from plain data.
//! Why:
//!  Text assembly has no compiler types in it,
//!  so ordinary unit tests can check every wording branch.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function primaryMessage(captures: string[], placement: Placement): string;
//! function helpMessage(captures: string[], placement: Placement): string;
//! ```

/// Import the placement categories.
use crate::placement::Placement;

/// The Dylint library name that appears in the suppression attribute (`dylint_lib = "..."`).
pub const LIBRARY_NAME: &str = "monochromatic_lint_closure";
/// The lint name that appears in the suppression attribute.
pub const LINT_NAME: &str = "avoidable_closure";

/// What:
///  Render the captured names as backticked identifiers joined by commas.
/// Why:
///  The message names the captures so the author sees which state the closure owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const list = captures.map((name) => '`' + name + '`').join(', ');
/// ```
pub fn capture_list(captures: &[String]) -> String {
    let mut list: String = String::new();
    for (index, name) in captures.iter().enumerate() {
        if index > 0 {
            list.push_str(", ");
        }
        list.push('`');
        list.push_str(name);
        list.push('`');
    }
    return list;
}

/// What:
///  The sentence fragment that says why the placement condition failed.
/// Why:
///  The author must learn which of the two conditions failed,
///  and for the second one,
///  in which way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function placementFailure(placement: Placement): string;
/// ```
fn placement_failure(placement: Placement) -> &'static str {
    return match placement {
        Placement::KeptByCallee => "",
        Placement::NotKeptByCallee => {
            "the callee parameter it is passed to is not required to be `'static`, so the callee does not keep it after the call"
        }
        Placement::LetInitializer => {
            "it is the initializer of a `let` statement, not a direct call argument"
        }
        Placement::UnresolvedCallee => {
            "the callee is not a declared function, so no parameter bound can be read from it"
        }
        Placement::Other => "it is not a direct argument of a call or method call",
    };
}

/// What:
///  The one-line message:
///  which condition failed,
///  naming the captures when there are any.
/// Why:
///  The criterion has two conditions (captures something,
///  and is handed to a callee that keeps it);
///  the message names the failed one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function primaryMessage(captures: string[], placement: Placement): string;
/// ```
pub fn primary_message(captures: &[String], placement: Placement) -> String {
    let failure: &str = placement_failure(placement);
    if captures.is_empty() {
        if failure.is_empty() {
            return String::from(
                "this closure captures no variables, so a named function can take its place",
            );
        }
        return format!(
            "this closure captures no variables, so a named function can take its place; also, {failure}"
        );
    }
    return format!(
        "this closure captures {}, but {failure}",
        capture_list(captures)
    );
}

/// What:
///  The suppression attribute text the help shows,
///  with the author's reason left as a placeholder.
/// Why:
///  A suppression must name this library,
///  so the attribute is only active when Dylint loaded it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attribute = `#[cfg_attr(dylint_lib = "${LIBRARY_NAME}", expect(${LINT_NAME}, reason = "..."))]`;
/// ```
fn suppression_attribute() -> String {
    return format!(
        "#[cfg_attr(dylint_lib = \"{LIBRARY_NAME}\", expect({LINT_NAME}, reason = \"...\"))]"
    );
}

/// What:
///  The remedy text:
///  a named function when nothing is captured,
///  the attribute when the callee keeps the state.
/// Why:
///  The author needs one concrete next step for each way the criterion can fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function helpMessage(captures: string[], placement: Placement): string;
/// ```
pub fn help_message(captures: &[String], placement: Placement) -> String {
    let attribute: String = suppression_attribute();
    if captures.is_empty() {
        return String::from(
            "move the body into a named function and pass the function's path; a closure is accepted only when it \
             captures a variable and the callee keeps it past the call",
        );
    }
    if placement == Placement::LetInitializer {
        return format!(
            "pass the closure directly to the call that needs it, or restructure with a named function or a loop; \
             if the callee keeps the state past the call, attach {attribute} to the `let` statement or to an \
             enclosing item (rustc rejects an attribute on a closure in a `let` initializer with E0658)"
        );
    }
    return format!(
        "replace the closure with a named function that takes the state as parameters, or with a loop; if the \
         callee keeps the state past the call, attach {attribute} to the closure argument or to an enclosing item"
    );
}
