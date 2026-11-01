import type {
  PatchOperation,
  PatchOutcome,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import type { CandidateProducer, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { RepairJudgedRound, } from '@monochromatic-dev/translation-repair-assembly/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

/**
 Nobody wrote the shipped text, because the untouched translation shipped.

 A NAMED STATE RATHER THAN AN ABSENT VALUE. Shipping the translation as it
 stood is a decision the stage reaches deliberately, on a rejection or when no
 operation survived the gate, so it reads as one of the answers rather than as
 a missing one.

 @example
 ```ts
 return { patch: rejectionFallback, shippedProducer: NOBODY_WROTE_IT, ... };
 ```
 */
export const NOBODY_WROTE_IT = { kind: 'unattributed', } as const;

/**
 Who wrote the text a stage shipped.

 @example
 ```ts
 const authors = (shipped.kind === 'unattributed') ? [] : producerModelIds(shipped,);
 ```
 */
export type ShippedProducer = CandidateProducer | typeof NOBODY_WROTE_IT;

//region Editor selection result
// What the editor ensemble's two judged stages DECIDED, apart from the code
// that decides it.
//
// Split from `editor-ensemble.ts` when that file reached its line budget, along
// the seam `translate-stage-result.ts` already established for the other lane:
// this is the record every later reader joins to, while the ensemble is the
// fan-out, the proposals and the judging that produce it.

/**
 One editor's proposal for a chunk.

 @example
 ```ts
 const candidate: EditorCandidate = { modelId, patch, };
 ```
 */
export type EditorCandidate = {
  /**
   Model that produced this proposal.
   */
  readonly modelId: RosterModelId;

  /**
   Apply-gate outcome of the operations it proposed.
   */
  readonly patch: PatchOutcome;
};

/**
 What per-envelope selection assembled, with the counts that say how much of
 the composite was actually voted on.

 @example
 ```ts
 const { operations, contributors, } = await selectPerEnvelope({ ... },);
 ```
 */
export type EnvelopeSelection = {
  /**
   Winning operation per envelope, in envelope order.
   */
  readonly operations: readonly PatchOperation[];

  /**
   Models whose operations the composite carries, in first-win order.
   */
  readonly contributors: readonly RosterModelId[];

  /**
   Envelopes adopted without a vote because only one editor proposed for
   them, counting envelopes where every proposal was identical.
   */
  readonly soleCount: number;

  /**
   Envelopes decided by a judged vote.
   */
  readonly judgedCount: number;

  /**
   Envelopes left unedited because judges declined every proposal.
   */
  readonly declinedCount: number;

  /**
   Degradation findings from every judge fan-out this pass ran.

   Carried up rather than logged because the caller writes findings into the
   per-entry artifact, and a log line only exists if something captured it.
   `soleCount`, `judgedCount` and `declinedCount` say how many envelopes were
   decided which way; they do
   not say which judge went silent, and that identity is what every
   voice-loss diagnosis has turned on.
   */
  readonly findings: readonly string[];

  /**
   Every judged envelope round, ballots and all.

   Envelopes adopted without a vote are absent rather than recorded empty:
   no judge was asked, so there is no reasoning to keep, and `soleCount`
   already says how many went that way.
   */
  readonly rounds: readonly RepairJudgedRound[];
};

/**
 Patch that ships, with the findings from judging it.

 Wrapped rather than widening `PatchOutcome`, which is shared across the apply
 path: putting a telemetry field there would attach it to every operation
 result in the pipeline. The wrapper keeps the reporting local to the stage
 that produced it.

 @example
 ```ts
 const { patch, findings, } = await selectChunkPatch({ client, candidates, ... },);
 ```
 */
export type ChunkPatchSelection = {
  /**
   Winning patch, or the fallback when judges decline.
   */
  readonly patch: PatchOutcome;

  /**
   Degradation findings from the judge fan-out, empty when no vote was held.
   */
  readonly findings: readonly string[];

  /**
   Ballots of the whole-chunk round, empty when no vote was held.
   */
  readonly rounds: readonly RepairJudgedRound[];

  /**
   Who wrote {@link ChunkPatchSelection.patch}, absent when the untouched
   translation ships and nobody wrote anything.

   RECORDED RATHER THAN RECONSTRUCTED FROM THE ROUNDS. A declined round holds
   ballots and no winner, yet one disposition still ships a real editor's
   patch, so a reader of the rounds alone cannot name that editor: the
   information is not in its input. The stage knows the answer at every exit
   and states it here, which is what lets the self-certification discount
   reach a checker judging text it wrote itself.
   */
  readonly shippedProducer: ShippedProducer;
};

//endregion Editor selection result
