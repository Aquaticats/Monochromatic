import {
  type DisposableSandbox,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type GatheredProbe,
  reportProbeTelemetry,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

//region Probe telemetry report tests
// What the PROBE report says when its denominator is empty.
//
// The report prints COUNTS, never wording, so every figure it prints is a small
// integer and no fixture here carries text a document could contain.

/**
 A roster summary with nothing recorded, for cases that do not turn on it.
 */
const SILENT_ROSTER = {
  offered: 0,
  degraded: 0,
  silent: 0,
};

/**
 Builds a gathered run whose counts are exactly the ones a case cares about.

 Every other member is empty, because the note under test reads only the
 repair-lane record count and the report's remaining lines are covered by the
 figures they print.

 @param repairShippedRecords - repair-lane records the run holds

 @param editorOffered - slices the editor was asked to repair

 @returns Gathered probe carrying those counts and nothing else

 @example
 ```ts
 const gathered = gatheredWith({ repairShippedRecords: 0, editorOffered: 4, },);
 ```
 */
function gatheredWith(
  {
    repairShippedRecords,
    editorOffered,
  }: {
    readonly repairShippedRecords: number;
    readonly editorOffered: number;
  },
): GatheredProbe {
  return {
    readings: [],
    byIssueId: new Map(),
    refinedIssueIds: new Set(),
    refinementReadings: [],
    editorRoster: {
      ...SILENT_ROSTER,
      offered: editorOffered,
    },
    refineRoster: SILENT_ROSTER,
    entriesWithRewrites: 0,
    entries: 1,
    repairShippedRecords,
    repairUnprobedRecords: 0,
  };
}

/**
 Reports one gathered run with every printed line collected instead of shown.

 @param gathered - run to report on

 @param sinon - calling case's own sandbox (`ctx.sinon`), threaded through to
 the console capture

 @returns Every line the report printed, in order

 @example
 ```ts
 const lines = reportedLines({ gathered, sinon: ctx.sinon, },);
 ```
 */
function reportedLines(
  {
    gathered,
    sinon,
  }: {
    readonly gathered: GatheredProbe;
    readonly sinon: DisposableSandbox;
  },
): readonly string[] {
  using capture = divertingConsoleLog({ sinon, },);
  reportProbeTelemetry({ gathered, },);

  // Read before disposal returns the real logger, so a case never asserts on a
  // capture that is still installed.
  return capture.lines;
}

/**
 Text identifying the note under test, short enough to survive rewording.
 */
const ZERO_NOTE = 'NOTE repairShippedRecords=0';

/**
 Text identifying the degraded-roster note.
 */
const DEGRADED_NOTE = 'NOTE editorDegraded';

/**
 Text identifying the silent-refiner note.
 */
const SILENT_NOTE = 'NOTE refineSilent';

/**
 Whether any printed line opens with a note.

 @param lines - every line a report printed

 @param note - text the note's line opens with

 @returns True where a line opens with the note

 @example
 ```ts
 const printed = opensWithNote({ lines, note: DEGRADED_NOTE, },);
 ```
 */
function opensWithNote(
  {
    lines,
    note,
  }: {
    readonly lines: readonly string[];
    readonly note: string;
  },
): boolean {
  return lines.some(function opensWith(line,): boolean {
    return line.startsWith(note,);
  },);
}

await describe({
  name: reportProbeTelemetry.name,
  // ONE AT A TIME: cases divert the process-wide `console.log` (ledger B79).
  concurrency: 1,
  children: [
    it({
      name:
        'NAMES AN EMPTY DENOMINATOR rather than letting seven zeros read as a clean result. With no '
          + 'repair-lane record to probe, every PROBE and CLAIMS figure is zero by construction, and '
          + 'majorityIntroduced=0 then says the probe found nothing wrong when it found nothing at all',
      fn: async (ctx) => {
        /**
         Report over a run holding no repair-lane records.
         */
        const lines = reportedLines({
          gathered: gatheredWith({
            repairShippedRecords: 0,
            editorOffered: 4,
          },),
          sinon: ctx.sinon,
        },);

        expect(lines.some(function carriesNote(line,): boolean {
          return line.startsWith(ZERO_NOTE,);
        },),).toBe(true,);
      },
    },),

    it({
      name:
        'STAYS SILENT ONCE THERE IS SOMETHING TO PROBE, so the note marks the empty case rather '
          + 'than riding along on every report and teaching readers to skip it',
      fn: async (ctx) => {
        /**
         Report over a run holding one repair-lane record.
         */
        const lines = reportedLines({
          gathered: gatheredWith({
            repairShippedRecords: 1,
            editorOffered: 4,
          },),
          sinon: ctx.sinon,
        },);

        expect(lines.some(function carriesNote(line,): boolean {
          return line.startsWith(ZERO_NOTE,);
        },),).toBe(false,);
      },
    },),

    it({
      name:
        'POINTS AT editorOffered, which is the only figure separating a lane that was asked and '
          + 'shipped nothing from one the critics never gave work to. Without that pointer the note '
          + 'would say a number means less than it looks and stop there',
      fn: async (ctx) => {
        /**
         Report over a run holding no repair-lane records.
         */
        const lines = reportedLines({
          gathered: gatheredWith({
            repairShippedRecords: 0,
            editorOffered: 0,
          },),
          sinon: ctx.sinon,
        },);

        /**
         The note itself, which must name where the ambiguity is settled.
         */
        const note = lines.find(function carriesNote(line,): boolean {
          return line.startsWith(ZERO_NOTE,);
        },) ?? '';

        expect(note.includes('editorOffered',),).toBe(true,);
      },
    },),

    it({
      name: 'NOTES THE DEGRADED EDITOR ROSTER, since chunks repaired with fewer editors than the roster '
        + 'configures lose the property the ensemble exists for, and stays silent where none degraded',
      fn: async (ctx,) => {
        const lines = reportedLines({
          gathered: {
            ...gatheredWith({ repairShippedRecords: 1, editorOffered: 1, },),
            editorRoster: { ...SILENT_ROSTER, degraded: 1, },
          },
          sinon: ctx.sinon,
        },);
        expect(opensWithNote({ lines, note: DEGRADED_NOTE, },),).toBe(true,);
        /**
         The same report over a roster where no chunk degraded.
         */
        const quiet = reportedLines({
          gathered: gatheredWith({ repairShippedRecords: 1, editorOffered: 1, },),
          sinon: ctx.sinon,
        },);
        expect(opensWithNote({ lines: quiet, note: DEGRADED_NOTE, },),).toBe(false,);
      },
    },),

    it({
      name: 'NOTES THE SILENT REFINER, since a naturalness lane no refiner answered could not run, and stays '
        + 'silent where every slice had a refiner answer',
      fn: async (ctx,) => {
        const lines = reportedLines({
          gathered: {
            ...gatheredWith({ repairShippedRecords: 1, editorOffered: 1, },),
            refineRoster: { ...SILENT_ROSTER, silent: 1, },
          },
          sinon: ctx.sinon,
        },);
        expect(opensWithNote({ lines, note: SILENT_NOTE, },),).toBe(true,);
        /**
         The same report over a roster where no slice lacked a refiner.
         */
        const quiet = reportedLines({
          gathered: gatheredWith({ repairShippedRecords: 1, editorOffered: 1, },),
          sinon: ctx.sinon,
        },);
        expect(opensWithNote({ lines: quiet, note: SILENT_NOTE, },),).toBe(false,);
      },
    },),
  ],
},);

//endregion Probe telemetry report tests
