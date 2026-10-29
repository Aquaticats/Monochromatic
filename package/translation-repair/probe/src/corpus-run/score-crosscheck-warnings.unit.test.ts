/**
 Tests for the warnings `score-crosscheck` prints beside its counts.

 Each warning prints nothing for a zero count, and each states its count with
 the noun and verb that agree with it, so a case holds the zero, the one and
 the several of each.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  printCrosscheckMalformed,
  printJoinFailureWarning,
  printUnjudgeableWarning,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 What a malformed-artifact warning says after the count, noun and verb.
 */
const MALFORMED_TAIL = 'NEITHER population this report counts, so every count is over the rest:';

/**
 What a join-failure warning says after its noun and verb.
 */
const JOIN_TAIL = 'entries that DO carry attribution yet have no proposer recorded. That is the two records '
  + 'disagreeing about claim identity, not a quiet critic, and it is reported apart from the legacy count so '
  + 'it cannot hide inside an expected number.';

/**
 What an unjudgeable warning says after its noun and verb.
 */
const UNJUDGEABLE_TAIL = 'the WHOLE roster, leaving no seat to judge. Such claims are reported here rather than '
  + 'dropped: they are the most corroborated claims in the run, and removing them would lift every rate by '
  + 'hiding exactly the strongest agreement in the population.';

await describe({
  name: 'score-crosscheck-warnings',
  concurrency: 1,
  children: [
    describe({
      name: printCrosscheckMalformed.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS nothing when every artifact could be read',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printCrosscheckMalformed({ malformed: [], },);

            expect(printed.lines,).toStrictEqual([],);
          },
        },),

        it({
          name: 'NAMES one artifact that could not be read, with its noun and verb in the singular',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printCrosscheckMalformed({
              malformed: [{ name: 'Biscuit.json', reason: 'cut off at byte 12', },],
            },);

            expect(printed.lines,).toStrictEqual([
              `WARNING 1 artifact could not be read and is in ${MALFORMED_TAIL}`,
              '  Biscuit.json: cut off at byte 12',
            ],);
          },
        },),

        it({
          name: 'NAMES several artifacts that could not be read, each on its own line, in the plural',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printCrosscheckMalformed({
              malformed: [
                { name: 'Biscuit.json', reason: 'cut off at byte 12', },
                { name: 'Pounce.json', reason: 'not a record', },
              ],
            },);

            expect(printed.lines,).toStrictEqual([
              `WARNING 2 artifacts could not be read and are in ${MALFORMED_TAIL}`,
              '  Biscuit.json: cut off at byte 12',
              '  Pounce.json: not a record',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: printJoinFailureWarning.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS nothing when every claim of an attributed entry has a proposer',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printJoinFailureWarning({ count: 0, },);

            expect(printed.lines,).toStrictEqual([],);
          },
        },),

        it({
          name: 'WARNS of one claim with its noun and verb in the singular',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printJoinFailureWarning({ count: 1, },);

            expect(printed.lines,).toStrictEqual([`WARNING 1 claim sits on ${JOIN_TAIL}`,],);
          },
        },),

        it({
          name: 'WARNS of several claims with their noun and verb in the plural',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printJoinFailureWarning({ count: 2, },);

            expect(printed.lines,).toStrictEqual([`WARNING 2 claims sit on ${JOIN_TAIL}`,],);
          },
        },),
      ],
    },),

    describe({
      name: printUnjudgeableWarning.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS nothing when every claim has a seat to judge it',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printUnjudgeableWarning({ count: 0, },);

            expect(printed.lines,).toStrictEqual([],);
          },
        },),

        it({
          name: 'WARNS of one claim with its noun and verb in the singular',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printUnjudgeableWarning({ count: 1, },);

            expect(printed.lines,).toStrictEqual([`WARNING 1 claim was proposed by ${UNJUDGEABLE_TAIL}`,],);
          },
        },),

        it({
          name: 'WARNS of several claims with their noun and verb in the plural',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printUnjudgeableWarning({ count: 2, },);

            expect(printed.lines,).toStrictEqual([`WARNING 2 claims were proposed by ${UNJUDGEABLE_TAIL}`,],);
          },
        },),
      ],
    },),
  ],
},);
