/**
 Property tests proving the fork's matching invariants: single-pattern core
 against an independent backtracking oracle, multi-pattern combination
 rules, order and duplication preservation, empty-pattern behavior, and
 constructor-input totality.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  anything,
  assert,
  property,
} from 'fast-check';

import {
  InvalidInputsError,
  InvalidPatternsError,
  isMatch,
  matcher,
} from '@monochromatic-dev/upstream-fork-matcher/ts';

import { fuzzRuns, } from './fuzz-budget.ts';
import {
  matcherCaseArb,
  pairArb,
} from './match-arbitrary.ts';
import { referenceIsMatch, } from './reference-oracle.ts';

await describe({
  name: 'fork invariants',
  children: [
    it({
      name: 'single-pattern core agrees with the backtracking oracle',
      fn: async () => {
        await assert(
          property(pairArb, (pair,) => {
            /**
             Fork verdict for this pair.
             */
            const got = isMatch({
              inputs: [pair.input],
              patterns: [pair.pattern],
              options: { caseSensitive: pair.caseSensitive, },
            },);
            /**
             Oracle verdict for this pair.
             */
            const wanted = referenceIsMatch({
              input: pair.input,
              pattern: pair.pattern,
              caseSensitive: pair.caseSensitive,
            },);
            expect(got,).toBe(wanted,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'matcher keeps exactly the inputs isMatch would accept',
      fn: async () => {
        await assert(
          property(matcherCaseArb, (matchCase,) => {
            /**
             Inputs the filter keeps.
             */
            const kept = matcher({
              inputs: matchCase.inputs,
              patterns: matchCase.patterns,
              options: matchCase.options,
            },);
            for (const input of kept)
              expect(isMatch({
                inputs: [input],
                patterns: matchCase.patterns,
                options: matchCase.options,
              },),).toBe(true,);
            /**
             Whether any input matches under the single-input rule.
             */
            const someMatches = matchCase.inputs.some(function probe(input: string,): boolean {
              return isMatch({
                inputs: [input],
                patterns: matchCase.patterns,
                options: matchCase.options,
              },);
            },);
            if (!matchCase.options.allPatterns)
              expect(isMatch({
                inputs: matchCase.inputs,
                patterns: matchCase.patterns,
                options: matchCase.options,
              },),).toBe(someMatches,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'matcher preserves input order and duplicates',
      fn: async () => {
        await assert(
          property(matcherCaseArb, (matchCase,) => {
            /**
             Inputs the filter keeps.
             */
            const kept = matcher({
              inputs: matchCase.inputs,
              patterns: matchCase.patterns,
              options: matchCase.options,
            },);
            /**
             Kept inputs re-filtered: filtering is idempotent.
             */
            const rekept = matcher({
              inputs: kept,
              patterns: matchCase.patterns,
              options: matchCase.options,
            },);
            expect(rekept,).toEqual(kept,);
            /**
             Cursor walking the original inputs in order.
             */
            const cursor = { index: 0, };
            for (const input of kept) {
              /**
               Next occurrence of this input at or after the cursor.
               */
              const found = matchCase.inputs.indexOf(
                input,
                cursor.index,
              );
              expect(found,).toBeGreaterThanOrEqual(0,);
              cursor.index = found + 1;
            }
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'empty patterns match nothing and empty pattern matches only empty input',
      fn: async () => {
        await assert(
          property(matcherCaseArb, (matchCase,) => {
            expect(matcher({
              inputs: matchCase.inputs,
              patterns: [],
            },),).toEqual([],);
            expect(isMatch({
              inputs: matchCase.inputs,
              patterns: [],
            },),).toBe(false,);
            for (const input of matchCase.inputs)
              expect(isMatch({
                inputs: [input],
                patterns: [''],
              },),).toBe(input === '',);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'input totality: any value yields a list verdict or one validation error',
      fn: async () => {
        await assert(
          property(
            anything(),
            anything(),
            (inputs: unknown, patterns: unknown,) => {
              /**
               Outcome classification for these arguments.
               */
              let outcome = 'verdict';
              try {
                matcher({
                  inputs: inputs as never,
                  patterns: patterns as never,
                },);
              }
              catch (error) {
                if (error instanceof InvalidInputsError)
                  outcome = 'invalidInputs';
                else if (error instanceof InvalidPatternsError)
                  outcome = 'invalidPatterns';
                else if (error instanceof TypeError)
                  outcome = 'typeError';
                else
                  outcome = 'unexpected';
              }
              expect([
                'verdict',
                'invalidInputs',
                'invalidPatterns',
              ],).toContain(outcome,);
            },
          ),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),
  ],
},);
