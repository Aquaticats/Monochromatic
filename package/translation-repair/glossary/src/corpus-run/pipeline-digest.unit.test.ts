/**
 Tests for identifying a built pipeline by what its output directory holds.

 The digest decides which settled entries may be pooled and whether an
 accumulation may resume, so every property here is load-bearing: an
 order-dependent digest would refuse every resume, and a digest blind to a
 changed file would pool two pipelines as one.

 Each case that asserts two directories agree is paired with one that must
 disagree. A sameness claim from a probe that cannot show a difference proves
 nothing at all, which is this package's recurring way of being wrong.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertPipelineDigest,
  digestPipeline,
  isDigestShaped,
  PipelineDigestError,
} from '../../dist/final/node/index.mjs';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Contents of a minimal built output directory.
 */
const BUILT = {
  'index.mjs': 'export const mittens = 1;\n',
  'chunk-Whiskers.mjs': 'export const pepper = 2;\n',
} as const;

/**
 Writes a throwaway output directory.

 @param files - file name to contents, at any depth

 @returns Built directory, removed when its `await using` scope ends

 @example
 ```ts
 await using built = await writeBuild({ files: BUILT, },);
 const dir = built.dir;
 ```
 */
async function writeBuild(
  { files, }: { readonly files: Readonly<Record<string, string>>; },
): Promise<{ readonly dir: string; } & AsyncDisposable> {
  // Disposable root for this case.
  return await scratchDirWith({
    prefix: 'pipeline-digest-',
    setup: async function seeded({ path: dir, },): Promise<{ readonly dir: string; }> {
      for (const [name, text,] of Object.entries(files,)) {
        /**
         Path this file takes inside the fixture.
         */
        const path = join(
          dir,
          name,
        );

        /* oxlint-disable-next-line no-await-in-loop -- fixture setup, and nesting means a later file can need a directory an earlier one created */
        await mkdir(
          join(
            path,
            '..',
          ),
          { recursive: true, },
        );
        /* oxlint-disable-next-line no-await-in-loop -- same, sequential by design */
        await writeFile(
          path,
          text,
        );
      }

      return { dir, };
    },
  },);
}

/**
 The whole message a refusal to identify a build carries, written out here so
 a case checks every line of it rather than a fragment.

 @param dir - directory the digest was to be taken over

 @param reason - clause saying what made it unusable

 @returns Message the refusal should carry

 @example
 ```ts
 const expected = refusalMessage({ dir, reason: 'it holds no file that could execute', },);
 ```
 */
