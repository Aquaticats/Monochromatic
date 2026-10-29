import { join, } from 'node:path';

import {
  logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import { isMeasured, } from './measure.ts';
import { objectsDirSize, } from './objects-size.ts';
import { spawnResult, } from './spawn.ts';

/**
 Outcome of the depth-1 shallow clone probe. `shallowBytes` is present only
 when `ok`; a failed clone or an unreadable object store leaves it absent
 rather than recording a fabricated zero.
 */
export type ShallowResult = {
  readonly shallowBytes?: number;
  readonly clonePath: string;
  readonly ok: boolean;
};

/**
 Performs a depth-1 bare clone of the default branch into `dest` and measures
 its object store, yielding the compressed tip size `C1`. `--depth 1` implies
 `--single-branch`, matching the default-metric shallow side. The clone path is
 returned so the deepen probe can extend the same store without re-cloning.
 
 @param url - remote clone URL
 
 @param dest - temp directory the bare clone is created inside
 
 @param signal - abort signal enforcing the wall-clock budget
 
 @returns shallow object-store bytes, the clone path, and an `ok` flag
 
 @example
 ```ts
 const shallow = await cloneShallow({ url, dest: tmp.path });
 ```
 */
export async function cloneShallow(
  {
    url,
    dest,
    signal,
  }: {
    readonly url: string;
    readonly dest: string;
    readonly signal: AbortSignal;
  },
): Promise<ShallowResult> {
  /**
   Tagged logger naming the shallow probe.
   */
  const rl = tagged({
    tag: cloneShallow.name,
    l: logger,
  },);

  /**
   Bare clone target inside the disposable temp directory.
   */
  const clonePath = join(
    dest,
    'shallow.git',
  );

  /**
   Exit code and stderr from the depth-1 bare clone.
   */
  const {
    exitCode,
    stderr,
  } = await spawnResult({
    signal,
    command: 'git',
    args: [
      'clone',
      '--bare',
      '--depth',
      '1',
      url,
      clonePath,
    ],
  },);
  if (exitCode !== 0) {
    rl.debug(`shallow clone failed: ${stderr}`,);
    return {
      clonePath,
      ok: false,
    };
  }

  /**
   Compressed tip object-store size after the depth-1 clone.
   */
  const shallowBytes = await objectsDirSize({ repoPath: clonePath, },);
  if (!isMeasured(shallowBytes,)) {
    rl.debug('shallow tip object store unmeasured',);
    return {
      clonePath,
      ok: false,
    };
  }
  rl.debug(`shallow tip object store: ${String(shallowBytes,)} bytes`,);
  return {
    shallowBytes,
    clonePath,
    ok: true,
  };
}
