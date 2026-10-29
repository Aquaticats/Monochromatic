/**
 Tests for the resume guard that keeps one accumulation at one built pipeline.

 The failure these exist for was measured, not imagined. One accumulation
 directory held 22 settled entries across FOUR generations. None of the four
 was a decision: the pass stops at its soft budget, a fresh invocation resumes
 it, and that invocation builds again. Four resumes across an evening of
 ordinary commits produced four generations, and every reader that computes a
 rate then refuses the whole pool.

 THE HALVES ARE CALLED AS THE PASS CALLS THEM (ledger B30): the placement
 census first, and the build check over what it returns. A wrapper running
 both was tested here in their place while the pass ran a schema check
 between them and never called it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertArtifactsPlaceable,
  assertBuildGenerationResumable,
  GenerationDriftError,
  LegacyPipelineError,
  readDriftOptIn,
  UnplaceableArtifactError,
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
 A third, for the case where a directory is already mixed before this run.
 */
const DIGEST_C = `sha256-tree-v1:${'c'.repeat(64,)}`;

/**
 Commit every fixture artifact records, since these cases turn on the build
 rather than on provenance.
 */
const FIXED_TIP = '1111111111111111111111111111111111111111';

/**
 Environment variable the drift opt-in reads.
 */
const ALLOW_DRIFT_VAR = 'TRANSLATION_REPAIR_ALLOW_GENERATION_DRIFT';

/**
 Sets the drift opt-in for the life of a scope and restores it on exit.

 Restored rather than left set, since a leaked opt-in would read as asked for
 in every later case in this process.

 @param value - value to set, exact opt-in or otherwise

 @returns Disposable restoring the previous value, including its absence

 @example
 ```ts
 using _override = withDriftVar({ value: 'yes', },);
 ```
 */
function withDriftVar({ value, }: { readonly value: string; },): Disposable {
  /**
   Value before this scope; absent means the variable was unset.
   */
  const original = process.env[ALLOW_DRIFT_VAR];
  process.env[ALLOW_DRIFT_VAR] = value;
  return {
    [Symbol.dispose](): void {
      if (original === undefined)
        Reflect.deleteProperty(process.env, ALLOW_DRIFT_VAR,);
      else
        process.env[ALLOW_DRIFT_VAR] = original;
    },
  };
}

/**
 Diverts `console.log` into a list until disposed, for the one line the
 build check prints when drift is permitted.

 @param lines - where diverted lines are appended

 @returns Disposable putting the terminal's writer back

 @example
 ```ts
 using _capture = printingInto({ lines, },);
 ```
 */
function printingInto({ lines, }: { readonly lines: string[]; },): Disposable {
  /**
   The terminal's own writer, put back on disposal.
   */
  const terminal = console.log;
  console.log = (...parts: readonly unknown[]) => {
    lines.push(parts.map(String,)
      .join(' ',),);
  };
  return {
    [Symbol.dispose](): void {
      console.log = terminal;
    },
  };
}

/**
 Writes artifacts into a caller-owned directory.

 @param dir - case-owned directory to write into

 @param generations - one artifact per entry, each recording the given built
 pipeline alongside a fixed commit

 @returns Nothing; the files are the effect

 @example
 ```ts
 await writeArtifacts({ dir: scratch.path, generations: { Mittens: DIGEST_A, }, },);
 ```
 */
async function writeArtifacts(
  { dir, generations, }: {
    readonly dir: string;
    readonly generations: Readonly<Record<string, string>>;
  },
): Promise<void> {
  await Promise.all(
    Object.entries(generations,)
      .map(async function writeOne([entryId, digest,],) {
        await writeFile(
          join(
            dir,
            `${entryId}.json`,
          ),
          JSON.stringify({
            id: entryId,
            tip: FIXED_TIP,
            pipelineDigest: digest,
            status: 'repaired',
          },),
          'utf8',
        );
      },),
  );
}

