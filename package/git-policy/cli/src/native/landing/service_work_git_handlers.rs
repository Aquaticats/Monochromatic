//! What:
//!  The handlers that run real Git for the service's work:
//!  the context every Git child of the service runs in,
//!  the review credential and the review runner a round hands them,
//!  the Git one push runs,
//!  and the review a landed commit is given.
//! Why:
//!   A round copies these settings and holds its copy for as long as it runs,
//!  so a credential never outlives the work it was woken for,
//!  and a control that reviews in the service's process gives its reviews a runner of their own
//!  (issue 6500).
//!  The type and the review are in one file because the review reads the settings this type keeps to itself;
//!  the other kinds' steps are the files named for them
//!  (`service_work_pushes.rs`,
//!  `service_work_worktree_upkeep.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class GitHandlers {
//!   constructor(context: GitContext) { /* the settings every kind of work reads */ }
//!   pushGit(item: WorkItem): PushGit { /* ... */ }
//! }
//! function review(handlers: GitHandlers, item: WorkItem): Verdict { /* ... */ }
//! ```

use super::service_work_handlers_and_verdicts::Verdict;
use crate::auto_push_single_flight::PushGit;
use crate::child_environment::environment_with_overlay;
/// The synchronous configuration read,
/// which only the control that builds the push's Git without the runtime asks.
#[cfg(test)]
use crate::config_snapshot::read_config_snapshot;
use crate::config_snapshot::read_config_snapshot_awaiting;
use crate::lfs_pre_push::uses_lfs;
use crate::service_review::{
    options_for,
    review_item_in,
    review_item_in_awaiting,
    review_variables,
};
use crate::service_review_key::ReviewKey;
use crate::standards_review_run::ReviewOptions;
use crate::transaction_git::GitContext;
use crate::work_item::WorkItem;
use std::path::{
    Path,
    PathBuf,
};

/// The handlers that run real Git.
pub struct GitHandlers {
    /// The context every Git child of the service runs in.
    pub context: GitContext,
    /// The review credential of each repository whose wake message carried one,
    /// for the round these handlers were made for.
    /// Held
    /// in memory only and never changed after the handlers are made:
    /// a round makes its own copy (`for_round`) and drops it when
    /// the round ends,
    /// so the workers of a round only read it and nothing waits for it.
    review_keys: std::collections::HashMap<PathBuf, ReviewKey>,
    /// Where tests point the review's gateway call,
    /// so no test reaches the real endpoint.
    #[cfg(test)]
    review_endpoint: Option<String>,
    /// The runner a test's reviews take a job of,
    /// when the control gives the handlers one of its own;
    /// `None` is the runner the whole service process shares.
    /// Why:
    ///   A runner admits one review and answers every other caller `Contained::Busy`,
    /// and a crate's tests run as threads of one process,
    /// so a control that reviewed in the shared one failed while another control's review held it (issue 6500).
    #[cfg(test)]
    review_runner: Option<&'static crate::standards_review_run::Containment>,
    /// Variables a test adds to the environment of the push,
    /// such as a short stall window.
    #[cfg(test)]
    push_environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
}

/// Methods over the real handlers.
impl GitHandlers {
    /// Handlers that run real Git in `context`,
    /// with no review credential yet.
    pub fn new(context: GitContext) -> GitHandlers {
        return GitHandlers {
            context,
            review_keys: std::collections::HashMap::new(),
            #[cfg(test)]
            review_endpoint: None,
            #[cfg(test)]
            review_runner: None,
            #[cfg(test)]
            push_environment: Vec::new(),
        };
    }

    /// The same handlers with variables added to the environment of every push;
    /// tests only.
    #[cfg(test)]
    pub fn with_push_environment(
        mut self,
        variables: &[(&str, &str)],
    ) -> GitHandlers {
        self.push_environment = variables
            .iter()
            .map(|(name, value)| {
                return (
                    std::ffi::OsString::from(name),
                    std::ffi::OsString::from(value),
                );
            })
            .collect();
        return self;
    }

    /// The same handlers with the review's gateway call sent to `endpoint`;
    /// tests only.
    #[cfg(test)]
    pub fn with_review_endpoint(
        mut self,
        endpoint: &str,
    ) -> GitHandlers {
        self.review_endpoint = Some(endpoint.to_string());
        return self;
    }

