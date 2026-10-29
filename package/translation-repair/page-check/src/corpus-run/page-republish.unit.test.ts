/**
 Tests for a pass rewriting from its artifact every page that is missing or
 disagrees (ledger A16c; the owner, 2026-09-27: a run always ships).

 Every case runs in a throwaway runs directory with an artifact the builder
 wrote over a real preparation, and reads the pair from fixture text rather
 than a corpus clone.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { parseSettledTwoLaneArtifact, } from '@monochromatic-dev/translation-repair-artifact';
import { CorpusReadError, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  type CorpusPairReader,
  pageAgreement,
  type RepublishOutcome,
  republishSettledPages,
} from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import { settledArtifactText, } from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-artifact.test-fixture.ts';
import { sourceDroppedRunOf, } from './published-run.test-fixture.ts';

/**
 Original with two sections.
 */
const SOURCE_DOC = '## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n猫猫有自己的碗。\n';

/**
 Archive English of the same shape.
 */
const TARGET_DOC = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n';

/**
 The archive's own note that everything below it was written in English.
 */
const ENGLISH_ORIGINAL_NOTE = '<!-- 这段话以下全部，原文都是英文，中文是反向翻译的 -->';

/**
 Original whose archive seals its last paragraph as the English original.
 */
const SEALED_SOURCE_DOC = `## 第一节\n\n猫猫在窗台上睡觉。\n\n${ENGLISH_ORIGINAL_NOTE}\n\n五月四日，猫猫打了个盹。\n`;

/**
 Archive English whose last paragraph writes a date the page would put month
 first, were it not sealed.
 */
const SEALED_TARGET_DOC
  = `## Section one\n\nThe cat sleeps on the sill.\n\n${ENGLISH_ORIGINAL_NOTE}\n\nOn 4 May the cat napped.\n`;

/**
 Entry every case settles.
 */
const ENTRY = 'CatEntry1';

/**
 Logger the republish lines go through.
 */
const l = tagged({ tag: 'page-republish-test', },);

/**
 Raised by a pair reader whose corpus cannot be reached.
 */
class LitterBoxClosedError extends Error {
  /**
   Names the class for the refusal the republish records.
   */
  override readonly name = 'LitterBoxClosedError';
}

/**
 A throwaway runs directory holding one settled artifact over a pair, and
 where its page goes.

 @param strip - preparation keys to delete from the artifact, which is how a
 file written before those fields existed looks

 @param sourceText - original the artifact is settled over

 @param targetText - archive English the artifact is settled over

 @returns Artifact and page roots, and the page path, removed when its
 `await using` scope ends

 @example
 ```ts
 await using run = await settledRunOver({ strip: [], sourceText: SOURCE_DOC, targetText: TARGET_DOC, },);
 ```
 */
async function settledRunOver(
  {
    strip,
    sourceText,
    targetText,
  }: {
    readonly strip: readonly string[];
    readonly sourceText: string;
    readonly targetText: string;
  },
): Promise<{
  readonly artifactsDir: string;
  readonly publishDir: string;
  readonly pagePath: string;
} & AsyncDisposable> {
  // The runs directory.
  return await scratchDirWith({
    prefix: 'page-republish-',
    setup: async function seeded({ path: runsDir, },): Promise<{
      readonly artifactsDir: string;
      readonly publishDir: string;
      readonly pagePath: string;
    }> {
      /**
       Where the artifact lives.
       */
      const artifactsDir = join(
        runsDir,
        'artifacts',
      );
      await mkdir(artifactsDir,);
      /**
       The artifact as the builder writes it, over the carve a current pass makes.
       */
      const written = JSON.parse(settledArtifactText({
        prepared: prepareDocumentPair({
          sourceText,
          targetText,
          includeFrontMatter: true,
          sealArchiveOriginal: true,
        },),
        entryId: ENTRY,
      },),) as { readonly preparation: Record<string, unknown>; };
      await writeFile(
        join(
          artifactsDir,
          `${ENTRY}.json`,
        ),
        JSON.stringify({
          ...written,
          preparation: Object.fromEntries(Object.entries(written.preparation,)
            .filter(function kept([key,],): boolean {
              return !strip.includes(key,);
            },),),
        },),
      );
      /**
       Root of the mirrored tree.
       */
      const publishDir = join(
        runsDir,
        'fixed',
      );
      return {
        artifactsDir,
        publishDir,
        pagePath: join(
          publishDir,
          'people',
          ENTRY,
          'page.en.md',
        ),
      };
    },
  },);
}

