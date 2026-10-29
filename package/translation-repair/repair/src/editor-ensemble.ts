import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  applyPatchOperations,
  type PatchOperation,
  type PatchOutcome,
  type PreservationMode,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type Candidate,
  describeProducer,
  producerModelIds,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { selectBestCandidate, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import {
  CHUNK_DECLINE_CONSEQUENCE,
  repairSelectionSourceEvidence,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import { collectEnvelopeProposals, } from './editor-proposals.ts';
import {
  CHUNK_SELECTION_CRITERIA,
  CHUNK_SELECTION_TASK,
  ENVELOPE_SELECTION_CRITERIA,
  ENVELOPE_SELECTION_TASK,
} from './editor-selection-sheet.ts';
import {
  type ChunkPatchSelection,
  type EditorCandidate,
  type EnvelopeSelection,
  NOBODY_WROTE_IT,
} from './editor-selection-result.ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type {
  EditableEnvelope,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  CHUNK_SCOPE_ENVELOPE,
  describeAdoptedRound,
  describeJudgedRound,
  type RepairJudgedRound,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import { envelopeContext, } from './editor-envelope-context.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

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
//
// Proposals are read from each candidate's APPLIED operations, not from what its
// model proposed. The deterministic gate rejects stale hashes, drifted
// envelopes, and no-op replacements, so judging pre-gate text would spend model
// calls choosing between operations that cannot ship.

/**
 Chooses one replacement text per envelope by judging the distinct proposals
 models made for it, then assembles the winners into one operation set.

 An envelope only one model proposed for needs no vote: there is nothing to
 compare it against, and it still faces the chunk-level judges afterwards.

 @param client - injected model client

 @param candidates - editor proposals, in roster order

 @param envelopes - envelopes of this chunk

 @param judgeModelIds - whole roster; a judge that wrote a candidate still votes, at
 the self-vote weight for that candidate

 @param sourceText - original chunk text, evidence for judges

 @param targetText - translation chunk text, for the surrounding context each
 replacement has to fit

 @param neighbouringSourceText - local factual evidence for the replacement

 @param documentSourceText - same-entry original evidence beyond the local window

 @param identityContext - declared names and handles, for the judges (ledger S14)

 @param referenceContext - what the pages the original cites say (ledger S14)

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Composite operation set built from per-envelope winners

 @example
 ```ts
 const composite = await selectPerEnvelope({ client, candidates, envelopes, ... },);
 ```
 */
export async function selectPerEnvelope(
  {
    client,
    candidates,
    envelopes,
    judgeModelIds,
    sourceText,
    targetText,
    neighbouringSourceText,
    documentSourceText,
    identityContext,
    referenceContext,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly candidates: readonly EditorCandidate[];
    readonly envelopes: readonly EditableEnvelope[];
    readonly judgeModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly targetText: string;
    readonly neighbouringSourceText?: string;
    readonly documentSourceText?: string;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<EnvelopeSelection> {
  /**
   Logger tagged with this selection pass.
   */
  const el = tagged({
    tag: selectPerEnvelope.name,
    l,
  },);

  /**
   Winning operation per envelope, filled in envelope order.
   */
  const winners: PatchOperation[] = [];

  /**
   Contributing models in first-win order, deduplicated by `seen`.
   */
  const contributors: RosterModelId[] = [];

  /**
   Models already credited as contributors.
   */
  const seen = new Set<RosterModelId>();

  /**
   Degradation findings gathered from every envelope's judge fan-out.
   */
  const selectionFindings: string[] = [];

  /**
   Ballots of every envelope round that reached the judges.
   */
  const rounds: RepairJudgedRound[] = [];

  /**
   How each envelope was decided.
   */
  const counters = {
    sole: 0,
    judged: 0,
    declined: 0,
  };
  for (const envelope of envelopes) {
    /**
     Every distinct APPLIED proposal for this envelope, each credited to
     every model that wrote it.
     */
    const proposals = collectEnvelopeProposals({
      candidates,
      envelope,
    },);

    if (proposals.length === 0)
      continue;

    /**
     Sole distinct proposal, adopted without a vote because there is nothing
     to compare it against; chunk-level judging still sees it.
     */
    const [sole,] = proposals;
    if ((proposals.length === 1) && (sole !== undefined)) {
      counters.sole += 1;
      winners.push(sole.value,);
      // Recorded as a round of its own kind, so the authors of what shipped
      // survive into the attribution and the artifact.
      rounds.push(describeAdoptedRound({
        stage: 'envelope',
        envelopeId: envelope.envelopeId,
        candidate: sole,
      },),);
      for (const modelId of producerModelIds(sole.producer,)) {
        if (seen.has(modelId,))
          continue;
        seen.add(modelId,);
        contributors.push(modelId,);
      }
      continue;
    }

    /* oxlint-disable no-await-in-loop -- current envelope winners mutate ordered attribution state; replacement DAG must separate concurrent selection from ordered reduction */
    /**
     Judges verdict over the distinct proposals for this envelope.
     */
    const outcome = await selectBestCandidate({
      client,
      candidates: proposals,
      judgeModelIds,
      sourceText,
      task: ENVELOPE_SELECTION_TASK,
      criteria: ENVELOPE_SELECTION_CRITERIA,
      evidence: [
        {
          label: 'ORIGINAL (Chinese)',
          text: sourceText,
        },
        {
          label: 'PASSAGE BEING REPLACED (current English)',
          text: envelope.baseText,
        },
        {
          label: 'SURROUNDING ENGLISH, for register and tense only',
          text: envelopeContext({
            targetText,
            envelope,
          },),
        },
        ...repairSelectionSourceEvidence({
          ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
          ...((documentSourceText === undefined) ? {} : { documentSourceText, }),
          ...((identityContext === undefined) ? {} : { identityContext, }),
          ...((referenceContext === undefined) ? {} : { referenceContext, }),
        },),
      ],
      signal,
      perCallTimeoutMs,
      l,
      stageSettings,
    },);
    /* oxlint-enable no-await-in-loop */
    selectionFindings.push(...outcome.findings,);
    rounds.push(describeJudgedRound({
      stage: 'envelope',
      envelopeId: envelope.envelopeId,
      candidates: proposals,
      outcome,
    },),);
    if (outcome.kind === 'declined') {
      counters.declined += 1;
      el.info(
        `envelope ${envelope.envelopeId}: ${outcome.reason}; leaving it unedited in the composite`,
      );
      continue;
    }
    counters.judged += 1;
    winners.push(outcome.value,);
    for (const modelId of producerModelIds(outcome.producer,)) {
      if (seen.has(modelId,))
        continue;
      seen.add(modelId,);
      contributors.push(modelId,);
    }
  }
  return {
    operations: winners,
    contributors,
    soleCount: counters.sole,
    judgedCount: counters.judged,
    declinedCount: counters.declined,
    findings: selectionFindings,
    rounds,
  };
}

/**
 Judges whole-chunk candidates and returns the patch that ships.

 @param client - injected model client

 @param candidates - whole-chunk proposals including the composite

 @param judgeModelIds - whole roster; a judge that wrote a candidate still votes, at
 the self-vote weight for that candidate

 @param sourceText - original chunk text, evidence for judges

 @param neighbouringSourceText - local factual evidence for repaired details

 @param documentSourceText - same-entry original evidence beyond the local window

 @param identityContext - declared names and handles, for the judges (ledger S14)

 @param referenceContext - what the pages the original cites say (ledger S14)

 @param indecisionFallback - patch adopted when judges answered but failed to
 converge; callers must pass a patch that actually repairs something

 @param rejectionFallback - patch adopted when judges affirmatively found no
 candidate acceptable; normally the untouched translation

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Winning patch, or the fallback when judges decline, plus the
 judge fan-out findings for the caller to carry into the artifact

 @example
 ```ts
 const patch = await selectChunkPatch({ client, candidates, fallback, ... },);
 ```
 */
export async function selectChunkPatch(
  {
    client,
    candidates,
    judgeModelIds,
    sourceText,
    neighbouringSourceText,
    documentSourceText,
    identityContext,
    referenceContext,
    indecisionFallback,
    rejectionFallback,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly candidates: readonly Candidate<PatchOutcome>[];
    readonly judgeModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly neighbouringSourceText?: string;
    readonly documentSourceText?: string;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly indecisionFallback: Candidate<PatchOutcome>;
    readonly rejectionFallback: PatchOutcome;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ChunkPatchSelection> {
  /**
   Logger tagged with this selection pass.
   */
  const cl = tagged({
    tag: selectChunkPatch.name,
    l,
  },);

  /**
   Sole candidate, which needs no vote.

   Callers deduplicate by rendered text before calling, so one candidate here
   means every editor and the composite agreed on the same text. That is
   unanimity rather than an unexamined survivor, and the text still faces the
   resolution checkers and the unchanged-versus-repaired selection after this.
   */
  const [sole,] = candidates;
  if ((candidates.length === 1) && (sole !== undefined)) {
    cl.info(`every proposal was identical; shipping ${describeProducer(sole.producer,)} unjudged`,);
    // No judges were asked, so there is no fan-out to report on.
    return {
      patch: sole.value,
      findings: [],
      rounds: [],
      shippedProducer: sole.producer,
    };
  }

  /**
   Judges verdict over the whole-chunk candidates.
   */
  const outcome = await selectBestCandidate({
    client,
    candidates,
    judgeModelIds,
    sourceText,
    task: CHUNK_SELECTION_TASK,
    criteria: CHUNK_SELECTION_CRITERIA,
    // WHAT DECLINING DOES HERE, which the shared promise misstates: only a
    // unanimous decline keeps the existing English (ledger L14(c)).
    declineConsequence: CHUNK_DECLINE_CONSEQUENCE,
    evidence: [
      {
        label: 'ORIGINAL (Chinese)',
        text: sourceText,
      },
      ...repairSelectionSourceEvidence({
        ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
        ...((documentSourceText === undefined) ? {} : { documentSourceText, }),
        ...((identityContext === undefined) ? {} : { identityContext, }),
        ...((referenceContext === undefined) ? {} : { referenceContext, }),
      },),
      {
        label: 'EXISTING ENGLISH BEFORE REPAIR, comparison context for what was already carried and what each candidate changes; not independent factual authority',
        text: rejectionFallback.patchedText,
      },
    ],
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);
  /**
   This round's ballots, recorded before any branch, so a refusal keeps the
   reasoning that produced it exactly as a win does.
   */
  const rounds = [
    describeJudgedRound({
      stage: 'chunk-patch',
      envelopeId: CHUNK_SCOPE_ENVELOPE,
      candidates,
      outcome,
    },),
  ];
  if (outcome.kind === 'declined') {
    // Judges failing to RANK the candidates says nothing against any of them,
    // and dropping every repair over that would turn a disagreement about
    // wording into a recall loss. The repair is not unexamined either: the
    // checkers and then `selectRepairCandidate` still make it beat the
    // untouched text on measurements before it can ship.
    //
    // Judges REJECTING every candidate is the opposite: a substantive verdict
    // that none of this is good enough. Shipping a repair over that would
    // overrule the panel rather than route around its silence.
    if (outcome.disposition === 'indecision') {
      cl.info(`${outcome.reason}; shipping the strongest repair anyway`,);
      return {
        patch: indecisionFallback.value,
        findings: outcome.findings,
        rounds,
        shippedProducer: indecisionFallback.producer,
      };
    }
    cl.info(`${outcome.reason}; shipping no repair for this chunk`,);
    return {
      patch: rejectionFallback,
      findings: outcome.findings,
      rounds,
      // The untouched translation ships, so no editor wrote it and no checker
      // can be judging its own work here.
      shippedProducer: NOBODY_WROTE_IT,
    };
  }
  cl.info(
    `chunk patch from ${describeProducer(outcome.producer,)} won weight ${String(outcome.voteWeight,)} of ${
      String(outcome.tally
        .ballots,)
    } ${
      wordForCount({
        count: outcome.tally
          .ballots,
        one: 'ballot',
        many: 'ballots',
      },)
    }`,
  );
  return {
    patch: outcome.value,
    findings: outcome.findings,
    rounds,
    shippedProducer: outcome.producer,
  };
}

/**
 Applies one candidate's operations, so a composite can be scored the same way
 a model's own proposal is.

 @param targetText - translation chunk text

 @param envelopes - envelopes of this chunk

 @param operations - operations to apply

 @returns Apply-gate outcome

 @example
 ```ts
 const patch = applyCandidate({ targetText, envelopes, operations, preservation, },);
 ```
 */
export function applyCandidate(
  {
    targetText,
    envelopes,
    operations,
    preservation,
  }: {
    readonly targetText: string;
    readonly envelopes: readonly EditableEnvelope[];
    readonly operations: readonly PatchOperation[];
    readonly preservation: PreservationMode;
  },
): PatchOutcome {
  return applyPatchOperations({
    targetText,
    envelopes,
    operations,
    preservation,
  },);
}

//endregion Editor ensemble
