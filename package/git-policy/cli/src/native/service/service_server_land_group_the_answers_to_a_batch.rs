//! What:
//!  What one landed batch tells its requests:
//!  what each of them came to,
//!  the repeats of an identity answered from the first one's commit,
//!  what this batch landed remembered for the next request,
//!  and the note that names the repository to the next service.
//! Why:
//!   A landing answers many clients from one thread,
//!  and the answer of each is a different message with its own words in the log;
//!  the store that keeps a note is not asked again within a batch once one write of it has not answered,
//!  so a batch of many commits holds one unanswered note and pays one ceiling,
//!  never one for each of its commits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const [queued, outcome] of zip(fresh, outcomes)) logAnswer(times, queued, position, answers.answer(request, stream, outcome));
//! ```

use crate::service_captures::CaptureMemory;
use crate::service_claim_watch_note_within_a_ceiling::{
    NoteForTheNextService,
    note_for_the_next_service_within,
};
use crate::service_land::{
    LandContext,
    Outcome,
};
use crate::service_protocol::{
    CommitRequest,
    FailureCode,
    ServiceMessage,
};
use crate::service_reply::Reply;
use crate::service_server_land_group_answer_log::{
    BatchTimes,
    landing_figures_of,
    log_answer,
    path_of_change,
};
use crate::service_server_land_group_owed_pushes::hold_the_push;
use crate::service_server_queue_types_and_send::Queued;
use std::collections::{
    HashMap,
    HashSet,
};
use std::time::Duration;

/// What:
///  The answers of one landed batch,
///  and everything they change or read.
/// Why:
///   Every answer writes one of these:
///  what the batch landed,
///  the requests whose tree a client reads again,
///  what this service captured,
///  and whether the note to the next service still answers within this batch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TheAnswersToABatch { context; instance; noteCeiling; noteStore; theStoreAnswers; noThreadWasSaid; landed; handedBack; captures; times; }
/// ```
pub(crate) struct TheAnswersToABatch<'a> {
    /// The landing this batch belongs to,
    /// for the journal it names and the rules it warms.
    context: &'a LandContext,
    /// The instance this service serves,
    /// and nothing in a development run.
    instance: Option<&'a str>,
    /// The wait a note to the next service is granted.
    note_ceiling: Duration,
    /// The journal directory the notes for the next service are kept in.
    note_store: std::path::PathBuf,
    /// Whether the store still answers the notes of this batch.
    the_store_answers: bool,
    /// Whether the refusal of a thread has already been said for this batch.
    no_thread_was_said: bool,
    /// What this batch landed,
    /// remembered before any answer.
    landed: &'a mut HashMap<String, String>,
    /// The requests whose tree the client reads again.
    handed_back: &'a mut HashSet<String>,
    /// What the commits of this service captured,
    /// which a request prepared again is told about.
    captures: &'a mut CaptureMemory,
    /// What the answers of this batch are logged with.
    times: &'a BatchTimes,
}

impl<'a> TheAnswersToABatch<'a> {
    /// What:
    ///  The answers of one batch,
    /// with the store of the notes for the next service already named.
    /// Why:
    ///   The store is asked per landed commit,
    /// so it is named once here instead of once per answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// TheAnswersToABatch.of(context, instance, noteCeiling, times, landed, handedBack, captures)
    /// ```
    pub(crate) fn of(
        context: &'a LandContext,
        instance: Option<&'a str>,
        note_ceiling: Duration,
        times: &'a BatchTimes,
        landed: &'a mut HashMap<String, String>,
        handed_back: &'a mut HashSet<String>,
        captures: &'a mut CaptureMemory,
    ) -> Self {
        return Self {
            context,
            instance,
            note_ceiling,
            note_store: context
                .journal
                .join(crate::service_claim_watch::WATCH_DIRECTORY),
            the_store_answers: true,
            no_thread_was_said: false,
            landed,
            handed_back,
            captures,
            times,
        };
    }

