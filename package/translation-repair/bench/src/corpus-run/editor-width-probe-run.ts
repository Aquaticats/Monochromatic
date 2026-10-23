import { tagged, } from '@monochromatic-dev/module-logger/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  readAskedCount,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type { sampleBenchSlices, } from './bench-sample.ts';
import type { widthControlHolds, } from './editor-width-control.ts';
import type { gatherWidthInput, } from './editor-width-input.ts';
import type { WidthDraw, } from './editor-width-model.ts';
import type { writeWidthReport, } from './editor-width-report.ts';
import type { runWidthSlice, } from './editor-width-slice.ts';
import {
  halfOfSample,
  readWidthDraw,
} from './editor-width-probe-draw.ts';
import { runWidthDraw, } from './editor-width-probe-loop.ts';
import {
  type CommandLineOf,
  RUN_MODELS,
  RUN_ROSTER,
} from '@monochromatic-dev/translation-repair-run/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor width probe run
// The width question: does seating more EDITORS buy a better repair, with the judging panel
// held fixed.
//
// NEITHER WIDTH IS WRITTEN HERE. The narrow arm is whatever `RUN_MODELS` seats
// as editors today and the wide arm is the whole roster, so the probe keeps
// answering the right question when the configuration moves. The owner's
// standing instruction against hardcoding a four or a six is satisfied by
// deriving both ends rather than by picking a better number.
//
// THE CONTROL RUNS FIRST and the draw is abandoned if it fails. An instrument
// that cannot prefer intact text over the same text with a sentence removed
// cannot see the finer difference the draw asks about, and spending an hour to
// collect unreadable numbers is worse than spending a few calls to find out.
//
// SPENDS QUOTA. Point `TRANSLATION_REPAIR_RUNS_DIR` at a throwaway directory.
//
// WHAT THE PROCESS HANDS IN, because this module reads neither the environment
// nor the command line itself: the way to build the client, the corpus draw, the
// commit read and the four stages that touch the runs directory or ask a model.
//
// THE CLIENT IS BUILT ONCE THE DRAW HOLDS A SLICE, right before the control
// asks the panel, so every refusal that needs no provider (the count, the draw's
// name, an empty draw) reaches the operator first. Until 2026-10-10 the entry
// built it before anything else, and a process holding no key refused there.

/**
 Slices drawn when the caller names no count.

 The sample is HALVED into two disjoint draws, so this is the size of the
 whole sample rather than of the draw that runs.
 */
const DEFAULT_SLICES = 18;

/**
 Fewest slices a sample must hold for each draw to take one.

 Draw A takes the first, draw B the second, so B needs two.
 */
const SLICES_A_DRAW_NEEDS: Readonly<Record<WidthDraw, number>> = {
  a: 1,
  b: 2,
};

/**
 Runs the whole probe and writes its report.

 @param line - the probe's command line, read whole by `reportingRefusals`

 @param newClient - builds the client every call goes through; called once,
 after the draw is known to hold a slice and before the control is asked, so
 a refusal that needs no provider is printed before a missing key is

 @param drawSample - draws the whole sample, spread across the corpus the pin names

 @param pin - corpus clone and commit the sample is drawn from

 @param readHeadSha - reads the pipeline commit the rows are stamped with

 @param controlHolds - asks whether the panel can tell a deleted sentence from an intact passage

 @param gather - finds a slice's work for the editors, or why it has none

 @param runSlice - runs one slice at both widths

 @param writeReport - writes the report into the runs directory

 @param runsDir - runs directory the report is written under, which the entry
 resolves from its environment

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws {@link StatedRefusalError} when the panel fails the positive control,
 since every number the draw would produce is unreadable once that happens

 @throws {@link StatedRefusalError} when the named draw is neither half

 @throws {@link StatedRefusalError} when the named draw holds no slice of the
 sample, before the client is built

 @throws whatever building the client refuses with, such as a missing provider
 key, after the header and the draw are printed

 @example
 ```ts
 await runEditorWidthProbe({
   line,
   newClient: createProbeRunClient,
   drawSample,
   pin,
   readHeadSha,
   controlHolds,
   gather,
   runSlice,
   writeReport,
   runsDir,
   stageSettings,
 },);
 ```
 */
