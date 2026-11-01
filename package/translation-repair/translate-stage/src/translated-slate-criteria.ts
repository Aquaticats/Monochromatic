import type { SelectEvidence, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  renderedBreakContract,
  renderedBreakFacts,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  translateSelectionCriteria,
  type TranslateSlateEntry,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

/**
 Adds the rendered-structure instruction to the actual anonymous slate's criteria.
 The original criteria object survives unchanged where no extra instruction is due.
 The instruction is fixed text; the measured counts of this call travel as evidence
 (`translatedSlateBreakEvidence`), because a criteria sentence is part of the cached
 system prompt and the counts are this call's own values.

 @param sourceText - exact original, excluding supplemental evidence

 @param archiveText - canonical archive wording, not a generated standing

 @param lineStructured - existing blank-separated verse-unit fact

 @param syntax - dedicated metadata grammar when present

 @returns Selection criteria with the source-only rendered instruction when applicable

 @example
 ```ts
 const criteria = translatedSlateCriteria({ sourceText, archiveText, lineStructured });
 ```
 */
export function translatedSlateCriteria(
  {
    sourceText,
    archiveText,
    lineStructured,
    syntax,
  }: {
    readonly sourceText: string;
    readonly archiveText: string;
    readonly lineStructured: boolean;
    readonly syntax?: SliceSyntax;
  },
): readonly string[] {
  /**
   Existing criteria retain their established precedence.
   */
  const criteria = translateSelectionCriteria({
    lineStructured,
    ...((syntax === undefined) ? {} : { syntax, }),
  },);
  /**
   The fixed instruction, empty where the passage does not call for it.
   */
  const rendered = renderedBreakContract({
    sourceText,
    archiveText,
    ...((syntax === undefined) ? {} : { syntax, }),
  },);
  return rendered === '' ? criteria : [
    ...criteria,
    rendered,
  ];
}

/**
 Measured rendered-structure facts of the actual anonymous slate, as an evidence block.

 @param sourceText - exact original, excluding supplemental evidence

 @param archiveText - canonical archive wording, not a generated standing

 @param syntax - dedicated metadata grammar when present

 @param slate - recorded order also given to the judge renderer

 @returns One evidence block with the source-only rendered facts, none where they are not due

 @example
 ```ts
 const evidence = translatedSlateBreakEvidence({ sourceText, archiveText, slate });
 ```
 */
export function translatedSlateBreakEvidence(
  {
    sourceText,
    archiveText,
    syntax,
    slate,
  }: {
    readonly sourceText: string;
    readonly archiveText: string;
    readonly syntax?: SliceSyntax;
    readonly slate: readonly TranslateSlateEntry[];
  },
): readonly SelectEvidence[] {
  /**
   Labels come from the recorded rotated slate, never producer identities.
   */
  const facts = renderedBreakFacts({
    sourceText,
    archiveText,
    ...((syntax === undefined) ? {} : { syntax, }),
    renderings: slate.map(function displayed(entry,) {
      return {
        label: `Candidate ${String(entry.index,)}`,
        text: entry.text,
      };
    },),
  },);
  if (facts === '')
    return [];
  return [
    {
      label: 'RENDERED LINE STRUCTURE, measured facts',
      text: facts,
    },
  ];
}
