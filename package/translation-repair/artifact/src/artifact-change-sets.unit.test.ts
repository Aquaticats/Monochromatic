/**
 Tests for reading a settled artifact's schema generation and index sets.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  ARTIFACT_SCHEMA_VERSION_V1,
  ARTIFACT_SCHEMA_VERSION_V2,
  KNOWN_ARTIFACT_SCHEMA_VERSIONS,
  parseSettledArtifact,
  readArtifactChangeSets,
  readArtifactSchemaVersion,
} from '../dist/final/node/index.mjs';

/**
 Reads change sets out of one artifact record, returning whatever it threw.

 @param artifact - artifact record under test

 @returns Failure it raised, or `undefined` when it accepted the record

 @example
 ```ts
 const refusal = changeSetFailure({ artifact: { shippedChunkIndices: [], }, },);
 ```
 */
function changeSetFailure(
  { artifact, }: { readonly artifact: Readonly<Record<string, unknown>>; },
): unknown {
  try {
    readArtifactChangeSets({
      artifact,
      path: 'Mittens',
    },);
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Artifact as the pass writes one today, with both sets and their count.
 */
const VERSIONED_ARTIFACT = {
  artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V1,
  sliceCount: 5,
  shippedChunkIndices: [
    3,
    0,
  ],
  withdrawnChunkIndices: [4,],
} as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readArtifactSchemaVersion.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads an artifact with no version field as UNVERSIONED rather than as version zero, which is '
            + 'the whole reason the reading is a union: every artifact settled to date is one of these',
          fn: async () => {
            expect(readArtifactSchemaVersion({
              artifact: { id: 'Mittens', },
              path: 'Mittens',
            },).kind,).toBe('unversioned',);
          },
        },),
        it({
          name: 'reads the generation the pass writes today',
          fn: async () => {
            expect(readArtifactSchemaVersion({
              artifact: VERSIONED_ARTIFACT,
              path: 'Mittens',
            },),).toEqual({
              kind: 'versioned',
              version: ARTIFACT_SCHEMA_VERSION_V1,
            },);
          },
        },),
        it({
          name: 'REFUSES a generation written after this reader was compiled, rather than parsing it on the '
            + 'assumption that fields it knows still mean what they did: an instrument that reports a wrong '
            + 'number is worse than one that reports none',
          fn: async () => {
            /**
             What readFutureVersion raised, read for its class as well as its wording.
             */
            const refusalOfReadFutureVersion = caught(function readFutureVersion() {
              readArtifactSchemaVersion({
                // One past the newest version this reader knows, taken from the
                // list rather than written as a literal, so it stays a FUTURE
                // version as versions are added.
                artifact: { artifactSchemaVersion: Math.max(...KNOWN_ARTIFACT_SCHEMA_VERSIONS,) + 1, },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfReadFutureVersion,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfReadFutureVersion as Error).message,).toContain('this reader knows',);
          },
        },),
        it({
          name: 'ACCEPTS version 2 as a generation it KNOWS, which is what separates dispatching from '
            + 'reading: naming the generation is this function`s whole job, and whether a given reader can '
            + 'answer for that generation is the reader`s to say',
          fn: async () => {
            expect(readArtifactSchemaVersion({
              artifact: { artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V2, },
              path: 'Mittens',
            },),).toEqual({
              kind: 'versioned',
              version: ARTIFACT_SCHEMA_VERSION_V2,
            },);
          },
        },),
        it({
          name: 'REFUSES a version 2 artifact rather than answering with one change set, because a two-lane '
            + 'artifact records a ledger PER LANE and has no singular anything to answer with: reading its '
            + 'absent top-level sets as unrecorded would report a run that changed nothing',
          fn: async () => {
            /**
             What changeSetReadOfVersionTwo raised, read for its class as well as its wording.
             */
            const refusalOfChangeSetRead = caught(function changeSetReadOfVersionTwo() {
              readArtifactChangeSets({
                artifact: {
                  artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V2,
                  id: 'Mittens',
                },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfChangeSetRead,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfChangeSetRead as Error).message,).toContain('one change set',);
          },
        },),
        it({
          name: 'REFUSES version zero as firmly as a future one. Zero is what a reader that defaulted an '
            + 'absent field would have invented, so accepting it would let the invention back in through '
            + 'the field itself',
          fn: async () => {
            /**
             What readZeroVersion raised, read for its class as well as its wording.
             */
            const refusalOfReadZeroVersion = caught(function readZeroVersion() {
              readArtifactSchemaVersion({
                artifact: { artifactSchemaVersion: 0, },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfReadZeroVersion,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfReadZeroVersion as Error).message,).toContain('this reader knows',);
          },
        },),
        it({
          name: 'REFUSES a version that is not a count at all, before comparing it to anything: a string, a '
            + 'fraction and a negative each mean the writer and this reader disagree about the field',
          fn: async () => {
            /**
             What readStringVersion raised, read for its class as well as its wording.
             */
            const refusalOfReadStringVersion = caught(function readStringVersion() {
              readArtifactSchemaVersion({
                artifact: { artifactSchemaVersion: '1', },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfReadStringVersion,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfReadStringVersion as Error).message,).toContain('a number',);
            /**
             What readFractionalVersion raised, read for its class as well as its wording.
             */
            const refusalOfReadFractionalVersion = caught(function readFractionalVersion() {
              readArtifactSchemaVersion({
                artifact: { artifactSchemaVersion: 1.5, },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfReadFractionalVersion,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfReadFractionalVersion as Error).message,).toContain('a non-negative integer',);
            /**
             What readNegativeVersion raised, read for its class as well as its wording.
             */
            const refusalOfReadNegativeVersion = caught(function readNegativeVersion() {
              readArtifactSchemaVersion({
                artifact: { artifactSchemaVersion: -1, },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfReadNegativeVersion,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfReadNegativeVersion as Error).message,).toContain('a non-negative integer',);
          },
        },),
      ],
    },),

    describe({
      name: readArtifactChangeSets.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads an artifact that recorded neither set as UNRECORDED, carrying neither array: a run '
            + 'nobody wrote index sets for must never read as a run that changed nothing, and an empty array '
            + 'is exactly how those two become indistinguishable',
          fn: async () => {
            /** Reading over an artifact from before index recording existed. */
            const sets = readArtifactChangeSets({
              artifact: {
                id: 'Mittens',
                status: 'repaired',
              },
              path: 'Mittens',
            },);
            expect(sets,).toEqual({ kind: 'unrecorded', },);
            expect(Object.hasOwn(sets, 'shipped',),).toBe(false,);
            expect(Object.hasOwn(sets, 'withdrawn',),).toBe(false,);
          },
        },),
        it({
          name: 'reads both sets of a versioned artifact ascending, bounded by the count it records',
          fn: async () => {
            expect(readArtifactChangeSets({
              artifact: VERSIONED_ARTIFACT,
              path: 'Mittens',
            },),).toEqual({
              kind: 'counted',
              sliceCount: 5,
              shipped: [
                0,
                3,
              ],
              withdrawn: [4,],
            },);
          },
        },),
        it({
          name: 'reads two EMPTY recorded sets as a run that genuinely changed nothing, whether or not it '
            + 'was versioned. This is the reading the unknown must never be folded into, so it has to be '
            + 'reachable in its own right',
          fn: async () => {
            expect(readArtifactChangeSets({
              artifact: {
                artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V1,
                sliceCount: 3,
                shippedChunkIndices: [],
                withdrawnChunkIndices: [],
              },
              path: 'Mittens',
            },),).toEqual({
              kind: 'counted',
              sliceCount: 3,
              shipped: [],
              withdrawn: [],
            },);
            expect(readArtifactChangeSets({
              artifact: {
                shippedChunkIndices: [],
                withdrawnChunkIndices: [],
              },
              path: 'Mittens',
            },),).toEqual({
              kind: 'uncounted',
              shipped: [],
              withdrawn: [],
            },);
          },
        },),
        it({
          name: 'reads an artifact that recorded both sets WITHOUT a slice count as uncounted, keeping every '
            + 'rule a count is not needed for on BOTH sets: the 2026-08-15 generation wrote the arrays before '
            + 'anything wrote their denominator, and skipping all of its checks for want of one would accept '
            + 'a repeat or an overlap it can plainly see',
          fn: async () => {
            expect(readArtifactChangeSets({
              artifact: {
                shippedChunkIndices: [
                  9,
                  2,
                ],
                withdrawnChunkIndices: [],
              },
              path: 'Mittens',
            },),).toEqual({
              kind: 'uncounted',
              shipped: [
                2,
                9,
              ],
              withdrawn: [],
            },);
            /**
             What repeatedShipped raised, read for its class as well as its wording.
             */
            const refusalOfRepeatedShipped = caught(function repeatedShipped() {
              readArtifactChangeSets({
                artifact: {
                  shippedChunkIndices: [
                    2,
                    2,
                  ],
                  withdrawnChunkIndices: [],
                },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfRepeatedShipped,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfRepeatedShipped as Error).message,).toContain('shipped slices repeat',);
            /**
             What repeatedWithdrawn raised, read for its class as well as its wording.
             */
            const refusalOfRepeatedWithdrawn = caught(function repeatedWithdrawn() {
              readArtifactChangeSets({
                artifact: {
                  shippedChunkIndices: [],
                  withdrawnChunkIndices: [
                    4,
                    4,
                  ],
                },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfRepeatedWithdrawn,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfRepeatedWithdrawn as Error).message,).toContain('withdrawn slices repeat',);
          },
        },),
        it({
          name: 'REFUSES one index set without the other, whichever half is missing: every generation wrote '
            + 'both or neither, so one alone means the record was edited or truncated, and reading it would '
            + 'report a shipped set with no withdrawals as though a run had said so',
          fn: async () => {
            /**
             What shippedAlone raised, read for its class as well as its wording.
             */
            const refusalOfShippedAlone = caught(function shippedAlone() {
              readArtifactChangeSets({
                artifact: { shippedChunkIndices: [1,], },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfShippedAlone,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfShippedAlone as Error).message,).toContain('both index sets or neither',);
            /**
             What withdrawnAlone raised, read for its class as well as its wording.
             */
            const refusalOfWithdrawnAlone = caught(function withdrawnAlone() {
              readArtifactChangeSets({
                artifact: { withdrawnChunkIndices: [1,], },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfWithdrawnAlone,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfWithdrawnAlone as Error).message,).toContain('both index sets or neither',);
          },
        },),
        it({
          name: 'REFUSES an unversioned artifact that records a slice count, which no writer ever produced. '
            + 'What produces it is a CURRENT artifact whose version field was lost to an edit or a merge, and '
            + 'reading that as an older generation would throw away a denominator the run recorded',
          fn: async () => {
            /**
             What countWithoutVersion raised, read for its class as well as its wording.
             */
            const refusalOfCountWithoutVersion = caught(function countWithoutVersion() {
              readArtifactChangeSets({
                artifact: {
                  sliceCount: 2,
                  shippedChunkIndices: [],
                  withdrawnChunkIndices: [],
                },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfCountWithoutVersion,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfCountWithoutVersion as Error).message,).toContain('records the slice count that arrived with one',);
          },
        },),
        it({
          name: 'treats an explicit null as PRESENT and refuses it, rather than reading it as the absence of a '
            + 'field: a writer that emitted null said something, and what it said is not a set of indices',
          fn: async () => {
            /**
             What nullShipped raised, read for its class as well as its wording.
             */
            const refusalOfNullShipped = caught(function nullShipped() {
              readArtifactChangeSets({
                artifact: {
                  shippedChunkIndices: null,
                  withdrawnChunkIndices: [],
                },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfNullShipped,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNullShipped as Error).message,).toContain('an array',);
          },
        },),
        it({
          name: 'REFUSES a versioned artifact that omits the sets or their count, since the version is what '
            + 'promises them: a missing field is a defect there rather than an older generation',
          fn: async () => {
            /**
             What versionedWithoutSets raised, read for its class as well as its wording.
             */
            const refusalOfVersionedWithoutSets = caught(function versionedWithoutSets() {
              readArtifactChangeSets({
                artifact: {
                  artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V1,
                  sliceCount: 2,
                },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfVersionedWithoutSets,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfVersionedWithoutSets as Error).message,).toContain('index sets',);
            /**
             What versionedWithoutCount raised, read for its class as well as its wording.
             */
            const refusalOfVersionedWithoutCount = caught(function versionedWithoutCount() {
              readArtifactChangeSets({
                artifact: {
                  artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V1,
                  shippedChunkIndices: [0,],
                  withdrawnChunkIndices: [],
                },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfVersionedWithoutCount,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfVersionedWithoutCount as Error).message,).toContain('sliceCount',);
          },
        },),
        it({
          name: 'REFUSES an index outside the slices a versioned artifact says it prepared, on either side, '
            + 'and reports it as a defect of the ARTIFACT naming the entry. A reader holding a file cannot '
            + 'know whether the run, an edit or a truncation put it there, so it says what the file contains '
            + 'rather than who broke the contract',
          fn: async () => {
            /** Failure the out-of-range shipped index raised. */
            const refusalOfBothOutOfRange = changeSetFailure({
              artifact: {
                artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V1,
                sliceCount: 2,
                shippedChunkIndices: [2,],
                withdrawnChunkIndices: [],
              },
            },);
            expect(refusalOfBothOutOfRange,).toBeInstanceOf(ArtifactParseError,);
            expect(String(refusalOfBothOutOfRange,),).toContain('Mittens index sets',);
            expect(String(refusalOfBothOutOfRange,),).toContain('of 2 prepared',);
            /**
             What withdrawnOutOfRange raised, read for its class as well as its wording.
             */
            const refusalOfWithdrawnOutOfRange = caught(function withdrawnOutOfRange() {
              readArtifactChangeSets({
                artifact: {
                  artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V1,
                  sliceCount: 2,
                  shippedChunkIndices: [],
                  withdrawnChunkIndices: [7,],
                },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfWithdrawnOutOfRange,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfWithdrawnOutOfRange as Error).message,).toContain('of 2 prepared',);
          },
        },),
        it({
          name: 'REFUSES a slice recorded as both shipped and withdrawn, which the writing lanes call '
            + 'impossible by construction: found in a file afterwards it describes the same contradiction',
          fn: async () => {
            /**
             What overlapping raised, read for its class as well as its wording.
             */
            const refusalOfOverlapping = caught(function overlapping() {
              readArtifactChangeSets({
                artifact: {
                  artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V1,
                  sliceCount: 4,
                  shippedChunkIndices: [
                    1,
                    3,
                  ],
                  withdrawnChunkIndices: [3,],
                },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfOverlapping,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfOverlapping as Error).message,).toContain('both shipped and withdrawn',);
          },
        },),
        it({
          name: 'REFUSES an entry that is not a slice index, naming the position it sits at, and refuses one '
            + 'too large for JSON to carry exactly however whole it looks',
          fn: async () => {
            /**
             What fractionalIndex raised, read for its class as well as its wording.
             */
            const refusalOfFractionalIndex = caught(function fractionalIndex() {
              readArtifactChangeSets({
                artifact: {
                  shippedChunkIndices: [
                    0,
                    1.5,
                  ],
                  withdrawnChunkIndices: [],
                },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfFractionalIndex,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfFractionalIndex as Error).message,).toContain('shippedChunkIndices[1]',);
            /**
             What unsafeIndex raised, read for its class as well as its wording.
             */
            const refusalOfUnsafeIndex = caught(function unsafeIndex() {
              readArtifactChangeSets({
                artifact: {
                  shippedChunkIndices: [Number.MAX_SAFE_INTEGER + 2,],
                  withdrawnChunkIndices: [],
                },
                path: 'Mittens',
              },);
            },);

            expect(refusalOfUnsafeIndex,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfUnsafeIndex as Error).message,).toContain('no larger than JSON carries exactly',);
          },
        },),
      ],
    },),

    describe({
      name: `${parseSettledArtifact.name} change sets`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'carries the unknown through to the parsed artifact, so a consumer counting shipped slices over '
            + 'a directory of older runs cannot mistake them for runs that shipped none',
          fn: async () => {
            expect(parseSettledArtifact({
              value: {
                id: 'Mittens',
                status: 'unchanged',
                issues: [],
              },
            },).changeSets
              .kind,).toBe('unrecorded',);
          },
        },),
        it({
          name: 'reads the sets of an artifact the pass wrote under the current schema',
          fn: async () => {
            expect(parseSettledArtifact({
              value: {
                ...VERSIONED_ARTIFACT,
                id: 'Mittens',
                status: 'repaired',
                issues: [],
              },
            },).changeSets,).toEqual({
              kind: 'counted',
              sliceCount: 5,
              shipped: [
                0,
                3,
              ],
              withdrawn: [4,],
            },);
          },
        },),
        it({
          name: 'REFUSES a generation from the future through the parser too, since a consumer scanning a '
            + 'directory meets artifacts a newer pass wrote there rather than hand-built records',
          fn: async () => {
            /**
             What parseFutureArtifact raised, read for its class as well as its wording.
             */
            const refusalOfParseFutureArtifact = caught(function parseFutureArtifact() {
              parseSettledArtifact({
                value: {
                  ...VERSIONED_ARTIFACT,
                  artifactSchemaVersion: Math.max(...KNOWN_ARTIFACT_SCHEMA_VERSIONS,) + 1,
                  id: 'Mittens',
                  status: 'repaired',
                  issues: [],
                },
              },);
            },);

            expect(refusalOfParseFutureArtifact,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfParseFutureArtifact as Error).message,).toContain('this reader knows',);
          },
        },),
        it({
          name: 'REFUSES a VERSION 2 artifact through the parser rather than reading its version 1 fields, '
            + 'which is the difference between a version this reader knows and one it can answer for: the '
            + 'top-level status and issues a version 2 artifact does not have would come back as absences',
          fn: async () => {
            /**
             What singleLaneParseOfVersionTwo raised, read for its class as well as its wording.
             */
            const refusalOfSingleLaneParse = caught(function singleLaneParseOfVersionTwo() {
              parseSettledArtifact({
                value: {
                  ...VERSIONED_ARTIFACT,
                  artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V2,
                  id: 'Mittens',
                  status: 'repaired',
                  issues: [],
                },
              },);
            },);

            expect(refusalOfSingleLaneParse,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfSingleLaneParse as Error).message,).toContain('one change set',);
          },
        },),
      ],
    },),
  ],
},);
