/**
 Tests for the source-spelling expansion reader.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { shellWordCarriesExpansion, } from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Fixtures

/**
 Word spellings whose value the shell computes, with the expansion each carries.
 */
const EXPANDING_WORDS: readonly { readonly source: string; readonly why: string; }[] = [
  { source: '"$d"', why: 'a parameter inside double quotes', },
  { source: '$d', why: 'a bare parameter', },
  { source: `"\${d}"`, why: 'a braced parameter', },
  { source: `"\${d:-fallback}"`, why: 'a parameter with a default', },
  { source: '"$1"', why: 'a positional parameter', },
  { source: '"$@"', why: 'the argument list', },
  { source: '"$*"', why: 'the argument list joined', },
  { source: '"$?"', why: 'the last status', },
  { source: '"$$"', why: 'the shell process id', },
  { source: '"$(pwd)"', why: 'a command substitution', },
  { source: '"$(dirname "$0")/build"', why: 'a nested command substitution', },
  { source: '$((1 + 1))', why: 'an arithmetic expansion', },
  { source: '`pwd`/x', why: 'a backtick substitution', },
  { source: '"`pwd`/x"', why: 'a backtick substitution inside double quotes', },
  { source: '"it\'s $d"', why: 'a single quote inside double quotes, where a parameter still expands', },
  { source: String.raw`$'\x2d'$d`, why: 'an ANSI-C quoted region followed by a parameter', },
];

/**
 Word spellings whose value is literal text, whatever characters they hold.
 */
const LITERAL_WORDS: readonly { readonly source: string; readonly why: string; }[] = [
  { source: `'$d'`, why: 'a single-quoted dollar', },
  { source: `'$d/$n'`, why: 'a single-quoted path fragment', },
  { source: `'$(pwd)'`, why: 'a single-quoted substitution', },
  { source: '/home/user/temp/agent/scratch', why: 'an absolute path', },
  { source: '*.log', why: 'a glob', },
  { source: '~/temp/scratch', why: 'a home-relative path', },
  { source: '\'$d\'', why: 'a quoted literal amid single quotes', },
  { source: String.raw`$'\x2d'`, why: 'an ANSI-C quoted escape', },
  { source: String.raw`\$d`, why: 'an escaped dollar sign', },
  { source: 'plain', why: 'plain text', },
];

//endregion Fixtures

await describe({
  name: shellWordCarriesExpansion.name,
  children: [
    //region Expanding words
    describe({
      name: 'a word the shell computes',
      children: EXPANDING_WORDS.map(function expandingWord(fixture,) {
        return it({
          name: `reads ${fixture.why} in ${fixture.source}`,
          fn: async function testExpandingWord() {
            expect(shellWordCarriesExpansion(fixture.source,),).toBe(true,);
          },
        },);
      },),
    },),
    //endregion Expanding words

    //region Literal words
    describe({
      name: 'a word the shell copies',
      children: LITERAL_WORDS.map(function literalWord(fixture,) {
        return it({
          name: `reads ${fixture.why} in ${fixture.source}`,
          fn: async function testLiteralWord() {
            expect(shellWordCarriesExpansion(fixture.source,),).toBe(false,);
          },
        },);
      },),
    },),
    //endregion Literal words

    //region Order
    describe({
      name: 'a word whose regions follow one another',
      children: [
        it({
          name: 'reads a parameter after a literal region',
          fn: async function testParameterAfterLiteral() {
            expect(shellWordCarriesExpansion(`'a'$d`,),).toBe(true,);
          },
        },),
        it({
          name: 'reads a parameter between two literal regions',
          fn: async function testParameterBetweenLiterals() {
            expect(shellWordCarriesExpansion(`'a'$d'b'`,),).toBe(true,);
          },
        },),
        it({
          name: 'reads an unterminated literal region as literal text',
          fn: async function testUnterminatedLiteral() {
            expect(shellWordCarriesExpansion('\'$d',),).toBe(false,);
          },
        },),
      ],
    },),
    //endregion Order
  ],
},);
