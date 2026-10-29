import type { Candidate, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { TranslateCandidateValue, } from './translate-candidates.ts';

//region Translate absence
// What the translate lane does when there is no incumbent to fall back on.
//
// Every fallback in the stage ships the translation that is already there. That
// is the right answer for a slice the archive HAS translated: leaving a passage
// as it stands is the state the run began in, while shipping text no judge
// vetted is a new claim about the archive. For a slice the archive has NOT
// translated, the same fallback ships nothing at all, reports a settled slice,
// and the run reads as having delivered a translation it never produced.
//
// A MODE RATHER THAN AN INFERENCE. Absence is decided once, from the target
// chunk being an insertion anchor, and travels as a discriminant. Reading it
// off `incumbentText === ''` would conflate two different slices: an anchor,
// where a rendering belongs and none exists, and a content span that genuinely
// holds nothing but whitespace, where the archive's own wording is the blank.
//
// AN INTERNAL CORRECTION SIGNAL RATHER THAN A RESULT, because there is no honest
// result to build. Judging raises it with latest evidence; translate stage uses
// that evidence to produce next distinct task. It must not escape production as
// a terminal quality verdict or become publishable empty text.

/**
 Whether a slice has a translation to fall back on.

 @example
 ```ts
 const incumbentKind: IncumbentKind = isInsertionChunk(slice.target,) ? 'absent' : 'present';
 ```
 */
export type IncumbentKind =
  /**
   Archive holds a translation for this slice, which stands unless judges
   prefer something else.
   */
  | 'present'
  /**
   Archive holds none, so there is nothing to fall back on and the slice is
   either filled by this run or left as the gap it is.
   */
    | 'absent';

/**
 Why a slice with no incumbent could not be filled.

 @example
 ```ts
 const reason: TranslateAbsenceReason = 'no-candidate';
 ```
 */
export type TranslateAbsenceReason =
  /**
   Translators answered and none of their answers was usable, and there is no
   incumbent to stand in their place.

   SAYS SOMETHING ABOUT THE PASSAGE. Models that were heard and proposed
   nothing a guard would accept are evidence that this slice is hard, so the
   gap it leaves is one a re-run would probably meet again.
   */
  | 'no-candidate'
  /**
   No translator was heard at all: every voice on the slate was lost.

   SAYS SOMETHING ABOUT THE HOUR RATHER THAN THE PASSAGE, which is why it is
   a separate reason. `no-candidate` used to cover this case as well, so a
   transient failure and a genuinely hard slice left identical gaps in the
   published page and identical absences in the artifact, and nothing
   downstream could tell a reader or a resumed pass which it had met.

   Measured on 2026-08-24: voice loss ran at 1 percent for two hours and 20
   percent for the next, across two passes on different builds and different
   entries, spread evenly over all six models. In that hour XIEPT2's translate
   lane went from 13 computed and 0 unfilled to 9 computed and 10 unfilled.
   Every one of those ten would have been recorded as a property of the
   passage.
   */
    | 'no-voice-heard'
  /**
   Judges could not settle on one candidate.
   */
    | 'declined-indecision'
  /**
   Judges rejected every candidate they were shown.
   */
    | 'declined-rejection'
  /**
   Judges declined the same slate TWICE, so the panel was asked again and
   still backed nothing.

   Stronger than either single-round decline, and recorded instead of them once
   the retry is spent. Those two describe one round's mood, where this says the
   slate itself never won a voice from a panel that saw it twice.

   A REASON RATHER THAN A LANDING: the slice goes where it would have gone
   anyway, keeping an incumbent where there is one and leaving the gap where
   there is not.
   */
    | 'no-candidate-backed'
  /**
   The floor could compare nothing written for this slice, so the stage
   asked nobody (ledger B43).

   SAYS SOMETHING ABOUT THE ORIGINAL OR THE GRAMMAR, never about a model:
   an original no grammar reads, or a page none does. No candidate could
   pass there, and a rendering nobody could check is not one to write into
   the page, so the gap stays until the grammar reads the passage.
   */
    | 'unfloored';

/**
 Why a slice with no incumbent was left unfilled, including reasons no stage
 can produce.

 WIDER THAN {@link TranslateAbsenceReason} ON PURPOSE. Every reason there is
 the stage's own answer about the slice it was asked to render: a round that
 was paid for and came back empty, or the one question the stage settles
 before any round (`unfloored`), since only the stage holds what the floor
 reads. The driver can also decline to buy a round at all, from what the
 document around the slice shows, and giving the stage a word for that would
 let a reader think it might return one.
 */
export type UnfilledReason =
  | TranslateAbsenceReason
  /**
   The page has no room to be missing this passage, so nothing was bought.

   Decided before anything is spent, by
   `doc/decision/translation-repair-absence-verdict.md`: a pairing leaving an
   original unplaced is one signature and a page measurably shorter than its
   source predicts is the other, and both are required. A page of ordinary
   length more likely merged that passage than dropped it, and writing it in
   would put a second rendering of it into a memorial document.
   */
    | 'not-corroborated';

/**
 Raised when selection returned text that says nothing for a source that does.

 A DIFFERENT FAULT FROM ABSENCE, and separate because the two have opposite
 remedies. An unfilled passage is a slice the run could not translate, which
 costs that slice and leaves the archive's gap; a blank winner means selection
 chose a deletion, which is a defect in this code rather than an outcome, and
 a caller that treated it as an unfilled passage would record a slice the
 archive DOES translate as one it never did.

 Reached by no ordinary path while the slate builder and the reply guards ask
 `rendersAsNothing` (`renders-as-nothing.ts`): a proposal or an incumbent that
 shows nothing never becomes a candidate. That was claimed unreachable while
 those checks asked `trim()`, and was not: a reply of one zero-width space
 passed them, the intake fold emptied it, and an empty candidate reached the
 judges (ledger B40). It exists so a route that changes this is a loud
 failure rather than a deletion.

 @example
 ```ts
 throw new BlankSelectionError({ findings, },);
 ```
 */
export class BlankSelectionError extends Error {
  /**
   Declares this message safe to forward: it is one fixed sentence; the findings ride beside it as a field and never enter it.
   */
  readonly messageNamesOnly: true = true;

  /**
   What the stage had gathered before the winner came back blank.
   */
  public readonly findings: readonly string[];

  /**
   Builds the failure with the evidence the round produced.

   @param findings - stage findings up to this point

   @example
   ```ts
   throw new BlankSelectionError({ findings, },);
   ```
   */
  public constructor({ findings, }: { readonly findings: readonly string[]; },) {
    super(
      'selection chose text that says nothing for a source that says something, '
        + 'so shipping it would delete the passage',
    );
    this.name = 'BlankSelectionError';
    this.findings = findings;
  }
}

/**
 Why a run-off is over the candidates it is over, which the line announcing it
 has to say (ledger B806).

 -   `tied-at-top`: two or more candidates drew the same highest weight.
 -   `leader-and-runner-up`: one candidate drew the highest weight and still
     was named by fewer than the two ballots a winner needs, so the run-off is over it and the candidate
     behind it; nothing was tied.

 @example
 ```ts
 const basis: RunoffBasis = 'leader-and-runner-up';
 ```
 */
export type RunoffBasis =
  | 'tied-at-top'
    | 'leader-and-runner-up';

/**
 Raised internally when one absent-passage slate produced no translation.

 CARRIES ITS FINDINGS, because the work that led here is real evidence: which
 translators were heard, what collapsed, what the judges counted. Thrown away
 with the exception, that evidence would leave a run reporting an unfilled
 passage with nothing to say about why.

 @example
 ```ts
 throw new TranslateAbsenceError({ reason: 'no-candidate', findings, },);
 ```
 */
export class TranslateAbsenceError extends Error {
  /**
   Declares this message safe to forward: it names one of a closed set of absence reasons, which the type enforces.
   */
  readonly messageNamesOnly: true = true;

  /**
   Why the slice could not be filled.
   */
  public readonly reason: TranslateAbsenceReason;

  /**
   What the stage had gathered before it gave up.
   */
  public readonly findings: readonly string[];

  /**
   Candidates that drew a ballot in a tied round, when some of the slate drew
   none, so the challenge round can be a run-off over them (class
   fifty-three, `translate-runoff.ts`).
   */
  public readonly finalists?: readonly Candidate<TranslateCandidateValue>[];

  /**
   Why those candidates are the finalists: present exactly when `finalists` is
   (ledger B806).
   */
  public readonly basis?: RunoffBasis;

  /**
   Builds the refusal with the evidence the stage had collected.

   @param reason - why nothing could be written

   @param findings - stage findings up to this point, kept for the record

   @param finalists - candidates a tied round backed, when fewer than the slate

   @param basis - why those are the finalists, given with them

   @example
   ```ts
   throw new TranslateAbsenceError({ reason: 'declined-indecision', findings, },);
   ```
   */
  public constructor(
    {
      reason,
      findings,
      finalists,
      basis,
    }: {
      readonly reason: TranslateAbsenceReason;
      readonly findings: readonly string[];
      readonly finalists?: readonly Candidate<TranslateCandidateValue>[];
      readonly basis?: RunoffBasis;
    },
  ) {
    super(
      `slice has no translation in the archive and produced none (${reason}), `
        + 'so there is nothing to write and nothing to fall back on',
    );
    this.name = 'TranslateAbsenceError';
    this.reason = reason;
    this.findings = findings;
    if (finalists !== undefined)
      this.finalists = finalists;
    if (basis !== undefined)
      this.basis = basis;
  }
}

/**
 The absence a catch clause caught, or what it caught rethrown unchanged.

 ONE NARROWING FOR EVERY CATCH THAT ACTS ON AN ABSENCE ALONE. The judges'
 retry and the stage's depth-two rounds each caught a round's failure, acted
 on an absence and rethrew anything else from a statement of their own; the
 stage's rethrow was reachable only through a fault inside judging, since a
 provider's fault reaches judging as a lost voice (`stage-call.ts`), and no
 stage case could send one there. Here it is one statement with its own
 cases.

 @param error - value the catch clause bound

 @returns Absence itself

 @throws What was caught, by identity, when it is not an absence

 @example
 ```ts
 const rejection = requireTranslateAbsence({ error, },);
 ```
 */
export function requireTranslateAbsence({ error, }: { readonly error: unknown; },): TranslateAbsenceError {
  if (error instanceof TranslateAbsenceError)
    return error;
  throw error;
}

/**
 Reports whether a winning text says nothing about a source that says
 something.

 Asked of the SOURCE rather than of the winner alone, because a rendering of
 nothing is nothing: it is only a defect where there was a passage to render.

 @param winner - text selection chose

 @param sourceText - original that text is meant to render

 @returns Whether shipping it would delete a passage

 @example
 ```ts
 const empty = blankAgainst({ winner: outcome.value.text, sourceText, },);
 ```
 */
export function blankAgainst(
  {
    winner,
    sourceText,
  }: {
    readonly winner: string;
    readonly sourceText: string;
  },
): boolean {
  /**
   Whether the winner shows a reader anything at all (ledger B40: invisible
   characters `trim()` keeps are nothing too).
   */
  const saysNothing = rendersAsNothing({ text: winner, },);

  /**
   Whether there was something to render.
   */
  const sourceSaysSomething = !rendersAsNothing({ text: sourceText, },);

  return saysNothing && sourceSaysSomething;
}

/**
 Refuses to settle a passage the archive never translated on text that
 shows a reader nothing.

 STATED WHERE THE RECORD IS BUILT rather than trusted to the paths that
 produce it: every way of producing nothing throws before a record is
 built, and such a record is cached and read back as finished work, so a
 route that changes that fails here rather than caching a deletion. A
 statement in the settling function that no case reached was the census's
 only view of it (ledger T8, sixth batch); here it has its own cases.

 @param incumbentKind - whether the archive has wording for the slice

 @param text - text the slice would settle on

 @param findings - stage findings, carried by the refusal

 @throws {@link BlankSelectionError} when there is no archive wording and
 the text shows nothing (ledger B40: invisible characters `trim()` keeps
 are nothing too)

 @example
 ```ts
 assertAbsentSliceFilled({ incumbentKind, text: stageResult.text, findings: stageResult.findings, },);
 ```
 */
export function assertAbsentSliceFilled(
  {
    incumbentKind,
    text,
    findings,
  }: {
    readonly incumbentKind: IncumbentKind;
    readonly text: string;
    readonly findings: readonly string[];
  },
): void {
  if ((incumbentKind === 'absent') && rendersAsNothing({ text, },))
    throw new BlankSelectionError({ findings, },);
}

/**
 Names an unfilled slice the way every other stage finding is named.

 @param reason - why nothing could be written

 @returns Finding in scorecard-stable wording

 @example
 ```ts
 const finding = absenceFinding({ reason: 'no-candidate', },);
 ```
 */
export function absenceFinding(
  { reason, }: { readonly reason: UnfilledReason; },
): string {
  return `translate-unfilled (${reason})`;
}

//endregion Translate absence
