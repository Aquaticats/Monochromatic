# Worktree copy in the resident service

The worktree-copy feature copies a source worktree's ignored files into each linked worktree that a forwarded
`git worktree add` registers.
Until this change a per-repository settlement lock (a directory published by rename,
an owner record,
liveness checks,
a 20 ms poll while a live owner held it,
and an inherited lease token for nested wrappers) serialized every creating
command and every recovery pass.
The owner's requirements ("Use as few locks as possible",
"Thoughts on locks:
I believe they're an anti-pattern",
and the
reading of "Didn't I say locks are bad?"
as no flock,
no lock files,
no owner records,
no polling) are quoted in
`doc/planning/cli-git-resident-service.md`,
which also decided a single-writer service (one per account in the first form,
and one per repository after the owner's
later change,
which this document follows where it matters).
This document records what the lock protected,
why the first replacement was rejected,
the design that stands,
and the
test that answers each review finding.

## What the lock protected

The lock was held from before real Git started until the copy finished,
for `git worktree add` and `git worktree move`
only,
and briefly by every other forwarded command that found pending state.
It protected five things.

- Attribution of new worktrees:
  they are found by comparing the administrative directory names under
  `<common>/worktrees` before and after Git,
  and the lock made the difference exactly this command's worktrees.
- One copy per destination:
  overlapping creating commands both see both new names.
- A destination that is complete:
  a peer's worktree is registered before its checkout finishes,
  and installing ignored
  files into it can make the peer's checkout fail.
- Recovery against a live copy:
  a pass that removes a stage no journal names would delete a stage a live copy is building.
- Recovery against recovery:
  two passes would resume one journal into one destination.

The lock never protected a copy against `git worktree remove`;
registration checks guard that race.

## The first replacement, and why it was rejected

One claim file per destination,
held open by the copying process,
with the process table as proof of life,
passed its own
tests and was rejected by a review by another model vendor.
Its findings,
ranked,
were:
(1) an expired claim bypassed the holder check,
so two ordinary recoverers could both settle
it;
(2) a process table is no evidence across hosts or when descriptors are unreadable;
(3) the marker and the claim were
keyed by the administrative name,
which Git reuses after a removal;
(4) a destination still being filled could be another
creator's source;
(5) the `initializing` text is absent for `--lock`,
in other languages,
and spoofable by `--reason`;
(6) a kill between the journal's removal and the marker forgot the copy;
(7) macOS never adopted an abandoned claim;
(8) cleanup could remove a live publisher's temporary;
(9) a worktree named like an adoption key lost its marker;
(10) a
copy that failed or never got to its journal was never tried again;
and a journal-listing race.
Every one of them except (3) to (6) and (10) exists only because several processes decide among themselves who may act.

## The design that stands

The copy moved into the resident commit service,
which is the single writer the owner decided on.
Nothing that decides between processes is left.

- **The request.**
  After real Git returns,
  the wrapper writes one durable request per new worktree it may fill,
  under
  `<common>/cli-git-worktree-copy/v1/requests`,
  named by a time-ordered identity.
  It holds the registration's administrative name and identity,
  the destination,
  the source and the selection.
  A request is written whole (private temporary,
  synchronization,
  rename) and removed only when its copy has ended.
- **The registration's identity** is a random token written once into `worktrees/<name>/cli-git-registration`.
  Git deletes it with the directory,
  so a worktree that reuses the name starts without one.
  The marker `worktrees/<name>/cli-git-copied` holds the identity of the registration whose copy is over.
- **The message.**
  The wrapper sends one `copy` message (protocol version 5) naming its requests and waits on its
  connection for the answers:
  a line when a copy starts,
  notices,
  and how each request ended.
  A copy that outlasts a second is reported as still running.
  Waiting on a reply is not a lock;
  the wrapper holds nothing and records nothing about itself.
