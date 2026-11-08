/**
 Tests that no shell the move tools start reads the user's `~/.bashrc` or `~/.bash_profile`: a stage run through the
 guided runner's shell, and the shells the copy tool starts for its send and receive pipelines, leave no marker that a
 startup file would write; and no `bash` the source starts (an argument list, or text a stage hands to `sudo`) lacks
 `--noprofile --norc`. Each run uses a disposable home; nothing real is touched.

 Why `SHLVL=0` appears: bash 5.3 takes a non-interactive `bash -c` whose standard input is a socket (what Node gives a
 piped child) for a remote shell and reads `~/.bashrc`, but only when the `SHLVL` it inherits is not a positive number
 (measured here 2026-11-07: unset, `0` and `x` read it, `1` and `2` do not). A run from an interactive terminal inherits
 a positive `SHLVL`, so the tests set `0` to stand for a run from systemd, `sudo`, or a scrubbed environment.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readdir,
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
  execFileAsync,
} from '../dist/final/node/index.mjs';
import { makeFixture, } from './move-fixture.ts';
import { runCopyTool, } from './move-fixture-run.ts';

const INDEX = new URL('../dist/final/node/index.mjs', import.meta.url,).pathname;
const SOURCE_FOLDER = new URL('.', import.meta.url,).pathname;
const MARKER_NAME = 'startup-file-was-read';
const REQUIRED_FLAGS = '--noprofile--norc';
const SEPARATORS = new Set(['\'', '"', '`', ',', ' ', '\t', '\n',],);
const OPENERS = new Set(['\'', '"', '`', ' ', '\t', '\n',],);
const CLOSERS = new Set(['\'', '"', ' ', '\t', '\n',],);
const FLAG_WINDOW = 80;
const ARGS_LABEL = 'args:[';
const NOT_A_SHELL_LEVEL = '0';

type Home = {
  readonly path: string;
  readonly marker: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/** A home whose `.bashrc` and `.bash_profile` each write a marker file when bash reads them. */
