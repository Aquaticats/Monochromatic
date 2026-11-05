# Closure lint suppressions in the reader, push, lock, subvolume and index folders

The folders `repo_read`, `push`, `lock`, `subvolume` and `index` under `src/native/` had their `rust/avoidable-closure` warnings
cleared by named functions, loops and `match` expressions.
The closures listed here remain, each with an `expect(avoidable_closure, reason = "...")` on its `let` statement or item.
`doc/decision/rust-closure-lint-dylint.md` allows exactly this form:
the rule reports a closure unless it captures state and a callee keeps it, and "only rustc's `expect` suppresses the Dylint lint".

## Why configuration cannot do it

The rule declares itself unsuppressible by comment, and its severity can only be raised or lowered per path
(`package/linter/monochromatic-lint/README.md`, "The rule has no comment directive").
Lowering the severity for these folders would loosen the rule for every closure in them,
which `CLAUDE.md` LN7 forbids without the owner's approval.
The `expect` is per closure, and rustc reports one that suppresses nothing (`unfulfilled_lint_expectations`),
so a suppression cannot go stale unnoticed.

## Why no named function, loop or match can replace these closures

### A closure that `catch_unwind` runs

Sites: `index/index_reset_file.rs` (`reset_index_file`) and `repo_read/repo_read_hook.rs` (`attempt`).

`std::panic::catch_unwind` takes a value that can be called with no arguments.
A function item cannot carry the borrowed arguments of the reader it guards, so the arguments must be captured.
The callee runs the closure at once and does not keep it, so the lint's help ("a loop or a named function with parameters") has nothing to put in its place:
the call that must be guarded is the closure's body.

### A state-carrying `dyn Fn` or `dyn FnMut` parameter of a seam

Sites:

- `subvolume/subvolume_index_files.rs`: the stat lookup and the reference check handed to `rewrite_index_stat_with`.
- `subvolume/subvolume_migrate.rs` and `subvolume/subvolume_rollback.rs`: the line collector handed to `refresh_all_indexes`.
- `push/auto_push_reconcile.rs` and `push/push_stacking.rs`: the push handed to `integrate_and_push`.
- `push/manual_push_probe.rs`: the query handed to `resolve_push_remote`.
- `push/start_retry.rs`: the start handed to `retry_refused_start` (the `impl RetryStart for Command` item and `exec_retrying`).
- `repo_read/repo_read_odb.rs`: the base reader handed to `Pack::read`.

Each of these parameters is a `&mut dyn FnMut(..)` or `&dyn Fn(..)` that the unit tests and, for `install_index_in` and `refresh_all_indexes`,
other folders fill with closures of their own.
The state (the Git context, the tokens, the list the lines go into) has to travel with the callable.
A named function cannot carry it, and a loop has no call to replace.
Removing these closures means turning each parameter into a trait with a named implementing type,
which changes the public signatures that `worktree/worktree_clone.rs` and many test sidecars call;
that is a design change for the owner, not a lint fix, and it is left open.

### Callbacks given to the ordered worker pool

Sites: `subvolume/subvolume_index_files.rs` (`refresh_all_indexes_with`),
`push/manual_push_lifecycle.rs` (`scopes_of_updates_on`) and `push/manual_push_probe.rs` (`probe_manual_push_updates_on`).

`run_ordered` takes a `work` callback that runs on worker threads and an `on_event` callback that collects results in order.
Both carry the caller's state (the Git context, the result lists, the first failure) into the pool.
The callbacks are the pool's interface, and its other callers (the parallel runner's own controls) use closures too.
