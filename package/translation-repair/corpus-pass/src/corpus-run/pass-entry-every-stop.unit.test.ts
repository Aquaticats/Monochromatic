/**
 Tests for the page every stop of an entry ships (B1220 to B1229): the owner's rule is "no matter what, we must ship",
 and the shipper once answered for a run-ending refusal and a stage that heard no voice alone, so XingZ6023 spent its
 whole budget and shipped no page because one guard threw.

 Each case drives `shipInterimOnStop` over the error a real stop raises, with the progress the entry held at the time.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NaturalnessRepairInterruptedError,
  prepareDocumentPair,
} from '@monochromatic-dev/translation-repair-adjudication';
import type { ArtifactConsolidateSlice, } from '@monochromatic-dev/translation-repair-artifact';
import { catRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/two-lane-comparison-row.test-fixture.ts';
import {
  type PipelineDigest,
  readInterimEntries,
} from '@monochromatic-dev/translation-repair-glossary';
import {
  fixedPagePath,
  UnansweredContestSliceError,
} from '@monochromatic-dev/translation-repair-page-check';
import { SliceSpliceError, } from '@monochromatic-dev/translation-repair-page-pass';
import { EveryProviderDryError, } from '@monochromatic-dev/translation-repair-provider';
import { PromptPayloadStoreError, } from '@monochromatic-dev/translation-repair-stage';
import { TranslationRepairInterruptedError, } from '@monochromatic-dev/translation-repair-translate-stage';
import {
  shipInterimOnStop,
  startEntryProgress,
} from '../../dist/final/node/index.mjs';
import { VisualEvidenceInterruptedError, } from '@monochromatic-dev/translation-repair-slice-cache';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Original with two sections.
 */
const SOURCE_DOC = '## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n猫猫有自己的碗。\n';

/**
 Archive English of the same shape.
 */
const TARGET_DOC = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n';

/**
 Wording a consolidation record settled for the first section.
 */
const SETTLED_FIRST = '## Section one\n\nThe cat naps on the sill.\n';

/**
 A page whose front matter is no YAML, which the deterministic preparation refuses.
 */
const BROKEN_FRONT_MATTER = '---\n: [\n---\nThe cat sleeps.\n';

/**
 Built pipeline the cases claim to have run under.
 */
const DIGEST = `sha256-tree-v1:${'c'.repeat(64,)}` as PipelineDigest;

/**
 Preparation the cases slice by, as the shipper's own deterministic slicing makes it.
 */
function preparation(): ReturnType<typeof prepareDocumentPair> {
  return prepareDocumentPair({
    sourceText: SOURCE_DOC,
    targetText: TARGET_DOC,
    includeFrontMatter: true,
    sealArchiveOriginal: true,
  },);
}

/**
 Progress of an entry that was sliced and went no further.
 */
function slicedProgress(): ReturnType<typeof startEntryProgress> {
  /**
   Preparation the entry was sliced by.
   */
  const prepared = preparation();
  return {
    ...startEntryProgress(),
    prepared,
    archiveText: prepared.targetText,
  };
}

/**
 Record the consolidation left at a slice, shipping the wording it consolidated.

 @param sliceIndex - slice the record answers

 @param shipped - what the record ships

 @returns Record

 @example
 ```ts
 const record = recordAt({ sliceIndex: 0, shipped: { kind: 'consolidated', text: SETTLED_FIRST, }, },);
 ```
 */
function recordAt(
  {
    sliceIndex,
    shipped,
  }: {
    readonly sliceIndex: number;
    readonly shipped: ArtifactConsolidateSlice['shipped'];
  },
): ArtifactConsolidateSlice {
  return {
    sliceIndex,
    terminal: 'consolidated',
    shipped,
    rewrapped: false,
    demoted: false,
    verdicts: [],
    gate: { kind: 'not-asked', },
  };
}

/**
 Where one case's pages and records go.
 */
type Dirs = {
  readonly publishDir: string;
  readonly interimDir: string;
  readonly rootPath: string;
};

/**
 Ships over a scratch tree with the arguments every case shares.

 @param dirs - scratch tree

 @param error - what stopped the entry

 @param progress - what the entry had finished

 @param messages - collects the log lines

 @returns Nothing; the shipper never throws

 @example
 ```ts
 await shipOver({ dirs, error, progress, messages, },);
 ```
 */
