/**
 Strip functions that remove non-prose regions from assistant message
 text before the uncertainty engine scans it. Delegates the actual
 span-removal work to {@link stripBetweenDelims} and
 {@link stripLinesStartingWith} from the shared text-scan module so this
 file stays regex-free.
 
 @module
 */

import {
  isWordChar,
  stripBetweenDelims,
  stripLinesStartingWith,
} from '@monochromatic-dev/agent-harness-shared-text-scan/ts';
import { RIGHT_SINGLE_QUOTE, } from './uncertainty-phrases.ts';

/**
 Strips fenced code blocks (triple-backtick) so uncertainty scans don't
 match words inside code. Mirrors the legacy regex pass that replaced
 triple-backtick fenced blocks with empty strings.
 
 @param text - raw message text that may contain fenced code blocks
 
 @returns text with all fenced code blocks removed
 
 @example
 ```ts
 stripCodeBlocks('text ```js\nmaybe();\n``` more text')
 // => 'text  more text'
 ```
 */
function stripCodeBlocks(text: string,): string {
  return stripBetweenDelims({
    text,
    openDelim: '```',
    closeDelim: '```',
  },);
}

/**
 Strips inline code spans (single-backtick) so uncertainty scans don't
 match words inside code. Mirrors the legacy regex pass that replaced
 backtick-bounded spans with empty strings.
 
 @param text - text that may contain inline code spans
 
 @returns text with all inline code spans removed
 
 @example
 ```ts
 stripInlineCode('use `maybe` function')
 // => 'use  function'
 ```
 */
function stripInlineCode(text: string,): string {
  return stripBetweenDelims({
    text,
    openDelim: '`',
    closeDelim: '`',
  },);
}

/**
 Strips markdown blockquote lines so uncertainty scans don't match words
 Claude is quoting from source material. Mirrors the legacy regex pass
 that removed every line beginning with the blockquote marker.
 
 @param text - text that may contain markdown blockquotes
 
 @returns text with blockquote lines removed
 
 @example
 ```ts
 stripBlockquotes('normal line\n> probably a quote\nanother line')
 // => 'normal line\nanother line'
 ```
 */
function stripBlockquotes(text: string,): string {
  return stripLinesStartingWith({
    text,
    prefix: '>',
  },);
}

/**
 Reports whether the character at an index is an apostrophe between two word characters.

 `charAt` answers an empty string past either end, which is not a word character, so an
 apostrophe at the start or end of the text is never a contraction.

 @param text - text being scanned

 @param index - position of the character to test

 @returns whether it is an in-word apostrophe

 @example
 ```ts
 isContractionApostrophe({ text: "it's", index: 2 }); // true
 isContractionApostrophe({ text: "'it", index: 0 }); // false
 ```
 */
function isContractionApostrophe(
  {
    text,
    index,
  }: {
    readonly text: string;
    readonly index: number;
  },
): boolean {
  if (text.charAt(index,) !== "'") {
    return false;
  }
  if (!isWordChar(text.charAt(index - 1,),)) {
    return false;
  }
  return isWordChar(text.charAt(index + 1,),);
}

/**
 Replaces apostrophes inside a word with a curly apostrophe so they cannot open a quote.

 Without this, `the workflow's comments say I didn't check` reads as a quoted string from
 `'s` to `'t`, and everything between the two contractions, hedges included, is removed
 before the scan. Only an apostrophe with a word character on both sides is folded, so a
 quote mark after a space or punctuation still opens a quoted string.

 @param text - text that may contain contractions or possessives

 @returns text with in-word apostrophes replaced by `U+2019`

 @example
 ```ts
 foldContractionApostrophes("it's 'maybe'"); // => 'it’s \'maybe\''
 ```
 */
function foldContractionApostrophes(text: string,): string {
  return Array.from(
    { length: text.length, },
    function foldAt(
      _slot,
      index,
    ) {
      return isContractionApostrophe({
        text,
        index,
      },)
        ? RIGHT_SINGLE_QUOTE
        : text.charAt(index,);
    },
  )
    .join('',);
}

/**
 Strips inline quoted strings (double and single quotes) so uncertainty
 scans don't match words Claude is quoting verbatim. Mirrors the legacy
 regex pipeline that stripped one quote shape after the other; here it
 chains {@link stripBetweenDelims} with `disallowedInside: '\n'` to keep
 the strip bounded to a single line.
 
 @param text - text that may contain inline quoted strings
 
 @returns text with inline quoted strings removed
 
 @example
 ```ts
 stripQuotedStrings('the word "maybe" appears here')
 // => 'the word  appears here'
 ```
 */
function stripQuotedStrings(text: string,): string {
  return stripBetweenDelims({
    text: stripBetweenDelims({
      text: foldContractionApostrophes(text,),
      openDelim: '"',
      closeDelim: '"',
      disallowedInside: '\n',
    },),
    openDelim: "'",
    closeDelim: "'",
    disallowedInside: '\n',
  },);
}

/**
 Prepares message text for uncertainty scanning by removing regions
 where uncertain language is expected or acceptable (code blocks, inline
 code, blockquotes, quoted strings).
 
 Stripping order matters: fenced code blocks first (largest spans),
 then inline code, then blockquotes, then quoted strings.
 
 @param text - raw assistant message text
 
 @returns cleaned text ready for pattern matching
 
 @example
 ```ts
 const prose = stripNonProseRegions('Look at ```js\nmaybe()\n``` and "perhaps"');
 ```
 */
function stripNonProseRegions(text: string,): string {
  return stripQuotedStrings(
    stripBlockquotes(
      stripInlineCode(stripCodeBlocks(text,),),
    ),
  );
}

export {
  stripBlockquotes,
  stripCodeBlocks,
  stripInlineCode,
  stripNonProseRegions,
  stripQuotedStrings,
};
