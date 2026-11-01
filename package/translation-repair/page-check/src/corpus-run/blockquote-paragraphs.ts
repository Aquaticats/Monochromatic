//region Blockquote paragraphs
// The blockquote reading the class one hundred sixty-eight pass
// (`blockquote-quote-unify.ts`) stands on: a line split into its markers and
// its words, and a slice's blockquote paragraphs as line ranges. Kept apart so
// neither file outgrows the line budget.

/**
 Units a blockquote line's prefix is made of: markers and the spaces
 between them.
 */
const PREFIX_UNITS = '> ';

/**
 One blockquote line split into its markers and its words.
 */
export type QuotedLine = {
  readonly prefix: string;
  readonly content: string;
};

/**
 First and last line index of one blockquote paragraph.
 */
export type LineRange = {
  readonly first: number;
  readonly last: number;
};

/**
 Splits a line into its blockquote markers and its words; a line with no
 marker has an empty prefix.

 @param line - one line of a slice

 @returns Prefix of markers and spaces, and the rest

 @example
 ```ts
 splitQuotedLine({ line: '> Purr.', },); // { prefix: '> ', content: 'Purr.' }
 ```
 */
export function splitQuotedLine({ line, }: { readonly line: string; },): QuotedLine {
  // Code-unit scan: the prefix units are BMP characters, so a surrogate half
  // reads as a word unit and ends the prefix.
  /**
   Line as code units.
   */
  const units = Array.from(
    { length: line.length, },
    function unit(
      _unused,
      index,
    ): string {
      return line.charAt(index,);
    },
  );
  /**
   First unit past the prefix, or none on a line of markers alone.
   */
  const firstWord = units.findIndex(function isWord(unit,): boolean {
    return !PREFIX_UNITS.includes(unit,);
  },);
  /**
   Where the prefix ends.
   */
  const end = firstWord === (-1) ? line.length : firstWord;
  /**
   Leading run, which is a prefix only when it holds a marker.
   */
  const lead = line.slice(
    0,
    end,
  );
  if (!lead.includes('>',)) {
    return {
      prefix: '',
      content: line,
    };
  }
  return {
    prefix: lead,
    content: line.slice(end,),
  };
}

/**
 Line ranges of a text's blockquote paragraphs: runs of quoted lines with
 words, ended by a bare marker line or a line outside the blockquote.

 @param lines - text split at line feeds

 @returns Each paragraph's first and last line

 @example
 ```ts
 blockquoteParagraphs({ lines: ['> Purr.', '>', '> Nap.',], },); // two ranges
 ```
 */
export function blockquoteParagraphs({ lines, }: { readonly lines: readonly string[]; },): readonly LineRange[] {
  // ONE PASS THAT APPENDS, since copying the paragraphs found so far at every
  // line costs the square of the line count (ledger B70).
  /**
   Paragraphs found so far, the last one extended while lines continue it.
   */
  const paragraphs: LineRange[] = [];
  for (const [index, line,] of lines.entries()) {
    /**
     This line's markers and words.
     */
    const {
      prefix,
      content,
    } = splitQuotedLine({ line, },);
    /**
     Words of the line without edge spaces.
     */
    const words = content.trim();
    if ((prefix === '') || (words === ''))
      continue;
    /**
     Paragraph the previous line may have continued.
     */
    const open = paragraphs.at(-1,);
    /**
     Line before this one.
     */
    const previous = index - 1;
    if ((open !== undefined) && (open.last === previous)) {
      paragraphs[paragraphs.length - 1] = {
        first: open.first,
        last: index,
      };
      continue;
    }
    paragraphs.push({
      first: index,
      last: index,
    },);
  }
  return paragraphs;
}

/**
 Words of one paragraph's lines, joined.

 @param lines - text split at line feeds

 @param range - lines of the paragraph

 @returns Paragraph words with line feeds between lines

 @example
 ```ts
 paragraphWords({ lines: ['> “Purr.”',], range: { first: 0, last: 0, }, },); // '“Purr.”'
 ```
 */
export function paragraphWords(
  {
    lines,
    range,
  }: {
    readonly lines: readonly string[];
    readonly range: LineRange;
  },
): string {
  return lines
    .slice(
    range.first,
    range.last + 1,
    )
    .map(function words(line,): string {
      return splitQuotedLine({ line, },)
        .content;
    },)
    .join('\n',);
}

//endregion Blockquote paragraphs
