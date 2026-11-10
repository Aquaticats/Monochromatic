/**
 Checks that every workflow whose pull-request run is path-filtered gates its merge-queue run on a diff of the same
 paths.

 A `merge_group` trigger cannot be path-filtered (a GitHub limitation), so a workflow that lists paths on
 `pull_request` would otherwise install, build and test on every merge-queue entry, including the ones that touch none
 of its packages. The remedy the repository already uses is a Scope step that compares the queued diff and sets
 `run=false`, with every heavy step reading `steps.scope.outputs.run`.

 Workflows that list no paths on `pull_request` are exempt: their merge-queue run costs what their pull-request run
 costs. The files are read as text, because a workflow is checked exactly as the runner reads it, and the check names
 the file and the missing step rather than a parsed structure.
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

//region Reading a workflow's triggers

/**
 How many spaces start a line.
 */
function indentOf(line: string,): number {
  return line.length - line.trimStart()
    .length;
}

/**
 The lines below a key, ending at the first non-blank line indented less than the key's own children.

 @param options - lines of the file or block, index of the key's line, and whether the key line itself holds the block

 @returns the block's lines, without the key line

 @example
 ```ts
 children({ lines: ['on:', '  push:'], keyAt: 0 });
 ```
 */
function children({ lines, keyAt, }: {
  readonly lines: readonly string[];
  readonly keyAt: number;
},): readonly string[] {
  const below = lines.slice(keyAt + 1,);
  const end = below.findIndex(function isOutside(line,) {
    return (line.trim() !== '') && (indentOf(line,) <= indentOf(lines[keyAt] ?? '',));
  },);
  return end === (-1) ? below : below.slice(0, end,);
}

/**
 The `on:` section of a workflow, which holds one block per trigger event.

 @param lines - lines of the workflow file

 @returns the section's lines, empty for a file without an `on:` key
 */
function onSection(lines: readonly string[],): readonly string[] {
  const keyAt = lines.indexOf('on:',);
  return keyAt === (-1) ? [] : children({ lines, keyAt, },);
}

/**
 The block of one trigger event inside the `on:` section.

 @param section - lines of the `on:` section

 @param event - trigger name, such as `pull_request`

 @returns the event's lines, empty when the workflow has no such trigger
 */
function triggerBlock({ section, event, }: {
  readonly section: readonly string[];
  readonly event: string;
},): readonly string[] {
  const keyAt = section.findIndex(function isEvent(line,) {
    return line.trim() === `${event}:`;
  },);
  return keyAt === (-1) ? [] : children({ lines: section, keyAt, },);
}

/**
 Whether a trigger block lists the paths it runs for, either way of spelling the key.

 @param block - lines of one trigger's block

 @returns true when the workflow runs only for the listed paths
 */
function listsPaths(block: readonly string[],): boolean {
  return block.some(function isPathsKey(line,) {
    const trimmed = line.trim();
    return trimmed.startsWith('paths:') || trimmed.startsWith('paths-ignore:');
  },);
}

/**
 The `run` output the Scope step publishes, as a step condition reads it.
 */
const scopeGate = 'steps.scope.outputs.run';

/**
 One workflow file: its repository-relative path and its lines.
 */
type Workflow = {
  readonly path: string;
  readonly lines: readonly string[];
};

/**
 The repository root, which the task runner does not leave as the working directory: a test file runs from its package
 folder, so a glob without this would match nothing and the control below would be the only thing that noticed.
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

//endregion

//region The workflows the check covers

/**
 Every workflow that triggers on a merge queue.
 */
const mergeGroupWorkflows = workflows.filter(function hasMergeGroup(workflow,) {
  return triggerBlock({ section: onSection(workflow.lines,), event: 'merge_group', },).length > 0;
},);

/**
 The merge-queue workflows whose pull-request run is path-filtered, so a queue entry that touches none of the paths
 would still pay the whole run without a Scope step.
 */
const pathFilteredMergeGroupWorkflows = mergeGroupWorkflows.filter(function isPathFiltered(workflow,) {
  return listsPaths(triggerBlock({ section: onSection(workflow.lines,), event: 'pull_request', },),);
},);

//endregion

await describe({
  name: 'merge-queue workflows',
  children: [
    it({
      name: 'are found in this repository, so an empty selection cannot pass the check below',
      fn: async function hasWorkflows() {
        expect(mergeGroupWorkflows.length,).toBeGreaterThan(0,);
        expect(pathFilteredMergeGroupWorkflows.length,).toBeGreaterThan(0,);
        expect(mergeGroupWorkflows.length,).toBeGreaterThanOrEqual(pathFilteredMergeGroupWorkflows.length,);
      },
    },),
    it({
      name: 'gate a path-filtered run on a diff of the same paths, so a queue entry that touches none of them skips the heavy steps',
      fn: async function gatesEveryQueueRun() {
        const ungated = pathFilteredMergeGroupWorkflows.filter(function hasNoScopeStep(workflow,) {
          return !workflow.lines.some(function isScopeStep(line,) {
            return line.trim() === 'id: scope';
          },);
        },)
          .map(function name(workflow,) {
            return `${workflow.path}: no Scope step`;
          },);
        const readsGate = pathFilteredMergeGroupWorkflows.filter(function hasNoGatedStep(workflow,) {
          return !workflow.lines.join('\n',)
            .includes(scopeGate,);
        },)
          .map(function name(workflow,) {
            return `${workflow.path}: no step reads ${scopeGate}`;
          },);
        expect(ungated,).toEqual([],);
        expect(readsGate,).toEqual([],);
      },
    },),
  ],
},);
