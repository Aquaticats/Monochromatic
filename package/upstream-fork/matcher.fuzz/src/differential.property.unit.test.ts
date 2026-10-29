/**
 Differential property tests: the fork must produce the same observable
 verdicts as upstream `matcher` 6.1.0 on the same generated cases.
 
 The oracle is upstream itself: every wildcard, negation, escaping, case
 folding, option-mode, and validation behavior this fork claims is checked
 against the implementation it was derived from.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  assert,
  property,
} from 'fast-check';
import {
  isMatch as isMatchUpstream,
  matcher as matcherUpstream,
} from 'matcher';

import {
  isMatch,
  matcher,
} from '@monochromatic-dev/upstream-fork-matcher/ts';

import { fuzzRuns, } from './fuzz-budget.ts';
import { matcherCaseArb, } from './match-arbitrary.ts';

await describe({
  name: 'upstream matcher parity',
  children: [
    it({
      name: 'matcher verdicts match upstream matcher on every generated case',
      fn: async () => {
        await assert(
          property(matcherCaseArb, (matchCase,) => {
            /**
             Fork verdict for this case.
             */
            const forked = matcher({
              inputs: [...matchCase.inputs],
              patterns: [...matchCase.patterns],
              options: { ...matchCase.options, },
            },);
            /**
             Upstream verdict for this case.
             */
            const upstream = matcherUpstream(
              [...matchCase.inputs],
              [...matchCase.patterns],
              { ...matchCase.options, },
            );
            expect(forked,).toEqual(upstream,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'isMatch verdicts match upstream isMatch on every generated case',
      fn: async () => {
        await assert(
          property(matcherCaseArb, (matchCase,) => {
            /**
             Fork verdict for this case.
             */
            const forked = isMatch({
              inputs: [...matchCase.inputs],
              patterns: [...matchCase.patterns],
              options: { ...matchCase.options, },
            },);
            /**
             Upstream verdict for this case.
             */
            const upstream = isMatchUpstream(
              [...matchCase.inputs],
              [...matchCase.patterns],
              { ...matchCase.options, },
            );
            expect(forked,).toBe(upstream,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'validation errors match upstream matcher error messages',
      fn: async () => {
        await assert(
          property(matcherCaseArb, (matchCase,) => {
            for (const bad of [
              0,
              null,
              false,
            ]) {
              /**
               Fork inputs error message, if any.
               */
              let forkInputsMessage = '';
              try {
                matcher({
                  inputs: bad as never,
                  patterns: [...matchCase.patterns],
                },);
              }
              catch (error) {
                forkInputsMessage = (error as Error).message;
              }
              /**
               Upstream inputs error message, if any.
               */
              let upstreamInputsMessage = '';
              try {
                matcherUpstream(
                  bad as never,
                  [...matchCase.patterns],
                );
              }
              catch (error) {
                upstreamInputsMessage = (error as Error).message;
              }
              expect(forkInputsMessage,).toBe(upstreamInputsMessage,);

              /**
               Fork patterns error message, if any.
               */
              let forkPatternsMessage = '';
              try {
                matcher({
                  inputs: [...matchCase.inputs],
                  patterns: bad as never,
                },);
              }
              catch (error) {
                forkPatternsMessage = (error as Error).message;
              }
              /**
               Upstream patterns error message, if any.
               */
              let upstreamPatternsMessage = '';
              try {
                matcherUpstream(
                  [...matchCase.inputs],
                  bad as never,
                );
              }
              catch (error) {
                upstreamPatternsMessage = (error as Error).message;
              }
              expect(forkPatternsMessage,).toBe(upstreamPatternsMessage,);
            }
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),
  ],
},);
