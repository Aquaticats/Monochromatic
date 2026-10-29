import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  type DisputedWording,
  type FloorGroundDisagreementError,
  requireComparedVerdict,
  type TranslateReportWire,
  validateTranslatedSlice,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Translate floor
// THE SIXTY-SIXTH CLASS (XingZ615, 2026-09-19). The translate lane sends a
// candidate the deterministic publication rule refuses back to its author
// once (the owner's decision of 2026-08-15), and kept it on the slate when
// the revision failed too, on the reading that the original was at least
// what the model produced with the whole sheet in front of it. The judges
// do not apply the rule: on the closing poem they chose the rendering that
// moved the attribution line out of the quote (three blocks against the
// original's two) over two renderings that kept it, the lane contest
// flagged the winner "retryable" and moved on, and the consolidation
// withheld it as the standing with nothing valid left to ship. A text the
// rule refuses cannot reach the page, so after its turn it is withheld from
// the judges and kept as a finding; the repair turn and the model's own
// defence are unchanged.

/**
 Finding prefix a withheld candidate is recorded under.
 */
const REFUSED_FINDING = 'translate-candidate-refused';

/**
 Withholds from the slate every candidate the deterministic publication rule
 still refuses after its repair turn.

 @param voices - candidates as the repair turn left them

 @param sourceText - original slice

 @param incumbentText - incumbent the stage judges, which the rule reads as
 the page shape: the archive's rendering, or on a disputed slice the repair
 lane's text standing in its place where that may stand

 @param syntax - syntax role of the slice, when it has one

 @param lineStructured - whether the slice owes one line per line

 @param declared - name pairs the front matter declares, which the rule
 reads for a linked title naming a declared person (class one hundred
 fourteen); none leaves that floor silent

 @param disputedWordings - wordings a disputed slice refuses, so a candidate
 copying the archive there is withheld (owner, 2026-09-27); none elsewhere

 @param l - stage logger, which names each candidate withheld: the finding
 reaches only the artifact, and a slate thinned with nothing in the log reads
 as judges choosing among fewer voices (ledger E5)

 @returns Candidates the rule accepts, with a finding per candidate withheld

 @throws {@link FloorGroundDisagreementError}
 when the rule says it compared nothing, which the stage rules out before
 any candidate is written (ledger B43)

 @example
 ```ts
 const floored = floorTranslateVoices({ voices, sourceText, incumbentText, lineStructured, l, },);
 ```
 */
export function floorTranslateVoices(
  {
    voices,
    sourceText,
    incumbentText,
    syntax,
    lineStructured,
    declared = [],
    disputedWordings = [],
    l,
  }: {
    readonly voices: readonly HeardVoice<TranslateReportWire>[];
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly syntax?: SliceSyntax;
    readonly lineStructured: boolean;
    readonly declared?: readonly DeclaredNamePair[];
    readonly disputedWordings?: readonly DisputedWording[];
    readonly l: Logger;
  },
): {
  readonly voices: readonly HeardVoice<TranslateReportWire>[];
  readonly findings: readonly string[];
} {
  /**
   Logger tagged with this floor.
   */
  const fl = tagged({
    tag: floorTranslateVoices.name,
    l,
  },);
  /**
   Findings for the candidates withheld, in voice order.
   */
  const findings: string[] = [];
  /**
   Candidates the rule accepts, in voice order.
   */
  const accepted = voices.filter(function acceptedByRule(voice,): boolean {
    /**
     What the rule makes of this candidate: a pass or a refusal, since the
     stage asks nobody on a slice the rule can compare nothing on (ledger
     B43), and an answer that it compared nothing here is a fault rather
     than a candidate to keep.
     */
    const validation = requireComparedVerdict({
      verdict: validateTranslatedSlice({
        sourceText,
        candidateText: voice.value
          .translation,
        pageText: incumbentText,
        ...((syntax === undefined) ? {} : { syntax, }),
        lineStructured,
        declared,
        disputedWordings,
      },),
    },);
    if (validation.kind === 'valid')
      return true;
    /**
     Why the rule refused it, as one line.
     */
    const reason = validation.findings
      .join(' ',);
    findings.push(`${REFUSED_FINDING} (${voice.modelId}): ${reason}`,);
    fl.warn(
      `withheld ${voice.modelId} from the judges: the publication rule still refuses its candidate after its repair turn: ${reason}`,
    );
    return false;
  },);
  return {
    voices: accepted,
    findings,
  };
}

//endregion Translate floor