- **The service** runs the requests of a repository on one thread,
  one at a time,
  in the order they were written,
  except
  that a worktree another request copies from is filled first.
  Before the requests it recovers every journal on disk:
  no live copy can be writing one,
  so nothing has to be proven dead.
  For each request it checks the registration by identity,
  removes the stage a cut-off run left,
  stages,
  writes the journal
  naming the registration,
  checks the registration again,
  installs,
  writes the marker durably,
  and only then removes the
  journal,
  the stage and the request.
- **A killed client** leaves the copy to the service,
  and a failure nobody reads becomes a notice for the next commit.
  **A killed service** leaves requests and journals;
  the manager starts it on the next connection and the wrapper that was
  waiting sends the same message again.
  **A command that creates nothing** nudges the service,
  without waiting,
  when a request or a journal is pending.
- **Where no service can run** (no user service manager,
  no program beside the wrapper) the copy fails loudly:
  status 2,
  the message says the worktree was created,
  that the copy is saved,
  and what to run.
  The request stays;
  the next command that finds a service finishes it.
- **Attribution.**
  A command may fill a new worktree that Git does not lock and the one its command line asked
  `git worktree add` to create,
  even when `--lock` locked it.
  Git locks a worktree while it creates it,
  so a locked worktree it did not ask for is another command's,
  whatever the
  language or `--reason` of the lock text.
  The lock file's presence is the evidence;
  its text is not read for this.
- **Retries.**
  A copy that fails stays a request and runs again at the next message,
  up to three tries;
  then it moves to
  `given-up` with a notice.

## Findings and their tests

Each test is in `service_copy_tests.rs` unless another file is named,
and each guard is a planted control in
`bin/native-planted-controls-landing.ts` that fails the named test when the guard is removed.

