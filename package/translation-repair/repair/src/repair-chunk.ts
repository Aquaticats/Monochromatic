import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { recordIssuesWithFilers, } from './claim-filers.ts';
import {
  aggregateClaims,
  dedupeAcceptedIssues,
  deriveEditableEnvelopes,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { runChunkCriticPhase, } from './chunk-critic-phase.ts';
import {
  candidateConfirmedIssueIds,
  readAppliedEnvelopes,
} from './chunk-measure.ts';
import { buildEditorAddendum, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { chunkEvidence, } from './repair-chunk-evidence.ts';
import { unchangedChunkOutcome, } from './repair-unchanged-outcome.ts';
import type { ChunkRepairOutcome, } from '@monochromatic-dev/translation-repair-assembly/ts';
import {
  assertCheckerBench,
  standingSeating,
} from './repair-checker-reseat.ts';
import { proveSheddingWorseVoted, } from './repair-worse-strip.ts';
import { runEditorStage, } from './repair-editor-stage.ts';
import { foldStageFindings, } from './repair-stage-findings.ts';
import {
  criticOriginalsOf,
  runPanelStage,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import { screenAttestedAdditions, } from '@monochromatic-dev/translation-repair-reference/ts';
import { settleShippedPatch, } from './repair-chunk-settle.ts';
import type { RepairChunkRequest, } from './repair-chunk-request.ts';

//region Chunk repair
// One chunk pair through the whole loop: critics, aggregation, panel,
// envelopes, editor, apply gate, checkers, the introduced-defect probe,
// measurement, selection. Every early exit returns the chunk unchanged with
// whatever issues were decided; the unchanged text always competes and wins by
// default.
// The roster and outcome types live in repair-contract.ts, and the critic stage
// with its vote screening in chunk-critic-phase.ts.

/**
 Runs one chunk pair through the whole repair loop.

 @param client - injected model client

 @param sliceIndex - chunk position carried onto the outcome

 @param sourceText - original chunk text

 @param targetText - translation chunk text

 @param lineStructured - whether the ENCLOSING chunk's original is
 line-structured, decided by the caller because a slice is too small a unit to
 decide it on; see `buildEditorAddendum`

 @param models - role roster

 @param reseat - reads the seating again at the checker stage, so a chunk
 in flight when a provider runs dry asks the bench a fresh reading seats
 (class one hundred nine)

 @param identityContext - declared names from both sides' front matter,
 passed down from the whole document because chunk text carries no front
 matter of its own

 @param referenceContext - what the original's cited pages say, shown to
 the critic and the panel so a detail the archive took from a reference is
 not deleted as an addition (class thirty-five), and to the checkers so one
 is not counted as damage (ledger L14)

 @param attestedDetails - archive details a cited reference states, attested
 word for word at preparation; an addition claim on one is rejected before
 the panel (class thirty-seven)

 @param declaredNames - same declarations as strings to compare rather than
 prose to read, which is a different job: one tells a model what is true, the
 other decides whether a patch may ship

 @param declaredSubjectForms - the declared forms among them that name the subject, which may stand in for one another

 @param neighbouringSourceText - original of the passages either side, shown to
 the critic, panel and editor as CONTEXT they may neither quote against nor
 edit. Relocation is why it exists: shown one slice alone, a critic reads a passage
 the archive carried in from next door as an addition with no source, and the
 editor then removes wording that the document does need, just not here

 @param neighbouringIncumbentText - archive English of those same two passages,
 which is the half that shows the relocation rather than merely the subject:
 the original says each thing once in its own place while the archive says it
 next door

 @param documentSourceText - same-entry factual evidence for panels and repair selectors, not extra coverage

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Chunk outcome with the winning text

 @example
 ```ts
 const outcome = await repairChunk({ ... },);
 ```
 */
export async function repairChunk(
  {
    client,
    sliceIndex,
    sourceText,
    targetText,
    lineStructured,
    models,
    reseat = standingSeating,
    identityContext,
    referenceContext,
    attestedDetails = [],
    declaredNames,
    declaredSubjectForms,
    neighbouringIncumbentText,
    neighbouringSourceText,
    documentSourceText,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<RepairChunkRequest>,
): Promise<ChunkRepairOutcome> {
  assertCheckerBench({ models, },);

  /**
   Window and parsed pair every stage of this chunk reads (`chunkEvidence`).
   */
  const {
    windowFragment,
    documents,
  } = chunkEvidence({
    sourceText,
    targetText,
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
  },);

  /**
   Critics plus the deterministic screen over their non-translation votes.
   */
  const critic = await runChunkCriticPhase({
    client,
    criticModelIds: models.criticModelIds,
    sourceText,
    targetText,
    documents,
    ...(identityContext === undefined ? {} : { identityContext, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
    ...windowFragment,
    sliceIndex,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   Unchanged outcome shared by every early exit.
   */
  const unchangedOutcome = unchangedChunkOutcome({
    sliceIndex,
    targetText,
    critic,
  },);
  // STANDING NON-TRANSLATION VOTES NO LONGER END THE SLICE. They used to return
  // here with the input unchanged, which threw away whatever this chunk's repair
  // would have produced. Question 3 answer B keeps the critics as EVIDENCE for
  // the judges and removes every early return they owned:
  // `doc/decision/translation-repair-question-answers.md`.
  //
  // WHY IT MATTERED MOST ON THE SLICES THE LANE EXISTS FOR. On a sparse target,
  // a chunk whose critics call it untranslated is the common case rather than
  // the rare one, so the exit fired exactly where the work was most needed and
  // discarded it.
  //
  // NOTHING IS LOST BY PROCEEDING: `nonTranslationStanding` rides on the outcome
  // either way and `critic.findings` are folded into `stageFindings`, so a
  // reader still learns the votes stood. What changes is that the votes now
  // inform rather than decide.
  if (critic.votesStand) {
    l.warn(
      `chunk ${String(sliceIndex,)}: ${
        String(critic.nonTranslationVotes,)
      } non-translation ${
        wordForCount({
          count: critic.nonTranslationVotes,
          one: 'vote stands',
          many: 'votes stand',
        },)
      }; proceeding, votes carried as evidence`,
    );
  }
  /**
   Claims after the reference screen (class thirty-seven): an addition claim
   on an attested archive detail is recorded rejected here and never reaches
   the panel or the editor.
   */
  const screened = screenAttestedAdditions({
    claims: critic.claims,
    attested: attestedDetails,
    targetText,
    // The attested quotes were placed in the archive as its sheet showed it, cut against the whole source page
    // (`attestPassReferences` is given the page's source). `documentSourceText` is that page
    // (`prepared.sourceText` in `repair-slice-settle.ts`), so a comment of the original that sits in another
    // chunk stays in view here as it did there; the chunk's own originals are added for a caller that gives none,
    // and only then does this cut differ from the attestation's.
    original: [
      criticOriginalsOf({
        sourceText,
        ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
      },),
      documentSourceText ?? '',
    ].join('\n',),
  },);
  for (const finding of screened.findings)
    l.info(`chunk ${String(sliceIndex,)}: ${finding}`,);
  if (screened.claims
    .length
    === 0) {
    l.info(`chunk ${String(sliceIndex,)}: no validated claims, unchanged`,);
    return {
      ...unchangedOutcome,
      issues: screened.issues,
      findings: [
        ...critic.findings,
        ...screened.findings,
      ],
    };
  }

  /**
   Merge-proposal clusters over the validated claims.
   */
  const { clusters, } = aggregateClaims({ claims: screened.claims, },);

  /**
   Panel decision over the clusters.
   */
  const panel = await runPanelStage({
    ...((documentSourceText === undefined) ? {} : { documentSourceText, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
    ...((identityContext === undefined) ? {} : { identityContext, }),
    client,
    panelModelIds: models.panelModelIds,
    sourceText,
    targetText,
    ...windowFragment,
    clusters,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   The unchanged outcome of an exit past the panel, which also says whether the panel heard nobody on some packet:
   a settlement the panel was deaf to reads "nothing to edit" and would be cached as examined.
   */
  const unchangedAfterPanel = {
    ...unchangedOutcome,
    silentStages: panel.silentStages,
  };

  /**
   Panel issues with same-place accepted duplicates merged into one.

   Applied HERE, before envelopes are cut, because the cost a duplicate
   imposes is the editor repairing one defect twice and cutting two
   overlapping envelopes for it. Deduplicating after that work is done would
   correct the arithmetic and keep the waste.
   */
  const deduped = dedupeAcceptedIssues({ issues: panel.issues, },);

  /**
   Every issue this chunk records with its filers attached and logged: the
   reference screen's rejections first, then the panel's.
   */
  const recordedIssues = recordIssuesWithFilers({
    sliceIndex,
    issues: [
      ...screened.issues,
      ...deduped.issues,
    ],
    attributions: critic.claimAttributions,
    l,
  },);

  /**
   Findings across the stages so far.
   */
  const stageFindings = foldStageFindings({
    critic,
    screened: screened.findings,
    panel: panel.findings,
    deduped: deduped.findings,
  },);

  /**
   Envelopes cut from accepted issues.
   */
  const {
    envelopes,
    unenveloped,
  } = deriveEditableEnvelopes({
    issues: deduped.issues,
    targetText,
  },);
  if (envelopes.length === 0) {
    l.info(`chunk ${String(sliceIndex,)}: nothing to edit, unchanged`,);
    return {
      ...unchangedAfterPanel,
      issues: recordedIssues,
      findings: stageFindings,
    };
  }

  /**
   Accepted issues, the editor's and checkers' work list.
   */
  const acceptedIssues = deduped.issues
    .filter(function isAccepted(issue,) {
      return issue.status === 'accepted';
    },);

  /**
   Editor rules for this slice, with the line-structure fact appended when the
   enclosing chunk's ORIGINAL is line-structured.
   */
  const editorAddendum = buildEditorAddendum({
    baseAddendum: models.editorRuleAddendum ?? '',
    lineStructured,
  },);

  /**
   Editor result through the apply gate.
   */
  const editor = await runEditorStage({
    ...((documentSourceText === undefined) ? {} : { documentSourceText, }),
    ...((identityContext === undefined) ? {} : { identityContext, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
    client,
    editorModelIds: models.editorModelIds,
    judgeModelIds: models.judgeModelIds,
    ...((editorAddendum === '') ? {} : { editorRuleAddendum: editorAddendum, }),
    sourceText,
    targetText,
    ...windowFragment,
    envelopes,
    issues: deduped.issues,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);
  if (editor.patch
    .applied
    .length
    === 0) {
    l.info(`chunk ${String(sliceIndex,)}: no operation survived the gate, unchanged`,);
    return {
      ...unchangedAfterPanel,
      issues: recordedIssues,
      rounds: editor.rounds,
      findings: [
        ...stageFindings,
        ...editor.findings,
      ],
    };
  }

  /**
   What the whole patch's envelopes bought: issues eligible to count toward
   the patched candidate, and who wrote the text answering for them. See
   `selectCreditableIssues` for why the rest are excluded.
   */
  const wholeEnvelopes = readAppliedEnvelopes({
    acceptedIssues,
    envelopes,
    editor,
  },);

  /**
   Checker proof and the shadow probe over the patched candidate, on the
   bench as seated at the stage (class one hundred nine); an edit the
   checkers did not confirm and two or more voted worse is stripped, and the
   reduced patch proved once more (ledger L3, the owner's ruling of
   2026-09-28, raised from one worse ballot on 2026-10-25).
   */
  const {
    editor: shipped,
    appliedEnvelopes,
    proof: {
      checker,
      repairRegions,
      introducedDefects,
    },
    findings: shedFindings,
  } = await proveSheddingWorseVoted({
    client,
    models,
    reseat,
    sourceText,
    targetText,
    envelopes,
    editor,
    acceptedIssues,
    authorship: wholeEnvelopes.authorship,
    neighbours: windowFragment,
    identityContext: identityContext ?? '',
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);
  if (shipped.patch
    .applied
    .length
    === 0) {
    l.info(`chunk ${String(sliceIndex,)}: every edit was stripped as worse-voted, unchanged`,);
    return {
      ...unchangedAfterPanel,
      issues: recordedIssues,
      rounds: editor.rounds,
      findings: [
        ...stageFindings,
        ...editor.findings,
        ...shedFindings,
      ],
    };
  }

  /**
   Which candidate won, whether the returned text moved at all, which issues
   the checkers confirmed, and the declared-name refusal the patch owes
   (`repair-chunk-settle.ts`).
   */
  const {
    repairedText,
    patchSelected,
    changed,
    resolvedIssueIds,
    refusal,
  } = settleShippedPatch({
    sliceIndex,
    declaredNames,
    ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
    targetText,
    shipped,
    appliedEnvelopes,
    tallies: checker.tallies,
    envelopes,
    targetDocument: documents.target,
    sourceDocument: documents.source,
    acceptedCount: acceptedIssues.length,
    unenvelopedCount: unenveloped.length,
    l,
  },);

  return {
    sliceIndex,
    repairedText,
    changed,
    issues: recordedIssues,
    resolvedIssueIds: changed ? resolvedIssueIds : [],
    checkerReadings: checker.readings,
    // THE NATURALNESS LANE'S FIELD. It rewrites text that already passed a
    // checker round; a worse-voted strip's recheck here (ledger L3) is the
    // proof of the patch that ships, so it fills `checkerReadings` instead.
    recheckReadings: {},
    candidateResolvedIssueIds: candidateConfirmedIssueIds({
      acceptedIssues,
      tallies: checker.tallies,
    },),
    repairRegions,
    authorship: appliedEnvelopes.authorship,
    introducedDefects,
    accuracyPatchSelected: patchSelected,
    refined: false,
    rounds: editor.rounds,
    ...refusal.record,
    nonTranslationVotes: critic.nonTranslationVotes,
    nonTranslationContradicted: critic.contradicted,
    nonTranslationStanding: critic.votesStand,
    heardCritics: critic.heardCritics,
    heardCriticIds: critic.heardCriticIds,
    claimAttributions: critic.claimAttributions,
    silentStages: panel.silentStages,
    findings: [
      ...stageFindings,
      ...editor.findings,
      ...checker.findings,
      ...shedFindings,
      // The probe fans out to a roster like every other stage and loses voices
      // like every other stage, and its findings were dropped here. That is
      // the exact complaint of the prober calibration work: a quiet stage
      // reads identically to a clean run, so a prober that went silent looked
      // like a prober that found nothing.
      ...introducedDefects.findings,
      ...refusal.findings,
    ],
  };
}

//endregion Chunk repair
