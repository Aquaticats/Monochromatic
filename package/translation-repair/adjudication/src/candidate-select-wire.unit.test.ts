/**
 Tests for the judge-ballot wire guard.

 `isCandidateBallotWire` is the only check between a malformed judge reply and
 code that reads `best` as an index. It had no test.

 One boundary is deliberately absent from the guard and is pinned here so
 nobody adds it: there is no UPPER bound on `best`. A ballot naming a
 candidate that does not exist is admitted on purpose, because
 `selectBestCandidate` counts it as an abstention rather than discarding the
 whole ballot, and that behavior has its own test. Rejecting the out-of-range
 ballot here would turn an abstention into a lost voice and change what a
 quorum means.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  buildCandidateSelectMessages,
  CANDIDATE_NONE,
  isCandidateBallotAsSent,
  isCandidateBallotWire,
  readCandidateBallotWire,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: buildCandidateSelectMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A CANDIDATE LACKING THE COMMUNITY RENDERING by its number after the candidates when '
            + 'the caller passes the original, and adds nothing without it (owner, 2026-09-09)',
          fn: async () => {
            /** Shared request over one kept and one lost rendering of 自切. */
            const request = {
              task: 'Pick one.',
              criteria: ['Faithful.',],
              evidence: [{
                label: 'ORIGINAL (Chinese)',
                text: '那只猫讲起自切的经历时，朋友们都安静地听着。',
              },],
              rendered: [
                'When the cat told of her self-surgery, her friends listened quietly.',
                'When the cat told of her operation, her friends listened quietly.',
              ],
            };
            /** What the judge is shown with the original passed. */
            const shown = buildCandidateSelectMessages({
              ...request,
              sourceText: '那只猫讲起自切的经历时，朋友们都安静地听着。',
            },).at(1,)?.content ?? '';
            expect(shown,).toContain('COMMUNITY RENDERINGS, evidence to weigh, not a verdict:',);
            expect(shown,).toContain('- CANDIDATE 2 carries none of the community\'s renderings of 自切',);
            expect(shown.includes('CANDIDATE 1 carries none',),).toBe(false,);
            expect(shown.indexOf('COMMUNITY RENDERINGS',) > shown.indexOf('CANDIDATE 2\n',),).toBe(true,);
            /** What the judge is shown without it. */
            const bare = buildCandidateSelectMessages(request,).at(1,)?.content ?? '';
            expect(bare.includes('COMMUNITY RENDERINGS',),).toBe(false,);
          },
        },),

        it({
          name: 'STATES THE FENCE LENGTH IN THE USER TURN AND NOT THE SYSTEM TEXT (R4 of the second review of the '
            + 'Claude '
            + 'route): five signs for a text holding a run of four, six for a run of five, so the system text is the '
            + 'same '
            + 'for every call and the model is still told the true length',
          fn: async () => {
            /**
             The exchange for an original holding a run of the given number of equals signs.

             @param run - length of the run, zero for none

             @returns System text and the user turn

             @example
             ```ts
             const { system, user, } = exchangeWith(4,);
             ```
             */
            function exchangeWith(run: number,): {
              readonly system: string;
              readonly user: string;
            } {
              /**
               The messages the judge is sent.
               */
              const messages = buildCandidateSelectMessages({
                task: 'Pick one.',
                criteria: ['Faithful.',],
                evidence: [{
                  label: 'ORIGINAL (Chinese)',
                  text: `那只猫睡着了。\n${'='.repeat(run,)}`,
                },],
                rendered: [
                  'The cat fell asleep.',
                  'The cat slept.',
                ],
              },);
              return {
                system: messages.at(0,)?.content ?? '',
                user: messages.at(1,)?.content ?? '',
              };
            }
            /**
             The exchange with a run one short of the minimum fence.
             */
            const four = exchangeWith(4,);
            /**
             The exchange with a run as long as the minimum fence.
             */
            const five = exchangeWith(5,);
            expect(four.system,).toBe(five.system,);
            expect(four.system.includes('equals signs',),).toBe(false,);
            expect(
              four.user.startsWith('Every block below opens and closes with a line of 5 equals signs.',),
            ).toBe(true,);
            expect(
              five.user.startsWith('Every block below opens and closes with a line of 6 equals signs.',),
            ).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: isCandidateBallotWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'accepts a ballot naming a candidate by its one-based index',
          fn: async () => {
            expect(
              isCandidateBallotWire({
                best: 1,
                assessment: 'reads most naturally',
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'accepts the decline value, since refusing every candidate is a '
            + 'substantive verdict rather than a failure to answer',
          fn: async () => {
            expect(
              isCandidateBallotWire({
                best: CANDIDATE_NONE,
                assessment: 'none of these fix the dropped clause',
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'accepts an out-of-range index ON PURPOSE. The guard has no upper '
            + 'bound because it cannot know the slate size, and selection counts a '
            + 'ballot naming a nonexistent candidate as an ABSTENTION. Refusing it '
            + 'here would turn that abstention into a lost voice, which changes '
            + 'what a quorum means',
          fn: async () => {
            expect(
              isCandidateBallotWire({
                best: 999,
                assessment: 'the ninth one, whichever that was',
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'accepts an empty reason, so a judge that ranked without '
            + 'explaining still casts a counted vote; the reason is recorded for '
            + 'audit rather than required for validity',
          fn: async () => {
            expect(
              isCandidateBallotWire({
                best: 1,
                assessment: '',
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'ignores extra keys the model volunteered, since a chatty ballot '
            + 'is still a usable one',
          fn: async () => {
            expect(
              isCandidateBallotWire({
                best: 2,
                assessment: 'closer to the original',
                confidence: 0.8,
                runnerUp: 1,
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES a negative index, which is below the decline value and '
            + 'would index from the end of the slate rather than naming a '
            + 'candidate',
          fn: async () => {
            for (const best of [
              -1,
              -999,
            ])
              expect(
                isCandidateBallotWire({
                  best,
                  assessment: 'the last one',
                },),
              ).toBe(false,);
          },
        },),

        it({
          name: 'refuses a fractional or non-finite index, which passes a bare '
            + 'typeof check while naming no candidate at all',
          fn: async () => {
            for (const best of [
              1.5,
              Number.NaN,
              Number.POSITIVE_INFINITY,
              Number.NEGATIVE_INFINITY,
            ])
              expect(
                isCandidateBallotWire({
                  best,
                  assessment: 'somewhere around there',
                },),
              ).toBe(false,);
          },
        },),

        it({
          name: 'refuses a quoted index at the STRICT shape. The as-sent guard is '
            + 'where a canonical quoted index is admitted and read, so nothing '
            + 'downstream of this guard ever compares a string with an index',
          fn: async () => {
            expect(
              isCandidateBallotWire({
                best: '1',
                assessment: 'reads most naturally',
              },),
            ).toBe(false,);
          },
        },),

        it({
          name: 'refuses a missing or non-string reason, so a ballot cannot be '
            + 'counted without the audit trail the selection contract promises',
          fn: async () => {
            expect(isCandidateBallotWire({ best: 1, },),).toBe(false,);
            for (const reason of [
              null,
              7,
              [],
              { text: 'reads most naturally', },
            ])
              expect(
                isCandidateBallotWire({
                  best: 1,
                  assessment: reason,
                },),
              ).toBe(false,);
          },
        },),

        it({
          name: 'refuses a non-record entirely, including null, an array, and a '
            + 'bare number: a model that answered with just the index has not sent '
            + 'a ballot',
          fn: async () => {
            for (const value of [
              null,
              undefined,
              1,
              'candidate 1',
              [
                1,
                'reads most naturally',
              ],
            ])
              expect(isCandidateBallotWire(value,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isCandidateBallotAsSent.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'admits everything the strict guard admits',
          fn: async () => {
            for (const best of [
              CANDIDATE_NONE,
              1,
              999,
            ])
              expect(
                isCandidateBallotAsSent({
                  best,
                  assessment: 'a fine cat',
                },),
              ).toBe(true,);
          },
        },),

        it({
          name: 'admits a quoted canonical index, the shape deepseek-v4-flash-0731 '
            + 'sends about one select round in ten, a quoted decline included',
          fn: async () => {
            for (const best of [
              '0',
              '1',
              '8',
              '12',
            ])
              expect(
                isCandidateBallotAsSent({
                  best,
                  assessment: 'the cat in the window',
                },),
              ).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES every quoted shape that is not the canonical decimal text '
            + 'of a non-negative integer: signs, leading zeros, fractions, exponents, '
            + 'whitespace, hex, fullwidth digits, words and the empty string, each of '
            + 'which Number() or a looser scan would read as some index',
          fn: async () => {
            for (const best of [
              '-1',
              '+1',
              '08',
              '00',
              '1.0',
              '1e0',
              ' 1',
              '1 ',
              '1\n',
              '0x1',
              '１',
              '',
              'one',
            ])
              expect(
                isCandidateBallotAsSent({
                  best,
                  assessment: 'somewhere around there',
                },),
              ).toBe(false,);
          },
        },),

        it({
          name: 'still refuses a missing or non-string reason and a non-record, '
            + 'since widening the index shape widens nothing else',
          fn: async () => {
            expect(isCandidateBallotAsSent({ best: '1', },),).toBe(false,);
            expect(
              isCandidateBallotAsSent({
                best: '1',
                assessment: 7,
              },),
            ).toBe(false,);
            expect(isCandidateBallotAsSent(null,),).toBe(false,);
            expect(isCandidateBallotAsSent('1',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: readCandidateBallotWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads a quoted index as the number it spells, a quoted decline as '
            + 'the decline value, and leaves a numeric index alone, so selection '
            + 'compares numbers with numbers',
          fn: async () => {
            expect(
              readCandidateBallotWire({
                sent: {
                  best: '8',
                  assessment: 'the eighth cat',
                },
              },),
            ).toStrictEqual({
              best: 8,
              assessment: 'the eighth cat',
            },);
            expect(
              readCandidateBallotWire({
                sent: {
                  best: '0',
                  assessment: 'no cat here',
                },
              },),
            ).toStrictEqual({
              best: CANDIDATE_NONE,
              assessment: 'no cat here',
            },);
            expect(
              readCandidateBallotWire({
                sent: {
                  best: 3,
                  assessment: 'the third cat',
                },
              },),
            ).toStrictEqual({
              best: 3,
              assessment: 'the third cat',
            },);
          },
        },),
      ],
    },),
  ],
},);
