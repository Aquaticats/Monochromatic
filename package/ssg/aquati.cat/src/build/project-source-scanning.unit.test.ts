/**
 Tests for the linear scanners that read icon call sites out of project source.
 
 The expected values were produced by the regular expressions each scanner
 replaced, run over the same inputs before those patterns were deleted. A case
 whose expectation a reader cannot derive by hand is wrong, not the scanner.
 
 Every fixture text is built through {@link callSite} instead of written out.
 The font subsetting pass scans every file below `src/` for icon call sites and
 fails the build on a name that is not a Material Symbols icon, so a fixture
 written literally here would have to name a real icon, and the fixtures are
 deliberately names that resolve to nothing.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  collectIconCallNames,
  stripBlockComments,
  stripLineComments,
} from '../../dist/final/node/index.mjs';

/**
 Identifier the scanners look for, kept in pieces so this file's own text holds
 no call site for the font subsetting pass to resolve.
 */
const ICON_HELPER = 'icon';

/**
 Builds one call site, with the argument text given verbatim between the
 parentheses.
 
 @param argument - text the call receives, quotes and spaces included
 
 @returns the call as a source file would spell it
 */
function callSite(argument: string,): string {
  return `${ICON_HELPER}(${argument})`;
}

/**
 Block comment inputs, each paired with what the deleted block comment pattern
 left of them.
 */
const BLOCK_COMMENT_CASES = [
  {
    name: 'a comment between two calls',
    text: `${callSite("'a'")}; /* note */ ${callSite("'b'")};`,
    expected: `${callSite("'a'")};  ${callSite("'b'")};`,
    reason: 'the delimiters go and the space on either side stays',
  },
  {
    name: 'text without a comment',
    text: `${callSite("'a'")};`,
    expected: `${callSite("'a'")};`,
    reason: 'nothing to remove',
  },
  {
    name: 'empty text',
    text: '',
    expected: '',
    reason: 'no opening delimiter to find',
  },
  {
    name: 'an unterminated opening delimiter',
    text: 'a /* b',
    expected: 'a /* b',
    reason: 'a block comment the text never closes is not a comment',
  },
  {
    name: 'the smallest possible comment',
    text: '/**/',
    expected: '',
    reason: 'an opening immediately closed',
  },
  {
    name: 'an empty comment fenced by slashes',
    text: '/*//*/x',
    expected: 'x',
    reason: 'the first closing delimiter ends the comment, so the `//` inside it goes too',
  },
  {
    name: 'two adjacent comments',
    text: '/*a*//*b*/',
    expected: '',
    reason: 'the second opening is found after the first closing',
  },
  {
    name: 'comments around kept text',
    text: 'a/*b*/c/*d*/e',
    expected: 'ace',
    reason: 'every run between comments survives in order',
  },
  {
    name: 'a comment spanning lines',
    text: 'a/*\n\n*/b',
    expected: 'ab',
    reason: 'the character class matched newlines, so the stripper does too',
  },
] as const;

/**
 Line comment inputs, each paired with what the deleted line comment pattern
 left of them.
 */
const LINE_COMMENT_CASES = [
  {
    name: 'a comment after a call',
    text: `${callSite("'a'")}; // note\n${callSite("'b'")};`,
    expected: `${callSite("'a'")}; \n${callSite("'b'")};`,
    reason: 'the comment goes and the newline that ended it stays',
  },
  {
    name: 'a comment with nothing before it',
    text: '//only',
    expected: '',
    reason: 'a comment that starts the text and no newline after it',
  },
  {
    name: 'a comment at the end of a line',
    text: 'a//b',
    expected: 'a',
    reason: 'no newline, so the comment runs to the end of the text',
  },
  {
    name: 'comments on several lines',
    text: 'a//b\nc//d\ne',
    expected: 'a\nc\ne',
    reason: 'each search resumes at the newline the previous comment stopped at',
  },
  {
    name: 'three markers in a row',
    text: 'a///b',
    expected: 'a',
    reason: 'the character class matched the third slash too',
  },
  {
    name: 'a trailing newline',
    text: 'a//b\n',
    expected: 'a\n',
    reason: 'the newline is outside the comment',
  },
  {
    name: 'block comment delimiters',
    text: '/* a */',
    expected: '/* a */',
    reason: 'line comment markers are the only ones this stripper knows',
  },
  {
    name: 'a carriage return before the newline',
    text: 'a//b\r\nc',
    expected: 'a\nc',
    reason: 'the character class excluded only the newline, so the carriage return was consumed with the comment',
  },
  {
    name: 'no markers',
    text: `${callSite("'a'")};`,
    expected: `${callSite("'a'")};`,
    reason: 'nothing to remove',
  },
  {
    name: 'empty text',
    text: '',
    expected: '',
    reason: 'no marker to find',
  },
] as const;

/**
 Icon call inputs, each paired with the names the deleted call pattern captured
 from it, in match order.
 */
