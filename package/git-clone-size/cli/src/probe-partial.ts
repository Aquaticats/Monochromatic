import { join, } from 'node:path';

import {
  logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import { spawnResult, } from './spawn.ts';

/**
 Sentinel returned by {@link partialCommitCount} when tree:0 cloning is unsupported.
 */
export const NO_TREE0: unique symbol = Symbol('git-clone-size/tree-zero-partial-clone-unsupported',);

/**
 Sentinel returned by {@link partialChurn} when blobless cloning is unsupported.
 */
export const NO_CHURN: unique symbol = Symbol('git-clone-size/blobless-partial-clone-churn-unavailable',);

/**
 Commit count from a commits-only partial clone, with a lower-bound flag.
 */
export type CommitCountResult = {
  readonly count: number;
  readonly lowerBound: boolean;
};

/**
 Churn signal from a blobless partial clone: distinct historical blob/tree
 versions carrying a path, versus files at the tip. Approximate (the listing
 counts trees alongside blobs), so downstream weights it low with a wide band.
 */
export type ChurnResult = {
  readonly distinctPathObjects: number;
  readonly tipFiles: number;
};

/**
 Commit count via a `--filter=tree:0` partial clone (commits only, no trees or
 blobs), then `git rev-list --count --all`. Returns undefined when the server
 lacks partial-clone support, so the orchestrator falls back to deepen or host
 API rather than refusing.
 
 @param url - remote clone URL
 
 @param dest - temp directory the bare clone is created inside
 
 @param signal - abort signal enforcing the wall-clock budget
 
 @returns commit count, or {@link NO_TREE0} when the filtered clone is unsupported
 
 @example
 ```ts
 const n = await partialCommitCount({ url, dest: tmp.path });
 ```
 */
export async function partialCommitCount(
  {
    url,
    dest,
    signal,
  }: {
    readonly url: string;
    readonly dest: string;
    readonly signal: AbortSignal;
  },
): Promise<CommitCountResult | typeof NO_TREE0> {
  /**
   Tagged logger naming the commit-count probe.
   */
  const rl = tagged({
    tag: partialCommitCount.name,
    l: logger,
  },);

  /**
   Bare commits-only clone target.
   */
  const clonePath = join(
    dest,
    'tree0.git',
  );
  /**
   Result of the commits-only filtered clone.
   */
  const clone = await spawnResult({
    signal,
    command: 'git',
    args: [
      'clone',
      '--bare',
      '--filter=tree:0',
      url,
      clonePath,
    ],
  },);
  if (clone.exitCode !== 0) {
    rl.debug(`tree:0 partial clone unsupported or failed: ${clone.stderr}`,);
    return NO_TREE0;
  }

  /**
   Captured `rev-list --count --all` output and exit code.
   */
  const {
    stdout,
    exitCode,
  } = await spawnResult({
    command: 'git',
    args: [
      '-C',
      clonePath,
      'rev-list',
      '--count',
      '--all',
    ],
  },);
  /**
   Parsed commit count across all refs.
   */
  const count = Math.trunc(Number(stdout,),);
  if ((exitCode !== 0) || (!Number.isFinite(count,)))
    return NO_TREE0;
  rl.debug(`tree:0 commit count: ${String(count,)}`,);
  return {
    count,
    lowerBound: false,
  };
}

/**
 Counts entries with a path in `git rev-list --objects --all` output (blobs and
 trees across history), as a churn proxy.
 
 @param clonePath - bare blobless clone directory
 
 @returns number of object lines carrying a path
 */
async function countPathObjects({ clonePath, }: { readonly clonePath: string; },): Promise<number> {
  /**
   Captured `rev-list --objects --all` listing and exit code.
   */
  const {
    stdout,
    exitCode,
  } = await spawnResult({
    command: 'git',
    args: [
      '-C',
      clonePath,
      'rev-list',
      '--objects',
      '--all',
    ],
  },);
  if ((exitCode !== 0) || (stdout === ''))
    return 0;
  return stdout
    .split('\n',)
    .filter(function hasPath(line,) {
      return line.includes(' ',);
    },)
    .length;
}

/**
 Churn probe via a `--filter=blob:none` partial clone (commits + trees, no blob
 content). Counts distinct historical path-bearing objects against tip file
 count. Blob sizes stay unknown without download, so callers combine this with
 the tip size. Returns undefined when blobless cloning is unsupported.
 
 @param url - remote clone URL
 
 @param dest - temp directory the bare clone is created inside
 
 @param signal - abort signal enforcing the wall-clock budget
 
 @returns churn counts, or {@link NO_CHURN} when unsupported
 
 @example
 ```ts
 const churn = await partialChurn({ url, dest: tmp.path });
 ```
 */
export async function partialChurn(
  {
    url,
    dest,
    signal,
  }: {
    readonly url: string;
    readonly dest: string;
    readonly signal: AbortSignal;
  },
): Promise<ChurnResult | typeof NO_CHURN> {
  /**
   Tagged logger naming the churn probe.
   */
  const rl = tagged({
    tag: partialChurn.name,
    l: logger,
  },);

  /**
   Bare blobless clone target.
   */
  const clonePath = join(
    dest,
    'blobless.git',
  );
  /**
   Result of the blobless filtered clone.
   */
  const clone = await spawnResult({
    signal,
    command: 'git',
    args: [
      'clone',
      '--bare',
      '--filter=blob:none',
      url,
      clonePath,
    ],
  },);
  if (clone.exitCode !== 0) {
    rl.debug(`blob:none partial clone unsupported or failed: ${clone.stderr}`,);
    return NO_CHURN;
  }

  /**
   Distinct path-bearing objects across all history.
   */
  const distinctPathObjects = await countPathObjects({ clonePath, },);

  /**
   Files present at the tip, the churn denominator.
   */
  const tip = await spawnResult({
    command: 'git',
    args: [
      '-C',
      clonePath,
      'ls-tree',
      '-r',
      '--name-only',
      'HEAD',
    ],
  },);
  /**
   Tip file count; at least 1 to avoid a zero denominator.
   */
  const tipFiles = (tip.exitCode === 0) && (tip.stdout !== '') ? tip.stdout
    .split('\n',)
    .length : 1;

  rl.debug(`churn: ${String(distinctPathObjects,)} path objects over ${String(tipFiles,)} tip files`,);
  return {
    distinctPathObjects,
    tipFiles,
  };
}
