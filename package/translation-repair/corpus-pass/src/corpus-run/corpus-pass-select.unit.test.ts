/**
 Tests for the choice of entries a corpus pass runs: every complete unsettled
 pair at the pin, or only those the command line names, resuming cached
 progress first, with the restriction and every pair it could not read said
 on the way.

 The corpus is a throwaway repository of invented pages, never the
 operator's clone.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { selectPendingEntries, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { makeCorpusPassClone, } from './corpus-pass-clone.test-fixture.ts';

/**
 Instant the injected wall clock reads, so no case waits on the real one.
 */
const PRINTED_AT = new Date('2026-10-22T08:10:11.123Z',);

/**
 The clock time every line a case prints carries.
 */
const AT = 'at=2026-10-22T08:10:11.123Z';

/**
 Wall clock that reads the same instant every time it is asked.

 @returns Instant the line is printed at

 @example
 ```ts
 await selectPendingEntries({ ...input, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

/**
 Original page of an entry whose size puts it in the small band.
 */
const SMALL_PAGE = '猫睡觉。\n';

/**
 English page of an entry.
 */
const ENGLISH_PAGE = 'The cat naps.\n';

/**
 Complete pairs of the invented corpus, in the order the clone lists them.
 */
const COMPLETE = [
  {
    id: 'biscuit',
    sourceText: SMALL_PAGE,
    targetText: ENGLISH_PAGE,
  },
  {
    id: 'tabby',
    sourceText: SMALL_PAGE,
    targetText: ENGLISH_PAGE,
  },
] as const;

/**
 Ids of a list of pairs, in order.

 @param pairs - pairs to read

 @returns Their ids

 @example
 ```ts
 const ids = idsOf({ pairs, },);
 ```
 */
function idsOf(
  { pairs, }: { readonly pairs: readonly { readonly id: string; }[]; },
): readonly string[] {
  return pairs.map(function toId(pair,): string {
    return pair.id;
  },);
}