    /// What:
    ///  Remembers what this batch landed,
    /// before any answer is sent.
    /// Why:
    ///   A commit sent back in the same batch is told about it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const [queued, outcome] of zip(fresh, outcomes)) if (outcome.landed) captures.recordLanded(queued, outcome.commit);
    /// ```
    pub(crate) fn remember_what_landed(
        &mut self,
        fresh: &[Queued],
        outcomes: &[Outcome],
    ) {
        for ((request, _, _), outcome) in fresh.iter().zip(outcomes) {
            if let Outcome::Landed {
                commit,
            } = outcome
            {
                self.captures.record_landed(
                    request.common_dir.as_path(),
                    request.repository.as_path(),
                    commit.as_str(),
                    request.captured,
                );
            }
        }
    }

    /// What:
    ///  Answers each repeat of a request's identity with the commit the first one landed.
    /// Why:
    ///   The same request sent twice into one batch (a client that reconnected) is landed once and answered twice.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const [queued, outcome] of zip(fresh, outcomes)) if (outcome.landed) for (const repeat of repeats) if (repeat.id === queued.id) repeat.stream.send({ already: true, commit: outcome.commit });
    /// ```
    pub(crate) fn answer_the_repeats(
        &self,
        fresh: &[Queued],
        repeats: &[Queued],
        outcomes: &[Outcome],
    ) {
        for ((request, _, _), outcome) in fresh.iter().zip(outcomes) {
            if let Outcome::Landed {
                commit,
            } = outcome
            {
                for (repeat, stream, arrival) in repeats {
                    if repeat.id != request.id {
                        continue;
                    }
                    stream.send(&ServiceMessage::Landed {
                        id: repeat.id.clone(),
                        commit: commit.clone(),
                        already: true,
                        landing: None,
                    });
                    log_answer(self.times, repeat, arrival, 1, "already");
                }
            }
        }
    }

