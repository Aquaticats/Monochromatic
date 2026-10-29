import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  FIXED_TREE_DIR,
  fixedPagePath,
} from '@monochromatic-dev/translation-repair-page-check/ts';
import {
  type InterimEntryRecord,
  interimPageDigest,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Verify published interim
// HOW `verify-published` READS A PAGE A DRY STOP SHIPPED FOR AN ENTRY THAT IS STILL OPEN. Such a page has no artifact
// to be weighed against, so what it can be checked against is its own record: the digest of the page as it was written.
// A page that still has that digest is the interim page, and the report says so with what it is made of; any other page
// standing at that path is named as disagreeing, since nothing the record says is then true of it.

/**
 Counts a record gives, each with its noun.

 @param count - how many slices

 @param kind - what the slices carry

 @returns The count and its noun

 @example
 ```ts
 const text = slicesOf({ count: 1, kind: 'settled', },); // => '1 settled slice'
 ```
 */
function slicesOf(
  {
    count,
    kind,
  }: {
    readonly count: number;
    readonly kind: string;
  },
): string {
  return `${String(count,)} ${kind} ${
    wordForCount({
      count,
      one: 'slice',
      many: 'slices',
    },)
  }`;
}

/**
 The report line for one interim page.

 @param runsDir - run directory whose published tree holds the page

 @param record - the interim record the entry left

 @returns The line saying what the page is made of, or that it is not the page the record was written for

 @example
 ```ts
 console.log(await interimLine({ runsDir, record, },),);
 ```
 */
export async function interimLine(
  {
    runsDir,
    record,
  }: {
    readonly runsDir: string;
    readonly record: InterimEntryRecord;
  },
): Promise<string> {
  /**
   The page as it stands now.
   */
  const pageText = await readFile(
    fixedPagePath({
      publishDir: join(
        runsDir,
        FIXED_TREE_DIR,
      ),
      entryId: record.id,
    },),
    'utf8',
  );
  if (interimPageDigest({ pageText, },) !== record.pageSha256) {
    return `  INTERIM PAGE DISAGREES WITH ITS RECORD: ${record.id}. The page on disk is not the page the record was `
      + 'written for; the next pass that reaches the entry overwrites it';
  }
  return `  INTERIM PAGE: ${record.id}. A dry stop shipped it with ${
    slicesOf({
      count: record.settledSlices,
      kind: 'settled',
    },)
  }, ${
    slicesOf({
      count: record.archiveSlices,
      kind: 'archive',
    },)
  } and ${
    slicesOf({
      count: record.sourceSlices,
      kind: 'original',
    },)
  }; it stays open until a pass settles the entry again and overwrites it`;
}

//endregion Verify published interim
