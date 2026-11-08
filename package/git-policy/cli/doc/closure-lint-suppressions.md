# Closure lint suppressions in the Git wrapper crate

The Dylint lint `avoidable_closure` (`package/linter/monochromatic-lint-closure`) runs over this crate as the rule
`rust/avoidable-closure`.
The owner's decision,
`doc/decision/rust-closure-lint-dylint.md`,
reports a closure unless two things hold:
it captures state,
and it is a direct argument whose callee parameter requires `'static`,
"so the callee keeps the state past the call and no named function,
loop,
or `match` can replace the closure".
The only suppression is rustc's own attribute on the statement that holds the closure:

```text
# package/git-policy/cli/src/native/foundation/side_by_side_threads.rs
#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]
```

`expect` makes rustc report `unfulfilled_lint_expectations` as soon as it suppresses nothing,
so a suppression cannot outlive its closure.

This crate carries five suppressions,
one for each of three standard-library functions and two methods of tokio's task-local key that accept only a callable.
Every other closure that earlier passes of the lint left behind was replaced by a function and the state it reads,
a value with a method,
an enum,
or a guard (see "What replaced the other suppressions").

## Why configuration cannot do it

`monochromatic-lint.config.jsonc` gives the rule a severity and a library path per file pattern,
and nothing else
(`package/linter/monochromatic-lint/src/rust_rule_settings.rs`,
the settings of `rust/avoidable-closure`).
There is no allow-list of callees,
so a callee that takes a non-`'static` callable cannot be named there.
The rule declares itself unsuppressible by comment
(`package/linter/monochromatic-lint/README.md`,
section "Closure check").
Lowering the severity for the three files would also let a replaceable closure into them,
which `AGENTS.md` LN7 forbids.
The lint decides from rustc's own answer:
`parameter_requires_static` in `package/linter/monochromatic-lint-closure/src/static_bound.rs` reads the `TypeOutlives`
clauses of the callee and the lifetime of a `dyn` object in a `Box` parameter,
and `capture.rs` reads `tcx.upvars_mentioned`.
The three callees below have no `'static` clause,
so the lint reports each closure although no replacement exists.

## Standard-library functions that accept only a callable

The proof for each site is the signature in the sysroot source of the toolchain that builds the lint
(`nightly-2026-11-01`,
`library/std/src/thread/scoped.rs`,
`library/std/src/panic.rs`):

- `std::thread::scope`:
  `scope<'env, F, T>(f: F) -> T where F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> T`.
  The callable is bounded by the scope,
  not by `'static`,
  because the threads borrow the caller's data for the scope only.
- `Builder::spawn_scoped`:
  `F: FnOnce() -> T + Send + 'scope`,
  with no argument but the callable.
  The thread body must carry its state,
  so it is a closure.
- `std::panic::catch_unwind`:
  `catch_unwind<F: FnOnce() -> R + UnwindSafe, R>(f: F) -> Result<R>`.
  The guarded call takes no arguments,
  so the arguments it needs must be captured.

- `tokio::task::LocalKey::try_with`:
  `try_with<F, R>(&'static self, f: F) -> Result<R, AccessError> where F: FnOnce(&T) -> R`,
  which passes the stored value and nothing else.
  The caller's arguments and the function that reads the value must be captured.
- `tokio::task::LocalKey::sync_scope`:
  `sync_scope<F, R>(&'static self, value: T, f: F) -> R where F: FnOnce() -> R`,
  with no argument to its callable.
  The proofs for the two tokio methods are `src/task/task_local.rs` of tokio 1.54.1 in the local registry.

No named function can replace these closures,
because none of the five functions passes an argument to its callable.
No loop can,
because a loop cannot run on another thread,
and `catch_unwind` is the only way to turn a panic into a value.
The crate therefore makes each of these closures once,
in a foundation function that takes a plain function and its state,
and every caller passes a function and a state (a tuple of borrows,
or a small struct) and writes no closure.

The sites,
each with the one closure it holds:

- `src/native/foundation/side_by_side_threads.rs`,
  `in_thread_scope`:
  the closure given to `std::thread::scope` carries a `ScopeBody` value and runs its `run` method.
  Its users are `run_all_side_by_side`,
  `run_beside_caller`,
  `run_ordered` (`foundation/parallel_runner.rs`)
  and the syncs of `landing/service_durable.rs`.
- `src/native/foundation/scoped_start.rs`,
  `start_or_run_with`:
  the closure given to `Builder::spawn_scoped` carries the slot that holds the job (a function and its state),
  because `spawn_scoped` consumes its callable even when the thread cannot start,
  and the job must then run on the caller.
- `src/native/foundation/call_catching_panics.rs`,
  `call_catching_panics`:
  the closure given to `catch_unwind` carries the state and the function.
  Its users are `guarded` (`repo_read/repo_read_hook.rs`,
  called by that file and by `index/index_reset_file.rs`),
  `take_items` (`repair/package_repair_cargo_concurrent.rs`),
  the inline path of `start_or_run_with`
  and `while_panics_are_expected` (`invocation/panic_notice.rs`),
  which takes a function and its state and silences the panic notice around the call;
  its users are `run_blocking_job` (`foundation/blocking_job.rs`) and `build_with_deadline` (`foundation/lazy_runtime.rs`).