    /// What:
    ///  The same handlers whose reviews take a job of `runner`,
    /// a `Containment` of the control's own;
    /// tests only.
    /// Why:
    ///   A runner admits one job and answers every other caller `Contained::Busy`,
    /// and a crate's tests run as threads of one process,
    /// so two controls that reviewed in the service's runner refused one another (issue 6500).
    ///  A control declares `static OWN: Containment` and hands it here,
    /// as one that reviews an item itself calls `review_item_in(&OWN, ..)`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// handlers.withReviewRunner(own)
    /// ```
    #[cfg(test)]
    pub fn with_review_runner(
        mut self,
        runner: &'static crate::standards_review_run::Containment,
    ) -> GitHandlers {
        // What: `Some(runner)` is this instance's answer from now on: a runner of the control's, never the shared one.
        // Why: the control's reviews then see only each other's jobs, and never another control's.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.reviewRunner = runner;
        // ```
        self.review_runner = Some(runner);
        return self;
    }

    /// A copy of these handlers that holds `keys` as the review credentials of one round;
    /// the copy is dropped with the round.
    /// The test-only settings (the review endpoint,
    /// the review runner,
    /// the push environment) are carried into the copy,
    /// so a round runs the way the
    /// handlers it was made from are set to run.
    pub fn for_round(
        &self,
        keys: std::collections::HashMap<PathBuf, ReviewKey>,
    ) -> GitHandlers {
        return GitHandlers {
            context: self.context.clone(),
            review_keys: keys,
            #[cfg(test)]
            review_endpoint: self.review_endpoint.clone(),
            #[cfg(test)]
            review_runner: self.review_runner,
            #[cfg(test)]
            push_environment: self.push_environment.clone(),
        };
    }

    /// What:
    ///  The runner this instance's reviews take a job of:
    /// the one a control gave it (`with_review_runner`),
    /// or the one the whole service process shares.
    /// Why:
    ///   The service runs one review at a time for the whole process
    /// (`SPEC.md`,
    /// "The whole review of a commit is waited for `REVIEW_CEILING` ... no other review starts meanwhile"),
    /// and a control must not,
    /// because a run holds many of them at once in one process (issue 6500).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reviewRunnerOf(): Containment { return this.reviewRunner ?? RUNNER; }
    /// ```
    fn review_runner_of(&self) -> &'static crate::standards_review_run::Containment {
        #[cfg(test)]
        {
            // What: `Some(own)` is the runner a control handed these handlers, and it wins over the shared one.
            // Why: a control's reviews must not meet another control's job in one runner.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (this.reviewRunner) return this.reviewRunner;
            // ```
            if let Some(own) = self.review_runner {
                return own;
            }
        }
        // What: `&crate::standards_review_run::RUNNER` is the one runner of the service's process, borrowed for the job's life.
        // Why: a release has one service and one review at a time, which is what `SPEC.md` states.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return RUNNER;
        // ```
        return &crate::standards_review_run::RUNNER;
    }

    /// The credential held for this repository,
    /// if the round brought one.
    pub(super) fn review_key_of(
        &self,
        git_dir: &Path,
    ) -> Option<ReviewKey> {
        return self.review_keys.get(git_dir).cloned();
    }

    /// The push's Git,
    /// asked without the runtime;
    /// a control builds it this way.
    #[cfg(test)]
    pub(super) fn push_git(
        &self,
        item: &WorkItem,
    ) -> PushGit {
        let cwd: PathBuf = item.repository.clone();
        let programs: Option<crate::service_push_programs_held_in_memory::PushPrograms> =
            crate::service_push_programs_held_in_memory::held(
                item.git_dir.as_path(),
                item.id().as_str(),
            );
        let (inherited, overlay): (
            Vec<(std::ffi::OsString, std::ffi::OsString)>,
            Vec<(std::ffi::OsString, std::ffi::OsString)>,
        ) = self.push_environment_of(programs.as_ref());
        return PushGit {
            real_git: self.context.real_git.clone(),
            lfs: read_config_snapshot(&self.context, cwd.as_path())
                .as_ref()
                .is_none_or(uses_lfs),
            cwd,
            environment: environment_with_overlay(inherited.as_slice(), overlay.as_slice()),
        };
    }

    /// The same Git,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The configuration read starts Git,
    /// and the synchronous read enters the runtime and is refused from there;
    /// the environment and the directory are the synchronous face's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async pushGitOnTheRuntime(item): Promise<PushGit>
    /// ```
    pub(super) async fn push_git_awaiting(
        &self,
        item: &WorkItem,
    ) -> PushGit {
        let cwd: PathBuf = item.repository.clone();
        let programs: Option<crate::service_push_programs_held_in_memory::PushPrograms> =
            crate::service_push_programs_held_in_memory::held(
                item.git_dir.as_path(),
                item.id().as_str(),
            );
        let (inherited, overlay): (
            Vec<(std::ffi::OsString, std::ffi::OsString)>,
            Vec<(std::ffi::OsString, std::ffi::OsString)>,
        ) = self.push_environment_of(programs.as_ref());
        return PushGit {
            real_git: self.context.real_git.clone(),
            lfs: read_config_snapshot_awaiting(&self.context, cwd.as_path())
                .await
                .as_ref()
                .is_none_or(uses_lfs),
            cwd,
            environment: environment_with_overlay(inherited.as_slice(), overlay.as_slice()),
        };
    }

    /// What every Git child of a push receives,
    /// and the overlay the service adds to it.
    /// Why:
    ///   Both faces of the push Git build their environment here,
    /// so the two cannot hand Git different variables.
    ///  The list of the command that asked for the push is laid over the service's own,
    /// because the service takes the environment the user manager started it
    ///  with,
    /// which the person committing can neither see nor change
    /// (`service_work_push_environment`).
    /// The values are this service's memory,
    /// never a file:
    /// a service that restarted holds none,
    /// and the push either says whose list it searched or stops
    /// (`push_without_its_transport`).
    ///  Git writes a `~pid.lock` file beside every lock it takes when `core.lockfilePid` is on,
    /// which names the owner of a lock
    ///  that a stopped push leaves behind;
    /// the service's own environment does not turn it on.
    /// The overlay also says never to ask for
    ///  a login (`never_prompt_overlay`):
    /// the host's askpass program is in this environment,
    /// and a dialog nobody answers is a push that waits.
    /// It also marks that the push runs in the service,
    /// so a refusal that cannot find `git-lfs` names the list this push searched rather than the
    ///  shell's (`lfs_program_search::ProgramSearch`);
    /// the mark is present and carries the value `1`,
    /// which nothing reads.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const [inherited, overlay] = pushEnvironmentOf(programsOf(item));
    /// ```
    fn push_environment_of(
        &self,
        programs: Option<&crate::service_push_programs_held_in_memory::PushPrograms>,
    ) -> (
        Vec<(std::ffi::OsString, std::ffi::OsString)>,
        Vec<(std::ffi::OsString, std::ffi::OsString)>,
    ) {
        #[cfg(test)]
        let extra: &[(std::ffi::OsString, std::ffi::OsString)] = self.push_environment.as_slice();
        #[cfg(not(test))]
        let extra: &[(std::ffi::OsString, std::ffi::OsString)] = &[];
        return crate::service_work_push_environment::push_environment_of(extra, programs);
    }
}

