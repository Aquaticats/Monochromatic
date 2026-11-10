// Removal of the Select Graphic Rendition sequences the IDE's logger writes into its own log, so a reader can
// count the records of a log line that carries colours.
// What: one pass over the text that drops every ESC [ ... m sequence whose parameter is a run of digits and
// semicolons, and keeps everything else, including escapes that only look like one.
// Why: an inspection script asked for the same answer through a pattern, which the repository lint bans where a
// scan reads the same text in one pass.
// Tested in sgr-sequences.test.ts.

import { isAsciiDigit } from './scan-characters.ts';

/** Escape character that opens a Select Graphic Rendition sequence. */
const escape = '\u001b';

/** Introducer every Select Graphic Rendition sequence holds after its escape. */
const introducer = '[';

/**
 Text a terminal renders as written, without its colour sequences.
 
 @param text - Log text, possibly holding escape sequences.
 
 @returns The same text with every complete Select Graphic Rendition sequence removed.
 
 @example
 ```ts
 withoutSgrSequences('\u001b[31mred\u001b[0m'); // 'red'
 ```
 */
export function withoutSgrSequences(text: string): string {
  let kept = 0;
  let index = 0;
  let plain = '';
  while (index < text.length) {
    // Only an escape followed by the introducer can open a sequence; anything else is text to keep.
    if (text[index] !== escape || text[index + 1] !== introducer) {
      index++;
      continue;
    }
    let end = index + introducer.length + 1;
    while (end < text.length && (isAsciiDigit(text[end] ?? '') || text[end] === ';')) end++;
    // A run that ends in anything but the final m is not a sequence, so the escape stays as written.
    if (text[end] !== 'm') {
      index++;
      continue;
    }
    plain += text.slice(kept, index);
    index = end + 1;
    kept = index;
  }
  return plain + text.slice(kept);
}
