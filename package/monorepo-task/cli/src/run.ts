#!/usr/bin/env node

/**
 Mise-free task CLI for this monorepo's CI.

 Runs the same task graph CI used to run through `mise run`, but resolves every
 body from `task.ts` files and executes it with the current node, so CI no
 longer installs or invokes mise. Task bodies keep calling workspace tool
 binaries by bare name because `PATH` carries every `node_modules/.bin` up to
 the repository root, the way `mise.toml` `_.path` did. Supports plain `run`
 strings, `run` arrays (sequenced, with `{ shell }` items), `depends`, `extends`
 onto root templates, `usage` options mapped to `usage_*` variables like mise,
 root `build`/`test`/`lint` umbrella tasks, root inline tasks, and the
 `//package/...:task` form. Tasks without a `usage` spec get post-`--`
 arguments appended to their command as a real argv vector, the way mise did,
 and `usage_args` round-trips words that contain spaces.

 @example
 ```bash
 node package/monorepo-task/cli/src/run.ts //package/logger/core:build
 node package/monorepo-task/cli/src/run.ts //package/logger/core:test:unit -- --all
 node package/monorepo-task/cli/src/run.ts prepare:pnpm
 ```
 */

import {
  createTaskInvocation,
  type TaskEvent,
} from './task-invocation.ts';
import { findRepoRoot, } from './task-locations.ts';
import { dispatchTask, } from './task-dispatch.ts';

/**
 Writes task lifecycle events as exact terminal output without exposing inputs.

 @param event - Task identity and state emitted by the invocation.
 */
function reportTask(event: TaskEvent,): void {
  console.error(`task-run [${event.state}] ${JSON.stringify(event.name)}`);
}

/**
 CLI arguments remain literal words until a task's usage declaration interprets them.
 */
const [taskName, ...argv] = process.argv
  .slice(2);
if (taskName === undefined)
  throw new Error('task-cli: usage: task-run <task> [-- options] [-- passthrough]');
/**
 Repository root fixes the search boundary for this invocation.
 */
const root = await findRepoRoot();
/**
 Child requests inherit a snapshot rather than mutable global process state.
 */
const environment = { ...process.env };
/**
 Shared completion cache belongs to this one command invocation.
 */
const runTask = createTaskInvocation({
  execute: async function execute(context) { await dispatchTask({
    root,
    environment,
    ...context
  }); },
  report: reportTask,
});
await runTask({
  name: taskName,
  argv
});