/**
 A throwaway runs directory holding one settled artifact over the pair every
 case but the sealed one settles, and where its page goes.

 @param strip - preparation keys to delete from the artifact

 @returns Artifact and page roots, and the page path, removed when its
 `await using` scope ends

 @example
 ```ts
 await using run = await settledRun({ strip: [], },);
 ```
 */
async function settledRun(
  { strip, }: { readonly strip: readonly string[]; },
): Promise<Awaited<ReturnType<typeof settledRunOver>>> {
  return await settledRunOver({
    strip,
    sourceText: SOURCE_DOC,
    targetText: TARGET_DOC,
  },);
}

/**
 A pair reader handing back fixed text.

 @param sourceText - original to hand back

 @param targetText - archive English to hand back

 @returns Reader

 @example
 ```ts
 const readPair = pairOf({ sourceText: SOURCE_DOC, targetText: TARGET_DOC, },);
 ```
 */
function pairOf(
  {
    sourceText,
    targetText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
  },
): CorpusPairReader {
  return async function readPair() {
    return {
      sourceText,
      targetText,
    };
  };
}

/**
 The reader for the pair every artifact here was settled over.
 */
const SAME_PAIR = pairOf({
  sourceText: SOURCE_DOC,
  targetText: TARGET_DOC,
},);

/**
 Runs the republish over one runs directory and returns the entry's outcome.

 @param run - directories to read and write

 @param readPair - reader for the entry's pair

 @returns One entry's outcome

 @example
 ```ts
 const outcome = await republishIn({ run, readPair: SAME_PAIR, },);
 ```
 */
async function republishIn(
  {
    run,
    readPair,
  }: {
    readonly run: Awaited<ReturnType<typeof settledRun>>;
    readonly readPair: CorpusPairReader;
  },
): Promise<RepublishOutcome> {
  /**
   One row, for the one entry.
   */
  const [row,] = await republishSettledPages({
    entryIds: [ENTRY,],
    artifactsDir: run.artifactsDir,
    publishDir: run.publishDir,
    readPair,
    l,
  },);
  if (row === undefined)
    throw new Error('republishSettledPages returned no row for the one entry it was given',);
  return row.outcome;
}

/**
 Whether the page on disk carries what the artifact ships.

 @param run - directories holding both

 @returns Verdict

 @example
 ```ts
 expect(await verdictOf({ run, },),).toBe('agreed-weighed',);
 ```
 */
async function verdictOf(
  { run, }: { readonly run: Awaited<ReturnType<typeof settledRun>>; },
): Promise<string> {
  /**
   The artifact as a reader parses it.
   */
  const artifact = parseSettledTwoLaneArtifact({
    value: JSON.parse(await readFile(
      join(
        run.artifactsDir,
        `${ENTRY}.json`,
      ),
      'utf8',
    ),) as unknown,
  },);
  return pageAgreement({
    artifact,
    pageText: await readFile(
      run.pagePath,
      'utf8',
    ),
  },).agreement;
}

