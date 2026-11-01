/**
 Fixtures the cases of the page a stopped entry ships share: a two-section page, its archive, a scratch tree for the
 published pages and the interim records, and the shipper called over them with the arguments every case shares.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import type { ArtifactConsolidateSlice, } from '@monochromatic-dev/translation-repair-artifact';
import {
  assertPipelineDigest,
  type PipelineDigest,
} from '@monochromatic-dev/translation-repair-glossary';
import { fixedPagePath, } from '@monochromatic-dev/translation-repair-page-check';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  shipInterimOnStop,
  startEntryProgress,
} from '../../dist/final/node/index.mjs';

/**
 Original with two sections.
 */
export const SOURCE_DOC = '## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n猫猫有自己的碗。\n';

/**
 Archive English of the same shape.
 */
export const TARGET_DOC = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n';

/**
 Wording a consolidation record settled for the first section.
 */
export const SETTLED_FIRST = '## Section one\n\nThe cat naps on the sill.\n';

/**
 Wording a consolidation record settled for the second section.
 */
export const SETTLED_SECOND = '\n## Section two\n\nThe cat has a dish.\n';

/**
 Hexadecimal digits of a SHA-256 digest.
 */
const SHA_256_DIGITS = 64;

/**
 Hexadecimal digits of a commit name.
 */
const COMMIT_DIGITS = 40;

/**
 Text of the built pipeline the cases claim to have run under.
 */
const DIGEST_TEXT = `sha256-tree-v1:${'c'.repeat(SHA_256_DIGITS,)}`;
assertPipelineDigest(DIGEST_TEXT,);

/**
 Built pipeline the cases claim to have run under.
 */
export const DIGEST: PipelineDigest = DIGEST_TEXT;

/**
 Repository head the cases claim to have run at.
 */
export const TIP: string = 'a'.repeat(COMMIT_DIGITS,);

/**
 Corpus commit the cases claim to have read the entry at.
 */
export const CORPUS_SHA: string = 'b'.repeat(COMMIT_DIGITS,);

/**
 Entry the cases ship for.
 */
export const ENTRY_ID = 'CatEntry1';

/**
 Preparation the cases slice by, as the shipper's own deterministic slicing makes it.

 @returns Preparation of the two-section page

 @example
 ```ts
 const prepared = preparation();
 ```
 */
export function preparation(): ReturnType<typeof prepareDocumentPair> {
  return prepareDocumentPair({
    sourceText: SOURCE_DOC,
    targetText: TARGET_DOC,
    includeFrontMatter: true,
    sealArchiveOriginal: true,
  },);
}

/**
 Progress of an entry that was sliced, folded and went no further.

 @returns Progress holding the slicing and the archive it addresses

 @example
 ```ts
 const progress = slicedProgress();
 ```
 */
export function slicedProgress(): ReturnType<typeof startEntryProgress> {
  /**
   Preparation the entry was sliced by.
   */
  const prepared = preparation();
  return {
    ...startEntryProgress(),
    prepared,
    archiveText: prepared.targetText,
    folded: true,
  };
}

/**
 Record the consolidation left at a slice, shipping the wording it consolidated.

 @param sliceIndex - slice the record answers

 @param text - wording it consolidated

 @returns Record

 @example
 ```ts
 const record = consolidatedAt({ sliceIndex: 0, text: SETTLED_FIRST, },);
 ```
 */
export function consolidatedAt(
  {
    sliceIndex,
    text,
  }: {
    readonly sliceIndex: number;
    readonly text: string;
  },
): ArtifactConsolidateSlice {
  return {
    gate: { kind: 'not-asked', },
    verdicts: [],
    demoted: false,
    rewrapped: false,
    shipped: {
      kind: 'consolidated',
      text,
    },
    terminal: 'consolidated',
    sliceIndex,
  };
}

/**
 Where one case's pages and records go.
 */
export type Dirs = {
  readonly publishDir: string;
  readonly interimDir: string;
  readonly rootPath: string;
};

/**
 Makes the scratch tree of one case.

 @param root - scratch root

 @returns Where its pages and records go

 @example
 ```ts
 const dirs = dirsOf({ root: root.path, },);
 ```
 */
export function dirsOf({ root, }: { readonly root: string; },): Dirs {
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

/**
 Ships over a scratch tree with the arguments every case shares.

 @param dirs - scratch tree

 @param error - what stopped the entry

 @param progress - what the entry had finished

 @param messages - collects the log lines

 @param sourceText - original the entry was read from, the two-section page by default

 @param targetText - archive the entry was read from, the two-section archive by default

 @example
 ```ts
 await shipOver({ dirs, error, progress, messages, },);
 ```
 */
export async function shipOver(
  {
    dirs,
    error,
    progress,
    messages,
    sourceText = SOURCE_DOC,
    targetText = TARGET_DOC,
  }: {
    readonly dirs: Dirs;
    readonly error: unknown;
    readonly progress: ReturnType<typeof startEntryProgress>;
    readonly messages: string[];
    readonly sourceText?: string;
    readonly targetText?: string;
  },
): Promise<void> {
  await shipInterimOnStop({
    error,
    progress,
    entryId: ENTRY_ID,
    sourceText,
    targetText,
    publishDir: dirs.publishDir,
    interimDir: dirs.interimDir,
    tip: TIP,
    pipelineDigest: DIGEST,
    corpusSha: CORPUS_SHA,
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
export async function pageOf({ dirs, }: { readonly dirs: Dirs; },): Promise<string> {
  return await readFile(
    fixedPagePath({
      publishDir: dirs.publishDir,
      entryId: ENTRY_ID,
    },),
    'utf8',
  );
}

/**
 Writes a page at the entry's path as an earlier attempt, or an earlier pass, left it.

 @param dirs - scratch tree

 @param text - page text

 @example
 ```ts
 await standingPage({ dirs, text: 'the settled page\n', },);
 ```
 */
export async function standingPage(
  {
    dirs,
    text,
  }: {
    readonly dirs: Dirs;
    readonly text: string;
  },
): Promise<void> {
  await mkdir(
    join(
      dirs.publishDir,
      'people',
      ENTRY_ID,
    ),
    { recursive: true, },
  );
  await writeFile(
    fixedPagePath({
      publishDir: dirs.publishDir,
      entryId: ENTRY_ID,
    },),
    text,
  );
}
