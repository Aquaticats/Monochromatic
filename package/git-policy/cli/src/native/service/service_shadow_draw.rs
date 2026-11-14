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
/// The renewal that keeps a stocked tree from aging out while it waits,
/// and the pace a holding command renews at.
use crate::service_scratch_renewal::ScratchRenewal;
use crate::scratch_location::SCRATCH_REFRESH_EVERY;
/// The service's own log.
use crate::transaction_git::{GitContext, GitFailure};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

/// What:
///  How long the service keeps the claim on a drawn tree,
///  or nothing for as long as the service lives.
/// Why:
///   A drawn tree stands while its claim is held, and whether it stands with no end is the owner's question and not
///  this file's (`doc/handover/questions-for-the-owner.md`;
///  the brief of the branch that added the claim).
///  The answer is this one constant, for the tree waiting in the stock and for the tree the service has handed out:
///  `Some(span)` lets go at the first landing after the span, so the tree ages out and a sweep collects it,
///  and it must be longer than `DRAWN_ANSWER_DEADLINE` (`service_client_shadow_hand_out.rs`),
///  or a tree would be let go while the answer that names it is still travelling.
///  `None` keeps each of them until the landing that names the tree is over, and the stocked one until the service ends,
///  which is what this branch does today.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_DRAWN_TREE_IS_KEPT_FOR: Option<Duration> = undefined; // the owner's question, unanswered
/// ```
pub const A_DRAWN_TREE_IS_KEPT_FOR: Option<Duration> = None;

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
///  so a stock a restart forgets is drawn again and the trees it dropped are left to the sweep by their age,
///  which the draw marked with the moment it drew them.
///  A tree it still holds is marked by the renewal that travels with the entry,
///  so a stocked tree never reaches the age a sweep takes one at,
///  and the claim that renewal keeps says the same to a sweep whose clock stepped forward
///  (`tree_held_by_a_live_process.rs`),
///  and the answer itself reads nothing but memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DrawnShadows { #ready = []; note(tree) {...} handOut() {...} }
/// ```
#[derive(Debug, Default)]
pub struct DrawnShadows {
    /// Lock kept:
    ///  the trees nobody has asked for yet,
    ///  which the draw's own thread records and every connection's answer takes,
    ///  so two threads reach the list and a list of trees is not one word an atomic could hold.
    ///  Under it the list is changed and nothing else:
    ///  one `push` on the draw's thread and one `pop` where a commit's question is answered,
    ///  with no Git process, no file and no other lock,
    ///  and it guards nothing on disk and dies with the process,
    ///  so it is never a lock a commit can be left waiting on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// #ready: Waiting[] = []
    /// ```
    ready: Mutex<Vec<Waiting>>,
    /// Lock kept:
    ///  the trees the service has handed out and the claim it kept on each of them,
    ///  which the answer that hands one over records and the landing that names it lets go,
    ///  so two threads reach the list and a tree is never unheld while it changes hands.
    ///  Under it the list is changed and nothing else:
    ///  one `push` on the answer's thread and one `retain` where a landing or a draw reads it,
    ///  with no Git process, no file and no other lock.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// #handed: HandedOut[] = []
    /// ```
    handed: Mutex<Vec<HandedOut>>,
}

/// What:
///  One tree the service handed out,
///  with the claim it kept on the tree across the hand-out.
/// Why:
///  The answer travels to the commit over a socket, and the commit takes its own claim when it has read it,
///  so a sweep that ran in that moment would find a tree nobody held and, with the wall clock stepped forward,
///  take it for hours old.
///  The service therefore keeps its claim until the landing that names the tree is over,
///  and the commit's claim overlaps it, so the tree has a holder at every instant of the change of hands.
///  How long it is kept is `A_DRAWN_TREE_IS_KEPT_FOR`'s answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HandedOut = { path: string; since: Instant; claim: HeldTree };
/// ```
#[derive(Debug)]
struct HandedOut {
    /// Where the tree is.
    path: PathBuf,
    /// When the service handed it out,
    /// which `A_DRAWN_TREE_IS_KEPT_FOR` counts from.
    since: Instant,
    /// The claim the service kept,
    /// which stops when this value goes.
    _claim: crate::tree_held_by_a_live_process::HeldTree,
}

