import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { describeProducer, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { selectBestCandidate, } from './candidate-select-record.ts';
import { declineConsequenceFor, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  translatedSlateBreakEvidence,
  translatedSlateCriteria,
} from './translated-slate-criteria.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  blankAgainst,
  BlankSelectionError,
  describeSlate,
  type IncumbentKind,
  NOT_ON_SLATE,
  positionOf,
  rotateCandidates,
  TranslateAbsenceError,
  type TranslateCandidateValue,
  type TranslateJudgeResponsibility,
  translateSelectionTask,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { runoffFinalists, } from './translate-runoff.ts';
import { settleAbsentDecline, } from './translate-runoff-tie.ts';
import type { ProducedSlate, } from './translate-produce.ts';
import {
  EMPTY_TALLY,
  type TranslateStageResult,
} from './translate-stage-result.ts';
import { translateSlateEvidence, } from './translate-slate-evidence.ts';
import type {
  FanOutMode,
  StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Translate judge

// The half of the translate stage that CHOOSES: a slate that already exists is
// put to the judges, and one candidate ships.
//
// TAKES ITS SLATE RATHER THAN MAKING ONE, which is the whole reason for the
// split. Everything here is a question about a fixed set of texts, so the same
// slate can be asked more than once and the answers differ only in what the
// caller varied. The window trial varies the evidence the judges are shown;
// the judge-quality bench's position-bias attempt wants to vary ballot position. Neither is expressible
// against a stage that reproduces its candidates on every call, because a second
// call would resample them.
//
// The rosters are NOT checked here: `assertJudgeableProducerRoster` needs the
// producing side too, so it belongs to whoever holds both.

/**
 Puts an existing slate to the judges and returns what ships.

 @param client - injected model client

 @param produced - slate to judge, from {@link produceTranslateSlate}

 @param judgeModelIds - whole roster selection seats, translators included;
 a ballot for the judge's own rendering counts for less

 @param sourceText - original slice text

 @param incumbentText - translation as it stands, blank where this slice has
 none

 @param incumbentKind - whether there is a translation to fall back on,
 decided by the caller from the target chunk rather than from the text being
 blank: a content span holding only whitespace is the archive's own wording,
 and an anchor is a place where a rendering belongs and none exists

 @param identityContext - declared names from both sides' front matter,
 omitted when neither declares anything

 @param referenceContext - what the pages the original cites say, shown as
 evidence with its rule so accurate archive detail a candidate keeps is not
 read as invention (class thirty-six, 2026-09-16); omitted when it cites none

 @param archiveDisputeNote - why the incumbent shown is the repair lane's
 stand-in and which details are accepted additions, so a candidate leaving
 one out is not read as dropping page content (class one hundred eight,
 2026-09-24); omitted on an undisputed slice

 @param neighbouringSourceText - original of the sections either side, shown as
 CONTEXT the candidates are not expected to render. Absent by default, so a
 caller that does not ask for it gets the sheet production has always sent.
 Relocation is why it exists: where the archive carried a passage across a section
 boundary, a judge shown one slice pair sees invention on one side and omission
 on the other, and the judge-quality bench's alteration arm went from 12 of 16 to 15 of 16 when
 the same trial was given exactly this

 @param neighbouringIncumbentText - archive English of the sections either
 side, shown so a passage missing here can be recognised next door rather than
 read as one the archive never had

 @param responsibility - initial selection or prior-decline challenge

 @param runoff - whether this judging is the challenge round's run-off over
 finalists the prior round backed, which the ballot floor then decides under
 the weight minimum (class seventy-three)

 @param fallbackCondemned - whether a strict majority of the contest's ballots
 called the incumbent flawed, so a leader that at least two ballots named
 wins under the weight minimum although no run-off narrowed the question; the
 consolidation sets it over an eligible standing (one entry's slice 45,
 2026-10-19). Read by the challenge round alone: the first round still
 declines under the minimum, so a tie that backed two candidates is still run
 off over them (slice 34) and not decided on the first plurality

 @param withheldStanding - whether an absent incumbent stands for wording
 that exists and cannot ship (a standing the deterministic rule refused, an
 archive rendering the floor refused or the adjudicators disputed) rather
 than a passage with none, which the judges are told (owner, 2026-09-27,
 "Preference + polish")

 @param shipPastDecline - whether a challenge round declined with nothing
 left to narrow ships its preferred candidate; defaults to the withheld
 standing, and a caller with a further round of its own (the translate
 lane's follow-up production) defers it to that round

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for, since
 the window is rotated by the sheet text and a sheet edit elsewhere moves
 which seats it holds

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Shipped text with how it was decided

 @throws {@link TranslateAbsenceError} when a slice with no incumbent produced
 nothing to write, which every fallback here would otherwise report as a
 settled slice carrying the archive's own wording, of which there is none;
 withheld wording that exists stands instead on the round that ships past a
 decline, where translators were heard (ledger B39)

 @throws {@link BlankSelectionError} when selection chose text that says
 nothing for a source that says something, in EITHER mode, since that is a
 deletion rather than an outcome

 @example
 ```ts
 const decided = await judgeTranslateSlate({ client, produced, judgeModelIds, ... },);
 ```
 */
export async function judgeTranslateSlate(
  {
    client,
    produced,
    judgeModelIds,
    sourceText,
    incumbentText,
    incumbentKind,
    identityContext,
    lifeEnded,
    subjectNames,
    referenceContext,
    archiveDisputeNote,
    neighbouringIncumbentText,
    neighbouringSourceText,
    pictureContext,
    syntax,
    lineStructured,
    responsibility = 'initial-selection',
    runoff = false,
    fallbackCondemned = false,
    withheldStanding = false,
    shipPastDecline = withheldStanding,
    signal,
    perCallTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly produced: ProducedSlate;
    readonly judgeModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly incumbentKind: IncumbentKind;
    readonly identityContext?: string;
    /**
     Whether the original's entry records a death, which adds the tense
     candidates of the ballot texts to the slate's evidence.
     */
    readonly lifeEnded?: boolean;
    /**
     Names the translation declares for its subject, read by the tense
     candidates as named subjects.
     */
    readonly subjectNames?: readonly string[];
    readonly referenceContext?: string;
    readonly archiveDisputeNote?: string;
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly pictureContext?: string;
    readonly syntax?: SliceSyntax;

    /**
     Whether the enclosing chunk is governed by the verse rule.

     REQUIRED RATHER THAN OPTIONAL, which is the lesson the consolidation taught. The
     consolidation producers carried this same fact as an optional field for a
     day, no caller ever set it, and every verse passage was quietly told it
     was prose. An optional flag here would fail exactly that way again, and
     silently: the sheet would render, the judges would answer, and nothing
     anywhere would report that the rule went missing.
     */
    readonly lineStructured: boolean;
    readonly responsibility?: TranslateJudgeResponsibility;
    readonly runoff?: boolean;
    readonly fallbackCondemned?: boolean;
    readonly withheldStanding?: boolean;
    readonly shipPastDecline?: boolean;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<TranslateStageResult> {
  /**
   Logger tagged with this half.
   */
  const tl = tagged({
    tag: judgeTranslateSlate.name,
    l,
  },);

  /**
   Candidates in the order the judges will see them, rotated by the slice so
   the incumbent does not sit in one position across a document.
   */
  const rotated = rotateCandidates({
    candidates: produced.candidates,
    sourceText,
  },);

  /**
   That slate as a record, so a ballot naming a position can be read later.

   Derived from the SAME array the judges are shown rather than by rotating a
   second time. Two rotations agree today because the rotation is a pure
   function of the slice; they would stop agreeing the moment either call site
   changed, and the failure would be silent, since a ballot index is a valid
   index either way.
   */
  const slate = describeSlate({ candidates: rotated, },);

  /**
   The incumbent as it stands on the ballot, carrying every model that
   reproduced it exactly.

   Read off the slate rather than written fresh. Writing `matched: []` here
   erased the collapse on precisely the declined rounds where knowing that
   three models independently produced the archive's wording is the whole
   evidence that keeping it was right.
   */
  const incumbentOnSlate = slate.find(function isIncumbent(entry,): boolean {
    return entry.origin === 'incumbent';
  },);

  /**
   Shipping the slice exactly as it stands, which every failure path returns.

   Not an error while the archive HAS a translation here: leaving one as it
   stands is the state the run started in, while shipping text no judge could
   vet is a new claim about the archive. Where the archive has none, every
   path that would return this object refuses instead, since keeping nothing
   is not keeping anything.
   */
  const keepIncumbent: Omit<TranslateStageResult, 'decision' | 'findings'> = {
    text: incumbentText,
    origin: 'incumbent',
    // THE FALLBACK IS NOT A PHANTOM, and what it means is worth stating because
    // it reads like one. An incumbent that says something is always on the
    // slate, so the fallback is reached only where the incumbent says nothing
    // and could not be offered as a candidate. In `present` mode that is a
    // content span holding only whitespace, and the archive's own wording there
    // IS the blank: it stands, and no model matched it. In `absent` mode this
    // object is returned only for wording that exists and cannot ship, on the
    // round that ships past a decline and hears no usable proposal (ledger
    // B39); every other exit that would reach it refuses instead.
    producer: incumbentOnSlate?.producer
      ?? {
        kind: 'incumbent',
        matched: [],
      },
    voteWeight: 0,
    tally: EMPTY_TALLY,
    ballots: [],
    heardTranslators: produced.heardTranslators,
    candidateCount: produced.candidates
      .length,
    slate,
    selectedIndex: NOT_ON_SLATE,
    shippedIndex: positionOf({
      slate,
      text: incumbentText,
    },),
    perCandidate: [],
  };
  if (produced.candidates
    .length
    === 0) {
    /**
     Whether the slate is empty because nobody was heard, rather than because
     everybody was heard and proposed nothing usable.

     TWO DIFFERENT FACTS WEARING ONE WORD until the reason split. Translators that
     answered and proposed nothing a guard would accept are evidence about
     THE PASSAGE, and the gap they leave is one a re-run would probably meet
     again. A slate where every voice was lost is evidence about THE HOUR,
     and its gap is an artefact a later attempt would very likely fill.
     Recorded as one reason, a bad hour left holes in published pages that
     nothing could tell from real ones.
     */
    const nobodyHeard = produced.heardTranslators === 0;

    /**
     Reason and decision naming which of the two happened, kept together so
     the throwing path and the returning path cannot disagree about it.
     */
    const named = nobodyHeard
      ? {
        reason: 'no-voice-heard',
        finding: 'translate-no-voice-heard',
        said: 'no translator was heard at all',
      } as const
      : {
        reason: 'no-candidate',
        finding: 'translate-no-candidate',
        said: 'nothing proposed',
      } as const;

    /**
     Findings this exit reports either way, so the refusal carries the same
     evidence the returned result would have.
     */
    const noCandidateFindings = [
      ...produced.findings,
      named.finding,
    ];
    // NOTHING TO KEEP. With a translation in the archive, silence means it
    // stands and the slice is genuinely settled. With none, the same fallback
    // ships the empty string and reports a settled slice, so the run claims a
    // rendering it never produced for a passage that still has none.
    //
    // WORDING THAT EXISTS AND CANNOT SHIP STANDS (ledger B39) on the round
    // that ships past a decline, where translators were heard and none
    // proposed wording the floor accepts: the owner's rule keeps the archive
    // at a slice no wording passed and ships (2026-09-27, "No valid wording"),
    // where this exit raised, and the slice attempt rethrew the absence for a
    // slice the archive translates, stopping the entry. A slate nobody was
    // heard on still raises: that is the hour, not the passage.
    /**
     Whether the absent incumbent is withheld wording this round keeps.
     */
    const keepsWithheld = shipPastDecline && (!nobodyHeard);
    if ((incumbentKind === 'absent') && (!keepsWithheld)) {
      throw new TranslateAbsenceError({
        reason: named.reason,
        findings: noCandidateFindings,
      },);
    }
    tl.warn(`translate stage: ${named.said}; slice unchanged`,);
    return {
      ...keepIncumbent,
      decision: named.reason,
      findings: noCandidateFindings,
    };
  }

  /**
   Sole proposal, when the slate collapsed to one.
   */
  const [only,] = produced.candidates;

  /**
   Whether the only survivor is the text that was already there.
   */
  const soleIncumbent = (only !== undefined)
    && (produced.candidates
      .length
      === 1)
    && (only.value
      .origin
      === 'incumbent');

  // Shipping unjudged is safe only when nothing could change: the sole
  // survivor IS the incumbent, so the slice ships as it stands whatever the
  // judges would have said, and a fan-out would buy nothing. A sole FRESH
  // candidate is the opposite case and is judged, because the repair pipeline's
  // later safeguards, the resolution checkers and the unchanged-versus-patched
  // selection, do not exist on this path: nothing else would look at it.
  if (soleIncumbent && (only !== undefined)) {
    tl.info(
      `translate stage: every proposal was the incumbent (${
        describeProducer(only.producer,)
      }); shipping it unjudged`,
    );
    return {
      ...keepIncumbent,
      producer: only.producer,
      decision: 'sole-candidate',
      findings: produced.findings,
    };
  }

  /**
   Substantively distinct task after prior panel declined exact slate.
   */
  const task = translateSelectionTask({ responsibility, },);
  /**
   Judges' verdict over the whole-slice candidates.
   */
  const outcome = await selectBestCandidate<TranslateCandidateValue>({
    client,
    candidates: rotated,
    judgeModelIds,
    sourceText,
    // WHAT A DECLINE ACTUALLY COSTS HERE, which is not what the shared sheet
    // says by default. Judges are told declining is safe because the caller
    // keeps text it already trusts; at an anchor there is no such text, so that
    // sentence asks for caution by promising a fallback that does not exist and
    // buys a missing passage with it.
    // Wording that exists and cannot ship is neither: a declined challenge
    // round ships by preference there (owner, 2026-09-27).
    declineConsequence: declineConsequenceFor({
      incumbentKind,
      withheldStanding,
    },),
    task,
    runoff,
    // THE CHALLENGE ROUND ALONE. The first round declining under the minimum
    // is what lets a tie across two candidates be narrowed to a run-off, a
    // narrower question than the first plurality answered (slice 34 of the
    // same entry); only a round that is already the second ask has nothing
    // narrower left to give.
    fallbackCondemned: fallbackCondemned && (responsibility === 'decline-challenge'),
    criteria: translatedSlateCriteria({
      sourceText,
      archiveText: incumbentText,
      lineStructured,
      ...((syntax === undefined) ? {} : { syntax, }),
    },),
    evidence: [
      ...translateSlateEvidence({
        sourceText,
        ...((pictureContext === undefined) ? {} : { pictureContext, }),
        ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
        ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
        ...((identityContext === undefined) ? {} : { identityContext, }),
        ...((referenceContext === undefined) ? {} : { referenceContext, }),
        ...((archiveDisputeNote === undefined) ? {} : { archiveDisputeNote, }),
        lifeEnded: lifeEnded === true,
        ...((subjectNames === undefined) ? {} : { subjectNames, }),
        ballotTexts: rotated.map(function ballotText(candidate,): string {
          return candidate.rendered;
        },),
      },),
      ...translatedSlateBreakEvidence({
        sourceText,
        archiveText: incumbentText,
        slate,
        ...((syntax === undefined) ? {} : { syntax, }),
      },),
    ],
    signal,
    perCallTimeoutMs,
    l: tl,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
    stageSettings,
  },);
  if (outcome.kind === 'selected') {
    // UNREACHABLE while the slate is built the way it is, and here because the
    // day it becomes reachable is the day a slice ships a deletion. A proposal
    // or an incumbent that shows nothing never becomes a candidate
    // (`buildTranslateCandidates` asks `rendersAsNothing` of the folded bytes),
    // so a winner always shows something; this states that dependency rather
    // than relying on a reader of `candidate-select.ts` noticing it. An empty
    // candidate reached the slate while that filter asked `trim()` of the raw
    // reply (ledger B40).
    if (blankAgainst({
      winner: outcome.value
        .text,
      sourceText,
    },)) {
      // ITS OWN FAULT, not an absence. A slice whose archive wording exists is
      // not unfilled just because selection returned a deletion, and reporting
      // it as one would record a translated passage as one nobody ever
      // translated. This is a defect in the slate rather than an outcome, in
      // either mode.
      throw new BlankSelectionError({
        findings: [
          ...produced.findings,
          ...outcome.findings,
        ],
      },);
    }
    tl.info(
      `translate stage: ${describeProducer(outcome.producer,)} won weight ${
        String(outcome.voteWeight,)
      }`,
    );
    return {
      text: outcome.value
        .text,
      origin: outcome.value
        .origin,
      producer: outcome.producer,
      decision: 'judged',
      voteWeight: outcome.voteWeight,
      tally: outcome.tally,
      ballots: outcome.ballots,
      heardTranslators: produced.heardTranslators,
      candidateCount: produced.candidates
        .length,
      findings: [
        ...produced.findings,
        ...outcome.findings,
      ],
      slate,
      selectedIndex: outcome.selectedIndex,
      shippedIndex: outcome.selectedIndex,
      perCandidate: outcome.perCandidate,
    };
  }

  // Both declines keep the text that is already there, which is the difference
  // between this stage and the editor's. There, a decline still shipped the
  // strongest repair, because the panel had already ruled the defects real and
  // dropping them would have lost recall. Here the candidates are whole
  // translations of a passage nobody filed a complaint about, so replacing the
  // archive's own wording needs judges who agreed, not judges who could not.
  /**
   Findings the decline reports, whichever way this exit goes.
   */
  const declineFindings = [
    ...produced.findings,
    ...outcome.findings,
    `translate-declined (${outcome.disposition})`,
  ];

  /**
   Which decline this was, in the vocabulary both exits use.
   */
  const declined = (outcome.disposition === 'indecision')
    ? 'declined-indecision'
    : 'declined-rejection';
  // A DECLINE IS NOT A KEEP when there is nothing to keep. Judges who could not
  // agree have said nothing about the archive's wording, which is why keeping it
  // is right; where the archive has no wording, the same silence would ship the
  // empty string as though the judges had chosen it.
  /**
   Candidates a tie backed when some of the slate drew nothing, so the
   challenge round can be a run-off over them (class fifty-three), on either
   door (class one hundred six).
   */
  const narrowed = runoffFinalists({
    candidates: rotated,
    perCandidate: outcome.perCandidate,
    disposition: outcome.disposition,
  },);
  if (incumbentKind === 'absent') {
    // A challenge round tied across every valid candidate ships one by
    // preference (class one hundred seventy-four); any other decline raises.
    return settleAbsentDecline({
      challenged: responsibility === 'decline-challenge',
      outcome,
      rotated,
      keepIncumbent,
      declineFindings,
      declined,
      // Conditional spread keeps the field absent where the whole slate stands.
      ...((narrowed.kind === 'narrowed') ? {
        finalists: narrowed.finalists,
        basis: narrowed.basis,
      } : {}),
      shipPastDecline,
      l: tl,
    },);
  }
  tl.info(`translate stage: ${outcome.reason}; keeping the incumbent`,);
  return {
    ...keepIncumbent,
    tally: outcome.tally,
    ballots: outcome.ballots,
    perCandidate: outcome.perCandidate,
    decision: declined,
    findings: declineFindings,
    // Conditional spread keeps the field absent where the whole slate stands.
    ...((narrowed.kind === 'narrowed') ? {
      runoffFinalists: narrowed.finalists,
      runoffBasis: narrowed.basis,
    } : {}),
  };
}

//endregion Translate judge
