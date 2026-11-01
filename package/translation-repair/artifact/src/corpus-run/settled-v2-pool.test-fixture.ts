/**
 Settled version 2 artifacts written into a throwaway runs directory, for the
 cases of the damage command that read a pool.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure';
import { compareLanes, } from '@monochromatic-dev/translation-repair-artifact-pool';
import { parseSettledTwoLaneArtifact, } from '../../dist/final/node/index.mjs';
import { settledArtifactText, } from './settled-artifact.test-fixture.ts';

//region Settled version 2 pool
// An artifact of two lanes records, per lane, a delivery ledger with one row
// per slice, and a comparison with one row per slice naming what each lane
// delivered. The builder's artifact over an archive that is kept whole ships
// no replacement; a case that wants a shipped region rewrites one slice of
// both ledgers and of the comparison to say a lane replaced the archive's
// wording there.

/**
 Original with two sections.
 */
const SOURCE_DOC = '## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n猫猫有自己的碗。\n';

/**
 Archive English of the same shape.
 */
const TARGET_DOC = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n';

/**
 What the repair lane adds to the archive's wording of a slice it replaces.
 */
export const SHIPPED_ADDITION = ' It purrs.';

/**
 Writes one settled version 2 artifact over a two-section page.

 @param runsDir - throwaway runs directory

 @param entryId - corpus entry the artifact is for, which names its file

 @param shippedSlices - how many slices, from the first, the repair lane ships a replacement for, each making one region a draw can ask about

 @example
 ```ts
 await writeSettledV2({ runsDir, entryId: 'mittens', shippedSlices: 1, },);
 ```
 */
export async function writeSettledV2(
  {
    runsDir,
    entryId,
    shippedSlices,
  }: {
    readonly runsDir: string;
    readonly entryId: string;
    readonly shippedSlices: number;
  },
): Promise<void> {
  /**
   Directory every settled artifact of the run sits in.
   */
  const artifactsDir = join(
    runsDir,
    'artifacts',
  );
  await mkdir(
    artifactsDir,
    { recursive: true, },
  );

  /**
   The artifact as the builder writes it.
   */
  const text = settledArtifactText({
    prepared: prepareDocumentPair({
      sourceText: SOURCE_DOC,
      targetText: TARGET_DOC,
      includeFrontMatter: true,
      sealArchiveOriginal: true,
    },),
    entryId,
  },);
  await writeFile(
    join(
      artifactsDir,
      `${entryId}.json`,
    ),
    (shippedSlices === 0)
      ? text
      : withShippedSlices({
        text,
        count: shippedSlices,
      },),
    'utf8',
  );
}

/**
 Reads one part of an artifact the fixture rewrites, as a record.

 @param parent - record holding the part

 @param key - field naming the part

 @returns The part

 @throws Error when the part is not an object, which means the builder wrote an artifact this fixture does not know

 @example
 ```ts
 const lanes = partOf({ parent: written, key: 'lanes', },);
 ```
 */
function partOf(
  {
    parent,
    key,
  }: {
    readonly parent: Readonly<Record<string, unknown>>;
    readonly key: string;
  },
): Readonly<Record<string, unknown>> {
  /**
   What the field holds.
   */
  const part = parent[key];
  if (!isJsonRecord(part,))
    throw new Error(`the builder wrote an artifact whose ${key} is not an object`,);
  return part;
}

/**
 Puts a repair ledger, the raw result it must agree with and the comparison
 derived from the ledgers into the artifact as the builder wrote it, leaving
 every other field as written.

 @param written - the artifact as parsed from the builder's text

 @param repair - repair lane's ledger to record

 @param comparison - comparison the two ledgers derive

 @param shipped - ledger rows that shipped a replacement, which the raw result must say changed

 @returns The artifact with those parts replaced

 @throws Error when the artifact is not shaped as the builder writes one

 @example
 ```ts
 const artifact = replacedLedger({ written, repair, comparison, shipped, },);
 ```
 */
