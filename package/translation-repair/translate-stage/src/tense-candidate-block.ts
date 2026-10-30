import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  refusalText,
  requireMdxRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  presentTenseCandidates,
  type PresentTenseCandidate,
} from './present-tense-candidates.ts';

//region Tense candidate block
// THE BLOCK A JUDGING SHEET CARRIES FOR TENSE CANDIDATES. Shown only where the
// original's entry records a death, and only as sentences to check against the
// house tense rule. It is never a refusal and never evidence against a text: a
// candidate may be a trait, a standing fact or a misread quotation, which the
// rule leaves alone. A text with no candidate contributes no group.
//
// ONE PLACE LETS THE GROUPS DECIDE, by the owner's narrow exception of 2026-10-29
// to the ruling of 2026-10-11 ("Swap only where flagged", ledger B1050):
// `consolidate-tense-swap.ts` ships the translate lane's two-ballot text over an
// eligible standing the contest did not endorse, where no vote can seat two
// judges and the groups flag the standing and not that text. No sheet, refusal
// or count reads them as a verdict anywhere else.
//
// A TEXT THE SHARED GRAMMAR REFUSES contributes no group either, and is named
// in a warning. The detector is advisory, so a candidate wording that does not
// parse (an open brace, a tag left open) must not refuse the sheet that lets a
// judge weigh it; the judges' other checks already refuse such a wording.

/**
 Logger the unreadable texts are named on, which reads the entry and slice off the running call.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 One text on offer to a judging sheet, labelled as that sheet names it.

 @example
 ```ts
 const source: TenseCandidateSource = { label: 'CANDIDATE "repair"', text: 'Mittens naps.', };
 ```
 */
export type TenseCandidateSource = {
  /**
   Name the sheet gives the text, which heads its candidates.
   */
  readonly label: string;

  /**
   Text on offer, as the sheet shows it.
   */
  readonly text: string;
};

/**
 The candidates of one text on offer, under the name the sheet gives it.

 @example
 ```ts
 const group: TenseCandidateGroup = { label: 'ARCHIVE RENDERING', found: [], };
 ```
 */
export type TenseCandidateGroup = {
  /**
   Name the sheet gives the text.
   */
  readonly label: string;

  /**
   Candidates the detector flags in the text, in page order.
   */
  readonly found: readonly PresentTenseCandidate[];
};

/**
 Groups the texts on offer that have at least one tense candidate, in the order
 the sheet shows them.

 @param lifeEnded - whether the original's entry records a death, the typed flag the sheet carries

 @param subjectNames - names the translation declares for its subject, read as named subjects beside the pronouns; absent when it declares none

 @param texts - texts on offer, in the order the sheet shows them

 @returns One group per text with a candidate; empty when the entry records no death or no text has one

 @throws Whatever the detector raises other than the shared slice grammar's refusal, which names the text and lists none for it

 @example
 ```ts
 const groups = tenseCandidateGroups({ lifeEnded: true, texts: [{ label: 'ARCHIVE RENDERING', text: 'Mittens naps.', }], },);
 ```
 */
export function tenseCandidateGroups(
  {
    lifeEnded,
    subjectNames = [],
    texts,
  }: {
    readonly lifeEnded: boolean;
    readonly subjectNames?: readonly string[];
    readonly texts: readonly TenseCandidateSource[];
  },
): readonly TenseCandidateGroup[] {
  if (!lifeEnded)
    return [];
  /**
   Logger tagged with this function.
   */
  const tl = tagged({
    tag: tenseCandidateGroups.name,
    l,
  },);
  return texts
    .map(function candidatesOfText(source,): TenseCandidateGroup {
      try {
        return {
          label: source.label,
          found: presentTenseCandidates({
            text: source.text,
            subjectNames,
            died: lifeEnded,
          },),
        };
      }
      catch (error) {
        // Only the grammar's refusal is a handled state, and the slice grammar reads an exhaustion of its own parse as
        // that refusal (`parseSliceBody`); anything else, an exhaustion of the code that reads the blocks included,
        // keeps propagating.
        requireMdxRefusal({ error, },);
        tl.warn(
          `the text on offer labelled ${source.label} was refused by the shared slice grammar (${
            refusalText({ error, },)
          }), so its tense candidates are not listed`,
        );
        return {
          label: source.label,
          found: [],
        };
      }
    },)
    .filter(function hasCandidates({ found, },): boolean {
      return found.length > 0;
    },);
}

/**
 The explanatory text and the grouped candidates, one line each, for a sheet
 that shows them as text. Empty when there are no groups.

 @param groups - groups from {@link tenseCandidateGroups}

 @returns Lines naming each group and its candidates, with the rule the judge applies

 @example
 ```ts
 const lines = tenseCandidateLines({ groups: [], },);
 ```
 */
export function tenseCandidateLines(
  { groups, }: { readonly groups: readonly TenseCandidateGroup[]; },
): readonly string[] {
  if (groups.length === 0)
    return [];
  return [
    'These sentences match present-tense shapes in a memorial page about a person who has died.',
    'Check each against the house tense rule. A candidate may be a trait, a standing fact or a quotation',
    'the rule leaves alone, so it is a question for you and not a finding against the text.',
    ...groups.flatMap(function groupLines(group,): readonly string[] {
      return [
        `${group.label}:`,
        ...group.found
          .map(function candidateLine(candidate,): string {
          // Each text is written as a JSON string, so a quotation mark, a
          // backslash or a line break inside the sentence cannot end the line.
            return `- ${JSON.stringify(candidate.sentence,)} (matched ${JSON.stringify(candidate.matched,)}, rule ${candidate.rule})`;
          },),
      ];
    },),
  ];
}

//endregion Tense candidate block
