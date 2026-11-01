import type { ContestLedger, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type CalibrationGrace,
  readAskedCount,
  STRAGGLER_GRACE_VAR,
  type WriterGrace,
  writerGraceOverrideNote,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type { sampleBenchSlices, } from './bench-sample.ts';
import { editorChairFor, } from './editor-calibrate-chair.ts';
import { driveEditorCalibrate, } from './editor-calibrate-drive.ts';
import {
  printEditorCalibrateRefineReach,
  printEditorCalibrateShipped,
} from './editor-calibrate-closing.ts';
import {
  EDITOR_LANE_REFINERS,
  runEditorLane,
} from './editor-calibrate-lane.ts';
import type { SliceRounds, } from './editor-calibrate-slice.ts';
import { printEditorCalibrateStandings, } from './editor-calibrate-standings.ts';
import {
  type CommandLineOf,
  RUN_ROSTER,
} from '@monochromatic-dev/translation-repair-run/ts';
import { CALIBRATION_OVERLAP, } from '@monochromatic-dev/translation-repair-slice-cache/ts';

//region Editor calibrate run
// WHICH OF THE ROSTER SHOULD EDIT, measured on the editor's own job.
//
// `producer-calibrate.ts` seated the three writers, and its instrument drives
// `runTranslateStage`: a model writing English from Chinese, with nothing in
// front of it but the source. An editor does something else. It is handed
// archive text, a set of adjudicated claims against it, and a window of
// neighbouring prose, and asked to repair what the claims name without
// disturbing what they do not. Seating an editor on a writer's standing is one
// step removed from the job, and this closes that step.
//
// IT DRIVES THE LANE RATHER THAN REPLAYING IT. `ChunkRepairOutcome.rounds`
// already carries every slate judges saw with each candidate's producer
// attached, plus every ballot, which is what a standing counts.
// `repair-selection-rounds.ts` re-shapes it. So the critics, the panel and the
// window are all real: the claims an editor works from here are claims models
// actually raised about that passage, not fixtures.
//
// EVERY MODEL EDITS EVERY SLICE, for the reason the writer calibration gives:
// a narrow slate compares only the models that happened to be seated, and a
// standing then means something different for each of them. "EVERY MODEL" IS THE
// ROSTER'S WRITERS AND THE RUN'S OWN NAMED EDITORS AND REFINERS
// (`EDITOR_LANE_EDITORS`, `editor-calibrate-lane.ts`), since 2026-10-22 and the
// owner's Claude key: Sonnet 5.5 is an editor by the owner's word and holds
// `judge-unmeasured`, which keeps it off every roster-derived bench, so the lane
// seats it by name and this command can measure it; Haiku 5.5 is seated wide and
// not as an editor, holds `writer-unmeasured`, and so spends none of that key on
// edits here. Judging, critique and checking stay on the roster.
//
// CHECKERS SELF-CERTIFY HERE, AND ONLY HERE. Production forbids a checker from
// proving its own repair, so a full editor roster leaves nobody independent to
// check, whatever the roster's width (eight when this was written, nine since
// 2026-09-01). The alternative is rotating editors out, which
// re-introduces the survivorship the shape exists to avoid. The trade is safe
// for THIS measurement because checking runs after selection: a standing reads
// the envelope and chunk-patch ballots, which are cast before any checker is
// asked. What self-certification can move is how many rounds happen, not who
// won the ones that did.
//
// IT RUNS THE NATURALNESS LANE TOO, so the refiner seat is measured rather
// than assumed. `repairChunk` does NOT reach it: it takes `refinerModelIds`
// only to compute the union of models it must seat, sets `refined: false`, and
// returns. Refinement is a separate stage the document driver runs afterwards.
// This was found live, by a nine-slice run reporting zero refiner rounds and no
// refine activity in its log at all, against a module note here claiming the
// refiner standing came off the same spend. It did not; it now does.
//
// THE REFINER SEAT WAS RESEATED ON THE SAME WRITING EVIDENCE the editor seat
// was, so it is exactly as unmeasured, and leaving it that way while measuring
// the seat beside it would answer half the question this runner exists for.
//
// EMPTY DEFINITIONS, honestly. A drawn slice has no document-level glossary
// block behind it, so there is nothing to pass; that is a real value here, not
// an absence dressed as one.
//
// THE TWO SEATS ARE CREDITED SEPARATELY, which takes work because the lane
// unions them. `collectRefinedAuthors` merges the editors with any refiner
// whose rewrite won, so the refined outcome's authorship names both seats in
// one list that cannot be split back apart. The editor column is therefore read
// off the accuracy lane's own outcome, and the refiner column off
// `settleRefinedSlice`'s `refinedBy`.
//
// WHO IS MISSING FROM EACH TABLE IS REPORTED TOO, by `producer-silence.ts`, and
// the two reasons are separated rather than lumped: a model its provider
// refused wrote nothing, while a model whose wording every peer proposed word
// for word shipped without a ballot. The first is evidence nobody bought yet
// and the second is evidence already paid for.
//
// SPENDS QUOTA, and more per slice than the writer calibration does: a whole
// repair lane rather than one stage. Point `TRANSLATION_REPAIR_RUNS_DIR` at a
// throwaway directory.
//
// WHAT THE PROCESS HANDS IN, because this module reads neither the environment
// nor the command line itself: the overlap dial, the corpus draw, the two
// straggler windows and the client are each a function the entry file passes,
// called at the point of the run where the command used to reach for it, so
// the order of the output and of the refusals is the order the command had.
// The two windows it reads become the `StageSettings` every slice's lane runs
// under, beside the contest ledger the entry file read: the calibration once
// adopted its window by writing the round variable back for its rounds to
// read, and nothing writes the process's environment now.

/**
 Slices drawn when the caller names no count.

 SMALLER THAN THE WRITER CALIBRATION'S DEFAULT, because a slice here buys a
 whole lane rather than one stage.
 */
const DEFAULT_SLICES = 6;

/**
 Runs the calibration and prints both standings.

 Returns nothing: the report on stdout IS the output.

 @param line - the calibration's command line, read whole by `reportingRefusals`

 @param readOverlap - reads how many slices may be in flight at once from the
 environment; called after the count is read and before the sample is drawn,
 so a value nothing can read refuses before any work is done

 @param drawSample - draws the slices every model edits from the corpus the
 pin names

 @param pin - corpus clone and commit the sample is drawn from

 @param adoptGrace - adopts the straggler window the rounds wait under; called
 after the sample is drawn and before any round, so an unreadable override
 refuses the run before it spends anything

 @param readWriterGrace - reads both windows the lane's rounds run under, the
 round window with the calibration's own fallback, and the writer rounds' own
 when a launch gave them one

 @param newClient - builds the client every slice shares, called once and
 after the header is printed

 @param contestLedger - where the launch keeps its judged contests, which the
 entry file read

 @param editorBenchStart - seat the editor bench's rotation starts at, which
 only a fixture hands in so that the seats credited move with no sheet text
 (`StageSettings.editorBenchStart`); the launch never gives it, and the prompt
 decides

 @throws {@link StatedRefusalError} when the count, the overlap or a window
 is not one this run can read

 @example
 ```ts
 await runEditorCalibrate({ line, readOverlap, drawSample, pin, adoptGrace, readWriterGrace, newClient, contestLedger, },);
 ```
 */
export async function runEditorCalibrate(
  {
    line,
    readOverlap,
    drawSample,
    pin,
    adoptGrace,
    readWriterGrace,
    newClient,
    contestLedger,
    editorBenchStart,
  }: {
    readonly line: CommandLineOf<'editor-calibrate'>;
    readonly readOverlap: (input: { readonly fallback: number; },) => number;
    readonly drawSample: typeof sampleBenchSlices;
    readonly pin: CorpusPin;
    readonly adoptGrace: () => CalibrationGrace;
    readonly readWriterGrace: () => WriterGrace;
    readonly newClient: () => ModelClient;
    readonly contestLedger: ContestLedger;
    readonly editorBenchStart?: number;
  },
): Promise<void> {
  /**
   Slices asked for on the command line, or the default.
   */
  const wanted = readAskedCount({
    line,
    fallback: DEFAULT_SLICES,
    asks: 'slices',
  },);

  /**
   Editors this launch seats, and whether `--candidates` named them.

   READ RIGHT AFTER THE COUNT, so a card the roster does not know is refused before the overlap is read, the sample is
   drawn or anything is printed: a calibration started for a card it then ran without would print a standing over the
   wrong editors. Only the editors move; the critics, the panel, the judges and the checkers stay the whole roster.
   */
  const {
    editors: chairEditors,
    named: chairNamed,
  } = editorChairFor({ line, },);

  /**
   How many slices may be in flight at once.

   FOUR BY THE OWNER'S DECISION OF 2026-08-26, read from the environment so
   one build still serves both arms of a comparison: `1` reproduces the
   sequential driver exactly, which is what makes the sequential arm a control
   rather than a different program. Read before the sample is drawn, so a
   value nothing can read refuses before any work is done.
   */
  const overlap = readOverlap({ fallback: CALIBRATION_OVERLAP, },);

  /**
   Slices every model edits.
   */
  const sample = await drawSample({
    count: wanted,
    pin,
  },);

  console.log(
    `editor-calibrate: ${String(sample.length,)} ${
      wordForCount({
        count: sample.length,
        one: 'slice',
        many: 'slices',
      },)
    }, `
      + `${String(chairEditors.length,)} ${
        wordForCount({
          count: chairEditors.length,
          one: 'model',
          many: 'models',
        },)
      } editing and ${String(RUN_ROSTER.length,)} judging each, `
      + `${String(overlap,)} ${
        wordForCount({
          count: overlap,
          one: 'slice',
          many: 'slices',
        },)
      } in flight`,
  );

  // THE EDITORS ARE NAMED BY ID WHEN A LAUNCH NAMED THEM, because the header counts them and a count cannot say which
  // card a log was seated with. A launch that names none prints nothing more, so its log reads as it always has.
  if (chairNamed) {
    console.log(`editor-calibrate: editors ${chairEditors.join(', ',)}`,);
    // ONE EDITOR HAS NOBODY TO BE JUDGED AGAINST: a slate of one candidate ships without a vote, so the standing holds
    // no round and the only reading left is how often the edit shipped, which the closing paragraph says is no preference.
    if (chairEditors.length === 1) {
      console.log(
        'editor-calibrate: one editor is seated, so every slate holds one candidate and ships without a vote; '
          + 'the editor standing will hold no judged round, and EDITORS SHIPPED, which is not a preference, '
          + 'is the only reading',
      );
    }
  }

  /**
   Straggler window this run's rounds wait under, and where it came from.

   ADOPTED HERE, before any round, so an unreadable override refuses the run
   before it spends anything, and printed so the log says which window this
   run was under whether or not a voice was ever cut. The calibration's own
   window is 300000 ms under four slices in flight, the owner's decision of
   2026-08-26 on arm D; a launch that sets the variable is honored instead.
   */
  const grace = adoptGrace();
  console.log(
    `straggler window ${String(grace.effectiveMs,)}ms (${
      (grace.source === 'override') ? `${STRAGGLER_GRACE_VAR} override` : 'calibration default'
    })`,
  );

  /**
   Both windows the lane's rounds run under: the round window read with the
   calibration's own fallback, so it is the window the straggler window line
   names, and the writer rounds' own.
   */
  const writerGrace = readWriterGrace();

  /**
   Note naming the writer rounds' window when a launch gave them their own.

   PRINTED because this run's editor and refiner rounds ARE writer rounds: a
   launch that set the dial changed what the standing measures, and a reader
   must be able to tell that from the log alone.
   */
  const writerNote = writerGraceOverrideNote({ grace: writerGrace, },);
  if (writerNote !== '')
    console.log(writerNote,);

  /**
   Client every slice shares, built once for the run.

   ONCE RATHER THAN PER SLICE, because the client holds the budget cooldowns,
   the meter caches and the per-model limiters: built per slice, a provider
   held out on one slice was re-asked immediately on the next, and every
   slice re-read the meters. The two probes already build theirs in `main`.

   SHARED ACROSS SLICES IN FLIGHT TOO, on purpose. The per-model limiter is
   what production routes through, so a slice overlapping another meets the
   same slot rule a corpus pass would, and the comparison describes the
   program that would ship rather than one with the limiter taken out.
   */
  const client = newClient();

  /**
   What every slice produced, in sample order.
   */
  const perSlice = await driveEditorCalibrate({
    sample,
    overlap,
    runSlice: function laneOf({ slice, },): Promise<SliceRounds> {
      return runEditorLane({
        slice,
        client,
        editorModelIds: chairEditors,
        stageSettings: {
          grace: writerGrace,
          contestLedger,
          ...((editorBenchStart === undefined) ? {} : { editorBenchStart, }),
        },
      },);
    },
  },);

  printEditorCalibrateStandings({
    roster: chairEditors,
    refinerRoster: EDITOR_LANE_REFINERS,
    perSlice,
  },);

  printEditorCalibrateRefineReach({ perSlice, },);

  printEditorCalibrateShipped({ perSlice, },);
}

//endregion Editor calibrate run
