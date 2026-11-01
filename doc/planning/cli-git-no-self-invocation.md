# The wrapper never starts itself

Status: phase 1 built in this branch (LFS upload, merge push, bans); phase 2 (the hook dispatcher) is another change.

## The rule

The owner's instruction, quoted: "We should never invoke ourselves recursively. That's bad design. Audit, remove, and
ban that design aggressively." and "There are clean ways to have extensive explanatory error messages even with the
ban." The owner then added: "we don't have to use any shell!"

The rule as built:

- The wrapper never starts its own executable, whether as a child, as a Git hook command or `-c` argument, as a shim it
  writes, or through dispatch on the name it was started as (`argv[0]`).
- The wrapper never writes shell text: no `sh -c`, no script file, no shell function, no `printf` or `$?` in a hook.
- An explanation of a failure is printed by the parent process that owns the child, from facts the parent can read
  itself. Nothing that Git or a hook starts is asked to explain anything.

The rule is not in `AGENTS.md`. It lives here, in `package/git-policy/cli/README.md`,
in `package/git-policy/cli/SPEC.md` and in the mechanical bans (section "The bans").

## Audit inventory

Every way found by which the wrapper binary is started because of something the wrapper did. File and line numbers
are from the commit before this change (`5561c8b7cb`) unless a path says "now".

### Removed in phase 1

- The LFS pre-push hook. `lfs_pre_push.rs` `hook_arguments` wrote `hook.cli-git-lfs-pre-push.command='<wrapper>'
  cli-git lfs-pre-push`; Git ran it through `sh -c`; `entry.rs:289` recognised the words, and `run_hook_entry`
  started `git-lfs pre-push`. The call sites were `entry.rs:702`, `push_stacking.rs:194`,
  `auto_push_single_flight.rs:226` and `commit_command.rs:228`. Now the command is the two plain words
  `git-lfs pre-push` (`lfs_pre_push.rs` `HOOK_COMMAND`); Git starts `git-lfs` itself.
- The merge push of an integrated race. `push_stacking.rs:142` was `Command::new(request.own_executable)` running
  `push` with `CLI_GIT_STACKED_PUSH=1` in a cleared environment, so that the merge passed the push gates. This was a
  real recursive start that the first inventory did not list, because it looked like a use of `current_exe`. Now
  `stacked_merge_push.rs` runs `plan_invocation` for the merge push inside the running process, with the marker set only
  in the environment handed to the planner, and runs the planned action as a child of the same process.
- `push_stacking.rs:211` `current_exe`. It only produced the path for the start above; gone with it.
- The exec-forward site (`entry.rs:701-719`). A push in a repository with LFS keys was forwarded by process
  replacement, leaving no parent to explain a failure. Now such a push runs as a child like a stackable one
  (`Action::ForwardPush` with `lfs` set and `stack` unset when it is not stackable). Pushes in repositories
  without LFS keys are still process replacements.

### Kept, classified

- `real_git.rs` `process_resolution_inputs` calls `std::env::current_exe`. Identification: the path is excluded from the
  search for the real Git so that the wrapper never selects itself. Nothing is started from it. It carries
  `#[allow(clippy::disallowed_methods)]` with that reason.
- `entry.rs` `forwarded_to_self` and `FORWARD_TARGET_VARIABLE`. A guard that reports a wrapper found as its own
  real Git. It prevents an accidental loop; it starts nothing.
- `CLI_GIT_STACKED_PUSH`. Still set, now in the planner's environment and in the overlay of the merge push, so Git
  and its hooks can tell a merge push. It is not a start.
- `hooks_off.rs`. Points `core.hooksPath` at a directory that never exists, so traditional hook files
  (including the shim `git lfs install` writes) never run through the wrapper's Git children.
- Test support. Controls that start a helper copy of the test binary go through one excused helper per test crate
  (`test_support.rs` `this_test_binary`, `binary_support.rs` `this_test_binary`; one local allow in
  `candidate_consumer_tests.rs`, which is its own test crate). Binary-level controls start the wrapper on purpose;
  that is the subject under test, not a design of the wrapper.

### Left for phase 2 (the hook dispatcher)

- `main.rs` reads `args_os().next()` to dispatch on the name the wrapper was started as.
- `commit_command.rs:168` calls `current_exe` for the commit hook dispatcher.
- `hook_dispatch*.rs`, `commit_preparation.rs`, `commit_revalidation.rs`, `commit_conclusion.rs` and
  `hook_dispatch_entries.rs` implement the dispatcher.

Another change removes these. This document gets its own section for that phase in that branch; the allow-list entries
in `no_self_invocation_tests.rs` for `main.rs` and `commit_command.rs` go with it, and the stale-entry test enforces it.

### Found, not self-invocation, still open

