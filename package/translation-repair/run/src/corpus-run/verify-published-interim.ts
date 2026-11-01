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
// HOW `verify-published` READS A PAGE A STOP SHIPPED FOR AN ENTRY THAT IS STILL OPEN, the stop of every provider dry,
// of a stage that heard no voice or of any other error, each worded by the reason of the record (B1208, B1220). Such a page has no artifact
// to be weighed against, so what it can be checked against is its own record: the digest of the page as it was written.
// A page that still has that digest is the interim page, and the report says so with what it is made of; any other page
// standing at that path is named as disagreeing, since nothing the record says is then true of it.

/**
 Counts a record gives, each with its noun.

 @param count - how many slices

 @param kind - what the slices carry

 @returns Count and its noun

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
 The stop a record names, in the words of the report line.

 @param reason - why the entry stopped, as the record keeps it

 @param errorClass - class of the error that ended the entry, absent in a record written before the class was kept

 @returns Subject of the sentence that says what shipped the page

 @example
 ```ts
 const stop = stopOf({ reason: 'provider-unavailable', },); // => 'A stage that heard no voice stopped the entry and'
 ```
 */
function stopOf(
  {
    reason,
    errorClass,
  }: {
    readonly reason: InterimEntryRecord['reason'];
    readonly errorClass?: string;
  },
): string {
  if (reason === 'dry-stop')
    return 'A dry stop';
  if (reason === 'provider-unavailable')
    return 'A stage that heard no voice stopped the entry and';
  return (errorClass === undefined)
    ? 'An error stopped the entry and'
    : `An error (${errorClass}) stopped the entry and`;
}

/**
 The report line for one interim page.

 @param runsDir - run directory whose published tree holds the page

 @param record - interim record the entry left

 @returns Line saying what the page is made of, or that it is not the page the record was written for

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
  return `  INTERIM PAGE: ${record.id}. ${
    stopOf({
      reason: record.reason,
      ...((record.errorClass === undefined) ? {} : { errorClass: record.errorClass, }),
    },)
  } shipped it with ${
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