const ICON_CALL_CASES = [
  {
    name: 'a single-quoted call',
    text: callSite("'star'"),
    expected: ['star',],
    reason: 'the name between the quotes',
  },
  {
    name: 'a double-quoted call',
    text: callSite('"star"'),
    expected: ['star',],
    reason: 'either quote character opens the name',
  },
  {
    name: 'a call padded with spaces',
    text: callSite(" 'star' "),
    expected: ['star',],
    reason: 'whitespace around the argument is optional',
  },
  {
    name: 'a call with a trailing comma',
    text: callSite("'star',"),
    expected: ['star',],
    reason: 'the comma is optional on either side of the closing parenthesis',
  },
  {
    name: 'a call split over three lines',
    text: callSite("\n 'star'\n "),
    expected: ['star',],
    reason: 'the whitespace class includes newlines',
  },
  {
    name: 'a call padded with a no-break space',
    text: callSite("\u00A0'star'"),
    expected: ['star',],
    reason: 'the whitespace class includes the no-break space',
  },
  {
    name: 'a call padded with a byte order mark',
    text: callSite("\uFEFF'star'"),
    expected: ['star',],
    reason: 'the whitespace class includes the byte order mark',
  },
  {
    name: 'a name of letters, digits, and underscores',
    text: callSite("'a1_b'"),
    expected: ['a1_b',],
    reason: 'the name alphabet after the first letter',
  },
  {
    name: 'two calls on one line',
    text: `${callSite("'a'")} ${callSite("'b'")}`,
    expected: ['a', 'b',],
    reason: 'the scan resumes after the closing parenthesis',
  },
  {
    name: 'two calls with nothing between them',
    text: `${callSite("'a'")}${callSite("'b'")}`,
    expected: ['a', 'b',],
    reason: 'no separator is needed between calls',
  },
  {
    name: 'a call with a trailing comma and spaces',
    text: callSite("'a' , "),
    expected: ['a',],
    reason: 'whitespace may sit on either side of the comma',
  },
  {
    name: 'a repeated call',
    text: `${callSite("'a'")}, ${callSite("'a'")}`,
    expected: ['a', 'a',],
    reason: 'names repeat in call order; the caller owns deduplication',
  },
  {
    name: 'a call nested inside a failed candidate',
    text: `${ICON_HELPER}(${callSite("'a'")}`,
    expected: ['a',],
    reason: 'a candidate that is not a call is left one character further on',
  },
  {
    name: 'a call whose quotes do not pair',
    text: callSite("'a\""),
    expected: ['a',],
    reason: 'the class at each end accepted either quote, so pairing was never checked',
  },
  {
    name: 'a call preceded by a word character',
    text: `my${callSite("'star'")}`,
    expected: [],
    reason: 'the word boundary keeps a longer identifier from matching',
  },
  {
    name: 'a call preceded by an underscore',
    text: `_${callSite("'star'")}`,
    expected: [],
    reason: 'the underscore is a word character for the boundary',
  },
  {
    name: 'an identifier that starts like a call',
    text: 'iconx(\'star\')',
    expected: [],
    reason: 'the opening token needs its own parenthesis',
  },
  {
    name: 'a call with a space before the parenthesis',
    text: 'icon (\'star\')',
    expected: [],
    reason: 'the opening token is one word',
  },
  {
    name: 'an upper-case identifier',
    text: 'ICON(\'star\')',
    expected: [],
    reason: 'the opening token is case-sensitive',
  },
  {
    name: 'a name that starts with an upper-case letter',
    text: `${ICON_HELPER}('Star')`,
    expected: [],
    reason: 'the name alphabet is lower case only',
  },
  {
    name: 'an empty name',
    text: `${ICON_HELPER}('')`,
    expected: [],
    reason: 'the name needs at least one letter',
  },
  {
    name: 'a name that starts with a digit',
    text: `${ICON_HELPER}('1a')`,
    expected: [],
    reason: 'the first character must be a letter',
  },
  {
    name: 'a name with a dash in it',
    text: `${ICON_HELPER}('st-ar')`,
    expected: [],
    reason: 'a dash is outside the name alphabet',
  },
  {
    name: 'a variable argument',
    text: `${ICON_HELPER}(name)`,
    expected: [],
    reason: 'only string literals name an icon',
  },
  {
    name: 'an unclosed call',
    text: `${ICON_HELPER}('a'`,
    expected: [],
    reason: 'a call the text never closes is not a call',
  },
] as const;

await describe({
  name: '',
  children: [
    describe({
      name: stripBlockComments.name,
      children: BLOCK_COMMENT_CASES.map(function blockCommentCase({ name, text, expected, reason, },) {
        return it({
          name: `strips ${name} (${reason})`,
          fn: async function stripsBlockComment(): Promise<void> {
            expect(stripBlockComments(text,),).toBe(expected,);
          },
        },);
      },),
    },),
    describe({
      name: stripLineComments.name,
      children: LINE_COMMENT_CASES.map(function lineCommentCase({ name, text, expected, reason, },) {
        return it({
          name: `strips ${name} (${reason})`,
          fn: async function stripsLineComment(): Promise<void> {
            expect(stripLineComments(text,),).toBe(expected,);
          },
        },);
      },),
    },),
    describe({
      name: collectIconCallNames.name,
      children: ICON_CALL_CASES.map(function iconCallCase({ name, text, expected, reason, },) {
        return it({
          name: `collects from ${name} (${reason})`,
          fn: async function collectsIconCalls(): Promise<void> {
            expect(collectIconCallNames(text,),).toEqual(expected,);
          },
        },);
      },),
    },),
  ],
},);
