//! Reads the hosted application's accessibility tree over AT-SPI and formats it for the `a11y` command.

/// Standard library items.
use std::{sync::mpsc, time::Duration};

/// Error handling with context.
use anyhow::{Context, Result};
/// Structured logging.
use tracing::{debug, warn};
/// The D-Bus client and server library.
use zbus::{blocking::{Connection, Proxy}, zvariant::OwnedObjectPath};

/// Sibling modules of this crate.
use crate::{
    a11y_service::{A11yService, Application},
    protocol::Response,
    state::Compositor,
    wire_text::{encode_text, format_records},
};

/// Deepest level the reader descends to.
///
/// What:
///      `const MAX_DEPTH: usize = 40;`.
/// Why:
///       A cycle or a very deep tree must end the walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_DEPTH = 40;
/// ```
const MAX_DEPTH: usize = 40;

/// Most nodes one read returns.
///
/// What:
///      `const MAX_NODES: usize = 4000;`.
/// Why:
///       Bounds the answer and the time spent in the event loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_NODES = 4000;
/// ```
const MAX_NODES: usize = 4000;

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

/// AT-SPI state bit numbers this reader names.
///
/// What:
///      `const STATE_NAMES: &[(u32, &str)]`.
/// Why:
///       The state vector is a 64-bit set;
///  a test wants `focused` and `checked`,
///  not bit numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STATE_NAMES: [number, string][] = [[4, "checked"], [8, "enabled"], ...];
/// ```
const STATE_NAMES: &[(u32, &str)] = &[
    (4, "checked"),
    (5, "collapsed"),
    (7, "editable"),
    (8, "enabled"),
    (10, "expanded"),
    (11, "focusable"),
    (12, "focused"),
    (20, "pressed"),
    (23, "selected"),
    (24, "sensitive"),
    (25, "showing"),
    (30, "visible"),
];

/// AT-SPI role numbers with their role names,
///  from the `Role` enumeration of the AT-SPI specification.
///
/// What:
///      `const ROLE_NAMES: &[&str]`.
///  The index is the role number.
/// Why:
///       Applications answer `GetRole` with the number only,
///  and a test wants `push button`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ROLE_NAMES = ["invalid", "accelerator label", "alert", ...];
/// ```
const ROLE_NAMES: &[&str] = &[
    "invalid", "accelerator label", "alert", "animation", "arrow", "calendar", "canvas", "check box",
    "check menu item", "color chooser", "column header", "combo box", "date editor", "desktop icon",
    "desktop frame", "dial", "dialog", "directory pane", "drawing area", "file chooser", "filler",
    "focus traversable", "font chooser", "frame", "glass pane", "html container", "icon", "image",
    "internal frame", "label", "layered pane", "list", "list item", "menu", "menu bar", "menu item",
    "option pane", "page tab", "page tab list", "panel", "password text", "popup menu", "progress bar",
    "push button", "radio button", "radio menu item", "root pane", "row header", "scroll bar",
    "scroll pane", "separator", "slider", "spin button", "split pane", "status bar", "table",
    "table cell", "table column header", "table row header", "tearoff menu item", "terminal", "text",
    "toggle button", "tool bar", "tool tip", "tree", "tree table", "unknown", "viewport", "window",
    "extended", "header", "footer", "paragraph", "ruler", "application", "autocomplete", "edit bar",
    "embedded", "entry", "chart", "caption", "document frame", "heading", "page", "section",
    "redundant object", "form", "link", "input method window", "table row", "tree item",
    "document spreadsheet", "document presentation", "document text", "document web", "document email",
    "comment", "list box", "grouping", "image map", "notification", "info bar", "level bar", "title bar",
    "block quote", "audio", "video", "definition", "article", "landmark", "log", "marquee", "math",
    "rating", "timer", "static", "math fraction", "math root", "subscript", "superscript",
    "description list", "description term", "description value", "footnote", "content deletion",
    "content insertion", "mark", "suggestion", "push button menu",
];

/// Name an AT-SPI role number.
///
/// What:
///      `pub fn role_name(role: u32) -> String`.
///  `role-N` for a number the table does not know.
/// Why:
///       Newer specification revisions add roles;
///  an unknown one must still print.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const roleName = (role: number) => ROLE_NAMES[role] ?? `role-${role}`;
/// ```
#[must_use]
pub fn role_name(role: u32) -> String {
    return ROLE_NAMES.get(role as usize).map_or_else(|| return format!("role-{role}"), |name| return (*name).to_string())
}

/// Name the set bits of an AT-SPI state vector.
///
/// What:
///      `pub fn state_names(words: &[u32]) -> Vec<&'static str>`.
///  Bit `n` is bit `n % 32` of word `n / 32`.
/// Why:
///       Decoding is pure and tested without a bus.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stateNames(words: number[]): string[] { ... }
/// ```
#[must_use]
pub fn state_names(words: &[u32]) -> Vec<&'static str> {
    let mut names = Vec::new();
    for (bit, name) in STATE_NAMES {
        let word = words.get((bit / 32) as usize).copied().unwrap_or(0);
        if word & (1 << (bit % 32)) != 0 {
            names.push(*name);
        }
    }
    return names
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
    let bounds = node.bounds.map_or(String::new(), |(x, y, width, height)| return format!("{x},{y},{width},{height}"));
    return format!(
        "depth={} role={} name={} states={} bounds={bounds}",
        node.depth,
        encode_text(&node.role),
        encode_text(&node.name),
        node.states.join(","),
    )
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
    return format_records(&records)
}

