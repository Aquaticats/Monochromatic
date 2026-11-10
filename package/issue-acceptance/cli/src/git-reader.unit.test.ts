/**
 Tests for the branch reader, driven by a fake `git` executable.

 @module
 */

import {
  chmod,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ChildProcessError,
  createGitReader,
} from '../dist/final/node/index.mjs';

/**
 What the fake `git` does, written in its own words so each test reads as one question.
 */
const FAKE_GIT = String.raw`#!/usr/bin/env node
import { appendFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
appendFileSync(join(here, 'argv.log'), JSON.stringify(args) + '\n');
const last = args[args.length - 1];
const pattern = args[args.indexOf('-e') + 1];
const commit = args[args.indexOf('--is-ancestor') + 1];

if (args.includes('grep')) {
  if (pattern === 'two-files') {
    process.stdout.write('main:a.ts:2\nmain:b.ts:3');
    process.exit(0);
  }
  if (pattern === 'unreadable') {
    process.stderr.write('fatal: bad revision');
    process.exit(128);
  }
  if (pattern === 'not-a-count') {
    process.stdout.write('main:a.ts:many');
    process.exit(0);
  }
  process.exit(1);
}
if (args.includes('cat-file')) {
  if (last.endsWith(':present')) { process.exit(0); }
  if (last.endsWith(':present-unreadable')) { process.exit(0); }
  if (last.endsWith(':absent')) { process.exit(128); }
  process.stderr.write('fatal: not a git repository');
  process.exit(129);
}
if (args.includes('show')) {
  if (last.includes(':present-unreadable')) {
    process.stderr.write('fatal: could not read object');
    process.exit(1);
  }
  process.stdout.write(['line one', 'line two'].join(String.fromCharCode(10)));
  process.exit(0);
}
if (args.includes('ls-tree')) {
  if (last === 'empty') { process.exit(0); }
  if (last === 'unreadable') {
    process.stderr.write('fatal: bad revision');
    process.exit(128);
  }
  process.stdout.write('package/a/src/b.ts\nREADME.md\n');
  process.exit(0);
}
if (args.includes('merge-base')) {
  if (commit === 'onmain') { process.exit(0); }
  if (commit === 'unknownsha') {
    process.stderr.write('fatal: Not a valid object name');
    process.exit(128);
  }
  process.exit(1);
}
process.stderr.write('unexpected arguments');
process.exit(99);
`;

/**
 Writes the fake `git` into a fresh folder.
 
 @param directory - Folder the fake lives in.
 
 @returns The fake's path.
 
 @example
 ```ts
 const executable = await writeFakeGit('/tmp/x');
 ```
 */
async function writeFakeGit(directory: string,): Promise<string> {
  const path = join(directory, 'git',);
  await writeFile(path, FAKE_GIT, 'utf8',);
  await chmod(path, 0o700,);
  return path;
}

/**
 Reads the argument vectors the fake `git` was called with.
 
 @param directory - Folder the fake lives in.
 
 @returns One parsed vector per call.
 
 @example
 ```ts
 const calls = await recordedCalls('/tmp/x');
 ```
 */
async function recordedCalls(directory: string,): Promise<readonly (readonly string[])[]> {
  const text = await readFile(join(directory, 'argv.log',), 'utf8',);
  return text
    .split('\n',)
    .filter(function isCall(line,): boolean {
      return line !== '';
    },)
    .map(function toVector(line,): readonly string[] {
      return JSON.parse(line,) as readonly string[];
    },);
}

