import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  applyPatchOperations,
  type PatchOperation,
  type PatchOutcome,
} from './apply-patch.ts';
import {
  type Candidate,
  selectBestCandidate,
} from './candidate-select.ts';
import type { SyntheticClient, } from './chat-contract.ts';
import type { EditableEnvelope, } from './patch-model.ts';
import type { SyntheticModelId, } from './synthetic-catalog.ts';

//region Editor ensemble
// Several editors rewrite the same chunk, and judges that wrote none of the
// candidates choose what ships. The editor was previously the ONE stage where a
// single model decided alone, which the user ruled out: no single model should
// control any part of the pipeline.
//
// Selection runs at BOTH granularities, by user decision, because each catches
// what the other misses:
//
// - Per envelope, the best fix for each individual issue can win even when the
//   model that wrote it botched the rest of the chunk.
// - Per chunk, whole candidates compete, which is the only level at which
//   coherence across envelopes is visible at all.
//
// The per-envelope winners are assembled into a COMPOSITE candidate that then
// competes at chunk level against the whole-chunk candidates. That is what keeps
// best-of-breed assembly honest: a composite stitched from several models is
// text no model wrote or read as a whole, so it has to win on its merits against
// candidates that were written coherently, rather than being adopted by
// construction.

/**
 * One editor's proposal for a chunk.
 *
 * @example
 * ```ts
 * const candidate: EditorCandidate = { modelId, operations, patch, };
 * ```
 */
export type EditorCandidate = {
  /**
   * Model that produced this proposal.
   */
  readonly modelId: SyntheticModelId;

  /**
   * Operations it proposed, one per envelope it chose to edit.
   */
  readonly operations: readonly PatchOperation[];

  /**
   * Apply-gate outcome of those operations.
   */
  readonly patch: PatchOutcome;
};

/**
 * Chooses one replacement text per envelope by judging the distinct proposals
 * models made for it, then assembles the winners into one operation set.
 *
 * An envelope only one model proposed for needs no vote: there is nothing to
 * compare it against, and it still faces the chunk-level judges afterwards.
 *
 * @param client - injected model client
 *
 * @param candidates - editor proposals, in roster order
 *
 * @param envelopes - envelopes of this chunk
 *
 * @param judgeModelIds - roster judges are drawn from
 *
 * @param sourceText - original chunk text, evidence for judges
 *
 * @param signal - caller abort honored by every exchange
 *
 * @param perCallTimeoutMs - deadline per exchange
 *
 * @param l - pipeline logger
 *
 * @returns Composite operation set built from per-envelope winners
 *
 * @example
 * ```ts
 * const composite = await selectPerEnvelope({ client, candidates, envelopes, ... },);
 * ```
 */
export async function selectPerEnvelope(
  {
    client,
    candidates,
    envelopes,
    judgeModelIds,
    sourceText,
    signal,
    perCallTimeoutMs,
    l,
  }: ForeignBorrowed<{
    readonly client: SyntheticClient;
    readonly candidates: readonly EditorCandidate[];
    readonly envelopes: readonly EditableEnvelope[];
    readonly judgeModelIds: readonly SyntheticModelId[];
    readonly sourceText: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  }>,
): Promise<readonly PatchOperation[]> {
  /**
   * Logger tagged with this selection pass.
   */
  const el = tagged({
    tag: selectPerEnvelope.name,
    l,
  },);

  /**
   * Winning operation per envelope, filled in envelope order.
   */
  const winners: PatchOperation[] = [];
  for (const envelope of envelopes) {
    /**
     * Every distinct proposal for this envelope, first proposer winning ties
     * on identical text so the set stays deduplicated.
     */
    const proposals: Candidate<PatchOperation>[] = [];
    for (const candidate of candidates) {
      /**
       * This model's operation for this envelope, when it wrote one.
       */
      const operation = candidate.operations
        .find(function forEnvelope(op,) {
          return op.envelopeId === envelope.envelopeId;
        },);
      if (operation === undefined)
        continue;
      if (proposals.some(function sameText(existing,) {
        return existing.value
          .newText
          === operation.newText;
      },))
        continue;
      proposals.push({
        modelId: candidate.modelId,
        value: operation,
        rendered: operation.newText,
      },);
    }

    if (proposals.length === 0)
      continue;

    /**
     * Sole proposal, adopted without a vote because there is nothing to
     * compare it against; chunk-level judging still sees it.
     */
    const [sole,] = proposals;
    if ((proposals.length === 1) && (sole !== undefined)) {
      winners.push(sole.value,);
      continue;
    }

    /**
     * Judges verdict over the distinct proposals for this envelope.
     */
    /* oxlint-disable-next-line no-await-in-loop -- envelopes are judged sequentially so per-model concurrency stays at one, which the measured provider serialization requires */
    const outcome = await selectBestCandidate({
      client,
      candidates: proposals,
      judgeModelIds,
      task:
        'Each candidate replaces the SAME passage of an English translation of the Chinese ORIGINAL below.',
      criteria: [
        'Faithfulness to the ORIGINAL: no content added, dropped, or altered in meaning.',
        'Natural, idiomatic English that carries the ORIGINAL\'s feeling.',
        'Fits the surrounding text in register and tense.',
      ],
      evidence: `ORIGINAL (Chinese)\n=====\n${sourceText}\n=====`,
      signal,
      perCallTimeoutMs,
      l,
    },);
    if (outcome.kind === 'declined') {
      el.info(
        `envelope ${envelope.envelopeId}: ${outcome.reason}; leaving it unedited in the composite`,
      );
      continue;
    }
    winners.push(outcome.value,);
  }
  return winners;
}