/// What:
///  One tree the service drew and nobody has asked for yet:
///  where it is,
///  and the renewal that keeps a sweep from taking it while it waits.
/// Why:
///   A drawn tree is standing state until a commit takes it,
///  as a pool member is until a commit claims one:
///  the service drew it ahead precisely so that it stands there while it waits,
///  so its age must not be what decides it goes.
///  The renewal beats on a thread of its own,
///  the first beat at once and then at the pace a holding command uses,
///  which is half the age a sweep takes a tree at,
///  so no sweep ever finds a stocked tree old enough to take (`shadow_drawn.rs`, `nothing_holds`),
///  and it carries the claim on the tree, which no wall-clock step can take away
///  (`tree_held_by_a_live_process.rs`).
///  Both stop when the entry leaves the stock, and with the service:
///  a service that is gone renews and claims nothing, and the sweep collects the tree a bound later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Waiting = { path: string; renewal: ScratchRenewal };
/// ```
#[derive(Debug)]
struct Waiting {
    /// Where the tree is.
    path: PathBuf,
    /// When the draw stocked it,
    /// which `A_DRAWN_TREE_IS_KEPT_FOR` counts from.
    since: Instant,
    /// The renewal that keeps it from aging out while it waits,
    /// and the claim on it,
    /// both of which stop when this value goes.
    _renewal: ScratchRenewal,
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
    ///  Records a tree the draw made, for the next commit that asks,
    ///  with the renewal that keeps it from aging out while it waits.
    /// Why:
    ///   Only the draw and the answers touch the list,
    ///  and a poisoned lock is still a list of paths.
    ///  The renewal beats at the pace a holding command uses (`SCRATCH_REFRESH_EVERY`),
    ///  the first beat at once,
    ///  so a stocked tree is never half-way to the age a sweep takes one at.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stock.note(tree)
    /// ```
    pub fn note_ready(&self, tree: PathBuf) {
        // The renewal is started before the entry is listed, so no answer can name a tree that nothing marks.
        let renewal: ScratchRenewal = ScratchRenewal::start(tree.as_path(), SCRATCH_REFRESH_EVERY);
        let mut ready = self.ready.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        ready.push(Waiting { path: tree, since: Instant::now(), _renewal: renewal });
    }

    /// What:
    ///  Takes one drawn tree for a commit,
    ///  or nothing when the stock holds none.
    /// Why:
    ///  One tree leaves the list with each answer,
    ///  so two commits that ask at the same moment never get the same tree:
    ///  the second one is told the stock is empty and takes the pool's member or the creation,
    ///  which is the path it knows.
    ///  The answer reads memory and nothing else:
    ///  a query of the volume here would wait on the volume the drawn tree lives on,
    ///  which is the wait the draw-ahead exists to take off a commit
    ///  (`service_server_connection_handling.rs`, `answer_the_shadow_question`).
    ///  The renewal that kept the tree fresh while it waited stops with this answer,
    ///  and the commit's own renewal takes over from the moment it holds the tree
    ///  (`shadow_drawn.rs`, `HandedShadow`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const tree = stock.handOut(); // the tree, or undefined
    /// ```
    pub fn hand_out(&self) -> Option<PathBuf> {
        let taken: Option<Waiting> = {
            let mut ready = self.ready.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            ready.pop()
        };
        let waiting: Waiting = taken?;
        // The claim moves from the renewal to the handed entry in one step, so no sweep finds the tree unheld
        //  while the answer that names it travels to the commit that will claim it.
        if let Some(held) = waiting._renewal.into_claim() {
            let mut handed = self.handed.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            handed.push(HandedOut { path: waiting.path.clone(), since: Instant::now(), _claim: held });
        }
        return Some(waiting.path);
    }

