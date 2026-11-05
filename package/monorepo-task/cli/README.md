# Monorepo task CLI

Runs the repository's existing task definitions in CI,
where Mise is not installed.
The task-system replacement remains a separate proposal in
[`doc/planning/task-slice-pants-migration.md`](../../../doc/planning/task-slice-pants-migration.md).

## Execution

Use `task-run //package/<category>/<name>:<task>` or a root task name.
The source entry used by GitHub Actions is `src/run.ts`;
the published command is `dist/final/node/run.mjs`.

A CLI invocation shares each task's completion or failure for the same task name,
arguments,
and explicit inherited environment.
A separate CLI invocation starts fresh.
Dependencies run before body steps.
Template children run concurrently through the same invocation,
so shared prerequisites execute once.
Pending dependency cycles fail with their task path.
A completed task cannot start more dependencies through its old callback.

Task function bodies receive their working directory,
environment,
arguments,
and `runChildren` callback as explicit context.
They must await their work.
They do not change the process's global working directory or environment.
Shell and Node-inline command bodies retain their existing subprocess behavior.

## Files

- `src/types.ts` defines task declarations and direct async task context.
- `src/load.ts` loads definitions and merges inherited templates.
- `src/task-invocation.ts` owns shared completion and pending dependency checks.
- `src/task-locations.ts` locates workspace and package task files.
- `src/usage-declarations.ts` scans quoted flag and argument declarations.
- `src/task-arguments.ts` maps argument words into the usage environment.
- `src/task-script-prelude.ts` provides helpers for legacy Node-inline bodies.
- `src/task-process.ts` awaits commands with inherited terminal output.
- `src/task-execution.ts` runs dependency and command steps.
- `src/task-dispatch.ts` selects package and root tasks.
- `src/run.ts` owns a CLI invocation and its terminal progress output.

## Verification

The graph regression tests execute both the source consumer configured by GitHub Actions
and the built executable in disposable repositories.
They cover shared dependency execution,
root and package dispatch,
literal shell arguments,
declared options,
and explicit function-body context.
The invocation tests import the built library artifact.
Local tasks are declared in `mise.toml`;
the matching CI tasks are in `task.ts`.

## Ordered task execution

Scoped `eslint/no-await-in-loop` exceptions retain required ordering:
the repository search stops at the nearest workspace marker,
root fanout loads definitions in discovery order and waits within each bounded worker,
and task dependencies and body steps execute in their declared order.
Replacing these awaits with unrestricted `Promise.all` would change those contracts.
Independent template children and filesystem checks already run concurrently.

Oxlint `1.90.0` implements this rule as a configuration-free `NoAwaitInLoop` marker
and reports any await found inside a loop body.
It has no option for ordered dependencies or bounded worker loops:
[rule declaration and implementation][await-rule].
The repository enables it as a warning in `package/config/oxlint/src/rule/correctness.ts`;
`package/config/oxlint/src/config-base.ts` makes warnings fail the lint gate.
The exceptions stay on the functions whose control flow needs sequencing.

[await-rule]: https://github.com/oxc-project/oxc/blob/9b6fff42fa6edf0bec55861b6f0764c7b9acd86b/crates/oxc_linter/src/rules/eslint/no_await_in_loop.rs#L17-L107
