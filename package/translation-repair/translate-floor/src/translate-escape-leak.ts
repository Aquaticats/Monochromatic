//region Leaked JSON escapes
// Moved out of `translate-validate.ts` on 2026-09-23 when the class
// ninety-two floor took that file past the line budget; the rule and its
// history are unchanged.

/**
 A backslash before a double quotation mark, which is how a JSON string
 escapes the mark and never how a page writes one.
 */
const LEAKED_ESCAPE = String.raw`\"`;

/**
 Findings for JSON escapes that leaked into a candidate as text.

 WHY. A model answering in JSON sometimes escapes the quotation marks inside
 its string twice, and the decoded text then carries a literal backslash
 before each mark. The Carena0442 page published on 2026-09-02 shipped
 `so-called \"common sense.\"` that way: the consolidation producer's text
 carried it, every structural guard passed it, and the polish kept it. The
 pinned corpus carries no such sequence in any page, so a candidate carrying
 one where neither the original nor the page does is a leak, not a rendering.

 @param sourceText - original slice

 @param pageText - page slice the candidate would replace

 @param candidateText - candidate under validation

 @returns One finding when the candidate alone carries the sequence

 @example
 ```ts
 leakedEscapeFindings({ sourceText: '“常识”', pageText: '“common sense”', candidateText: '\\"common sense\\"', },);
 ```
 */
export function leakedEscapeFindings(
  {
    sourceText,
    pageText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly pageText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  if (!candidateText.includes(LEAKED_ESCAPE,))
    return [];
  if (sourceText.includes(LEAKED_ESCAPE,) || pageText.includes(LEAKED_ESCAPE,))
    return [];
  return [
    `Your translation carries a backslash before a quotation mark (${LEAKED_ESCAPE}), which is a JSON `
      + 'escape leaked into the text; write the quotation marks themselves, with nothing before them.',
  ];
}

//endregion Leaked JSON escapes
