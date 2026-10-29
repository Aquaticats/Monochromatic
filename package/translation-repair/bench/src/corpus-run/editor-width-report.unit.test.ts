/**
 Tests for the width probe's written report.

 THIS MODULE IS WHY THE COVERAGE MEASURE WAS REBUILT. It was the one exported
 function in the package whose module no test path reached, and it is live:
 `editor-width-probe.ts` calls it, and that probe is an operator entry script
 no test imports, so nothing carried a test to it. See
 `doc/planning/translation-repair-coverage-measure.md`.

 THE CASE WORTH THE FILE IS THE FAILED POSITIVE CONTROL. Before a width reading
 means anything, the panel has to be shown able to prefer intact text over the
 same text with a sentence removed. When it cannot, every count in the report is
 unreadable, and a report that printed them in the same voice either way would
 launder a broken instrument into a result. That is the exact failure this
 package keeps finding, so the report has to SAY the numbers are unreadable
 rather than merely omit a tick.

 THE SECOND CASE IS ABOUT NOT LOSING A READING. The sample is split in two so a
 result landing near its own null band has a second, untouched half available.
 That only holds if draw B lands beside draw A rather than on top of it, and a
 report writer keying both to one filename would destroy the very thing the
 split exists to preserve, silently, after the calls are paid for.

 Rows are invented. They carry no passage text by construction: `WidthRow`
 keeps counts and verdicts precisely because the corpus is unlicensed.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { dirname, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type WidthDraw,
  type WidthRow,
  writeWidthReport,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { runsDirPointedAt, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/runs-dir-pointed.test-fixture.ts';

//region Editor width report tests

/**
 Narrow roster the fixture reports on.
 */
const NARROW: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
];

/**
 Wide roster the fixture reports on.
 */
const WIDE: readonly RosterModelId[] = [
  ...NARROW,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
];

/**
 Panel, held fixed, as the probe holds it.
 */
const PANEL: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Commit the report stamps.
 */
const HEAD_SHA = 'f00dcafe1234';

/**
 Builds one row, varying only the two bits the paired reading is computed
 from.

 @param sliceIndex - position within the entry

 @param moved - whether the arms shipped different text

 @param churned - whether the narrow arm run twice disagreed with itself

 @returns Row shaped as one slice contributes

 @example
 ```ts
 const row = rowOf({ sliceIndex: 0, moved: true, churned: false, },);
 ```
 */
function rowOf(
  {
    sliceIndex,
    moved,
    churned,
  }: {
    readonly sliceIndex: number;
    readonly moved: boolean;
    readonly churned: boolean;
  },
): WidthRow {
  return {
    entryId: 'whiskers',
    sliceIndex,
    acceptedIssues: 1,
    comparison: moved ? 'differs' : 'same-text',
    heardNarrow: NARROW.length,
    heardWide: WIDE.length,
    narrowShipped: true,
    wideShipped: true,
    narrowRepeatAgreed: !churned,
    verdict: moved ? 'position-decided' : 'not-run',
    usableBallots: moved ? 4 : 0,
    narrowProducers: NARROW,
    wideProducers: WIDE,
  };
}

/**
 Rows where one slice moved without churning and two churned without moving,
 so the paired counts differ from each other and from the raw totals.
 */
const ROWS: readonly WidthRow[] = [
  rowOf({
    sliceIndex: 0,
    moved: true,
    churned: false,
  },),
  rowOf({
    sliceIndex: 1,
    moved: false,
    churned: true,
  },),
  rowOf({
    sliceIndex: 2,
    moved: false,
    churned: true,
  },),
];

/**
 Refusals that left slices with no work, as the probe tallies them.
 */
const SKIPPED: Readonly<Record<string, number>> = { 'no accepted issue': 5, };

/**
 Start of each case's runs directory name, so a leftover shows this file
 made it.
 */
const RUNS_PREFIX = 'whiskers-width-report-';

/**
 Writes one report and reads back what landed on disk.

 @param controlHeld - whether the panel passed its own positive control

 @param draw - which half of the split sample this run spent

 @returns Path written and the text at it

 @example
 ```ts
 const written = await reportFor({ controlHeld: true, draw: 'a', },);
 ```
 */
