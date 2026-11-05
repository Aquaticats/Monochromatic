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

import { createTaskInvocation, } from './task-invocation.ts';
import { findRepoRoot, } from './task-locations.ts';
import { dispatchTask, } from './task-dispatch.ts';

/**
 Runs one CLI invocation with one shared task completion cache.
 */
async function main(): Promise<void> {
  const [name, ...argv] = process.argv
    .slice(2);
  if (name === undefined)
    throw new Error('task-cli: usage: task-run <task> [-- options] [-- passthrough]');
  const root = findRepoRoot();
  const environment = { ...process.env };
  const runTask = createTaskInvocation({
    execute: async context => { await dispatchTask({
      root,
      environment,
      ...context
    }); },
    report: ({
      name,
      state,
    }) => { console.error(`task-run [${state}] ${JSON.stringify(name)}`); },
  });
  await runTask({
    name,
    argv
  });
}

await main();
