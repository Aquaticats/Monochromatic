/**
 Tests for the pinned reviewed-reference read: the checks that run before any
 file is touched (cancellation, revision, entry identifier), the two pinned
 file reads and what each refusal names, and the order of the references
 returned.

 Corpora here are disposable repositories holding invented pages; the
 default-manifest cases read the checked-in manifest's metadata only and stop
 at a refusal before any file access. No corpus content appears here.

 @module
 */

import { mkdir, writeFile, } from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CorpusReadError, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  type FidelityReferenceSpec,
  readReviewedFidelityReferences,
  REVIEWED_FIDELITY_REFERENCES,
} from '../dist/final/node/index.mjs';

import {
  makeNamingArchive,
  namingFixtureGit,
} from '@monochromatic-dev/translation-repair-corpus-pass/ts/archive-naming.test-fixture.ts';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import {
  expectRefusal,
  REVIEW_REFERENCE,
  REVIEW_SOURCE,
  reviewedFixture,
  settledRefusal,
  summaryOf,
} from './fidelity-reference.test-fixture.ts';

//region Pinned corpus fixtures
// Invented pages committed to an already disposable corpus repository.

/**
 One invented page to commit.
 */
type Page = {
  /**
   Path inside the repository.
   */
  readonly relPath: string;

  /**
   Page text.
   */
  readonly text: string;
};

/**
 Commits invented pages to a disposable corpus repository.

 @param cloneDir - owned repository

 @param pages - pages to write and commit

 @returns Commit holding them

 @example
 ```ts
 const commitSha = await commitPages({ cloneDir, pages: [{ relPath: 'people/starlit-cat/page.md', text, },], },);
 ```
 */
async function commitPages(
  {
    cloneDir,
    pages,
  }: {
    readonly cloneDir: string;
    readonly pages: readonly Page[];
  },
): Promise<string> {
  await Promise.all(pages.map(async function written(page,): Promise<void> {
    await mkdir(
      dirname(join(
        cloneDir,
        page.relPath,
      ),),
      { recursive: true, },
    );
    await writeFile(
      join(
        cloneDir,
        page.relPath,
      ),
      page.text,
    );
  },),);
  await namingFixtureGit({
    cloneDir,
    args: [
      'add',
      '--',
      ...pages.map(function pathOf(page,): string {
        return page.relPath;
      },),
    ],
  },);
  await namingFixtureGit({
    cloneDir,
    args: [
      'commit',
      '--message',
      'add invented pages',
    ],
  },);
  return await namingFixtureGit({
    cloneDir,
    args: [
      'rev-parse',
      'HEAD',
    ],
  },);
}

/**
 Invented source page of the reviewed entry.
 */
const SOURCE_PAGE = 'people/starlit-cat/page.md';

/**
 Entry identifiers that are not one literal directory name.
 */
const NONLITERAL_ENTRY_IDS: readonly string[] = [
  '../elsewhere',
  'nested/entry',
  String.raw`nested\entry`,
  '',
  '.',
  '..',
  'null\0entry',
];

/**
 Pin whose clone does not exist, so any file access would fail differently from the refusal a case expects.
 */
const UNUSED_CLONE = '/unused-fixture';