pub(super) fn review(
    handlers: &GitHandlers,
    item: &WorkItem,
) -> Verdict {
    let key: Option<ReviewKey> = handlers.review_key_of(item.git_dir.as_path());
    #[cfg_attr(
        not(test),
        expect(unused_mut, reason = "only a test replaces the endpoint")
    )]
    let mut options: ReviewOptions = options_for(item, &review_variables(key.as_ref()));
    #[cfg(test)]
    if let Some(endpoint) = handlers.review_endpoint.as_ref() {
        options.endpoint = endpoint.clone();
    }
    return review_item_in(
        handlers.review_runner_of(),
        &handlers.context,
        item,
        options,
        crate::wait_ceilings::REVIEW_CEILING,
    );
}

/// What:
///  The same review,
/// its wait for the contained job made on the runtime.
/// Why:
///   The round of the service's work is tasks of the runtime
/// (`service_round.rs`),
/// and the synchronous wait for a review's job blocks the thread that asks,
///  which a worker may not do:
/// the job starts Git.
/// The key,
/// the options and the endpoint are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reviewAwaiting(item) { return reviewItemAwaiting(context, item, optionsOf(item)); }
/// ```
pub(super) fn review_awaiting<'a>(
    handlers: &'a GitHandlers,
    item: &'a WorkItem,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
    return Box::pin(async move {
        let key: Option<ReviewKey> = handlers.review_key_of(item.git_dir.as_path());
        #[cfg_attr(
            not(test),
            expect(unused_mut, reason = "only a test replaces the endpoint")
        )]
        let mut options: ReviewOptions = options_for(item, &review_variables(key.as_ref()));
        #[cfg(test)]
        if let Some(endpoint) = handlers.review_endpoint.as_ref() {
            options.endpoint = endpoint.clone();
        }
        return review_item_in_awaiting(
            handlers.review_runner_of(),
            &handlers.context,
            item,
            options,
            crate::wait_ceilings::REVIEW_CEILING,
        )
        .await;
    });
}
