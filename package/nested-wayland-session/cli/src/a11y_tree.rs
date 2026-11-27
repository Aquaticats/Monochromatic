//! Reads the hosted application's accessibility tree over AT-SPI and formats it for the `a11y` command.
//!
//! The AT-SPI role and state names live in `role_and_state_names.rs`,
//!  and the walk over an application's tree in `reading_the_tree.rs`.

/// The time type the read budget is a value of.
use std::time::Duration;

/// Sibling modules of this crate.
use crate::{
    protocol::Response,
    state::Compositor,
    wire_text::{
        encode_text,
        format_records,
    },
};

/// The AT-SPI role and state names the answer prints.
mod role_and_state_names;

/// Reading an application's tree over AT-SPI.
mod reading_the_tree;

/// The names an AT-SPI role or state number stands for,
///  part of this module's interface.
pub use role_and_state_names::{
    role_name,
    state_names,
};

/// Reading every registered application's tree,
///  part of this module's interface.
pub use reading_the_tree::{
    read_all,
    read_all_within,
};

/// Time the whole read may take.
///
/// What:
///      `const READ_BUDGET: Duration = Duration::from_secs(5);`.
/// Why:
///       A stalled application must not freeze the compositor's event loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const READ_BUDGET_MS = 5000;
/// ```
const READ_BUDGET: Duration = Duration::from_secs(5);

/// One accessible object.
///
/// What:
///      `pub struct Node { depth, role, name, states, bounds }`.
/// Why:
///       Formatting is a pure function of these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Node = { depth: number; role: string; name: string; states: string[]; bounds?: [number, number, number, number] };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Distance from the application root.
    pub depth: usize,
    /// The AT-SPI role name,
    ///  such as `push button`.
    pub role: String,
    /// The accessible name,
    ///  such as a button label.
    pub name: String,
    /// Names of the set states from a fixed subset.
    pub states: Vec<&'static str>,
    /// Screen bounds as `(x, y, width, height)` when the object has a component interface.
    pub bounds: Option<(i32, i32, i32, i32)>,
}

/// Format one node as a record.
///
/// What:
///      `fn format_node(node: &Node) -> String`.
///  `depth=N role=... name=... states=a,b bounds=x,y,w,h`,
///  text percent-encoded.
/// Why:
///       A name with a space or a line break must not split a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const formatNode = (node: Node) => `depth=${node.depth} role=${enc(node.role)} ...`;
/// ```
fn format_node(node: &Node) -> String {
    let bounds = match node.bounds {
        Some((x, y, width, height)) => format!("{x},{y},{width},{height}"),
        None => String::new(),
    };
    return format!(
        "depth={} role={} name={} states={} bounds={bounds}",
        node.depth,
        encode_text(&node.role),
        encode_text(&node.name),
        node.states.join(","),
    );
}

/// Format the `a11y` answer.
///
/// What:
///      `pub fn format_tree(nodes: &[Node]) -> String`.
/// Why:
///       Same list grammar as the other listing commands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const formatTree = (nodes: Node[]) => formatRecords(nodes.map(formatNode));
/// ```
#[must_use]
pub fn format_tree(nodes: &[Node]) -> String {
    let records: Vec<String> = nodes.iter().map(format_node).collect();
    return format_records(&records);
}

/// Answer `a11y`.
///
/// What:
///      `pub fn response(state: &Compositor) -> Response`.
///  `ok 0` when no application has registered yet.
/// Why:
///       The command a test or reviewer uses to see what a screen reader would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function response(state: Compositor): Response { ... }
/// ```
#[must_use]
pub fn response(state: &Compositor) -> Response {
    let Some(service) = state.a11y.as_ref() else {
        return Response::Err("no accessibility service (start with --accessibility)".to_string());
    };
    match read_all(service) {
        Ok(nodes) => return Response::OkWith(format_tree(&nodes)),
        Err(error) => return Response::Err(format!("{error:#}")),
    }
}

/// What:
///      `#[cfg(test)] #[path = "a11y_tree_tests.rs"] mod tests;`.
/// Why:
///       Keep the formatting tests beside the formatter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./a11y_tree_tests";
/// ```
#[cfg(test)]
#[path = "a11y_tree_tests.rs"]
mod tests;
