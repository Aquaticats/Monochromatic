/**
 Removal of ANSI color sequences from the app's log lines.
 
 @example
 ```ts
 import { stripAnsi, } from './ansi.ts';
 
 stripAnsi('\u001B[32mINFO\u001B[0m ready',); // 'INFO ready'
 ```
 
 @packageDocumentation
 */

/**
 The escape character that starts every ANSI sequence.
 */
const ESCAPE = '\u001B';

/**
 Characters allowed between the bracket and the final letter of a color sequence.
 */
const PARAMETER_CHARACTERS: ReadonlySet<string> = new Set('0123456789;',);

/**
 Final letter of a color (select graphic rendition) sequence.
 */
const SEQUENCE_FINAL = 'm';

/**
 Counts the characters a color sequence occupies after its escape character.
 
 @param segment - text that follows one escape character
 
 @returns the sequence length, or 0 when the text does not start a color sequence
 
 @example
 ```ts
 sequenceLength('[32mINFO',); // 4
 ```
 */
function sequenceLength(segment: string,): number {
  if (!segment.startsWith('[',))
    return 0;
  for (let index = 1; index < segment.length; index++) {
    /**
     The character at the scan position.
     */
    const character = segment.charAt(index,);
    if (!PARAMETER_CHARACTERS.has(character,))
      // The bracket, the parameters and the final letter make up the sequence.
      return character === SEQUENCE_FINAL ? index + 1 : 0;
  }
  return 0;
}

/**
 Removes the color sequences of a text in one linear pass: split at each escape character, drop the sequence at the
 start of each following segment.
 
 @param text - log text that may hold color sequences
 
 @returns the text without color sequences; other escape characters stay
 
 @example
 ```ts
 stripAnsi('\u001B[1mbold\u001B[0m',); // 'bold'
 ```
 */
export function stripAnsi(text: string,): string {
  /**
   The text before the first escape character, then one segment per escape character.
   */
  const [
    first = '',
    ...segments
  ] = text.split(ESCAPE,);

  /**
   Segments with the leading color sequence removed.
   */
  const kept = segments.map(function dropSequence(segment,): string {
    /**
     Characters the color sequence at the segment start occupies.
     */
    const length = sequenceLength(segment,);
    return length === 0 ? ESCAPE + segment : segment.slice(length,);
  },);
  return first + kept.join('',);
}
