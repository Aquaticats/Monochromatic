//! What:
//!  The service's own stock of finished shadows drawn ahead of the commits that ask for one:
//!  the draw beside the landing loop,
//!  and the answers a connection takes out of it.
//! Why:
//!  The owner chose the service to rename a pool member into place ahead of a commit
//!  (`doc/planning/git-policy-cli-shadow-pool-claim-rename-options.md`,
//!  "The design the owner's decision made"),
//!  and the point of doing it ahead is that the commit never waits for the rename:
//!  the answer is one line written by the connection's own task,
//!  never by the landing thread,
//!  so a commit that waits for its tree does not queue behind another commit's landing.
//!  The draw itself runs on a thread of its own beside the landing,
//!  as the fill does,
//!  and a draw that stalls for the 5,606 ms the slowest measured rename took
//!  costs the commits of that window today's path and no wait of its own.
//!  The stock is memory only:
//!  a tree a restart forgets is never handed out twice,
//!  and it is left to the sweep by its age,
//!  so nothing here can hand one tree to two commits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stock = drawnShadows(); start({ context, request, stock }); const tree = stock.handOut();
//! ```

/// One thread per draw,
/// which the system may refuse.
use crate::scoped_start::{refused_thread_sentence, spawn_named};
/// What a commit request names.
use crate::service_protocol::CommitRequest;
/// The facts a creation is made from,
/// and where the pool's members are made from them.
use crate::shadow_create::ShadowTemplate;
use crate::shadow_drawn::{DRAWN_LIMIT, draw, sweep_drawn};
/// Where the pool holds what a draw moves out of it, and what a fill makes when it holds nothing.
use crate::shadow_pool::{members, pool_root_for};
/// The service's own log.
use crate::transaction_git::{GitContext, GitFailure};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Lock kept:
///  whether this service is already drawing one shadow,
///  which it sets and clears and nothing else touches:
/// one draw at a time is as much as a stock of one ever needs,
/// and a landing never waits for this flag,
/// because a wake that finds it set simply leaves the stock as it is.
static DRAWING: AtomicBool = AtomicBool::new(false);

/// What:
///  The drawn shadows this service holds ready,
///  one worktree's, since a service serves one.
/// Why:
///   A commit's question is answered from memory and not from the disk:
///  a listing of the drawn folder can wait on the volume,
///  and the answer must not.
///  The list is the whole of the service's own record of what it drew,
///  so a stock a restart forgets is drawn again and the trees it dropped are left to the sweep by their age.
///  The lock it is read under guards nothing on disk and dies with the process,
///  so it is never a lock a commit can be left waiting on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DrawnShadows { #ready = []; note(tree) {...} handOut() {...} }
/// ```
#[derive(Debug, Default)]
pub struct DrawnShadows {
    /// The trees nobody has asked for yet,
    /// in the order they were drawn.
    ready: Mutex<Vec<PathBuf>>,
}

impl DrawnShadows {
    /// What:
    ///  The stock of a service that has drawn nothing yet.
    /// Why:
    ///   A service's stock is its own,
    ///  and a restarted service adopts none of the trees an earlier one drew:
    ///  a tree a live commit holds is not one to hand out again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stock = new DrawnShadows();
    /// ```
    pub fn none_yet() -> Arc<Self> {
        return Arc::new(Self::default());
    }

    /// What:
    ///  Records a tree the draw made, for the next commit that asks.
    /// Why:
    ///   Only the draw and the answers touch the list,
    ///  and a poisoned lock is still a list of paths.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stock.note(tree)
    /// ```
    pub fn note_ready(&self, tree: PathBuf) {
        let mut ready = self.ready.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        ready.push(tree);
    }

    /// What:
    ///  Takes one drawn tree for a commit,
    ///  or nothing when the stock holds none.
    /// Why:
    ///  One tree leaves the list with each answer,
    ///  so two commits that ask at the same moment never get the same tree:
    ///  the second one is told the stock is empty and takes the pool's member or the creation,
    ///  which is the path it knows.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const tree = stock.handOut(); // the tree, or undefined
    /// ```
    pub fn hand_out(&self) -> Option<PathBuf> {
        let mut ready = self.ready.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        return ready.pop();
    }

