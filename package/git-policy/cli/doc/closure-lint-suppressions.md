# Closure lint suppressions in the Git wrapper

The Dylint lint `avoidable_closure` (`package/linter/monochromatic-lint-closure`) runs over this crate as the rule
`rust/avoidable-closure`.
The owner's decision `doc/decision/rust-closure-lint-dylint.md` reports a closure unless it captures state and is a
direct argument whose callee parameter requires `'static`.
A closure this crate cannot replace therefore carries rustc's own attribute on the statement that holds it:
`#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]`.
`expect` makes rustc report `unfulfilled_lint_expectations` as soon as it suppresses nothing,
so a suppression cannot outlive its closure.

Configuration cannot do it instead:
the rule's settings give a severity and a library path per rule
(`package/linter/monochromatic-lint/src/rust_rule_settings.rs`,
`rust_rule_settings`),
with no allow-list of callees,
and lowering the severity would also let a replaceable closure in
(`AGENTS.md` LN7).

## Standard-library functions that accept only a callable

Every suppression in this crate is one of these five functions,
read from the sysroot source of `nightly-2026-11-10` and from the locked `tokio` 1.54.1:

- `std::thread::scope`:
  `F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> T`
  (`library/std/src/thread/scoped.rs`).
  The callable is bounded by the scope and not by `'static`,
  and it is handed the scope as its only argument.
- `Builder::spawn_scoped`:
  `F: FnOnce() -> T + Send + 'scope`
  (`library/std/src/thread/scoped.rs`).
  It passes no argument to the callable,
  so the thread's body must carry its state.
- `std::panic::catch_unwind`:
  `F: FnOnce() -> R + UnwindSafe`
  (`library/std/src/panic.rs`).
  It passes no argument either,
  so the caught job must carry its state and its function.
- `tokio::task::LocalKey::sync_scope`:
  `F: FnOnce() -> R`
  (`tokio-1.54.1/src/task/task_local.rs`).
  No argument again,
  so the landing's body must carry its state.
- `tokio::task::LocalKey::try_with`:
  `F: FnOnce(&T) -> R`
  (`tokio-1.54.1/src/task/task_local.rs`).
  It passes only the task-local's value,
  so the closure must carry the arguments and the function it applies.

No named function can replace these closures,
because none of these functions passes the caller's state as an argument,
and no loop can run work on another thread.

## Where they are, and what each carries

- `src/native/foundation/side_by_side_threads.rs`,
  `in_thread_scope`:
  `std::thread::scope`,
  carrying the `ScopeBody` the caller wrote.
  This is the one place in the crate that makes a scope callable,
  so no other module writes a `std::thread::scope` closure.
- `src/native/foundation/scoped_start.rs`,
  `start_or_run_with`:
  `Builder::spawn_scoped`,
  carrying the slot that holds the job,
  because the builder consumes its callable even when the thread cannot start.
- `src/native/foundation/call_catching_panics.rs`,
  `call_catching_panics`:
  `std::panic::catch_unwind`,
  carrying the state and the function of the job run on the calling thread.
- `src/native/landing/landing_state_carried_by_its_task.rs`,
  `in_a_landing_with` and `on_the_landing_state`:
  `LocalKey::sync_scope` and `LocalKey::try_with`,
  carrying the arguments and the function each applies to the landing's state.
- `src/native/push/auto_push_reconcile.rs`,
  `BridgedReconcile::run`:
  `Builder::spawn_scoped`,
  carrying the channel of one reconcile run and the merge push of the command that asked for it.
  That push is `gated_merge_push` for a person's explicit `git push`,
  which starts Git synchronously and is refused from inside the runtime,
  so it runs on a plain thread beside the one reconcile loop.

Every other closure the rule found in this crate was replaced:
by a named function,
a path such as `str::trim`,
a loop,
a `match`,
or a struct that implements one of this crate's own traits:
`ScopeBody`,
`MergePusher`,
`MergePusherAwaiting`,
`BranchMove`,
and `GitRunner`.
A plain closure still implements each of those traits,
so a caller with nothing to carry may pass one.
