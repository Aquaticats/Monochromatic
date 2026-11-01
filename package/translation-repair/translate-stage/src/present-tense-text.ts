import { nestedSinglePairs, } from '@monochromatic-dev/translation-repair-typography/ts';
import { isWhitespace, } from './present-tense-words.ts';

//region Text reading
// The paragraph text the present-tense rules read: links unwrapped, marks and
// bare URLs dropped, and quoted speech blanked so the rules never read it. Every
// step keeps one scan over the text.

/**
 Offsets of one sentence within the text it was split from.
 */
export type SentenceSpan = {
  /**
   Offset of the first character.
   */
  readonly start: number;

  /**
   Exclusive end offset, after the run of sentence endings.
   */
  readonly end: number;
};

/**
 Characters that end a sentence; a run of them is one ending.
 */
const SENTENCE_ENDINGS: string = '.!?…';

/**
 Replaces runs of whitespace with one space and trims both ends.

 @param text - text to collapse

 @returns The collapsed text

 @example
 ```ts
 const collapsed = collapseSpaces({ text: 'Mittens \n naps', },);
 ```
 */
function collapseSpaces({ text, }: { readonly text: string; },): string {
  return (function collapse(): string {
    /**
     Pieces of the collapsed text, with no leading space and no doubled space, joined once at the end.
     */
    const pieces: string[] = [];

    /**
     Whether a whitespace run is waiting to become one space before the next word character.
     */
    let spacePending = false;
    for (const character of text) {
      if (isWhitespace({ character, },)) {
        spacePending = (pieces.length > 0);
        continue;
      }
      if (spacePending) {
        pieces.push(' ',);
        spacePending = false;
      }
      pieces.push(character,);
    }
    return pieces.join('',);
  })();
}

/**
 Reduces each complete Markdown link to its label, so `[Mittens](https://cats.example/m)` reads as `Mittens`.

 @param text - text that may contain links

 @returns The text with each complete link replaced by its label

 @example
 ```ts
 const labelled = unwrapLinks({ text: '[Mittens](https://cats.example/m) naps', },);
 ```
 */
function unwrapLinks({ text, }: { readonly text: string; },): string {
  return (function unwrap(): string {
    /**
     Pieces of the text read so far, each complete link reduced to its label, joined once at the end.
     */
    const output: string[] = [];

    /**
     Offset of the next character to read.
     */
    let index = 0;
    while (index < text.length) {
      if (text.charAt(index,) === '[') {
        /**
         Offset of the bracket that closes this label, or -1 when the label never closes.
         */
        const labelEnd = text.indexOf(
          ']',
          index + 1,
        );
        /**
         Offset just past the label's closing bracket, where a link target must open.
         */
        const afterLabel = labelEnd + 1;

        /**
         Whether a label closed after this link's opening bracket and a parenthesis opens the target.
         */
        const labelClosed = labelEnd > index;

        /**
         Whether a parenthesis opens the link target right after the label.
         */
        const targetOpens = text.charAt(afterLabel,) === '(';
        if (labelClosed && targetOpens) {
          /**
           Offset of the parenthesis that closes this target, or -1 when the target never closes.
           */
          const targetEnd = text.indexOf(
            ')',
            afterLabel + 1,
          );
          if (targetEnd > afterLabel) {
            output.push(text.slice(
              index + 1,
              labelEnd,
            ),);
            index = targetEnd + 1;
            continue;
          }
        }
      }
      output.push(text.charAt(index,),);
      index += 1;
    }
    return output.join('',);
  })();
}

/**
 Reads a paragraph as the page shows it: emphasis and code marks dropped, links unwrapped,
 bare URLs dropped, and whitespace collapsed.

 @param text - paragraph text as the parse holds it, newlines included

 @returns The paragraph words on one line

 @example
 ```ts
 const words = visibleWords({ text: '[Mittens](https://cats.example/m) *naps*.', },);
 ```
 */
export function visibleWords({ text, }: { readonly text: string; },): string {
  /**
   Paragraph with emphasis and code marks removed.
   */
  const unmarked = text
    .replaceAll(
    '*',
    '',
    )
    .replaceAll(
    '`',
    '',
    );
  /**
   Paragraph with each link reduced to its label.
   */
  const unlinked = unwrapLinks({ text: collapseSpaces({ text: unmarked, },), },);
  return collapseSpaces({
    text: unlinked
      .split(' ',)
      .filter(function isNotUrl(word,): boolean {
        return !(word.startsWith('http://') || word.startsWith('https://'));
      },)
      .join(' ',),
  },);
}

/**
 Blanks every character inside a quotation in double marks (curly, guillemets, low-nine or straight), marks included.

 @param text - visible words of one paragraph

 @returns The same text with the characters of quotations in double marks replaced by spaces

 @example
 ```ts
 const masked = maskDoubleQuotations({ text: 'The vet said, "She naps."', },);
 ```
 */
function maskDoubleQuotations({ text, }: { readonly text: string; },): string {
  return (function mask(): string {
    /**
     Whether the scan is inside a quotation, which any closing mark or a second straight quote ends.
     */
    let inQuotation = false;

    /**
     Pieces of the visible words, each quoted character replaced by a space, joined once at the end.
     */
    const masked: string[] = [];
    for (let index = 0; index < text.length; index += 1) {
      /**
       Code unit at this index; every quotation mark is one code unit, so offsets stay put.
       */
      const character = text.charAt(index,);
      if ('“«„'.includes(character,)) {
        inQuotation = true;
        masked.push(' ',);
        continue;
      }
      if ('”»'.includes(character,)) {
        inQuotation = false;
        masked.push(' ',);
        continue;
      }
      if (character === '"') {
        inQuotation = !inQuotation;
        masked.push(' ',);
        continue;
      }
      masked.push(inQuotation ? ' ' : character,);
    }
    return masked.join('',);
  })();
}

