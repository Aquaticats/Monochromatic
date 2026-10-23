import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  coverageGapLines,
  producerStandings,
  rankStandings,
  readStandingCoverage,
  standingLine,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  assertJudgeableProducerRoster,
  ProducerRosterError,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  readAskedCount,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type { BenchSlice, } from './bench-sample.ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import {
  probeRosterWith,
  readCandidateIds,
  readCandidatesAlone,
} from '@monochromatic-dev/translation-repair-probe/ts';
import {
  runCalibrationRound,
  type ProducerCalibrateRound,
} from './producer-calibrate-round.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Producer calibrate run
// WHICH OF THE ROSTER SHOULD WRITE, measured rather than assumed.
//
// The roster carries nine models (eight when this was written) and the writers
// stay at three. `roster-bench.ts`
// answered the question about WIDTH and found it changes nothing; this answers
// the question about WHO, which that bench was never shaped to ask.
//
// EVERY MODEL WRITES ON EVERY SLICE. A narrow slate would only ever compare the
// models that happened to be seated, and the head-of-roster seating the width
// bench uses would compare the first three against nothing. Running the whole
// roster against the same passage is the only shape where a standing means the same
// thing for each of them.
//
// EVERY MODEL ALSO JUDGES, matching production, and the standing then throws
// away each model's ballots on its own work. That is `producer-standing.ts`'s
// whole argument: counting self-votes would rank the most self-confident model
// first rather than the best-written one.
//
// ONE PASS PER SLICE. The sample size is the SLICE COUNT, not a repeat count,
// and the counts are reported beside every rate so a reader can see how much
// weight one leads by. A gap smaller than its own denominator supports is not
// a gap.
//
// WHO IS MISSING FROM THE TABLE IS REPORTED TOO. `producerStandings` carries a
// row only for a model somebody voted on, so a model its provider refused for
// budget simply vanishes, and an absent row reads exactly like a model that
// wrote and lost. This runner seated the writers on a day one of the two
// providers was empty. `producer-silence.ts` splits the seats three ways and
// names both silent groups; the slates are what tell them apart, which is why
// every round's authors are carried beside its ballots.
//
// SPENDS QUOTA, roughly twenty calls per slice before any re-ask. Point
// `TRANSLATION_REPAIR_RUNS_DIR` at a throwaway directory.

/**
 Slices drawn when the caller names no count.
 */
const DEFAULT_SLICES = 10;

/**
 Refuses a roster no round could select anything from, before any slice is
 drawn or any model is asked.

 THE STAGE REFUSES THE SAME ROSTER AT EVERY SLICE, and the loop reads a slice
 that fails as one fewer round, so a roster of one seat (`--candidates
 <id> --candidates-alone`) used to draw the corpus, lose every slice and print
 an empty standing at a clean exit.

 @param roster - every model that writes and judges

 @throws {@link StatedRefusalError} saying what is wrong with the roster, with
 the stage's own refusal as its cause

 @example
 ```ts
 refuseUnjudgeableRoster({ roster, },);
 ```
 */
function refuseUnjudgeableRoster({ roster, }: { readonly roster: readonly RosterModelId[]; },): void {
  try {
    assertJudgeableProducerRoster({
      producerModelIds: roster,
      judgeModelIds: roster,
      role: 'translator',
    },);
  } catch (error) {
    if (error instanceof ProducerRosterError)
      throw new StatedRefusalError({
        says: error.message,
        cause: error,
      },);
    throw new Error(
      'unreachable: assertJudgeableProducerRoster threw something other than its own roster error, which is all it raises',
      { cause: error, },
    );
  }
}

/**
 Runs the calibration and prints the standing.

 Returns nothing: the report on stdout IS the output.

 @param line - the calibration's command line, read whole by `reportingRefusals`

 @param newClient - builds the client one slice's calls go through; called once
 per slice, since the client answers a repeated prompt from the stored reply of
 the same client, so a slice must not meet the previous slice's replies

 @param drawSlices - draws the slices every model writes; the pinned corpus in
 a run, a script in a case

 @param readHead - reads the pipeline commit the calibration was produced by

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws {@link StatedRefusalError} when the slice count is not a count of at
 least one, a candidate is not seatable, the candidates are asked for alone and
 none is named, the roster could not select anything, a slice's client is
 refused (no provider key), or no slice finished a round; each is raised
 before the standing is printed and, but for the last two, before the corpus
 is drawn

 @example
 ```ts
 await runProducerCalibrate({
   line,
   newClient: createProbeRunClient,
   drawSlices,
   readHead: readHeadSha,
   stageSettings,
 },);
 ```
 */
