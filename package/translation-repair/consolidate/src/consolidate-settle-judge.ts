import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { INELIGIBLE_STANDING_WITHHELD_FINDING, } from './consolidate-ineligible-standing.ts';
import type { ConsolidationSubject, } from './consolidate-settle.ts';
import type { SettlementIdentity, } from './consolidate-settle-context.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type IncumbentKind,
  TranslateAbsenceError,
  type TranslateCandidateSet,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  judgeSlateWithRetry,
  judgeTranslateSlate,
  runoffTiedFinalists,
  type TranslateStageResult,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Consolidate settle judging
// THE SLATE JUDGING OF `consolidate-settle.ts`, split out at that file's line
// budget when its absence stopped throwing and started returning (owner,
// 2026-09-27, "Keep archive, ship"). The request is built here once, so the
// challenged and the single-round asks cannot drift apart.

/**
 Raised when the slate judging raises an absence over a withheld standing,
 whose slate the judging was told ships a candidate two judges named or keeps
 the archive wording.

 A FAULT IN THIS CODE rather than a fact about any text (ledger B51, as
 ledger B43 set out for the floor): a withheld slate reaches the judges only
 with a candidate on it, since the settlement keeps the archive first where
 no proposal survived the floor and no lane text is offered, and the lane
 offer drops blank texts; and on a slate with a candidate the judging returns
 a decision past every decline, the candidate two or more judges named or the
 archive wording staying where none drew two ballots (owner, 2026-09-27,
 "Preference + polish", narrowed on 2026-10-25 by "Two judges must name it").
 An absence here means one of those rules changed without the other.

 @example
 ```ts
 throw new WithheldSlateAbsenceError({ reason: error.reason, cause: error, },);
 ```
 */
export class WithheldSlateAbsenceError extends Error {
  /**
   Declares this message safe to forward: it is one fixed sentence; the
   absence's reason rides beside it as a field and never enters it.
   */
  readonly messageNamesOnly: true = true;

  /**
   Why the judging said the slate had nothing to ship.
   */
  public readonly reason: TranslateAbsenceError['reason'];

  /**
   Builds the failure around the absence the judging raised.

   @param reason - why the judging said the slate had nothing to ship

   @param cause - absence itself, kept for its findings

   @example
   ```ts
   throw new WithheldSlateAbsenceError({ reason: error.reason, cause: error, },);
   ```
   */
  public constructor(
    {
      reason,
      cause,
    }: {
      readonly reason: TranslateAbsenceError['reason'];
      readonly cause: TranslateAbsenceError;
    },
  ) {
    super(
      'the slate judging raised an absence over a withheld standing, whose slate ships a candidate two judges '
        + 'named or keeps the archive wording, so the settlement and the judging disagree about what a withheld '
        + 'slate can reach',
      { cause, },
    );
    this.name = 'WithheldSlateAbsenceError';
    this.reason = reason;
  }
}

/**
 The slate judging's decision, with an absence over a withheld standing
 named as the fault it is.

 ONE NARROWING AT THE ONE CALL SITE, so the answer that cannot come has one
 statement with its own case, as `requireComparedVerdict` has for the floor
 (ledger B43). An absence over an eligible standing, and anything else the
 judging raises, passes through unchanged.

 @param judged - judging in flight

 @param standingEligible - whether the standing passed the deterministic gate

 @returns Judges' decision

 @throws {@link WithheldSlateAbsenceError} when the judging raises an absence
 over a withheld standing

 @throws Whatever else the judging raises, unchanged

 @example
 ```ts
 const decided = await requireWithheldSlateDecided({ judged: judgeTranslateSlate(judging,), standingEligible, },);
 ```
 */
export async function requireWithheldSlateDecided<const DecidedT,>(
  {
    judged,
    standingEligible,
  }: {
    readonly judged: Promise<DecidedT>;
    readonly standingEligible: boolean;
  },
): Promise<DecidedT> {
  try {
    return await judged;
  }
  catch (error) {
    if (standingEligible || (!(error instanceof TranslateAbsenceError)))
      throw error;
    throw new WithheldSlateAbsenceError({
      reason: error.reason,
      cause: error,
    },);
  }
}

/**
 Whether the standing a tied slate kept drew no ballot in that tie.

 A standing that drew a ballot was ranked against the proposals, so its tie is
 the single round's own answer and stands. A standing that drew none was not
 chosen by any judge, so the tied finalists are asked again instead
 (`judgeSingleRoundOverStanding`).

 A STANDING NOT ON THE SLATE HAS NO BALLOT TO READ, and answers false, so the
 tie is left as the single round left it. An eligible standing with wording is
 always on the slate, since `buildTranslateCandidates` offers the incumbent
 first and a dedup keeps the first text it sees. A standing that is on the
 slate has a row in `perCandidate` for its position, since
 `countCandidateWeights` gives every candidate one.

 @param result - single round's decline, which kept the incumbent

 @returns Whether the standing is on the slate and no ballot named it

 @throws Error when the round has no `perCandidate` row for the standing's position

 @example
 ```ts
 const undrawn = standingDrewNoBallot({ result, },);
 ```
 */
