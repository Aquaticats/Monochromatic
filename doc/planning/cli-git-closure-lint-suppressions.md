# Closure lint suppressions in the Git wrapper crate

Status:
written on 2026-11-04 with the branch `refactor/git-policy-cli-lint-command-and-policy-folders`.
It records every `expect(avoidable_closure, ...)` that the folders `merge`, `lfs`, `repair`, `shadow`, `conflict`,
`commit`, `dependent_version`, `standards_review`, `health`, `candidate`, `config`, `command`, `policy`, `markdown`,
`binary`, `testing` and `source_guard` of `package/git-policy/cli/src/native/` carry,
the reason each one stays,
and why the lint's configuration cannot take its place.

## What the rule asks

[The decision][decision] makes the Dylint lint `avoidable_closure` report a closure unless two things hold:
the closure captures state,
and it is a direct argument whose callee parameter requires `'static`.
The lint's help says to replace a closure by a named function, or by a loop when it captures state.
Every other closure in those folders was replaced that way.
The ones below remain because the callee takes a closure that borrows the caller's data for a bounded time,
which no named function, loop or `match` can supply.

## Why configuration cannot do it

The lint has no allow-list, no per-callee setting and no per-file setting;
`monochromatic-lint` only turns the rule on and sets its severity
(`package/linter/monochromatic-lint-closure/README.md`, section "Criterion").
The decision makes `expect` the one suppression
(`#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]`),
and the crate's `Cargo.toml` carries the `check-cfg` entry for `dylint_lib` since commit `6cc9175063`.
Loosening the rule is not allowed.

## The suppressions, by reason

### Scoped threads that borrow the caller's data

`std::thread::scope` and `start_or_run` (`foundation/scoped_start.rs`) take a closure that lives for the scope, not for `'static`.
The threads read slices, paths and requests that the caller owns.
A `'static` thread would need an owned copy of each, which changes what the code costs and what it can borrow.

- `repair/package_repair.rs`, `repair_side_by_side`: the pnpm, toolchain and Cargo repairs side by side.
- `repair/package_repair_tree_clone.rs`, `place_slices_side_by_side`: one thread per slice of entries.
- `repair/package_repair_cargo_concurrent.rs`, `map_concurrent`: one worker per queue reader.

### A guarded call that borrows

`std::panic::catch_unwind` takes the guarded call as a closure.

- `repair/package_repair_cargo_concurrent.rs`, `take_items`: each item runs inside its own catch.

### The add timing wrappers

`add_timing::work` (`worktree/add_timing.rs`) takes the timed call as `impl FnOnce`.
Each suppression sits on a function that does nothing else than call it,
so no other closure hides under the attribute.
A guard-style timer in `add_timing.rs` (start, then finish) would let these callers drop their closures;
that file belongs to the `worktree` folder.

- `repair/package_repair_cargo.rs`: `timed_read_workspace`, `timed_clone_tree`, `timed_clean`, `timed_repair_workspace`.

### Wrappers whose callee lives in another folder

- `standards_review/standards_review_client.rs`, `post_decision`: `measured_quiet` (`process/thread_quiet.rs`) takes the
  call as `impl FnOnce`. A function that returns a measuring guard would let this caller drop its closure.
- `markdown/markdown_linter_guard.rs`, `wait_for`: `WaitReporter::poll` (`push/wait_progress.rs`) takes the notice
  builder as `impl FnOnce`. A `poll` that returns whether a notice is due, with the caller building the line, would let
  this caller drop its closure.

[decision]: ../decision/rust-closure-lint-dylint.md
