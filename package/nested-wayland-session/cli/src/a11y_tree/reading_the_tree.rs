//! Reading one application's accessibility tree over AT-SPI,
//!  and every registered application's tree inside a time budget.
//!
//! Split out of `a11y_tree.rs` to keep that file under the code-line limit.

/// Time the whole read may take.
use std::time::Duration;

/// Error handling with context.
use anyhow::{
    Context,
    Result,
};
/// Structured logging.
use tracing::{
    debug,
    warn,
};
/// The D-Bus client and server library.
use zbus::{
    Connection,
    OwnedObjectPath,
    Proxy,
};

/// The service that holds the registered applications.
use crate::a11y_service::{
    A11yService,
    Application,
};
/// Wait for one blocking step inside a budget.
use crate::bus_step_bound::block_on_within;
/// Error text with the step that failed named in it.
use crate::lazy_error_text::with_message;

/// One accessible object,
///  the two lookups that name its role and states,
///  and the budget a whole read may take.
use super::{
    Node,
    READ_BUDGET,
    role_name,
    state_names,
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

/// Build a proxy for one AT-SPI interface of one object.
///
/// What:
///      `async fn proxy<'a>(connection: &Connection, bus_name: &'a str, path: &'a str, interface: &'a str) -> Result<Proxy<'a>>`.
/// Why:
///       Every read goes through these proxies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function proxy(connection: Connection, busName: string, path: string, iface: string): Proxy { ... }
/// ```
async fn proxy<'a>(
    connection: &Connection,
    bus_name: &'a str,
    path: &'a str,
    interface: &'a str,
) -> Result<Proxy<'a>> {
    return with_message(
        Proxy::new(connection, bus_name, path, interface).await,
        format_args!("opening {interface} of {path} on {bus_name}"),
    );
}

/// Read one node and queue its children.
///
/// What:
///      `async fn read_node(connection, bus_name, path, depth) -> Result<(Node, Vec<(String, String)>)>`.
/// Why:
///       The walk is iterative over a work list,
///  so a deep tree cannot overflow the stack.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readNode(connection: Connection, busName: string, path: string, depth: number): [Node, Child[]] { ... }
/// ```
async fn read_node(
    connection: &Connection,
    bus_name: &str,
    path: &str,
    depth: usize,
) -> Result<(Node, Vec<(String, String)>)> {
    let accessible = proxy(connection, bus_name, path, "org.a11y.atspi.Accessible").await?;
    let role = role_name(
        accessible
            .call::<_, _, u32>("GetRole", &())
            .await
            .context("GetRole")?,
    );
    let name: String = accessible.get_property("Name").await.unwrap_or_default();
    let states: Vec<u32> = accessible.call("GetState", &()).await.unwrap_or_default();
    let children: Vec<(String, OwnedObjectPath)> = accessible
        .call("GetChildren", &())
        .await
        .unwrap_or_default();
    let bounds = match proxy(connection, bus_name, path, "org.a11y.atspi.Component").await {
        Ok(component) => component
            .call::<_, _, (i32, i32, i32, i32)>("GetExtents", &(0_u32,))
            .await
            .ok(),
        Err(error) => {
            debug!("no component interface for {path}: {error:#}");
            None
        }
    };
    let node = Node {
        depth,
        role,
        name,
        states: state_names(&states),
        bounds,
    };
    let mut queued: Vec<(String, String)> = Vec::new();
    for (child_bus, child_path) in children {
        queued.push((child_bus, child_path.as_str().to_string()));
    }
    return Ok((node, queued));
}

/// Read the whole tree of one application.
///
/// What:
///      `async fn read_application(connection: &Connection, application: &Application) -> Result<Vec<Node>>`.
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
async fn read_application(
    connection: &Connection,
    application: &Application,
) -> Result<Vec<Node>> {
    let mut nodes = Vec::new();
    let mut pending = vec![(
        application.bus_name.clone(),
        application.path.clone(),
        0_usize,
    )];
    while let Some((bus_name, path, depth)) = pending.pop() {
        if nodes.len() >= MAX_NODES || depth > MAX_DEPTH {
            warn!("accessibility tree read stopped at the node or depth limit");
            break;
        }
        let (node, children) = read_node(connection, &bus_name, &path, depth).await?;
        nodes.push(node);
        for (child_bus, child_path) in children.into_iter().rev() {
            pending.push((child_bus, child_path, depth + 1));
        }
    }
    return Ok(nodes);
}

/// Read every registered application on the calling thread inside a time budget.
///
/// What:
///      `pub fn read_all(service: &A11yService) -> Result<Vec<Node>>`.
///  One `zbus::block_on` call walks every application,
///  and the walk is dropped when `READ_BUDGET` has passed.
/// Why:
///       The calls wait for another process,
///  which may be slow or stopped,
///  and the compositor's event loop must not wait forever.
///  The walk is a future now,
///  so the budget ends it where the old reader thread was left running after the wait gave up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readAll(service: A11yService): Promise<Node[]> { ... } // with a 5 s timeout
/// ```
pub fn read_all(service: &A11yService) -> Result<Vec<Node>> {
    return read_all_within(service, READ_BUDGET);
}

/// Read every registered application on the calling thread inside `budget`.
///
/// What:
///      `pub fn read_all_within(service: &A11yService, budget: Duration) -> Result<Vec<Node>>`.
/// Why:
///       `read_all` fixes the budget at `READ_BUDGET`;
///  a test names a short one to see a stalled application end the read.
pub fn read_all_within(
    service: &A11yService,
    budget: Duration,
) -> Result<Vec<Node>> {
    let connection = service.connection().clone();
    let applications = service.applications();
    let walk = async move {
        let mut nodes = Vec::new();
        for application in &applications {
            nodes.append(&mut read_application(&connection, application).await?);
        }
        return Ok::<Vec<Node>, anyhow::Error>(nodes);
    };
    return block_on_within(budget, "the application's accessibility tree read", walk)
        .context("the application did not answer its accessibility tree");
}