async function reportFor(
  {
    controlHeld,
    draw,
    rows = ROWS,
  }: {
    readonly controlHeld: boolean;
    readonly draw: WidthDraw;
    readonly rows?: readonly WidthRow[];
  },
): Promise<{
  readonly path: string;
  readonly text: string;
}> {
  await using runs = await scratchDir({ prefix: RUNS_PREFIX, },);

  /**
   Where the writer says it put the report.
   */
  const path = await writeWidthReport({
    runsDir: runs.path,
    rows,
    skipped: SKIPPED,
    headSha: HEAD_SHA,
    narrowEditorIds: NARROW,
    wideEditorIds: WIDE,
    judgeModelIds: PANEL,
    controlHeld,
    draw,
  },);

  return {
    path,
    text: await readFile(
      path,
      'utf8',
    ),
  };
}

/**
 Writes BOTH draws into ONE runs directory, in sequence.

 SHARING THE DIRECTORY IS THE WHOLE POINT. Writing each draw into its own
 throwaway directory would make the two paths differ by directory no matter
 what the writer named them, so the case asking whether draw B lands on top of
 draw A would pass against a writer that gave both the same filename.

 @returns Both reports, draw A first

 @example
 ```ts
 const [drawA, drawB,] = await bothDraws();
 ```
 */
async function bothDraws(): Promise<readonly {
  readonly path: string;
  readonly text: string;
}[]> {
  await using runs = await scratchDir({ prefix: RUNS_PREFIX, },);

  /**
   Draws to write, in order, into the one directory.
   */
  const draws: readonly WidthDraw[] = [
    'a',
    'b',
  ];

  /**
   What each draw wrote.
   */
  const written: {
    path: string;
    text: string;
  }[] = [];

  /* oxlint-disable no-await-in-loop -- sequential on purpose: both draws share one runs directory, and writing them at once would race on the filename this case is about */
  for (const draw of draws) {
    /**
     Where this draw put its report.
     */
    const path = await writeWidthReport({
      runsDir: runs.path,
      rows: ROWS,
      skipped: SKIPPED,
      headSha: HEAD_SHA,
      narrowEditorIds: NARROW,
      wideEditorIds: WIDE,
      judgeModelIds: PANEL,
      controlHeld: true,
      draw,
    },);

    written.push({
      path,
      text: await readFile(
        path,
        'utf8',
      ),
    },);
  }
  /* oxlint-enable no-await-in-loop */

  return written;
}