function replacedLedger(
  {
    written,
    repair,
    comparison,
    shipped,
  }: {
    readonly written: unknown;
    readonly repair: readonly {
      readonly sliceIndex: number;
      readonly incumbentText: string;
      readonly shippedText: string;
    }[];
    readonly comparison: unknown;
    readonly shipped: ReadonlySet<number>;
  },
): unknown {
  if (!isJsonRecord(written,))
    throw new Error('the builder wrote an artifact that is not an object',);

  /**
   Both lanes of the artifact.
   */
  const lanes = partOf({
    parent: written,
    key: 'lanes',
  },);

  /**
   The repair lane.
   */
  const repairLane = partOf({
    parent: lanes,
    key: 'repair',
  },);

  /**
   The repair lane's raw result.
   */
  const result = partOf({
    parent: repairLane,
    key: 'result',
  },);

  /**
   The page text the raw result carries.
   */
  const {
    repairedText: archiveText,
    sliceTexts,
  } = result;
  if (((typeof archiveText) !== 'string') || (!Array.isArray(sliceTexts,)))
    throw new Error('the builder wrote a repair result without the page text and slice evidence',);

  /**
   The page text with each shipped slice's wording put in place of the archive's.
   */
  const repairedText = repair.reduce(
    function replaced(
      text,
      row,
    ): string {
      return shipped.has(row.sliceIndex,)
        ? text.replace(
          row.incumbentText,
          row.shippedText,
        )
        : text;
    },
    archiveText,
  );

  /**
   The slice evidence with the shipped slices' accepted wording changed.
   */
  const evidence = sliceTexts.map(function evidenceOf(
    row: unknown,
    index: number,
  ): unknown {
    /**
     The ledger row of this slice, which names the wording that shipped.
     */
    const ledgerRow = repair.at(index,);
    if (!shipped.has(index,))
      return row;
    if ((!isJsonRecord(row,)) || (ledgerRow === undefined))
      return row;
    return {
      ...row,
      outcome: {
        kind: 'decided',
        acceptedText: ledgerRow.shippedText,
      },
    };
  },);

  return {
    ...written,
    lanes: {
      ...lanes,
      repair: {
        ...repairLane,
        result: {
          ...result,
          repairedText,
          status: 'repaired',
          changedSliceIndices: [...shipped,],
          sliceTexts: evidence,
        },
        delivery: repair,
      },
    },
    comparison,
  };
}

/**
 Rewrites an artifact so the repair lane ships a replacement at its first
 slices, in the lane's ledger, in its raw result and in the comparison the
 two ledgers derive.

 @param text - artifact text as the builder wrote it

 @param count - how many slices, from the first, the lane ships a replacement for

 @returns Artifact text saying the repair lane replaced the archive's wording there

 @example
 ```ts
 const shipped = withShippedSlices({ text, count: 1, },);
 ```
 */
function withShippedSlices(
  {
    text,
    count,
  }: {
    readonly text: string;
    readonly count: number;
  },
): string {
  /**
   The artifact, read by the reader the damage command uses, which hands back
   the ledgers as typed rows.
   */
  const written: unknown = JSON.parse(text,);

  /**
   The ledgers of both lanes, as typed rows.
   */
  const parsed = parseSettledTwoLaneArtifact({ value: written, },);

  /**
   The ledger rows of each lane.
   */
  const {
    repair: repairLane,
    translate: translateLane,
  } = parsed.lanes;

  /**
   The repair lane's ledger rows.
   */
  const { delivery: repairRows, } = repairLane;

  /**
   The translate lane's ledger rows.
   */
  const { delivery: translateRows, } = translateLane;

  /**
   The repair ledger saying the lane replaced the archive's wording at the first slices.
   */
  const repair = repairRows.map(function shippedAt(
    row,
    index,
  ) {
    if (index >= count)
      return row;
    return {
      ...row,
      outcome: {
        kind: 'decided' as const,
        acceptedText: `${row.incumbentText}${SHIPPED_ADDITION}`,
      },
      shippedText: `${row.incumbentText}${SHIPPED_ADDITION}`,
      delivery: { kind: 'replacement-shipped' as const, },
    };
  },);

  /**
   Slice indexes of the rows that shipped a replacement.
   */
  const shipped = new Set(repair
    .filter(function wasShipped({ delivery, },): boolean {
      return delivery.kind === 'replacement-shipped';
    },)
    .map(function indexOf({ sliceIndex, },): number {
      return sliceIndex;
    },),);
  return JSON.stringify(replacedLedger({
    written,
    repair,
    shipped,
    comparison: compareLanes({
      repair,
      translate: translateRows,
    },),
  },),);
}

//endregion Settled version 2 pool
