/**
 Checks that no workflow overrides the Node or pnpm version the repository locks.

 `.github/actions/task-cli-setup` installs the versions listed in `toolchain.lock.json` whenever its `node-version` and
 `pnpm-version` inputs are empty, which is the documented way to read the lock. A workflow that passes a value runs a
 runtime no developer, no release job and no other workflow uses, and the value drifts away from the lock silently: the
 pinned Node of this repository has moved on while a workflow kept a hand-written version.

 The check reports the file and the line it fails on, so a job that needs another runtime on purpose can replace it with
 the reason it needs one.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  glob,
  readFile,
} from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

//region Reading the workflows

/**
 One workflow file: its repository-relative path and its lines.
 */
type Workflow = {
  readonly path: string;
  readonly lines: readonly string[];
};

/**
 The repository root, which the task runner does not leave as the working directory: a test file runs from its package
 folder, so a glob without this would match nothing and every override would go unseen.
 */
const repositoryRoot = fileURLToPath(new URL('../../../../', import.meta.url,),);

/**
 Repository-relative paths of every workflow file, in sorted order so a message does not depend on directory order.
 */
const workflowPaths = (await Array.fromAsync(glob('.github/workflows/*.yml', { cwd: repositoryRoot, },),)).toSorted();

/**
 Every workflow of the repository.
 */
const workflows: readonly Workflow[] = await Promise.all(
  workflowPaths.map(async function read(path,): Promise<Workflow> {
    return {
      path,
      lines: (await readFile(new URL(`../../../../${path}`, import.meta.url,), 'utf8',)).split('\n',),
    };
  },),
);

/**
 The action whose inputs decide which Node and pnpm a job installs.
 */
const setupAction = await readFile(new URL('../../../../.github/actions/task-cli-setup/action.yml', import.meta.url,), 'utf8',);

/**
 Version inputs of the setup action that a workflow may pass to override the lock, with the lock row each one falls back
 to when it is empty.
 */
const overridableInputs: readonly { readonly input: string; readonly lockRow: string }[] = [
  { input: 'node-version', lockRow: '.tools.node.version' },
  { input: 'pnpm-version', lockRow: '.tools.pnpm.version' },
];

//endregion

await describe({
  name: 'workflow toolchain versions',
  children: [
    it({
      name: 'finds the workflows it reads, so an empty selection cannot pass the checks below',
      fn: async function findsItsSubjects() {
        expect(workflowPaths.length,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'leave every version input of the setup action empty, so each job installs the toolchain the lock records',
      fn: async function noOverrides() {
        const overrides = workflows.flatMap(function overridesOf(workflow,) {
          return workflow.lines.flatMap(function inputsOf(line, index,) {
            const trimmed = line.trim();
            const passesInput = overridableInputs.some(function isVersionInput({ input, },) {
              return trimmed.startsWith(`${input}:`);
            },);
            return passesInput ? [`${workflow.path}:${index + 1}: ${trimmed}`] : [];
          },);
        },);
        expect(overrides,).toEqual([],);
      },
    },),
    it({
      name: 'take both versions from toolchain.lock.json when a job passes neither, so emptying an input cannot leave a guess in place',
      fn: async function defaultsReadTheLock() {
        for (const { lockRow } of overridableInputs) expect(setupAction,).toContain(lockRow,);
      },
    },),
  ],
},);
