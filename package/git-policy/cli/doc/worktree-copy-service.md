# Worktree copy in the resident service

The worktree-copy feature copies a source worktree's ignored files into each linked worktree that a forwarded
`git worktree add` registers.
Until this change a per-repository settlement lock (a directory published by rename, an owner record, liveness checks,
a 20 ms poll while a live owner held it, and an inherited lease token for nested wrappers) serialized every creating
command and every recovery pass.
The owner's requirements ("Use as few locks as possible", "Thoughts on locks: I believe they're an anti-pattern", and the
reading of "Didn't I say locks are bad?" as no flock, no lock files, no owner records, no polling) are quoted in
`doc/planning/cli-git-resident-service.md`, which also decided a single-writer service per account.
This document records what the lock protected, why the first replacement was rejected, the design that stands, and the
test that answers each review finding.

## What the lock protected

The lock was held from before real Git started until the copy finished, for `git worktree add` and `git worktree move`
only, and briefly by every other forwarded command that found pending state.
It protected five things.

- Attribution of new worktrees: they are found by comparing the administrative directory names under
  `<common>/worktrees` before and after Git, and the lock made the difference exactly this command's worktrees.
- One copy per destination: overlapping creating commands both see both new names.
- A destination that is complete: a peer's worktree is registered before its checkout finishes, and installing ignored
  files into it can make the peer's checkout fail.
- Recovery against a live copy: a pass that removes a stage no journal names would delete a stage a live copy is building.
- Recovery against recovery: two passes would resume one journal into one destination.

The lock never protected a copy against `git worktree remove`; registration checks guard that race.

## The first replacement, and why it was rejected

One claim file per destination, held open by the copying process, with the process table as proof of life, passed its own
tests and was rejected by a review by another model vendor.
Its findings, ranked, were: (1) an expired claim bypassed the holder check, so two ordinary recoverers could both settle
it; (2) a process table is no evidence across hosts or when descriptors are unreadable; (3) the marker and the claim were
keyed by the administrative name, which Git reuses after a removal; (4) a destination still being filled could be another
creator's source; (5) the `initializing` text is absent for `--lock`, in other languages, and spoofable by `--reason`;
(6) a kill between the journal's removal and the marker forgot the copy; (7) macOS never adopted an abandoned claim;
(8) cleanup could remove a live publisher's temporary; (9) a worktree named like an adoption key lost its marker; (10) a
copy that failed or never got to its journal was never tried again; and a journal-listing race.
Every one of them except (3) to (6) and (10) exists only because several processes decide among themselves who may act.

## The design that stands

The copy moved into the resident commit service, which is the single writer the owner decided on.
Nothing that decides between processes is left.

- **The request.** After real Git returns, the wrapper writes one durable request per new worktree it may fill, under
  `<common>/cli-git-worktree-copy/v1/requests`, named by a time-ordered identity.
  It holds the registration's administrative name and identity, the destination, the source and the selection.
  A request is written whole (private temporary, synchronization, rename) and removed only when its copy has ended.
- **The registration's identity** is a random token written once into `worktrees/<name>/cli-git-registration`.
  Git deletes it with the directory, so a worktree that reuses the name starts without one.
  The marker `worktrees/<name>/cli-git-copied` holds the identity of the registration whose copy is over.
- **The message.** The wrapper sends one `copy` message (protocol version 5) naming its requests and waits on its
  connection for the answers: a line when a copy starts, notices, and how each request ended.
  A copy that outlasts a second is reported as still running.
  Waiting on a reply is not a lock; the wrapper holds nothing and records nothing about itself.
- **The service** runs the requests of a repository on one thread, one at a time, in the order they were written, except
  that a worktree another request copies from is filled first.
  Before the requests it recovers every journal on disk: no live copy can be writing one, so nothing has to be proven dead.
  For each request it checks the registration by identity, removes the stage a cut-off run left, stages, writes the journal
  naming the registration, checks the registration again, installs, writes the marker durably, and only then removes the
  journal, the stage and the request.
- **A killed client** leaves the copy to the service, and a failure nobody reads becomes a notice for the next commit.
  **A killed service** leaves requests and journals; the manager starts it on the next connection and the wrapper that was
  waiting sends the same message again.
  **A command that creates nothing** nudges the service, without waiting, when a request or a journal is pending.
- **Where no service can run** (no user service manager, no program beside the wrapper) the copy fails loudly: status 2,
  the message says the worktree was created, that the copy is saved, and what to run.
  The request stays; the next command that finds a service finishes it.
- **Attribution.** A command may fill a new worktree that Git does not lock and the one its command line asked
  `git worktree add` to create, even when `--lock` locked it.
  Git locks a worktree while it creates it, so a locked worktree it did not ask for is another command's, whatever the
  language or `--reason` of the lock text.
  The lock file's presence is the evidence; its text is not read for this.
- **Retries.** A copy that fails stays a request and runs again at the next message, up to three tries; then it moves to
  `given-up` with a notice.

