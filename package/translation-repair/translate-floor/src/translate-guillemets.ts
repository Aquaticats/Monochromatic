import { withoutHtmlComments, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  noteRefusedMask,
  proseMaskReading,
  textNamed,
} from '@monochromatic-dev/translation-repair-typography/ts';

//region Guillemet floor
// LEDGER B24 (2026-09-29). English prose on these pages sets a quotation in
// quotation marks, “ ” and ‘ ’ inside it, and never in guillemets. The
// TianqiChen66611 run shipped a translated slice with «» around a quotation,
// though one judge's reason on its ballot named the marks: a judge reading
// it did not keep it off the page, so the marks are refused HERE, before any
// judge reads the candidate. Across the stored artifacts that slice is the
// only one whose lane text carries a guillemet its incumbent lacks (1 of
// 6,285 comparison rows), and no pinned original or archive carries one (the
// same scan finds curly doubles in 71 archives), so the floor stands aside
// wherever the original or the page it would replace carries one. Only
// prose is read: a guillemet in a code span or a tag is content, and
// comments are cut; a candidate the grammar refuses has no parse to tell its prose from its code (`typography-prose-mask.ts`), so
// every guillemet outside a comment in it is read as a possible quotation and named (n159, X10): the floors that run after this
// one refuse an unparseable candidate only where the original parses (`translate-validate.ts`, the skeleton read after
// `sourceCarryFindings`), and the verdict is unknown where the original or the page is blind, so this floor does not stand aside
// for one. The editor sheet's «REGION n» marker is the sheet-leak
// floor's to refuse, which runs first and names it as the sheet's own.

/**
 Guillemets, double and single, each one UTF-16 unit.
 */
const GUILLEMETS: ReadonlySet<string> = new Set([
  '«',
  '»',
  '‹',
  '›',
],);

/**
 Whether a text carries a guillemet anywhere.

 @param text - original or page slice

 @returns Whether any guillemet stands in it

 @example
 ```ts
 carriesGuillemet({ text: '猫说：«我饿了。»', },); // true
 ```
 */
function carriesGuillemet({ text, }: { readonly text: string; },): boolean {
  return Array.from(text,)
    .some(function isGuillemet(character,): boolean {
      return GUILLEMETS.has(character,);
    },);
}

/**
 Guillemets a text's prose carries, each once, in order of first appearance.

 @param text - candidate with its comments cut

 @returns Distinct guillemets standing in prose

 @example
 ```ts
 proseGuillemets({ text: 'The cat said, «I am hungry.» `‹x›`', },); // ['«', '»']
 ```
 */
function proseGuillemets({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Which units are prose rather than code or markup, and whether the grammar refused the text.
   */
  const {
    mask,
    refused,
  } = proseMaskReading({ text, },);
  if (refused) {
    noteRefusedMask({
      reader: 'guillemetFindings',
      name: textNamed({ text, },),
    },);
  }
  /**
   Guillemets seen in prose so far, in order.
   */
  const seen: string[] = [];
  for (
    let index = 0;
    index < text.length;
    index += 1
  ) {
    /**
     Unit at this position; every guillemet is one unit.
     */
    const unit = text.charAt(index,);
    /**
     Whether it is a prose guillemet not yet named.
     */
    const fresh = (refused || (mask[index] === true))
      && GUILLEMETS.has(unit,)
      && (!seen.includes(unit,));
    if (fresh)
      seen.push(unit,);
  }
  return seen;
}

/**
 Findings against a candidate whose prose sets a quotation in guillemets;
 empty where none stands in its prose, or where the original or the page it
 would replace carries one.

 @param sourceText - original passage

 @param candidateText - rendering under the floor

 @param pageText - text the rendering would replace, empty where the page
 has none

 @returns One finding naming the guillemets, or none

 @example
 ```ts
 guillemetFindings({ sourceText: '猫说：「我饿了。」', candidateText: 'The cat said, «I am hungry.»', pageText: '', },);
 // one finding
 ```
 */
export function guillemetFindings(
  {
    sourceText,
    candidateText,
    pageText = '',
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
    readonly pageText?: string;
  },
): readonly string[] {
  if (carriesGuillemet({ text: sourceText, },) || carriesGuillemet({ text: pageText, },))
    return [];
  /**
   Guillemets the candidate's prose carries.
   */
  const marks = proseGuillemets({ text: withoutHtmlComments({ text: candidateText, },), },);
  if (marks.length === 0)
    return [];
  return [
    `Your translation sets a quotation in the guillemets ${marks.join(' ',)}, which English prose does not use: `
      + 'write quotation marks, “ ” around a quotation and ‘ ’ around one inside it.',
  ];
}

//endregion Guillemet floor