await describe({
  name: selectPendingEntries.name,
  concurrency: 1,
  children: [
    it({
      name: 'CHOOSES EVERY COMPLETE PAIR in the corpus, hands them on as the pairs its walk over every entry '
        + 'found, and says nothing when nothing is restricted or missing',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: COMPLETE, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-select-', },);

        /**
         What the pass would run, and the complete pairs its walk found.
         */
        const selected = await selectPendingEntries({
          line: lineOf({
            command: 'corpus-pass',
            typed: [],
          },),
          pin: clone,
          done: new Set<string>(),
          attempts: new Map(),
          sliceCacheDir: scratch.path,
        },);

        expect(idsOf({ pairs: selected.pending, },),).toEqual([
          'biscuit',
          'tabby',
        ],);
        expect(selected.pairs,).toEqual({
          walked: 'every-entry',
          ids: new Set(idsOf({ pairs: COMPLETE, },),),
        },);
        expect(printed.lines,).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES OUT AN ENTRY FINISHED BEFORE and one with no English page, naming the entry with no English '
        + 'page, and hands on the finished entry among the complete pairs its walk found but not the entry with '
        + 'no English page',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [
          ...COMPLETE,
          {
            id: 'mittens',
            sourceText: SMALL_PAGE,
          },
        ], },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-select-', },);

        /**
         What the pass would run, and the complete pairs its walk found.
         */
        const selected = await selectPendingEntries({
          line: lineOf({
            command: 'corpus-pass',
            typed: [],
          },),
          pin: clone,
          done: new Set(['biscuit',],),
          attempts: new Map(),
          sliceCacheDir: scratch.path,
          wallClock: fixedWallClock,
        },);

        expect(idsOf({ pairs: selected.pending, },),).toEqual(['tabby',],);
        expect(selected.pairs,).toEqual({
          walked: 'every-entry',
          ids: new Set(idsOf({ pairs: COMPLETE, },),),
        },);
        // THE CLOCK TIME AND NO ATTEMPT: none has begun for an entry whose page is absent (ledger B960).
        expect(printed.lines,).toEqual([
          `INCOMPLETE mittens ${AT}: target page absent at the pin (corpus read failed for ${clone.commitSha}:`
            + 'people/mittens/page.en.md (missing-object); the commit has no such path: check the path, or pin a '
            + 'commit that has it.)',
        ],);
      },
    },),
    it({
      name: 'CHOOSES ONLY THE ENTRIES THE COMMAND LINE NAMES and says they were chosen by hand and run in the '
        + 'pass\'s own order, naming them in code point order, and hands on the pairs its walk over the named '
        + 'entries alone found',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: COMPLETE, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-select-', },);

        /**
         What the pass would run, and the complete pairs its walk found.
         */
        const selected = await selectPendingEntries({
          line: lineOf({
            command: 'corpus-pass',
            typed: [
              '--only',
              'tabby',
            ],
          },),
          pin: clone,
          done: new Set<string>(),
          attempts: new Map(),
          sliceCacheDir: scratch.path,
        },);

        expect(idsOf({ pairs: selected.pending, },),).toEqual(['tabby',],);
        expect(selected.pairs,).toEqual({
          walked: 'named-entries',
          ids: new Set(['tabby',],),
        },);
        expect(printed.lines,).toEqual([
          'ONLY tabby (chosen by hand in place of every pending pair at the pin; if still pending it runs in '
            + 'the pass\'s own order; run this into a throwaway TRANSLATION_REPAIR_RUNS_DIR so a hand-picked '
            + 'entry never enters a pool later draws treat as natural accumulation)',
        ],);
      },
    },),
    it({
      name: 'SAYS ONLY THE NAMED ENTRIES STILL PENDING RUN where the command line names one finished before beside '
        + 'one pending, and runs the pending one alone',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: COMPLETE, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-select-', },);

        /**
         What the pass would run, and the complete pairs its walk found.
         */
        const selected = await selectPendingEntries({
          line: lineOf({
            command: 'corpus-pass',
            typed: [
              '--only',
              'biscuit,tabby',
            ],
          },),
          pin: clone,
          done: new Set(['biscuit',],),
          attempts: new Map(),
          sliceCacheDir: scratch.path,
        },);

        expect(idsOf({ pairs: selected.pending, },),).toEqual(['tabby',],);
        expect(selected.pairs,).toEqual({
          walked: 'named-entries',
          ids: new Set(idsOf({ pairs: COMPLETE, },),),
        },);
        expect(printed.lines,).toEqual([
          'ONLY biscuit,tabby (chosen by hand in place of every pending pair at the pin; those still pending '
            + 'run in the pass\'s own order; run this into a throwaway TRANSLATION_REPAIR_RUNS_DIR so a hand-picked '
            + 'entry never enters a pool later draws treat as natural accumulation)',
        ],);
      },
    },),
    it({
      name: 'COUNTS A FINISHED ENTRY WHOSE ENGLISH PAGE THE PIN LACKS AS NO COMPLETE PAIR, and names no gap for '
        + 'it, since it runs no more',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [
          ...COMPLETE,
          {
            id: 'whiskers',
            sourceText: SMALL_PAGE,
          },
        ], },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-select-', },);

        /**
         What the pass would run, and the complete pairs its walk found.
         */
        const selected = await selectPendingEntries({
          line: lineOf({
            command: 'corpus-pass',
            typed: [],
          },),
          pin: clone,
          done: new Set([
            'biscuit',
            'whiskers',
          ],),
          attempts: new Map(),
          sliceCacheDir: scratch.path,
        },);

        expect(idsOf({ pairs: selected.pending, },),).toEqual(['tabby',],);
        expect(selected.pairs,).toEqual({
          walked: 'every-entry',
          ids: new Set(idsOf({ pairs: COMPLETE, },),),
        },);
        expect(printed.lines,).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES AN ENTRY THE CORPUS DOES NOT HOLD after saying which entries it was restricted to',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: COMPLETE, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-select-', },);

        await expect(selectPendingEntries({
          line: lineOf({
            command: 'corpus-pass',
            typed: [
              '--only',
              'nobody',
            ],
          },),
          pin: clone,
          done: new Set<string>(),
          attempts: new Map(),
          sliceCacheDir: scratch.path,
        },),).rejects.toThrow('--only asks for "nobody", which the corpus at the pin does not hold',);
        expect(printed.lines.length,).toBe(1,);
      },
    },),
    it({
      name: 'RESUMES AN ENTRY HOLDING CACHED SLICES FIRST, ahead of one that comes earlier in the corpus',
      fn: async () => {
        await using clone = await makeCorpusPassClone({ entries: COMPLETE, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-select-', },);
        await mkdir(join(
          scratch.path,
          'tabby',
        ),);
        await writeFile(
          join(
            scratch.path,
            'tabby',
            '0.json',
          ),
          '{}',
        );

        /**
         What the pass would run, and the complete pairs its walk found.
         */
        const selected = await selectPendingEntries({
          line: lineOf({
            command: 'corpus-pass',
            typed: [],
          },),
          pin: clone,
          done: new Set<string>(),
          attempts: new Map(),
          sliceCacheDir: scratch.path,
        },);

        expect(idsOf({ pairs: selected.pending, },),).toEqual([
          'tabby',
          'biscuit',
        ],);
      },
    },),
    it({
      name: 'PUTS AN ENTRY TRIED BEFORE BEHIND ONE NOT YET TRIED of its band, from the counts it is handed',
      fn: async () => {
        await using clone = await makeCorpusPassClone({ entries: COMPLETE, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-select-', },);

        /**
         What the pass would run, and the complete pairs its walk found.
         */
        const selected = await selectPendingEntries({
          line: lineOf({
            command: 'corpus-pass',
            typed: [],
          },),
          pin: clone,
          done: new Set<string>(),
          attempts: new Map([
            [
              'biscuit',
              4,
            ],
          ],),
          sliceCacheDir: scratch.path,
        },);

        expect(idsOf({ pairs: selected.pending, },),).toEqual([
          'tabby',
          'biscuit',
        ],);
      },
    },),
    it({
      name: 'RUNS THE ENTRIES --only NAMES IN THE PASS\'S OWN ORDER, resuming one holding cached slices ahead of one '
        + 'the command line and the corpus both name first',
      fn: async () => {
        await using clone = await makeCorpusPassClone({ entries: COMPLETE, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-select-', },);
        await mkdir(join(
          scratch.path,
          'tabby',
        ),);
        await writeFile(
          join(
            scratch.path,
            'tabby',
            '0.json',
          ),
          '{}',
        );

        /**
         What the pass would run when the command line names both entries.
         */
        const selected = await selectPendingEntries({
          line: lineOf({
            command: 'corpus-pass',
            typed: [
              '--only',
              'biscuit,tabby',
            ],
          },),
          pin: clone,
          done: new Set<string>(),
          attempts: new Map(),
          sliceCacheDir: scratch.path,
        },);

        expect(idsOf({ pairs: selected.pending, },),).toEqual([
          'tabby',
          'biscuit',
        ],);
      },
    },),
  ],
},);