await describe({
  name: writeWidthReport.name,
  children: [
    it({
      name: 'WRITES INTO THE RUNS DIRECTORY IT IS HANDED, not the one the environment names',
      fn: async () => {
        await using elsewhere = await scratchDir({ prefix: RUNS_PREFIX, },);
        using pointed = runsDirPointedAt({ path: elsewhere.path, },);
        await using runs = await scratchDir({ prefix: RUNS_PREFIX, },);
        /**
         Where the writer says it put the report.
         */
        const path = await writeWidthReport({
          runsDir: runs.path,
          rows: ROWS,
          skipped: SKIPPED,
          headSha: HEAD_SHA,
          narrowEditorIds: NARROW,
          wideEditorIds: WIDE,
          judgeModelIds: PANEL,
          controlHeld: true,
          draw: 'a',
        },);
        expect(dirname(path,),).toBe(runs.path,);
      },
    },),
    it({
      name: 'SAYS THE COUNTS ARE UNREADABLE when the panel failed its own positive control, '
        + 'rather than printing them in the same voice as a sound reading',
      fn: async () => {
        // An instrument that cannot see a deleted sentence cannot see the finer
        // difference the draw is asking about. Printing its counts without
        // saying so is how a broken instrument becomes a published result.
        /**
         Report written after the control failed.
         */
        const written = await reportFor({
          controlHeld: false,
          draw: 'a',
        },);

        expect(written.text.includes('THE PANEL DID NOT PREFER INTACT TEXT',),).toBe(true,);
        expect(written.text.includes('Every number in this report is unreadable',),).toBe(true,);
      },
    },),
    it({
      name: 'REPORTS a held control as held, so the warning discriminates instead of always firing',
      fn: async () => {
        // The positive control for the "SAYS THE COUNTS ARE UNREADABLE" case: a
        // report that always warned would pass that one and fail this.
        /**
         Report written after the control held.
         */
        const written = await reportFor({
          controlHeld: true,
          draw: 'a',
        },);

        expect(written.text.includes('THE PANEL DID NOT PREFER INTACT TEXT',),).toBe(false,);
        expect(written.text.includes('the numbers in this report are worth reading',),).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS THE TWO DRAWS IN SEPARATE FILES, so the held-back half cannot land on top of '
        + 'the reading it exists to be compared against',
      fn: async () => {
        /**
         Where each draw put its report, both inside ONE runs directory.
         */
        const [drawA, drawB,] = await bothDraws();

        expect(drawA?.path,).not.toBe(drawB?.path,);
      },
    },),
    it({
      name: 'TELLS each draw what the other half is for, since reading B instead of A rather '
        + 'than beside it throws away the split',
      fn: async () => {
        /**
         Reports from both halves.
         */
        const [drawA, drawB,] = await bothDraws();

        expect(drawA?.text.includes('The other half is untouched',),).toBe(true,);
        expect(drawB?.text.includes('read it beside draw A rather than instead of it',),)
          .toBe(true,);
      },
    },),
    it({
      name: 'PRINTS THE PAIRED READING, not only the move count, because a move count inside '
        + 'the null band is not evidence that widening did anything',
      fn: async () => {
        /**
         Report over rows where one slice moved and two churned.
         */
        const written = await reportFor({
          controlHeld: true,
          draw: 'a',
        },);

        // One row moved without churning, two churned without moving. Both
        // counts have to appear and they have to disagree, or the reader cannot
        // do the comparison the report tells them to do.
        expect(written.text.includes('moved WITHOUT churning',),).toBe(true,);
        expect(written.text.includes('churned without moving',),).toBe(true,);
        expect(written.text.includes('READ THE MOVE COUNT AGAINST THE NULL BAND',),).toBe(true,);
      },
    },),
    it({
      name: 'COUNTS the rows it was given, so the summary describes this draw and not a stale one',
      fn: async () => {
        /**
         Report over three rows.
         */
        const written = await reportFor({
          controlHeld: true,
          draw: 'a',
        },);

        expect(written.text.includes(`slices that produced a row: ${String(ROWS.length,)}`,),)
          .toBe(true,);
        expect(written.text.includes('moved WITHOUT churning, so widening changed what the '
          + 'lane would not have: 1',),).toBe(true,);
        expect(written.text.includes('churned without moving, so the lane changed its own '
          + 'mind alone: 2',),).toBe(true,);
      },
    },),
    it({
      name: 'NAMES the rosters and the commit, so a report cannot be read against the wrong run',
      fn: async () => {
        /**
         Report carrying its own provenance.
         */
        const written = await reportFor({
          controlHeld: true,
          draw: 'a',
        },);

        expect(written.text.includes(HEAD_SHA,),).toBe(true,);
        for (const seat of WIDE) {
          expect(written.text.includes(seat,),).toBe(true,);
        }
        for (const seat of PANEL) {
          expect(written.text.includes(seat,),).toBe(true,);
        }
      },
    },),
    it({
      name: 'LISTS the slices that carried no work, so a draw thinned by refusals is visible '
        + 'rather than merely small',
      fn: async () => {
        /**
         Report carrying the refusal tally.
         */
        const written = await reportFor({
          controlHeld: true,
          draw: 'a',
        },);

        expect(written.text.includes('no accepted issue: 5',),).toBe(true,);
      },
    },),

    it({
      name: 'RENDERS each row\'s narrow arm flag before its wide arm flag, as y/y where both shipped, n/n '
        + 'where neither did, y/n where only the narrow arm did and n/y where only the wide arm did',
      fn: async () => {
        const written = await reportFor({
          controlHeld: true,
          draw: 'a',
          rows: [{
            ...rowOf({ sliceIndex: 0, moved: true, churned: true, },),
            narrowShipped: true,
            wideShipped: true,
          }, {
            ...rowOf({ sliceIndex: 1, moved: false, churned: false, },),
            narrowShipped: false,
            wideShipped: false,
          }, {
            ...rowOf({ sliceIndex: 2, moved: false, churned: false, },),
            narrowShipped: true,
            wideShipped: false,
          }, {
            ...rowOf({ sliceIndex: 3, moved: false, churned: false, },),
            narrowShipped: false,
            wideShipped: true,
          },],
        },);
        /**
         Where the row of each slice opens, as far as its shipped flags.
         */
        const rowStart = `heard ${String(NARROW.length,)}/${String(WIDE.length,)}, shipped `;
        expect(written.text.includes(`\`whiskers\` slice 0, issues 1, ${rowStart}y/y, `,),).toBe(true,);
        expect(written.text.includes(`\`whiskers\` slice 1, issues 1, ${rowStart}n/n, `,),).toBe(true,);
        expect(written.text.includes(`\`whiskers\` slice 2, issues 1, ${rowStart}y/n, `,),).toBe(true,);
        expect(written.text.includes(`\`whiskers\` slice 3, issues 1, ${rowStart}n/y, `,),).toBe(true,);
      },
    },),
  ],
  concurrency: 1,
},);

//endregion Editor width report tests
