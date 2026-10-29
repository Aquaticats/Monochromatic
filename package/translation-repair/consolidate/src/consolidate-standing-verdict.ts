import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  describeStandingVerdict,
  INELIGIBLE_STANDING_REPLACED_FINDING,
} from './consolidate-ineligible-standing.ts';
import { contestStandingMayShip, } from './consolidate-standing.ts';
import type {
  ArtifactContestVerdict,
  LaneChoice,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  type DisputedWording,
  validateTranslatedSlice,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Consolidation standing verdict
// TWO VERDICTS ON ONE TEXT, kept apart on purpose. The deterministic gate says
// whether the standing may ever ship (`standingValid`); the contest says
// whether it has the endorsement to ship unchanged (`standingMayShip`). The
// owner's decision of 2026-09-04 (`translation-repair-ineligible-standing.md`)
// hangs on the first, the single-attempt rule on the second, and the run log
// has to name which one refused: on the luxuanwen3 pass of that day one
// warning covered both, and learning that a link destination the archive had
// rewritten was the cause took opening the slice records.
//
// THE INCUMBENT IS READ TOO, since the owner's addendum of 2026-09-09
// (`consolidate-ineligible-standing.ts`): where the standing fails the gate
// and the incumbent passes it, the incumbent becomes the wording the
// settlement runs against, with the replacement recorded and no endorsement.

/**
 Both verdicts on a slice's standing text, and the wording the settlement
 runs against once the incumbent has been read beside it.
 */
export type StandingVerdict = {
  /**
   Whether the wording in {@link StandingVerdict.settlementText} passes the
   deterministic publication rules.
   */
  readonly standingValid: boolean;

  /**
   Whether that wording has prior approval and may ship unchanged.
   */
  readonly standingMayShip: boolean;

  /**
   Wording the settlement runs against: the standing, or the incumbent
   where the standing failed the gate and the incumbent passes it.
   */
  readonly settlementText: string;

  /**
   Findings this reading adds to the settlement: the replacement, when it
   happened.
   */
  readonly findings: readonly string[];

  /**
   Whether the incumbent stood in, so the artifact can say a kept standing
   is text to write.
   */
  readonly incumbentStandsIn: boolean;

  /**
   Why the deterministic gate refused the standing, present only where the
   settlement runs against an ineligible text (class fifty-six, 2026-09-18).
   */
  readonly standingRefusal?: string;
};

/**
 Reads both verdicts on a standing text and logs a refusal by name.

 @param sourceText - original slice

 @param standingText - wording in place when consolidation begins

 @param incumbentText - page text this slice replaces

 @param syntax - explicit syntax role, absent for ordinary prose

 @param lineStructured - whether line-structure rule governs this slice

 @param choice - lane the contest chose

 @param contestVerdict - how the contest ended

 @param sliceIndex - prepared position of the slice, for the log line

 @param l - logger a refusal is written through

 @param declared - name pairs the front matter declares, which the rule
 reads for a linked title naming a declared person (class one hundred
 fourteen); none leaves that floor silent

 @param disputedWordings - wordings a disputed slice refuses as standing and
 as incumbent (owner, 2026-09-27, "No eligible standing"); none on an
 undisputed slice

 @returns Deterministic eligibility and contest endorsement of the wording
 the settlement runs against, that wording, and the replacement finding
 when the incumbent stands in

 @example
 ```ts
 const { standingValid, standingMayShip, settlementText, } = readStandingVerdict({
   sourceText, standingText, incumbentText, lineStructured: false, choice, contestVerdict, sliceIndex: 1, l,
 },);
 ```
 */
export function readStandingVerdict(
  {
    sourceText,
    standingText,
    incumbentText,
    syntax,
    lineStructured,
    choice,
    contestVerdict,
    sliceIndex,
    l,
    declared = [],
    disputedWordings = [],
  }: {
    readonly sourceText: string;
    readonly standingText: string;
    readonly incumbentText: string;
    readonly syntax?: SliceSyntax;
    readonly lineStructured: boolean;
    readonly choice: LaneChoice;
    readonly contestVerdict: ArtifactContestVerdict;
    readonly sliceIndex: number;
    readonly l: Logger;
    readonly declared?: readonly DeclaredNamePair[];
    readonly disputedWordings?: readonly DisputedWording[];
  },
): StandingVerdict {
  /**
   Syntax verdict for standing text, or ordinary prose admission.
   */
  const validation = validateTranslatedSlice({
    sourceText,
    candidateText: standingText,
    pageText: incumbentText,
    ...((syntax === undefined) ? {} : { syntax, }),
    lineStructured,
    declared,
    disputedWordings,
  },);
  /**
   Whether standing text itself passes syntax-bearing publication rules.
   */
  const standingValid = validation.kind === 'valid';
  if (standingValid) {
    /**
     Whether this baseline has prior approval and may ship unchanged.
     */
    const standingMayShip = contestStandingMayShip({
      choice,
      verdict: contestVerdict,
      standingValid,
    },);
    if (!standingMayShip) {
      l.warn(
        `slice ${String(sliceIndex,)}: consolidation standing text lacks contest endorsement and remains retryable`,
      );
    }
    return {
      standingValid,
      standingMayShip,
      settlementText: standingText,
      findings: [],
      incumbentStandsIn: false,
    };
  }

  /**
   Why the gate refused the standing, for the warning either branch logs.
   */
  const refusal = describeStandingVerdict({ validation, },);

  /**
   Gate's verdict on the incumbent, read only where it is a different text
   the slate could keep instead: an absent incumbent has nothing to offer,
   and a standing that IS the incumbent was refused as one text.
   */
  const incumbentValidation = ((incumbentText === '') || (incumbentText === standingText))
    ? undefined
    : validateTranslatedSlice({
      sourceText,
      candidateText: incumbentText,
      pageText: incumbentText,
      ...((syntax === undefined) ? {} : { syntax, }),
      lineStructured,
      declared,
      disputedWordings,
    },);
  if (incumbentValidation?.kind === 'valid') {
    l.warn(
      `slice ${String(sliceIndex,)}: consolidation standing text fails the deterministic publication rule `
        + `(${refusal}); the incumbent passes it and stands in as the wording the slate may keep, without contest `
        + 'endorsement',
    );
    return {
      standingValid: true,
      standingMayShip: false,
      settlementText: incumbentText,
      findings: [INELIGIBLE_STANDING_REPLACED_FINDING,],
      incumbentStandsIn: true,
    };
  }
  l.warn(
    `slice ${String(sliceIndex,)}: consolidation standing text fails the deterministic publication rule and is `
      + `withheld from the slate: ${refusal}${
        (incumbentValidation === undefined)
          ? ''
          : `; the incumbent fails it too: ${describeStandingVerdict({ validation: incumbentValidation, },)}`
      }`,
  );
  return {
    standingValid,
    standingMayShip: false,
    settlementText: standingText,
    findings: [],
    incumbentStandsIn: false,
    standingRefusal: refusal,
  };
}

//endregion Consolidation standing verdict
