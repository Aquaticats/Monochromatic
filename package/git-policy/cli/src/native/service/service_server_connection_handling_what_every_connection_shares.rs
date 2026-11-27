//! What:
//!  What every connection of one service run shares:
//!  the queues to the landing thread and to the work loop,
//!  the landing's board,
//!  the finished shadows drawn ahead of the commits,
//!  and the release and instance the service announces and serves.
//! Why:
//!  The acceptor hands one value to every connection
//!  (`service_server_accept_connections_as_tasks.rs`),
//!  so a connection reads what the run is instead of asking the service again,
//!  and the type stands in a file of its own so the connection's files name it without each other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ConnectionShared = { queue: Queue<Job>; work: Queue<Wake>; board: LandingBoard; drawn: DrawnShadows; release: string; instance?: string; ownGitDir?: string };
//! ```

use crate::service_busy::LandingBoard;
use crate::service_server_queue_types_and_send::Job;
use crate::service_work::Wake;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
/// What every connection of one service run shares:
/// the queues to the landing and the work loop,
/// the count of work in flight,
/// the landing's board,
/// and what the service announces and serves.
#[derive(Clone)]
pub(crate) struct ConnectionShared {
    /// Where requests for the landing go.
    pub queue: UnboundedSender<Job>,
    /// Where after-commit work is announced.
    pub work: UnboundedSender<Wake>,
    /// The count of work in flight that keeps the service from exiting idle.
    /// What the landing is inside,
    /// for the notes a waiting client is told.
    pub board: Arc<LandingBoard>,
    /// The finished shadows this service drew ahead of the commits that ask for one,
    /// which the answer to a commit's question takes one out of.
    pub drawn: Arc<crate::service_shadow_draw::DrawnShadows>,
    /// The release the service announces to every client.
    pub release: String,
    /// The worktree this instance serves,
    /// when it serves one.
    pub instance: Option<String>,
    /// The Git directory of the worktree this instance serves,
    /// as its note or journal named it when the service started,
    /// and nothing when that could not be told.
    /// A request that names this text is this instance's without asking the filesystem whose it is.
    pub own_git_dir: Option<PathBuf>,
}