async function shipOver(
  {
    dirs,
    error,
    progress,
    messages,
  }: {
    readonly dirs: Dirs;
    readonly error: unknown;
    readonly progress: ReturnType<typeof startEntryProgress>;
    readonly messages: string[];
  },
): Promise<void> {
  await shipInterimOnStop({
    error,
    progress,
    entryId: 'CatEntry1',
    sourceText: SOURCE_DOC,
    targetText: TARGET_DOC,
    publishDir: dirs.publishDir,
    interimDir: dirs.interimDir,
    tip: 'a'.repeat(40,),
    pipelineDigest: DIGEST,
    corpusSha: 'b'.repeat(40,),
    attempt: 1,
    l: capturingLogger({ messages, },),
  },);
}

/**
 Reads the page of the case's entry.

 @param dirs - scratch tree

 @returns Page text

 @example
 ```ts
 const page = await pageOf({ dirs, },);
 ```
 */
async function pageOf({ dirs, }: { readonly dirs: Dirs; },): Promise<string> {
  return await readFile(
    fixedPagePath({
      publishDir: dirs.publishDir,
      entryId: 'CatEntry1',
    },),
    'utf8',
  );
}

/**
 Makes the scratch tree of one case.

 @param root - scratch root

 @returns Where its pages and records go

 @example
 ```ts
 const dirs = dirsOf({ root: root.path, },);
 ```
 */
function dirsOf({ root, }: { readonly root: string; },): Dirs {
  return {
    rootPath: root,
    publishDir: join(
      root,
      'fixed',
    ),
    interimDir: join(
      root,
      'interim',
    ),
  };
}

