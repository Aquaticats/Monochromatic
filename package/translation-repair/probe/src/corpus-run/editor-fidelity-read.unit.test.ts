/**
 Tests for the compiled `editor-fidelity-read` command: what it prints for the logs of the arms it is given, which
 exit status a failed check gives it, and which requests it refuses before it reads a log.

 Each run is the built command in an owned empty directory with no provider key set. Fixtures are cat-themed invention.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';

/**
 Exit status the command gives a refusal it states in its own words.
 */
const REFUSED_AS_STATED = 6;

/**
 What a run left behind.
 */
type BuiltRun = {
  /**
   Exit status, zero for an accepted request.
   */
  readonly code: number;

  /**
   Standard output, one entry per line.
   */
  readonly stdout: readonly string[];

  /**
   Standard error, one entry per line.
   */
  readonly stderr: readonly string[];
};

/**
 Splits a stream into its nonempty lines.

 @param text - stream contents

 @returns Lines

 @example
 ```ts
 // yields ['a', 'b']
 const lines = linesOf({ text: 'a\nb\n', },);
 ```
 */
function linesOf({ text, }: { readonly text: string; },): readonly string[] {
  return text.split('\n',)
    .filter(function present(line,): boolean {
      return line !== '';
    },);
}

/**
 Runs the built command in an owned directory, after writing the files given into it.

 @param files - files to write, by name

 @param args - arguments after the script path, with `DIR` standing for the directory

 @returns Exit status and both output streams, one entry per line

 @example
 ```ts
 const run = await runAsBuilt({ files: { 'low.log': '', }, args: ['DIR/low.log',], },);
 ```
 */
async function runAsBuilt(
  {
    files,
    args,
  }: {
    readonly files: Readonly<Record<string, string>>;
    readonly args: readonly string[];
  },
): Promise<BuiltRun> {
  await using owned = await scratchDir({ prefix: 'editor-fidelity-read-as-built-', },);
  await Promise.all(Object.entries(files,)
    .map(function write([name, text,]: readonly [string, string],): Promise<void> {
      return writeFile(
        join(
          owned.path,
          name,
        ),
        text,
      );
    },),);
  /**
   What the built command did, whatever its exit code.
   */
  const run = await runBuiltCommand({
    command: 'editor-fidelity-read',
    packageEntry: PROBE_ENTRY,
    args: args.map(function resolved(arg,): string {
      return arg.replace(
        'DIR',
        owned.path,
      );
    },),
    cwd: owned.path,
    env: {},
  },);
  return {
    code: run.code,
    stdout: linesOf({ text: run.stdout, },),
    stderr: linesOf({ text: run.stderr, },),
  };
}

await describe({
  name: 'editor-fidelity-read as built',
  children: [
    it({
      name: 'PRINTS A FAIL LINE FOR EACH CHECK AN EMPTY LOG CANNOT PASS and exits six naming how many failed, so an '
        + 'arm that left nothing behind is not read as evidence',
      fn: async () => {
        const run = await runAsBuilt({
          files: { 'low.log': '', },
          args: ['DIR/low.log',],
        },);
        expect(run.code,).toBe(REFUSED_AS_STATED,);
        expect(run.stdout.filter(function isFail(line,): boolean {
          return line.startsWith('FAIL low: ',);
        },).length,).toBeGreaterThan(5,);
        expect(run.stdout.at(-1,)?.startsWith('CHECKS ',),).toBe(true,);
        expect(run.stdout.at(-1,)?.endsWith(' fail',),).toBe(true,);
        expect(run.stderr[0],).toContain('editor-fidelity-read: ',);
      },
    },),

    it({
      name: 'READS THE EXIT STATUS FROM THE FILE BESIDE THE LOG, named for the arm with .exit for .log, and fails the exit '
        + 'check for a status it does not read as zero',
      fn: async () => {
        const run = await runAsBuilt({
          files: {
            'low.log': '',
            'low.exit': '3\n',
          },
          args: ['DIR/low.log',],
        },);
        expect(run.stdout,).toContain('FAIL low: exit 0: the process exited with status 3',);
      },
    },),

    it({
      name: 'PRINTS THE BENCH CHECKS and exits six for logs that are not three or four distinct arms at the cards high, '
        + 'medium, a second medium if run, and low, so a bench of one log, or of a log repeated, is not read as the bench',
      fn: async () => {
        const one = await runAsBuilt({
          files: { 'low.log': '', },
          args: ['DIR/low.log',],
        },);
        expect(one.code,).toBe(REFUSED_AS_STATED,);
        expect(one.stdout,).toContain(
          'FAIL bench: three or four distinct arm logs: 1 log was given, and a bench is three or four',
        );
        expect(one.stdout.some(function isCards(line,): boolean {
          return line.startsWith('FAIL bench: cards high, medium, a second medium if run, low in the order run',);
        },),).toBe(true,);
        const three = await runAsBuilt({
          files: {
            'high.log': '',
            'medium.log': '',
            'low.log': '',
          },
          args: ['DIR/high.log', 'DIR/medium.log', 'DIR/low.log',],
        },);
        expect(three.code,).toBe(REFUSED_AS_STATED,);
        expect(three.stdout,).toContain(
          'FAIL bench: three or four distinct arm logs: high and medium hold the same lines',
        );
        expect(three.stdout.some(function isEarlier(line,): boolean {
          return line.startsWith('FAIL low: earlier spend declared: ',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES A LOG THAT IS NOT THERE, naming the path, before it prints any check',
      fn: async () => {
        const run = await runAsBuilt({
          files: {},
          args: ['DIR/nowhere.log',],
        },);
        expect(run.code,).toBe(REFUSED_AS_STATED,);
        expect(run.stdout,).toStrictEqual([],);
        expect(run.stderr[0],).toContain('nowhere.log',);
      },
    },),

    it({
      name: 'REFUSES A REQUEST THAT NAMES NO LOG, and a flag it does not declare',
      fn: async () => {
        const none = await runAsBuilt({
          files: {},
          args: [],
        },);
        expect(none.code,).toBe(REFUSED_AS_STATED,);
        const flagged = await runAsBuilt({
          files: {},
          args: ['--cap', '3', 'DIR/low.log',],
        },);
        expect(flagged.code,).toBe(REFUSED_AS_STATED,);
        expect(flagged.stderr[0],).toContain('editor-fidelity-read: ',);
      },
    },),
  ],
},);
