import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  baseIndentAt,
  leadingWhitespace,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region leadingWhitespace

    describe({
      name: leadingWhitespace.name,
      children: [
        it({
          name: 'returns empty string for empty input',
          fn: async () => {
            expect(leadingWhitespace('',),).toBe('',);
          },
        },),
        it({
          name: 'returns empty string when the first character is non-whitespace',
          fn: async () => {
            expect(leadingWhitespace('bar',),).toBe('',);
          },
        },),
        it({
          name: 'returns the leading run before content',
          fn: async () => {
            expect(leadingWhitespace('  foo',),).toBe('  ',);
          },
        },),
        it({
          name: 'returns the whole string when it is all whitespace',
          fn: async () => {
            expect(leadingWhitespace('   ',),).toBe('   ',);
          },
        },),
        it({
          name: 'collects every ASCII whitespace kind up to the first content char',
          fn: async () => {
            expect(leadingWhitespace('\t \n\r\f\vx',),).toBe('\t \n\r\f\v',);
          },
        },),
        it({
          name: 'stops at the first non-whitespace even when trailing whitespace follows',
          fn: async () => {
            expect(leadingWhitespace('  a  ',),).toBe('  ',);
          },
        },),
        it({
          name: 'treats a leading surrogate-pair character as non-whitespace',
          fn: async () => {
            expect(leadingWhitespace('\u{1F600}foo',),).toBe('',);
          },
        },),
        it({
          name: 'handles a long whitespace run in a single linear pass',
          fn: async () => {
            const runLength = 50_000;
            const input = `${' '.repeat(runLength,)}x`;
            expect(leadingWhitespace(input,),).toBe(' '.repeat(runLength,),);
          },
        },),
      ],
    },),

    //endregion leadingWhitespace

    //region baseIndentAt

    describe({
      name: baseIndentAt.name,
      children: [
        it({
          name: 'returns indentation of a single-line offset with no preceding newline',
          fn: async () => {
            expect(
              baseIndentAt({
                sourceText: '  foo(a, b);',
                offset: 6,
              },),
            ).toBe('  ',);
          },
        },),
        it({
          name: 'returns indentation of the line containing the offset',
          fn: async () => {
            // 'x\n    y': offset 6 sits on 'y'; its line is '    y'.
            expect(
              baseIndentAt({
                sourceText: 'x\n    y',
                offset: 6,
              },),
            ).toBe('    ',);
          },
        },),
        it({
          name: 'returns empty string when the offset line has no indentation',
          fn: async () => {
            expect(
              baseIndentAt({
                sourceText: 'foo\nbar',
                offset: 5,
              },),
            ).toBe('',);
          },
        },),
      ],
    },),

    //endregion baseIndentAt
  ],
},);