    /// What:
    ///  Lets go of the claim the service kept on a tree a landing named,
    ///  and answers whether it held one.
    /// Why:
    ///   The commit's own claim is held by then, because the request that names the tree is sent only after the build in
    ///  it, so the tree stays held by the commit while this claim goes.
    ///  The landing that named the tree reads its own scratch tree and not this one,
    ///  and removes it once that landing is over (`service_server.rs`, `remove_the_trees`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stock.letGoOf(tree) // whether the service held a claim on it
    /// ```
    pub fn let_go_of(&self, tree: &Path) -> bool {
        let mut handed = self.handed.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let counted: usize = handed.len();
        handed.retain(|out| { return out.path != tree; });
        return handed.len() != counted;
    }

    /// What:
    ///  Lets go of the claims the service has kept longer than `A_DRAWN_TREE_IS_KEPT_FOR`,
    ///  and answers how many it let go.
    /// Why:
    ///   The constant is the owner's answer to whether a drawn tree stands with no end while its claim is held;
    ///  with `None` nothing is let go here, and with a span a tree the service kept too long stops being kept at the first
    ///  landing that runs after it, so its age takes it and a sweep collects it.
    ///  A tree the commit took is let go by `let_go_of` when its request arrives, and this is for a tree whose commit
    ///  never sent one: a commit that gave the tree up, or one that was killed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stock.letGoOfWhatItKeptTooLong()
    /// ```
    pub fn let_go_of_what_it_kept_too_long(&self) -> usize {
        let Some(kept_for) = A_DRAWN_TREE_IS_KEPT_FOR else { return 0 };
        let mut let_go: usize = 0;
        {
            let mut handed = self.handed.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let counted: usize = handed.len();
            handed.retain(|out| { return out.since.elapsed() < kept_for; });
            let_go += counted - handed.len();
        }
        {
            let mut ready = self.ready.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let counted: usize = ready.len();
            ready.retain(|waiting| { return waiting.since.elapsed() < kept_for; });
            let_go += counted - ready.len();
        }
        return let_go;
    }

    /// What:
    ///  Drops the stocked trees that are not there any more,
    ///  and answers how many it dropped.
    /// Why:
    ///   The stock is memory only, so nothing tells it that a tree went:
    ///  a person,
    ///  another program,
    ///  or a removed worktree can take one out of the drawn folder.
    ///  A tree that is not there is no tree a commit can be handed,
    ///  and a stock that counted one would draw nothing in its place (`start`),
    ///  so the question is asked at every landing,
    ///  on the landing's own thread and never on an answer's,
    ///  and never while the stock's lock is held:
    ///  an answer that waited for this lock would wait on the volume through it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stock.dropWhatIsGone() // how many entries went
    /// ```
    pub fn drop_what_is_gone(&self) -> usize {
        // The paths are read under the lock and asked about without it, so no volume call is made inside the lock.
        let paths: Vec<PathBuf> = {
            let ready = self.ready.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let handed = self.handed.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut paths: Vec<PathBuf> = ready.iter().map(|waiting| { return waiting.path.clone(); }).collect();
            paths.extend(handed.iter().map(|out| { return out.path.clone(); }));
            paths
        };
        let mut gone: Vec<PathBuf> = Vec::new();
        for path in paths {
            if std::fs::symlink_metadata(path.as_path()).is_err() {
                gone.push(path);
            }
        }
        if gone.is_empty() {
            return 0;
        }
        let mut dropped: usize = 0;
        {
            let mut ready = self.ready.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let counted: usize = ready.len();
            ready.retain(|waiting| { return !gone.contains(&waiting.path); });
            dropped += counted - ready.len();
        }
        {
            let mut handed = self.handed.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let counted: usize = handed.len();
            handed.retain(|out| { return !gone.contains(&out.path); });
            dropped += counted - handed.len();
        }
        return dropped;
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
    // What: A tree the stock counted that is not there is no tree to hand a commit, and no reason to skip the draw.
    // Why:  The stock is memory only and the drawn folder is on the volume, where any command of the repository may collect
    //       a tree nothing marked; the landing's own thread asks here, so an answer that comes later reads memory alone.
    let _too_long: usize = stock.let_go_of_what_it_kept_too_long();
    let _gone: usize = stock.drop_what_is_gone();
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
