import { join } from 'node:path';

import {
  buildBranchHeadArguments,
  buildCompareArguments,
  buildTreeIdsRequest,
  type BranchComparison,
  movedForward,
  parseBranchComparison,
  parseBranchHead,
  parseChangedTreePaths,
} from './branch-api.ts';
import { ghProgram } from './constants.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import type { PushOptions } from './push-options.ts';
import type {
  ProgramCall,
  ProgramResult,
} from './run-program.ts';
import { buildGraphqlInputArguments } from './service-arguments.ts';
import {
  changedWatchedPaths,
  type WatchSet,
} from './watch-paths.ts';

/**
 Runs one program call to completion and reports how it ended.
 */
export type DecisionRunner = (call: ProgramCall) => Promise<ProgramResult>;

/**
 Name of the tree id request file inside the scratch directory.
 */
export const treeIdsRequestFileName = 'crate-release-tree-ids-request.json';

/**
 Describe why a failed `gh` call has no usable answer.

 @param result - finished call with a non-zero status

 @returns the call's error text, or its exit status when it printed none

 @example
 ```ts
 failureText({ status: 1, stdout: '', stderr: 'HTTP 404' });
 ```
 */
function failureText(result: ProgramResult): string {
  return result.stderr
    .trim()
    === '' ? `gh exited with status ${String(result.status)}` : result.stderr
      .trim();
}

/**
 Find, by the object names of the watched paths, which of them differ between two commits.

 Used when the compare endpoint's list reached its limit and may have been cut: an object name covers everything below
 a directory, so equal names prove a path unchanged whatever else changed on the branch.

 @param options - validated push options

 @param run - runs a program call and reports its status and output

 @param writeText - writes a file by absolute path

 @param base - full object name of the commit the checkout started from

 @param head - full object name of the commit the branch points to now

 @param watch - the paths the release depends on

 @returns the watched paths whose names differ, or a miss when the answer cannot be read

 @example
 ```ts
 await changedByTreeIds({ options, run, writeText, base, head, watch });
 ```
 */
async function changedByTreeIds({
  options,
  run,
  writeText,
  base,
  head,
  watch,
}: {
  readonly options: PushOptions;
  readonly run: DecisionRunner;
  readonly writeText: (input: {
    readonly path: string;
    readonly text: string
  }) => Promise<void>;
  readonly base: string;
  readonly head: string;
  readonly watch: WatchSet;
}): Promise<Lookup<readonly string[]>> {
  /**
   Every watched path, files then directories.
   */
  const paths = [...watch.files, ...watch.directories];
  /**
   Absolute path of the request file.
   */
  const requestFile = join(
    options.scratchDirectory,
    treeIdsRequestFileName
  );
  await writeText({
    path: requestFile,
    text: buildTreeIdsRequest({
      repo: options.repo,
      base,
      head,
      paths
    }),
  });
  /**
   How `gh` ended.
   */
  const result = await run({
    program: ghProgram,
    args: buildGraphqlInputArguments(requestFile),
    cwd: options.root
  });
  /**
   The paths whose names differ, or why that is unknown.
   */
  const changed = parseChangedTreePaths({
    body: result.stdout,
    paths
  });
  return changed.found || (result.status === 0) ? changed : missingValue(failureText(result));
}

/**
 Decide whether a release commit may be rebuilt on the branch's new head, after the branch moved during the release.

 The release verified the crate at the commit the checkout started from. The decision is yes only when nothing the
 release depends on changed between that commit and the new head, so what was verified is what the head holds. The new
 head is read, then compared with the base through the compare endpoint; when that list reached its limit and may be
 cut, the object names of the watched paths decide instead. Every step is printed: the base, the new head, the compared
 paths and the decision.

 @param options - validated push options

 @param run - runs a program call and reports its status and output

 @param writeText - writes a file by absolute path

 @param base - full object name of the commit the checkout started from

 @param watch - the paths the release depends on

 @param print - prints one line

 @returns the head to send the commit on again, or a miss that says why the push is refused

 @example
 ```ts
 await decideAfterStaleHead({ options, run, writeText, base, watch, print: console.log });
 ```
 */
export async function decideAfterStaleHead({
  options,
  run,
  writeText,
  base,
  watch,
  print,
}: {
  readonly options: PushOptions;
  readonly run: DecisionRunner;
  readonly writeText: (input: {
    readonly path: string;
    readonly text: string
  }) => Promise<void>;
  readonly base: string;
  readonly watch: WatchSet;
  readonly print: (line: string) => void;
}): Promise<Lookup<string>> {
  /**
   How `gh` ended when asked for the head.
   */
  const headResult = await run({
    program: ghProgram,
    args: buildBranchHeadArguments({
      repo: options.repo,
      branch: options.branch
    }),
    cwd: options.root,
  });
  /**
   The new head, or why it is unknown.
   */
  const head = headResult.status === 0 ? parseBranchHead(headResult.stdout) : missingValue(failureText(headResult));
  if (!head.found)
    return missingValue(`The new head of ${options.branch} cannot be read: ${head.reason}`);
  print(`${options.branch} now points to ${head.value}; the checkout started from ${base}`);
  if (head.value === base) {
    print('decision: the head is the checkout base, so nothing changed; send the commit again on it');
    return foundValue(head.value);
  }
  /**
   How `gh` ended when asked for the comparison.
   */
  const compareResult = await run({
    program: ghProgram,
    args: buildCompareArguments({
      repo: options.repo,
      base,
      head: head.value
    }),
    cwd: options.root,
  });
  /**
   The comparison, or why it is unknown.
   */
  const comparison: Lookup<BranchComparison> = compareResult.status === 0
    ? parseBranchComparison(compareResult.stdout)
    : missingValue(failureText(compareResult));
  if (!comparison.found)
    return missingValue(`${base}...${head.value} cannot be compared: ${comparison.reason}`);
  print(
    `compare ${base}...${head.value}: ${comparison.value.status}, ${String(comparison.value.changedPaths.length)} changed files${
      comparison.value.truncated ? ', a list as long as the endpoint ever gives, which may be cut' : ''
    }`,
  );
  if (!movedForward(comparison.value))
    return missingValue(
      `${options.branch} is ${comparison.value.status} relative to the checkout, not ahead of it, so a comparison cannot show what changed`,
    );
  /**
   The watched paths that changed, or why that is unknown.
   */
  const changed = comparison.value.truncated
    ? await changedByTreeIds({
      options,
      run,
      writeText,
      base,
      head: head.value,
      watch
    })
    : foundValue(changedWatchedPaths({
      changed: comparison.value.changedPaths,
      watch
    }));
  if (!changed.found)
    return missingValue(`The watched paths cannot be compared: ${changed.reason}`);
  if (comparison.value.truncated)
    print(`the object names of ${String(watch.files.length + watch.directories.length)} watched paths decided`);
  if (changed.value.length > 0)
    return missingValue(
      `Paths the release depends on changed on ${options.branch} since ${base}: ${changed.value.join(', ')}`,
    );
  print(`decision: none of the watched paths changed; send the commit again on ${head.value}`);
  return foundValue(head.value);
}