//endregion Pinned corpus fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readReviewedFidelityReferences.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the reviewed source and archive through the pinned corpus into a verified reference',
          fn: async () => {
            const fixture = reviewedFixture();
            await using corpus = await makeNamingArchive({ after: fixture.archiveFile, },);
            const commitSha = await commitPages({
              cloneDir: corpus.pin.cloneDir,
              pages: [{
                relPath: SOURCE_PAGE,
                text: fixture.sourceFile,
              },],
            },);
            const spec = {
              ...fixture.spec,
              corpusSha: commitSha,
            };
            const result = await readReviewedFidelityReferences({
              pin: {
                ...corpus.pin,
                commitSha,
              },
              specs: [spec,],
            },);
            expect(result.map((reference,) => summaryOf({ reference, },),),).toEqual([{
              spec,
              sourceText: REVIEW_SOURCE,
              referenceText: REVIEW_REFERENCE,
              damages: spec.damages,
            },],);
          },
        },),
        it({
          name: 'RETURNS one reference per specification in the order the specifications are given',
          fn: async () => {
            const fixture = reviewedFixture();
            await using corpus = await makeNamingArchive({ after: fixture.archiveFile, },);
            const commitSha = await commitPages({
              cloneDir: corpus.pin.cloneDir,
              pages: [
                {
                  relPath: SOURCE_PAGE,
                  text: fixture.sourceFile,
                },
                {
                  relPath: 'people/second-cat/page.md',
                  text: fixture.sourceFile,
                },
                {
                  relPath: 'people/second-cat/page.en.md',
                  text: fixture.archiveFile,
                },
              ],
            },);
            const first = {
              ...fixture.spec,
              corpusSha: commitSha,
            };
            const second = {
              ...first,
              id: 'second-reference',
              entryId: 'second-cat',
            };
            const result = await readReviewedFidelityReferences({
              pin: {
                ...corpus.pin,
                commitSha,
              },
              specs: [
                second,
                first,
              ],
            },);
            expect(result.map((reference,) => reference.spec.id,),).toEqual([
              'second-reference',
              'invented-reference',
            ],);
          },
        },),
        it({
          name: 'REFUSES a different corpus revision at the pin boundary before any file is read',
          fn: async () => {
            const fixture = reviewedFixture();
            const refusal = await settledRefusal(readReviewedFidelityReferences({
              pin: {
                cloneDir: UNUSED_CLONE,
                commitSha: 'b'.repeat(40,),
              },
              specs: [fixture.spec,],
            },),);
            expectRefusal({
              refusal,
              referenceId: 'invented-reference',
              operation: 'pin',
            },);
          },
        },),
        ...NONLITERAL_ENTRY_IDS.map(function entryCase(entryId,) {
          return it({
            name: `REFUSES the entry identifier ${JSON.stringify(entryId,)} at the request boundary before any file is `
              + 'read, since it is no one literal directory name',
            fn: async () => {
              const fixture = reviewedFixture();
              const refusal = await settledRefusal(readReviewedFidelityReferences({
                pin: {
                  cloneDir: UNUSED_CLONE,
                  commitSha: fixture.spec.corpusSha,
                },
                specs: [{
                  ...fixture.spec,
                  entryId,
                },],
              },),);
              expectRefusal({
                refusal,
                referenceId: 'invented-reference',
                operation: 'request',
              },);
            },
          },);
        },),
        it({
          name: 'REFUSES an empty manifest at the request boundary before any file is read',
          fn: async () => {
            const refusal = await settledRefusal(readReviewedFidelityReferences({
              pin: {
                cloneDir: UNUSED_CLONE,
                commitSha: 'b'.repeat(40,),
              },
              specs: [],
            },),);
            expectRefusal({
              refusal,
              referenceId: 'manifest',
              operation: 'request',
            },);
          },
        },),
        it({
          name: 'PASSES a missing source page on as the corpus read\'s own refusal, not an empty calibration',
          fn: async () => {
            const fixture = reviewedFixture();
            await using corpus = await makeNamingArchive({ after: fixture.archiveFile, },);
            const refusal = await settledRefusal(readReviewedFidelityReferences({
              pin: corpus.pin,
              specs: [{
                ...fixture.spec,
                corpusSha: corpus.pin.commitSha,
              },],
            },),);
            expect(refusal,).toBeInstanceOf(CorpusReadError,);
            expect(String(refusal,),).toBe(
              `CorpusReadError: corpus read failed for ${corpus.pin.commitSha}:${SOURCE_PAGE} (missing-object);`
                + ' the commit has no such path: check the path, or pin a commit that has it.',
            );
          },
        },),
        it({
          name: 'PASSES a missing archive page on as the corpus read\'s own refusal after the source page was read',
          fn: async () => {
            const fixture = reviewedFixture();
            await using corpus = await makeNamingArchive({ after: fixture.archiveFile, },);
            await commitPages({
              cloneDir: corpus.pin.cloneDir,
              pages: [{
                relPath: SOURCE_PAGE,
                text: fixture.sourceFile,
              },],
            },);
            await namingFixtureGit({
              cloneDir: corpus.pin.cloneDir,
              args: [
                'rm',
                '--',
                corpus.relPath,
              ],
            },);
            await namingFixtureGit({
              cloneDir: corpus.pin.cloneDir,
              args: [
                'commit',
                '--message',
                'remove invented archive',
              ],
            },);
            const commitSha = await namingFixtureGit({
              cloneDir: corpus.pin.cloneDir,
              args: [
                'rev-parse',
                'HEAD',
              ],
            },);
            const refusal = await settledRefusal(readReviewedFidelityReferences({
              pin: {
                ...corpus.pin,
                commitSha,
              },
              specs: [{
                ...fixture.spec,
                corpusSha: commitSha,
              },],
            },),);
            expect(refusal,).toBeInstanceOf(CorpusReadError,);
            expect(String(refusal,),).toBe(
              `CorpusReadError: corpus read failed for ${commitSha}:${corpus.relPath} (missing-object);`
                + ' the commit has no such path: check the path, or pin a commit that has it.',
            );
          },
        },),
        it({
          name: 'NAMES the source page when both pages are missing, the source being the one read first',
          fn: async () => {
            const fixture = reviewedFixture();
            await using corpus = await makeNamingArchive({
              after: fixture.archiveFile,
              relPath: 'people/other-cat/page.en.md',
            },);
            const refusal = await settledRefusal(readReviewedFidelityReferences({
              pin: corpus.pin,
              specs: [{
                ...fixture.spec,
                corpusSha: corpus.pin.commitSha,
              },],
            },),);
            expect(String(refusal,),).toBe(
              `CorpusReadError: corpus read failed for ${corpus.pin.commitSha}:${SOURCE_PAGE} (missing-object);`
                + ' the commit has no such path: check the path, or pin a commit that has it.',
            );
          },
        },),
        it({
          name: 'PASSES an already-cancelled caller\'s own reason on before any metadata or file work',
          fn: async () => {
            const controller = new AbortController();
            const reason = new Error('fixture canceled',);
            controller.abort(reason,);
            const refusal = await settledRefusal(readReviewedFidelityReferences({
              pin: {
                cloneDir: UNUSED_CLONE,
                commitSha: 'unused',
              },
              signal: controller.signal,
            },),);
            expect(refusal,).toBe(reason,);
          },
        },),
        it({
          name: 'CHECKS cancellation a third time once both file reads have settled, handing back the caller\'s own '
            + 'reason and no reference',
          fn: async () => {
            const fixture = reviewedFixture();
            await using corpus = await makeNamingArchive({ after: fixture.archiveFile, },);
            const commitSha = await commitPages({
              cloneDir: corpus.pin.cloneDir,
              pages: [{
                relPath: SOURCE_PAGE,
                text: fixture.sourceFile,
              },],
            },);
            const controller = new AbortController();
            const reason = new Error('fixture canceled after reads',);
            const original = controller.signal.throwIfAborted.bind(controller.signal,);
            const checks = { count: 0, };
            Object.defineProperty(
              controller.signal,
              'throwIfAborted',
              {
                value: function countedCheck(): void {
                  checks.count += 1;
                  if (checks.count === 3)
                    controller.abort(reason,);
                  original();
                },
              },
            );
            const refusal = await settledRefusal(readReviewedFidelityReferences({
              pin: {
                ...corpus.pin,
                commitSha,
              },
              specs: [{
                ...fixture.spec,
                corpusSha: commitSha,
              },],
              signal: controller.signal,
            },),);
            expect(refusal,).toBe(reason,);
            expect(checks.count,).toBe(3,);
          },
        },),
        it({
          name: 'READS the checked-in manifest when none is given, refusing at the pin boundary the first reference '
            + 'reviewed for another revision',
          fn: async () => {
            const first = nonNullishOrThrow(REVIEWED_FIDELITY_REFERENCES[0],);
            const refusal = await settledRefusal(readReviewedFidelityReferences({
              pin: {
                cloneDir: UNUSED_CLONE,
                commitSha: 'b'.repeat(first.corpusSha.length,),
              },
            },),);
            expectRefusal({
              refusal,
              referenceId: first.id,
              operation: 'pin',
            },);
          },
        },),
        it({
          name: 'FILTERS the checked-in manifest by entry before reading, naming the chosen entry\'s reference',
          fn: async () => {
            const last: FidelityReferenceSpec = nonNullishOrThrow(REVIEWED_FIDELITY_REFERENCES.at(-1,),);
            const refusal = await settledRefusal(readReviewedFidelityReferences({
              pin: {
                cloneDir: UNUSED_CLONE,
                commitSha: 'b'.repeat(last.corpusSha.length,),
              },
              onlyEntryIds: [last.entryId,],
            },),);
            expectRefusal({
              refusal,
              referenceId: last.id,
              operation: 'pin',
            },);
          },
        },),
        it({
          name: 'REFUSES an entry filter naming no reviewed entry at the request boundary, before the pin is compared',
          fn: async () => {
            const refusal = await settledRefusal(readReviewedFidelityReferences({
              pin: {
                cloneDir: UNUSED_CLONE,
                commitSha: 'b'.repeat(40,),
              },
              onlyEntryIds: ['unreviewed-cat',],
            },),);
            expectRefusal({
              refusal,
              referenceId: 'unreviewed-cat',
              operation: 'request',
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'the lines a stopped reference read writes',
      // ONE CASE AT A TIME: the case diverts `console.warn` across an await to
      // read what the package's root logger wrote.
      concurrency: 1,
      children: [
        it({
          name: 'WRITES EACH PINNED PAGE\'S OWN FAILURE AS THE CALLER STOPS THE READ, the source page\'s before the '
            + 'archive page\'s, each with the corpus read\'s own refusal, and refuses with the caller\'s reason',
          fn: async () => {
            const fixture = reviewedFixture();
            await using corpus = await makeNamingArchive({
              after: fixture.archiveFile,
              relPath: 'people/other-cat/page.en.md',
            },);
            const controller = new AbortController();
            const reason = new Error('the owner called the cats in',);
            const { result: refusal, warned, } = await warnLinesDuring({
              run: async function readAsTheCallerStops(): Promise<unknown> {
                // Both page reads are in flight when the call returns its
                // promise, so the stop lands while they run.
                const reading = readReviewedFidelityReferences({
                  pin: corpus.pin,
                  specs: [{
                    ...fixture.spec,
                    corpusSha: corpus.pin.commitSha,
                  },],
                  signal: controller.signal,
                },);
                controller.abort(reason,);
                return await settledRefusal(reading,);
              },
            },);
            expect(refusal,).toBe(reason,);
            expect(warned,).toEqual([
              `[translation-repair] [readReference] invented-reference: the source page could not be read, and the `
                + `caller stopped the read: corpus read failed for ${corpus.pin.commitSha}:${SOURCE_PAGE} `
                + '(missing-object); the commit has no such path: check the path, or pin a commit that has it.',
              `[translation-repair] [readReference] invented-reference: the archive page could not be read, and the `
                + `caller stopped the read: corpus read failed for ${corpus.pin.commitSha}:people/starlit-cat/page.en.md `
                + '(missing-object); the commit has no such path: check the path, or pin a commit that has it.',
            ],);
          },
        },),
      ],
    },),
  ],
},);