/// Build a proxy for one AT-SPI interface of one object.
///
/// What:
///      `fn proxy<'a>(connection: &Connection, bus_name: &'a str, path: &'a str, interface: &'a str) -> Result<Proxy<'a>>`.
/// Why:
///       Every read goes through these proxies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function proxy(connection: Connection, busName: string, path: string, iface: string): Proxy { ... }
/// ```
fn proxy<'a>(connection: &Connection, bus_name: &'a str, path: &'a str, interface: &'a str) -> Result<Proxy<'a>> {
    return Proxy::new(connection, bus_name, path, interface)
        .with_context(|| format!("opening {interface} of {path} on {bus_name}"))
}

/// Read one node and queue its children.
///
/// What:
///      `fn read_node(connection, bus_name, path, depth) -> Result<(Node, Vec<(String, String)>)>`.
/// Why:
///       The walk is iterative over a work list,
///  so a deep tree cannot overflow the stack.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readNode(connection: Connection, busName: string, path: string, depth: number): [Node, Child[]] { ... }
/// ```
fn read_node(connection: &Connection, bus_name: &str, path: &str, depth: usize) -> Result<(Node, Vec<(String, String)>)> {
    let accessible = proxy(connection, bus_name, path, "org.a11y.atspi.Accessible")?;
    let role = role_name(accessible.call::<_, _, u32>("GetRole", &()).context("GetRole")?);
    let name: String = accessible.get_property("Name").unwrap_or_default();
    let states: Vec<u32> = accessible.call("GetState", &()).unwrap_or_default();
    let children: Vec<(String, OwnedObjectPath)> = accessible.call("GetChildren", &()).unwrap_or_default();
    let bounds = match proxy(connection, bus_name, path, "org.a11y.atspi.Component") {
        Ok(component) => component.call::<_, _, (i32, i32, i32, i32)>("GetExtents", &(0_u32,)).ok(),
        Err(error) => {
            debug!("no component interface for {path}: {error:#}");
            None
        }
    };
    let node = Node { depth, role, name, states: state_names(&states), bounds };
    let queued = children.into_iter().map(|(child_bus, child_path)| return (child_bus, child_path.as_str().to_string())).collect();
    return Ok((node, queued))
}

/// Read the whole tree of one application.
///
/// What:
///      `fn read_application(connection: &Connection, application: &Application) -> Result<Vec<Node>>`.
///  Depth first,
///  parents before children,
///  stopping at `MAX_DEPTH` and `MAX_NODES`.
/// Why:
///       Gives the order a person reads a screen in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readApplication(connection: Connection, application: Application): Node[] { ... }
/// ```
fn read_application(connection: &Connection, application: &Application) -> Result<Vec<Node>> {
    let mut nodes = Vec::new();
    let mut pending = vec![(application.bus_name.clone(), application.path.clone(), 0_usize)];
    while let Some((bus_name, path, depth)) = pending.pop() {
        if nodes.len() >= MAX_NODES || depth > MAX_DEPTH {
            warn!("accessibility tree read stopped at the node or depth limit");
            break;
        }
        let (node, children) = read_node(connection, &bus_name, &path, depth)?;
        nodes.push(node);
        for (child_bus, child_path) in children.into_iter().rev() {
            pending.push((child_bus, child_path, depth + 1));
        }
    }
    return Ok(nodes)
}

/// Read every registered application on a helper thread with a time budget.
///
/// What:
///      `pub fn read_all(service: &A11yService) -> Result<Vec<Node>>`.
/// Why:
///       The calls wait for another process,
///  which may be slow or stopped,
///  and the compositor's event loop must not wait forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readAll(service: A11yService): Promise<Node[]> { ... } // with a 5 s timeout
/// ```
pub fn read_all(service: &A11yService) -> Result<Vec<Node>> {
    let connection = service.connection().clone();
    let applications = service.applications();
    let (sender, receiver) = mpsc::channel();
    std::thread::Builder::new()
        .name("nws-a11y-read".to_string())
        .spawn(move || {
            let mut nodes = Vec::new();
            for application in &applications {
                match read_application(&connection, application) {
                    Ok(mut found) => nodes.append(&mut found),
                    Err(error) => {
                        let _ = sender.send(Err(error));
                        return;
                    }
                }
            }
            let _ = sender.send(Ok(nodes));
        })
        .context("starting the accessibility reader thread")?;
    return receiver
        .recv_timeout(READ_BUDGET)
        .map_err(|error| return anyhow::anyhow!("the application did not answer its accessibility tree within {READ_BUDGET:?}: {error}"))?
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
        return Response::Err("no accessibility service (start with --accessibility)".to_string())
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
