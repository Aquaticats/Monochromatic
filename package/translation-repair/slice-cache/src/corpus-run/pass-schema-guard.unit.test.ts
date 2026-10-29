/**
 Tests for the resume guard that keeps one accumulation at one artifact shape.

 WHAT THIS EXISTS FOR is one narrow case, and the tests say which. The
 pipeline guard already refuses an ordinary mixed-generation resume, because a
 build writing one artifact shape cannot share a digest with a build writing
 another. Its drift opt-in is what lets a mixed directory through, on a promise
 that a rate over the pool stays usable once it names a required commit. That
 promise holds across BUILDS and not across SHAPES: a version 1 artifact
 cannot answer a two-lane question at any commit.

 AND ONE CASE THE FIRST VERSION MISSED, which an independent review found: the
 guard read the version LABEL and never the body, so a version 1 artifact
 relabelled as the generation this pass writes passed it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertArtifactsPlaceable,
  assertBuildGenerationResumable,
  assertResumableSchemaGeneration,
  censusBySchema,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 One built pipeline, as a digest-shaped invention.
 */
const DIGEST_A = `sha256-tree-v1:${'a'.repeat(64,)}`;

/**
 A second built pipeline, differing from {@link DIGEST_A} everywhere.
 */
const DIGEST_B = `sha256-tree-v1:${'b'.repeat(64,)}`;

/**
 Commit every fixture artifact records, since nothing here turns on provenance.
 */
const FIXED_TIP = '1111111111111111111111111111111111111111';

/**
 Preparation identity every fixture claims, syntactically valid and describing
 nothing, which is all a standalone reader checks.
 */
const PREPARATION_IDENTITY = `sha256-preparation-v1:${'a7'.repeat(32,)}`;

/**
 What one fixture artifact records about its generation.
 */
type Fixture = {
  /**
   Schema generation it names, absent when it carries no version field.
   */
  readonly version?: number;

  /**
   Built pipeline it records.
   */
  readonly digest: string;

  /**
   Whether the body must satisfy the generation it names, rather than merely
   carrying the label.
   */
  readonly wellFormed?: boolean;
};

/**
 A complete version 2 artifact describing a document with NO slices.

 EMPTY ON PURPOSE. Every per-slice relation the reader runs is vacuous here,
 so this is the smallest body that genuinely satisfies the generation rather
 than merely claiming it, which is what these cases need to tell a real
 artifact from a relabelled one.

 @param entryId - entry it settles

 @param digest - built pipeline it records

 @returns Artifact as JSON

 @example
 ```ts
 const artifact = emptyVersionTwoArtifact({ entryId: 'Mittens', digest: DIGEST_A, },);
 ```
 */
function emptyVersionTwoArtifact(
  {
    entryId,
    digest,
  }: {
    readonly entryId: string;
    readonly digest: string;
  },
): Record<string, unknown> {
  return {
    artifactSchemaVersion: 22,
    id: entryId,
    tip: FIXED_TIP,
    pipelineDigest: digest,
    corpusSha: 'b'.repeat(40,),
    callConfig: { perCallTimeoutMs: 600_000, },
    durationMs: 40,
    timestamp: '2026-08-17T04:00:00.000Z',
    preparation: {
      identity: PREPARATION_IDENTITY,
      sliceCount: 0,
      sourceChars: 0,
      targetChars: 0,
      sourceBytes: 0,
      alignmentPairCount: 0,
      alignmentFindings: [],
    },
    lanes: {
      repair: {
        result: {
          status: 'unchanged',
          sliceCount: 0,
          changedSliceIndices: [],
          withdrawnSliceIndices: [],
          findings: [],
          sliceTexts: [],
        },
        delivery: [],
      },
      translate: {
        result: {
          status: 'complete',
          sliceCount: 0,
          changedSliceCount: 0,
          refusedSliceCount: 0,
          withdrawnSliceCount: 0,
          changedSliceIndices: [],
          withdrawnSliceIndices: [],
          sliceTexts: [],
        },
        delivery: [],
      },
    },
    comparison: [],
    laneSelection: { kind: 'pending-human-decision', },
    consolidation: { kind: 'not-run', },
    pageAssembly: { trimmed: [], withdrawn: [], findings: [], },
    consistency: { groups: [], slices: [], findings: [], },
  };
}