export async function runProducerCalibrate(
  {
    line,
    newClient,
    drawSlices,
    readHead,
    stageSettings,
  }: {
    readonly line: CommandLineOf<'producer-calibrate'>;
    readonly newClient: () => ModelClient;
    readonly drawSlices: (input: { readonly count: number; },) => Promise<readonly BenchSlice[]>;
    readonly readHead: () => Promise<string>;
    readonly stageSettings: StageSettings;
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
   Every model writing and judging: the seated roster and any seatable
   candidate named after `--candidates`, measured beside it for this run only,
   or the candidates alone under `--candidates-alone`.
   */
  const roster = probeRosterWith({
    candidates: readCandidateIds({ line, },),
    alone: readCandidatesAlone({ line, },),
  },);

  refuseUnjudgeableRoster({ roster, },);

  /**
   Slices every model writes.
   */
  const sample = await drawSlices({ count: wanted, },);

  /**
   Pipeline commit this calibration was produced by.
   */
  const headSha = await readHead();

  console.log(
    `CALIBRATE ${String(sample.length,)} ${
      wordForCount({
        count: sample.length,
        one: 'slice',
        many: 'slices',
      },)
    }, all ${String(roster.length,)} writing`
      + ` and all ${String(roster.length,)} judging (${roster.join(', ',)}), at ${headSha}`,
  );

  /**
   Rounds accumulated as they finish, so a killed run still reports.

   SEQUENTIAL rather than fanned out: each round already asks twenty models,
   and running slices concurrently on top would multiply that into the
   providers at once for no gain in what is being measured.
   */
  const rounds: ProducerCalibrateRound[] = [];

  for (const slice of sample) {
    try {
      // oxlint-disable-next-line no-await-in-loop -- one round already asks twenty models
      rounds.push(await runCalibrationRound({
        slice,
        roster,
        client: newClient(),
        stageSettings,
      },),);
      console.log(
        `  ${slice.entryId}#${String(slice.index,)}: round ${String(rounds.length,)}`
          + ` of ${String(sample.length,)}`,
      );
    } catch (error) {
      // A REFUSAL IS NOT A LOST SLICE: a run with no provider key would
      // otherwise print every slice as lost and a standing over nothing, at a
      // clean exit.
      if (error instanceof StatedRefusalError)
        throw error;

      // A slice that fails is one fewer round, not a failed calibration; the
      // denominators report how much evidence survived.
      console.log(
        `  ${slice.entryId}#${String(slice.index,)}: LOST (${
          errorName({ error, },)
        })`,
      );
    }
  }

  // A CALIBRATION OVER NO ROUND MEASURED NOTHING, and a standing table with no
  // rows and a clean exit reads as a result. The LOST lines are already printed.
  if (rounds.length === 0)
    throw new StatedRefusalError({
      says: `${String(sample.length,)} ${
        wordForCount({
          count: sample.length,
          one: 'slice was',
          many: 'slices were',
        },)
      } drawn and no round finished, so the calibration measured nothing and prints no standing; `
        + 'the LOST lines name the error each slice was lost to',
    },);

  /**
   What the surviving rounds came to.
   */
  const standings = producerStandings({
    rounds: rounds.map(function toRound(sliceRound,): SelectionRound {
      return sliceRound.round;
    },),
  },);

  console.log(`\nSTANDING over ${String(rounds.length,)} ${
    wordForCount({
      count: rounds.length,
      one: 'round',
      many: 'rounds',
    },)
  }, best first:`,);
  for (const standing of rankStandings({ standings, },)) {
    console.log(`  ${standingLine({ standing, },)}`,);
  }

  /**
   Which of the seated models that table actually describes.

   READ AFTER THE TABLE IS PRINTED, so a run whose evidence disagrees with its
   own roster still leaves every standing it paid for on stdout before the
   refusal.
   */
  const coverage = readStandingCoverage({
    roster,
    standings,
    produced: rounds.flatMap(function authorsOf(sliceRound,): readonly RosterModelId[] {
      return sliceRound.authors;
    },),
    // THE PRODUCING STAGES CARRY ONLY A HEARD COUNT OUT (`heardTranslators`,
    // `heardEditors`), never the ids, so this table cannot tell a producer that
    // answered and was dropped before judging from one that never answered.
    // The line it prints says so and points at the SEAT lines; carrying the
    // ids out is still open.
    answered: { kind: 'unrecorded', },
  },);

  for (const gapLine of coverageGapLines({ coverage, },)) {
    console.log(`  ${gapLine}`,);
  }

  console.log(
    '\nA lead smaller than its own denominator supports is not a lead.'
      + ' Read the counts before seating anyone.',
  );
}

//endregion Producer calibrate run