/**
 Blanks every character inside a quotation in single curly marks, marks included, where the marks pair. A closing single mark is the
 character of an apostrophe (`cat’s`, `cats’ bowls`), so no mark is taken for a closing one alone: the text is read with its curly
 single marks written straight by the package's pairing reader (`nestedSinglePairs`, `nested-single-quotes.ts`), which pairs a mark
 that opens after a space or an opening mark with a mark that closes after a non-space with no word after it, and leaves an apostrophe
 inside a word, an opening mark that no mark closes, and every mark of an apostrophe unpaired. Only a pair whose opening is `‘` and
 whose closing is `’` is blanked, the outermost where pairs nest.

 @param text - visible words of one paragraph, its double quotations already blanked

 @returns The same text with the characters of each paired quotation in single curly marks replaced by spaces

 @example
 ```ts
 const masked = maskSingleCurlyQuotations({ text: 'The vet said, ‘She naps.’ Then she left.', },); // the quotation is spaces
 ```
 */
function maskSingleCurlyQuotations({ text, }: { readonly text: string; },): string {
  if ((!text.includes('‘',)) || (!text.includes('’',)))
    return text;
  /**
   The text with each curly single mark written as the one unit it is, a straight one, the form the pairing reader reads.
   */
  const straight = text
    .replaceAll(
    '‘',
    '\'',
    )
    .replaceAll(
    '’',
    '\'',
    );
  /**
   Offsets of the paired marks.
   */
  const {
    openings,
    closings,
  } = nestedSinglePairs({
    text: straight,
    mask: Array.from(
      { length: text.length, },
      function prose(): boolean {
        return true;
      },
    ),
  },);
  /**
   Spans of the paired quotations whose opening mark is `‘` and whose closing mark is `’`, each from its opening mark to its closing
   mark, found in the order the marks stand. Where such pairs nest the spans nest, so blanking their offsets blanks the outermost.
   */
  const spans: SentenceSpan[] = [];
  /**
   Offsets of the paired opening marks still waiting for their closing mark, innermost last, mutated only inside this function.
   The pairing reader pairs a closing mark with the innermost opening mark that waits, so the mark that closes at an offset is
   the opening mark most recently pushed.
   */
  const waiting: number[] = [];
  for (let index = 0; index < text.length; index += 1) {
    if (openings.has(index,)) {
      waiting.push(index,);
      continue;
    }
    if (!closings.has(index,))
      continue;
    /**
     The opening mark of the pair this closing mark ends, none where no paired mark waits.
     */
    const opening = waiting.pop();
    if (opening === undefined)
      continue;
    /**
     Whether the pair opens with the opening curly mark and closes with the closing curly mark, the only pairs that are speech here.
     */
    const isCurlyPair = (text.charAt(opening,) === '‘') && (text.charAt(index,) === '’');
    if (isCurlyPair) {
      spans.push({
        start: opening,
        end: index + 1,
      },);
    }
  }
  /**
   Whether an offset lies inside a span, which the spans of one scan make a single pass over the text.
   */
  const blanked = new Set<number>(spans.flatMap(function offsetsOf({
    start,
    end,
  },): readonly number[] {
    return Array.from(
      { length: end - start, },
      function offsetAt(
        _unused,
        position,
      ): number {
        return start + position;
      },
    );
  },),);
  return text
    .split('',)
    .map(function masked(
      character,
      index,
    ): string {
      return blanked.has(index,) ? ' ' : character;
    },)
    .join('',);
}

/**
 Blanks every character inside a quotation, so the rules never read speech: double marks (curly, guillemets, low-nine, straight) and
 paired single curly marks, marks included. The result keeps the length of the input, so offsets into it match the visible text.

 @param text - visible words of one paragraph

 @returns The same text with quoted characters replaced by spaces

 @example
 ```ts
 const masked = maskQuotedSpeech({ text: 'The vet said, "She naps."', },);
 ```
 */
export function maskQuotedSpeech({ text, }: { readonly text: string; },): string {
  return maskSingleCurlyQuotations({ text: maskDoubleQuotations({ text, },), },);
}

/**
 Splits text into sentences at runs of sentence endings, keeping each run with its sentence.

 @param text - masked paragraph text

 @returns Offsets of each sentence, in order

 @example
 ```ts
 const spans = sentenceSpans({ text: 'Mittens naps. Did she?', },);
 ```
 */
export function sentenceSpans({ text, }: { readonly text: string; },): readonly SentenceSpan[] {
  return (function split(): readonly SentenceSpan[] {
    /**
     Sentences found so far, in order.
     */
    const spans: SentenceSpan[] = [];

    /**
     Offset where the sentence being read begins.
     */
    let start = 0;

    /**
     Offset of the next character to read.
     */
    let index = 0;
    while (index < text.length) {
      if (SENTENCE_ENDINGS.includes(text.charAt(index,),)) {
        /**
         Offset just past the run of sentence endings that begins at index.
         */
        let end = index;
        while ((end < text.length) && SENTENCE_ENDINGS.includes(text.charAt(end,),)) {
          end += 1;
        }
        spans.push({
          start,
          end,
        },);
        start = end;
        index = end;
      } else {
        index += 1;
      }
    }
    if (start < text.length) {
      spans.push({
        start,
        end: text.length,
      },);
    }
    return spans;
  })();
}
//endregion Text reading
