import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { formatModelDisplay, } from '../dist/final/node/statusline.mjs';

//region Case tables

/**
 One formatter case: display name in, concise display name out.
 */
type DisplayCase = {
  /**
   What the case proves about the formatter.
   */
  readonly name: string;
  /**
   Display name as Claude Code sends it.
   */
  readonly raw: string;
  /**
   Concise display name the statusline shows.
   */
  readonly expected: string;
};

/**
 Cases where the grammar parses and a piece is kept or stripped.
 */
const STRIPPED_CASES: readonly DisplayCase[] = [
  {
    name: 'strips the latest version and the default context of a known family',
    raw: 'Opus 4.8 (1M context)',
    expected: 'Opus',
  },
  {
    name: 'strips both pieces for Sonnet at its defaults',
    raw: 'Sonnet 4.6 (200K context)',
    expected: 'Sonnet',
  },
  {
    name: 'strips both pieces for Haiku at its defaults',
    raw: 'Haiku 4.5 (200K context)',
    expected: 'Haiku',
  },
  {
    name: 'strips a default context that arrives without a version piece',
    raw: 'Opus (1M context)',
    expected: 'Opus',
  },
  {
    name: 'strips a default version that arrives without a context piece',
    raw: 'Sonnet 4.6',
    expected: 'Sonnet',
  },
  {
    name: 'keeps an older version and strips the default context',
    raw: 'Opus 4.5 (1M context)',
    expected: 'Opus 4.5',
  },
  {
    name: 'keeps an older version with no context piece',
    raw: 'Opus 4.5',
    expected: 'Opus 4.5',
  },
  {
    name: 'strips the default version and keeps a non-default context',
    raw: 'Sonnet 4.6 (1M context)',
    expected: 'Sonnet (1M)',
  },
  {
    name: 'keeps a non-default context that arrives without a version piece',
    raw: 'Sonnet (1M context)',
    expected: 'Sonnet (1M)',
  },
  {
    name: 'keeps a family name that carries no pieces',
    raw: 'Opus',
    expected: 'Opus',
  },
  {
    name: 'keeps every piece of a family with no recorded defaults',
    raw: 'GPT 5.1 (300K context)',
    expected: 'GPT 5.1 (300K)',
  },
  {
    name: 'matches families case-sensitively, so a lowercase name keeps its defaults',
    raw: 'opus 4.8 (1M context)',
    expected: 'opus 4.8 (1M)',
  },
  {
    name: 'keeps the version of a family name inherited from Object.prototype',
    raw: 'constructor 1.0',
    expected: 'constructor 1.0',
  },
  {
    name: 'keeps a zero-width space inside a context piece, since it is not whitespace',
    raw: 'Sonnet 4.6 (1M\u200BX context)',
    expected: 'Sonnet (1M\u200BX)',
  },
  {
    name: 'keeps a next-line character inside a context piece, since a regex does not treat it as whitespace',
    raw: 'Sonnet 4.6 (1M\u0085X context)',
    expected: 'Sonnet (1M\u0085X)',
  },
  {
    name: 'keeps an astral character as a whole context piece',
    raw: 'Sonnet 4.6 (\u{1F600} context)',
    expected: 'Sonnet (\u{1F600})',
  },
  {
    name: 'keeps an unpaired surrogate as a whole context piece',
    raw: 'Sonnet 4.6 (\uD800 context)',
    expected: 'Sonnet (\uD800)',
  },
  {
    name: 'keeps punctuation inside a context piece',
    raw: 'Sonnet 4.6 (1M/2 context)',
    expected: 'Sonnet (1M/2)',
  },
] as const;

/**
 Names that do not fit the grammar, which the formatter must hand back untouched.
 */
