/**
 Tests for the linear scanners that read icon call sites out of project source.
 
 The expected values were produced by the regular expressions each scanner
 replaced, run over the same inputs before those patterns were deleted. A case
 whose expectation a reader cannot derive by hand is wrong, not the scanner.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { stripBlockComments, stripLineComments, } from '../../dist/final/node/index.mjs';

/**
 Block comment inputs, each paired with what the deleted `\/\*[\s\S]*?\*\/`
 pattern left of them.
 */
const BLOCK_COMMENT_CASES = [
  {
    name: 'a comment between two calls',
    text: "icon('a'); /* note */ icon('b');",
    expected: "icon('a');  icon('b');",
    reason: 'the delimiters go and the space on either side stays',
  },
  {
    name: 'text without a comment',
    text: "icon('a');",
    expected: "icon('a');",
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
 Line comment inputs, each paired with what the deleted `\/\/[^\n]*` pattern
 left of them.
 */
const LINE_COMMENT_CASES = [
  {
    name: 'a comment after a call',
    text: "icon('a'); // note\nicon('b');",
    expected: "icon('a'); \nicon('b');",
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
    text: "icon('a');",
    expected: "icon('a');",
    reason: 'nothing to remove',
  },
  {
    name: 'empty text',
    text: '',
    expected: '',
    reason: 'no marker to find',
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
  ],
},);
