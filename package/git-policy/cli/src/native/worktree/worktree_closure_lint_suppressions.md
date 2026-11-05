# Closure lint suppressions in worktree and worktree_copying

The Dylint lint `avoidable_closure` (`package/linter/monochromatic-lint-closure`) runs over this crate as the rule
`rust/avoidable-closure`.
The owner's decision is `doc/decision/rust-closure-lint-dylint.md`:
a closure passes only when it captures state and is a direct argument whose callee parameter requires `'static`,
so that the callee keeps the state past the call and no named function, loop or `match` can replace the closure.
Everything else in `src/native/worktree/` and `src/native/worktree_copying/` was rewritten as a named function,
a loop, a `match` or a small named type.
The sites below remain, each with an `expect(avoidable_closure, reason = "...")` on the statement that holds the closure.
`expect` makes rustc report the attribute as `unfulfilled_lint_expectations` as soon as it suppresses nothing,
so a suppression cannot outlive its closure.

## Why configuration cannot do it

`monochromatic-lint.config.jsonc` gives the rule two settings and no others, `severity` and `library`
(`package/linter/monochromatic-lint/src/rust_rule_settings.rs`, the settings of `rust/avoidable-closure`).
There is no allow-list of callees, so a callee that takes a non-`'static` callback cannot be named there.
The lint decides from rustc's own answer:
`parameter_requires_static` in `package/linter/monochromatic-lint-closure/src/static_bound.rs` reads the `TypeOutlives`
clauses of the callee, and `capture.rs` reads `tcx.upvars_mentioned`.
The callees below have no `'static` clause (a scope bound, or a trait object with an anonymous bound),
so the lint reports the closure although no replacement exists.
Loosening the rule is not an option (`CLAUDE.md` LN7).

## Group scope

`std::thread::scope(|scope| ...)` and `start_or_run(scope, || ...)` (`src/native/foundation/scoped_start.rs`)
take a closure bounded by the scope.
The worker must borrow the caller's paths, counters and slices for the length of the scope;
a named function cannot receive them without a closure, and a loop cannot run on another thread.
The attribute sits on the `let` that binds the scope's result, which covers the scope body and the workers inside it.

- `worktree_copying/worktree_copy_compare.rs`, the two manifest walks of the final source comparison.
- `worktree_copying/worktree_copy_install.rs`, the install workers over slices of leaves.
- `worktree_copying/worktree_copy_journal_removal.rs`, the removal workers over subtrees.
- `worktree_copying/worktree_copy_stage_leaf_copying.rs`, the copy workers over slices of the manifest.
- `worktree/worktree_clean.rs`, the untracked scan beside the tracked proof.
- `worktree/worktree_snapshot_source.rs`, the configuration read and the ignore probe beside the clean check.

## Group run_ordered

`OrderedRun` (`src/native/foundation/parallel_runner.rs`) takes `work: &(dyn Fn(usize, &T, &mut dyn FnMut(&str)) -> R + Sync)`
and `on_event: &mut dyn FnMut(Event<'_, R>)`.
Both callbacks capture the caller's state (the platform, the options, the counters the events update),
and the type is owned outside these two folders.

- `worktree/worktree_convert_command.rs`, the conversion of several worktrees side by side.
- `worktree/worktree_trash_entries.rs`, the removal of the entries of one trash directory side by side.

## Group install_index_in

`install_index_in` (`src/native/subvolume/subvolume_index_files.rs`) takes the stat lookup as
`stats: &mut dyn FnMut(&[u8]) -> Option<StatFields>`, owned outside these two folders.
The lookup reads the table or the cursor the caller built.

- `worktree/worktree_clone.rs`, the table of fields the clones made.
- `worktree/worktree_snapshot.rs`, the table of the walk and the cursor over entries the walk did not cover.

## Group git runner

`GitRunner` (`src/native/worktree_copying/worktree_copy.rs`) is a `dyn FnMut(&GitStep) -> Result<GitStatus, _>` that the
ladder calls once per Git step.
The runner built in `worktree_copying/worktree_copy_forward.rs` carries the invocation, the real Git, the overlay and the
parallel-checkout decision that the first checkout step makes.
Replacing it means changing `GitRunner` and every ladder file to take a trait object or a struct;
that is a design change of its own, not part of this lint pass.

## What would remove them

- The callee types named above could take a struct that implements a trait instead of a closure;
  `complete_journal` did this in this pass (`SnapshotSource` in `worktree_copying/worktree_copy_transaction.rs`).
- A scoped-thread helper that takes the state and a function pointer would remove the scope group,
  the same way `begin_step` and `end_step` in `worktree/add_timing.rs` removed the closures of the timed steps.
