/**
 Tests for how a persisted rendering audit reads on a terminal.

 THESE NUMBERS EXIST TO BE QUOTED INTO A DOCUMENT, which is what makes their
 wording load-bearing rather than cosmetic. The module says so itself: a count
 with no denominator is the single most quotable wrong number a telemetry probe
 can emit. Every case in this file therefore asserts the denominator beside the count,
 not just that a number reached the page.

 `printBand` CARRIES THE ONE REAL DECISION IN THE FILE. A band of zero over
 zero pairs and a band of zero over forty pairs are opposite findings, and a
 row of zeroes reads as the second. So an empty band says NOTHING PAIRED and
 prints no numbers at all, and both halves of that get a case: the refusal must
 appear, and the numbers must not.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`. `describe` runs children concurrently by default, and two
 cases swapping the same reporter interleave: one reads lines a sibling wrote,
 or reads none because a sibling already restored the real one. The runner
 spawns a process per test file, so nothing outside this file is touched and
 nothing inside it may overlap.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type AudienceSplit,
  type AuditRelocationPair,
  type AuditRepeatBand,
  type PageRelationTally,
  printBand,
  printRelations,
  printRelocations,
  printSplit,
  printVoices,
  type VoiceRate,
} from '../../dist/final/node/index.mjs';

//region Settled audit printing tests

/**
 Auditor whose rate these fixtures carry.
 */
const AUDITOR = 'cat-house/tabbyscribe-2';

/**
 Second auditor, so a column has more than one row to line up.
 */
const OTHER_AUDITOR = 'cat-house/mouser-mini';

/**
 Run set the relocation fixtures belong to.
 */
const RUN_SET = 'naptime-20260825';

/**
 Collects what would have gone to stdout, restoring the real one on disposal.

 @param lines - collector the caller reads afterwards

 @returns Collected lines, and the restore that disposal runs

 @example
 ```ts
 using printed = collectingLines({ lines: [], },);
 ```
 */
function collectingLines(
  { lines, }: { readonly lines: string[]; },
): { readonly lines: readonly string[]; } & Disposable {
  /**
   Real reporter, put back on disposal.
   */
  const reported = console.log;

  console.log = (...parts: readonly unknown[]) => {
    lines.push(parts.map(String,)
      .join(' ',),);
  };
  return {
    lines,
    [Symbol.dispose]: () => {
      console.log = reported;
    },
  };
}

/**
 Builds one half of the audited population.

 @param audits - which text this half was audited against

 @returns Half, summed

 @example
 ```ts
 const split = half({ audits: 'archive', },);
 ```
 */
function half(
  { audits, }: { readonly audits: 'archive' | 'fresh'; },
): AudienceSplit {
  return {
    audits,
    subjects: 40,
    claimed: 63,
    subjectsWithClaims: 27,
    corroborated: 19,
    agreed: 12,
    near: 5,
    degraded: 2,
  };
}

/**
 Builds a band over a stated number of paired texts.

 @param pairs - how many texts were audited twice

 @returns Band carrying that many pairs

 @example
 ```ts
 const band = spread({ pairs: 0, },);
 ```
 */
