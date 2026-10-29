/**
 Tests the progressive generator on the local exact path: a coarse first
 snapshot, then a final very-high-confidence snapshot.
 
 @module
 */

import { execFileSync, } from 'node:child_process';
import {
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  DEFAULT_MAX_DEEPEN_COMMITS,
  DEFAULT_MAX_PACK_BYTES,
  DEFAULT_MAX_PROBE_SECONDS,
} from './constants.ts';
import {
  estimate,
  type EstimateOptions,
} from './stream.ts';
import type { EstimateSnapshot, } from './types.ts';

/**
 Default options for the local path.
 */
const OPTIONS: EstimateOptions = {
  defaultBranchOnly: false,
  maxProbeSeconds: DEFAULT_MAX_PROBE_SECONDS,
  maxDeepenCommits: DEFAULT_MAX_DEEPEN_COMMITS,
  maxPackBytes: DEFAULT_MAX_PACK_BYTES,
};

/**
 A scratch directory that removes itself at the end of a `using` scope.
 */
type DisposableDir = { readonly path: string; readonly [Symbol.dispose]: () => void; };

/**
 Creates a self-cleaning throwaway repo with explicit-pathspec commits.
 
 @param commits - number of commits
 
 @returns disposable repository handle
 */
function makeRepo(commits: number): DisposableDir {
  const path = mkdtempSync(join(tmpdir(), 'gcs-stream-'));
  execFileSync('git', ['init', '-q', path,]);
  execFileSync('git', ['-C', path, 'config', 'user.email', 't@t.t',]);
  execFileSync('git', ['-C', path, 'config', 'user.name', 't',]);
  for (let i = 0; i < commits; i += 1) {
    const file = `file${String(i)}.txt`;
    writeFileSync(join(path, file), `body ${String(i)} content\n`);
    execFileSync('git', ['-C', path, 'add', file,]);
    execFileSync('git', ['-C', path, 'commit', '-q', '-m', `c${String(i)}`, file,]);
  }
  return {
    path,
    [Symbol.dispose]() {
      rmSync(path, { recursive: true, force: true, });
    },
  };
}

/**
 Creates a self-cleaning throwaway repo with an unborn HEAD (no commits).
 
 @returns disposable repository handle
 */
function makeUnbornRepo(): DisposableDir {
  const path = mkdtempSync(join(tmpdir(), 'gcs-stream-unborn-'));
  execFileSync('git', ['init', '-q', path,]);
  return {
    path,
    [Symbol.dispose]() {
      rmSync(path, { recursive: true, force: true, });
    },
  };
}

await describe({
  name: estimate.name,
  children: [
    it({
      name: 'streams a coarse prior then a final exact snapshot for a local repo',
      fn: async ({ expect, }) => {
        using repo = makeRepo(4);
        const snapshots: EstimateSnapshot[] = [];
        for await (const snapshot of estimate({ source: { kind: 'local', path: repo.path, }, options: OPTIONS, })) {
          snapshots.push(snapshot);
        }
        expect(snapshots.length).toBe(2);
        const [first, last,] = snapshots;
        expect(first?.done).toBe(false);
        expect(first?.shallow).toBeUndefined();
        expect(first?.pending).toEqual(['local-exact',]);
        expect(last?.done).toBe(true);
        expect(last?.full.confidence).toBe('very high');
        expect(last?.ratio?.point).toBeGreaterThan(0);
        expect(last?.pending).toEqual([]);
      },
    }),

    it({
      name: 'streams snapshots for an unborn-HEAD repo without rejecting',
      fn: async ({ expect, }) => {
        using repo = makeUnbornRepo();
        const snapshots: EstimateSnapshot[] = [];
        for await (const snapshot of estimate({ source: { kind: 'local', path: repo.path, }, options: OPTIONS, })) {
          snapshots.push(snapshot);
        }
        expect(snapshots.length).toBe(2);
        const last = snapshots.at(-1,);
        expect(last?.done).toBe(true);
        expect(last?.shallow).toBeUndefined();
        expect(last?.ratio).toBeUndefined();
      },
    }),
  ],
});
