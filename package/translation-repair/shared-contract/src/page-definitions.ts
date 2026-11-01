import { parseDocument, } from './parse-document.ts';
import { collectDefinitions, } from './refine-envelope.ts';

//region Page definitions
// THE DEFINITIONS OF A PAGE, which a slice parsed alone cannot see. A reference (`[the dogs][b]`, `[dogs' bowl]`) or a footnote
// reference (`[^1]`) whose definition stands in another slice is plain text to a parse of its own slice and a label to a parse
// of the page, so a reader that parses a slice alone reads a different thing than the page shows: the label's characters are
// words, a mark beside it has no run to step over, and a rule that curls the reference does not curl the definition. The
// refinement gate met the same limit and collects the page's definition lines for it (`collectDefinitions`, `refine-envelope.ts`).
// This is that reader for a page held as text: the one place the readers that parse a slice (the prose mask of the typography
// rules, the prose view and the paragraphs of the consistency run) take the definitions from, so they agree with the gate.
//
// A reader appends the definitions after the slice with a blank line, so no offset of the slice moves, and keeps only what lies
// inside the slice. A page that holds no definition costs no parse: a definition has a closing bracket and a colon after its
// label, so a text without the two characters in a row holds none.

/**
 What a definition line holds after its label: the closing bracket and the colon.
 */
const DEFINITION_MARKER = ']:';

/**
 How many page texts the memory of definitions holds: the pass asks about the one page it works on for every place.
 */
const REMEMBERED_PAGES = 4;

/**
 The page texts read before and the definitions they held, the newest last.
 */
const rememberedDefinitions: {
  readonly pageText: string;
  readonly definitions: string;
}[] = [];

/**
 The link and footnote definition lines of a page, joined the way the refinement gate joins them.

 @param pageText - the page, or the slices of it that carry text, joined by blank lines

 @returns The definition lines in page order, one per definition, empty when the page holds none

 @example
 ```ts
 const definitions = pageDefinitionsOf({ pageText: 'See [a][x].\n\n[x]: https://x.example/x', },); // '[x]: https://x.example/x'
 ```
 */
export function pageDefinitionsOf({ pageText, }: { readonly pageText: string; },): string {
  if (!pageText.includes(DEFINITION_MARKER,))
    return '';
  /**
   The answer read before for an equal page.
   */
  const known = rememberedDefinitions.find(function isSame(entry,): boolean {
    return entry.pageText === pageText;
  },);
  if (known !== undefined)
    return known.definitions;
  /**
   The definitions of the page, read the way the gate reads them.
   */
  const definitions = collectDefinitions({ document: parseDocument({ text: pageText, },), },);
  rememberedDefinitions.push({
    pageText,
    definitions,
  },);
  if (rememberedDefinitions.length > REMEMBERED_PAGES)
    rememberedDefinitions.shift();
  return definitions;
}

/**
 A slice with the page's definitions after it, which the grammar parses so a reference in the slice resolves. The slice stays at
 the start, so every offset of the slice is an offset of the result.

 @param text - the slice

 @param definitions - the page's definitions, empty for a slice read alone

 @returns The slice alone, or the slice, a blank line and the definitions

 @example
 ```ts
 const parsed = withDefinitions({ text: 'See [a][x].', definitions: '[x]: https://x.example/x', },);
 ```
 */
export function withDefinitions(
  {
    text,
    definitions,
  }: {
    readonly text: string;
    readonly definitions: string;
  },
): string {
  if (definitions === '')
    return text;
  return `${text}\n\n${definitions}`;
}

//endregion Page definitions
