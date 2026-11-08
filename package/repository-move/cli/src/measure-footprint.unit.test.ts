/**
 Tests of the footprint measurement tool against a stand-in `btrfs`: a measurement that succeeds, one whose walk loses a
 worktree that was removed meanwhile and is made again, one that fails for any other reason, one that loses a worktree every
 time, and one that does not finish. The stand-in is a shell script in a disposable folder first on `PATH`; no real `btrfs`
 and no real tree is touched.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  childFailureOf,
  execFileAsync,
} from '../dist/final/node/index.mjs';

const TOOL = new URL('measure-footprint.ts', import.meta.url,).pathname;
const EXECUTABLE_MODE = 0o755;
/** The total row the stand-in prints when a measurement succeeds. */
const TOTAL_ROW = '  300  100  50  /the/tree';
const HEADER_ROW = '     Total   Exclusive  Set shared  Filename';
const VANISHED_NAME = 'commit-timing-names-every-step';

type Folder = { readonly path: string; readonly [Symbol.asyncDispose]: () => Promise<void>; };

async function makeFolder(): Promise<Folder> {
  const path = await mkdtemp(join(scratchRoot(), 'measure-footprint-',),);
  return { path, async [Symbol.asyncDispose]() { await rm(path, { recursive: true, force: true, },); }, };
}

/** A stand-in `btrfs` that counts its calls in `calls` and behaves as `body` says on call number `$count`. */
async function standIn({ folder, body, }: { folder: string; body: string; },): Promise<string> {
  const bin = join(folder, 'bin',);
  await mkdir(bin, { recursive: true, },);
  await writeFile(join(bin, 'btrfs',), `#!/bin/sh\ncount=$(( $(cat '${folder}/calls' 2>/dev/null || echo 0) + 1 ))\necho $count > '${folder}/calls'\n${body}\n`,);
  await chmod(join(bin, 'btrfs',), EXECUTABLE_MODE,);
  return bin;
}

async function measure({ folder, bin, args = ['3', '5',], }: { folder: string; bin: string; args?: readonly string[]; },) {
  try {
    const done = await execFileAsync(process.execPath, [TOOL, '/the/tree', ...args,], { env: { ...process.env, PATH: `${bin}:/usr/bin:/bin`, }, encoding: 'utf8', },);
    return { code: 0, out: `${done.stdout}${done.stderr}`, calls: Number((await readFile(join(folder, 'calls',), 'utf8',)).trim(),), };
  }
  catch (error) {
    const failure = childFailureOf(error,);
    return { code: failure.code, out: `${failure.stdout}${failure.stderr}`, calls: Number((await readFile(join(folder, 'calls',), 'utf8',)).trim(),), };
  }
}

await describe({
  name: 'measure-footprint',
  concurrency: 1,
  children: [
    it({
      name: 'a measurement that succeeds is one attempt, with the total row last so the preflight reads it',
      fn: async () => {
        await using folder = await makeFolder();
        const bin = await standIn({ folder: folder.path, body: `echo '${HEADER_ROW}'\necho '${TOTAL_ROW}'`, },);
        const result = await measure({ folder: folder.path, bin, },);
        expect(result.code,).toBe(0,);
        expect(result.calls,).toBe(1,);
        expect(result.out,).toContain('measured: attempt 1 of 3; nothing was removed during the measurement',);
        expect(result.out.trim().split('\n',).at(-1,),).toBe(TOTAL_ROW.trim(),);
      },
    },),
    it({
      name: 'a walk that loses a worktree removed meanwhile is measured again, and the removed one is named',
      fn: async () => {
        await using folder = await makeFolder();
        const bin = await standIn({
          folder: folder.path,
          body: `if [ "$count" = 1 ]; then\n  echo "WARNING: cannot access '${VANISHED_NAME}': No such file or directory" >&2\n  echo "ERROR: cannot check space of '/the/tree': No such file or directory" >&2\n  exit 1\nfi\necho '${HEADER_ROW}'\necho '${TOTAL_ROW}'`,
        },);
        const result = await measure({ folder: folder.path, bin, },);
        expect(result.code,).toBe(0,);
        expect(result.calls,).toBe(2,);
        expect(result.out,).toContain(`measured: attempt 2 of 3; removed during the measurement: ${VANISHED_NAME}`,);
        expect(result.out.trim().split('\n',).at(-1,),).toBe(TOTAL_ROW.trim(),);
      },
    },),
    it({
      name: 'any other error fails at once without another attempt, and says what btrfs said',
      fn: async () => {
        await using folder = await makeFolder();
        const bin = await standIn({ folder: folder.path, body: `echo "ERROR: cannot check space of '/the/tree': Input/output error" >&2\nexit 1`, },);
        const result = await measure({ folder: folder.path, bin, },);
        expect(result.code,).toBe(1,);
        expect(result.calls,).toBe(1,);
        expect(result.out,).toContain('Input/output error',);
        expect(result.out,).toContain('STOP',);
      },
    },),
    it({
      name: 'a tree that loses a worktree in every attempt fails after the last, naming what was lost',
      fn: async () => {
        await using folder = await makeFolder();
        const bin = await standIn({
          folder: folder.path,
          body: `echo "WARNING: cannot access 'gone-$count': No such file or directory" >&2\necho "ERROR: cannot check space of '/the/tree': No such file or directory" >&2\nexit 1`,
        },);
        const result = await measure({ folder: folder.path, bin, },);
        expect(result.code,).toBe(1,);
        expect(result.calls,).toBe(3,);
        expect(result.out,).toContain('gone-1',);
        expect(result.out,).toContain('gone-3',);
        expect(result.out,).toContain('every one of 3 attempts',);
      },
    },),
    it({
      name: 'an attempt that does not finish inside its deadline fails the measurement instead of waiting',
      fn: async () => {
        await using folder = await makeFolder();
        const bin = await standIn({ folder: folder.path, body: 'exec sleep 30', },);
        const started = performance.now();
        const result = await measure({ folder: folder.path, bin, args: ['3', '1',], },);
        expect(result.code,).toBe(1,);
        expect(result.calls,).toBe(1,);
        expect(result.out,).toContain('did not finish within 1 s',);
        expect(performance.now() - started,).toBeLessThan(20_000,);
      },
    },),
  ],
},);