/**
 Writes artifacts into a caller-owned directory.

 @param dir - case-owned directory to write into

 @param entries - one fixture per entry id

 @returns Nothing; the files are the effect

 @example
 ```ts
 await writeArtifacts({ dir: scratch.path, entries: { Mittens: { version: 2, digest: DIGEST_A, }, }, },);
 ```
 */
async function writeArtifacts(
  { dir, entries, }: {
    readonly dir: string;
    readonly entries: Readonly<Record<string, Fixture>>;
  },
): Promise<void> {

  await Promise.all(
    Object.entries(entries,)
      .map(async function writeOne([
        entryId,
        {
          version,
          digest,
          wellFormed = false,
        },
      ],): Promise<void> {
        /**
         Fields every fixture carries, whatever generation it claims: these are
         what the PIPELINE guard reads, so a case can reach the schema guard.
         */
        const common = {
          id: entryId,
          tip: FIXED_TIP,
          pipelineDigest: digest,
        };

        /**
         Body this fixture writes: a real version 2 artifact when the case
         needs one, and otherwise the label alone over a body that is not one.
         */
        const body = wellFormed
          ? emptyVersionTwoArtifact({
            entryId,
            digest,
          },)
          : {
            ...common,
            ...((version === undefined) ? {} : { artifactSchemaVersion: version, }),
          };

        await writeFile(
          join(
            dir,
            `${entryId}.json`,
          ),
          JSON.stringify(body,),
          'utf8',
        );
      },),
  );
}

/**
 Runs the guard and reports what it said, or that it accepted.

 @param artifactsDir - directory to check

 @returns Refusal text, or a sentinel no assertion here matches

 @example
 ```ts
 const said = await refusalOf({ artifactsDir, },);
 ```
 */
