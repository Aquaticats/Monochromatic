import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  slatedAuthors,
  standingReportLines,
} from './editor-calibrate-standing.ts';
import type { SliceRounds, } from './editor-calibrate-slice.ts';

//region Editor calibrate standings
// THE EDITOR STANDING AND THE REFINER STANDING of one calibration, each read
// off the rounds its seat was judged in, printed one after the other.

/**
 Prints the editor standing and then the refiner standing.

 @param roster - every model the run seated as an editor, which the editor table accounts for, and the refiner table
 too where no refiner roster is given

 @param refinerRoster - every model the run seated as a refiner, which the refiner table accounts for; absent reads as
 {@link roster}

 @param perSlice - what every slice produced, in sample order

 @example
 ```ts
 printEditorCalibrateStandings({ roster: RUN_ROSTER, perSlice, },);
 ```
 */
export function printEditorCalibrateStandings(
  {
    roster,
    refinerRoster = roster,
    perSlice,
  }: {
    readonly roster: readonly RosterModelId[];
    readonly refinerRoster?: readonly RosterModelId[];
    readonly perSlice: readonly SliceRounds[];
  },
): void {
  /**
   Editor rounds, grouped by the slice that bought them.
   */
  const editorPerSlice = perSlice.map(function editorRounds(rounds,): readonly SelectionRound[] {
    return rounds.editor;
  },);

  /**
   Refiner rounds, grouped the same way.
   */
  const refinerPerSlice = perSlice.map(function refinerRounds(rounds,): readonly SelectionRound[] {
    return rounds.refiner;
  },);

  for (
    const standingLine of standingReportLines({
      seat: 'EDITOR',
      roster,
      perSlice: editorPerSlice,
      // JUDGED AUTHORS PLUS SHIPPING ONES, because a slice where every editor
      // proposed the same text ships it with no round at all, and a model seen
      // only there wrote something no ballot names.
      produced: [
        ...slatedAuthors({ perSlice: editorPerSlice, },),
        ...perSlice.flatMap(function shippingEditors(rounds,): readonly RosterModelId[] {
          return rounds.editorShipped;
        },),
      ],
      // THE EDITOR STAGE CARRIES NO ANSWER LIST OUT OF THE CHUNK OUTCOME, only a
      // count for `editor-width-arm`, so this seat cannot tell an editor that
      // answered and was dropped before judging from one that never answered.
      // The line it prints says so and points at the SEAT lines.
      answered: { kind: 'unrecorded', },
    },)
  ) {
    console.log(standingLine,);
  }

  for (
    const standingLine of standingReportLines({
      seat: 'REFINER',
      roster: refinerRoster,
      perSlice: refinerPerSlice,
      produced: [
        ...slatedAuthors({ perSlice: refinerPerSlice, },),
        ...perSlice.flatMap(function shippingRefiners(rounds,): readonly RosterModelId[] {
          return rounds.refinerShipped;
        },),
      ],
      // WHO THE REFINE STAGE HEARD, so a rewriter that answered every ask and
      // left every paragraph as it stood is reported as answered, not silent.
      answered: {
        kind: 'recorded',
        modelIds: perSlice.flatMap(function heardRefiners(rounds,): readonly RosterModelId[] {
          return rounds.refinerHeard;
        },),
      },
    },)
  ) {
    console.log(standingLine,);
  }
}

//endregion Editor calibrate standings