await describe({
  name: createGitReader.name,
  children: [
    it({
      name: 'sums the per-file counts of one grep and keeps the checkout in the argument vector',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const executable = await writeFakeGit(directory.path,);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'main',
          executable,
        },);
        const total = await reader.countMatches({
          pattern: 'two-files',
          paths: ['package/a/src',],
        },);
        expect(total,).toBe(5,);
        const calls = await recordedCalls(directory.path,);
        expect(calls[0],).toEqual([
          '-C',
          '/repo',
          'grep',
          '--count',
          '-E',
          '-e',
          'two-files',
          'main',
          '--',
          'package/a/src',
        ],);
      },
    },),
    it({
      name: 'answers zero when grep finds nothing',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'main',
          executable: await writeFakeGit(directory.path,),
        },);
        expect(await reader.countMatches({
          pattern: 'nothing-here',
          paths: ['package/a/src',],
        },),).toBe(0,);
      },
    },),
    it({
      name: 'refuses a grep that failed for a reason other than no match',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'main',
          executable: await writeFakeGit(directory.path,),
        },);
        let caught: unknown;
        try {
          await reader.countMatches({
            pattern: 'unreadable',
            paths: ['package/a/src',],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ChildProcessError,);
        expect((caught as Error).message,).toContain('git grep failed',);
      },
    },),
    it({
      name: 'refuses a count it cannot read',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'main',
          executable: await writeFakeGit(directory.path,),
        },);
        let caught: unknown;
        try {
          await reader.countMatches({
            pattern: 'not-a-count',
            paths: ['package/a/src',],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ChildProcessError,);
        expect((caught as Error).message,).toContain('cannot read',);
      },
    },),
    it({
      name: 'answers presence from cat-file, treating its missing status as absent',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'main',
          executable: await writeFakeGit(directory.path,),
        },);
        expect(await reader.pathExists('present',),).toBe(true,);
        expect(await reader.pathExists('absent',),).toBe(false,);
      },
    },),
    it({
      name: 'refuses a presence question Git could not answer',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'main',
          executable: await writeFakeGit(directory.path,),
        },);
        let caught: unknown;
        try {
          await reader.pathExists('unreadable-repository',);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ChildProcessError,);
        expect((caught as Error).message,).toContain('could not answer whether',);
      },
    },),
    it({
      name: 'reads a tracked file',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'main',
          executable: await writeFakeGit(directory.path,),
        },);
        expect(await reader.fileText('present',),).toEqual({
          kind: 'text',
          text: 'line one\nline two',
        },);
      },
    },),
    it({
      name: 'answers nothing for a file the branch does not track',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'main',
          executable: await writeFakeGit(directory.path,),
        },);
        expect(await reader.fileText('absent',),).toEqual({ kind: 'absent', },);
      },
    },),
    it({
      name: 'refuses a show that failed for a tracked file',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'main',
          executable: await writeFakeGit(directory.path,),
        },);
        let caught: unknown;
        try {
          await reader.fileText('present-unreadable',);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ChildProcessError,);
        expect((caught as Error).message,).toContain('could not read',);
      },
    },),
    it({
      name: 'lists the tracked paths without the blank last line',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'main',
          executable: await writeFakeGit(directory.path,),
        },);
        expect(await reader.trackedPaths(),).toEqual(['package/a/src/b.ts', 'README.md',],);
      },
    },),
    it({
      name: 'refuses a listing it could not read',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'unreadable',
          executable: await writeFakeGit(directory.path,),
        },);
        let caught: unknown;
        try {
          await reader.trackedPaths();
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ChildProcessError,);
        expect((caught as Error).message,).toContain('could not list',);
      },
    },),
    it({
      name: 'answers ancestry for a commit the branch holds and for one it does not',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'main',
          executable: await writeFakeGit(directory.path,),
        },);
        expect(await reader.isOnBranch('onmain',),).toBe(true,);
        expect(await reader.isOnBranch('offmain',),).toBe(false,);
      },
    },),
    it({
      name: 'refuses an ancestry question about a commit Git cannot resolve',
      fn: async () => {
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-git-',),);
        const reader = createGitReader({
          checkout: '/repo',
          branch: 'main',
          executable: await writeFakeGit(directory.path,),
        },);
        let caught: unknown;
        try {
          await reader.isOnBranch('unknownsha',);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ChildProcessError,);
        expect((caught as Error).message,).toContain('cannot resolve',);
      },
    },),
  ],
},);
