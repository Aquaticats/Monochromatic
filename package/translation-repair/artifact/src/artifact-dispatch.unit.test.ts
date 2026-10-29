/**
 Tests for choosing which generation's reader an artifact belongs to.

 WHAT THESE PIN is that the answer names the generation. A reader that
 returned one merged shape would have to pick a lane for a version 2 artifact,
 which is the question nobody has decided, and a caller that cannot tell which
 generation it is holding cannot ask a question either one can answer.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  ARTIFACT_SCHEMA_VERSION_V1,
  KNOWN_ARTIFACT_SCHEMA_VERSIONS,
  readSettledArtifact,
  TWO_LANE_GENERATIONS,
} from '../dist/final/node/index.mjs';

/**
 Archive wording every fixture here shares.
 */
const ARCHIVE_NAP = 'The cat sleeps on the sill.';

/**
 A version 1 artifact, which records one lane at the top level.

 @returns Artifact as JSON

 @example
 ```ts
 const artifact = versionOneArtifact();
 ```
 */
function versionOneArtifact(): Record<string, unknown> {
  return {
    artifactSchemaVersion: 1,
    id: 'CatEntry1',
    status: 'unchanged',
    issues: [],

    // BOTH SETS or neither, which is what that generation wrote and what its
    // reader holds artifacts to, with the slice count those sets are out of.
    sliceCount: 2,
    shippedChunkIndices: [],
    withdrawnChunkIndices: [],
  };
}