- `src/native/landing/landing_state_carried_by_its_task.rs`,
  `on_the_landing_state` (the closure given to `try_with` carries the arguments and the function) and `in_a_landing_with`
  (the closure given to `sync_scope` carries the arguments and the body).
  The users of `on_the_landing_state` are the named functions of the landing's state items (`service_durable.rs`,
  `service_index_claim.rs`,
  `service_index_ownership.rs`,
  `service_index_prestage.rs`,
  `service_index_refused_lock.rs`,
  `service_land_git_steps_and_import_stream.rs`,
  `service/service_busy_step_record_on_this_thread.rs`);
  the user of `in_a_landing_with` is `serve` (`service/service_server.rs`).

A change that lets one of these callees take a function and an argument (a `catch_unwind_with`,
for example)
removes the matching suppression,
and rustc then reports it as `unfulfilled_lint_expectations` until it is deleted.

## What replaced the other suppressions

The earlier lint passes left about fifty `expect`s in four notes
(`src/native/worktree/worktree_closure_lint_suppressions.md`,
`doc/closure-lint-suppressions-reader-push-lock-subvolume-index.md`,
`doc/closure-lint-suppressions-service-landing-and-runner-folders.md`
and `doc/planning/cli-git-closure-lint-suppressions.md`).
All four notes are deleted;
this one replaces them.
Each callee took a named type in place of a closure parameter,
by one of five shapes.

- A guard,
  begun before the call and finished after it:
  `begin_step` and `end_step`,
  `begin_work` and `end_work` (`worktree/add_timing.rs`) in place of `step` and `work`;
  `QuietMeasurement` (`process/thread_quiet.rs`) in place of `measured_quiet`;
  `WaitReporter::notice_due` with `print_notice` (`push/wait_progress.rs`) in place of `poll`.
- A function and the state it reads,
  run by the foundation helpers above:
  the copy,
  install and removal workers of `worktree_copying/`,
  the walks and scans of `worktree/`,
  the repairs of `repair/`,
  and `guarded`.
- A trait that the caller implements on a small struct that holds the state:
  `OrderedWork` and `OrderedEvents` (`foundation/parallel_runner.rs`),
  `LineSink` (`foundation/line_sink.rs`),
  `StatLookup` and `StatTrust` (`subvolume/subvolume_index.rs`),
  `RefusableStart` (`push/start_retry.rs`),
  `MergePusher` (`push/auto_push_reconcile.rs`),
  `BaseByName` (`repo_read/repo_read_pack.rs`),
  `Announcer` (`landing/service_index_lock.rs`),
  `Record` (`landing/service_index_hold.rs`),
  `ClaimExpiry` (`landing/work_item_recovery.rs`),
  `NoteSource` (`service/service_reply.rs`),
  `EnvironmentVariables` and `LandedCommitReader` (`standards_review/standards_review_run.rs`),
  `CommitNamer` (`push/push_failure_notice.rs`),
  `GitRunner` (`worktree_copying/worktree_copy.rs`),
  `GitQuery` (`lfs/lfs_upload_settings.rs`) and `WatchTermsSource` (`transaction/transaction_git.rs`).
- An enum that names the cases:
  `StaleMark` (`landing/service_index_reclaim.rs`).
- A value that the caller computes first:
  the lists of states that `run_all_side_by_side` receives.

## Test closures and the blanket implementations

Test code is exempt from the lint ("tests,
fuzz crates and bench crates are exempt",
owner answer in
`doc/decision/rust-closure-lint-dylint.md`),
and the lint compiles the library and binaries only.
Most seams above have test sidecars that fill them with closures.
Each such sidecar holds a blanket `impl<F: Fn(..)> Trait for F`,
declared inside the sidecar,
so it exists only in test builds:

- `foundation/parallel_runner_tests.rs`:
  `OrderedWork` and `OrderedEvents`.
- `foundation/line_sink_tests.rs`:
  `LineSink`.
- `subvolume/subvolume_index_tests.rs`:
  `StatLookup` and `StatTrust`.
- `push/start_retry_tests.rs`:
  `RefusableStart`.
- `push/auto_push_reconcile_tests.rs`:
  `MergePusher`.
- `push/push_failure_notice_tests.rs`:
  `CommitNamer`.
- `transaction/transaction_git_tests.rs`:
  `WatchTermsSource`.
- `landing/service_index_hold_tests.rs`:
  `Record`.
- `landing/service_index_lock_tests.rs`:
  `Announcer`.
- `landing/work_item_tests.rs`:
  `ClaimExpiry`.
- `standards_review/standards_review_run_tests.rs`:
  `EnvironmentVariables` and `LandedCommitReader`.
- `worktree_copying/worktree_copy_lifecycle_tests.rs`:
  `GitRunner`.

Because the blanket impl is absent from the release and lint builds,
production code cannot route a closure through it:
a production closure given to one of these parameters would not compile.
Three blanket implementations remain in production code and predate this change:
`GitQuery` (`lfs/lfs_upload_settings.rs`),
`ConcurrentWork` and `ItemLabel` (`repair/package_repair_cargo_concurrent.rs`).
Shipped code implements each of the three on a named struct,
so no closure is passed through them,
and a production closure given to one of them would still be reported,
because the parameter is a `&dyn` without `'static`.
