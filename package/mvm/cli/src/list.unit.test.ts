/**
 Equivalence tests for `splitOnWhitespace`.
 
 Capture the pre-refactor behavior of the whitespace splitter so the
 linear-pass rewrite stays behavior-identical: empty and all-whitespace
 inputs, leading and trailing whitespace, interior whitespace runs, each
 ASCII whitespace variety, the documented examples, realistic virsh data
 rows, and long inputs that exercise stack-safety and linear time.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { splitOnWhitespace, } from '@monochromatic-dev/mvm-cli';

/** Length for the long single-run cases; large enough to overflow a per-character recursion under an engine without tail-call elimination. */
const STACK_RUN = 100_000;

/** Token count for the linear-time guard; the former `[...acc, token]` accumulator was O(n^2) and could not complete this, so it regresses if quadratic behavior returns. */
const LINEAR_GUARD_TOKENS = 100_000;

await describe({
  name: splitOnWhitespace.name,
  children: [
    it({
      name: 'returns empty array for empty string',
      fn: async () => {
        expect(splitOnWhitespace('',),).toEqual([],);
      },
    },),

    it({
      name: 'returns empty array for a run of spaces',
      fn: async () => {
        expect(splitOnWhitespace('     ',),).toEqual([],);
      },
    },),

    it({
      name: 'returns empty array for mixed whitespace varieties',
      fn: async () => {
        expect(splitOnWhitespace('\t\n\r\f\v ',),).toEqual([],);
      },
    },),

    it({
      name: 'returns a single token unchanged',
      fn: async () => {
        expect(splitOnWhitespace('abc',),).toEqual(['abc',],);
      },
    },),

    it({
      name: 'drops leading whitespace',
      fn: async () => {
        expect(splitOnWhitespace('   abc',),).toEqual(['abc',],);
      },
    },),

    it({
      name: 'drops trailing whitespace',
      fn: async () => {
        expect(splitOnWhitespace('abc   ',),).toEqual(['abc',],);
      },
    },),

    it({
      name: 'drops leading and trailing whitespace together',
      fn: async () => {
        expect(splitOnWhitespace('  abc  ',),).toEqual(['abc',],);
      },
    },),

    it({
      name: 'splits two space-separated tokens',
      fn: async () => {
        expect(splitOnWhitespace('a b',),).toEqual([
          'a',
          'b',
        ],);
      },
    },),

    it({
      name: 'collapses an interior whitespace run',
      fn: async () => {
        expect(splitOnWhitespace('a    b',),).toEqual([
          'a',
          'b',
        ],);
      },
    },),

    it({
      name: 'splits on each whitespace variety',
      fn: async () => {
        expect(splitOnWhitespace('a\tb\nc\rd\fe\vf',),).toEqual([
          'a',
          'b',
          'c',
          'd',
          'e',
          'f',
        ],);
      },
    },),

    it({
      name: 'matches the first documented example',
      fn: async () => {
        expect(splitOnWhitespace('  a  b\tc',),).toEqual([
          'a',
          'b',
          'c',
        ],);
      },
    },),

    it({
      name: 'matches the second documented example',
      fn: async () => {
        expect(splitOnWhitespace('   ',),).toEqual([],);
      },
    },),

    it({
      name: 'treats hyphen and underscore as token characters (only whitespace splits)',
      fn: async () => {
        expect(splitOnWhitespace('a-b c_d',),).toEqual([
          'a-b',
          'c_d',
        ],);
      },
    },),

    it({
      name: 'tokenizes a realistic virsh data row',
      fn: async () => {
        expect(splitOnWhitespace(' 1    mvm-foo    running',),).toEqual([
          '1',
          'mvm-foo',
          'running',
        ],);
      },
    },),

    it({
      name: 'keeps a multi-word state as separate trailing tokens',
      fn: async () => {
        expect(splitOnWhitespace(' -    mvm-bar    shut off',),).toEqual([
          '-',
          'mvm-bar',
          'shut',
          'off',
        ],);
      },
    },),

    it({
      name: 'returns empty for a long whitespace run without overflowing the stack',
      fn: async () => {
        expect(
          splitOnWhitespace(' '.repeat(STACK_RUN,),),
        ).toEqual([],);
      },
    },),

    it({
      name: 'returns one token for a long non-whitespace run without overflowing the stack',
      fn: async () => {
        const token = 'a'.repeat(STACK_RUN,);
        const result = splitOnWhitespace(token,);
        expect(result.length,).toBe(1,);
        expect(result[0],).toBe(token,);
      },
    },),

    it({
      name: 'splits many single-character tokens',
      fn: async () => {
        const tokenCount = 50;
        const result = splitOnWhitespace('a '.repeat(tokenCount,),);
        expect(result.length,).toBe(tokenCount,);
        expect(result[0],).toBe('a',);
        expect(result[tokenCount - 1],).toBe('a',);
      },
    },),

    it({
      name: 'splits a large number of tokens in linear time',
      fn: async () => {
        const result = splitOnWhitespace('a '.repeat(LINEAR_GUARD_TOKENS,),);
        expect(result.length,).toBe(LINEAR_GUARD_TOKENS,);
        expect(result[0],).toBe('a',);
        expect(result[LINEAR_GUARD_TOKENS - 1],).toBe('a',);
      },
    },),
  ],
},);
