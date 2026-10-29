/**
 Tests for keeping interpolated text inside a line-based sheet's items
 (ledger L14(d)): which characters end a line, a carriage return before a
 line feed counting once, and the checker's JSON quotes holding no line
 separator `JSON.stringify` leaves raw.

 Fixtures are cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import { indentContinuation, } from '@monochromatic-dev/translation-repair-house-rule';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { buildResolutionMessages, } from '../dist/final/node/index.mjs';

/**
 Every character a reader may take as ending a line, spelled as escapes so
 none sits raw in this file.
 */
const BREAKS: readonly string[] = [
  '\n',
  '\v',
  '\f',
  '\r',
  '\u0085',
  '\u2028',
  '\u2029',
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: indentContinuation.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        ...BREAKS.map(function toCase(lineBreak,) {
          return it({
            name: `ENDS a line at U+${lineBreak.codePointAt(0,)?.toString(16,).padStart(4, '0',) ?? ''} and indents the next`,
            fn: async () => {
              expect(indentContinuation({ text: `hunts${lineBreak}at noon`, indent: '  ', },),).toBe('hunts\n  at noon',);
            },
          },);
        },),
        it({
          name: 'COUNTS a carriage return before a line feed as one line end',
          fn: async () => {
            expect(indentContinuation({ text: 'hunts\r\nat noon', indent: '  ', },),).toBe('hunts\n  at noon',);
          },
        },),
        it({
          name: 'LEAVES one-line text as it was',
          fn: async () => {
            expect(indentContinuation({ text: 'Mittens  naps.', indent: '  ', },),).toBe('Mittens  naps.',);
          },
        },),
      ],
    },),

    describe({
      name: 'the checker\'s JSON quotes',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ESCAPE the separators JSON.stringify leaves raw, so no quote holds a line end',
          fn: async () => {
            /**
             Quote carrying next line, line separator and paragraph separator.
             */
            const quote = 'hunts\u0085at\u2028noon\u2029today';
            /**
             Every message of one checker sheet, joined.
             */
            const sheet = buildResolutionMessages({
              sourceText: '咪咪在中午打盹。',
              patchedText: 'Mittens naps at noon.',
              issues: [
                {
                  issueId: 'issue/nap',
                  status: 'accepted',
                  severity: 'major',
                  claims: [
                    {
                      claimId: 'claim/nap',
                      claim: {
                        category: 'accuracy/mistranslation',
                        severity: 'major',
                        summary: 'Napping is rendered as hunting.',
                        spans: [
                          {
                            side: 'target',
                            nodeId: 'block/1',
                            nodeHash: hashContent({ content: quote, },),
                            startOffset: 0,
                            endOffset: quote.length,
                            quotedText: quote,
                          },
                        ],
                      },
                    },
                  ],
                  tallies: {},
                },
              ],
            },).messages
              .map(function textOf(message,) {
                return messageText({ message, },);
              },)
              .join('\n',);
            expect({
              raw: BREAKS.slice(-3,).some(function isRaw(lineBreak,) {
                return sheet.includes(lineBreak,);
              },),
              escaped: sheet.includes(String.raw`"hunts\u0085at\u2028noon\u2029today"`,),
            },).toEqual({
              raw: false,
              escaped: true,
            },);
          },
        },),
      ],
    },),
  ],
},);
