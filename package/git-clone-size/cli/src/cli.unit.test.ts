/**
 Integration test: runs the built bin against a throwaway repo and asserts the
 stdout is pure JSONL (one EstimateSnapshot per line) with diagnostics on stderr.
 
 @module
 */

import { execFileSync, } from 'node:child_process';
import {
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  findRootCached,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';
import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn, { type SubprocessError, } from 'nano-spawn';

import type { EstimateSnapshot, } from './types.ts';

/**
 Monorepo root so the built bin path is invariant to the launch directory.
 */
const REPO_ROOT = await findRootCached({ marker: MISE_MONOREPO, },);

/**
 Path to the built node bin.
 */
const BIN = 'package/git-clone-size/cli/dist/final/node/index.mjs';

/**
 A scratch directory that removes itself at the end of a `using` scope.
 */
type DisposableDir = { readonly path: string; readonly [Symbol.dispose]: () => void; };

/**
 Creates a self-cleaning throwaway git repo with real files and
 explicit-pathspec commits.
 
 @param commits - number of commits to create
 
 @returns disposable repository handle
 */
function makeRepo(commits: number): DisposableDir {
  const path = mkdtempSync(join(scratchRoot(), 'gcs-cli-'));
  execFileSync('git', ['init', '-q', path,]);
  execFileSync('git', ['-C', path, 'config', 'user.email', 't@t.t',]);
  execFileSync('git', ['-C', path, 'config', 'user.name', 't',]);
  for (let i = 0; i < commits; i += 1) {
    const file = `file${String(i)}.txt`;
    writeFileSync(join(path, file), `content ${String(i)} padded with text\n`);
    execFileSync('git', ['-C', path, 'add', file,]);
    execFileSync('git', ['-C', path, 'commit', '-q', '-m', `commit ${String(i)}`, file,]);
  }
  return {
    path,
    [Symbol.dispose]() {
      rmSync(path, { recursive: true, force: true, });
    },
  };
}

/**
 Runs the bin against a path and returns captured streams plus exit code.
 
 @param path - source path argument
 
 @param env - extra environment variables
 
 @returns stdout, stderr, and exit code
 */
async function runBin(
  path: string,
  env: Readonly<Record<string, string>>,
): Promise<{ stdout: string; stderr: string; exitCode: number; }> {
  try {
    const result = await spawn('node', [BIN, path, '--color=never',], { cwd: REPO_ROOT, env: { ...process.env, ...env, }, });
    return { exitCode: 0, stderr: result.stderr, stdout: result.stdout, };
  } catch (error: unknown) {
    const spawnError = error as SubprocessError;
    return { exitCode: spawnError.exitCode ?? 1, stderr: spawnError.stderr, stdout: spawnError.stdout, };
  }
}

await describe({
  name: 'git-clone-size integration',
  children: [
    it({
      name: 'emits only valid JSONL on stdout, ending with done:true',
      fn: async ({ expect, }) => {
        using repo = makeRepo(5);
        const result = await runBin(repo.path, { NO_COLOR: '1', });
        expect(result.exitCode).toBe(0);
        const lines = result.stdout.trim().split('\n');
        expect(lines.length).toBeGreaterThanOrEqual(1);
        for (const line of lines) {
          const snapshot = JSON.parse(line) as EstimateSnapshot;
          expect(typeof snapshot.metric).toBe('string');
          expect(typeof snapshot.scope).toBe('string');
          expect(Array.isArray(snapshot.basis)).toBe(true);
          expect(Array.isArray(snapshot.pending)).toBe(true);
        }
        const last = JSON.parse(lines.at(-1) ?? '{}') as EstimateSnapshot;
        expect(last.done).toBe(true);
        expect(last.full.confidence).toBe('very high');
        expect(last.ratio?.point).toBeGreaterThan(0);
        expect(last.savings?.point).toBeGreaterThan(0);
      },
    }),

    it({
      name: 'keeps stdout free of ANSI escapes under --color=never',
      fn: async ({ expect, }) => {
        using repo = makeRepo(3);
        const result = await runBin(repo.path, { NO_COLOR: '1', });
        expect(
          result.stdout.includes(String.fromCodePoint(27)),
        ).toBe(false);
      },
    }),
  ],
});
