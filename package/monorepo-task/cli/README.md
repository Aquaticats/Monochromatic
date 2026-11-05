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
- `src/task-arguments.ts` interprets the existing usage declarations.
- `src/task-script-prelude.ts` provides helpers for legacy Node-inline bodies.
- `src/task-execution.ts` runs dependency and command steps.
- `src/task-dispatch.ts` selects package and root tasks.
- `src/run.ts` owns a CLI invocation and its terminal progress output.

## Verification

The graph regression tests execute the source consumer configured by GitHub Actions in disposable repositories.
The invocation tests import the built library artifact.
Local tasks are declared in `mise.toml`;
the matching CI tasks are in `task.ts`.
