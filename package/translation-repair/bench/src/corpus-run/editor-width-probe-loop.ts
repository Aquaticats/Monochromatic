import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { BenchSlice, } from './bench-sample.ts';
import type { gatherWidthInput, } from './editor-width-input.ts';
import type {
  WidthDraw,
  WidthRow,
} from './editor-width-model.ts';
import type { writeWidthReport, } from './editor-width-report.ts';
import type { runWidthSlice, } from './editor-width-slice.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor width probe loop
// SPENDS THE DRAW, one slice at a time, printing a line for each and
// rewriting the report after every one.

/**
 Runs the drawn slices at both widths and writes the report as they settle.

 Sequential by design: every arm of every slice must meet the same provider
 conditions, which fanning the draw out would destroy, and the run is bounded
 by quota rather than by wall time.

 @param client - client every call goes through

 @param drawn - slices this run spends

 @param narrowEditorIds - seats of the narrow arm

 @param wideEditorIds - seats of the wide arm

 @param judgeModelIds - panel held fixed

 @param signal - cancellation shared by every call

 @param l - logger

 @param controlHeld - whether the panel passed the positive control, which
 the report states

 @param draw - half of the sample these slices came from

 @param headSha - pipeline commit the rows were produced by, read before the
 draw so the report can be republished from inside the loop

 @param gather - finds a slice's work for the editors, or why it has none

 @param runSlice - runs one slice at both widths

 @param writeReport - writes the report over the rows so far

 @param runsDir - runs directory the report is written under, which the entry
 resolves from its environment

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @example
 ```ts
 await runWidthDraw({ client, drawn, narrowEditorIds, wideEditorIds, judgeModelIds, signal, l, controlHeld, draw, headSha, gather, runSlice, writeReport, runsDir, stageSettings, },);
 ```
 */
export async function runWidthDraw(
  {
    client,
    drawn,
    narrowEditorIds,
    wideEditorIds,
    judgeModelIds,
    signal,
    l,
    controlHeld,
    draw,
    headSha,
    gather,
    runSlice,
    writeReport,
    runsDir,
    stageSettings,
  }: {
    readonly client: ModelClient;
    readonly drawn: readonly BenchSlice[];
    readonly narrowEditorIds: readonly RosterModelId[];
    readonly wideEditorIds: readonly RosterModelId[];
    readonly judgeModelIds: readonly RosterModelId[];
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly controlHeld: boolean;
    readonly draw: WidthDraw;
    readonly headSha: string;
    readonly gather: typeof gatherWidthInput;
    readonly runSlice: typeof runWidthSlice;
    readonly writeReport: typeof writeWidthReport;
    readonly runsDir: string;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Rows accumulated as they finish.
   */
  const rows: WidthRow[] = [];

  /**
   Slices that carried no work, counted by the wall they hit; a map until
   handed to the report, as every record filled by a key is (ledger B77).
   */
  const skipped = new Map<string, number>();

  /**
   Rewrites the report over everything settled so far.

   CALLED AFTER EVERY SLICE, not once at the end. A draw of twenty slices runs
   for hours, and a run killed at slice eighteen with the write still ahead of
   it would throw away every hour it had already spent. Rewriting a few
   kilobytes of markdown twenty times costs nothing worth measuring against
   that.

   @returns Path written
   */
  async function publish(): Promise<string> {
    return await writeReport({
      runsDir,
      rows,
      skipped: Object.fromEntries(skipped,),
      headSha,
      narrowEditorIds,
      wideEditorIds,
      judgeModelIds,
      controlHeld,
      draw,
    },);
  }

  for (const slice of drawn) {
    /* oxlint-disable eslint/no-await-in-loop -- sequential by design: every arm of every slice must meet the same provider conditions, which fanning the draw out would destroy, and the run is bounded by quota rather than by wall time */
    /**
     Work the critics and panel found in this slice.
     */
    const outcome = await gather({
      client,
      slice,
      signal,
      l,
      stageSettings,
    },);
    /* oxlint-enable eslint/no-await-in-loop */

    if (outcome.kind === 'skipped') {
      skipped.set(
        outcome.refusal,
        (skipped.get(outcome.refusal,) ?? 0) + 1,
      );
      console.log(
        `WIDTH ${outcome.entryId} slice ${String(outcome.sliceIndex,)}: ${outcome.refusal}`,
      );
      // oxlint-disable-next-line eslint/no-await-in-loop -- the republish is the durability of this run: it must land before the next slice starts, which is exactly what awaiting it here means
      await publish();
      continue;
    }

    /* oxlint-disable eslint/no-await-in-loop -- sequential for the same reason the draw's gather is */
    /**
     That slice run at both widths, with the null band beside it.
     */
    const row = await runSlice({
      client,
      input: outcome.input,
      narrowEditorIds,
      wideEditorIds,
      judgeModelIds,
      signal,
      l,
      stageSettings,
    },);
    /* oxlint-enable eslint/no-await-in-loop */

    rows.push(row,);
    console.log(
      `WIDTH ${row.entryId} slice ${String(row.sliceIndex,)}: ${row.comparison}, repeat ${
        row.narrowRepeatAgreed ? 'agreed' : 'FLIPPED'
      }, ${row.verdict}`,
    );
    // oxlint-disable-next-line eslint/no-await-in-loop -- the same durability as the per-slice republish: a run killed mid-draw must still leave every slice it paid for
    await publish();
  }

  /**
   Where the report landed.
   */
  const path = await publish();

  console.log(`WIDTH wrote ${path}`,);
}

//endregion Editor width probe loop
