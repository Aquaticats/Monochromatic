import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  GradedItem,
  GradeVerdict,
} from './grade-sheet-read.ts';
import { parseRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Grade agreement
// Scoring a blind pre-grade against the human's grades, and scoring precision
// off the human's grades alone.
//
// Both denominators exclude items the human left unscored, and that exclusion is
// the whole reason `unscored` is a verdict rather than a missing value. Round
// two's sheet carries answers like "Not enough context to grade" and "Not sure
// which tense is best here"; counting either as a false positive would move a
// declined question into the precision denominator and report the pipeline as
// worse than measured, while counting it as a real defect would do the reverse.
//
// The agreement rate is what decides whether an agent may pre-filter a later
// round. It is deliberately computed from a round the human graded WITHOUT
// seeing the pre-grades, because an agreement rate measured against grades the
// agent influenced would certify the agent on its own suggestion.

/**
 How a pre-grade compared against the human's grades.

 @example
 ```ts
 const tally: AgreementTally = {
   compared: 48, agreed: 44, disagreed: [3, 17,], unscored: [10, 12,],
 };
 ```
 */
export type AgreementTally = {
  /**
   Items the human scored, which is the agreement denominator.
   */
  readonly compared: number;

  /**
   Items where the pre-grade matched.
   */
  readonly agreed: number;

  /**
   Sheet positions where the two disagreed, in order. Named rather than
   counted, because every disagreement is a calibration case worth reading.
   */
  readonly disagreed: readonly number[];

  /**
   Sheet positions the human declined to score, excluded from the denominator
   and reported so their number is never invisible.
   */
  readonly unscored: readonly number[];
};

/**
 Precision over the items a human actually scored.

 @example
 ```ts
 const precision: PrecisionTally = { scored: 48, realDefects: 39, unscored: [10,], };
 ```
 */
export type PrecisionTally = {
  /**
   Items carrying a verdict, the precision denominator.
   */
  readonly scored: number;

  /**
   Items graded a real defect, the numerator.
   */
  readonly realDefects: number;

  /**
   Sheet positions the human declined to score.
   */
  readonly unscored: readonly number[];

  /**
   Sheet positions the human marked as the same defect as an earlier item.

   Reported apart from `unscored` and excluded from every denominator, on the
   user's decision of 2026-08-12. A duplicate is a defect already counted at
   another position, so counting it again measures how often the pipeline
   repeats itself rather than how often it is right.
   */
  readonly duplicates: readonly number[];

  /**
   Items the rates are taken over: everything drawn, less the duplicates.
   */
  readonly gradeable: number;
};

/**
 Verdict meaning the grader declined to answer.
 */
const UNSCORED: GradeVerdict = 'unscored';

/**
 Verdict meaning the item repeats a defect graded at an earlier position.
 */
const DUPLICATE: GradeVerdict = 'duplicate';

/**
 Verdicts a recorded pre-grade may carry, which are exactly the verdicts a
 sheet reader produces.

 KEYED BY `GradeVerdict` rather than listed, so the compiler refuses a verdict
 the sheet reader gains and this lacks. The list it replaced was written six
 days before `duplicate` joined the sheet's verdicts and never gained it, so a
 blind pre-grade marking a repeated defect had its whole file refused, though
 round three's pre-grader met seven such items and said so in its notes.
 */
const KNOWN_VERDICTS: Readonly<Record<GradeVerdict, true>> = {
  'real-defect': true,
  'false-positive': true,
  unscored: true,
  duplicate: true,
};

/**
 Guards an untrusted verdict string from a recorded pre-grade file.

 A guard rather than a membership test plus an assertion: `Set.has` on a set
 of strings proves nothing to the type system, so the assertion it forced was
 the only thing tying the runtime check to the type, and nothing would have
 caught the two drifting apart.

 @param value - candidate from parsed JSON

 @returns Whether value names one known verdict

 @example
 ```ts
 isGradeVerdict('real-defect',);
 ```
 */
function isGradeVerdict(value: unknown,): value is GradeVerdict {
  if ((typeof value) !== 'string')
    return false;

  return Object.hasOwn(
    KNOWN_VERDICTS,
    value,
  );
}

/**
 Parses recorded blind pre-grades.

 Strict for the same reason `artifact-read.ts` is strict: this is a
 measurement instrument, and a pre-grade quietly dropped for being malformed
 would shift the agreement denominator without leaving a trace. Every failure
 names the position it happened at.

 @param text - pre-grade file contents, as JSON

 @returns Pre-graded items in file order

 @throws {@link Error} when the file is not an array of usable pre-grades

 @example
 ```ts
 const agent = parsePreGrades({ text: await readFile(path, 'utf8',), },);
 ```
 */
export function parsePreGrades(
  { text, }: { readonly text: string; },
): readonly GradedItem[] {
  /**
   Raw parsed file, untyped until checked.
   */
  const raw: unknown = parseRunJson({
    text,
    from: 'pre-grades file',
  },);
  if (!Array.isArray(raw,))
    throw new StatedRefusalError({ says: 'pre-grades file must hold an array of graded items', },);

  return raw.map(function toItem(
    value: unknown,
    position: number,
  ): GradedItem {
    if (!isJsonRecord(value,))
      throw new StatedRefusalError({ says: `pre-grade ${String(position,)} is not an object`, },);

    /**
     Fields one recorded pre-grade carries.
     */
    const {
      index,
      verdict,
      note,
    } = value;
    if ((typeof index) !== 'number')
      throw new StatedRefusalError({ says: `pre-grade ${String(position,)} has no numeric index`, },);
    if (!isGradeVerdict(verdict,))
      // THE VERDICT IS NOT QUOTED: the file is agent-written, and the rule that
      // keeps text out of a refusal goes by class, not by trust.
      throw new StatedRefusalError({
        says: `pre-grade ${String(index,)} carries a verdict outside the vocabulary`,
      },);
    return {
      index,
      verdict,
      note: (typeof note) === 'string' ? note : '',
    };
  },);
}

/**
 Sheet positions the human declined to score.

 @param human - human's graded items

 @returns Positions carrying no verdict, in sheet order

 @example
 ```ts
 const declined = unscoredPositions({ human, },);
 ```
 */
function unscoredPositions(
  { human, }: { readonly human: readonly GradedItem[]; },
): readonly number[] {
  return human.filter(function declined(item,) {
    return item.verdict === UNSCORED;
  },)
    .map(function toIndex(item,) {
      return item.index;
    },);
}

/**
 Scores a blind pre-grade against the human's grades.

 A PRE-GRADE THAT DECLINED OR MARKED A DUPLICATE where the human scored the
 item counts as a disagreement, named in `disagreed` like any other: it did
 not give the item's answer, and a rate that decides whether the pre-grader
 may filter a later round must not shrink its denominator by the items the
 pre-grader passed on. Where the human declined or marked the duplicate, the
 item is out of the denominator whatever the pre-grade said.

 @param agent - pre-grades, keyed by sheet position

 @param human - human's graded items off the same sheet

 @returns Agreement over the items the human scored

 @throws {@link StatedRefusalError} when the pre-grades name a sheet position
 more than once, or the two sets cover different sheet positions, which would
 silently compare one round's grades against another's

 @example
 ```ts
 const tally = scoreGradeAgreement({ agent, human, },);
 ```
 */
export function scoreGradeAgreement(
  {
    agent,
    human,
  }: {
    readonly agent: readonly GradedItem[];
    readonly human: readonly GradedItem[];
  },
): AgreementTally {
  /**
   Pre-grades by sheet position; a map until read, as every record filled by
   a key is (ledger B77).
   */
  const byIndex = new Map<number, GradedItem['verdict']>();
  for (const item of agent) {
    // A POSITION NAMED TWICE IS REFUSED where the map is built: the later
    // verdict would replace the first, and a file carrying one row too many
    // for it passes the length comparison with the right size.
    if (byIndex.has(item.index,)) {
      throw new StatedRefusalError({
        says: `pre-grades name sheet position ${
          String(item.index,)
        } more than once, so which verdict belongs to it cannot be read`,
      },);
    }
    byIndex.set(
      item.index,
      item.verdict,
    );
  }
  if (agent.length !== human.length)
    throw new StatedRefusalError({
      says: `pre-grades cover ${String(agent.length,)} ${
        wordForCount({
          count: agent.length,
          one: 'item',
          many: 'items',
        },)
      } but the graded sheet has `
        + `${String(human.length,)}; they are not the same draw`,
    },);

  // COUNT ALONE IS NOT ENOUGH: a pre-grade file indexed from zero, or shifted
  // by one, has the right size and misses every sheet position, and every
  // `byIndex` lookup would then read as a disagreement rather than as a file fault.
  for (const item of human) {
    if (!byIndex.has(item.index,))
      throw new StatedRefusalError({
        says: `pre-grades carry no entry for sheet position ${String(item.index,)}; they do not index the same draw`,
      },);
  }

  /**
   Items the human scored, which is the only population an agreement rate
   means anything over.

   Duplicates are excluded for the same reason declines are, and it is not a
   technicality: on a duplicate the human answered a question about the SHEET,
   that this defect was already graded elsewhere, while the pre-grade answered
   the question the item asks. Counting those as disagreements charged the
   agent seven wrong answers for reaching the same conclusion by another
   route, since its notes named the same seven as repeats.
   */
  const scored = human.filter(function hasVerdict(item,) {
    return (item.verdict !== UNSCORED) && (item.verdict !== DUPLICATE);
  },);

  /**
   Positions where the pre-grade differed.
   */
  const disagreed = scored.filter(function differs(item,) {
    return byIndex.get(item.index,) !== item.verdict;
  },)
    .map(function toIndex(item,) {
      return item.index;
    },);

  return {
    compared: scored.length,
    agreed: scored.length - disagreed.length,
    disagreed,
    unscored: unscoredPositions({ human, },),
  };
}

/**
 Scores accepted-issue precision off a graded sheet.

 @param human - human's graded items

 @returns Precision counts over the items carrying a verdict

 @example
 ```ts
 const tally = scoreGradedPrecision({ human, },);
 ```
 */
export function scoreGradedPrecision(
  { human, }: { readonly human: readonly GradedItem[]; },
): PrecisionTally {
  /**
   Items the rates are taken over, duplicates removed.
   */
  const gradeable = human.filter(function notDuplicate(item,) {
    return item.verdict !== DUPLICATE;
  },);

  return {
    scored: gradeable.filter(function hasVerdict(item,) {
      return item.verdict !== UNSCORED;
    },)
      .length,
    realDefects: gradeable.filter(function isReal(item,) {
      return item.verdict === 'real-defect';
    },)
      .length,
    unscored: unscoredPositions({ human, },),
    duplicates: human.filter(function isDuplicate(item,) {
      return item.verdict === DUPLICATE;
    },)
      .map(function toIndex(item,) {
        return item.index;
      },),
    gradeable: gradeable.length,
  };
}

//endregion Grade agreement
