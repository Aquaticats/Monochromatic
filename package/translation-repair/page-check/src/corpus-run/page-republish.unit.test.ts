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
import {
  CorpusReadError,
  type PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source';
import type { SliceDeliveryRecord, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  type CorpusPairReader,
  pageAgreement,
  type RepublishOutcome,
  republishSettledPages,
} from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

import {
  settledArtifactOverRows,
  settledArtifactText,
} from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-artifact.test-fixture.ts';
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
 Hex digits in the corpus commit every fixture artifact claims.
 */
const COMMIT_HEX_LENGTH = 40;

/**
 A line the strict grammar refuses: a JSX string literal in curly quotes, which no slice floor reads once the wording
 is spliced beside the rest of the page.
 */
const REFUSED_LINE = '\n> <PhotoScroll photos={[\u{201C}a.webp\u{201D}]} />\n';

/**
 The archive of the second section with a brace opened and never closed, so the archive's own page is refused too.
 */
const UNCLOSED_TARGET_DOC
  = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat has { a bowl.\n';

/**
 A page an earlier build wrote: it parses, it keeps the second section's own wording, and it disagrees with the
 artifacts here, whose pages carry the archive's wording or the refused line.
 */
const OLD_PAGE = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat owns a bowl.\n';

/**
 The artifact the builder writes over a preparation, every slice kept as the archive has it.

 @param prepared - preparation the artifact records

 @returns Artifact's JSON text

 @example
 ```ts
 const text = plainArtifactText(prepared,);
 ```
 */
function plainArtifactText(prepared: PreparedDocumentPair,): string {
  return settledArtifactText({
    prepared,
    entryId: ENTRY,
  },);
}

/**
 The artifact whose both lanes decided, at the last slice, the archive's wording followed by a line the strict grammar
 refuses, over a contest that lets agreed wording ship.

 @param prepared - preparation the artifact records

 @returns Artifact's JSON text

 @example
 ```ts
 const text = artifactDecidingRefusedWording(prepared,);
 ```
 */
function artifactDecidingRefusedWording(prepared: PreparedDocumentPair,): string {
  /**
   Slice the refused wording is decided at.
   */
  const refusedAt = prepared.slices.length - 1;
  /**
   Both lanes' rows: the archive kept everywhere but at that slice.
   */
  const rows = prepared.slices.map(function toRow({
    source,
    target,
  },): SliceDeliveryRecord {
    /**
     Wording both lanes decided for this slice.
     */
    const wording = (target.sliceIndex === refusedAt) ? `${target.text}${REFUSED_LINE}` : target.text;
    return {
      sliceIndex: target.sliceIndex,
      sourceText: source.text,
      incumbentKind: 'present',
      incumbentText: target.text,
      outcome: {
        kind: 'decided',
        acceptedText: wording,
      },
      shippedText: wording,
      delivery: (target.sliceIndex === refusedAt) ? { kind: 'replacement-shipped', } : { kind: 'incumbent-retained', },
    };
  },);
  return JSON.stringify({
    ...settledArtifactOverRows({
      prepared,
      entryId: ENTRY,
      corpusSha: 'b'.repeat(COMMIT_HEX_LENGTH,),
      rows,
    },),
    laneSelection: {
      kind: 'contested',
      slices: [],
    },
  },);
}

/**
 A throwaway runs directory holding one settled artifact over a pair, and
 where its page goes.

 @param strip - preparation keys to delete from the artifact, which is how a
 file written before those fields existed looks

 @param sourceText - original the artifact is settled over

 @param targetText - archive English the artifact is settled over

 @param artifactTextOf - writes the artifact over the preparation, the builder's
 plain one where absent

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
    artifactTextOf = plainArtifactText,
  }: {
    readonly strip: readonly string[];
    readonly sourceText: string;
    readonly targetText: string;
    readonly artifactTextOf?: (prepared: PreparedDocumentPair,) => string;
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
      const written = JSON.parse(artifactTextOf(prepareDocumentPair({
        sourceText,
        targetText,
        includeFrontMatter: true,
        sealArchiveOriginal: true,
      },),),) as { readonly preparation: Record<string, unknown>; };
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
    it({
      name: 'LEAVES A PAGE THAT DISAGREES, byte for byte, when the page rebuilt from its artifact would keep the archive '
        + 'at a slice the strict grammar refuses (the owner, 2026-09-27: a page the build will not take stays as it was '
        + 'and is reported), naming the defect and never the wording',
      fn: async () => {
        await using run = await settledRunOver({
          strip: [],
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          artifactTextOf: artifactDecidingRefusedWording,
        },);
        await mkdir(
          dirname(run.pagePath,),
          { recursive: true, },
        );
        await writeFile(
          run.pagePath,
          OLD_PAGE,
        );
        expect(await verdictOf({ run, },),).toBe('disagreed',);

        expect(await republishIn({
          run,
          readPair: SAME_PAIR,
        },),).toStrictEqual({
          kind: 'left',
          why: 'disagreed',
          because: 'the rebuilt page does not parse under the strict grammar (page-grammar-archive-kept), so the page '
            + 'stays as it was',
        },);
        expect(await readFile(
          run.pagePath,
          'utf8',
        ),).toBe(OLD_PAGE,);
      },
    },),
    it({
      name: 'LEAVES A PAGE THAT DISAGREES, byte for byte, when even the archive\'s page is refused, naming '
        + 'page-unparseable',
      fn: async () => {
        await using run = await settledRunOver({
          strip: [],
          sourceText: SOURCE_DOC,
          targetText: UNCLOSED_TARGET_DOC,
        },);
        await mkdir(
          dirname(run.pagePath,),
          { recursive: true, },
        );
        await writeFile(
          run.pagePath,
          OLD_PAGE,
        );
        expect(await verdictOf({ run, },),).toBe('disagreed',);

        expect(await republishIn({
          run,
          readPair: pairOf({
            sourceText: SOURCE_DOC,
            targetText: UNCLOSED_TARGET_DOC,
          },),
        },),).toStrictEqual({
          kind: 'left',
          why: 'disagreed',
          because: 'the rebuilt page does not parse under the strict grammar (page-unparseable), so the page stays as '
            + 'it was',
        },);
        expect(await readFile(
          run.pagePath,
          'utf8',
        ),).toBe(OLD_PAGE,);
      },
    },),
    it({
      name: 'WRITES A MISSING PAGE WITH THE ARCHIVE KEPT at the slice the strict grammar refuses and names the defect, '
        + 'since there is no page to keep and a run always ships (the owner, 2026-09-27)',
      fn: async () => {
        await using run = await settledRunOver({
          strip: [],
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          artifactTextOf: artifactDecidingRefusedWording,
        },);
        /**
         Every line the republish logged, behind its level.
         */
        const lines: string[] = [];

        expect(await republishSettledPages({
          entryIds: [ENTRY,],
          artifactsDir: run.artifactsDir,
          publishDir: run.publishDir,
          readPair: SAME_PAIR,
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
        ),).toBe(TARGET_DOC,);
        expect(lines.some(function namesDefect(line,): boolean {
          return line.startsWith('warn publish: shipping with defect page-grammar-archive-kept:',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'WRITES A MISSING PAGE THE SITE BUILD WILL NOT TAKE when even the archive\'s page is refused, naming '
        + 'page-unparseable in the log',
      fn: async () => {
        await using run = await settledRunOver({
          strip: [],
          sourceText: SOURCE_DOC,
          targetText: UNCLOSED_TARGET_DOC,
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
            sourceText: SOURCE_DOC,
            targetText: UNCLOSED_TARGET_DOC,
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
        ),).toBe(UNCLOSED_TARGET_DOC,);
        expect(lines.some(function namesDefect(line,): boolean {
          return line.startsWith('warn publish: shipping with defect page-unparseable:',);
        },),).toBe(true,);
      },
    },),
  ],
},);
