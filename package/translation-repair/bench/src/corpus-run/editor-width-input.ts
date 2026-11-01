import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  type AdjudicatedIssue,
  aggregateClaims,
  dedupeAcceptedIssues,
  deriveEditableEnvelopes,
  type EditableEnvelope,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { runChunkCriticPhase, } from '@monochromatic-dev/translation-repair-repair/ts';
import { runPanelStage, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { BenchSlice, } from './bench-sample.ts';
import {
  RUN_MODELS,
  RUN_PER_CALL_TIMEOUT_MS,
} from '@monochromatic-dev/translation-repair-run/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor width input
// Work for the editors to do, gathered the way production gathers it.
//
// The width comparison needs slices that carry ACCEPTED ISSUES, because the
// repair lane edits nothing else. Two routes exist and this takes the more
// expensive one deliberately.
//
// The cheap route reads issues out of a settled artifact. It was rejected: the
// artifacts were written by earlier slicings, two later slicing fixes (declined
// blocks leaving the slice, unpaired sections reaching subdivision as absent)
// both moved where slice boundaries fall, and sliceIndex has meant different
// things to different stamps. Issues carrying offsets into a
// target that today's slicer cuts differently would place envelopes over the
// wrong words, and the probe would be measuring editors handed nonsense.
//
// So the critics and the panel run fresh over the drawn slice. That costs a
// critic round and a panel round per slice, buys inputs that cannot be
// misaligned with their own text, and is what the whole comparison rests on.

/**
 One slice with real work attached, ready to hand to editors at any width.
 */
export type WidthProbeInput = {
  /**
   Entry the slice came from.
   */
  readonly entryId: string;

  /**
   Position within that entry.
   */
  readonly sliceIndex: number;

  /**
   Original passage.
   */
  readonly sourceText: string;

  /**
   Translation the editors repair.
   */
  readonly targetText: string;

  /**
   Accepted issues, deduplicated exactly as `repairChunk` deduplicates them.
   */
  readonly issues: readonly AdjudicatedIssue[];

  /**
   Envelopes cut from those issues.
   */
  readonly envelopes: readonly EditableEnvelope[];

  /**
   Everything the critic and panel rounds recorded.
   */
  readonly findings: readonly string[];
};

/**
 Why one slice contributed no input, named rather than returned as an empty
 list, so a draw that produced nothing reports which wall it hit.
 */
export type WidthInputRefusal = 'no-claims' | 'no-accepted-issues' | 'no-envelopes';

/**
 A slice with work, or the reason it has none.

 @internal
 */
export type WidthInputOutcome =
  | {
    readonly kind: 'ready';
    readonly input: WidthProbeInput;
  }
    | {
      readonly kind: 'skipped';
      readonly refusal: WidthInputRefusal;
      readonly entryId: string;
      readonly sliceIndex: number;
    };

/**
 Runs the critics and the panel over one slice, as production does.

 ROSTERS ARE PRODUCTION'S, not the probe's arms. Whatever the editors are
 later asked at, the work put in front of them has to be the work the corpus
 would really produce; drawing issues from a narrower or wider critic roster
 would change the input alongside the variable under test.

 @param client - injected model client

 @param slice - drawn slice to find work in

 @param signal - cancellation for every call this makes

 @param l - logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Slice with its accepted issues, or why it has none

 @example
 ```ts
 const outcome = await gatherWidthInput({ client, slice, signal, l, stageSettings, },);
 ```

 @internal
 */
export async function gatherWidthInput(
  {
    client,
    slice,
    signal,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly slice: BenchSlice;
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<WidthInputOutcome> {
  /**
   Both sides parsed, which the critics need for offsets and containers.

   The slice stands as its own document here. A bench has no surrounding page
   to place it in, and the critic phase reads these for structure rather than
   for position within a larger file.
   */
  const documents = {
    source: parseDocument({ text: slice.sourceText, },),
    target: parseDocument({ text: slice.incumbentText, },),
  };

  /**
   Critic claims plus the non-translation screen.
   */
  const critic = await runChunkCriticPhase({
    client,
    criticModelIds: RUN_MODELS.criticModelIds,
    sourceText: slice.sourceText,
    targetText: slice.incumbentText,
    documents,
    sliceIndex: slice.index,
    signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    stageSettings,
  },);

  /**
   Claims the critics filed and the screen let through.
   */
  const { claims, } = critic;

  if (claims.length === 0)
    return {
      kind: 'skipped',
      refusal: 'no-claims',
      entryId: slice.entryId,
      sliceIndex: slice.index,
    };

  /**
   Merge-proposal clusters over the validated claims.
   */
  const { clusters, } = aggregateClaims({ claims, },);

  /**
   Panel decision over those clusters.
   */
  const panel = await runPanelStage({
    client,
    panelModelIds: RUN_MODELS.panelModelIds,
    sourceText: slice.sourceText,
    targetText: slice.incumbentText,
    clusters,
    signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    stageSettings,
  },);

  /**
   Same-place accepted duplicates merged, before envelopes are cut, exactly
   where `repairChunk` does it.
   */
  const deduped = dedupeAcceptedIssues({ issues: panel.issues, },);

  /**
   Accepted issues, which are the only ones an envelope can be cut from.
   */
  const accepted = deduped
    .issues
    .filter(function isAccepted(issue,) {
      return issue.status === 'accepted';
    },);

  if (accepted.length === 0)
    return {
      kind: 'skipped',
      refusal: 'no-accepted-issues',
      entryId: slice.entryId,
      sliceIndex: slice.index,
    };

  /**
   Envelopes cut from those issues.
   */
  const { envelopes, } = deriveEditableEnvelopes({
    issues: deduped.issues,
    targetText: slice.incumbentText,
  },);

  if (envelopes.length === 0)
    return {
      kind: 'skipped',
      refusal: 'no-envelopes',
      entryId: slice.entryId,
      sliceIndex: slice.index,
    };

  return {
    kind: 'ready',
    input: {
      entryId: slice.entryId,
      sliceIndex: slice.index,
      sourceText: slice.sourceText,
      targetText: slice.incumbentText,
      issues: deduped.issues,
      envelopes,
      findings: [
        ...critic.findings,
        ...panel.findings,
        ...deduped.findings,
      ],
    },
  };
}

//endregion Editor width input