    /// What:
    ///  Answers one request of the batch with what it came to,
    /// and says in one word what that was.
    /// Why:
    ///   The word is what the answer is logged with,
    ///  and each outcome writes a different message:
    ///  a landed commit holds its push and names its repository for the next service before it is answered,
    ///  and a commit written and kept says what its record says.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const word = match (outcome) { case 'landed': ...; case 'held': ...; case 'retryPrepare': ...; };
    /// ```
    pub(crate) fn answer(
        &mut self,
        request: &CommitRequest,
        stream: &Reply,
        outcome: Outcome,
    ) -> &'static str {
        let word: &str = match outcome {
            Outcome::Landed {
                commit,
            } => {
                // The compiled rules of the repository are warmed on the blocking pool, off this thread and off the answer's path.
                let paths: Vec<String> = request.changes.iter().map(path_of_change).collect();
                crate::service_rules_precompile::after_landing(
                    self.context.journal.as_path(),
                    request.repository.as_path(),
                    paths.as_slice(),
                );
                // The push is held before the answer, so a client killed between the two cannot lose it, and the post-commit
                //  gate is what promotes it.
                hold_the_push(request, commit.as_str());
                // The repository is written down for the next service in the same instant, so a wrapper killed between
                //  promoting that push and waking the service is still found when a service starts
                //  (`service_claim_watch.rs`, `note_for_the_next_service`). The wait for that write is bounded
                //  (`wait_ceilings.rs`, `LANDING_WATCH_NOTE_CEILING`), because the commit has landed and its push is held
                //  by now, and the work loop writes the same note when this answer's wake sends it to look.
                self.note_the_repository(request);
                self.landed.insert(request.id.clone(), commit.clone());
                stream.send(&ServiceMessage::Landed {
                    id: request.id.clone(),
                    commit,
                    already: false,
                    landing: Some(landing_figures_of(self.times)),
                });
                "landed"
            }
            Outcome::Held {
                commit,
                words,
            } => {
                // The commit is written and kept, and its branch has not moved: the client exits 0 and says what the
                //  record says, and no push is held for a commit no branch names yet.
                stream.send(&ServiceMessage::Held {
                    id: request.id.clone(),
                    commit,
                    words,
                });
                "held"
            }
            Outcome::RetryPrepare {
                base,
            } => {
                let known = self
                    .captures
                    .remembered(request.common_dir.as_path(), request.repository.as_path());
                stream.send(&ServiceMessage::RetryPrepare {
                    id: request.id.clone(),
                    base,
                    landed: known,
                });
                // The client prepares the commit again from the change this attempt captured, which it reads from
                //  this attempt's own tree: the tree is its client's again, and no landing reads it.
                self.handed_back.insert(request.id.clone());
                "retry-prepare"
            }
            Outcome::BranchMoved => {
                stream.send(&ServiceMessage::Failed {
                    id: request.id.clone(),
                    code: FailureCode::BranchMoved,
                });
                // Nothing landed on this tree, and the client reads the change it prepared from it to make the
                //  commit on the branch as it is now.
                self.handed_back.insert(request.id.clone());
                "branch-moved"
            }
            // The client went away before its commit landed, so the commit is not made and no answer is written for a
            //  reader that is not there: its tree stays where a client that reconnects reads it, and the sweep takes it
            //  by age otherwise.
            Outcome::ClientGone => {
                self.handed_back.insert(request.id.clone());
                "client-gone"
            }
        };
        return word;
    }

    /// What:
    ///  Writes down the repository a landed commit belongs to for the next service,
    /// within the wait this batch was given.
    /// Why:
    ///   A store that has not answered one write of this batch is not asked again within it,
    ///  and a refused thread is said once,
    ///  so a batch of many commits holds one unanswered note and pays one ceiling;
    ///  nothing ran for a refused thread,
    ///  so the next commit of the batch asks again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (!theStoreAnswers) return; switch (noteForTheNextServiceWithin(ceiling, store, instance, gitDir)) { ... }
    /// ```
    fn note_the_repository(
        &mut self,
        request: &CommitRequest,
    ) {
        if !self.the_store_answers {
            return;
        }
        match note_for_the_next_service_within(
            self.note_ceiling,
            self.note_store.as_path(),
            self.instance,
            request.git_dir.as_path(),
        ) {
            NoteForTheNextService::Answered => {}
            NoteForTheNextService::AnEarlierWriteHasNotEnded => {
                self.the_store_answers = false;
                crate::service_warn!(
                    "the note that names {} for the next service is not written, because a note an earlier commit handed over has not ended after {:?}; this commit is answered and stays landed, and the after-commit work writes that note when the wake of that answer sends it to look at the repository",
                    crate::service_timing::in_the_repository_at(request.repository.as_path()),
                    self.note_ceiling
                );
            }
            NoteForTheNextService::NoThreadToStart => {
                // Nothing ran, so this batch asks again for the next commit it lands, and only the first
                //  refusal is said: a system short of threads refuses many, and one line is what a person
                //  can act on.
                if !self.no_thread_was_said {
                    self.no_thread_was_said = true;
                    crate::service_warn!(
                        "the system gave no thread for the note that names {} for the next service, so it is not written yet; this commit is answered and stays landed, the next commit of this batch or the after-commit work writes that note, and the system's thread and process limits are what to look at",
                        crate::service_timing::in_the_repository_at(request.repository.as_path())
                    );
                }
            }
            NoteForTheNextService::NoAnswer => {
                self.the_store_answers = false;
                crate::service_warn!(
                    "the note that names {} for the next service gave no answer within {:?} and is left behind; this commit is answered and stays landed, and the after-commit work writes that note when the wake of this answer sends it to look at the repository",
                    crate::service_timing::in_the_repository_at(request.repository.as_path()),
                    self.note_ceiling
                );
            }
        }
    }
}
