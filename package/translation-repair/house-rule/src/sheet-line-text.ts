//region Sheet line text
// TEXT INTERPOLATED INTO A LINE-BASED SHEET OBEYS THAT SHEET'S LINE GRAMMAR
// (ledger L14(d), 2026-09-28). The panel, checker, editor and probe sheets,
// the human grading sheets and the filing log lines tell their items apart
// by the lines that open them, and rendered claim summaries and quotes raw:
// 349 of 45,860 quotes and 3 of 23,714 summaries over every artifact carry a
// line break, so a later line stood as a line of its own, and one reading
// like `CLAIM 2` would renumber every ballot after it. A summary is one
// sentence of prose and folds to one line; a quote is evidence and keeps its
// lines, each later one indented under its item.

/**
 Characters a reader may take as ending a line: line feed, vertical tab,
 form feed, carriage return, next line, line separator and paragraph
 separator.
 */
const LINE_BREAKS: ReadonlySet<string> = new Set([
  '\n',
  '\v',
  '\f',
  '\r',
  '\u0085',
  '\u2028',
  '\u2029',
],);

/**
 Collapses whitespace runs to single spaces and trims, so a quote that differs
 from the text only in wrapping still matches, and prose written across lines
 reads as the one line a sheet item holds.

 Written as a linear scan rather than a pattern: the rule is one predicate per
 character with one bit of carried state, which reads more plainly this way
 and cannot backtrack over adversarial input.

 @param text - text to normalize

 @returns Text with whitespace runs collapsed

 @example
 ```ts
 flattenSpace({ text: 'The  cat\n naps', },);
 ```
 */
export function flattenSpace({ text, }: { readonly text: string; },): string {
  /**
   Characters kept so far, whitespace already collapsed.
   */
  const kept: string[] = [];
  for (const character of text) {
    if (character.trim() !== '') {
      kept.push(character,);
      continue;
    }
    if (kept.at(-1,) !== ' ')
      kept.push(' ',);
  }
  return kept.join('',)
    .trim();
}

/**
 Keeps every line of a text inside the sheet item it is written into: each
 line after the first opens with the indent, so none can open an item of its
 own. A carriage return before a line feed ends one line, not two.

 A linear scan, for the reason {@link flattenSpace} gives.

 @param text - text written after an item's opening, such as a quote

 @param indent - spaces the item's continuation lines carry

 @returns Text whose later lines all open with the indent

 @example
 ```ts
 indentContinuation({ text: 'hunts\nat noon', indent: '  ', },);
 ```
 */
export function indentContinuation(
  {
    text,
    indent,
  }: {
    readonly text: string;
    readonly indent: string;
  },
): string {
  return Array.from(
    // A carriage return before a line feed ends one line, not two.
    text.replaceAll(
      '\r\n',
      '\n',
    ),
    function toKept(character,) {
      return LINE_BREAKS.has(character,) ? `\n${indent}` : character;
    },
  )
    .join('',);
}

//endregion Sheet line text
