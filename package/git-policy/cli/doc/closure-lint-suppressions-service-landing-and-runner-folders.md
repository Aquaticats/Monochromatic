# Closure lint suppressions in the landing, service, transaction, invocation, process and foundation folders

The Dylint lint `avoidable_closure` (`package/linter/monochromatic-lint-closure`) reports a closure unless it captures
state and is a direct argument whose callee parameter requires `'static`
(`doc/decision/rust-closure-lint-dylint.md`, section "Accepted decision").
The six folders `landing`, `service`, `transaction`, `invocation`, `process` and `foundation` under `src/native`
replace every other closure with a named function, a loop or a `match`.
The closures listed here remain, each under rustc's own attribute

```text
#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]
```

because the callee takes a callable whose type is not `'static`, so the lint reports it,
and no named function, loop or `match` can carry the state the callable needs.

## Why configuration cannot do it

The two things that decide a report are the closure's captures (`TypeckResults::closure_min_captures_flattened`) and the
callee parameter's `'static` bound (`parameter_requires_static` in `package/linter/monochromatic-lint-closure/src/static_bound.rs`,
which reads the declared signature of the callee).
The linter's configuration (`monochromatic-lint.config.jsonc`, block `rust/avoidable-closure`) has three settings:
`severity`, `library` and the per-file `off` for tests.
There is no callee allow-list and no per-path exemption inside a source file,
and `package/linter/monochromatic-lint/README.md`, section "Closure check", states that the rule "declares itself
unsuppressible" by comment directive.
Turning the rule off for these files would loosen it for closures that can be replaced, which the owner has not approved
(`CLAUDE.md` LN7).
So the attribute on the closure's function or `let` statement is the only mechanism left.

## The linted values and why none can be replaced

Every callee below declares a parameter of a type with no `'static` bound
(`&dyn Fn(..)`, `&mut dyn FnMut(..)`, a generic `F: FnOnce() -> T + Send + 'scope`, or an `impl Fn` return type),
and the closure's captured state is the whole reason it exists:

- `foundation/parallel_runner.rs`, `work_lanes`:
  `work` is `&(dyn Fn(usize, &T, &mut dyn FnMut(&str)) -> R + Sync)`,
  and the callback it hands to the work must send each line through the worker's channel with the item's index.
- `foundation/parallel_runner.rs`, `run_ordered`:
  `std::thread::scope` and `start_or_run` take only a callable;
  the scope closure carries the items, lanes and work, and the worker closure carries the same plus the sender.
- `foundation/scoped_start.rs`, `start_or_run`:
  `Builder::spawn_scoped` takes only a callable, which carries the slot that holds the caller's closure.
- `invocation/wrapped_command_notices_snapshots_and_manual_push.rs`, `show_push_failures`:
  `take_push_failures_for` and `take_orphaned_failures` (`push/push_failure_notice.rs`, outside the six folders)
  take `&dyn Fn(&str) -> String`;
  the callback asks Git for a short name with the context, the directory and the object directory.
- `landing/service_index_reclaim.rs`, `remove_stale`:
  `reclaim_if` takes `&dyn Fn(&Marked) -> bool`, and the test for one batch carries the batch number.
- `landing/service_land_finish.rs`, `follow_up`:
  `release_flushed_with_record` takes `&dyn Fn() -> std::io::Result<()>`, and the step carries the journal and the intent.
- `landing/service_round.rs`, `drain_directories`:
  `OrderedRun.work` and `OrderedRun.on_event` are `&dyn Fn` and `&mut dyn FnMut` fields;
  the work carries the handlers and the time, and the event sink carries the answer being built.
- `landing/work_item_recovery.rs`, `requeue_expired_reporting`:
  `requeue_with` takes its expiry test as `&dyn Fn(&ClaimTime) -> bool`, and its tests pass their own callables;
  this test carries the reading and the lease.
- `service/service_review.rs`, `review_variables`:
  the environment lookup is returned as `impl Fn(&str) -> Option<String>` and carries the exposed key.
- `service/service_review.rs`, `run_review`: `review_landed` takes `&dyn Fn(&[String]) -> ..`, and the reader carries the
  context, the directory, the commit and the subject.
- `service/service_server_connection_handling.rs`, `stuck_notice`:
  the note source is returned as `impl FnMut() -> Option<ServiceMessage>` and carries the board, the request identity and the
  last key it said.
- `service/service_server_land_group.rs`, `land_group`:
  `land_batch_timed` takes `&dyn Fn(Announce)`, and the announcer carries the streams of the batch.
- `landing/service_durable.rs`, `sync_all_at_once`:
  `std::thread::scope` and `start_or_run` as above, carrying the batch of pending syncs.
- `transaction/transaction_git.rs`, `run_git`: `add_timing::step` (outside the six folders) runs a callable under a timing
  label, and the callable carries the context and the request.
- `transaction/transaction_git.rs`, `run_git_pausing`: `retry_refused_start_with` (`start_retry.rs`, outside the six folders)
  takes `&mut dyn FnMut() -> std::io::Result<_>`, and the attempt carries the context, the request and the output it keeps.
- `transaction/transaction_git.rs`, `run_git_once`: `run_git_once_with` takes `&dyn Fn() -> WatchTerms`, and its tests pass
  their own callables; the production one carries the request.

A callable that cannot be replaced by changing a signature inside the six folders was left as it is.
Where the callee is inside the six folders and the closures it receives were few and local, the callee was changed instead
(for example `recover_selected` in `landing/service_land_finish.rs` takes an instance name where it took a `&dyn Fn`).
