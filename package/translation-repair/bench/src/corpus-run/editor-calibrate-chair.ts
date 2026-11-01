import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import {
  probeRosterWith,
  readCandidateIds,
  readCandidatesAlone,
} from '@monochromatic-dev/translation-repair-probe/ts';
import {
  assertOneWireModelPerBench,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  EDITOR_LANE_EDITORS,
  editorLaneChairOf,
} from './editor-calibrate-lane.ts';

//region Editor calibrate chair
// WHICH EDITORS ONE EDITOR CALIBRATION SEATS, read off the command line.
//
// THE SAME TWO FLAGS AS THE PROBES, read by the same readers (`probe-candidates.ts`): `--candidates` names seatable
// card ids and `--candidates-alone` leaves everything but those cards out of the editor chair. An unknown card is
// refused in the probes' own words, since a calibration started for a card it then ran without would print a standing
// over the wrong editors.
//
// THE CARDS REPLACE THE RUN'S NAMED EDITORS, they do not join them. The run names plain Sonnet 5.5 as an editor, and a
// Sonnet effort card is the same wire model, which one bench may not seat twice (`assertOneWireModelPerBench`), so
// joining would refuse the very card this flag exists to bench. The chair beside the writers is therefore the roster's
// writers and then the named cards, and the chair alone is the named cards.
//
// ONLY THE EDITOR CHAIR MOVES. Critics, panel, judges and checkers stay the whole roster, so `--candidates-alone`
// confines the EDITING and not the spend: a launch still needs the keys of every provider the roster reads.

/**
 What the command line decided the editor chair is.
 */
export type EditorChair = {
  /**
   Editors the calibration seats, in seat order.
   */
  readonly editors: readonly RosterModelId[];

  /**
   Whether the command line named the chair, so the run prints it; false when the lane's own editors stand.
   */
  readonly named: boolean;
};

/**
 Bench name the chair is refused under when it seats one wire model twice.
 */
const BENCH = 'editor calibration editors';

/**
 Reads the editors this launch seats.

 @param line - the calibration's command line, read whole by `reportingRefusals`

 @returns The lane's own editors when neither flag is written; else the roster's writers then the named cards, or the
 named cards alone under `--candidates-alone`

 @throws StatedRefusalError when `--candidates` names no id or one the roster does not know, or
 `--candidates-alone` is written without a card

 @throws {@link WireModelRepeatedError} when the chair seats one wire model more than once, such as two efforts of
 Sonnet 5.5

 @example
 ```ts
 const { editors, named, } = editorChairFor({ line, },);
 ```
 */
export function editorChairFor(
  { line, }: { readonly line: CommandLineOf<'editor-calibrate'>; },
): EditorChair {
  /**
   Cards named after `--candidates`, read first so a bad id is refused before anything else is.
   */
  const candidates = readCandidateIds({ line, },);

  /**
   Whether the editing is confined to those cards.
   */
  const alone = readCandidatesAlone({ line, },);
  if (alone) {
    /**
     The cards by themselves, each once, which refuses `--candidates-alone` over nobody.
     */
    const editors = probeRosterWith({
      candidates,
      alone,
    },);
    assertOneWireModelPerBench({
      bench: BENCH,
      modelIds: editors,
    },);
    return {
      editors,
      named: true,
    };
  }
  if (candidates.length === 0) {
    return {
      editors: EDITOR_LANE_EDITORS,
      named: false,
    };
  }
  return {
    editors: editorLaneChairOf({
      bench: BENCH,
      named: candidates,
    },),
    named: true,
  };
}

//endregion Editor calibrate chair
