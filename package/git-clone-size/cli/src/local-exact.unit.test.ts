/**
 Tests the exact local pack-objects path against real throwaway repositories,
 validating the full size against a reference bare clone.
 
 @module
 */

import { execFileSync, } from 'node:child_process';
import {
  mkdtempSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  countObjectsSizePack,
  localExact,
} from './local-exact.ts';
import { isMeasured, } from './measure.ts';

/**
 Sums the byte size of every `*.pack` file under a repo's pack directory,
 the transfer-pack basis (excludes the client-generated `.idx`).
 
 @param gitDir - bare git directory
 
 @returns total pack-file bytes
 */
function packBytes(gitDir: string): number {
  const packDir = join(gitDir, 'objects', 'pack');
  return readdirSync(packDir)
    .filter((name) => name.endsWith('.pack'))
    .reduce((total, name) => total + statSync(join(packDir, name)).size, 0);
}

/**
 A scratch directory that removes itself at the end of a `using` scope.
 */
type DisposableDir = { readonly path: string; readonly [Symbol.dispose]: () => void; };

/**
 Creates a self-cleaning temp directory.
 
 @param prefix - leading name fragment
 
 @returns disposable directory handle
 */
function disposableDir(prefix: string): DisposableDir {
  const path = mkdtempSync(join(tmpdir(), prefix));
  return {
    path,
    [Symbol.dispose]() {
      rmSync(path, { recursive: true, force: true, });
    },
  };
}

/**
 Creates a self-cleaning throwaway git repo with `commits` explicit-pathspec
 commits (satisfying the repo's commit-only enforcement guard).
 
 @param commits - number of commits to create
 
 @returns disposable repository handle
 */
function makeRepo(commits: number): DisposableDir {
  const dir = disposableDir('gcs-fixture-');
  execFileSync('git', ['init', '-q', dir.path,]);
  execFileSync('git', ['-C', dir.path, 'config', 'user.email', 't@t.t',]);
  execFileSync('git', ['-C', dir.path, 'config', 'user.name', 't',]);
  for (let i = 0; i < commits; i += 1) {
    const file = `file${String(i)}.txt`;
    writeFileSync(join(dir.path, file), `content of file ${String(i)} with some bytes to pack\n`);
    execFileSync('git', ['-C', dir.path, 'add', file,]);
    execFileSync('git', ['-C', dir.path, 'commit', '-q', '-m', `commit ${String(i)}`, file,]);
  }
  return dir;
}

await describe({
  name: localExact.name,
  children: [
    it({
      name: 'measures an exact full/shallow size with very high confidence',
      fn: async ({ expect, }) => {
        using repo = makeRepo(6);
        const result = await localExact({ path: repo.path, });
        const fullBytes = nonNullishOrThrow(result.fullBytes,);
        const shallowBytes = nonNullishOrThrow(result.shallowBytes,);
        expect(result.confidence).toBe('very high');
        expect(fullBytes).toBeGreaterThan(0);
        expect(shallowBytes).toBeGreaterThan(0);
        expect(fullBytes).toBeGreaterThanOrEqual(shallowBytes);
      },
    }),

    it({
      name: 'matches a reference bare clone objects size within a small band',
      fn: async ({ expect, }) => {
        using repo = makeRepo(8);
        using refParent = disposableDir('gcs-ref-');
        const reference = join(refParent.path, 'ref.git');
        const result = await localExact({ path: repo.path, });
        const fullBytes = nonNullishOrThrow(result.fullBytes,);
        execFileSync('git', ['clone', '--bare', '-q', `file://${repo.path}`, reference,]);
        execFileSync('git', ['-C', reference, 'repack', '-adq',]);
        const ratio = fullBytes / packBytes(reference);
        expect(ratio).toBeGreaterThan(0.8);
        expect(ratio).toBeLessThan(1.25);
      },
    }),

    it({
      name: 'never rejects on an unborn HEAD, omitting the unmeasurable shallow tip',
      fn: async ({ expect, }) => {
        using repo = disposableDir('gcs-unborn-');
        execFileSync('git', ['init', '-q', repo.path,]);
        const result = await localExact({ path: repo.path, });
        const fullBytes = nonNullishOrThrow(result.fullBytes,);
        expect(result.shallowBytes).toBeUndefined();
        expect(fullBytes).toBeGreaterThanOrEqual(0);
        expect(fullBytes).toBeLessThan(1_024);
      },
    }),
  ],
});

await describe({
  name: countObjectsSizePack.name,
  children: [
    it({
      name: 'returns a non-negative packed size',
      fn: async ({ expect, }) => {
        using repo = makeRepo(3);
        execFileSync('git', ['-C', repo.path, 'repack', '-adq',]);
        const bytes = await countObjectsSizePack({ path: repo.path, });
        expect(isMeasured(bytes,) && (bytes >= 0)).toBe(true);
      },
    }),
  ],
});