const UNPARSED_CASES: readonly DisplayCase[] = [
  {
    name: 'returns an empty name untouched',
    raw: '',
    expected: '',
  },
  {
    name: 'returns a family fused to a digit untouched',
    raw: 'Opus4',
    expected: 'Opus4',
  },
  {
    name: 'returns a name with two separators before the version untouched',
    raw: 'Opus  4.8',
    expected: 'Opus  4.8',
  },
  {
    name: 'returns a name separated by a tab untouched',
    raw: 'Opus\t4.8',
    expected: 'Opus\t4.8',
  },
  {
    name: 'returns a version without a minor number untouched',
    raw: 'Opus 4. (1M context)',
    expected: 'Opus 4. (1M context)',
  },
  {
    name: 'returns a version without a major number untouched',
    raw: 'Opus .8 (1M context)',
    expected: 'Opus .8 (1M context)',
  },
  {
    name: 'returns a version with a third number untouched',
    raw: 'Opus 4.8.9',
    expected: 'Opus 4.8.9',
  },
  {
    name: 'returns a name with text after the closed context piece untouched',
    raw: 'Opus 4.8 (1M context)x',
    expected: 'Opus 4.8 (1M context)x',
  },
  {
    name: 'returns a context piece holding an en quad space untouched',
    raw: 'Sonnet 4.6 (1M\u2001X context)',
    expected: 'Sonnet 4.6 (1M\u2001X context)',
  },
  {
    name: 'returns a name whose context piece follows a no-break space untouched',
    raw: 'Sonnet 4.6\u00A0(200K context)',
    expected: 'Sonnet 4.6\u00A0(200K context)',
  },
  {
    name: 'returns a name whose version follows a no-break space untouched',
    raw: 'Sonnet\u00A04.6',
    expected: 'Sonnet\u00A04.6',
  },
  {
    name: 'returns a name whose context piece follows a line separator untouched',
    raw: 'Sonnet 4.6\u2028(200K context)',
    expected: 'Sonnet 4.6\u2028(200K context)',
  },
  {
    name: 'returns a name with a trailing carriage return and newline untouched',
    raw: 'Opus 4.8 (1M context)\r\n',
    expected: 'Opus 4.8 (1M context)\r\n',
  },
  {
    name: 'returns a name with a trailing carriage return untouched',
    raw: 'Sonnet\r',
    expected: 'Sonnet\r',
  },
  {
    name: 'returns an unclosed context piece untouched',
    raw: 'Opus (1M context',
    expected: 'Opus (1M context',
  },
  {
    name: 'returns a misspelled context closing untouched',
    raw: 'Opus (1M contexts)',
    expected: 'Opus (1M contexts)',
  },
  {
    name: 'returns an empty context piece untouched',
    raw: 'Opus 4.8 ( context)',
    expected: 'Opus 4.8 ( context)',
  },
  {
    name: 'returns a name with a trailing space untouched',
    raw: 'Opus 4.8 (1M context) ',
    expected: 'Opus 4.8 (1M context) ',
  },
  {
    name: 'returns a name with a trailing newline untouched',
    raw: 'Opus\n',
    expected: 'Opus\n',
  },
  {
    name: 'returns a name whose pieces are out of order untouched',
    raw: '4.8 Opus',
    expected: '4.8 Opus',
  },
  {
    name: 'returns a name that starts with a digit untouched',
    raw: '4 Opus',
    expected: '4 Opus',
  },
] as const;

//endregion Case tables

//region Suites

await describe({
  name: '',
  children: [
    describe({
      name: formatModelDisplay.name,
      children: [
        ...STRIPPED_CASES.map(function mapStrippedCase({
          name,
          raw,
          expected,
        },) {
          return it({
            name,
            fn: async function testStrippedCase(): Promise<void> {
              expect(formatModelDisplay(raw,),).toBe(expected,);
            },
          },);
        },),
        ...UNPARSED_CASES.map(function mapUnparsedCase({
          name,
          raw,
          expected,
        },) {
          return it({
            name,
            fn: async function testUnparsedCase(): Promise<void> {
              expect(formatModelDisplay(raw,),).toBe(expected,);
            },
          },);
        },),
      ],
    },),
  ],
},);

//endregion Suites
