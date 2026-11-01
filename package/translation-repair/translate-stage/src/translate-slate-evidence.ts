import type { SelectEvidence, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import { citedReferenceEvidence, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { JUDGE_PICTURE_SCOPE_RULE, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  tenseCandidateGroups,
  tenseCandidateLines,
} from './tense-candidate-block.ts';

//region Translate slate evidence
// WHAT THE TRANSLATE SLATE SHOWS BESIDE ITS CANDIDATES, in one place so the
// rendered-sheets fixture renders what `judgeTranslateSlate` sends rather than
// a copy that can drift from it (ledger B28). Moved from `translate-judge.ts`
// unchanged.

/**
 Evidence the translate slate shows beside its candidates: the original, then
 each context block the caller has.

 @param sourceText - original slice text

 @param pictureContext - corroborated readings of the pictures the passage
 shows, omitted when there are none

 @param neighbouringSourceText - original of the sections either side,
 omitted unless the caller asked for the wider window

 @param neighbouringIncumbentText - archive English of the sections either
 side, omitted unless the caller asked for the wider window

 @param identityContext - declared names from both sides' front matter,
 omitted when neither declares anything

 @param referenceContext - what the pages the original cites say, omitted
 when it cites none

 @param archiveDisputeNote - why the incumbent shown is the repair lane's
 stand-in, omitted on an undisputed slice

 @param lifeEnded - whether the original's entry records a death, which adds
 the tense candidates of the ballot texts; omitted when it records none

 @param subjectNames - names the translation declares for its subject, its display
 name and its aliases, read as named subjects by the tense candidates; omitted
 when it declares none

 @param ballotTexts - the texts on the ballot as the judges see them, one per
 candidate in the order the sheet numbers them, read only for tense candidates

 @returns Labelled evidence entries, in the order the judges read them

 @example
 ```ts
 const evidence = translateSlateEvidence({ sourceText, identityContext, },);
 ```
 */
export function translateSlateEvidence(
  {
    sourceText,
    pictureContext,
    neighbouringSourceText,
    neighbouringIncumbentText,
    identityContext,
    referenceContext,
    archiveDisputeNote,
    lifeEnded = false,
    subjectNames,
    ballotTexts = [],
  }: {
    readonly sourceText: string;
    readonly pictureContext?: string;
    readonly neighbouringSourceText?: string;
    readonly neighbouringIncumbentText?: string;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly archiveDisputeNote?: string;
    readonly lifeEnded?: boolean;
    readonly subjectNames?: readonly string[];
    readonly ballotTexts?: readonly string[];
  },
): readonly SelectEvidence[] {
  /**
   Tense candidates of the ballot texts, one group per text that has any.
   */
  const tenseGroups = tenseCandidateGroups({
    lifeEnded,
    ...((subjectNames === undefined) ? {} : { subjectNames, }),
    texts: ballotTexts.map(function ballotSource(
      text,
      index,
    ) {
      return {
        label: `CANDIDATE ${String(index + 1,)}`,
        text,
      };
    },),
  },);
  return [
    {
      label: 'ORIGINAL (Chinese)',
      text: sourceText,
    },
    // WHAT THE PICTURES SAY, placed beside the original because that is what
    // it is: a passage showing a picture has a source a judge cannot read,
    // and English transcribing that picture looks like invention until this
    // block exists. Only corroborated readings appear here; a picture nobody
    // could read is a finding rather than a hedge for a judge to weigh.
    ...((pictureContext === undefined) || (pictureContext === '')
      ? []
      : [
        {
          // The scope rides in the label (ledger S16, H7): read with no rule,
          // a neighbouring picture's transcript looked like faithful content.
          label: `WHAT THE PICTURES HERE SAY, transcribed by two readers that agreed; ${JUDGE_PICTURE_SCOPE_RULE}`,
          text: pictureContext,
        },
      ]),
    // Neighbouring sections travel as CONTEXT, never as something to render,
    // and only when a caller asked for them, so a run that does not want the
    // wider window renders the sheet this stage has always sent. The label
    // carries the caveat because a judge that reads it as required content
    // starts filing coverage complaints against every candidate.
    ...((neighbouringSourceText === undefined) || (neighbouringSourceText === '')
      ? []
      : [
        {
          label: 'SURROUNDING ORIGINAL (Chinese), context only: the candidates are not expected to render this',
          text: neighbouringSourceText,
        },
      ]),
    // THE OTHER HALF OF THE SAME WINDOW, and the half that names the failure
    // relocation causes. A judge shown only this slice cannot tell a passage
    // the archive INVENTED from one it carried across a boundary, and it
    // condemns the archive for both. Measured over 92 entries and 1260
    // slices, every relocation pair is adjacent, so the passage it is looking
    // for is in this block whenever it is anywhere.
    ...((neighbouringIncumbentText === undefined) || (neighbouringIncumbentText === '')
      ? []
      : [
        {
          label: 'SURROUNDING EXISTING TRANSLATION (English), context only: '
            + 'wording that belongs to the passages either side of this one. A '
            + 'candidate is not expected to render any of it. Where wording the '
            + 'ORIGINAL of THIS passage calls for is already sitting here, the '
            + 'archive carried it across a boundary rather than inventing it.',
          text: neighbouringIncumbentText,
        },
      ]),
    // Declared names travel as evidence rather than as part of a candidate,
    // because a judge cannot check criterion three without them. The existing
    // translation deliberately does NOT travel here: it is on the ballot,
    // anonymously, and showing it twice would tell the judges which candidate
    // is the incumbent.
    ...((identityContext === undefined) || (identityContext === '')
      ? []
      : [
        {
          label: 'DECLARED NAMES',
          text: identityContext,
        },
      ]),
    // The pages the original cites, after the names, so a candidate keeping
    // a detail only a cited page states is judged on evidence rather than
    // condemned for saying what the original does not.
    ...citedReferenceEvidence({ referenceContext: referenceContext ?? '', },),
    // Why the incumbent on the ballot is the repair lane's stand-in, after
    // the references, so a candidate leaving a disputed detail out is not
    // read as dropping the page's own content (class one hundred eight).
    ...((archiveDisputeNote === undefined) || (archiveDisputeNote === '')
      ? []
      : [
        {
          label: 'ARCHIVE RENDERING DISPUTED',
          text: archiveDisputeNote,
        },
      ]),
    // Tense candidates of the ballot texts, the last of the evidence. The sheet
    // prints all evidence before the numbered candidates, so a judge reads the
    // flags first and the texts they name after, under the numbers the flags
    // give them. Shown only where the entry records a death.
    ...((tenseGroups.length === 0)
      ? []
      : [
        {
          label: 'TENSE CANDIDATES, to check against the house tense rule',
          text: tenseCandidateLines({ groups: tenseGroups, },)
            .join('\n',),
        },
      ]),
  ];
}

//endregion Translate slate evidence
