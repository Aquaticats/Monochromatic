//! Shared fixtures for Language integration tests:
//!  a child process per test,
//!  a scripted server,
//! and a polling probe around the worker handle.
//!
//! Helix derives every server root from the process working directory and keeps the first
//! value it reads.
//!  A test that starts a server therefore runs its body in a child process whose
//! working directory was set when it was spawned;
//!  the test process itself never changes directory.
//!
//! What:
//!  `#[path = "..."]` names the file a module is read from;
//!  this module's parts live under `tests/language/support/`.
//! Why:
//!  One test crate links the application library once and lets every test share the support
//!       code without unused-code warnings,
//!  and the parts are read one role at a time.

#[path = "support/child_process.rs"]
mod child_process;

#[path = "support/probe.rs"]
mod probe;

/// What the worker's answers and states say,
///  as a second `impl` of the probe.
#[path = "support/probe_answers.rs"]
mod probe_answers;

#[path = "support/reports.rs"]
mod reports;

#[path = "support/scripted_server.rs"]
mod scripted_server;

pub use child_process::{
    Layout,
    PRODUCT_TIMEOUT,
    SERVER,
    child_root,
    report_path,
    run_child,
    scratch,
    standard,
};
pub use probe::Probe;
pub use reports::{
    ReportWait,
    all_received,
    children,
    children_until_none,
    count_received,
    first_received,
    first_started,
    index_of,
    instance_of,
    received,
    report,
    server_text,
    server_text_until,
};
pub use scripted_server::{
    scripted,
    scripted_with_roots,
};