export async function runEditorWidthProbe(
  {
    line,
    newClient,
    drawSample,
    pin,
    readHeadSha,
    controlHolds,
    gather,
    runSlice,
    writeReport,
    runsDir,
    stageSettings,
  }: {
    readonly line: CommandLineOf<'editor-width-probe'>;
    readonly newClient: () => ModelClient;
    readonly drawSample: typeof sampleBenchSlices;
    readonly pin: CorpusPin;
    readonly readHeadSha: () => Promise<string>;
    readonly controlHolds: typeof widthControlHolds;
    readonly gather: typeof gatherWidthInput;
    readonly runSlice: typeof runWidthSlice;
    readonly writeReport: typeof writeWidthReport;
    readonly runsDir: string;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Logger for the probe.
   */
  const l = tagged({ tag: 'editor-width', },);

  /**
   Cancellation shared by every call, never fired: the probe runs to the end
   or dies with the process.
   */
  const { signal, } = new AbortController();

  /**
   Seats in the narrow arm, read off the configuration rather than written
   here.
   */
  const narrowEditorIds = RUN_MODELS.editorModelIds;

  /**
   Every model, which is the widest the roster can go.
   */
  const wideEditorIds = RUN_ROSTER;

  /**
   Panel, held fixed so a difference between the arms is about the seats.
   */
  const judgeModelIds = RUN_ROSTER;

  /**
   Slices asked for on the command line, or the default.
   */
  const wanted = readAskedCount({
    line,
    fallback: DEFAULT_SLICES,
    asks: 'slices',
  },);

  console.log(
    `WIDTH narrow ${String(narrowEditorIds.length,)} against wide ${
      String(wideEditorIds.length,)
    }, panel of ${String(judgeModelIds.length,)} held fixed`,
  );

  /**
   Half of the sample this run spends.
   */
  const draw = readWidthDraw({ line, },);

  /**
   Whole sample, spread across the corpus.
   */
  const sample = await drawSample({
    count: wanted,
    pin,
  },);

  /**
   Slices this run spends, leaving the other half untouched.
   */
  const drawn = halfOfSample({
    sample,
    draw,
  },);

  console.log(
    `WIDTH sample ${String(sample.length,)}, draw ${draw.toUpperCase()} ${
      String(drawn.length,)
    }, other half held back`,
  );

  // A DRAW THAT HOLDS NO SLICE IS REFUSED BEFORE THE CONTROL, which is the first
  // thing this run buys. Draw B takes the second, fourth and later slices, so a
  // sample of one leaves it nothing: the run used to buy the control anyway and
  // write a report with no row, a clean finish over no work.
  if (drawn.length === 0) {
    throw new StatedRefusalError({
      says: `editor width probe refused: draw ${draw.toUpperCase()} holds no slice of the ${
        String(sample.length,)
      } ${
        wordForCount({
          count: sample.length,
          one: 'slice',
          many: 'slices',
        },)
      } drawn, so spending it would buy the positive control and report nothing; ask for at least ${
        String(SLICES_A_DRAW_NEEDS[draw],)
      } ${
        wordForCount({
          count: SLICES_A_DRAW_NEEDS[draw],
          one: 'slice',
          many: 'slices',
        },)
      }`,
    },);
  }

  /**
   Client every call of this run goes through, built only now that the draw
   holds work to spend it on.
   */
  const client = newClient();

  /**
   Whether the panel can tell a deleted sentence from an intact passage.
   */
  const controlHeld = await controlHolds({
    client,
    slices: sample,
    judgeModelIds,
    signal,
    l,
    stageSettings,
  },);

  if (!controlHeld)
    throw new StatedRefusalError({
      says: 'editor width probe refused: the panel did not prefer intact text over the same '
        + 'text with a sentence removed, so it cannot read the finer difference this draw '
        + 'asks about and the draw was not spent',
    },);

  console.log(`WIDTH control held; running draw ${draw.toUpperCase()}`,);

  /**
   Pipeline commit these rows were produced by, read BEFORE the draw so the
   report can be republished from inside the loop.
   */
  const headSha = await readHeadSha();

  await runWidthDraw({
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
  },);
}

//endregion Editor width probe run
