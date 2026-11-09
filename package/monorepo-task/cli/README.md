# Monorepo task CLI

Runs the repository's existing task definitions in CI,
where Mise is not installed.
The task-system replacement remains a separate proposal in
[`doc/planning/task-slice-pants-migration.md`](../../../doc/planning/task-slice-pants-migration.md).

## Execution

Use `task-run //package/<category>/<name>:<task>` or a root task name.
The source entry used by GitHub Actions is `src/run.ts`;
the published command is `dist/final/node/run.mjs`.

## Package entries

`package.json` declares three entries alongside the `task-run` command.

- `.` is the library: the task shapes and loader helpers from `src/index.ts`,
  built to `dist/final/node/index.mjs`,
  so importing the package name loads no command line.
- `./ts` is the TypeScript source of the same library,
  the subpath workspace packages import
  ([workspace TypeScript source imports](../../../doc/decision/workspace-ts-source-imports.md)).
- `./ts/*` is any other module under `src/`.

`src/run.ts` stays the command line: it runs `main()` when it loads,
 so nothing re-exports it and an import of the package name never dispatches a task.
`bin` names the built `dist/final/node/run.mjs`,
 which keeps the `#!/usr/bin/env node` shebang.

Every `task.ts` file in this repository,
 including the root one,
 keeps importing `src/types.ts` and `src/load.ts` by relative path.
The CLI loads those declarations before any dependency install:
CI jobs bootstrap from a filtered `pnpm install`,
 so a package-name import there would need a workspace link those jobs never create.
See `.github/actions/task-cli-setup`,
 whose `install-filter` input exists because a task body importing a workspace package by name
 stopped with `ERR_MODULE_NOT_FOUND` when the link was missing.

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

During the pnpr upgrade at merge `1f6a6f8dc8`,
scoped lint reported no warnings or errors,
type checking passed,
and the named consumer suites passed.
Removing the loader's validation calls and rebuilding made all 47 rejection cases fail;
restoring the committed source and rebuilding made the suite pass.
An initial malformed ablation edit failed to compile and was excluded from that evidence.

The consumer suite exceeded an agent-command limit of 128 tasks:
the cgroup peaked at 128 and recorded 16 denied process/thread creations.
The unchanged suite passed with an agent-command limit of 512,
peaked at 193,
and recorded no denials.
These are local verification limits,
not shipped task-runner settings.

The readonly-analysis bridge reported `project-not-found` for the package's `task.ts`;
that file was outside its configured TypeScript project.
Runtime validation separately loaded the root templates,
root tasks,
and every package task file.
The CLI tests supply the repository root explicitly;
they do not establish default loader-path behavior for a separately installed library artifact.

On 2026-11-09 the exports-map tests (`src/package-exports.unit.test.ts`) imported the package root,
`./ts`,
and `./ts/*`,
loaded this package's `task.ts` through the root entry,
and imported the root in a child process with no arguments.
Deleting the exports map made the test file fail with `ERR_MODULE_NOT_FOUND`;
pointing `.` at `src/run.ts` made the same import throw
`task-cli: usage: task-run <task> [-- options] [-- passthrough]`.
Restoring the map and rebuilding made all four cases pass,
and a throwaway consumer outside the repository imported every subpath by package name
and executed the installed `task-run` command.

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
