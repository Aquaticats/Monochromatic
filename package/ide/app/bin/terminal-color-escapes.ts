// Terminal color escapes, removed from captured output before a bundle check reads it.
// What: the select-graphic-rendition sequences a program writes to color its terminal output, ESC followed by `[`, the
// digits and semicolons of the parameters, and `m`, dropped in one pass while every other character is kept.
// Why: the checks read the compositor's and the application's combined output as lines, and a color sequence inside a
// line would sit between the phrases they look for and the reader's eye. Only the color form is dropped: the other
// escape forms this output can carry, such as erasing a line, are the program's own text and stay.
// The function is one linear pass and is tested in terminal-color-escapes.test.ts.

/**
 The character every escape sequence opens with.
 */
const escape = '\u001b';

/**
 The characters a color sequence's parameters are written with.
 */
const parameterCharacters = '0123456789;';

/**
 The character that ends a color sequence.
 */
const colorTerminator = 'm';

/**
 The end of the color escape that starts at `at`, or this value when no escape starts there.
 */
const noColorEscape = -1;

/**
 Where the color escape starting at `at` ends, exclusive, or {@link noColorEscape} when none starts there.
 *
 A sequence is complete only when its terminator follows its parameters, so an escape cut short at the end of the
 output is text the program wrote rather than a color it asked for.
 *
 @param options - `text` is the output being read; `at` the character to read an escape at.
 *
 @returns The index just past the sequence, or {@link noColorEscape}.
 */
const colorEscapeEnd = ({ text, at }: { text: string; at: number }): number => {
  if (text[at] !== escape || text[at + 1] !== '[') {
    return noColorEscape;
  }
  let end = at + 2;
  while (end < text.length && parameterCharacters.includes(text[end] ?? '')) {
    end++;
  }
  return text[end] === colorTerminator ? end + 1 : noColorEscape;
};

/**
 `text` without its terminal color escapes.
 *
 @param text - the captured output to read.
 *
 @returns The output with every color sequence dropped.
 */
export const withoutTerminalColors = (text: string): string => {
  const kept: string[] = [];
  // `copied` is the first character of the current run of text that stays, so each kept run is one slice.
  let copied = 0;
  let at = 0;
  while (at < text.length) {
    const end = colorEscapeEnd({ text, at });
    if (end !== noColorEscape) {
      kept.push(text.slice(copied, at));
      copied = end;
      at = end;
      continue;
    }
    at++;
  }
  kept.push(text.slice(copied));
  return kept.join('');
};