- `manual_push_probe.rs` `hook_source` writes a `#!/bin/sh` script that records the update lines of a private dry-run
  push. It starts nothing recursive, but it is shell text written by the wrapper, which the owner's rule forbids. It is
  on the allow-list with that reason. A shell-free replacement needs a way to capture Git's update lines without a
  program of our own; the candidates are the trace-based and dry-run-based routes in the decisions below. This is the
  next item for the owner.
- `doctor.rs` holds the text git-lfs itself writes as its hook shim, only to recognise it. The wrapper never writes it.
  On the allow-list with that reason.

## Measurements (this host)

- Git prepends its exec path to `PATH` for the programs it starts (`setup_path` in Git's `exec-cmd.c`). A hook run by
  real Git therefore sees `/usr/libexec/git-core/git` for `git`, so `git-lfs` started from a hook calls the real Git,
  not the wrapper. No `PATH` shim is needed or wanted. The control
  `binary_lfs_parent_tests.rs` `git_starts_the_upload_and_no_wrapper_stands_between` records the executable of every
  ancestor of the stand-in `git-lfs`: the nearest is Git and exactly one wrapper is above it, the process the caller
  started.
- A `hook.<name>.command` without shell metacharacters is started by Git directly; the stand-in saw its parent
  process be Git, with no `sh` between. Git's own trace records `use_shell: true` for it, which is Git's request
  flag; the run itself needs no shell.
- `git push --dry-run` runs the configured `pre-push` hook (a dry run uploaded nothing only because the wrapper does
  not add the hook to dry runs), and it contacts the remote (a `git-receive-pack` child starts before the hook).
- A failing `pre-push` hook gives exit status 1 and `error: failed to push some refs to '<url>'` for every hook
  status tried (1 and 7), the same as a remote refusal. The text is translated (German run:
  `Fehler: Fehler beim Versenden einiger Referenzen nach ...`). Neither status nor text identifies the hook.
- With `GIT_TRACE2_EVENT` naming a file, Git writes for each hook a `child_start` event (`child_class` `hook`,
  `hook_name` `pre-push`, `argv[0]` the command string) and a `child_exit` event with the `code`, in any language.
  `-c trace2.eventTarget=...` is ignored (Git reads trace2 targets from the environment and the system and global
  files only).
- The remote value in the update line is the remote's advertised value for the ref, all zeros for a new ref, not the
  local remote-tracking ref (first push: zeros; second: the advertised old tip).

## Decisions

### Who starts `git-lfs`, and who explains

Options, with the ranking required by the repository rules:

- A. Git runs the plain hook command `git-lfs pre-push`; the parent reads Git's trace2 events for the hook
  and prints the explanation. Pros: Git passes the exact update lines, remote and URL; one config pair and one environment variable;
  no shell, no wrapper start; exact attribution in every locale. Cons: a caller who sets `GIT_TRACE2_EVENT`
  gets no explanation (the push still stops; their trace holds the evidence); depends on trace2 event fields of Git.
- B. The parent runs `git-lfs pre-push` itself with update lines it computes, then pushes with the hook off. Pros:
  the parent owns the whole step. Cons: the lines need the remote's advertised values, so a faithful computation
  needs a private dry-run push (a second remote round trip) and the recording hook that the probe writes as shell
  text; arbitrary refspecs and options would need a second implementation of Git's push planning that must stay in
  sync; the pushed result can differ from the probed one.
- C. A plain hook plus a text match on Git's `failed to push some refs` line. Cons: translated, and identical for
  remote refusals, other hooks and rules, so it blames the wrong cause.
- D. A hook that reports its own status through shell text. Banned by the owner.

Ranking: A > B > C > D. A over B because A passes Git's own lines with no second mechanism and no extra round trip,
where B must reproduce them. B over C because B at least attributes the failure correctly. C over D because C is
shell-free, though wrong in some cases. A was built.

Rejected inside B: auto-push already knows the remote, branch and tip, so it could run `git-lfs pre-push` itself
with the right input. It is rejected because the remote value is the remote's, not the tracking ref's, so it needs a
contact with the remote; and because it would be a second mechanism beside the one every other push needs.

The Rust `git-lfs` lookup (`lfs_push_precheck.rs`) refuses before Git starts when no `PATH` directory holds an
executable `git-lfs`, with the same words as before. It reads Git's push options with the existing table
(`command_push.rs`, so `--no-verify` in any accepted abbreviation, and `--dry-run`, skip it) and the caller's
`-c hook.cli-git-lfs-pre-push.enabled=false`. It reads the invocation's `PATH`, not the exec path Git prepends, so a
`git-lfs` that is only in the Git exec path would be refused; that is rare and the message names `--no-verify`.

### No runtime marker that refuses a nested start

A refusal such as "if `CLI_GIT_NESTED` is set, do not run" is not added. A user's own hook or script legitimately runs
`git status` through an absolute path or an installed wrapper; a runtime refusal would break that, and the forward
target guard (`forwarded_to_self`) already covers the one accidental loop. The design is removed and banned at build
time instead.

### What happens when the explanation inputs are odd

- A remote's message that looks like a trace event or like the explanation text cannot add an explanation: the trace
  file is written by Git alone, and only events of the session that wrote the first line, for a `pre-push` hook whose
  command string is exactly the upload command, count. Control:
  `binary_lfs_parent_tests.rs`
  `a_remote_refusal_is_not_blamed_on_the_upload_even_when_it_imitates_a_trace`,
  and the unit controls in `lfs_hook_trace_tests.rs`.
- Another configured `pre-push` hook that fails is not blamed on the upload (control
  `another_pre_push_hook_that_fails_is_not_blamed_on_the_upload`).
- A hook that fails twice reports the first failure. A cut-off last line is skipped.
- A caller's `GIT_TRACE2_EVENT` is never replaced. A global `trace2.eventTarget` is overridden by the environment
  variable for that one push, as Git defines it.

## Hang analysis: the owner's suspicion of a concurrent LFS deadlock

The owner's words: "I suspect sometimes the wrapper hangs in post-commit auto-push due to concurrent LFS deadlock.
Either fix it or prove me wrong." No stall was observed; the notice `waiting for process N to finish pushing ...;
giving up after N seconds` is printed when a wait begins and names its limit, so it records a queue and not a
give-up.

### Lock graph of the post-commit auto-push

Reading of `commit_command.rs`, `post_commit.rs`, `auto_push.rs`, `auto_push_single_flight.rs`, `bounded_push.rs`,
`process_tree.rs`, `lfs_pre_push.rs` and `owner_lock.rs`:

- Startup recovery runs in every wrapper process before configuration loads. It takes no lock that a push holds.
- The commit transaction holds the landing lock (`commit_landing_loop.rs`) while it lands; auto-push starts after
  `run_commit_transaction` returns, so the landing lock is not held during the push.
- Auto-push holds one per-branch owner lock (`<common dir>/cli-git/push/<key>.lock`) from its decision to push until
  it has written the record. Under it run: bounded queries, one bounded push (with the upload inside Git), and, for a
  stale rejection only, up to `MAX_RECONCILE_ATTEMPTS` rounds of one bounded fetch plus one bounded push.
- A waiter polls that directory and gives up at `lock_wait_limit`. A dead or reused holder is retired by
  `owner_lock_holder_is_alive` (checks the process birth identity, treats zombies as dead, and handles another PID
  namespace).
- The processes below the push (Git, `git-lfs`, a transfer agent, the Git commands `git-lfs` runs) take Git's own
  locks (`index.lock`, ref locks) and LFS's own files. None of them takes or waits for a wrapper lock: since this
  change, none of them can start the wrapper at all. In the old design the nested start at the hook entry
  (`entry.rs:289`) ran after `hooks_off::activate()` and before recovery, configuration or any lock, so it also could
  not wait for a lock the parent held.

So the wrapper's wait-for graph has no cycle through the upload: the push tree waits for nothing the wrapper holds, and
the wrapper waits for the push tree only through bounded waits.

### Experiments

- `binary_lfs_hang_tests.rs`
  `two_worktrees_committing_at_once_with_a_slow_upload_both_finish_without_waiting_for_each_other`: two worktrees of one repository commit at the same moment, both with LFS keys and a stand-in `git-lfs` that takes
  its delay. Both exit 0, neither prints a wait notice, both branches reach the origin, nothing is left. Not
  reproduced: no hang.
- `a_detached_upload_helper_holding_the_output_open_does_not_hold_the_commit`: the stand-in starts a helper in a new
  session whose parent has gone, holding the push's output file open, and itself never exits. The wrapper stops the
  push at its deadline and returns within the deadline plus `STOP_GRACE`. Refuted: the push output goes to an
  anonymous regular file that is read to its current end (`bounded_push.rs` `read_output`), so no holder of the file
  can delay the read. The helper itself is a leak that the tree walk cannot see after its parent exited; it is a leak
  and not a hang. Making such helpers findable would need the wrapper to become a child subreaper, which makes it
  responsible for reaping adopted processes; not done.
- Every other pipe read in the source (`git_metadata.rs`, `manual_push_probe.rs`, `manual_push_range.rs`,
  `child_watch.rs`, `candidate_reader.rs`, `index_lock_holders.rs`, `process_identity.rs`)
  is outside the auto-push path.
  Two of them, the probe's dry-run push and its `ls-remote`, have no deadline of their own and run before an explicit
  push; a remote that never answers would hold such a push until Git's own timeouts end it. This is a finding for the
  owner, not changed here.

### The four hypotheses

- Detached descendant keeping the pipe open: refuted above for the auto-push (a file, not a pipe).
- Waiting for EOF after the kill: the stop path killed the child and then called a blocking `wait`. A child in
  uninterruptible I/O is not collected until the kernel releases it, so that wait could outlast every bound. Fixed:
  `stop_push` now polls `try_wait` for at most the grace and leaves the child to be collected when the process ends.
  Not reproducible without a stuck kernel state; covered by the unchanged stop controls.
- A stalled transfer holding the push for the whole deadline while waiters queue: the first waiter joins the failed
  record when the holder's push is stopped. What was found by reading is that the waiter's limit
  (`lock_wait_limit`: push deadline plus `STOP_GRACE` plus `LOCK_WAIT_MARGIN`) covers one push, while a holder that
  meets a stale rejection runs up to `MAX_RECONCILE_ATTEMPTS` rounds of a fetch and a push under the same lock, so a
  legitimate holder can outlast a waiter's limit. The waiter then reports a timed-out wait, the commit stays
  saved, nothing hangs; it is a mismatch of two bounds. Recommended fix, not built because it changes the owner's
  chosen limits: one time budget shared by everything a holder does under the lock, with the waiter's limit
  derived from that budget.
- Liveness of a holder record: the check compares process birth identity, treats zombies and exited processes as dead
  and has a rule for another PID namespace (`process_identity.rs`, `owner_lock_record.rs`). A holder that is alive and
  stuck is outlasted by the waiter's limit and then reported and left in place. Refuted as a source of a hang.

### The two failures under load

- `an_auto_push_to_a_server_that_hangs_up_keeps_the_commit_and_holds_nothing` failed at about 9.4 seconds in four
  containers at once. The panic text was not captured. This host shows 0.5 seconds idle and 1.3 seconds under a
  quarter-CPU cap (`systemd-run --user --property=CPUQuota=25%`), which is not enough slowdown to reproduce.
  The test has no 10-second bound: the commit is bounded by `COMMIT_BOUND_SECONDS`,
  the server and tripwire by `DEADLINE_MILLISECONDS`, and
  its second commit sets a one-second push deadline, which is also the one-second deadline of every Git query of that
  commit (`query_deadline` is the smaller of the push deadline and `QUERY_CEILING`), so a query slowed past a
  second under load makes that commit report that it could not run. The next gate failure needs
  `--failure-output final` to capture the text before this can be decided; it is not explained by a real stall.
- `an_auto_push_with_a_git_lfs_that_never_exits_is_stopped_with_everything_it_started` uses the same one-second
  deadline for the commit and then the second commit; the same remark applies.

### Every bound that limits a hang

- `CLI_GIT_PUSH_DEADLINE_SECONDS` / `DEFAULT_PUSH_DEADLINE` in `bounded_push.rs`: one push or one fetch (30 seconds
  on main at the time of writing).
- `STOP_GRACE` (5 seconds): after the polite stop and again after the kill.
- `QUERY_CEILING` (120 seconds) with `query_deadline`: every Git query of the auto-push.
- `LOCK_WAIT_MARGIN` (60 seconds) and `lock_wait_limit`; `CLI_GIT_PUSH_LOCK_WAIT_SECONDS`: a waiter.
- `MAX_PUSH_OUTPUT` (4 MiB): a flooding push is stopped.
- `LOW_SPEED_LIMIT` and `LOW_SPEED_TIME` (1000 bytes per second for 60 seconds): Git ends a stalled HTTP transfer.
- `MAX_RECONCILE_ATTEMPTS` (10): rounds of merge and push under one lock.
- `CLI_GIT_CHILD_WAIT_SECONDS` and `CLI_GIT_LOCK_WAIT_SECONDS` (1800 seconds in the notices): other children and the
  index and landing waits.
- The hook trace read stops at 16 MiB.

The waiter's notice now also says how long the holder has held the lock, the holder's command line (Linux), and that its
push is stopped at the push deadline (`lock_holder_facts.rs`).

## The bans

- `clippy.toml` in `package/git-policy/cli` bans `std::env::current_exe` (the file also repeats the repository root
  file's entries, because Clippy reads the nearest file only). Remaining uses carry
  `#[allow(clippy::disallowed_methods, reason = ...)]` naming what they identify.
- `src/native/no_self_invocation_tests.rs` scans the release source for: the running executable's path, `argv[0]`
  reads, shell text, hook command definitions other than the plain upload command, management words passed to a
  process, and `Command::new` on the wrapper's own path. Each allow-list entry names its file and rule and must still
  match code.
- Both guards were shown to fail: a planted `std::env::current_exe()` failed Clippy with the rule's note, and planted
  shell and hook-command text failed the scan with `file:line [rule]`. Both plants were removed.

## Phase 2: the hook dispatcher

Filled by the change that removes the argv[0] dispatcher and the commit-path hook dispatch.
