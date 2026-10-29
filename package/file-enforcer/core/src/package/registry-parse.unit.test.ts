import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  firstWhitespaceToken,
} from '../../dist/final/node/index.mjs';

await describe({
  name: firstWhitespaceToken.name,
  children: [
    it({
      name: 'returns empty token for an empty string',
      fn: async function emptyString() {
        expect(firstWhitespaceToken('',),).toBe('',);
      },
    },),
    it({
      name: 'returns empty token for an all-whitespace line',
      fn: async function allWhitespace() {
        expect(firstWhitespaceToken('   ',),).toBe('',);
        expect(firstWhitespaceToken('\t\t',),).toBe('',);
        expect(firstWhitespaceToken(' \t \n ',),).toBe('',);
      },
    },),
    it({
      name: 'returns empty token for each single whitespace kind alone',
      fn: async function eachWhitespaceKindAlone() {
        expect(firstWhitespaceToken(' ',),).toBe('',);
        expect(firstWhitespaceToken('\t',),).toBe('',);
        expect(firstWhitespaceToken('\n',),).toBe('',);
        expect(firstWhitespaceToken('\r',),).toBe('',);
        expect(firstWhitespaceToken('\f',),).toBe('',);
        expect(firstWhitespaceToken('\v',),).toBe('',);
      },
    },),
    it({
      name: 'returns the whole string when it has no whitespace',
      fn: async function noWhitespace() {
        expect(firstWhitespaceToken('ripgrep',),).toBe('ripgrep',);
      },
    },),
    it({
      name: 'skips leading whitespace before the token',
      fn: async function leadingWhitespace() {
        expect(firstWhitespaceToken('   ripgrep',),).toBe('ripgrep',);
        expect(firstWhitespaceToken('\t\tripgrep',),).toBe('ripgrep',);
      },
    },),
    it({
      name: 'stops at trailing whitespace after the token',
      fn: async function trailingWhitespace() {
        expect(firstWhitespaceToken('ripgrep   ',),).toBe('ripgrep',);
        expect(firstWhitespaceToken('ripgrep\t',),).toBe('ripgrep',);
      },
    },),
    it({
      name: 'returns only the first token when leading and trailing whitespace surround it',
      fn: async function surroundingWhitespace() {
        expect(firstWhitespaceToken('   ripgrep   ',),).toBe('ripgrep',);
      },
    },),
    it({
      name: 'returns only the first token of multiple whitespace-separated tokens',
      fn: async function multipleTokens() {
        expect(firstWhitespaceToken('ripgrep aqua:BurntSushi/ripgrep',),).toBe(
          'ripgrep',
        );
        expect(firstWhitespaceToken('  ripgrep  cargo:ripgrep  ',),).toBe(
          'ripgrep',
        );
      },
    },),
    it({
      name: 'treats every whitespace kind as a leading skip and as a separator',
      fn: async function allWhitespaceKindsAroundToken() {
        expect(firstWhitespaceToken(' \t\n\r\f\vtool',),).toBe('tool',);
        expect(firstWhitespaceToken('tool \t\n\r\f\vrest',),).toBe('tool',);
      },
    },),
    it({
      name: 'parses the tool name from a tab-separated mise registry line',
      fn: async function miseRegistryLine() {
        expect(
          firstWhitespaceToken('node\taqua:nodejs/node core:node',),
        ).toBe('node',);
      },
    },),
    it({
      name: 'keeps non-whitespace punctuation inside the token',
      fn: async function punctuationInToken() {
        expect(firstWhitespaceToken('aqua:BurntSushi/ripgrep rest',),).toBe(
          'aqua:BurntSushi/ripgrep',
        );
      },
    },),
    it({
      name: 'preserves an astral character inside the token',
      fn: async function astralCharacter() {
        expect(firstWhitespaceToken('a\u{1F600}b rest',),).toBe('a\u{1F600}b',);
      },
    },),
    it({
      name: 'handles a long leading whitespace run, linear and stack-safe',
      fn: async function longLeadingWhitespaceRun() {
        const runLength = 100_000;
        const line = `${' '.repeat(runLength,)}tool`;
        expect(firstWhitespaceToken(line,),).toBe('tool',);
      },
    },),
    it({
      name: 'handles a long non-whitespace token, linear and stack-safe',
      fn: async function longTokenRun() {
        const runLength = 100_000;
        const token = 'x'.repeat(runLength,);
        expect(firstWhitespaceToken(`${token} rest`,),).toBe(token,);
        expect(firstWhitespaceToken(token,),).toBe(token,);
      },
    },),
  ],
},);
