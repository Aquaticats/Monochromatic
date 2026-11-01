import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type {
  ModelClient,
  ReaskClaims,
  ReaskMode,
  ReaskSeat,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type {
  RosterModelId,
  SeatRole,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type DisputedWording,
  disputedWordingFindings,
} from './disputed-wording.ts';
import {
  CONTRIBUTOR_AUTHORITY_FINDING,
  type DeclaredNamePair,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  type HeardVoice,
  seatOfVoice,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { askAboutFindings, } from './translate-repair-ask.ts';
import { requireComparedVerdict, } from './translate-floor-ground.ts';
import { validateTranslatedSlice, } from './translate-validate.ts';
import type { TranslateReportWire, } from './translate-wire.ts';
import { foldTranslation, } from './translator-answer-fold.ts';
import { sameWording, } from './wording-key.ts';

//region Translate repair
// Structural validation, and the conversation a failing candidate gets instead
// of being dropped.
//
// User decision, 2026-08-15. The alternatives were dropping an invalid
// candidate, showing judges everything, and dropping only on reference damage;
// all three were rejected in favour of asking the model that wrote it. What
// comes back is one of three answers, and the interesting one is the third: a
// model can say the finding is a fact about the passage rather than about its
// work, which no filter could ever have collected.
//
// The INCUMBENT never passes through here. It is the fallback and the thing
// being defended, so a validator that could drop it would be a validator that
// could delete the archive.

/**
 One translator's final text after validation, with what happened to it.

 @example
 ```ts
 const outcome: RepairOutcome = { voice, findings: [], };
 ```
 */
export type RepairOutcome = {
  /**
   Voice to build candidate from,
   absent when non-defensible contributor violation was not repaired.
   */
  readonly voice?: HeardVoice<TranslateReportWire>;

  /**
   Candidates of other models that the follow-up turn brought: a revision the author's reasoning runaway sent to another
   translator, recorded under that model's own id beside the author's candidate (ledger B1100).
   */
  readonly extraVoices?: readonly HeardVoice<TranslateReportWire>[];

  /**
   What validation and the follow-up turn recorded, in scorecard-stable
   wording.
   */
  readonly findings: readonly string[];
};

/**
 What becomes of a refused candidate whose author revised nothing usable, in
 the words the run log uses.

 @param contributorViolation - whether it respelled a contributor name, which
 no defence keeps

 @returns Clause finishing a log line about that candidate

 @example
 ```ts
 l.warn(`${modelId} gave no usable answer, so ${unrevisedFate({ contributorViolation, },)}`,);
 ```
 */
function unrevisedFate({ contributorViolation, }: { readonly contributorViolation: boolean; },): string {
  return contributorViolation
    ? 'its candidate is dropped, since a contributor respelling is not defensible'
    : 'its candidate goes on as written, for the floor to judge';
}

/**
 Validates one candidate and, when it fails, asks its author about it.

 @param client - injected model client

 @param voice - this translator's reply

 @param sourceText - original slice the candidate renders

 @param incumbentText - translation as it stands, which a matching candidate
 collapses into

 @param pageText - text this candidate would replace, whose block shape it has
 to carry. The same as `incumbentText` for a translator, and the ARCHIVE
 rather than the winning lane for a consolidator

 @param priorMessages - exact messages that produced the candidate

 @param reaskSeat - role, bench and shared record of the authors of the slate, which the re-ask of a reasoning runaway
 reads (ledger B1079, B1080)

 @param signal - caller abort honored by the follow-up exchange

 @param perCallTimeoutMs - deadline for it

 @param lineStructured - whether the line-structure rule governs this slice,
 which makes merging its lines a fault the author is sent back to fix

 @param declared - name pairs the front matter declares, which the
 publication rule reads for a linked title naming a declared person (class
 one hundred fourteen)

 @param disputedWordings - wordings a disputed slice refuses, which a
 candidate copying one is sent back over rather than collapsed into (owner,
 2026-09-27, "No eligible standing")

 @param l - stage logger

 @returns Final voice for this model plus what was recorded

 @throws {@link import('./translate-floor-ground.ts').FloorGroundDisagreementError}
 when the floor says it compared nothing, which both callers rule out before
 any writer is asked (ledger B43, B45)

 @throws Whatever the follow-up exchange raised, once the caller aborted,
 having written it on this seat's line and marked it written
 (`markFailureWritten`) unless it is the caller's reason itself

 @example
 ```ts
 const outcome = await repairOneCandidate({ client, voice, sourceText, ... },);
 ```
 */
async function repairOneCandidate(
  {
    client,
    voice,
    sourceText,
    incumbentText,
    pageText,
    syntax,
    lineStructured,
    declared = [],
    disputedWordings = [],
    priorMessages,
    reaskSeat,
    signal,
    perCallTimeoutMs,
    l,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly voice: HeardVoice<TranslateReportWire>;
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly pageText: string;
    readonly syntax?: SliceSyntax;
    readonly lineStructured: boolean;
    readonly declared?: readonly DeclaredNamePair[];
    readonly disputedWordings?: readonly DisputedWording[];
    readonly priorMessages: readonly ChatMessage[];
    readonly reaskSeat: ReaskSeat;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  }>,
): Promise<RepairOutcome> {
  /**
   Logger tagged with this candidate's turn.
   */
  const rl = tagged({
    tag: repairOneCandidate.name,
    l,
  },);
  // A candidate that reproduced the incumbent is about to COLLAPSE into it, and
  // the incumbent is never validated, so validating this copy would spend a
  // follow-up call to repair text that is not going to be on the ballot. Worse,
  // a revision would break the match and destroy the `translate-matched-incumbent`
  // signal, on exactly the slices where the incumbent diverges from its source
  // and the match is most worth knowing about.
  //
  // NOT ON A DISPUTED SLICE, whose incumbent is refused rather than kept: a
  // copy of it is sent back like any other refused text (owner, 2026-09-27).
  /**
   What the rule says of this candidate as a disputed wording, none where it
   is not one.
   */
  const disputedCopy = disputedWordingFindings({
    candidateText: voice.value
      .translation,
    disputedWordings,
  },);
  if (
    sameWording({
      proposal: voice.value
        .translation,
      standing: incumbentText,
      lineStructured,
    },)
      && (disputedCopy.length === 0)
  )
    return {
      voice,
      findings: [],
    };

  /**
   Structural verdict over what this model returned: a pass or a refusal.

   NEVER THAT THE FLOOR COMPARED NOTHING. Both stages that ask for this turn
   settle a slice the floor can compare nothing on before any writer is
   asked (ledger B43 for the translate stage, B45 for the consolidation), so
   no candidate reaches here on such a slice; this turn let one stand
   unvalidated there, and an admitted insertion wrote it into the page.
   */
  const validation = requireComparedVerdict({
    verdict: validateTranslatedSlice({
      sourceText,
      candidateText: voice.value
        .translation,
      pageText,
      ...((syntax === undefined) ? {} : { syntax, }),
      lineStructured,
      declared,
      disputedWordings,
    },),
  },);
  // A PASS ON A DOWNGRADED PAGE IS RECORDED, never silent. The strict grammar
  // refuses a span cut through an element, and the page side falls back to
  // plain markdown so the floor still has blocks to compare. That reading is
  // looser than the one a candidate is held to, so a pass under it is weaker
  // evidence and says so here.
  if ((validation.kind === 'valid') && (validation.pageGrammar === 'relaxed')) {
    rl.info(`${voice.modelId}: candidate passed against a page read as plain markdown, since strict MDX refused the page`,);
    return {
      voice,
      findings: ['translate-page-downgraded (page read as plain markdown; strict MDX refused it)',],
    };
  }
  if (validation.kind === 'valid')
    return {
      voice,
      findings: [],
    };

  /**
   What validation found, recorded whatever the author answers.
   */
  const found = validation.findings
    .map(function toFinding(finding,): string {
      return `translate-invalid (${voice.modelId}): ${finding}`;
    },);
  /**
   Whether original candidate violates non-defensible target authority floor.
   */
  const contributorViolation = validation.findings
    .includes(CONTRIBUTOR_AUTHORITY_FINDING,);
  /**
   What becomes of the original candidate when no revision is taken, as the
   log says it.
   */
  const fate = unrevisedFate({ contributorViolation, },);
  rl.warn(
    `${voice.modelId}: candidate fails the publication rule, so its author is asked about it: ${
      validation.findings
        .join(' ',)
    }`,
  );
  /**
   The author's answer to its own findings, or the answer of the model its runaway was re-asked of.
   */
  const answer = await askAboutFindings({
    client,
    voice,
    findings: validation.findings,
    priorMessages,
    sourceText,
    reaskSeat,
    signal,
    perCallTimeoutMs,
    l: rl,
  },);
  /**
   Model that gave the answer where it was not the author, the re-ask of the author's reasoning runaway having gone to
   another translator; for an answer nothing could read, the model the re-ask went to.
   */
  const stood = answer.heard
    ? answer.answeredBy
    : ('askedInstead' in answer) ? answer.askedInstead : undefined;

  /**
   The author, and the model that answered for it where another did, as a finding names them (ledger B1146).
   */
  const named = (stood === undefined) ? voice.modelId : `${voice.modelId} -> ${stood}`;
  if (!answer.heard) {
    rl.warn(`${named} gave no usable answer to its findings, so ${fate}`,);
    return {
      ...((contributorViolation) ? {} : { voice, }),
      findings: [
        ...found,
        `translate-repair-unheard (${named})`,
      ],
    };
  }

  /**
   What the author decided, and why.
   */
  const {
    resolution,
    translation,
    translatorNote: explanation,
  } = answer.value;
  if (resolution !== 'revised') {
    rl.warn(`${named} answered ${resolution} to its findings (${explanation}), so ${fate}`,);
    return {
      ...((contributorViolation) ? {} : { voice, }),
      findings: [
        ...found,
        `translate-repair-${resolution} (${named}): ${explanation}`,
      ],
    };
  }

  /**
   The revision as it would ship, folded before the recheck reads it, as the
   lane folded the answer it revises (ledger B112).
   */
  const revision = foldTranslation({
    modelId: voice.modelId,
    translation,
  },);

  /**
   Whether the revision actually resolved what was found, a pass or a
   refusal on the ground the first verdict was read on.
   */
  const rechecked = requireComparedVerdict({
    verdict: validateTranslatedSlice({
      sourceText,
      candidateText: revision.translation,
      pageText,
      ...((syntax === undefined) ? {} : { syntax, }),
      lineStructured,
      declared,
      disputedWordings,
    },),
  },);

  // A revision that still fails is NOT taken. The model was asked to fix these
  // findings and did not, so nothing says the new text is better, while the
  // original is at least what it produced with the whole sheet in front of it.
  if (rechecked.kind === 'invalid') {
    rl.warn(`${named}: revision still fails the publication rule and is not taken, so ${fate}`,);
    return {
      ...((contributorViolation) ? {} : { voice, }),
      findings: [
        ...found,
        ...revision.findings,
        `translate-repair-unresolved (${named}): ${explanation}`,
      ],
    };
  }
  /**
   Model that gave the revision where it was not the author, as the answer carries it.
   */
  const { answeredBy, } = answer;
  if (answeredBy !== undefined) {
    // THE REVISION IS THE OTHER MODEL'S OWN CANDIDATE (ledger B1100, the owner's answer "Other model, own name"): it goes
    // on the slate under that model's id, where its ballot for it weighs half as for any text its author judges, and the
    // author's candidate stays as it was written (dropped only where it respells a contributor, which no defence keeps).
    rl.info(`translate-repair: ${answeredBy} revised the candidate of ${voice.modelId} as a candidate of its own`,);
    return {
      ...((contributorViolation) ? {} : { voice, }),
      extraVoices: [
        {
          modelId: answeredBy,
          value: { translation: revision.translation, },
        },
      ],
      findings: [
        ...found,
        ...revision.findings,
        `translate-repair-revised-by-another (${voice.modelId} -> ${answeredBy})`,
      ],
    };
  }
  rl.info(`translate-repair: ${voice.modelId} revised its candidate`,);
  return {
    // THE VOICE KEEPS ITS SEAT (ledger B1161): a voice that answered for a runaway seat is ordered by that seat after the
    // revision as before it.
    voice: {
      ...voice,
      value: { translation: revision.translation, },
    },
    findings: [
      ...found,
      ...revision.findings,
      `translate-repair-revised (${voice.modelId})`,
    ],
  };
}

/**
 Validates every fresh candidate and gives each failing one back to its
 author.

 Candidates are handled CONCURRENTLY, one follow-up call per failing model at
 most, so a slice where every translator diverged costs one extra round rather
 than one extra round each.

 @param client - injected model client

 @param voices - heard translator replies

 @param benchModelIds - every seat of the gather that produced the replies, a seat that ran away and left no voice
 included, so a runaway author is never re-asked of a seat that ran away on this question; required, so a caller cannot
 leave it out (ledger B1202)

 @param sourceText - original every candidate renders

 @param incumbentText - translation as it stands, so a candidate reproducing
 it is left alone

 @param pageText - text these candidates would replace, whose blocks they have
 to carry, defaulting to the incumbent because that is what a translator
 replaces. A consolidator replaces the ARCHIVE while its incumbent is the lane
 that won, and the two are different texts

 @param priorMessages - exact messages every translator was given

 @param role - role of the seats that wrote the candidates, which the re-ask of an author's reasoning runaway reads to
 ask a model the role may take (ledger B1079, B1100): the translator role for the translate lane, the writer role for
 the consolidation

 @param reask - whose answer the re-ask of an author's reasoning runaway may give, from the launch's `StageSettings`;
 required, with no default, so that a measurement cannot be asked in the pass's mode (ledger B1141)

 @param reaskClaims - record of the models asked for the seats of this question, the same one the gather that produced
 the candidates used, so a model that took a runaway seat there cannot give a second voice here (ledger B1100)

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param lineStructured - whether the line-structure rule governs this slice,
 which makes merging its lines a fault the author is sent back to fix

 @param declared - name pairs the front matter declares, which the
 publication rule reads for a linked title naming a declared person (class
 one hundred fourteen)

 @param disputedWordings - wordings a disputed slice refuses, which a
 candidate copying one is sent back over rather than collapsed into (owner,
 2026-09-27, "No eligible standing")

 @param l - stage logger

 @returns Final voices in the order given, plus every finding

 @throws {@link import('./translate-floor-ground.ts').FloorGroundDisagreementError}
 when the floor says it compared nothing, which both callers rule out before
 any writer is asked (ledger B43, B45)

 @throws Whatever the first failing candidate's turn raised, in input order,
 once the caller aborted. Each follow-up exchange that fails of its own as
 the caller stops writes that failure on its seat's line as it ends, which
 for a later-listed seat still in flight can be after this rejects

 @example
 ```ts
 const { voices, findings, } = await repairInvalidCandidates({ ... },);
 ```
 */
export async function repairInvalidCandidates(
  {
    client,
    voices,
    benchModelIds,
    sourceText,
    incumbentText,
    pageText = incumbentText,
    syntax,
    lineStructured = false,
    declared = [],
    disputedWordings = [],
    priorMessages,
    role,
    reask,
    reaskClaims,
    signal,
    perCallTimeoutMs,
    l,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly voices: readonly HeardVoice<TranslateReportWire>[];
    readonly benchModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly pageText?: string;
    readonly syntax?: SliceSyntax;
    readonly lineStructured?: boolean;
    readonly declared?: readonly DeclaredNamePair[];
    readonly disputedWordings?: readonly DisputedWording[];
    readonly priorMessages: readonly ChatMessage[];
    readonly role: SeatRole;
    readonly reask: ReaskMode;
    readonly reaskClaims: ReaskClaims;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  }>,
): Promise<{
  readonly voices: readonly HeardVoice<TranslateReportWire>[];
  readonly findings: readonly string[];
}> {
  /**
   The models seated on this question or heard on it: every seat of the gather bench, a seat that ran away and left no
   voice included (ledger B1202), then the model of every voice and, where another model answered for a runaway seat,
   that seat too (ledger B1161), each once.
   */
  const authors = [
    ...new Set([
      ...benchModelIds,
      ...voices.flatMap(function modelsOf(voice,): readonly RosterModelId[] {
        return [
          voice.modelId,
          seatOfVoice({ voice, },),
        ];
      },),
    ],),
  ];

  /**
   One outcome per heard voice.
   */
  const outcomes = await allInInputOrder({
    members: voices.map(async function repairEach(voice,): Promise<RepairOutcome> {
      return await repairOneCandidate({
        client,
        voice,
        sourceText,
        incumbentText,
        pageText,
        ...((syntax === undefined) ? {} : { syntax, }),
        lineStructured,
        declared,
        disputedWordings,
        priorMessages,
        // AN AUTHOR IS ASKED ABOUT ITS FINDINGS IN THE ROLE THE CALLER GIVES, with the bench of the question (every seat
        // of the gather bench, which includes a seat that ran away and left no voice, and the model of every voice with
        // the seat it answered for) and the record the gather that produced them used, so a runaway author is re-asked of
        // a model its role may take that wrote none of the candidates, took no seat of the question and did not itself
        // run away on it (ledger B1079, B1100, B1161, B1202). Without a seat the re-ask asks no other model at all.
        reaskSeat: {
          role,
          bench: authors,
          claims: reaskClaims,
          // THE LAUNCH'S MODE (ledger B1141): a measurement is re-asked of the author it scores.
          reask,
        },
        signal,
        perCallTimeoutMs,
        l,
      },);
    },),
  },);

  return {
    voices: outcomes.flatMap(function toVoice(
      outcome,
    ): readonly HeardVoice<TranslateReportWire>[] {
      return [
        ...((outcome.voice === undefined) ? [] : [outcome.voice,]),
        ...(outcome.extraVoices ?? []),
      ];
    },),
    findings: outcomes.flatMap(function toFindings(outcome,): readonly string[] {
      return outcome.findings;
    },),
  };
}

//endregion Translate repair
