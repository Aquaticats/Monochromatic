/**
 Unit tests for the shared separator tokenizer.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isWhitespace, } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

import { splitSeparatedTokens, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Separators used by the model-id and speed-signal call sites.
 */
const PUNCTUATION_SEPARATORS = new Set([
  '.',
  '_',
  '-',
  ':',
  '/',
],);

/**
 Check whether a character separates model text.
 
 @param character - character from a model text field
 
 @returns whether the character is punctuation or ASCII whitespace
 */
function isModelTextSeparator(
  character: string,
): boolean {
  return PUNCTUATION_SEPARATORS.has(character,)
    || isWhitespace(character,);
}

//endregion Fixtures

await describe({
  name: splitSeparatedTokens.name,
  children: [
    it({
      name: 'splits at punctuation and whitespace separators',
      fn: async function testPunctuationSeparators(): Promise<void> {
        expect(splitSeparatedTokens({
          value: 'claude-3.5 sonnet:latest',
          isSeparator: isModelTextSeparator,
        },),).toEqual([
          'claude',
          '3',
          '5',
          'sonnet',
          'latest',
        ],);
      },
    },),
    it({
      name: 'drops empty tokens at every separator run and at both ends',
      fn: async function testEmptyTokens(): Promise<void> {
        expect(splitSeparatedTokens({
          value: '--a..b  c-',
          isSeparator: isModelTextSeparator,
        },),).toEqual([
          'a',
          'b',
          'c',
        ],);
        expect(splitSeparatedTokens({
          value: '',
          isSeparator: isModelTextSeparator,
        },),).toEqual([],);
        expect(splitSeparatedTokens({
          value: '---',
          isSeparator: isModelTextSeparator,
        },),).toEqual([],);
      },
    },),
    it({
      name: 'keeps separator characters that the caller does not name',
      fn: async function testCallerOwnedSeparators(): Promise<void> {
        expect(splitSeparatedTokens({
          value: 'a-b/c',
          isSeparator: function isHyphen(character,) {
            return character === '-';
          },
        },),).toEqual([
          'a',
          'b/c',
        ],);
      },
    },),
    it({
      name: 'keeps characters outside the BMP inside one token',
      fn: async function testSurrogatePairs(): Promise<void> {
        expect(splitSeparatedTokens({
          value: 'a\u{1F600}b-c',
          isSeparator: function isHyphen(character,) {
            return character === '-';
          },
        },),).toEqual([
          'a\u{1F600}b',
          'c',
        ],);
      },
    },),
  ],
},);