function spread(
  { pairs, }: { readonly pairs: number; },
): AuditRepeatBand {
  return {
    pairs,
    agreedExactly: 9,
    widest: 4,
    totalGap: 11,
    silentOnOneSide: 3,
    leftClaimed: 31,
    rightClaimed: 28,
    leftCorroborated: 14,
    rightCorroborated: 13,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: printSplit.name,
      children: [
        it({
          name: 'NAMES the archive half as archive, with every count beside its label',
          fn: async () => {
            using printed = collectingLines({ lines: [], },);

            printSplit({ split: half({ audits: 'archive', },), },);

            /**
             The one line this printer writes.
             */
            const line = printed.lines[0] ?? '';

            expect(printed.lines.length,).toBe(1,);
            expect(line.includes('ARCHIVE text',),).toBe(true,);
            expect(line.includes('subjects=40',),).toBe(true,);
            expect(line.includes('drew a claim=27',),).toBe(true,);
            expect(line.includes('claims=63',),).toBe(true,);
            expect(line.includes('corroborated=19',),).toBe(true,);
            expect(line.includes('agreed=12',),).toBe(true,);
            expect(line.includes('near=5',),).toBe(true,);
            expect(line.includes('degraded=2',),).toBe(true,);
          },
        },),
        it({
          name: 'NAMES the other half as fresh, so two halves cannot be read as one',
          fn: async () => {
            using printed = collectingLines({ lines: [], },);

            printSplit({ split: half({ audits: 'fresh', },), },);

            /**
             The one line this printer writes.
             */
            const line = printed.lines[0] ?? '';

            expect(line.includes('FRESH',),).toBe(true,);
            expect(line.includes('ARCHIVE',),).toBe(false,);
          },
        },),
      ],
      concurrency: 1,
    },),

    describe({
      name: printRelations.name,
      children: [
        it({
          name: 'PRINTS one row per relation, each with both denominators',
          fn: async () => {
            using printed = collectingLines({ lines: [], },);

            /**
             Two relations, one of them the longest label the reader can produce.
             */
            const tallies: readonly PageRelationTally[] = [
              {
                label: 'displaced',
                subjects: 7,
                claimed: 11,
              },
              {
                label: 'silent:contest-declined-and-archive-silent',
                subjects: 3,
                claimed: 0,
              },
            ];

            printRelations({ tallies, },);

            /**
             Everything the printer said, as one body to search.
             */
            const said = printed.lines.join('\n',);

            expect(said.includes('displaced',),).toBe(true,);
            expect(said.includes('subjects=7',),).toBe(true,);
            expect(said.includes('claims=11',),).toBe(true,);
            expect(said.includes('silent:contest-declined-and-archive-silent',),).toBe(true,);
            expect(said.includes('subjects=3',),).toBe(true,);
          },
        },),
        it({
          name: 'KEEPS the note that an undecided subject is waiting, not overruled',
          fn: async () => {
            using printed = collectingLines({ lines: [], },);

            printRelations({ tallies: [], },);

            /**
             Everything the printer said, as one body to search.
             */
            const said = printed.lines.join('\n',);

            // The note is the difference between a reader treating an undecided
            // subject as a finding and treating it as work not yet done, so it is
            // printed even when no relation was tallied.
            expect(said.includes('waiting on a decision, not overruled',),).toBe(true,);
          },
        },),
      ],
      concurrency: 1,
    },),

    describe({
      name: printVoices.name,
      children: [
        it({
          name: 'PRINTS one row per auditor, keeping asked, answered and the loss between them apart '
            + 'from spoke, since asked= used to mean answered and per-model loss was unreadable',
          fn: async () => {
            using printed = collectingLines({ lines: [], },);

            /**
             Two auditors, one of which was lost on two subjects and quiet on two
             more.
             */
            const rates: readonly VoiceRate[] = [
              {
                modelId: AUDITOR,
                asked: 40,
                answered: 38,
                spoke: 36,
                claims: 31,
                dropped: 2,
              },
              {
                modelId: OTHER_AUDITOR,
                asked: 40,
                answered: 40,
                spoke: 40,
                claims: 12,
                dropped: 0,
              },
            ];

            printVoices({ rates, },);

            /**
             Everything the printer said, as one body to search.
             */
            const said = printed.lines.join('\n',);

            expect(said.includes(AUDITOR,),).toBe(true,);
            expect(said.includes(OTHER_AUDITOR,),).toBe(true,);
            expect(said.includes('asked=40 answered=38 lost=2 spoke on=36 claims=31 dropped=2',),).toBe(true,);
            expect(said.includes('asked=40 answered=40 lost=0 spoke on=40 claims=12 dropped=0',),).toBe(true,);
          },
        },),
        it({
          name: 'PRINTS its heading even when no auditor answered at all',
          fn: async () => {
            using printed = collectingLines({ lines: [], },);

            printVoices({ rates: [], },);

            expect(printed.lines.length,).toBe(1,);
            expect((printed.lines[0] ?? '').includes('WHAT EACH AUDITOR THOUGHT WAS WORTH A CLAIM',),)
              .toBe(true,);
          },
        },),
      ],
      concurrency: 1,
    },),

    describe({
      name: printRelocations.name,
      children: [
        it({
          name: 'COUNTS the candidates in its heading, claim pairs and slice pairs apart, then names '
            + 'each one',
          fn: async () => {
            using printed = collectingLines({ lines: [], },);

            /**
             One omission paired with one addition elsewhere in the same entry.
             */
            const pairs: readonly AuditRelocationPair[] = [
              {
                runSet: RUN_SET,
                entryId: 'whiskers',
                omissionAt: 3,
                additionAt: 7,
                omissionReason: 'the passage about the windowsill is gone',
                additionReason: 'the passage about the windowsill appears here',
              },
            ];

            printRelocations({ pairs, },);

            /**
             Everything the printer said, as one body to search.
             */
            const said = printed.lines.join('\n',);

            expect(said.includes('RELOCATION CANDIDATES: claim pairs=1 slice pairs=1',),).toBe(true,);
            expect(said.includes(`${RUN_SET}/whiskers`,),).toBe(true,);
            expect(said.includes('omission at 3 <-> addition at 7',),).toBe(true,);
          },
        },),
        it({
          name: 'PRINTS FEWER SLICE PAIRS THAN CLAIM PAIRS when two voices filed on one move, so the '
            + 'quotable number is the move and not the number of voices that saw it',
          fn: async () => {
            using printed = collectingLines({ lines: [], },);

            /**
             Two voices' claims about one relocation.
             */
            const pairs: readonly AuditRelocationPair[] = [
              {
                runSet: RUN_SET,
                entryId: 'whiskers',
                omissionAt: 3,
                additionAt: 4,
                omissionReason: 'the passage about the windowsill is gone',
                additionReason: 'the passage about the windowsill appears here',
              },
              {
                runSet: RUN_SET,
                entryId: 'whiskers',
                omissionAt: 3,
                additionAt: 4,
                omissionReason: 'the windowsill line is missing',
                additionReason: 'the windowsill line was added',
              },
            ];

            printRelocations({ pairs, },);

            expect((printed.lines[0] ?? '').includes('claim pairs=2 slice pairs=1',),).toBe(true,);
          },
        },),
        it({
          name: 'SAYS zero rather than staying silent when nothing paired',
          fn: async () => {
            using printed = collectingLines({ lines: [], },);

            printRelocations({ pairs: [], },);

            expect(printed.lines.length,).toBe(1,);
            expect((printed.lines[0] ?? '').includes('RELOCATION CANDIDATES: claim pairs=0 slice pairs=0',),)
              .toBe(true,);
          },
        },),
      ],
      concurrency: 1,
    },),

    describe({
      name: printBand.name,
      children: [
        it({
          name: 'REFUSES to print a band over nothing, saying so in words',
          fn: async () => {
            using printed = collectingLines({ lines: [], },);

            printBand({
              band: spread({ pairs: 0, },),
              over: 'texts audited twice inside this run',
            },);

            /**
             Everything the printer said, as one body to search.
             */
            const said = printed.lines.join('\n',);

            expect(said.includes('NOTHING PAIRED',),).toBe(true,);
            expect(said.includes('No band is quotable',),).toBe(true,);
          },
        },),
        it({
          name: 'PRINTS no numbers at all for an empty band, since zeroes read as a finding',
          fn: async () => {
            // The other half of the same decision, and the half that would break
            // silently: a printer that added the numbers back below the refusal
            // would still pass the "REFUSES to print a band over nothing" case while
            // publishing the row of zeroes
            // that the refusal exists to prevent.
            using printed = collectingLines({ lines: [], },);

            printBand({
              band: spread({ pairs: 0, },),
              over: 'texts audited twice inside this run',
            },);

            /**
             Everything the printer said, as one body to search.
             */
            const said = printed.lines.join('\n',);

            expect(said.includes('pairs=',),).toBe(false,);
            expect(said.includes('widest gap=',),).toBe(false,);
            expect(said.includes('claims 31 against 28',),).toBe(false,);
          },
        },),
        it({
          name: 'PRINTS every reading once the band is over something',
          fn: async () => {
            using printed = collectingLines({ lines: [], },);

            printBand({
              band: spread({ pairs: 22, },),
              over: 'texts audited twice inside this run',
            },);

            /**
             Everything the printer said, as one body to search.
             */
            const said = printed.lines.join('\n',);

            expect(said.includes('NOTHING PAIRED',),).toBe(false,);
            expect(said.includes('pairs=22',),).toBe(true,);
            expect(said.includes('same claim count=9',),).toBe(true,);
            expect(said.includes('widest gap=4',),).toBe(true,);
            expect(said.includes('total gap=11',),).toBe(true,);
            expect(said.includes('silent on one side only=3',),).toBe(true,);
            expect(said.includes('claims 31 against 28',),).toBe(true,);
            expect(said.includes('corroborated 14 against 13',),).toBe(true,);
          },
        },),
        it({
          name: 'NAMES what the band is over, so two bands cannot be confused',
          fn: async () => {
            using printed = collectingLines({ lines: [], },);

            printBand({
              band: spread({ pairs: 22, },),
              over: 'texts audited in two different runs',
            },);

            expect((printed.lines[0] ?? '')
              .includes('INSTRUMENT BAND over texts audited in two different runs',),).toBe(true,);
          },
        },),
      ],
      concurrency: 1,
    },),
  ],
},);

//endregion Settled audit printing tests
