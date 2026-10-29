/**
 Tests for color-mode decision and the JSON token highlighter.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  colorizeJson,
  shouldColor,
} from './color.ts';

/**
 ESC byte that begins an ANSI escape sequence.
 */
const ESC = String.fromCodePoint(27);

/**
 Removes ANSI escape sequences from a string via a single linear scan.
 
 @param text - possibly-colored text
 
 @returns the text with `ESC ... m` sequences removed
 */
function stripAnsi(text: string): string {
  const cursor = { i: 0, };
  const out: string[] = [];
  while (cursor.i < text.length) {
    if (text[cursor.i] === ESC) {
      while ((cursor.i < text.length) && (text[cursor.i] !== 'm'))
        cursor.i += 1;
      cursor.i += 1;
      continue;
    }
    out.push(text[cursor.i] ?? '');
    cursor.i += 1;
  }
  return out.join('');
}

await describe({
  name: shouldColor.name,
  children: [
    it({
      name: 'honors explicit always/never regardless of stream',
      fn: async ({ expect, }) => {
        expect(shouldColor({ mode: 'always', stream: {}, })).toBe(true);
        expect(shouldColor({ mode: 'never', stream: { isTTY: true, }, })).toBe(false);
      },
    }),

    it({
      name: 'auto follows the stream TTY status when no env override is set',
      fn: async ({ expect, }) => {
        const savedNo = process.env.NO_COLOR;
        const savedForce = process.env.FORCE_COLOR;
        delete process.env.NO_COLOR;
        delete process.env.FORCE_COLOR;
        expect(shouldColor({ mode: 'auto', stream: { isTTY: true, }, })).toBe(true);
        expect(shouldColor({ mode: 'auto', stream: { isTTY: false, }, })).toBe(false);
        if (savedNo !== undefined)
          process.env.NO_COLOR = savedNo;
        if (savedForce !== undefined)
          process.env.FORCE_COLOR = savedForce;
      },
    }),

    it({
      name: 'auto disables under NO_COLOR even on a TTY',
      fn: async ({ expect, }) => {
        const savedNo = process.env.NO_COLOR;
        process.env.NO_COLOR = '1';
        expect(shouldColor({ mode: 'auto', stream: { isTTY: true, }, })).toBe(false);
        if (savedNo === undefined)
          delete process.env.NO_COLOR;
        else
          process.env.NO_COLOR = savedNo;
      },
    }),
  ],
});

await describe({
  name: colorizeJson.name,
  children: [
    it({
      name: 'round-trips to the same JSON after stripping ANSI',
      fn: async ({ expect, }) => {
        const json = JSON.stringify({ a: 1, b: 'x y', c: true, d: null, e: [1, 2,], });
        const colored = colorizeJson({ json, });
        expect(colored.includes(ESC)).toBe(true);
        expect(stripAnsi(colored)).toBe(json);
      },
    }),
  ],
});