async function makeHomeWithStartupFiles(): Promise<Home> {
  const path = await mkdtemp(join(scratchRoot(), 'bash-startup-home-',),);
  const marker = join(path, MARKER_NAME,);
  await writeFile(join(path, '.bashrc',), `echo bashrc >> '${marker}'\n`,);
  await writeFile(join(path, '.bash_profile',), `echo bash_profile >> '${marker}'\n`,);
  return {
    path,
    marker,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

async function exists({ folder, name, }: { folder: string; name: string; },): Promise<boolean> {
  return (await readdir(folder,)).includes(name,);
}

/** Runs one shell command through the runner's `runShell` in a child node whose environment names the disposable home. */
async function runThroughRunnerShell({ home, command, }: { home: Home; command: string; },): Promise<string> {
  const driver = `
    import { runShell } from ${JSON.stringify(INDEX,)};
    const status = await runShell({ command: ${JSON.stringify(command,)}, outputFile: ${JSON.stringify(join(home.path, 'out.txt',),)},
      outputFileEnv: 'out.txt', seconds: 60, killGrace: 5, out: () => {} });
    process.exitCode = status;
  `;
  await execFileAsync(
    process.execPath,
    ['--input-type=module', '--eval', driver,],
    { encoding: 'utf8', env: { PATH: process.env.PATH ?? '', HOME: home.path, SHLVL: NOT_A_SHELL_LEVEL, }, },
  );
  return readFile(join(home.path, 'out.txt',), 'utf8',);
}

/** The source files a run of the tools can start a shell from: every `.ts` file of `src` that is not a test. */
async function sourceFiles(): Promise<readonly string[]> {
  return (await readdir(SOURCE_FOLDER,)).filter((name,) => (name.endsWith('.ts',) && (!name.endsWith('.test.ts',))),);
}

/**
 The text of a source file without the block comments that start a line and the whole-line `//` comments, which name
 `bash` without starting it. A line that opens a block comment ends the block when it holds the closing mark, else a
 later line does.
 */
function withoutComments(text: string,): string {
  return text.split('\n',).reduce<{ readonly inBlock: boolean; readonly kept: readonly string[]; }>(
    function step(state, line,) {
      const trimmed = line.trimStart();
      if (state.inBlock) {
        return { inBlock: !trimmed.includes('*/',), kept: state.kept, };
      }
      if (trimmed.startsWith('/*',)) {
        return { inBlock: !trimmed.includes('*/',), kept: state.kept, };
      }
      return { inBlock: false, kept: trimmed.startsWith('//',) ? state.kept : [...state.kept, line,], };
    },
    { inBlock: false, kept: [], },
  ).kept.join('\n',);
}

/** The text without the quotes, commas and white space that separate the words of an argument list or a command. */
function withoutSeparators(text: string,): string {
  return text.split('',).filter((character,) => !SEPARATORS.has(character,)).join('',);
}

/**
 What follows each place the text starts `bash`: the word after a quote, backtick or white space (never `/bin/bash`),
 not a name being assigned, cut to a window long enough to hold the flags.
 */
function afterBashStarts(text: string,): readonly string[] {
  const pieces = text.split('bash',);
  return pieces.slice(1,).flatMap((piece, index,) => {
    const before = (pieces[index] ?? '').slice(-1,);
    const assigned = piece.trimStart().startsWith('=',);
    return (OPENERS.has(before,) && CLOSERS.has(piece.slice(0, 1,),) && (!assigned)) ? [piece.slice(0, FLAG_WINDOW,),] : [];
  },);
}

/** What follows a start of bash, without separators and without the `args: [` of an argument list written beside `command`. */
function flagsAfter(following: string,): string {
  const flags = withoutSeparators(following,);
  return flags.startsWith(ARGS_LABEL,) ? flags.slice(ARGS_LABEL.length,) : flags;
}

await describe({
  name: 'bash without startup files',
  concurrency: 1,
  children: [
    it({
      name: 'the probe sees a startup file: a plain bash started with a piped input reads the disposable home',
      fn: async () => {
        await using home = await makeHomeWithStartupFiles();
        await execFileAsync(
          'bash',
          ['-c', 'true',],
          { encoding: 'utf8', env: { PATH: process.env.PATH ?? '', HOME: home.path, SHLVL: NOT_A_SHELL_LEVEL, }, },
        );
        expect(await exists({ folder: home.path, name: MARKER_NAME, },),).toBe(true,);
      },
    },),
    it({
      name: 'a stage run through the runner shell leaves no marker from .bashrc or .bash_profile and still runs the command',
      fn: async () => {
        await using home = await makeHomeWithStartupFiles();
        const output = await runThroughRunnerShell({ home, command: 'echo stage-ran', },);
        expect(output,).toContain('stage-ran',);
        expect(await exists({ folder: home.path, name: MARKER_NAME, },),).toBe(false,);
      },
    },),
    it({
      name: 'every shell the copy tool starts for its first and final passes leaves no marker',
      fn: async () => {
        await using home = await makeHomeWithStartupFiles();
        await using fixture = await makeFixture({ names: ['a',], },);
        const env = { HOME: home.path, SHLVL: NOT_A_SHELL_LEVEL, };
        expect((await runCopyTool({ fixture, pass: 'first', env, },)).status,).toBe(0,);
        expect((await runCopyTool({ fixture, pass: 'freeze', env, },)).status,).toBe(0,);
        const final = await runCopyTool({ fixture, pass: 'final', env, },);
        expect(final.status,).toBe(0,);
        expect(await readFile(fixture.doubleLog, 'utf8',),).toContain('"receive"',);
        expect(await exists({ folder: home.path, name: MARKER_NAME, },),).toBe(false,);
      },
    },),
    it({
      name: 'no bash in the source, as an argument list or as text handed to sudo, lacks --noprofile --norc',
      fn: async () => {
        const files = await sourceFiles();
        const texts = await Promise.all(files.map(async (name,) => withoutComments(await readFile(join(SOURCE_FOLDER, name,), 'utf8',),),),);
        const unflagged = files.flatMap(function unflaggedIn(name, index,) {
          const followings = afterBashStarts(texts[index] ?? '',);
          return followings
            .filter((following,) => !flagsAfter(following,).startsWith(REQUIRED_FLAGS,),)
            .map((following,) => `${name}: bash${following.replaceAll('\n', ' ',)}`,);
        },);
        expect(unflagged,).toEqual([],);
      },
    },),
  ],
},);