function refusalMessage(
  {
    dir,
    reason,
  }: {
    readonly dir: string;
    readonly reason: string;
  },
): string {
  return [
    `Cannot identify the built pipeline in ${dir}: ${reason}.`,
    '',
    'This digest is what every artifact records as the generation that',
    'produced it, so a pass that cannot compute one would settle entries',
    'stamped with nothing and the pool could not tell its versions apart.',
    '',
    'Every corpus-run task depends on the build and runs the built file, so',
    'reaching this means the build did not run or its output was moved.',
  ].join('\n',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: digestPipeline.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'gives one digest for the same files whatever order they were '
            + 'written in, which is what makes a digest comparable at all: a '
            + 'directory read is not ordered, so an order-dependent digest would '
            + 'refuse resumes at random',
          fn: async () => {
            await using builtOne = await writeBuild({ files: BUILT, },);
            const one = builtOne.dir;
            await using builtTwo = await writeBuild({
              files: {
                'chunk-Whiskers.mjs': BUILT['chunk-Whiskers.mjs'],
                'index.mjs': BUILT['index.mjs'],
              },
            },);
            const two = builtTwo.dir;

            expect((await digestPipeline({ dir: one, },)).digest,)
              .toBe((await digestPipeline({ dir: two, },)).digest,);
          },
        },),

        it({
          name: 'MOVES when one executable byte changes, the positive control '
            + 'every sameness claim here rests on: without it, a digest that never '
            + 'changed would satisfy every other case in this file',
          fn: async () => {
            await using builtOne = await writeBuild({ files: BUILT, },);
            const one = builtOne.dir;
            await using builtTwo = await writeBuild({
              files: {
                ...BUILT,
                'index.mjs': 'export const mittens = 2;\n',
              },
            },);
            const two = builtTwo.dir;

            expect((await digestPipeline({ dir: one, },)).digest,)
              .not
              .toBe((await digestPipeline({ dir: two, },)).digest,);
          },
        },),

        it({
          name: 'IGNORES TypeScript declarations, because they cannot execute and '
            + 'they carry every TSDoc block verbatim while the built .mjs carries '
            + 'no comments at all. Hashing them would make a comment-only edit a '
            + 'new generation and force a fresh accumulation directory for a change '
            + 'that cannot alter a single result',
          fn: async () => {
            await using builtOne = await writeBuild({ files: BUILT, },);
            const one = builtOne.dir;
            await using builtTwo = await writeBuild({
              files: {
                ...BUILT,
                'index.d.mts': '/** Whiskers. */\nexport declare const mittens: number;\n',
              },
            },);
            const two = builtTwo.dir;

            expect((await digestPipeline({ dir: one, },)).digest,)
              .toBe((await digestPipeline({ dir: two, },)).digest,);
          },
        },),

        it({
          name: 'covers files at any depth, since a build free to emit chunks into '
            + 'a subdirectory would otherwise have half its code outside its own '
            + 'identity',
          fn: async () => {
            await using builtFlat = await writeBuild({ files: BUILT, },);
            const flat = builtFlat.dir;
            await using builtNested = await writeBuild({
              files: {
                ...BUILT,
                'inner/deep.mjs': 'export const biscuit = 3;\n',
              },
            },);
            const nested = builtNested.dir;

            expect((await digestPipeline({ dir: nested, },)).fileCount,).toBe(3,);
            expect((await digestPipeline({ dir: flat, },)).digest,)
              .not
              .toBe((await digestPipeline({ dir: nested, },)).digest,);
          },
        },),

        it({
          name: 'REFUSES a symbolic link rather than following or skipping it. The '
            + 'build emits none, so one being there means something else wrote into '
            + 'the output directory, and both other answers are wrong: following it '
            + 'digests bytes from outside the pipeline, skipping it drops code that '
            + 'will run',
          fn: async () => {
            await using built = await writeBuild({ files: BUILT, },);
            const { dir, } = built;
            await symlink(
              join(
                dir,
                'index.mjs',
              ),
              join(
                dir,
                'linked.mjs',
              ),
            );

            /**
             What digestPipeline refused with, read for class as well as wording.
             */
            const refusalOfDigestPipeline = digestPipeline({ dir, },);

            await expect(refusalOfDigestPipeline,).rejects.toBeInstanceOf(PipelineDigestError,);
            await expect(refusalOfDigestPipeline,).rejects.toHaveProperty(
              'message',
              refusalMessage({
                dir,
                reason: 'it holds 1 entry that is neither a regular file nor a directory, such as linked.mjs, which '
                  + 'the build never emits',
              },),
            );
          },
        },),

        it({
          name: 'COUNTS every such entry and names the first by name, so a directory holding several says how '
            + 'many and gives the same example whatever order the directory lists them in',
          fn: async () => {
            await using built = await writeBuild({ files: BUILT, },);
            const { dir, } = built;
            await symlink(
              join(
                dir,
                'index.mjs',
              ),
              join(
                dir,
                'linked.mjs',
              ),
            );
            await symlink(
              join(
                dir,
                'index.mjs',
              ),
              join(
                dir,
                'aliased.mjs',
              ),
            );

            /**
             What digestPipeline refused with, read for class as well as wording.
             */
            const refusalOfTwoLinks = digestPipeline({ dir, },);

            await expect(refusalOfTwoLinks,).rejects.toBeInstanceOf(PipelineDigestError,);
            await expect(refusalOfTwoLinks,).rejects.toHaveProperty(
              'message',
              refusalMessage({
                dir,
                reason: 'it holds 2 entries that are neither a regular file nor a directory, such as aliased.mjs, '
                  + 'which the build never emits',
              },),
            );
          },
        },),

        it({
          name: 'REFUSES a directory holding nothing that could execute, since a '
            + 'digest over nothing is a constant every empty build would share, and '
            + 'a pass stamping it would claim a pipeline that does not exist',
          fn: async () => {
            await using built = await writeBuild({
              files: { 'index.d.mts': 'export declare const mittens: number;\n', },
            },);
            const { dir, } = built;

            /**
             What digestPipeline refused with, read for class as well as wording.
             */
            const refusalOfDigestPipeline = digestPipeline({ dir, },);

            await expect(refusalOfDigestPipeline,).rejects.toBeInstanceOf(PipelineDigestError,);
            await expect(refusalOfDigestPipeline,).rejects.toHaveProperty(
              'message',
              refusalMessage({
                dir,
                reason: 'it holds no file that could execute',
              },),
            );
          },
        },),

        it({
          name: 'reports the file count beside the digest, so a log line naming a '
            + 'truncated output directory is legible as one: a digest alone is '
            + 'unfalsifiable to a reader',
          fn: async () => {
            await using built = await writeBuild({ files: BUILT, },);
            const { dir, } = built;

            expect((await digestPipeline({ dir, },)).fileCount,).toBe(2,);
          },
        },),
      ],
    },),

    describe({
      name: isDigestShaped.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'accepts what this module emits and refuses everything else, which '
            + 'is what keeps a foreign value out of the field the pool partitions '
            + 'by: uppercase is refused because one pipeline spelled two ways would '
            + 'count as two generations',
          fn: async () => {
            expect(isDigestShaped({ value: `sha256-tree-v1:${'a'.repeat(64,)}`, },),).toBe(true,);
            expect(isDigestShaped({ value: `sha256-tree-v1:${'A'.repeat(64,)}`, },),).toBe(false,);
            expect(isDigestShaped({ value: `sha256-tree-v1:${'a'.repeat(63,)}`, },),).toBe(false,);
            expect(isDigestShaped({ value: `sha256-tree-v1:${'a'.repeat(65,)}`, },),).toBe(false,);
            expect(isDigestShaped({ value: `sha256-tree-v2:${'a'.repeat(64,)}`, },),).toBe(false,);
            expect(isDigestShaped({ value: 'a'.repeat(64,), },),).toBe(false,);
            expect(isDigestShaped({ value: '', },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: assertPipelineDigest.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'throws on a value no build produced, rather than narrowing it, so '
            + 'a digest read back from an artifact cannot enter the type system '
            + 'without passing the same test the writer passed',
          fn: async () => {
            /**
             What narrows raised, read for its class as well as its wording.
             */
            const refusalOfNarrows = caught(function narrows() {
              assertPipelineDigest('not-a-digest',);
            },);

            expect(refusalOfNarrows,).toBeInstanceOf(TypeError,);
            // THE WHOLE SHAPE, scheme name first: the sentence once described only
            // the hex half, so a value missing its scheme name read as refused for
            // its hex (ledger B34).
            expect((refusalOfNarrows as Error).message,)
              .toBe('A pipeline digest is "sha256-tree-v1:" followed by 64 lowercase hex characters; '
                + 'received "not-a-digest".',);
          },
        },),
      ],
    },),
  ],
},);