function standingDrewNoBallot(
  { result, }: { readonly result: TranslateStageResult; },
): boolean {
  /**
   The standing's entry on the slate the round judged, absent when it is not on it.
   */
  const standing = result.slate
    .find(function isStanding(entry,): boolean {
      return entry.origin === 'incumbent';
    },);
  if (standing === undefined)
    return false;
  /**
   The round's count for the standing's position.
   */
  const drawn = nonNullishOrThrow(
    result.perCandidate
      .find(function namesStanding(row,): boolean {
        return row.index === standing.index;
      },),
  );
  return drawn.ballots === 0;
}

/**
 Asks the slate judges once over an eligible standing, and asks the tied
 finalists once more when the tie left the standing without a ballot (owner
 answer 2026-10-11, "Run off the tied finalists"). A decision in the single
 round, or a tie that drew a ballot for the standing, is returned as the
 single round left it.

 @param judging - everything {@link judgeTranslateSlate} needs

 @returns Judges' decision, from whichever round decided it

 @throws Whatever the judging raises, unchanged

 @example
 ```ts
 const decided = await judgeSingleRoundOverStanding({ judging, },);
 ```
 */
async function judgeSingleRoundOverStanding(
  { judging, }: { readonly judging: Parameters<typeof judgeTranslateSlate>[0]; },
): Promise<TranslateStageResult> {
  /**
   What the single round decided.
   */
  const single = await judgeTranslateSlate(judging,);
  /**
   Finalists the tie backed, absent where the round did not tie with the incumbent kept.
   */
  const finalists = single.runoffFinalists;
  /**
   Why those are the finalists, set wherever the finalists are.
   */
  const basis = single.runoffBasis;
  if ((single.decision !== 'declined-indecision') || (finalists === undefined))
    return single;
  // The basis is set wherever the finalists are (`judgeTranslateSlate`), so a result with one and not the other is a fault.
  if (basis === undefined)
    throw new Error('unreachable: a round that narrowed its run-off carries the finalists without the basis they were chosen on',);
  if (!standingDrewNoBallot({ result: single, },))
    return single;
  return await runoffTiedFinalists({
    judging,
    first: single,
    finalists,
    basis,
  },);
}

