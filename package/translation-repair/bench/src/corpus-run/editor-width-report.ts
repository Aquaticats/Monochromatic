import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import type {
  ReadonlyEntry,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  summarizeWidths,
  type WidthDraw,
  type WidthRow,
} from './editor-width-model.ts';

//region Editor width report
// What the draw is allowed to say, written where the run's other artifacts go.
//
// NO PASSAGE TEXT, EVER. The corpus is unlicensed and this file is the durable
// output, so rows carry entry ids, slice indices, counts and model ids. What
// each arm actually wrote stays in the process that wrote it.
//
// NO RATES EITHER. Every number here is a count over a draw of a dozen or so
// slices, and rendering four of eleven as a percentage invites reading three
// decimal places into a sample that cannot carry one. The reader can divide.

/**
 Builds the name this draw's report is written under.

 NAMED PER DRAW so running the held-back half cannot overwrite the reading it
 exists to be checked against.

 @param draw - half of the sample this report describes

 @returns File name for that draw

 @example
 ```ts
 const name = reportName('b',);
 ```
 */
function reportName(draw: WidthDraw,): string {
  return `editor-width-${draw}.md`;
}

/**
 Renders one row as a line of counts.

 @param row - one slice's comparison

 @returns Line naming the slice and what it contributed

 @example
 ```ts
 const line = renderRow(row,);
 ```
 */
function renderRow(row: WidthRow,): string {
  return [
    `-   \`${row.entryId}\` slice ${String(row.sliceIndex,)}`,
    `issues ${String(row.acceptedIssues,)}`,
    `heard ${String(row.heardNarrow,)}/${String(row.heardWide,)}`,
    `shipped ${row.narrowShipped ? 'y' : 'n'}/${row.wideShipped ? 'y' : 'n'}`,
    row.comparison,
    `repeat ${row.narrowRepeatAgreed ? 'agreed' : 'FLIPPED'}`,
    `${row.verdict} on ${String(row.usableBallots,)} ${
      wordForCount({
        count: row.usableBallots,
        one: 'ballot',
        many: 'ballots',
      },)
    }`,
  ].join(', ',);
}

/**
 Writes the draw's report.

 Probe scaffolding rather than lane contract, exported so the rendering can be
 exercised on fixtures instead of being seen for the first time at the end of a
 draw that already spent its quota.

 @internal

 @param runsDir - runs directory the report is written under, which the entry
 resolves from its environment and hands down; until 2026-10-10 this read the
 environment itself

 @param rows - every slice that reached a comparison

 @param skipped - slices that carried no work, by refusal

 @param headSha - pipeline commit these rows were produced by

 @param narrowEditorIds - seats in the narrow arm

 @param wideEditorIds - seats in the wide arm

 @param judgeModelIds - panel held fixed across both arms

 @param controlHeld - whether the positive control preferred intact text

 @param draw - half of the sample these rows came from, which names the file
 so the held-back reading cannot overwrite the first one

 @returns Path written, so the caller can name it

 @example
 ```ts
 const path = await writeWidthReport({ runsDir, rows, skipped, headSha, narrowEditorIds, wideEditorIds, judgeModelIds, controlHeld, draw, },);
 ```
 */