/**
 What the placement census reads off a directory of settled entries, as the
 pass reads it before the build check.

 @param generations - one artifact per entry, each recording the given built
 pipeline

 @returns Census the build check is handed

 @example
 ```ts
 const census = await censusOf({ generations: { Mittens: DIGEST_A, }, },);
 ```
 */
async function censusOf(
  { generations, }: { readonly generations: Readonly<Record<string, string>>; },
): Promise<Parameters<typeof assertBuildGenerationResumable>[0]['census']> {
  await using scratch = await scratchDir({ prefix: 'pass-generation-guard-', },);
  await writeArtifacts({ dir: scratch.path, generations, },);
  return await assertArtifactsPlaceable({ artifactsDir: scratch.path, },);
}

/**
 What the build check refused a directory of settled entries with.

 @param generations - one artifact per entry, each recording the given built
 pipeline

 @param digest - built pipeline this invocation would stamp

 @param driftAllowed - whether the operator asked for a mixed directory

 @returns What the build check threw

 @throws {@link Error} when the build check passed

 @example
 ```ts
 const refusal = await buildRefusalOf({ generations: { Mittens: DIGEST_A, }, digest: DIGEST_B, driftAllowed: false, },);
 ```
 */
async function buildRefusalOf(
  {
    generations,
    digest,
    driftAllowed,
  }: {
    readonly generations: Readonly<Record<string, string>>;
    readonly digest: string;
    readonly driftAllowed: boolean;
  },
): Promise<unknown> {
  /**
   What every placeable artifact records.
   */
  const census = await censusOf({ generations, },);
  return caught(function buildCheck(): unknown {
    assertBuildGenerationResumable({
      census,
      digest,
      driftAllowed,
    },);
    return undefined;
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertBuildGenerationResumable.name,
      // ONE AT A TIME: a case diverts the process-wide `console.log` (ledger B79).
      concurrency: 1,
      children: [
        it({
          name: 'passes a FRESH directory, since a first invocation has nothing to '
            + 'disagree with and must not be made to look like a fault',
          fn: async () => {
            assertBuildGenerationResumable({
              census: await censusOf({ generations: {}, },),
              digest: DIGEST_A,
              driftAllowed: false,
            },);
          },
        },),

        it({
          name: 'passes a resume under the SAME build, which is the ordinary case '
            + 'this guard must not make expensive: a pass stopped at its soft '
            + 'budget and is being continued with nothing landed in between',
          fn: async () => {
            assertBuildGenerationResumable({
              census: await censusOf({
                generations: {
                  Mittens: DIGEST_A,
                  Pepper: DIGEST_A,
                },
              },),
              digest: DIGEST_A,
              driftAllowed: false,
            },);
          },
        },),

        it({
          name: 'REFUSES a resume that would stamp a second pipeline into one pool. '
            + 'This is the whole guard: by the time a reader refuses the mixed '
            + 'pool the budget is already spent, so the refusal has to happen '
            + 'before any entry is settled rather than after',
          fn: async () => {
            /**
             What the build check refused with, read for class as well as wording.
             */
            const refusal = await buildRefusalOf({
              generations: {
                Mittens: DIGEST_A,
                Pepper: DIGEST_A,
              },
              digest: DIGEST_B,
              driftAllowed: false,
            },);

            expect(refusal,).toBeInstanceOf(GenerationDriftError,);
            expect((refusal as Error).message,).toContain('built by a different pipeline',);
          },
        },),

        it({
          name: 'names EVERY pipeline already present, not just one, so an operator '
            + 'reading the refusal can see a directory that is already mixed '
            + 'rather than believing it holds a single clean generation',
          fn: async () => {
            /**
             What the build check refused with, read for class as well as wording.
             */
            const refusal = await buildRefusalOf({
              generations: {
                Mittens: DIGEST_A,
                Pepper: DIGEST_B,
              },
              digest: DIGEST_C,
              driftAllowed: false,
            },);

            expect(refusal,).toBeInstanceOf(GenerationDriftError,);
            // Sixteen characters, not the full id: every digest opens with the
            // same scheme name, so the abbreviation grows past its floor to the
            // first character that differs. Asserting the full id would pass
            // only if the message stopped abbreviating at all.
            expect((refusal as Error).message,).toContain('sha256-tree-v1:a',);
            expect((refusal as Error).message,).toContain('sha256-tree-v1:b',);
          },
        },),

        it({
          name: 'permits drift when asked EXPLICITLY, so an operator who wants a '
            + 'deliberately mixed directory is not blocked',
          fn: async () => {
            assertBuildGenerationResumable({
              census: await censusOf({ generations: { Mittens: DIGEST_A, }, },),
              digest: DIGEST_B,
              driftAllowed: true,
            },);
          },
        },),

        it({
          name: 'SAYS how many foreign pipelines a permitted drift resumes across, in the number the count takes',
          fn: async () => {
            /**
             What a directory under one foreign pipeline reads as.
             */
            const oneForeign = await censusOf({ generations: { Mittens: DIGEST_A, }, },);
            /**
             What a directory under two foreign pipelines reads as.
             */
            const twoForeign = await censusOf({
              generations: {
                Mittens: DIGEST_A,
                Pepper: DIGEST_B,
              },
            },);
            /**
             Lines the build check printed.
             */
            const printed: string[] = [];
            {
              using _capture = printingInto({ lines: printed, },);
              assertBuildGenerationResumable({
                census: oneForeign,
                digest: DIGEST_C,
                driftAllowed: true,
              },);
              assertBuildGenerationResumable({
                census: twoForeign,
                digest: DIGEST_C,
                driftAllowed: true,
              },);
            }

            expect(printed,).toContain(
              `POOL resuming across 1 foreign pipeline because ${ALLOW_DRIFT_VAR}=yes; a rate over this directory must name a required commit`,
            );
            expect(printed,).toContain(
              `POOL resuming across 2 foreign pipelines because ${ALLOW_DRIFT_VAR}=yes; a rate over this directory must name a required commit`,
            );
          },
        },),
      ],
    },),

    describe({
      name: assertArtifactsPlaceable.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES to resume a directory whose artifacts predate generation '
            + 'identity, and does NOT tell the operator to delete them. They are '
            + 'sound results recorded under a commit, which covers any number of '
            + 'builds, so nothing can say whether this run is the pipeline that '
            + 'wrote them; the remedy is a fresh directory, not a smaller one',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'pass-generation-guard-', },);
            const dir = scratch.path;
            await writeArtifacts({ dir, generations: {}, },);
            await writeFile(
              join(
                dir,
                'Mittens.json',
              ),
              JSON.stringify({
                id: 'Mittens',
                tip: FIXED_TIP,
                status: 'repaired',
              },),
              'utf8',
            );

            /**
             What the placement census refused with, read for class as well as wording.
             */
            const refusal = assertArtifactsPlaceable({ artifactsDir: dir, },);

            await expect(refusal,).rejects.toBeInstanceOf(LegacyPipelineError,);
            await expect(refusal,).rejects.toThrow('Deleting them is NOT the remedy',);
          },
        },),

        it({
          name: 'REFUSES to resume a directory holding an UNTAGGED artifact, which '
            + 'reading only the tip groups missed entirely: a directory of nothing '
            + 'but unplaceable artifacts produced no groups and sailed through. Such '
            + 'an entry is counted as settled by the scheduler so it never retries, '
            + 'and excluded by the pool filter so it never appears in a rate; it '
            + 'ceases to exist and no count says so',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'pass-generation-guard-', },);
            const dir = scratch.path;
            await writeArtifacts({ dir, generations: { Pepper: DIGEST_A, }, },);
            await writeFile(
              join(
                dir,
                'Mittens.json',
              ),
              JSON.stringify({ status: 'repaired', },),
              'utf8',
            );

            /**
             What the placement census refused with, read for class as well as wording.
             */
            const refusal = assertArtifactsPlaceable({ artifactsDir: dir, },);

            await expect(refusal,).rejects.toBeInstanceOf(UnplaceableArtifactError,);
            await expect(refusal,).rejects.toThrow('Mittens',);
          },
        },),

        it({
          name: 'REFUSES a MALFORMED artifact for the same reason, and names it, '
            + 'since deleting the file is the whole remedy and an operator cannot '
            + 'delete what the refusal does not name',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'pass-generation-guard-', },);
            const dir = scratch.path;
            await writeArtifacts({ dir, generations: { Pepper: DIGEST_A, }, },);
            await writeFile(
              join(
                dir,
                'Biscuit.json',
              ),
              '{ "tip": "aaaaaaaaa", "status": "rep',
              'utf8',
            );

            /**
             What the placement census refused with, read for class as well as wording.
             */
            const refusal = assertArtifactsPlaceable({ artifactsDir: dir, },);

            await expect(refusal,).rejects.toBeInstanceOf(UnplaceableArtifactError,);
            await expect(refusal,).rejects.toThrow('Biscuit',);
          },
        },),

        it({
          name: 'COUNTS the entries each refusal names in the number the count takes, one artifact or several',
          fn: async () => {
            /**
             The count line and the named entries opening a refusal's message.

             @param error - refusal built over some entries

             @param named - how many entries it names

             @returns Those lines
             */
            function openingOf(
              {
                error,
                named,
              }: {
                readonly error: Error;
                readonly named: number;
              },
            ): readonly string[] {
              return error.message
                .split('\n',)
                .slice(
                0,
                named + 1,
                );
            }

            expect({
              legacyOne: openingOf({
                error: new LegacyPipelineError({ entryIds: ['Mittens',], },),
                named: 1,
              },),
              legacyTwo: openingOf({
                error: new LegacyPipelineError({ entryIds: ['Mittens', 'Pepper',], },),
                named: 2,
              },),
              unplaceableOne: openingOf({
                error: new UnplaceableArtifactError({ entryIds: ['Mittens',], },),
                named: 1,
              },),
              unplaceableTwo: openingOf({
                error: new UnplaceableArtifactError({ entryIds: ['Mittens', 'Pepper',], },),
                named: 2,
              },),
            },).toEqual({
              legacyOne: ['1 artifact here records a pipeline this build cannot name:', '  Mittens',],
              legacyTwo: ['2 artifacts here record a pipeline this build cannot name:', '  Mittens', '  Pepper',],
              unplaceableOne: ['1 artifact in this directory records no readable pipeline:', '  Mittens',],
              unplaceableTwo: ['2 artifacts in this directory record no readable pipeline:', '  Mittens', '  Pepper',],
            },);
          },
        },),
      ],
    },),

    describe({
      name: readDriftOptIn.name,
      // ONE AT A TIME as well: a case points the process's own variable
      // elsewhere while the read runs (ledger B79).
      concurrency: 1,
      children: [
        it({
          name: 'reads the exact opt-in and NOTHING ELSE from the environment it is handed: '
            + 'a stray 0, an empty value, the opt-in in capitals or with a trailing space, true, and no variable at all '
            + 'leave the guard on',
          fn: async () => {
            expect(readDriftOptIn({ env: { [ALLOW_DRIFT_VAR]: 'yes', }, },),).toBe(true,);
            expect([
              '0',
              '',
              'YES',
              'yes ',
              'true',
            ].map(function optInOf(value,): boolean {
              return readDriftOptIn({ env: { [ALLOW_DRIFT_VAR]: value, }, },);
            },),).toEqual([false, false, false, false, false,],);
            expect(readDriftOptIn({ env: {}, },),).toBe(false,);
          },
        },),

        it({
          name: 'READS THE ENVIRONMENT IT IS HANDED, never the process\'s own, whose opt-in would let a '
            + 'resume across a foreign build through, and whose absence would refuse one the launch asked for',
          fn: async () => {
            {
              using _optedIn = withDriftVar({ value: 'yes', },);
              expect(readDriftOptIn({ env: {}, },),).toBe(false,);
            }
            {
              using _optedOut = withDriftVar({ value: 'no', },);
              expect(readDriftOptIn({ env: { [ALLOW_DRIFT_VAR]: 'yes', }, },),).toBe(true,);
            }
          },
        },),
      ],
    },),
  ],
},);