- **1,
  2,
  7,
  8 (exclusion by evidence,
  platform,
  publisher's temporary).**
  The design has no claim,
  no process table and no platform branch,
  so they have no code to be wrong in.
  `simultaneous_requests_are_served_one_at_a_time` (one copy thread,
  however many clients),
  `two_clients_each_get_their_own_answer_and_each_copy_runs_once` (`service_copy_client_tests.rs`,
  the real service loop),
  `the_copy_modules_do_not_consult_the_process_table`,
  and `a_complete_temporary_is_adopted_and_nothing_is_deleted_by_age`
  (`worktree_copy_request_tests.rs`:
  a temporary that a writer may still rename is published,
  never deleted;
  the second review replaced the first design's deletion by age).
- **3 (a name reused by a later registration).**
  `a_name_reused_by_a_later_registration_is_copied_again`,
  `a_request_for_a_replaced_registration_creates_nothing`,
  `a_worktree_replaced_while_staging_is_not_filled` (the path and the name are the same,
  only the identity differs).
- **4 (a destination used as a source).**
  `a_source_is_filled_before_a_request_copies_from_it`,
  `a_destination_used_as_a_source_is_copied_from_after_it_is_filled`.
- **5 (readiness without the lock text).**
  `only_the_added_path_may_be_filled_while_locked`,
  `a_locked_worktree_this_command_added_is_copied_and_a_locked_peer_is_not`
  (`--reason initializing` on the one asked for,
  another language's text on the peer),
  `the_added_path_is_read_from_the_command_line` (`worktree_copy_forward_tests.rs`),
  `a_lock_file_is_evidence_and_only_the_english_word_is_the_heuristic` (`worktree_copy_registry_tests.rs`),
  and `overlapping_creations_each_copy_their_own_worktree_once` against two real wrappers (`binary_worktree_copy_tests.rs`).
- **6 (the finished copy is forgotten).**
  `a_service_killed_between_the_install_and_the_marker_does_not_forget_the_copy`,
  `a_service_killed_after_the_marker_only_cleans_up`,
  `finishing_a_journal_marks_before_it_removes`
  (`worktree_copy_recovery_tests.rs`),
  and `a_service_killed_at_each_point_of_a_copy_is_restarted_and_the_copy_finishes`,
  which kills the real service at each durable point with the wrapper waiting.
- **9 (worktree names).**
  `a_worktree_named_journal_is_copied_and_marked`.
- **10 (failed or unstarted copies).**
  `a_failed_copy_is_tried_again_and_then_given_up_on`,
  `a_copy_that_keeps_failing_is_given_up_on`,
  `a_service_killed_before_staging_copies_on_the_next_run`,
  `a_service_killed_while_staging_starts_over_without_the_half_stage`,
  and in `binary_worktree_copy_tests.rs` `a_request_left_by_a_killed_wrapper_is_run_by_the_next_command`,
  `a_copy_with_no_service_fails_loudly_and_is_finished_when_one_is_there`,
  `a_failed_copy_ends_with_status_two_and_its_message`.
- **The journal listing.**
  `a_journal_removed_between_the_stat_and_the_check_is_skipped` (`worktree_copy_journal_tests.rs`).
- **Two copies into one destination,
  a copy against a removal,
  a copy killed at each point** are the cases above:
  `a_request_for_a_marked_registration_runs_nothing`,
  `a_worktree_removed_while_staging_is_not_filled`,
  `a_worktree_removed_before_its_request_runs_is_never_staged`,
  and the five kill tests.

Honest note on the review's wording:
the findings describe interleavings of the first design's internals,
which no longer
exist,
so the tests above check the property each finding broke,
in the design that stands.

## What the second review changed

A second review by the other model vendor confirmed that the service is sound as the serialization mechanism and closed
the claim,
process-table,
adoption and journal-listing findings.
It left these,
and each is answered below with the test that fails when its guard is removed
(the guard removals are in `bin/native-planted-controls-landing.ts`).

- **A wrapper killed after Git returns leaves a worktree nobody remembers.**
  The wrapper now writes an intent before it runs Git,
  under `<common>/cli-git-worktree-copy/v1/intents`.
  It holds the source,
  the selection,
  the administrative names before Git ran and the path the command line named.
  When Git returns the wrapper marks the intent as returned;
  once the requests for what Git created are written,
  the wrapper removes it.
  The service reads every intent at each drain,
  compares the names,
  and writes the requests of the worktrees it finds
  (a locked one only when it is the one the command line named,
  since a locked worktree is another command's while it is
  created);
  an intent that says Git returned is dropped once its requests are written,
  and any other is kept for an hour,
  because its wrapper may still be waiting for Git,
  and then dropped.
  The intent is written without a synchronization:
  it stands between the person's command and Git,
  on a disk where a
  synchronization can take seconds,
  and it is a hint for processes that were killed,
  which the page cache survives.
  Tests:
  `a_wrapper_killed_after_git_returns_leaves_the_worktree_to_the_service`,
  `the_intent_stands_while_git_runs_and_goes_when_the_requests_are_written`,
  `a_returned_intent_goes_once_its_requests_are_written`,
  `an_intent_whose_git_has_not_returned_is_kept_after_it_is_read`,
  `an_intent_is_kept_for_an_hour_and_then_removed`,
  `an_intent_fills_the_locked_worktree_it_named_and_leaves_a_locked_peer`,
  and with real processes `a_wrapper_killed_after_git_returned_leaves_the_copy_to_the_service`,
  which ends the wrapper
  right after Git returned and right after it wrote the requests.
- **Nothing enforces one copy writer.**
  The enforcement is the operating system's,
  and the tests prove its parts.
  The service is socket activated by a socket unit that does not accept connections into instances,
  so the manager runs
  at most one service process for a socket (one per repository once the service is keyed by the repository's common
  directory:
  the copy thread then serves one repository,
  and a copy never waits behind another repository's),
  and a process that retires is gone before the manager starts the next.
  A second service on the same address cannot bind it.
  A client that meets an older service asks it to retire and waits for its replacement for as long as a start takes,
  never sending it anything else,
  instead of pausing a fixed moment.
  A service started by hand on another address against a repository a managed service also serves is not supported:
  nothing could tell their copies apart without a lock,
  which the owner ruled out.
  Tests:
  `two_services_cannot_listen_on_one_address`,
  `the_shipped_units_run_one_service_instance`,
  `a_client_waits_through_an_older_service_that_is_slow_to_exit`,
  `a_wake_retires_an_older_service_and_wakes_the_current_one`.
- **A token could tag the replacement of the registration that was observed.**
  The wrapper records the identity of the administrative directory (device,
  inode and,
  where the file system keeps it,
  birth time) when it reads the worktree,
  and writes the token through a handle to that directory:
  a replacement under the
  same name has another identity and is refused (`Gone`),
  never tagged.
  The request carries the identity,
  and the service compares it again before it stages and after.
  The service also refuses a worktree that Git still has locked unless its creator asked for the lock:
  the request waits,
  with no try counted.
  Tests:
  `a_token_is_never_written_into_a_replacement_directory`,
  `a_replaced_registration_is_gone_and_is_not_tagged`,
  `a_request_for_a_directory_that_is_not_the_observed_one_is_refused`,
  `a_locked_worktree_waits_and_the_creators_own_lock_does_not`.
- **The token file could be empty for good.**
  The token is written to a temporary name in the directory,
  synchronized,
  and published with a link under its final name,
  so it appears whole or not at all,
  and a publisher that loses the race reads the winner's.
  An empty file that an earlier release left is replaced.
  Tests:
  `a_token_is_published_whole_and_kept`,
  `a_publisher_killed_before_it_linked_leaves_no_token`,
  `an_empty_token_from_an_older_release_is_replaced`,
  `racing_publishers_agree_on_one_token`.
- **Dependants ran after a prerequisite failed,
  and missed prerequisites published meanwhile.**
  The drain reads the requests again after each run.
  A request whose source worktree has a request of its own that did not end well in this drain waits,
  with no try counted;
  one whose source's copy was given up on,
  and which is still not marked,
  is given up on with that reason.
  Tests:
  `a_dependant_of_a_failed_prerequisite_waits_for_it`,
  `a_dependant_of_a_given_up_prerequisite_is_given_up`,
  `a_prerequisite_published_during_the_drain_runs_first`.
- **A terminal failure could become success after a restart.**
  Giving up writes the request,
  with the tries it took,
  into `given-up` and only then removes the pending one;
  a request
  that ended well writes a record into `done` before it goes.
  The service answers a client it did not tell from the disk:
  pending is waiting,
  given up is a failure,
  finished is its
  outcome,
  and a request nothing knows is a failure,
  never a success.
  A failure is written as a notice before the client is told;
  the client withdraws the notice after it has shown the
  failure,
  so a client killed first leaves the notice for the next commit.
  Tests:
  `a_request_given_up_before_a_kill_is_answered_as_a_failure_after_the_restart`,
  `a_finished_request_is_answered_from_its_record_after_a_restart`,
  `a_request_nobody_knows_is_answered_as_a_failure`,
  `a_pending_request_the_service_has_not_run_is_reported_as_waiting`,
  `a_failure_is_left_as_a_notice_until_the_client_has_shown_it`,
  `a_failure_for_a_client_that_is_gone_stays_a_notice`,
  `a_shown_failure_withdraws_its_notice`,
  `giving_up_leaves_a_durable_record_with_the_tries`,
  `a_finished_request_leaves_its_outcome`.
- **Completion waited for the whole drain,
  and the client was left without progress.**
  A client is told it is finished as soon as its own requests are answered,
  and returns then.
  A client that queues behind a running copy is told so at once,
  and every client is told it is still waiting every half
  minute,
  in the words that fit (copying a named worktree,
  or waiting for the service to get to it).
  A connection that breaks or falls silent after the request was sent says the result is not known,
  not that the files were
  not copied.
  Tests:
  `a_client_is_finished_when_its_own_requests_end_not_when_the_drain_does`,
  `a_client_returns_when_its_own_requests_are_answered`,
  `progress_is_repeated_while_a_copy_runs`,
  `a_queued_client_hears_that_it_waits`,
  `a_client_queued_behind_a_running_copy_is_told_at_once`,
  `every_copy_failure_is_plain`.
- **A request temporary was deleted by age.**
  A complete temporary is published,
  which is what its writer was about to do;
  one that does not parse is left alone.
  A writer that finds its temporary gone and its request standing with the same text has written it.
  Tests:
  `a_complete_temporary_is_adopted_and_nothing_is_deleted_by_age`,
  `a_writer_whose_temporary_was_adopted_succeeds`.
- **`worktree add` was misread.**
  The command line is read the way Git's option parser reads it:
  clustered short options,
  an abbreviated long option,
  a
  value attached with `=`.
  A form the reading does not know is unreadable,
  and a locked new worktree that cannot be matched to the command line
  is a failure that names it,
  never a silent skip.
  Tests:
  `the_added_path_is_read_from_the_command_line`,
  `the_added_path_survives_clustered_and_abbreviated_options`,
  `an_unknown_command_line_is_unreadable_and_never_guessed`,
  `a_locked_worktree_that_cannot_be_attributed_fails_loudly`.
- **A wake neither retired an older service nor set the service up,
  and a creating command that created nothing did not
  nudge.**
  The wake reaches the service as a request does and sets the service up once;
  the creating command nudges when a copy is
  pending.
  Tests:
  `a_wake_sets_the_service_up_once`,
  `a_creating_command_that_creates_nothing_nudges_a_pending_copy`.
- **A complete journal bypassing the marker.**
  The installation never writes the complete phase;
  only the cleanup after the marker does,
  so a complete journal that has
  no marker is the cleanup of a rolled-back or discarded copy,
  which recovery removes without marking.
  Tests:
  `the_journal_is_not_complete_when_the_installation_returns`,
  `a_complete_unmarked_journal_is_cleaned_up_and_never_marked`.

## Known limits

- One service process serves a repository on one host (an account's,
  until the service is keyed by the repository).
  Two accounts,
  or two hosts sharing a repository over a network file system,
  each have their own service,
  and nothing
  keeps their copies of one repository apart;
  the lock it replaces did not either (its owner records named a process of
  one host).
- The order of two copies where one worktree is the other's source is known to the service only once both have a request.
  The service writes the requests from intents before it orders,
  so the window in which a source has no request is closed
  for worktrees created through the wrapper;
  a worktree Git created without it has no intent and is not ordered.
- The names before Git ran are what tells a command's worktrees from the others',
  so a worktree that another command
  removes and adds again under a name that was already there while this command runs is not attributed to either;
  the identity of the directory is compared from the moment a worktree is observed,
  not before.
- The identity of an administrative directory is its device,
  inode and birth time.
  A file system that reports no birth time and reuses an inode number at once could present a replacement as the
  directory that was observed within the milliseconds between the wrapper's read and its write of the token.
- The service writes requests for what an intent finds,
  so a worktree created by a concurrent command that opted out
  (`--no-worktree-copy`) while another command's intent was open is copied too;
  opting out applies to the command,
  and the administrative directory does not say which command created a worktree.
- An intent,
  a finished-request record and the returned mark are written without a synchronization,
  and a request
  temporary that does not parse is never removed:
  a power failure can lose the first three (the copy is then found
  late or answered as unknown,
  never as success) and a writer killed in the middle leaves a few bytes behind.
- A copy that fails and is not retried successfully in three runs is given up on;
  the request is kept in `given-up`.
- The English word `initializing` is still read in one place:
  a creating command that runs from inside a checkout it
  cannot copy from yet (a filter or hook of an enclosing `git worktree add`) only forwards.
  A localized Git,
  or a lock reason that spells the word,
  defeats that heuristic;
  nothing else depends on it.
  A `post-checkout` hook that creates a worktree is copied from its enclosing source,
  not from the real one.
- Two overlapping `git worktree add` runs from different linked sources can copy a destination from either source,
  because the administrative directory does not say which command created it.
- Release builds that still use the settlement lock must not run copies at the same time as this version;
  a leftover
  `settlement.lock` directory,
  `*.stage` and `*.claim` files are not read and may be deleted.
- `CopyFacts.global_prefix` and `CopyFacts.environment` are no longer read;
  removing them touches callers outside the copy.
