/**
 Tests for the linear scanners that read icon call sites out of project source.
 
 The expected values were produced by the three regular expressions these
 scanners replaced, run over the same inputs before those patterns were deleted:
 `\/\*[\s\S]*?\*\/` with the `gu` flags for block comments, and one entry per
 case naming the input it came from. A case whose expectation a reader cannot
 derive by hand is wrong, not the scanner.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { stripBlockComments, } from '../../dist/final/node/index.mjs';

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
  ],
},);