/**
 Asks the slate judges over one consolidation slate.

 A WITHHELD SLATE IS ALWAYS DECIDED (ledger B51). It used to return the
 absence the judging raised over a withheld standing, which the settlement
 answered by keeping the archive (owner, 2026-09-27, "Keep archive, ship");
 since the judging returns a decision past every decline over a withheld
 standing (a candidate two judges named, else the archive wording staying,
 which the settlement answers by keeping the archive), and the settlement
 keeps the archive before judging a slate with nothing on it, no judging
 reaches that absence, and it is now the fault `WithheldSlateAbsenceError`
 names.

 A TIE WITH THE STANDING WITHHELD IS CHALLENGED ONCE (class fifty-five,
 XingZ605 slice 13, 2026-09-18): four valid proposals, the judges 2 to 2
 between two renderings, and the decline stopped the entry at 4h08m over
 a slate that had nothing wrong with it. The translate lane has re-asked a
 declined slate under `decline-challenge` since class fifty-three, with a
 run-off over the candidates the tie backed; the consolidation gets the
 same second round wherever the standing is withheld, and a challenge that
 declines with nothing left to narrow ships the candidate two or more judges
 named, and keeps the archive where none did (owner, 2026-09-27, "Preference +
 polish", narrowed on 2026-10-25 by "Two judges must name it"). An eligible standing keeps the single
 round, because there a decline keeps text the contest already endorsed,
 except where a strict majority of the contest ballots called the archive
 flawed (`runoffOverStanding`, class one hundred six, owner answer
 2026-10-11), where a tie is run off too. An eligible standing that the single
 round ties while drawing no ballot gets one run-off over the tied finalists
 (owner answer 2026-10-11, "Run off the tied finalists"), because a standing
 no judge chose is no endorsement to keep. This said the eligible standing
 always kept the single round until ledger H10.

 @param client - provider client the round borrows

 @param judgeModelIds - voices seated to judge

 @param subject - slice in the archive's terms

 @param built - distinct proposals the judges see

 @param survivors - proposals the floor passed, counted for the judges

 @param producedFindings - what gathering and repairing recorded

 @param standingEligible - whether the standing passed the deterministic gate

 @param incumbent - what the slate offers to keep, empty when withheld

 @param identity - front matter and reference context the judges read

 @param evidence - pictures and neighbouring passages the producers saw

 @param lineStructured - whether the verse rule governs this slice

 @param challenged - whether a tie is run off once

 @param signal - cancellation for the whole round

 @param perCallTimeoutMs - ceiling on each call

 @param l - stage logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Judges' decision

 @throws {@link WithheldSlateAbsenceError} when the judging raises an absence
 over a withheld standing, a fault in this code

 @throws Whatever else the judging raises, unchanged

 @example
 ```ts
 const decided = await judgeConsolidationSlate({ client, judgeModelIds, subject, built, survivors, producedFindings, standingEligible, incumbent, identity, evidence, lineStructured, challenged, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function judgeConsolidationSlate(
  {
    client,
    judgeModelIds,
    subject,
    built,
    survivors,
    producedFindings,
    standingEligible,
    incumbent,
    identity,
    evidence,
    lineStructured,
    challenged,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly judgeModelIds: readonly RosterModelId[];
    readonly subject: ConsolidationSubject;
    readonly built: TranslateCandidateSet;
    readonly survivors: number;
    readonly producedFindings: readonly string[];
    readonly standingEligible: boolean;
    readonly incumbent: Readonly<{
      incumbentText: string;
      incumbentKind: IncumbentKind;
    }>;
    readonly identity: SettlementIdentity;
    readonly evidence: Readonly<{
      pictureContext?: string;
      neighbouringSourceText?: string;
      neighbouringIncumbentText?: string;
    }>;
    readonly lineStructured: boolean;
    readonly challenged: boolean;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<TranslateStageResult> {
  /**
   Everything one slate judging takes, built once so the challenged and
   the single-round asks cannot drift apart.
   */
  const judging: Parameters<typeof judgeTranslateSlate>[0] = {
    client,
    produced: {
      candidates: built.candidates,

      // SURVIVORS RATHER THAN VOICES HEARD, because this number exists to
      // tell the judges how thin the slate they are deciding over is, and
      // a refused proposal is not on it. A census reading this for
      // transport health would misread a refusal as a lost voice;
      // `verdicts` is what separates them.
      heardTranslators: survivors,

      findings: [
        ...producedFindings,
        ...(standingEligible ? [] : [INELIGIBLE_STANDING_WITHHELD_FINDING,]),
        ...built.findings,
      ],
    },
    judgeModelIds,
    sourceText: subject.sourceText,
    incumbentText: incumbent.incumbentText,

    // PRESENT WHENEVER THE STANDING MAY SHIP, ABSENT WHEN THE GATE REFUSED
    // IT. What the judges fall back on at this stage is `standingText`,
    // not the archive's own wording, and the `standingText === ''` exit
    // in `settleConsolidation` returns before any judge is bought; so a
    // slate reaching this call has a text to keep unless that text is
    // ineligible, in which case there is nothing to keep. Threading the
    // slice's own `incumbentKind` here would say something different and
    // wrong: an anchor whose lanes both produced wording has a standing text
    // to fall back on even though the archive holds none.
    incumbentKind: incumbent.incumbentKind,
    ...((subject.syntax === undefined) ? {} : { syntax: subject.syntax, }),
    ...identity,
    // WHAT THE PRODUCERS WERE SHOWN, forwarded rather than recomputed.
    // One change put the pictures in front of the producers and left the
    // judges blind, which is worse than both being blind: a producer that
    // used a picture correctly then looked to its judge like one
    // inventing detail.
    ...evidence,
    // THE SAME FLAG THE PRODUCERS WERE GIVEN, which the settlement has held
    // since it was written and passed to nobody. It was then given to the
    // consolidation producers; leaving the judges out of it would have the
    // judges mark down exactly the unmerging the producers were told to do.
    lineStructured,
    // WORDING THAT CANNOT SHIP IS WITHHELD, NOT ABSENT (owner answer
    // 2026-09-27, "Preference + polish"): the judges are told a declined
    // slate still ships a candidate two judges named, or keeps the archive,
    // where a challenge round declined with nothing left to narrow stopped
    // the entry (owner, 2026-09-27 and 2026-10-25).
    withheldStanding: !standingEligible,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  };
  return await requireWithheldSlateDecided({
    judged: challenged
      ? judgeSlateWithRetry({ judging, },)
      : judgeSingleRoundOverStanding({ judging, },),
    standingEligible,
  },);
}


//endregion Consolidate settle judging
