/**
 Tests for the run of the editor fidelity pool draw over scripted parts: what it prints, what it writes, and what a
 check says when the committed files are not what a fresh draw gives. No clone is read; entries and files are handed in.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  type FidelityReferenceSpec,
  type PoolFile,
  runEditorFidelityPoolDraw,
} from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { inventedReference, } from '../editor-fidelity.test-fixture.ts';
import {
  DRAW_PIN,
  readInventedEntry,
} from '../editor-fidelity-pool.test-fixture.ts';

/**
 What a scripted run did.
 */
type DrawSeen = {
  /**
   Lines logged.
   */
  readonly lines: readonly string[];

  /**
   Files written, by directory.
   */
  readonly written: readonly { readonly dir: string; readonly files: readonly PoolFile[]; }[];

  /**
   What the run threw.
   */
  readonly failures: readonly unknown[];
};

/**
 Runs the draw over six invented entries.

 @param typed - what the operator wrote after the script path

 @param onDisk - files the check reads, by name

 @returns What the run did

 @example
 ```ts
 const seen = await drawnWith({ typed: ['--want', '4',], onDisk: [], },);
 ```
 */
async function drawnWith(
  {
    typed,
    onDisk,
    committed = [],
  }: {
    readonly typed: readonly string[];
    readonly onDisk: readonly PoolFile[];
    readonly committed?: readonly FidelityReferenceSpec[];
  },
): Promise<DrawSeen> {
  /**
   Lines the run logged.
   */
  const logged = capturingLoggerPair();
  /**
   Files the run wrote.
   */
  const written: { readonly dir: string; readonly files: readonly PoolFile[]; }[] = [];
  /**
   What the run threw.
   */
  const failures: unknown[] = [];
  try {
    await runEditorFidelityPoolDraw({
      line: lineOf({
        command: 'editor-fidelity-pool-draw',
        typed,
      },),
      pin: DRAW_PIN,
      listEntries: function listEntries(): Promise<readonly string[]> {
        return Promise.resolve(['Acorn', 'Briar', 'Cinder', 'Dapple', 'Ember', 'Fennel',],);
      },
      readEntry: function readEntry({ entryId, },) {
        return Promise.resolve(readInventedEntry({ entryId, },),);
      },
      readFiles: function readFiles(): Promise<readonly PoolFile[]> {
        return Promise.resolve(onDisk,);
      },
      writeFiles: function writeFiles({ dir, files, },): Promise<void> {
        written.push({
          dir,
          files,
        },);
        return Promise.resolve();
      },
      committed,
      reviewedEntryIds: [],
      reviewedTrials: 0,
      readReferences: function readReferences() {
        return Promise.resolve([inventedReference(),],);
      },
      log: logged.logger,
    },);
  }
  catch (error) {
    failures.push(error,);
  }
  return {
    lines: logged.lines,
    written,
    failures,
  };
}

await describe({
  name: runEditorFidelityPoolDraw.name,
  children: [
    it({
      name: 'SAYS THE SEED, THE TRIALS WANTED, THE ENTRIES IT COULD DRAW AND TOOK, THE KINDS AND THE SKIPS, and writes '
        + 'nothing when no directory is named',
      fn: async () => {
        const seen = await drawnWith({
          typed: ['--seed', 'a seed', '--want', '5',],
          onDisk: [],
        },);
        expect(seen.failures,).toStrictEqual([],);
        expect(seen.lines,).toContain('draw: seed `a seed`, 5 trials wanted (0 reviewed counted), 6 entries listed',);
        expect(seen.lines,).toContain('draw: eligible entries 6, drawn entries 3, trials 6 (insertion 3, alteration 3)',);
        expect(seen.lines.some(function isSheets(line,): boolean {
          return line.startsWith('draw: sheet chars min ',);
        },),).toBe(true,);
        expect(seen.written,).toStrictEqual([],);
      },
    },),

    it({
      name: 'WRITES THE PARTS to the directory named and says how many files',
      fn: async () => {
        const seen = await drawnWith({
          typed: ['--seed', 'a seed', '--want', '5', '--out', 'somewhere',],
          onDisk: [],
        },);
        expect(seen.written.length,).toBe(1,);
        expect(nonNullishOrThrow(seen.written[0],).dir,).toBe('somewhere',);
        expect(seen.lines.some(function wrote(line,): boolean {
          return line.startsWith('wrote ',) && line.endsWith(' to somewhere',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'CHECKS INSTEAD OF WRITING: the files on disk equal a fresh draw, so the check passes and writes nothing',
      fn: async () => {
        const first = await drawnWith({
          typed: ['--seed', 'a seed', '--want', '5', '--out', 'somewhere',],
          onDisk: [],
        },);
        const { files, } = nonNullishOrThrow(first.written[0],);
        const checked = await drawnWith({
          typed: ['--seed', 'a seed', '--want', '5', '--out', 'somewhere', '--check',],
          onDisk: files,
        },);
        expect(checked.failures,).toStrictEqual([],);
        expect(checked.written,).toStrictEqual([],);
        expect(checked.lines.some(function same(line,): boolean {
          return line === `check: ${String(files.length,)} files equal a fresh draw`;
        },),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES A CHECK WHOSE FILES DIFFER FROM A FRESH DRAW, naming the files, so a manifest that was edited '
        + 'by hand or drawn under another seed is not taken for the draw',
      fn: async () => {
        const first = await drawnWith({
          typed: ['--seed', 'a seed', '--want', '5', '--out', 'somewhere',],
          onDisk: [],
        },);
        const { files, } = nonNullishOrThrow(first.written[0],);
        const edited = files.map(function edit(file,): PoolFile {
          return file.name === 'editor-fidelity-pool-drawn-1.ts'
            ? {
              ...file,
              text: `${file.text}// edited\n`,
            }
            : file;
        },);
        const checked = await drawnWith({
          typed: ['--seed', 'a seed', '--want', '5', '--out', 'somewhere', '--check',],
          onDisk: edited,
        },);
        const [failure,] = checked.failures;
        expect(failure instanceof StatedRefusalError,).toBe(true,);
        expect((failure as Error).message,).toContain('editor-fidelity-pool-drawn-1.ts',);
      },
    },),

    it({
      name: 'REPLAYS THE COMMITTED POOL in a check: reads every reference through the hash-verifying reader, builds '
        + 'its trials and says how many a scripted ideal editor fixes cleanly while no cheap reply does',
      fn: async () => {
        const first = await drawnWith({
          typed: ['--seed', 'a seed', '--want', '5', '--out', 'somewhere',],
          onDisk: [],
        },);
        const checked = await drawnWith({
          typed: ['--seed', 'a seed', '--want', '5', '--out', 'somewhere', '--check',],
          onDisk: nonNullishOrThrow(first.written[0],).files,
          committed: [inventedReference().spec,],
        },);
        expect(checked.failures,).toStrictEqual([],);
        expect(checked.lines,).toContain('replay: 1 reference read and verified, 2 trials, 2 scoreable',);
      },
    },),

    it({
      name: 'REFUSES A CHECK WITH NO DIRECTORY to compare against',
      fn: async () => {
        const checked = await drawnWith({
          typed: ['--check',],
          onDisk: [],
        },);
        const [failure,] = checked.failures;
        expect(failure instanceof StatedRefusalError,).toBe(true,);
        expect((failure as Error).message,).toContain('--check compares the draw with the files in --out',);
      },
    },),
  ],
},);