/**
 * Judges whole-chunk candidates and returns the patch that ships.
 *
 * @param client - injected model client
 *
 * @param candidates - whole-chunk proposals including the composite
 *
 * @param judgeModelIds - roster judges are drawn from
 *
 * @param sourceText - original chunk text, evidence for judges
 *
 * @param fallback - patch adopted when judges decline
 *
 * @param signal - caller abort honored by every exchange
 *
 * @param perCallTimeoutMs - deadline per exchange
 *
 * @param l - pipeline logger
 *
 * @returns Winning patch, or the fallback when judges decline
 *
 * @example
 * ```ts
 * const patch = await selectChunkPatch({ client, candidates, fallback, ... },);
 * ```
 */
export async function selectChunkPatch(
  {
    client,
    candidates,
    judgeModelIds,
    sourceText,
    fallback,
    signal,
    perCallTimeoutMs,
    l,
  }: ForeignBorrowed<{
    readonly client: SyntheticClient;
    readonly candidates: readonly Candidate<PatchOutcome>[];
    readonly judgeModelIds: readonly SyntheticModelId[];
    readonly sourceText: string;
    readonly fallback: PatchOutcome;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  }>,
): Promise<PatchOutcome> {
  /**
   * Logger tagged with this selection pass.
   */
  const cl = tagged({
    tag: selectChunkPatch.name,
    l,
  },);

  /**
   * Sole candidate, which needs no vote.
   */
  const [sole,] = candidates;
  if ((candidates.length === 1) && (sole !== undefined))
    return sole.value;

  /**
   * Judges verdict over the whole-chunk candidates.
   */
  const outcome = await selectBestCandidate({
    client,
    candidates,
    judgeModelIds,
    task:
      'Each candidate is a full English translation of the Chinese ORIGINAL below, after repairs were applied.',
    criteria: [
      'Faithfulness to the ORIGINAL: no content added, dropped, or altered in meaning.',
      'Natural, idiomatic English reading as one coherent passage, not as stitched fragments.',
      'Consistent voice, tense, and terminology across the whole passage.',
    ],
    evidence: `ORIGINAL (Chinese)\n=====\n${sourceText}\n=====`,
    signal,
    perCallTimeoutMs,
    l,
  },);
  if (outcome.kind === 'declined') {
    // Falling back to no repair at all would discard fixes the panel already
    // ruled real, turning a disagreement about wording into a recall loss, so
    // the caller's fallback is a repaired patch rather than the original text.
    cl.info(`${outcome.reason}; shipping the fallback patch`,);
    return fallback;
  }
  cl.info(
    `chunk patch from ${outcome.modelId} won ${String(outcome.votes,)} of ${
      String(outcome.ballots,)
    } ballots`,
  );
  return outcome.value;
}

/**
 * Applies one candidate's operations, so a composite can be scored the same way
 * a model's own proposal is.
 *
 * @param targetText - translation chunk text
 *
 * @param envelopes - envelopes of this chunk
 *
 * @param operations - operations to apply
 *
 * @returns Apply-gate outcome
 *
 * @example
 * ```ts
 * const patch = applyCandidate({ targetText, envelopes, operations, },);
 * ```
 */
export function applyCandidate(
  {
    targetText,
    envelopes,
    operations,
  }: {
    readonly targetText: string;
    readonly envelopes: readonly EditableEnvelope[];
    readonly operations: readonly PatchOperation[];
  },
): PatchOutcome {
  return applyPatchOperations({
    targetText,
    envelopes,
    operations,
  },);
}

//endregion Editor ensemble
