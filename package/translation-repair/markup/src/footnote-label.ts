//region Footnote label
// THE LABEL OF A GFM FOOTNOTE, a reference `[^label]` or a definition `[^label]: body`, as the parser the corpus is
// built with reads it (`micromark-extension-gfm-footnote@2.1.0`, `lib/syntax.js:354-381` for the definition's
// `labelInside` and `labelEscape`, and the same rules in the reference's `callData` and `callEscape`): at least one
// character after the caret; no space, tab, carriage return or line feed; no opening bracket unless a backslash
// escapes it; a backslash before a bracket or another backslash takes that character with it, so the closing bracket
// of `[^a\]` is no closing bracket; and at most 999 characters between the caret and the closing bracket, an escape
// counting two. Any other character is part of the name, a no-break space among them, which is why
// the label is not tested with `trim()`.

/**
 The most characters GFM reads between the caret of a footnote label and its closing bracket.
 */
const MAX_LABEL_LENGTH = 999;

/**
 The characters a footnote label cannot hold unless a backslash escapes a bracket: white space the grammar names.
 */
const LABEL_BREAKS: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\r',
  '\n',
],);

/**
 The characters a backslash takes along into a footnote label, which then do not end or break it.
 */
const LABEL_ESCAPABLE: ReadonlySet<string> = new Set([
  '[',
  '\\',
  ']',
],);

/**
 Where the closing bracket of a footnote label stands, when the characters from an index on make a label.

 @param text - text that holds the label

 @param from - index of the first character after `[^`

 @returns Index of the closing bracket, or -1 when the characters are no label: one is a break or an unescaped opening
 bracket, the label is empty or longer than GFM reads, or no closing bracket follows

 @example
 ```ts
 const close = footnoteLabelEnd({ text: 'the [^1] cat', from: 6, },); // 6
 ```
 */
export function footnoteLabelEnd(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  /**
   Index of the character read.
   */
  const walk = { at: from, };
  while (walk.at < text.length) {
    /**
     Character at the index.
     */
    const character = text.charAt(walk.at,);
    if ((walk.at - from) > MAX_LABEL_LENGTH)
      return -1;
    if (character === ']')
      return (walk.at === from) ? -1 : walk.at;
    if ((character === '[') || LABEL_BREAKS.has(character,))
      return -1;
    walk.at += ((character === '\\') && LABEL_ESCAPABLE.has(text.charAt(walk.at + 1,),)) ? 2 : 1;
  }
  return -1;
}

/**
 Whether text between `[^` and the closing bracket is a footnote label.

 @param label - text between the caret and the closing bracket

 @returns Whether GFM reads it as the label of a footnote reference or definition

 @example
 ```ts
 isGfmFootnoteLabel({ label: '1', },); // true
 ```
 */
export function isGfmFootnoteLabel({ label, }: { readonly label: string; },): boolean {
  return footnoteLabelEnd({
    text: `${label}]`,
    from: 0,
  },) === label.length;
}

//endregion Footnote label
