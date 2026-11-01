import type {
  EditableEnvelope,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  RepairJudgedRound,
  RepairSlateEntry,
} from '../dist/final/node/index.mjs';

//region Issue author fixtures
// AN EDITABLE ENVELOPE, A SLATE ENTRY, AND THE ENVELOPE ROUND THAT PICKED ONE
// OF ITS CANDIDATES, for cases that build editor stage results and check
// which model's work a shipped patch credits.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The issue-author-scope and issue-authors
// tests kept their own copy of these builders; both now import them from
// here.

/**
 Builds an editable envelope carrying the issues it serves.

 @param envelopeId - envelope's own identifier

 @param issueIds - issues the envelope serves

 @returns Envelope shaped as the editor stage emits one

 @example
 ```ts
 const envelope = envelopeOf({ envelopeId: 'envelope-1', issueIds: ['adjudicated/whisker',], },);
 ```
 */
export function envelopeOf(
  {
    envelopeId,
    issueIds,
  }: {
    readonly envelopeId: string;
    readonly issueIds: readonly string[];
  },
): EditableEnvelope {
  return {
    envelopeId,
    startOffset: 0,
    endOffset: 1,
    baseText: 'the cat naps',
    baseHash: `hash-${envelopeId}`,
    issueIds,
  };
}

/**
 Builds one slate entry, whose `index` is the ONE-BASED number judges saw and
 need not match its position in the array.

 @param index - one-based ballot number the slate entry carries

 @param modelId - model that produced this candidate

 @returns Slate entry shaped as a judged round carries one

 @example
 ```ts
 const entry = slateEntryOf({ index: 1, modelId: AUTHOR, },);
 ```
 */
export function slateEntryOf(
  {
    index,
    modelId,
  }: {
    readonly index: number;
    readonly modelId: RosterModelId;
  },
): RepairSlateEntry {
  return {
    index,
    rendered: `candidate ${String(index,)}`,
    hash: `slate-${String(index,)}`,
    producer: {
      kind: 'model',
      modelId,
    },
  };
}

/**
 Builds an envelope round that picked the candidate carrying `selectedIndex`.

 @param envelopeId - envelope the round settled

 @param slate - candidates the round judged

 @param selectedIndex - one-based index of the candidate picked

 @returns Judged round recording that selection

 @example
 ```ts
 const round = selectedRound({ envelopeId: 'envelope-1', slate: [entry,], selectedIndex: 1, },);
 ```
 */
export function selectedRound(
  {
    envelopeId,
    slate,
    selectedIndex,
  }: {
    readonly envelopeId: string;
    readonly slate: readonly RepairSlateEntry[];
    readonly selectedIndex: number;
  },
): RepairJudgedRound {
  return {
    kind: 'selected',
    stage: 'envelope',
    envelopeId,
    slate,
    ballots: [],
    tally: {
      judgesAvailable: 3,
      ballots: 3,
      abstentions: 0,
      selfVotes: 0,
    },
    perCandidate: [],
    selectedIndex,
    voteWeight: 2,
  };
}

//endregion Issue author fixtures