await describe({
  name: republishSettledPages.name,
  children: [
    it({
      name: 'WRITES a missing page from its artifact, and the page then agrees with it',
      fn: async () => {
        await using run = await settledRun({ strip: [], },);

        expect(await republishIn({
          run,
          readPair: SAME_PAIR,
        },),).toStrictEqual({
          kind: 'republished',
          why: 'missing',
        },);
        expect(await verdictOf({ run, },),).toBe('agreed-weighed',);
      },
    },),
    it({
      name: 'WRITES THE SAME PAGE when the pair reader also hands the entry\'s info.yml record, which adds a date to '
        + 'the identity block of the rebuilt preparation and no wording to the page',
      fn: async () => {
        await using bare = await settledRun({ strip: [], },);
        await using informed = await settledRun({ strip: [], },);
        await republishIn({
          run: bare,
          readPair: SAME_PAIR,
        },);
        expect(await republishIn({
          run: informed,
          readPair: async function readPairWithEntryInfo() {
            return {
              sourceText: SOURCE_DOC,
              targetText: TARGET_DOC,
              sourceEntryInfo: { info: { died: '2025-02-26', }, },
            };
          },
        },),).toStrictEqual({
          kind: 'republished',
          why: 'missing',
        },);
        expect(await readFile(
          informed.pagePath,
          'utf8',
        ),).toBe(await readFile(
          bare.pagePath,
          'utf8',
        ),);
      },
    },),
    it({
      name: 'LEAVES an agreeing page as it is, the control that a rewrite is not every page\'s fate',
      fn: async () => {
        await using run = await settledRun({ strip: [], },);
        await republishIn({
          run,
          readPair: SAME_PAIR,
        },);
        /**
         The page the first pass wrote.
         */
        const written = await readFile(
          run.pagePath,
          'utf8',
        );

        expect(await republishIn({
          run,
          readPair: SAME_PAIR,
        },),).toStrictEqual({ kind: 'agreed', },);
        expect(await readFile(
          run.pagePath,
          'utf8',
        ),).toBe(written,);
      },
    },),
    it({
      name: 'REWRITES a page that disagrees with its artifact back to what the artifact ships',
      fn: async () => {
        await using run = await settledRun({ strip: [], },);
        await republishIn({
          run,
          readPair: SAME_PAIR,
        },);
        /**
         The page as it should stand.
         */
        const written = await readFile(
          run.pagePath,
          'utf8',
        );
        await writeFile(
          run.pagePath,
          written.replace(
            'The cat has a bowl.',
            'The cat has.',
          ),
        );
        expect(await verdictOf({ run, },),).toBe('disagreed',);

        expect(await republishIn({
          run,
          readPair: SAME_PAIR,
        },),).toStrictEqual({
          kind: 'republished',
          why: 'disagreed',
        },);
        expect(await readFile(
          run.pagePath,
          'utf8',
        ),).toBe(written,);
      },
    },),
    it({
      name: 'LEAVES A PAGE THE COMPOSITION SHIPS WITHOUT THE ORIGINAL OF A SLICE, which reads as agreed against the '
        + 'artifact without that original and is not published again',
      fn: async () => {
        await using run = await settledRun({ strip: [], },);
        const {
          artifactText,
          pageText,
        } = sourceDroppedRunOf({ entryId: ENTRY, },);
        await writeFile(
          join(
            run.artifactsDir,
            `${ENTRY}.json`,
          ),
          artifactText,
        );
        await mkdir(
          dirname(run.pagePath,),
          { recursive: true, },
        );
        await writeFile(
          run.pagePath,
          pageText,
        );
        expect(await republishIn({
          run,
          readPair: SAME_PAIR,
        },),).toStrictEqual({ kind: 'agreed', },);
        expect(await readFile(
          run.pagePath,
          'utf8',
        ),).toBe(pageText,);
        expect(await verdictOf({ run, },),).toBe('agreed-weighed',);
      },
    },),
    it({
      name: 'SPLICES INTO THE ARCHIVE THE ARTIFACT STORED, not the corpus copy, which the pass reshapes before it carves',
      fn: async () => {
        await using run = await settledRun({ strip: [], },);

        expect(await republishIn({
          run,
          readPair: pairOf({
            sourceText: SOURCE_DOC,
            targetText: `${TARGET_DOC}\nA stray corpus line the run never carved.\n`,
          },),
        },),).toStrictEqual({
          kind: 'republished',
          why: 'missing',
        },);
        expect((await readFile(
          run.pagePath,
          'utf8',
        )).includes('stray corpus line',),).toBe(false,);
        expect(await verdictOf({ run, },),).toBe('agreed-weighed',);
      },
    },),
    it({
      name: 'REPUBLISHES A PAGE WHOSE ARCHIVE SEALS ITS LAST PARAGRAPH AS THE ENGLISH ORIGINAL, the spans the '
        + 'artifact stored reaching the publish check, which finds nothing lost',
      fn: async () => {
        await using run = await settledRunOver({
          strip: [],
          sourceText: SEALED_SOURCE_DOC,
          targetText: SEALED_TARGET_DOC,
        },);
        /**
         Every line the republish logged, behind its level.
         */
        const lines: string[] = [];

        expect(await republishSettledPages({
          entryIds: [ENTRY,],
          artifactsDir: run.artifactsDir,
          publishDir: run.publishDir,
          readPair: pairOf({
            sourceText: SEALED_SOURCE_DOC,
            targetText: SEALED_TARGET_DOC,
          },),
          l: levelCapturingLogger({ lines, },),
        },),).toStrictEqual([{
          entryId: ENTRY,
          outcome: {
            kind: 'republished',
            why: 'missing',
          },
        },],);
        expect(await readFile(
          run.pagePath,
          'utf8',
        ),).toBe(SEALED_TARGET_DOC,);
        expect(lines,).toEqual([
          'info publish: wrote 1 slice into a page of 104 characters',
          'info REPUBLISHED entry=CatEntry1 why=missing: page rewritten from its artifact',
          'info republish: 1 settled page judged, 1 rewritten, 0 left',
        ],);
      },
    },),
    it({
      name: 'FALLS BACK TO THE CORPUS COPY for an artifact that predates storing the archive',
      fn: async () => {
        await using run = await settledRun({ strip: ['archiveText',], },);

        expect(await republishIn({
          run,
          readPair: SAME_PAIR,
        },),).toStrictEqual({
          kind: 'republished',
          why: 'missing',
        },);
        expect(await verdictOf({ run, },),).toBe('agreed-unweighed',);
      },
    },),
    it({
      name: 'LEAVES a page whose original no longer carves as the run carved it, saying where it moved',
      fn: async () => {
        await using run = await settledRun({ strip: [], },);

        /**
         What became of the page.
         */
        const outcome = await republishIn({
          run,
          readPair: pairOf({
            sourceText: SOURCE_DOC.replace(
              '猫猫有自己的碗。',
              '猫猫有两个碗。',
            ),
            targetText: TARGET_DOC,
          },),
        },);
        expect(outcome.kind,).toBe('left',);
        expect((outcome.kind === 'left') && outcome.because.startsWith('carve moved:',),).toBe(true,);
        await expect(readFile(
          run.pagePath,
          'utf8',
        ),).rejects.toThrow('ENOENT',);
      },
    },),
    it({
      name: 'LEAVES a page when the pair cannot be read, naming the class and never its message',
      fn: async () => {
        await using run = await settledRun({ strip: [], },);

        expect(await republishIn({
          run,
          readPair: async function closed() {
            throw new LitterBoxClosedError('the corpus clone is shut for the night',);
          },
        },),).toStrictEqual({
          kind: 'left',
          why: 'missing',
          because: 'refused by LitterBoxClosedError',
        },);
      },
    },),
    it({
      name: 'LEAVES a page whose pair is at a commit the clone lacks, saying the kind of the read\'s failure and '
        + 'its remedy in the refusal\'s own words rather than its class name',
      fn: async () => {
        await using run = await settledRun({ strip: [], },);

        expect(await republishIn({
          run,
          readPair: async function atLackedCommit({
            entryId,
            corpusSha,
          },) {
            throw new CorpusReadError({
              detail: `${corpusSha}:people/${entryId}/page.md`,
              cause: { stderr: `fatal: path 'people/${entryId}/page.md' does not exist in '${corpusSha}'`, },
              commit: 'lacking',
            },);
          },
        },),).toStrictEqual({
          kind: 'left',
          why: 'missing',
          because: `corpus read failed for ${'b'.repeat(40,)}:people/${ENTRY}/page.md (missing-commit); the clone `
            + 'holds no commit by that revision: fetch it, or pin a commit the clone holds.',
        },);
      },
    },),
    it({
      name: 'LEAVES an artifact it cannot read, and goes on',
      fn: async () => {
        await using run = await settledRun({ strip: [], },);
        await writeFile(
          join(
            run.artifactsDir,
            `${ENTRY}.json`,
          ),
          '{"id":',
        );

        /**
         What became of the page.
         */
        const outcome = await republishIn({
          run,
          readPair: SAME_PAIR,
        },);
        expect(outcome.kind,).toBe('left',);
        expect((outcome.kind === 'left') && outcome.why,).toBe('unreadable',);
      },
    },),
  ],
},);