    /// How many trees the stock holds now.
    ///  The draw asks it, and a control reads it.
    pub fn ready(&self) -> usize {
        let ready = self.ready.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        return ready.len();
    }
}

/// What:
///  Starts one draw for the worktree a request names,
/// unless the stock already holds what it would draw or a draw is already running.
/// Why:
///   Every answer the service gives is a commit the person waits for,
///  so the draw is started beside the loop and never inside it.
///  The two checks cost one lock and one atomic exchange,
///  and no Git start,
///  which is what a busy repository does on every landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// start({ context, request, stock }) // void
/// ```
pub fn start(context: &GitContext, request: &CommitRequest, stock: &Arc<DrawnShadows>) {
    if stock.ready() >= DRAWN_LIMIT {
        return;
    }
    if DRAWING.swap(true, Ordering::AcqRel) {
        return;
    }
    let own: GitContext = context.clone();
    let named: String = request.common_dir.display().to_string();
    let common_dir: PathBuf = request.common_dir.clone();
    let git_dir: PathBuf = request.git_dir.clone();
    let repository: PathBuf = request.repository.clone();
    let held: Arc<DrawnShadows> = Arc::clone(stock);
    let started = spawn_named("shadow-pool-draw", move || {
        // The flag is cleared however the draw ends, so one refused Git start does not stop every later draw.
        let outcome: Result<(), GitFailure> = draw_one(&own, &common_dir, &git_dir, &repository, held.as_ref());
        DRAWING.store(false, Ordering::Release);
        if let Err(error) = outcome {
            crate::service_warn!("the shadow drawn ahead for {named} was not drawn: {}", error.0);
        }
    });
    if let Err(error) = started {
        DRAWING.store(false, Ordering::Release);
        crate::service_warn!("{}", refused_thread_sentence("draw a shadow ahead", &error));
    }
}

/// What:
///  One draw,
/// from the facts of the repository the request names,
/// with the trees nothing renewed swept beside it.
/// Why:
///   The facts are read here rather than in the request,
///  because a request carries what a landing needs and not what a creation does;
///  a fact this derivation gets wrong makes a tree the commit refuses and hands back,
///  never a wrong commit.
///  The sweep runs here because a draft of a tree is the service's own work,
///  and an idle repository whose trees nobody collected is exactly what it is for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drawOne({ context, commonDir, gitDir, repository, stock })
/// ```
fn draw_one(
    context: &GitContext,
    common_dir: &std::path::Path,
    git_dir: &std::path::Path,
    repository: &std::path::Path,
    stock: &DrawnShadows,
) -> Result<(), GitFailure> {
    let template: ShadowTemplate = crate::service_shadow_pool_fill::template_of(context, common_dir, git_dir, repository)?;
    let _swept: Vec<PathBuf> = sweep_drawn(common_dir, SystemTime::now());
    // A pool with no member yet is the fill's work, and this thread does that work too: a draw that found the pool empty
    //  at the moment the landing started its fill drew nothing, which is every commit after the first on a repository
    //  whose commits each take the one member the pool holds.
    if members(pool_root_for(common_dir, git_dir).as_path()).is_empty() {
        let _made: Option<PathBuf> = crate::shadow_pool::fill(context, &template)?;
    }
    // A pool that stays empty (a reftable repository, a worktree that keeps its own info, a refused fill) is no reason to
    //  say anything: the next landing draws again, and the commit that asked took its own path.
    let Some(tree) = draw(context, &template)? else { return Ok(()) };
    crate::diagnostic_log::debug("shadowDrawn", format!("drew {} ahead of a commit", tree.display()).as_str());
    stock.note_ready(tree);
    return Ok(());
}

/// The draw and the stock are controlled in the test module.
#[cfg(test)]
#[path = "service_shadow_draw_tests.rs"]
mod tests;
