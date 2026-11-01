/**
 Tests for the shape guards every artifact reader shares.

 These matter more than their size suggests. They stand at the mouth of the
 precision measurement: each one is the difference between a malformed
 artifact aborting loudly and an accepted issue vanishing from the
 denominator without a trace. So the cases in this file are mostly about what the
 guards REFUSE, not what they pass.

 Fixtures are cat-themed invention mirroring artifact structure only.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ArtifactParseError,
  requireArray,
  requireArtifactParseRefusal,
  requireBoolean,
  requireCount,
  requireRecord,
  requireString,
} from '../dist/final/node/index.mjs';

/**
 Path threaded through every case, so a failure names where it happened the
 way a real parse would.
 */
const PATH = 'Mittens issues[2].issue.status';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: ArtifactParseError.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'names both the path and the expected shape, since a parse failure '
            + 'deep in an artifact is unactionable without them',
          fn: async () => {
            /**
             Failure built directly, standing in for one a guard would throw.
             */
            const error = new ArtifactParseError({
              path: PATH,
              reason: 'a string',
            },);

            expect(error.message,).toBe(
              `artifact parse failed at ${PATH}: expected a string.`,
            );
            expect(error.name,).toBe('ArtifactParseError',);
          },
        },),
      ],
    },),

    describe({
      name: requireArtifactParseRefusal.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS the parse refusal a catch around an artifact reader holds, the same object',
          fn: async () => {
            /**
             Refusal a guard raised.
             */
            const refusal = new ArtifactParseError({
              path: PATH,
              reason: 'a string',
            },);

            expect(requireArtifactParseRefusal({ error: refusal, },),).toBe(refusal,);
          },
        },),
        it({
          name: 'RETHROWS anything else unchanged, an error or not, since an unexpected state must keep propagating',
          fn: async () => {
            /**
             A failure that is not a parse refusal.
             */
            const stray = new TypeError('the cat knocked the artifact off the table',);

            expect(caught(function narrowStray(): void {
              requireArtifactParseRefusal({ error: stray, },);
            },),).toBe(stray,);
            expect(caught(function narrowString(): void {
              requireArtifactParseRefusal({ error: 'hairball', },);
            },),).toBe('hairball',);
          },
        },),
      ],
    },),

    describe({
      name: requireString.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'returns a string unchanged, including the empty one, which is a '
            + 'valid value and not a missing field',
          fn: async () => {
            expect(requireString({
              value: 'accepted',
              path: PATH,
            },),).toBe('accepted',);
            expect(requireString({
              value: '',
              path: PATH,
            },),).toBe('',);
          },
        },),

        it({
          name: 'refuses every non-string, notably a number that would otherwise '
            + 'stringify into a plausible-looking identifier',
          fn: async () => {
            for (const value of [
              7,
              true,
              null,
              undefined,
              {},
              ['accepted',],
            ])
              expect(function readString() {
                requireString({
                  value,
                  path: PATH,
                },);
              },).toThrow(ArtifactParseError,);
          },
        },),
      ],
    },),

    describe({
      name: requireBoolean.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'returns both booleans unchanged, so false is a value rather than '
            + 'an absence',
          fn: async () => {
            expect(requireBoolean({
              value: false,
              path: PATH,
            },),).toBe(false,);
            expect(requireBoolean({
              value: true,
              path: PATH,
            },),).toBe(true,);
          },
        },),

        it({
          name: 'refuses the falsy and truthy look-alikes a JSON writer might emit '
            + 'in a boolean field',
          fn: async () => {
            for (const value of [
              0,
              1,
              'true',
              '',
              null,
              undefined,
            ])
              expect(function readBoolean() {
                requireBoolean({
                  value,
                  path: PATH,
                },);
              },).toThrow(ArtifactParseError,);
          },
        },),
      ],
    },),

    describe({
      name: requireRecord.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'returns an object unchanged, empty included',
          fn: async () => {
            /**
             Record standing in for one issue of an artifact.
             */
            const issue = { status: 'accepted', };

            expect(requireRecord({
              value: issue,
              path: PATH,
            },),).toBe(issue,);
            expect(requireRecord({
              value: {},
              path: PATH,
            },),).toStrictEqual({},);
          },
        },),

        it({
          name: 'REFUSES an array, as `isJsonRecord` does since ledger B92: an array '
            + 'reaching a record reader returns an object whose every named field '
            + 'is undefined, so the real shape error would resurface later as a '
            + 'complaint about a missing property',
          fn: async () => {
            expect(function readArrayAsRecord() {
              requireRecord({
                value: [{ status: 'accepted', },],
                path: PATH,
              },);
            },).toThrow(ArtifactParseError,);
            expect(function readEmptyArrayAsRecord() {
              requireRecord({
                value: [],
                path: PATH,
              },);
            },).toThrow(ArtifactParseError,);
          },
        },),

        it({
          name: 'refuses null, which is typeof object and is exactly the value a '
            + 'dropped field arrives as',
          fn: async () => {
            for (const value of [
              null,
              undefined,
              'accepted',
              7,
              false,
            ])
              expect(function readRecord() {
                requireRecord({
                  value,
                  path: PATH,
                },);
              },).toThrow(ArtifactParseError,);
          },
        },),
      ],
    },),

    describe({
      name: requireArray.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'returns an array unchanged, empty included, since no spans is a '
            + 'real answer and not a malformed one',
          fn: async () => {
            /**
             Array standing in for the spans of a claim.
             */
            const spans = [{ quotedText: '猫猫', },];

            expect(requireArray({
              value: spans,
              path: PATH,
            },),).toBe(spans,);
            expect(requireArray({
              value: [],
              path: PATH,
            },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'refuses an object, so a writer that emitted a keyed map where a '
            + 'list belongs cannot pass silently',
          fn: async () => {
            for (const value of [
              { 0: 'first', },
              'spans',
              null,
              undefined,
              3,
            ])
              expect(function readArray() {
                requireArray({
                  value,
                  path: PATH,
                },);
              },).toThrow(ArtifactParseError,);
          },
        },),
      ],
    },),

    describe({
      name: requireCount.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'returns zero and positive integers, because zero heard probers is '
            + 'a reading rather than a missing one',
          fn: async () => {
            expect(requireCount({
              value: 0,
              path: PATH,
            },),).toBe(0,);
            expect(requireCount({
              value: 6,
              path: PATH,
            },),).toBe(6,);
          },
        },),

        it({
          name: 'separates the two refusals by message: a non-number is reported as '
            + 'expecting a number, while a number of the wrong kind is reported as '
            + 'expecting a non-negative integer, so the diagnostic says which '
            + 'disagreement occurred',
          fn: async () => {
            /**
             What readStringAsCount raised, read for its class as well as its wording.
             */
            const refusalOfReadStringAsCount = caught(function readStringAsCount() {
              requireCount({
                value: '6',
                path: PATH,
              },);
            },);

            expect(refusalOfReadStringAsCount,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfReadStringAsCount as Error).message,).toContain('expected a number',);

            /**
             What readFractionAsCount raised, read for its class as well as its wording.
             */
            const refusalOfReadFractionAsCount = caught(function readFractionAsCount() {
              requireCount({
                value: 2.7,
                path: PATH,
              },);
            },);

            expect(refusalOfReadFractionAsCount,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfReadFractionAsCount as Error).message,).toContain('expected a non-negative integer',);
          },
        },),

        it({
          name: 'refuses a whole number too large for JSON to carry exactly. Above '
            + 'two to the fifty-third, round trips and arithmetic stop being exact, '
            + 'so a value that size is not a tally or an index anybody wrote, however '
            + 'integral it looks to a check that only asks whether it is whole',
          fn: async () => {
            /**
             What readUnsafeCount raised, read for its class as well as its wording.
             */
            const refusalOfReadUnsafeCount = caught(function readUnsafeCount() {
              requireCount({
                value: Number.MAX_SAFE_INTEGER + 2,
                path: PATH,
              },);
            },);

            expect(refusalOfReadUnsafeCount,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfReadUnsafeCount as Error).message,).toContain('no larger than JSON carries exactly',);
            expect(requireCount({
              value: Number.MAX_SAFE_INTEGER,
              path: PATH,
            },),).toBe(Number.MAX_SAFE_INTEGER,);
          },
        },),

        it({
          name: 'refuses a fraction, which is the shape that would mean the writer '
            + 'and reader disagree about what the field holds; every count here '
            + 'tallies votes or regions and cannot be partial',
          fn: async () => {
            for (const value of [
              2.7,
              0.5,
              -1,
              -0.5,
              Number.NaN,
              Number.POSITIVE_INFINITY,
            ])
              expect(function readCount() {
                requireCount({
                  value,
                  path: PATH,
                },);
              },).toThrow(ArtifactParseError,);
          },
        },),

        it({
          name: 'refuses every non-number, including the numeric string a JSON '
            + 'writer produces when a field was quoted by mistake',
          fn: async () => {
            for (const value of [
              '6',
              null,
              undefined,
              true,
              {},
              [6,],
            ])
              expect(function readCount() {
                requireCount({
                  value,
                  path: PATH,
                },);
              },).toThrow(ArtifactParseError,);
          },
        },),
      ],
    },),
  ],
},);
