import { tagged, } from '@monochromatic-dev/module-logger/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  repairChunk,
  settleRefinedSlice,
} from '@monochromatic-dev/translation-repair-repair/ts';
import {
  EDITOR_ROUND_STAGES,
  REFINER_ROUND_STAGES,
  selectionRoundsFor,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import type { BenchSlice, } from './bench-sample.ts';
import {
  shippedAuthors,
  type SliceRounds,
} from './editor-calibrate-slice.ts';
import {
  RUN_MODELS,
  RUN_PER_CALL_TIMEOUT_MS,
  RUN_ROSTER,
  RUN_WRITERS,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  assertOneWireModelPerBench,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor calibrate lane
// ONE SLICE THROUGH THE WHOLE REPAIR LANE, every model editing and every model
// judging, as the editor calibration measures it (`editor-calibrate-run.ts`
// holds the module note that says why the lane is driven rather than replayed,
// why checkers self-certify here and why the refiner seat is run too).

/**
 The models of a run's roster that write, followed by the ones the run seats to write by name that the roster-derived
 benches leave out.

 THE WRITERS ARE {@link RUN_WRITERS}, the roster less the models whose writing no calibration has measured
 (`writer-unmeasured`): such a model takes no chair, so Haiku 5.5, which the owner seated "wide" and not as an editor,
 spends none of the Claude key on edits and refinements here. THE NAMED ONES ARE THE RUN'S OWN LIST: Sonnet 5.5 is an editor on the
 owner's word and holds `judge-unmeasured`, which keeps it off every roster-derived bench, so without its name here the
 lane could never measure it (C4 of the review of the Claude route).

 @param bench - name of the chair, for the failure when a wire model is seated twice

 @param named - the run's own list for the chair

 @returns The chair's models, the writers first and in roster order, then the named ones not among them

 @throws {@link WireModelRepeatedError} when the chair seats one wire model more than once: the lane is built outside
 the run's own bench list (`RUN_BENCHES`), so the one-wire-model rule does not reach it unless it is asked here (R11 of the
 second review of the Claude route)

 @example
 ```ts
 const editors = editorLaneChairOf({ bench: 'editor calibration editors', named: RUN_MODELS.editorModelIds, },);
 ```
 */
export function editorLaneChairOf(
  {
    bench,
    named,
  }: {
    readonly bench: string;
    readonly named: readonly RosterModelId[];
  },
): readonly RosterModelId[] {
  /**
   The chair as built, checked before the lane can seat it.
   */
  const chair = [
    ...RUN_WRITERS,
    ...named.filter(function notAlreadySeated(modelId,): boolean {
      return !RUN_WRITERS.includes(modelId,);
    },),
  ];
  assertOneWireModelPerBench({
    bench,
    modelIds: chair,
  },);
  return chair;
}

/**
 Editors the calibration lane seats: the roster's writers, then the run's editors that are not among them.
 */
export const EDITOR_LANE_EDITORS: readonly RosterModelId[] = editorLaneChairOf({
  bench: 'editor calibration editors',
  named: RUN_MODELS.editorModelIds,
},);

/**
 Refiners the calibration lane seats: the roster's writers, then the run's refiners that are not among them.
 */
export const EDITOR_LANE_REFINERS: readonly RosterModelId[] = editorLaneChairOf({
  bench: 'editor calibration refiners',
  named: RUN_MODELS.refinerModelIds,
},);

/**
 Runs one slice through the whole repair lane, every model editing and
 every model judging.

 @param slice - passage to repair, with the archive text it stands against

 @param client - client every slice of the run shares

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Rounds that slice produced, split by seat

 @example
 ```ts
 const rounds = await runEditorLane({ slice, client, stageSettings, },);
 ```
 */
export async function runEditorLane(
  {
    slice,
    client,
    stageSettings,
  }: {
    readonly slice: BenchSlice;
    readonly client: ModelClient;
    readonly stageSettings: StageSettings;
  },
): Promise<SliceRounds> {
  /**
   Logger tagged for this slice.
   */
  const l = tagged({ tag: `editor-calibrate-${slice.entryId}-${String(slice.index,)}`, },);

  /**
   Abort signal for the whole slice.
   */
  const { signal, } = new AbortController();

  /**
   Every seat filled by every model, matching the module note, except the editor and refiner chairs: those are the
   roster's writers and the run's named editors and refiners ({@link EDITOR_LANE_EDITORS}).
   */
  const models = {
    criticModelIds: RUN_ROSTER,
    panelModelIds: RUN_ROSTER,
    editorModelIds: EDITOR_LANE_EDITORS,
    judgeModelIds: RUN_ROSTER,
    refinerModelIds: EDITOR_LANE_REFINERS,
    checkerModelIds: RUN_ROSTER,
    // See the module note: a full editor roster leaves nobody independent,
    // and checking runs after the ballots a standing reads.
    checkerSelfCertificationPermitted: true,
  };

  /**
   Everything the accuracy lane decided about this passage.
   */
  const outcome = await repairChunk({
    client,
    sliceIndex: slice.index,
    sourceText: slice.sourceText,
    targetText: slice.incumbentText,
    lineStructured: slice.lineStructured,
    models,
    declaredNames: [],
    signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    stageSettings,
  },);

  /**
   Same slice after the naturalness lane, whose rounds land on the outcome
   beside the accuracy lane's own.
   */
  const refined = await settleRefinedSlice({
    client,
    outcome,
    sourceText: slice.sourceText,
    incumbentText: slice.incumbentText,
    // See the module note: a drawn slice carries no document glossary.
    definitions: '',
    models,
    refinerModelIds: EDITOR_LANE_REFINERS,
    declaredNames: [],
    signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    stageSettings,
  },);

  return {
    editor: selectionRoundsFor({
      rounds: refined.outcome
        .rounds,
      stages: EDITOR_ROUND_STAGES,
    },),
    refiner: selectionRoundsFor({
      rounds: refined.outcome
        .rounds,
      stages: REFINER_ROUND_STAGES,
    },),
    refineAsked: refined.asked,
    editorShipped: shippedAuthors({ authorship: outcome.authorship, },),
    refinerShipped: refined.refinedBy,
    refinerHeard: refined.refinersHeard,
  };
}

//endregion Editor calibrate lane
