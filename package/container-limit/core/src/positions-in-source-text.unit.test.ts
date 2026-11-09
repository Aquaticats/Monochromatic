import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  blankRange,
  blockCommentEnd,
  bracketPairEnd,
  closingMultiLineQuote,
  continuesName,
  endOfQuotedString,
  endOfSingleLineString,
  isNameCharacter,
  isQuote,
  lineEnd,
  nameMatchesAt,
  statementEnd,
} from './positions-in-source-text.ts';

/**
 Characters of a fixture, which every finder takes as a list.
 */
function charactersOf(text: string,): readonly string[] {
  return text.split('',);
}

await describe({
  name: '',
  children: [
    describe({
      name: isNameCharacter.name,
      children: [
        it({
          name: 'accepts every character a name can hold',
          fn: async () => {
            for (const character of ['a', 'Z', '0', '_', '$',])
              expect(isNameCharacter(character,),).toBe(true,);
          },
        },),
        it({
          name: 'rejects punctuation, spaces and the end of the text',
          fn: async () => {
            for (const character of ['-', '.', ' ', ':', ''])
              expect(isNameCharacter(character,),).toBe(false,);
            expect(isNameCharacter(),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: continuesName.name,
      children: [
        it({
          name: 'carries a name on through a letter',
          fn: async () => {
            expect(continuesName({
              characters: charactersOf('abc',),
              index: 1,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'ends a word at the escaped letter of a one-line script',
          fn: async () => {
            expect(continuesName({
              characters: charactersOf(String.raw`\nconst`,),
              index: 2,
            },),).toBe(false,);
          },
        },),
        it({
          name: 'ends a word at a character no name holds',
          fn: async () => {
            expect(continuesName({
              characters: charactersOf('a b',),
              index: 1,
            },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isQuote.name,
      children: [
        it({
          name: 'accepts the three quotes a string can open with',
          fn: async () => {
            for (const quote of ['"', '\'', '`',])
              expect(isQuote(quote,),).toBe(true,);
          },
        },),
        it({
          name: 'rejects anything else, including the end of the text',
          fn: async () => {
            expect(isQuote('a',),).toBe(false,);
            expect(isQuote(),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: nameMatchesAt.name,
      children: [
        it({
          name: 'finds a whole word',
          fn: async () => {
            expect(nameMatchesAt({
              characters: charactersOf("spawn('podman', args)",),
              index: 7,
              name: 'podman',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects a name inside a longer word',
          fn: async () => {
            expect(nameMatchesAt({
              characters: charactersOf('mypodman',),
              index: 2,
              name: 'podman',
            },),).toBe(false,);
            expect(nameMatchesAt({
              characters: charactersOf('podmans',),
              index: 0,
              name: 'podman',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'finds a declaration that follows the escaped newline of a one-line script',
          fn: async () => {
            expect(nameMatchesAt({
              characters: charactersOf(String.raw`\nconst`,),
              index: 2,
              name: 'const',
            },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: blankRange.name,
      children: [
        it({
          name: 'blanks the range and keeps its newlines',
          fn: async () => {
            const characters = [...charactersOf('ab\ncd',)];
            blankRange({
              characters,
              from: 0,
              to: 5,
            },);
            expect(characters.join('',),).toBe('  \n  ',);
          },
        },),
      ],
    },),
    describe({
      name: lineEnd.name,
      children: [
        it({
          name: 'stops at the newline',
          fn: async () => {
            expect(lineEnd({
              characters: charactersOf('ab\ncd',),
              from: 0,
              to: 5,
            },),).toBe(2,);
          },
        },),
        it({
          name: 'stops at the end of the range when the text holds no newline',
          fn: async () => {
            expect(lineEnd({
              characters: charactersOf('abcd',),
              from: 0,
              to: 4,
            },),).toBe(4,);
          },
        },),
      ],
    },),
    describe({
      name: endOfQuotedString.name,
      children: [
        it({
          name: 'reads past the closing quote',
          fn: async () => {
            expect(endOfQuotedString({
              characters: charactersOf("'run', 'x'",),
              quote: 7,
              to: 12,
            },),).toBe(10,);
          },
        },),
        it({
          name: 'reads past an escaped quote',
          fn: async () => {
            expect(endOfQuotedString({
              characters: charactersOf(String.raw`'a\'b'`,),
              quote: 0,
              to: 6,
            },),).toBe(6,);
          },
        },),
        it({
          name: 'stops at the end of the range when the string never closes',
          fn: async () => {
            expect(endOfQuotedString({
              characters: charactersOf("'run",),
              quote: 0,
              to: 4,
            },),).toBe(4,);
          },
        },),
      ],
    },),
    describe({
      name: endOfSingleLineString.name,
      children: [
        it({
          name: 'reads past the closing quote of a string on one line',
          fn: async () => {
            expect(endOfSingleLineString({
              characters: charactersOf("'a'\nb",),
              quote: 0,
              to: 5,
            },),).toBe(3,);
          },
        },),
        it({
          name: 'treats a prose apostrophe as one character',
          fn: async () => {
            expect(endOfSingleLineString({
              characters: charactersOf("the program's license",),
              quote: 11,
              to: 21,
            },),).toBe(12,);
          },
        },),
      ],
    },),
    describe({
      name: blockCommentEnd.name,
      children: [
        it({
          name: 'reads past the closing marker',
          fn: async () => {
            expect(blockCommentEnd({
              characters: charactersOf('/* a */ b',),
              from: 0,
              to: 9,
            },),).toBe(7,);
          },
        },),
        it({
          name: 'stops at the end of the range when the comment never closes',
          fn: async () => {
            expect(blockCommentEnd({
              characters: charactersOf('/* a',),
              from: 0,
              to: 4,
            },),).toBe(4,);
          },
        },),
      ],
    },),
    describe({
      name: closingMultiLineQuote.name,
      children: [
        it({
          name: 'finds the marker that closes the string',
          fn: async () => {
            expect(closingMultiLineQuote({
              characters: charactersOf('a """b',),
              from: 0,
              marker: '"""',
            },),).toBe(2,);
          },
        },),
        it({
          name: 'reports the end of the text when nothing closes it',
          fn: async () => {
            expect(closingMultiLineQuote({
              characters: charactersOf('abc',),
              from: 0,
              marker: '"""',
            },),).toBe(3,);
          },
        },),
      ],
    },),
    describe({
      name: bracketPairEnd.name,
      children: [
        it({
          name: 'reads past a nested pair',
          fn: async () => {
            expect(bracketPairEnd({
              characters: charactersOf('[a, [b], c] d',),
              opening: 0,
              to: 12,
            },),).toBe(11,);
          },
        },),
        it({
          name: 'does not end a pair at the bracket of a quoted string',
          fn: async () => {
            expect(bracketPairEnd({
              characters: charactersOf("[']', 'a']",),
              opening: 0,
              to: 10,
            },),).toBe(10,);
          },
        },),
        it({
          name: 'stops at the end of the range for a character that opens no pair',
          fn: async () => {
            expect(bracketPairEnd({
              characters: charactersOf('abc',),
              opening: 0,
              to: 3,
            },),).toBe(3,);
          },
        },),
        it({
          name: 'stops at the end of the range for a pair that never closes',
          fn: async () => {
            expect(bracketPairEnd({
              characters: charactersOf('[a',),
              opening: 0,
              to: 2,
            },),).toBe(2,);
          },
        },),
      ],
    },),
    describe({
      name: statementEnd.name,
      children: [
        it({
          name: 'reads past the semicolon that ends the statement',
          fn: async () => {
            expect(statementEnd({
              characters: charactersOf('a; b;',),
              from: 0,
            },),).toBe(2,);
          },
        },),
        it({
          name: 'ignores a semicolon inside a quoted specifier',
          fn: async () => {
            expect(statementEnd({
              characters: charactersOf("from 'a;b' ; c",),
              from: 0,
            },),).toBe(12,);
          },
        },),
        it({
          name: 'reports the end of the text for a statement that never closes',
          fn: async () => {
            expect(statementEnd({
              characters: charactersOf('a',),
              from: 0,
            },),).toBe(1,);
          },
        },),
      ],
    },),
  ],
},);
