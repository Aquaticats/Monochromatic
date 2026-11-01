import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { RENDERING_GLOSSARY, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  droppedAddressFindings,
  droppedSuicideFindings,
  latinTitleFindings,
  strayClosingQuoteFindings,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import { communityTermFindings, } from './translate-community-term.ts';
import { declaredLinkNameFindings, } from './translate-declared-link-name.ts';
import { addedFormulaFindings, } from './translate-formula.ts';
import { guillemetFindings, } from './translate-guillemets.ts';
import { hanTitleFindings, } from './translate-han-title.ts';
import { droppedMarkerFindings, } from './translate-marker-drop.ts';
import { unwrappedLinkFindings, } from './translate-unwrapped-link.ts';

//region Source carry floors
// What the original passage carries that every candidate must carry too,
// read before any judge: its footnote markers (class ninety-two), its
// quotations opened before they close (class one hundred sixty-five), no
// formula it does not set (ledger X22), its
// second-person address (class ninety-seven), the suicide it names (class
// one hundred fifty), its bracketed work titles
// in English (class ninety-eight) and set in quotation marks rather than
// 《》 (class one hundred forty-one), its quotations in quotation marks rather
// than guillemets (ledger B24), its community terms as the glossary
// renders them (class one hundred nineteen), its ordinary words without the
// calques the rendering glossary refuses (class one hundred twenty-three),
// its worded links as links (class one
// hundred fifteen) and a declared name inside a linked title in its
// declared form (class one hundred fourteen). The floors run in that order and the
// first one that speaks decides, so a candidate is refused for one thing at
// a time.

/**
 Findings of the first source-carry floor a candidate fails, empty where it
 passes them all.

 @param sourceText - original passage

 @param candidateText - rendering under the floors

 @param pageText - text the rendering would replace, empty where the page
 has none

 @param declared - name pairs the front matter declares, none where the
 caller has no front matter to read

 @returns Findings of the first failing floor, or none

 @example
 ```ts
 const findings = sourceCarryFindings({ sourceText, candidateText, pageText, },);
 ```
 */
export function sourceCarryFindings(
  {
    sourceText,
    candidateText,
    pageText,
    declared = [],
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
    readonly pageText: string;
    readonly declared?: readonly DeclaredNamePair[];
  },
): readonly string[] {
  /**
   Footnote markers the candidate dropped.
   */
  const markerFindings = droppedMarkerFindings({
    sourceText,
    candidateText,
  },);
  if (markerFindings.length > 0)
    return markerFindings;
  /**
   Closing quotation marks the candidate never opened.
   */
  const quoteFindings = strayClosingQuoteFindings({
    sourceText,
    candidateText,
  },);
  if (quoteFindings.length > 0)
    return quoteFindings;
  /**
   Formulas the candidate forms and the original does not.
   */
  const formulaFindings = addedFormulaFindings({
    sourceText,
    candidateText,
  },);
  if (formulaFindings.length > 0)
    return formulaFindings;
  /**
   Second-person address the candidate turned into the third person.
   */
  const addressFindings = droppedAddressFindings({
    sourceText,
    candidateText,
  },);
  if (addressFindings.length > 0)
    return addressFindings;
  /**
   Suicide the original names and the candidate does not say.
   */
  const suicideFindings = droppedSuicideFindings({
    sourceText,
    candidateText,
  },);
  if (suicideFindings.length > 0)
    return suicideFindings;
  /**
   Bracketed work titles the candidate left in Han.
   */
  const titleFindings = hanTitleFindings({
    sourceText,
    candidateText,
    pageText,
  },);
  if (titleFindings.length > 0)
    return titleFindings;
  /**
   English titles the candidate set in the Chinese title marks.
   */
  const latinFindings = latinTitleFindings({
    candidateText,
    pageText,
  },);
  if (latinFindings.length > 0)
    return latinFindings;
  /**
   A quotation the candidate set in guillemets (ledger B24).
   */
  const guillemetMarks = guillemetFindings({
    sourceText,
    candidateText,
    pageText,
  },);
  if (guillemetMarks.length > 0)
    return guillemetMarks;
  /**
   Community terms the candidate kept in Han or wrote in a refused form.
   */
  const termFindings = communityTermFindings({
    sourceText,
    candidateText,
  },);
  if (termFindings.length > 0)
    return termFindings;
  /**
   Ordinary words the candidate kept in Han or wrote as a refused calque.
   */
  const renderingFindings = communityTermFindings({
    sourceText,
    candidateText,
    glossary: RENDERING_GLOSSARY,
    noun: 'word',
  },);
  if (renderingFindings.length > 0)
    return renderingFindings;
  /**
   Worded links the candidate unwrapped while keeping the destination.
   */
  const linkFindings = unwrappedLinkFindings({
    sourceText,
    candidateText,
  },);
  if (linkFindings.length > 0)
    return linkFindings;
  return declaredLinkNameFindings({
    sourceText,
    candidateText,
    declared,
    pageText,
  },);
}

//endregion Source carry floors
