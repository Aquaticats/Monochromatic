/**
 Node programs the container suite registers as config hooks and installs as editors.

 Hooks and editors are Node programs rather than shell scripts (AGENTS.md SCR).
 Every hook appends one JSON line per run to `hooks.jsonl`,
 writes a `<token>.<event>` marker,
 and blocks while a `<token>.<event>.hold` file exists until `<token>.<event>.release` appears,
 which lets scenarios pin interleavings and inject faults at named phases.

 @module
 */

import {
  chmod,
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

//region Programs

/**
 Shared hook program.
 Arguments:
 hook event name,
 then Git's hook arguments.
 `E2E_HOOK_MODE=lint-staged` adds lint-staged's backup-stash and hide-unstaged sequence to `pre-commit`.
 */
export const HOOK_PROGRAM: string = String.raw`#!/usr/bin/env node
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const [event = 'unknown', ...args] = process.argv.slice(2);
const token = process.env.E2E_TOKEN || 'untokened';
const markers = process.env.E2E_MARKER_DIR;
const logs = process.env.E2E_LOG_DIR;
const pause = (ms) => Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
fs.appendFileSync(path.join(logs, 'hooks.jsonl'), JSON.stringify({
  event, token, pid: process.pid,
  indexFile: process.env.GIT_INDEX_FILE || '', workTree: process.env.GIT_WORK_TREE || '', cwd: process.cwd(),
}) + '\n');
fs.writeFileSync(path.join(markers, token + '.' + event), '');
if (fs.existsSync(path.join(markers, token + '.' + event + '.hold'))) {
  const release = path.join(markers, token + '.' + event + '.release');
  const deadline = Date.now() + 120000;
  while (!fs.existsSync(release)) {
    if (Date.now() > deadline) { process.stderr.write('e2e hook barrier timed out\n'); process.exit(1); }
    pause(10);
  }
}
if (event === 'commit-msg') fs.appendFileSync(args[0], '\nE2E-Hook-Checked: yes\n');
if (event === 'pre-commit' && process.env.E2E_HOOK_MODE === 'lint-staged') {
  const git = (gitArgs) => execFileSync('git', gitArgs, { encoding: 'utf8' });
  const split = (text) => text.split('\0').filter((entry) => entry !== '');
  const backup = git(['stash', 'create']).trim();
  if (backup !== '') git(['stash', 'store', '--quiet', '--message', 'lint-staged automatic backup', backup]);
  const staged = split(git(['diff', '--staged', '--name-only', '-z', '--diff-filter=ACMR']));
  const unstaged = new Set(split(git(['diff', '--name-only', '-z'])));
  const partial = staged.filter((file) => unstaged.has(file));
  const patch = path.join(logs, 'lint-staged-' + process.pid + '.patch');
  if (partial.length > 0) {
    git(['diff', '--binary', '--unified=0', '--no-color', '--no-ext-diff', '--src-prefix=a/', '--dst-prefix=b/',
      '--patch', '--submodule=short', '--output', patch, '--', ...partial]);
    git(['checkout', '--force', '--', ...partial]);
  }
  for (const file of staged) fs.readFileSync(file);
  if (partial.length > 0) git(['apply', '-v', '--whitespace=nowarn', '--recount', '--unidiff-zero', patch]);
  if (backup !== '') {
    const entry = git(['stash', 'list', '--format=%gd %gs']).split('\n')
      .find((line) => line.includes('lint-staged automatic backup'));
    if (entry !== undefined) git(['stash', 'drop', '--quiet', entry.split(' ')[0]]);
  }
}
`;

/**
 Editor that holds the message file open until released,
 so a real-Git `commit` keeps `index.lock` for as long as the scenario needs.
 */
export const HOLDING_EDITOR_PROGRAM: string = String.raw`#!/usr/bin/env node
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const token = process.env.E2E_TOKEN || 'untokened';
const markers = process.env.E2E_MARKER_DIR;
fs.writeFileSync(path.join(markers, token + '.editor'), '');
const release = path.join(markers, token + '.editor.release');
const deadline = Date.now() + Number(process.env.E2E_EDITOR_MAX_MS || '60000');
while (!fs.existsSync(release) && Date.now() < deadline) Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 10);
fs.writeFileSync(process.argv[2], 'e2e editor commit [' + token + ']\n');
`;

//endregion Programs

//region Installation

/**
 Hook events the suite can install.
 */
export type HookEvent = 'commit-msg' | 'post-commit' | 'pre-commit' | 'prepare-commit-msg';

/**
 Executable mode for installed programs.
 */
const EXECUTABLE_MODE = 0o755;

/**
 Writes the shared programs into a tools directory.

 @param toolsDir - absolute directory

 @returns absolute program paths

 @example
 ```ts
 await installPrograms('/work/s1/tools');
 ```
 */
export async function installPrograms(toolsDir: string,): Promise<Readonly<{
  hook: string;
  editor: string;
}>> {
  await mkdir(
    toolsDir,
    { recursive: true, },
  );
  /**
   Shared hook program path.
   */
  const hook = join(
    toolsDir,
    'e2e-hook.cjs',
  );
  /**
   Holding editor path.
   */
  const editor = join(
    toolsDir,
    'holding-editor.cjs',
  );
  await writeFile(
    hook,
    HOOK_PROGRAM,
    { mode: EXECUTABLE_MODE, },
  );
  await writeFile(
    editor,
    HOLDING_EDITOR_PROGRAM,
    { mode: EXECUTABLE_MODE, },
  );
  await chmod(
    hook,
    EXECUTABLE_MODE,
  );
  await chmod(
    editor,
    EXECUTABLE_MODE,
  );
  return {
    hook,
    editor,
  };
}

/**
 Config section name of the hook registered for an event.

 @param event - hook event

 @returns friendly name shared by `hook.<name>.command`,
 `hook.<name>.event`,
 and `hook.<name>.enabled`

 @example
 ```ts
 configHookName('pre-commit');
 ```
 */
export function configHookName(event: HookEvent,): string {
  return `e2e-${event}`;
}

/**
 Git `-c` arguments that switch the suite's config hooks off for one real-Git run,
 so fixture setup commits do not run workload hooks.
 `core.hooksPath=/dev/null` does not do this:
 it only silences traditional hooks,
 which the wrapper does not support
 (`doc/troubleshooting/git-hook-disable-switches.md`).

 @param events - events whose hooks are switched off

 @returns `-c` argument pairs to put before the Git subcommand

 @example
 ```ts
 hookDisableArguments(['pre-commit']);
 ```
 */
export function hookDisableArguments(events: readonly HookEvent[],): readonly string[] {
  return events.flatMap(function disable(event,) {
    return [
      '-c',
      `hook.${configHookName(event,)}.enabled=false`,
    ];
  },);
}

/**
 Git config arguments registering config-based hooks (Git 2.54.0 or newer).

 @param hookProgram - shared program path

 @param events - events to register

 @returns `config` argument vectors,
 one per key

 @example
 ```ts
 configHookArguments({ hookProgram, events: ['pre-commit'] });
 ```
 */
export function configHookArguments({
  hookProgram,
  events,
}: Readonly<{
  hookProgram: string;
  events: readonly HookEvent[];
}>,): readonly (readonly string[])[] {
  return events.flatMap(function hookKeys(event,) {
    /**
     Config section name for this hook.
     */
    const name = configHookName(event,);
    return [
      [
        'config',
        `hook.${name}.command`,
        `${hookProgram} ${event}`,
      ],
      [
        'config',
        '--add',
        `hook.${name}.event`,
        event,
      ],
    ];
  },);
}

//endregion Installation