await describe({
  name: 'the page every stop of an entry ships',
  children: [
    it({
      name: 'SHIPS THE INTERIM PAGE for an operational error that is none of the two stops the shipper once answered '
        + 'for, recording the reason `entry-error` and the class of the error, and writes no artifact',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-every-stop-', },);
        const dirs = dirsOf({ root: root.path, },);
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: slicedProgress(),
          messages: [],
        },);
        expect(await pageOf({ dirs, },),).toBe(TARGET_DOC,);
        /**
         The record the stop left.
         */
        const record = (await readInterimEntries({ interimDir: dirs.interimDir, },)).get('CatEntry1',);
        expect({ reason: record?.reason, errorClass: record?.errorClass, },).toEqual({
          reason: 'entry-error',
          errorClass: 'RangeError',
        },);
        expect(await readdir(root.path,),).toEqual(['fixed', 'interim',],);
      },
    },),

    ...([
      new UnansweredContestSliceError({ sliceIndex: 1, },),
      new SliceSpliceError({ message: 'two replacements name one slice', },),
      new NaturalnessRepairInterruptedError({ reason: 'contributor-structure', },),
      new PromptPayloadStoreError({
        promptDigest: 'fixture-digest',
        operation: 'write',
        reason: 'the disk is full',
      },),
      new VisualEvidenceInterruptedError({ unavailableCount: 1, },),
      new TranslationRepairInterruptedError({
        reason: 'provider-unavailable',
        findings: [],
      },),
      new EveryProviderDryError({ measured: 'the meters of the cases read dry', },),
    ] as const).map(function shippedClass(error,) {
      return it({
        name: `SHIPS A PAGE FOR ${error.name} after the slicing and keeps its class in the record`,
        fn: async () => {
          await using root = await scratchDir({ prefix: 'pass-entry-every-stop-', },);
          const dirs = dirsOf({ root: root.path, },);
          await shipOver({
            dirs,
            error,
            progress: slicedProgress(),
            messages: [],
          },);
          expect(await pageOf({ dirs, },),).toBe(TARGET_DOC,);
          expect((await readInterimEntries({ interimDir: dirs.interimDir, },)).get('CatEntry1',)?.errorClass,)
            .toBe(error.name,);
        },
      },);
    },),

    it({
      name: 'KEEPS THE ARCHIVE AT THE SLICE THE CONTEST NEVER ANSWERED and ships the settled wording of the other, when '
        + 'the reading of what ships throws `UnansweredContestSliceError` inside the interim page itself, which the '
        + 'shipper once met by shipping nothing',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-every-stop-', },);
        const dirs = dirsOf({ root: root.path, },);
        /**
         Preparation both slices of the page address.
         */
        const prepared = preparation();
        const [firstSlice, secondSlice,] = prepared.slices;
        if ((firstSlice === undefined) || (secondSlice === undefined))
          throw new Error('the archive has two slices',);
        /**
         The slice the contest was bound to decide and named nowhere.
         */
        const unanswered = secondSlice.target.sliceIndex;
        await shipOver({
          dirs,
          error: new UnansweredContestSliceError({ sliceIndex: unanswered, },),
          progress: {
            ...slicedProgress(),
            projected: {
              comparison: [
                {
                  ...catRow({
                    sliceIndex: firstSlice.target.sliceIndex,
                    repairText: SETTLED_FIRST,
                    translateText: SETTLED_FIRST,
                  },),
                  incumbentText: firstSlice.target.text,
                },
                {
                  ...catRow({
                    sliceIndex: unanswered,
                    repairText: 'The cat has a dish.',
                    translateText: 'The cat has a saucer.',
                  },),
                  incumbentText: secondSlice.target.text,
                },
              ],
            },
            contestSlices: [],
            consolidated: [
              recordAt({
                sliceIndex: firstSlice.target.sliceIndex,
                shipped: {
                  kind: 'consolidated',
                  text: SETTLED_FIRST,
                },
              },),
              recordAt({
                sliceIndex: unanswered,
                shipped: { kind: 'unchanged', },
              },),
            ],
          },
          messages: [],
        },);
        expect(await pageOf({ dirs, },),).toBe(`${SETTLED_FIRST}\n## Section two\n\nThe cat has a bowl.\n`,);
      },
    },),

    it({
      name: 'SHIPS THE ARCHIVE\'S OWN TEXT, and names the fault in the log, when even the interim page cannot be '
        + 'composed because the comparison names one slice twice (`SliceSpliceError` inside the splice)',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-every-stop-', },);
        const dirs = dirsOf({ root: root.path, },);
        const messages: string[] = [];
        /**
         Preparation both slices of the page address.
         */
        const prepared = preparation();
        const first = prepared.slices[0]?.target.sliceIndex ?? 0;
        /**
         One row, which the comparison below names twice.
         */
        const row = {
          ...catRow({
            sliceIndex: first,
            repairText: SETTLED_FIRST,
            translateText: SETTLED_FIRST,
          },),
          incumbentText: prepared.slices[0]?.target.text ?? '',
        };
        await shipOver({
          dirs,
          error: new SliceSpliceError({ message: 'two replacements name one slice', },),
          progress: {
            ...slicedProgress(),
            projected: { comparison: [row, row,], },
            contestSlices: [],
            consolidated: [
              recordAt({
                sliceIndex: first,
                shipped: {
                  kind: 'consolidated',
                  text: SETTLED_FIRST,
                },
              },),
            ],
          },
          messages,
        },);
        expect(await pageOf({ dirs, },),).toBe(TARGET_DOC,);
        expect(messages.some(function namesTheFault(message,): boolean {
          return message.includes('SliceSpliceError',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'SHIPS THE ARCHIVE AS THE PAGE for a stop before the slicing exists, cut by the deterministic preparation of '
        + 'the two texts the entry was read from, and records it as interim with the class of the error',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-every-stop-', },);
        const dirs = dirsOf({ root: root.path, },);
        await shipOver({
          dirs,
          error: new VisualEvidenceInterruptedError({ unavailableCount: 1, },),
          progress: startEntryProgress(),
          messages: [],
        },);
        expect(await pageOf({ dirs, },),).toBe(TARGET_DOC,);
        /**
         The record the stop left.
         */
        const record = (await readInterimEntries({ interimDir: dirs.interimDir, },)).get('CatEntry1',);
        expect({
          reason: record?.reason,
          errorClass: record?.errorClass,
          counts: [record?.settledSlices, record?.archiveSlices, record?.sourceSlices,],
        },).toEqual({
          reason: 'entry-error',
          errorClass: 'VisualEvidenceInterruptedError',
          counts: [0, preparation().slices.length, 0,],
        },);
      },
    },),

    it({
      name: 'SHIPS THE REVIEWED ARCHIVE when the stop comes after the preparation returned and before the fold, since '
        + 'the page the preparation corrected is the archive the entry reads from there on',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-every-stop-', },);
        const dirs = dirsOf({ root: root.path, },);
        /**
         Preparation over an archive the review corrected.
         */
        const corrected = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC.replace(
            'sleeps',
            'naps',
          ),
          includeFrontMatter: true,
          sealArchiveOriginal: true,
        },);
        await shipOver({
          dirs,
          error: new VisualEvidenceInterruptedError({ unavailableCount: 1, },),
          progress: {
            ...startEntryProgress(),
            prepared: corrected,
            archiveText: corrected.targetText,
          },
          messages: [],
        },);
        expect(await pageOf({ dirs, },),).toBe(TARGET_DOC.replace(
          'sleeps',
          'naps',
        ),);
      },
    },),

    it({
      name: 'SHIPS THE RAW ARCHIVE when the deterministic preparation of the texts cannot be made either, so a stop '
        + 'that came before anything existed still leaves a page',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-every-stop-', },);
        const dirs = dirsOf({ root: root.path, },);
        const messages: string[] = [];
        await shipInterimOnStop({
          error: new Error('scripted',),
          progress: startEntryProgress(),
          entryId: 'CatEntry1',
          // A front matter that is no YAML, which the preparation refuses with FrontMatterParseError.
          sourceText: BROKEN_FRONT_MATTER,
          targetText: BROKEN_FRONT_MATTER,
          publishDir: dirs.publishDir,
          interimDir: dirs.interimDir,
          tip: 'a'.repeat(40,),
          pipelineDigest: DIGEST,
          corpusSha: 'b'.repeat(40,),
          attempt: 1,
          l: capturingLogger({ messages, },),
        },);
        expect(await pageOf({ dirs, },),).toBe(BROKEN_FRONT_MATTER,);
        expect(messages.some(function namesTheClass(message,): boolean {
          return message.includes('class=FrontMatterParseError',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'LEAVES THE SETTLED PAGE ALONE when the stop comes after the settled page was written, since an interim '
        + 'page written over it would replace the better page with the worse and leave an interim record beside it',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-every-stop-', },);
        const dirs = dirsOf({ root: root.path, },);
        await mkdir(
          join(
            dirs.publishDir,
            'people',
            'CatEntry1',
          ),
          { recursive: true, },
        );
        await writeFile(
          fixedPagePath({
            publishDir: dirs.publishDir,
            entryId: 'CatEntry1',
          },),
          'the settled page\n',
        );
        await shipOver({
          dirs,
          error: new Error('the artifact could not be written',),
          progress: {
            ...slicedProgress(),
            settledPageWritten: true,
          },
          messages: [],
        },);
        expect(await pageOf({ dirs, },),).toBe('the settled page\n',);
        expect([...await readInterimEntries({ interimDir: dirs.interimDir, },),],).toEqual([],);
      },
    },),

    it({
      name: 'NAMES THE CLASS ON THE INTERIM LINE, as `class=` after the reason, so the line that reports an interim '
        + 'ship can be counted by class',
      fn: async (ctx,) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using root = await scratchDir({ prefix: 'pass-entry-every-stop-', },);
        const dirs = dirsOf({ root: root.path, },);
        await shipOver({
          dirs,
          error: new SliceSpliceError({ message: 'two replacements name one slice', },),
          progress: slicedProgress(),
          messages: [],
        },);
        expect(printed.lines.some(function namesTheClass(line,): boolean {
          return line.startsWith('INTERIM CatEntry1 ',)
            && line.endsWith(' reason=entry-error class=SliceSpliceError',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'NAMES THE CLASS WHEN THE PAGE COULD NOT BE SHIPPED, in the error line, so a stop that left no page is not '
        + 'a stop that left no trace of what it was',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-every-stop-', },);
        const dirs = dirsOf({ root: root.path, },);
        // A published tree that is a file, so no page can be made under it.
        await writeFile(
          dirs.publishDir,
          'not a directory',
        );
        const messages: string[] = [];
        await shipOver({
          dirs,
          error: new PromptPayloadStoreError({
            promptDigest: 'fixture-digest',
            operation: 'write',
            reason: 'the disk is full',
          },),
          progress: slicedProgress(),
          messages,
        },);
        expect(messages.some(function namesTheClass(message,): boolean {
          return message.includes('class=PromptPayloadStoreError',)
            && message.includes('could not be shipped',);
        },),).toBe(true,);
      },
    },),
  ],
},);
