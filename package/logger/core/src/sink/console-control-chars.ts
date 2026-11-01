/**
 Console-bound text crosses a syntax boundary: a terminal interprets C0 and
 C1 control characters as commands (clear screen, set title, move cursor,
 write clipboard). Log messages can carry attacker-influenced text, so the
 console sink neutralizes every control character except newline and tab
 before the text reaches `console.*` or `process.stderr`.
 
 @module
 */

/**
 First code unit above the C0 control range; everything below it except
 newline and tab is neutralized.
 */
const C0_CONTROL_LIMIT = 0x20;

/**
 Newline stays literal: multi-line messages (stack traces) are a core use.
 */
const NEWLINE_CODE_UNIT = 0x0A;

/**
 Tab stays literal: indentation in multi-line messages is harmless.
 */
const TAB_CODE_UNIT = 0x09;

/**
 DEL sits alone above the printable ASCII range and is a control character.
 */
const DELETE_CODE_UNIT = 0x7F;

/**
 First code unit of the C1 control range (8-bit CSI, OSC, and friends).
 */
const C1_CONTROL_START = 0x80;

/**
 Last code unit of the C1 control range.
 */
const C1_CONTROL_END = 0x9F;

/**
 Radix for the hexadecimal digits inside a `\uXXXX` escape.
 */
const HEX_RADIX = 16;

/**
 Digit count of a `\uXXXX` escape, zero-padded on the left.
 */
const UNICODE_ESCAPE_WIDTH = 4;

/**
 Reports whether one UTF-16 code unit is a control character the console
 sink must neutralize.
 
 @param codeUnit - UTF-16 code unit read from the message.
 
 @returns Whether the code unit is a C0 control other than newline and tab,
 DEL, or a C1 control.
 
 @example
 ```ts
 isNeutralizedControl(0x1B); // true (ESC)
 isNeutralizedControl(0x0A); // false (newline stays)
 ```
 */
function isNeutralizedControl(codeUnit: number,): boolean {
  if (codeUnit < C0_CONTROL_LIMIT)
    return (codeUnit !== NEWLINE_CODE_UNIT) && (codeUnit !== TAB_CODE_UNIT);

  if (codeUnit === DELETE_CODE_UNIT)
    return true;

  return (codeUnit >= C1_CONTROL_START) && (codeUnit <= C1_CONTROL_END);
}

/**
 Renders one code unit as a `\uXXXX` escape with uppercase hex digits (the
 repository's escape-case convention) so the attempted control stays
 visible for forensics instead of vanishing.
 
 @param codeUnit - UTF-16 code unit to escape.
 
 @returns Six-character escape such as `\u001B`.
 
 @example
 ```ts
 escapeCodeUnit(0x1B); // '\\u001B'
 ```
 */
function escapeCodeUnit(codeUnit: number,): string {
  return `\\u${
    codeUnit
      .toString(HEX_RADIX,)
      .toUpperCase()
      .padStart(
        UNICODE_ESCAPE_WIDTH,
        '0',
      )
  }`;
}

/**
 Neutralizes terminal control characters in console-bound text. One linear
 pass over the code points: each neutralized control becomes a `\uXXXX`
 escape, everything else is copied through, and newline and tab pass
 untouched. Well-formed and malformed escape sequences get no
 special treatment because the introducer byte itself is neutralized, so a
 trailing lone ESC, an unterminated OSC, and a nested ESC all lose their
 teeth the same way.
 
 @param text - Message text destined for `console.*` or `process.stderr`.
 
 @returns Text with every neutralized control rendered as `\uXXXX`.
 
 @example
 ```ts
 neutralizeControlCharacters('title:\u001B]0;x\u0007 ok\n\tnext');
 // => 'title:\\u001B]0;x\\u0007 ok\n\tnext'
 ```
 */
export function neutralizeControlCharacters(text: string,): string {
  /**
   Output pieces in input order: each code point either verbatim or as its
   escape, joined once at the end so no per-character string rebuild occurs.
   */
  const pieces: string[] = [];
  for (const character of text) {
    /**
     Leading code unit of this iteration element. String iteration walks
     code points, so a surrogate pair arrives as one two-unit string whose
     lead surrogate is never a control, and a lone surrogate passes the
     same way.
     */
    // oxlint-disable-next-line unicorn/prefer-code-point -- Classifier reads the lead code unit on purpose; controls below U+00A0 never sit inside a surrogate pair, so code-point decoding adds nothing.
    const codeUnit = character.charCodeAt(0,);
    pieces.push(isNeutralizedControl(codeUnit,) ? escapeCodeUnit(codeUnit,) : character,);
  }
  return pieces.join('',);
}
