import { isIdeograph, } from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Untranslated candidate
// MEASURED ON MIO25'S FIRST ATTEMPT (slice 16, 2026-09-17): two of the four
// translate candidates for the closing poem were the Chinese original
// returned as it stood, one judge chose one of them as "the most faithful
// rendering", the others abstained or split, the select missed its weight
// floor twice, and the entry ended with the translate lane's absence error
// because that slice has no archive translation to fall back on. A candidate
// that repeats a Han-carrying original is not a rendering of it and never
// belongs on a slate.
//
// REPETITION IN ALL BUT WHITESPACE. Exact repetition after trimming the
// ends was the first reading; on XingZ615 (2026-09-19, class sixty-five) a
// consolidation writer returned the closing poem's original with a space
// after each bare quote line, it passed the floor as the slate's only valid
// candidate, three of four judges abstained on "a verbatim copy of the
// Chinese original" and the entry stopped. A copy that differs only in
// whitespace is the original. A slice that carries nothing to translate (a
// bare link, a component) is returned as it stands and must pass; the
// ideograph test is what separates the two.

/**
 Finding text for the repeated original.
 */
const UNTRANSLATED_FINDING = 'Your translation repeats the ORIGINAL untranslated, in all but whitespace. '
  + 'Write the passage in English; keep only the names, handles, links and code the original carries.';

/**
 Text with every whitespace character removed, so two spellings of the same
 characters compare equal whatever their line breaks and spacing.

 @param text - text to strip

 @returns Its non-whitespace characters in order

 @example
 ```ts
 const bare = withoutWhitespace({ text: '> 猫 \r\n>\n', },); // '>猫>'
 ```
 */
export function withoutWhitespace({ text, }: { readonly text: string; },): string {
  /**
   Characters kept, in order.
   */
  const kept: string[] = [];
  // ONE LINEAR PASS over code points; a character that trims to nothing is
  // whitespace by the same definition `trim` uses.
  for (const character of text) {
    if (character.trim() !== '')
      kept.push(character,);
  }
  return kept.join('',);
}

/**
 Finding for a candidate that repeats a Han-carrying original untranslated.

 @param sourceText - original slice

 @param candidateText - candidate under validation

 @returns One finding when the candidate is the original in all but
 whitespace and the original carries an ideograph; none otherwise

 @example
 ```ts
 const findings = untranslatedFindings({ sourceText: '猫睡了。', candidateText: '猫睡了。', },);
 ```
 */
export function untranslatedFindings(
  {
    sourceText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  /**
   Original without any whitespace.
   */
  const source = withoutWhitespace({ text: sourceText, },);
  /**
   Candidate without any whitespace.
   */
  const candidate = withoutWhitespace({ text: candidateText, },);
  if (source !== candidate)
    return [];
  // ONE LINEAR PASS over code points; a character in the ideograph range is
  // what makes the repetition an untranslated one rather than a bare link or
  // component returned as it stands.
  for (const character of source) {
    if (isIdeograph(character,))
      return [UNTRANSLATED_FINDING,];
  }
  return [];
}

//endregion Untranslated candidate