async function refusalOf(
  { artifactsDir, }: { readonly artifactsDir: string; },
): Promise<string> {
  try {
    await assertResumableSchemaGeneration({ artifactsDir, },);
    return 'the guard accepted it';
  } catch (error) {
    return caughtValueText(error,);
  }
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertResumableSchemaGeneration.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'ACCEPTS a fresh directory and one holding real artifacts of the generation this pass writes, '
              + 'which are the two ordinary cases and the only ones that must stay silent',
          fn: async () => {
            await using emptyScratch = await scratchDir({ prefix: 'pass-schema-guard-', },);
            await writeArtifacts({ dir: emptyScratch.path, entries: {}, },);
            await assertResumableSchemaGeneration({ artifactsDir: emptyScratch.path, },);

            await using filledScratch = await scratchDir({ prefix: 'pass-schema-guard-', },);
            await writeArtifacts({
              dir: filledScratch.path,
              entries: {
                Mittens: {
                  version: 5,
                  digest: DIGEST_A,
                  wellFormed: true,
                },
                Pouncer: {
                  version: 5,
                  digest: DIGEST_A,
                  wellFormed: true,
                },
              },
            },);
            await assertResumableSchemaGeneration({ artifactsDir: filledScratch.path, },);
          },
        },),
        it({
          name:
            'REFUSES a body that DECLARES this generation and is not one, which the first version of this '
              + 'guard accepted: it read the label and never the body, so a version 1 artifact relabelled as '
              + 'the generation this pass writes was counted as settled, never re-run, and left for whichever '
              + 'reader asked it a two-lane question first',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'pass-schema-guard-', },);
            /**
             A version 1 body carrying this generation's label and nothing else
             of that generation.
             */
            const artifactsDir = scratch.path;
            await writeArtifacts({
              dir: artifactsDir,
              entries: {
                Mittens: {
                  version: 22,
                  digest: DIGEST_A,
                },
              },
            },);

            /**
             What the guard said about it.
             */
            const said = await refusalOf({ artifactsDir, },);
            expect(said,).toContain('Mittens declares schema version 22',);
            expect(said,).toContain('and is not one',);
          },
        },),
        it({
          name:
            'REFUSES a version 1 artifact that the drift opt-in just waved past, which is the whole reason '
              + 'this guard is separate: the pipeline guard accepts that directory once an operator opts in, '
              + 'and the promise it made them, that naming a required commit keeps the pool readable, is a '
              + 'promise about builds rather than about shapes',
          fn: async () => {
            /**
             A directory holding one artifact of each generation, both stamped with
             pipelines this invocation is not.
             */
            await using scratch = await scratchDir({ prefix: 'pass-schema-guard-', },);
            const artifactsDir = scratch.path;
            await writeArtifacts({
              dir: artifactsDir,
              entries: {
                Mittens: {
                  version: 1,
                  digest: DIGEST_A,
                },
                Pouncer: {
                  version: 5,
                  digest: DIGEST_A,
                  wellFormed: true,
                },
              },
            },);

            // POSITIVE CONTROL, and the point of the case: the pipeline guard's two
            // halves, run as the pass runs them with the opt-in asked for, are
            // silent here. Without it this would be checking a directory two guards
            // refuse and proving nothing about which one did the work.
            assertBuildGenerationResumable({
              census: await assertArtifactsPlaceable({ artifactsDir, },),
              digest: DIGEST_B,
              driftAllowed: true,
            },);
            expect(await refusalOf({ artifactsDir, },),).toContain('schema version 1: 1 settled, Mittens',);
          },
        },),
        it({
          name:
            'ACCEPTS a directory of several BUILDS all writing this generation, so the guard is not a second '
              + 'digest check: an operator who opted into build drift keeps exactly what they opted into',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'pass-schema-guard-', },);
            await writeArtifacts({
              dir: scratch.path,
              entries: {
                Mittens: {
                  version: 5,
                  digest: DIGEST_A,
                  wellFormed: true,
                },
                Pouncer: {
                  version: 5,
                  digest: DIGEST_B,
                  wellFormed: true,
                },
              },
            },);
            await assertResumableSchemaGeneration({ artifactsDir: scratch.path, },);
          },
        },),
        it({
          name:
            'REFUSES an artifact carrying NO version field, which is the generation the pipeline guard is '
              + 'least likely to catch: those files record a digest, so they are neither unplaceable nor '
              + 'legacy, and one written by this very build would pass every check but this one',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'pass-schema-guard-', },);
            await writeArtifacts({
              dir: scratch.path,
              entries: {
                // No `version` key at all rather than one holding `undefined`,
                // which is also what such an artifact looks like on disk.
                Mittens: { digest: DIGEST_A, },
              },
            },);
            expect(
              await refusalOf({ artifactsDir: scratch.path, },),
            ).toContain('no schema version at all',);
          },
        },),
        it({
          name:
            'REFUSES a generation written AFTER this build, rather than reading it as one it knows: a reader '
              + 'meeting a later shape knows only that it does not know the shape',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'pass-schema-guard-', },);
            await writeArtifacts({
              dir: scratch.path,
              entries: {
                Mittens: {
                  version: 99,
                  digest: DIGEST_A,
                },
              },
            },);
            expect(
              await refusalOf({ artifactsDir: scratch.path, },),
            ).toContain('a schema generation this build cannot read',);
          },
        },),
        it({
          name:
            'names every way forward including moving the incompatible artifacts aside, and names deleting '
              + 'them as the one thing to avoid rather than as no remedy at all: a moved file is re-run and a '
              + 'deleted one is re-run too, and only one of the two keeps the result it already was',
          fn: async () => {
            /**
             Whatever the refusal said.
             */
            await using scratch = await scratchDir({ prefix: 'pass-schema-guard-', },);
            await writeArtifacts({
              dir: scratch.path,
              entries: {
                Mittens: {
                  version: 1,
                  digest: DIGEST_A,
                },
              },
            },);
            const said = await refusalOf({ artifactsDir: scratch.path, },);
            expect(said,).toContain('TRANSLATION_REPAIR_RUNS_DIR',);
            expect(said,).toContain('Restore the code those entries were settled under',);
            expect(said,).toContain('Move the incompatible artifacts to an archive directory',);
            expect(said,).toContain('Deleting them outright is the one thing to avoid',);
            expect(said,).toContain('this pass writes schema version 22',);
          },
        },),
        it({
          name:
            'REFUSES a file that is not an artifact at all APART, with removing it as the remedy, rather than '
              + 'naming it among the foreign generations with the advice to archive it as the generation it is '
              + '(ledger B53)',
          fn: async () => {
            /**
             A directory holding a sound artifact of another generation beside
             two files that are not artifacts.
             */
            await using scratch2 = await scratchDir({ prefix: 'pass-schema-guard-', },);
            const artifactsDir = scratch2.path;
            await writeArtifacts({
              dir: artifactsDir,
              entries: {
                Mittens: {
                  version: 1,
                  digest: DIGEST_A,
                },
              },
            },);
            await writeFile(
              join(
                artifactsDir,
                'Smudge.json',
              ),
              'not json {',
              'utf8',
            );
            await writeFile(
              join(
                artifactsDir,
                'Tuxedo.json',
              ),
              '["a", "list", "of", "cats"]',
              'utf8',
            );

            /**
             What the guard said about it.
             */
            const said = await refusalOf({ artifactsDir, },);
            expect(said,).toContain('2 artifacts in this directory record no readable pipeline',);
            expect(said,).toContain('  Smudge',);
            expect(said,).toContain('  Tuxedo',);
            expect(said,).toContain('Deleting the file is the whole remedy',);
            expect(said,).not.toContain('another schema generation',);
            expect(said,).not.toContain('Mittens',);
          },
        },),
        it({
          name:
            'NAMES the first five entries of a generation and COUNTS the rest, so a corpus-sized directory '
              + 'still produces a readable refusal',
          fn: async () => {
            /**
             Six entries of one foreign generation.
             */
            await using scratch3 = await scratchDir({ prefix: 'pass-schema-guard-', },);
            const artifactsDir = scratch3.path;
            await writeArtifacts({
              dir: artifactsDir,
              entries: Object.fromEntries(
                ['Ash', 'Biscuit', 'Clover', 'Dusty', 'Ember', 'Fig',].map(function versionOne(entryId,): [string, Fixture,] {
                  return [
                    entryId,
                    {
                      version: 1,
                      digest: DIGEST_A,
                    },
                  ];
                },),
              ),
            },);

            expect(await refusalOf({ artifactsDir, },),).toContain(
              'schema version 1: 6 settled, Ash, Biscuit, Clover, Dusty, Ember, and 1 more',
            );
          },
        },),
      ],
    },),

    describe({
      name: censusBySchema.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'classifies every settled entry rather than grouping them by the sentence a refusal would '
              + 'print, so a file that is not an artifact at all stays distinguishable from a sound artifact '
              + 'of a generation this build cannot read, and each can be offered the remedy that fits',
          fn: async () => {
            /**
             A directory holding four different answers at once.
             */
            await using scratch4 = await scratchDir({ prefix: 'pass-schema-guard-', },);
            const artifactsDir = scratch4.path;
            await writeArtifacts({
              dir: artifactsDir,
              entries: {
                Pouncer: {
                  version: 1,
                  digest: DIGEST_A,
                },
                Mittens: {
                  version: 5,
                  digest: DIGEST_B,
                  wellFormed: true,
                },
                Whiskers: { digest: DIGEST_B, },
                Tabby: {
                  version: 99,
                  digest: DIGEST_A,
                },
              },
            },);

            // A file that is not JSON at all, written directly since no fixture
            // shape produces one.
            await writeFile(
              join(
                artifactsDir,
                'Smudge.json',
              ),
              'not json {',
              'utf8',
            );

            /**
             Every entry, in directory order.
             */
            const rows = await censusBySchema({ artifactsDir, },);
            expect(
              rows.map(function toPair({ entryId, classification, },): readonly [
                string,
                string,
              ] {
                return [
                  entryId,
                  classification.kind,
                ];
              },),
            ).toEqual([
              [
                'Mittens',
                'declared',
              ],
              [
                'Pouncer',
                'declared',
              ],
              [
                'Smudge',
                'malformed',
              ],
              [
                'Tabby',
                'unreadable-version',
              ],
              [
                'Whiskers',
                'unversioned',
              ],
            ],);
          },
        },),
      ],
    },),
  ],
},);