export async function writeWidthReport(
  {
    runsDir,
    rows,
    skipped,
    headSha,
    narrowEditorIds,
    wideEditorIds,
    judgeModelIds,
    controlHeld,
    draw,
  }: {
    readonly runsDir: string;
    readonly rows: readonly WidthRow[];
    readonly skipped: Readonly<Record<string, number>>;
    readonly headSha: string;
    readonly narrowEditorIds: readonly RosterModelId[];
    readonly wideEditorIds: readonly RosterModelId[];
    readonly judgeModelIds: readonly RosterModelId[];
    readonly controlHeld: boolean;
    readonly draw: WidthDraw;
  },
): Promise<string> {
  /**
   Counts the decision reads.
   */
  const summary = summarizeWidths({ rows, },);

  /**
   Where the report lands.
   */
  const path = join(
    runsDir,
    reportName(draw,),
  );

  await writeFile(
    path,
    [
      '# Editor width: does seating more editors buy a better repair',
      '',
      `Pipeline commit \`${headSha}\`, draw ${draw.toUpperCase()} of a sample split in two.`,
      draw === 'a'
        ? 'The other half is untouched, and is the reading to run if this one lands near its band.'
        : 'This is the held-back half; read it beside draw A rather than instead of it.',
      '',
      `Narrow arm seats ${String(narrowEditorIds.length,)}: ${narrowEditorIds.join(', ',)}.`,
      `Wide arm seats ${String(wideEditorIds.length,)}: ${wideEditorIds.join(', ',)}.`,
      `Panel is held fixed at ${String(judgeModelIds.length,)}: ${judgeModelIds.join(', ',)}.`,
      '',
      '## Positive control',
      '',
      controlHeld
        ? 'The panel preferred intact text over the same text with a sentence removed, '
          + 'so it can tell damage from repair and the numbers in this report are worth reading.'
        : 'THE PANEL DID NOT PREFER INTACT TEXT over the same text with a sentence '
          + 'removed. Every number in this report is unreadable: an instrument that cannot see a '
          + 'deleted sentence cannot see the finer difference this draw is asking about.',
      '',
      '## Counts',
      '',
      `-   slices that produced a row: ${String(summary.slices,)}`,
      `-   of those, neither arm shipped anything: ${String(summary.nothingShipped,)}`,
      `-   shipped text moved when the editors widened: ${String(summary.moved,)}`,
      `-   of those, only the NARROW arm shipped, so widening suppressed a repair: ${
        String(summary.narrowOnly,)
      }`,
      `-   of those, only the WIDE arm shipped, so widening bought a repair: ${
        String(summary.wideOnly,)
      }`,
      `-   NULL BAND, narrow arm run twice shipped different text: ${String(summary.churned,)}`,
      '',
      'THE PAIRED READING, which is what actually decides it:',
      `-   moved WITHOUT churning, so widening changed what the lane would not have: ${
        String(summary.movedNotChurned,)
      }`,
      `-   churned without moving, so the lane changed its own mind alone: ${
        String(summary.churnedNotMoved,)
      }`,
      '',
      `-   head-to-head, wide preferred in both orders: ${String(summary.wideWins,)}`,
      `-   head-to-head, narrow preferred in both orders: ${String(summary.narrowWins,)}`,
      `-   head-to-head, the seat decided rather than the text: ${String(summary.positionDecided,)}`,
      `-   head-to-head, the panel would not separate them: ${String(summary.tied,)}`,
      '',
      'READ THE MOVE COUNT AGAINST THE NULL BAND, never on its own.',
      'The band is the same arm run twice, so it already contains every reason the',
      'shipped text changes that has nothing to do with width.',
      'A move count inside it is not evidence that widening did anything.',
      '',
      'Both bits are measured on the SAME slice, so slices where they agree decide',
      'nothing: one that moved and churned would have changed anyway, and one that did',
      'neither says nothing. Read the two paired counts against each other.',
      'Both counts are read over every row, the trivial ones included, so they share a',
      'denominator and can be compared directly.',
      'A move is not by itself an improvement: the wide arm fields twice the candidates',
      'against the same selection minimum, so it can split its own vote and keep the',
      'incumbent where the narrow arm shipped a repair. The two "only the ... arm shipped" counts',
      'separate that from a genuinely different rewrite.',
      'The "neither arm shipped anything" count is broken out to be seen, not to be subtracted.',
      '',
      '## Slices that carried no work',
      '',
      ...Object.entries(skipped,)
        .map(function toLine([refusal, count,]: ReadonlyEntry<string, number>,) {
          return `-   ${refusal}: ${String(count,)}`;
        },),
      '',
      '## Rows',
      '',
      ...rows.map(renderRow,),
      '',
    ].join('\n',),
    'utf8',
  );

  return path;
}

//endregion Editor width report
