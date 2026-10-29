/**
 Tests for the warnings `score-attribution` prints beside its counts.

 Each warning prints nothing for a zero count, and each states its count with
 the noun, verb and pronoun that agree with it, so a case holds the zero, the
 one and the several of each.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  printMalformedArtifacts,
  printPartialJoinWarning,
  printUnattributedWarning,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 What a malformed-artifact warning says after the count, noun and verb.
 */
const MALFORMED_TAIL = 'NEITHER population this report counts, so every count is over the rest. '
  + 'Named rather than summarized, because a truncated artifact is a different problem from a '
  + 'malformed one:';

/**
 What a partial-join warning says after its pronoun and verb.
 */
const PARTIAL_TAIL = 'held out of every other count in this report rather than counted as support, '
  + 'because the unattributed member may have come from a critic that got no credit. A nonzero '
  + 'number here is a defect in the join, not a fact about critics.';

/**
 What an unattributed warning says after its verb.
 */
const UNATTRIBUTED_TAIL = 'no attribution, meaning a claim id the index does not hold. That is a '
  + 'defect in the join, not a quiet critic.';

await describe({
  name: 'score-attribution-warnings',
  concurrency: 1,
  children: [
    describe({
      name: printMalformedArtifacts.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS nothing when every artifact could be read',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printMalformedArtifacts({ malformed: [], },);

            expect(printed.lines,).toStrictEqual([],);
          },
        },),

        it({
          name: 'NAMES one artifact that could not be read, with its noun and verb in the singular',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printMalformedArtifacts({
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

            printMalformedArtifacts({
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
      name: printPartialJoinWarning.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS nothing when no accepted issue joined partially',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printPartialJoinWarning({ count: 0, },);

            expect(printed.lines,).toStrictEqual([],);
          },
        },),

        it({
          name: 'WARNS of one accepted issue with its noun, pronoun and verb in the singular',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printPartialJoinWarning({ count: 1, },);

            expect(printed.lines,).toStrictEqual([
              `WARNING 1 accepted issue joined only SOME of its claims to attribution. It is ${PARTIAL_TAIL}`,
            ],);
          },
        },),

        it({
          name: 'WARNS of several accepted issues with their noun, pronoun and verb in the plural',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printPartialJoinWarning({ count: 2, },);

            expect(printed.lines,).toStrictEqual([
              `WARNING 2 accepted issues joined only SOME of their claims to attribution. Those are ${PARTIAL_TAIL}`,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: printUnattributedWarning.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS nothing when every accepted issue carries attribution',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printUnattributedWarning({ count: 0, },);

            expect(printed.lines,).toStrictEqual([],);
          },
        },),

        it({
          name: 'WARNS of one accepted issue with its noun and verb in the singular',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printUnattributedWarning({ count: 1, },);

            expect(printed.lines,).toStrictEqual([
              `WARNING 1 accepted issue on ELIGIBLE entries carries ${UNATTRIBUTED_TAIL}`,
            ],);
          },
        },),

        it({
          name: 'WARNS of several accepted issues with their noun and verb in the plural',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printUnattributedWarning({ count: 2, },);

            expect(printed.lines,).toStrictEqual([
              `WARNING 2 accepted issues on ELIGIBLE entries carry ${UNATTRIBUTED_TAIL}`,
            ],);
          },
        },),
      ],
    },),
  ],
},);