await describe({
  name: readSettledArtifact.name,
  children: [
    it({
      name:
        'names the generation it read, so a caller holding the answer knows which questions it can ask: '
          + 'version 1 answers with one status and one issue list, and version 2 has two lanes and no '
          + 'singular anything',
      fn: async () => {
        /**
         A version 1 artifact, read.
         */
        const versioned = readSettledArtifact({ value: versionOneArtifact(), },);
        expect(versioned.kind,).toBe('version-1',);
        expect(versioned.artifact
          .id,).toBe('CatEntry1',);
      },
    },),
    it({
      name:
        'reads an artifact with NO version field as legacy rather than guessing a generation from which '
          + 'fields it happens to carry, since the absence of the field is itself the fact',
      fn: async () => {
        // NOT version 1 with its version field removed, which the reader
        // rightly refuses: the slice count and the two index sets ARRIVED with
        // the version field, so an artifact carrying them and no version is a
        // file nobody wrote rather than an older one.
        expect(readSettledArtifact({
          value: {
            id: 'CatEntry1',
            status: 'unchanged',
            issues: [],
          },
        },).kind,).toBe('legacy',);
      },
    },),
    it({
      name:
        'REFUSES a version 2 SHAPE carrying no version field, rather than recognising it by its lanes: '
          + 'the version field is what says which generation wrote a file, and a reader that inferred a '
          + 'generation from shape would read the next generation as whichever old one it resembles',
      fn: async () => {
        /**
         What versionTwoShapeWithoutVersion raised, read for its class as well as its wording.
         */
        const refusalOfVersionTwoShapeWithoutVersion = caught(function versionTwoShapeWithoutVersion() {
          readSettledArtifact({
            value: {
              id: 'CatEntry1',
              preparation: {
                identity: `sha256-preparation-v1:${'a7'.repeat(32,)}`,
                sliceCount: 1,
              },
              lanes: {
                repair: {
                  result: {},
                  delivery: [],
                },
                translate: {
                  result: {},
                  delivery: [],
                },
              },
              comparison: [],
            },
          },);
        },);

        expect(refusalOfVersionTwoShapeWithoutVersion,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfVersionTwoShapeWithoutVersion as Error).message,).toContain('status',);
      },
    },),
    it({
      name:
        'ACCEPTS an explicit version 1, which is a different question from whether any version 1 '
          + 'artifacts exist: how many files of a generation a corpus holds is a fact about that corpus, '
          + 'and a reader that understands a generation should read it',
      fn: async () => {
        expect(readSettledArtifact({
          value: {
            ...versionOneArtifact(),
            artifactSchemaVersion: 1,
          },
        },).kind,).toBe('version-1',);
      },
    },),
    it({
      name:
        'HANDS EVERY TWO-LANE GENERATION to that reader, not just the first of them. This named version '
          + '2 alone once, and generation 3 spent its whole life reported here as a generation nothing '
          + 'reads: the reader had learned it and the dispatch had not',
      fn: async () => {
        // READ FROM THE LIST THE READER ITSELF ACCEPTS, so a fourth generation
        // cannot be added to one and forgotten by the other without this
        // failing. A body of the wrong SHAPE is fine and is the point: what is
        // under test is WHICH reader answered, and the two-lane reader saying
        // the body is wrong proves it ran.
        expect(TWO_LANE_GENERATIONS.map(function answeredBy(version,): string {
          /**
           What the dispatch did with a version 1 body under this stamp.
           */
          const refusal = caught(function readsUnderStamp() {
            readSettledArtifact({
              value: {
                ...versionOneArtifact(),
                artifactSchemaVersion: version,
              },
            },);
          },);

          return ((refusal as Error).message.includes('CatEntry1.status',))
            ? 'the two-lane reader'
            : (refusal as Error).message;
        },),).toEqual(TWO_LANE_GENERATIONS.map(function everyOne(): string {
          return 'the two-lane reader';
        },),);
      },
    },),
    it({
      name:
        'GIVES EVERY GENERATION THIS BUILD KNOWS A READER, version 1 or the two-lane one, so the refusal of a '
          + 'known generation nothing reads is reached only when a generation joins the known list alone',
      fn: async () => {
        expect(KNOWN_ARTIFACT_SCHEMA_VERSIONS.filter(function unread(version,): boolean {
          return (version !== ARTIFACT_SCHEMA_VERSION_V1) && (!TWO_LANE_GENERATIONS.includes(version,));
        },),).toEqual([],);
      },
    },),
    it({
      name:
        'REFUSES a generation written after this build, because a reader meeting one knows exactly one '
          + 'thing about the file: that it does not know its shape',
      fn: async () => {
        /**
         One past every generation this build knows, derived rather than typed
         so adding a generation does not quietly make this case vacuous.
         */
        const unknownVersion = Math.max(...KNOWN_ARTIFACT_SCHEMA_VERSIONS,) + 1;
        /**
         What laterGeneration raised, read for its class as well as its wording.
         */
        const refusalOfLaterGeneration = caught(function laterGeneration() {
          readSettledArtifact({
            value: {
              ...versionOneArtifact(),
              artifactSchemaVersion: unknownVersion,
            },
          },);
        },);

        expect(refusalOfLaterGeneration,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfLaterGeneration as Error).message,).toContain('artifactSchemaVersion',);
      },
    },),
    it({
      name:
        'REFUSES a version field that is PRESENT and not a version, rather than reading it as absent: '
          + 'a null or a version spelled as text is a file this reader cannot account for, and treating it '
          + 'as legacy would answer with a generation nobody wrote',
      fn: async () => {
        expect([
          null,
          '1',
          1.5,
        ].map(function refuses(version,): string {
          try {
            readSettledArtifact({
              value: {
                ...versionOneArtifact(),
                artifactSchemaVersion: version,
              },
            },);
            return 'accepted';
          } catch (error) {
            return Error.isError(error,) ? error.name : 'threw a non-error';
          }
        },),).toEqual([
          'ArtifactParseError',
          'ArtifactParseError',
          'ArtifactParseError',
        ],);
      },
    },),
    it({
      name:
        'REFUSES anything that is not a record at all, at the outermost path, so a file holding a bare '
          + 'string reports its shape rather than a missing field',
      fn: async () => {
        /**
         What notARecord raised, read for its class as well as its wording.
         */
        const refusalOfNotARecord = caught(function notARecord() {
          readSettledArtifact({ value: `"${ARCHIVE_NAP}"`, },);
        },);

        expect(refusalOfNotARecord,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfNotARecord as Error).message,).toContain('artifact',);
      },
    },),
  ],
},);