## Findings and their tests

Each test is in `service_copy_tests.rs` unless another file is named, and each guard is a planted control in
`bin/native-planted-controls-landing.ts` that fails the named test when the guard is removed.

- **1, 2, 7, 8 (exclusion by evidence, platform, publisher's temporary).**
  The design has no claim, no process table and no platform branch, so they have no code to be wrong in.
  `simultaneous_requests_are_served_one_at_a_time` (one copy thread, however many clients),
  `two_clients_each_get_their_own_answer_and_each_copy_runs_once` (`service_copy_client_tests.rs`, the real service loop),
  `the_copy_modules_do_not_consult_the_process_table`, and `only_a_stale_temporary_is_removed`
  (`worktree_copy_request_tests.rs`: a killed writer's temporary goes by age, which no other process's progress depends on).
- **3 (a name reused by a later registration).**
  `a_name_reused_by_a_later_registration_is_copied_again`, `a_request_for_a_replaced_registration_creates_nothing`,
  `a_worktree_replaced_while_staging_is_not_filled` (the path and the name are the same, only the identity differs).
- **4 (a destination used as a source).**
  `a_source_is_filled_before_a_request_copies_from_it`, `a_destination_used_as_a_source_is_copied_from_after_it_is_filled`.
- **5 (readiness without the lock text).**
  `only_the_added_path_may_be_filled_while_locked`, `a_locked_worktree_this_command_added_is_copied_and_a_locked_peer_is_not`
  (`--reason initializing` on the one asked for, another language's text on the peer),
  `the_added_path_is_read_from_the_command_line` (`worktree_copy_forward_tests.rs`),
  `a_lock_file_is_evidence_and_only_the_english_word_is_the_heuristic` (`worktree_copy_registry_tests.rs`),
  and `overlapping_creations_each_copy_their_own_worktree_once` against two real wrappers (`binary_worktree_copy_tests.rs`).
- **6 (the finished copy is forgotten).**
  `a_service_killed_between_the_install_and_the_marker_does_not_forget_the_copy`,
  `a_service_killed_after_the_marker_only_cleans_up`, `finishing_a_journal_marks_before_it_removes`
  (`worktree_copy_recovery_tests.rs`), and `a_service_killed_at_each_point_of_a_copy_is_restarted_and_the_copy_finishes`,
  which kills the real service at each durable point with the wrapper waiting.
- **9 (worktree names).** `a_worktree_named_journal_is_copied_and_marked`.
- **10 (failed or unstarted copies).**
  `a_failed_copy_is_tried_again_and_then_given_up_on`, `a_copy_that_keeps_failing_is_given_up_on`,
  `a_service_killed_before_staging_copies_on_the_next_run`, `a_service_killed_while_staging_starts_over_without_the_half_stage`,
  and in `binary_worktree_copy_tests.rs` `a_request_left_by_a_killed_wrapper_is_run_by_the_next_command`,
  `a_copy_with_no_service_fails_loudly_and_is_finished_when_one_is_there`, `a_failed_copy_ends_with_status_two_and_its_message`.
- **The journal listing.** `a_journal_removed_between_the_stat_and_the_check_is_skipped` (`worktree_copy_journal_tests.rs`).
- **Two copies into one destination, a copy against a removal, a copy killed at each point** are the cases above:
  `a_request_for_a_marked_registration_runs_nothing`, `a_worktree_removed_while_staging_is_not_filled`,
  `a_worktree_removed_before_its_request_runs_is_never_staged`, and the five kill tests.

Honest note on the review's wording: the findings describe interleavings of the first design's internals, which no longer
exist, so the tests above check the property each finding broke, in the design that stands.

## Known limits

- One service serves one account on one host.
  Two accounts, or two hosts sharing a repository over a network file system, each have their own service, and nothing
  keeps their copies of one repository apart; the lock it replaces did not either (its owner records named a process of
  one host).
- A request is written after Git returned.
  A command that creates a worktree from a worktree whose own copy has not been requested yet copies from the unfilled one;
  the window is the time between the first command's Git returning and its request being written.
- A copy that fails and is not retried successfully in three runs is given up on; the request is kept in `given-up`.
- The English word `initializing` is still read in one place: a creating command that runs from inside a checkout it
  cannot copy from yet (a filter or hook of an enclosing `git worktree add`) only forwards.
  A localized Git, or a lock reason that spells the word, defeats that heuristic; nothing else depends on it.
  A `post-checkout` hook that creates a worktree is copied from its enclosing source, not from the real one.
- Two overlapping `git worktree add` runs from different linked sources can copy a destination from either source,
  because the administrative directory does not say which command created it.
- Release builds that still use the settlement lock must not run copies at the same time as this version; a leftover
  `settlement.lock` directory, `*.stage` and `*.claim` files are not read and may be deleted.
- `CopyFacts.global_prefix` and `CopyFacts.environment` are no longer read; removing them touches callers outside the copy.
