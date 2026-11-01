import { readFile, } from 'node:fs/promises';

import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  interimPageDigest,
  readInterimEntry,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import { errorClassOf, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  fixedPagePath,
  type PageStanding,
} from '@monochromatic-dev/translation-repair-page-check/ts';
import { rethrowUnlessMissingPath, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Pass entry page standing
// WHAT PAGE ALREADY STANDS WHERE A STOPPED ENTRY IS ABOUT TO SHIP ITS INTERIM PAGE (B1242, B1243). An interim page never
// replaces a better page, and the only evidence the pass has across attempts, and across passes, is on disk:
//
// - no page: nothing stands;
// - a page whose bytes the entry's interim record names (its digest): an interim page of an earlier attempt, as good as
//   its settled slices;
// - any other page: a page no interim record describes, which is the settled page of an attempt whose artifact write
//   failed (the settled page is written before its artifact, and the artifact is what ends an entry). A page whose
//   record never landed is read the same way, a stated limit: it is kept until a settled page or a fuller one replaces it.
//
// An entry that has its artifact is finished and never reaches a stop, so no artifact is read here.

/**
 What reading an entry's page came to.
 */
type PageRead =
  | {
    /**
     A page stands there.
     */
    readonly kind: 'page';

    /**
     Its text.
     */
    readonly pageText: string;
  }
    | {
    /**
     No page stands there.
     */
      readonly kind: 'absent';
    };

/**
 Reads what page stands at an entry's path.

 NEVER THROWS, because it runs in the shipper's path: a page or a record that cannot be read leaves the page to be shipped
 as it would have been, and the log says the read failed and with what class.

 @param publishDir - root of the mirrored tree

 @param interimDir - directory of interim records

 @param entryId - entry about to ship

 @param l - entry logger

 @returns Standing page as far as the disk shows it

 @example
 ```ts
 const standing = await pageStandingOf({ publishDir, interimDir, entryId, l, },);
 ```
 */
export async function pageStandingOf(
  {
    publishDir,
    interimDir,
    entryId,
    l,
  }: {
    readonly publishDir: string;
    readonly interimDir: string;
    readonly entryId: string;
    readonly l: Logger;
  },
): Promise<PageStanding> {
  try {
    /**
     The page on disk, absent where none stands.
     */
    const page = await readPageIfThere({
      publishDir,
      entryId,
    },);
    if (page.kind === 'absent')
      return { kind: 'none', };
    /**
     The record the entry's interim ship left, if any.
     */
    const read = await readInterimEntry({
      interimDir,
      entryId,
    },);
    if (read.kind === 'record') {
      /**
       The record the page may be the page of.
       */
      const { record, } = read;
      /**
       The page that is on disk.
       */
      const { pageText, } = page;
      if (interimPageDigest({ pageText, },) === record.pageSha256) {
        return {
          kind: 'interim',
          settledSlices: record.settledSlices,
        };
      }
    }
    return { kind: 'settled', };
  }
  catch (error) {
    l.warn(
      `entry ${entryId}: the page that stands could not be read (class=${errorClassOf({ error, },)}; ${
        refusalText({ error, },)
      }), so the interim page is shipped as if none stood`,
    );
    return { kind: 'none', };
  }
}

/**
 Reads an entry's page.

 @param publishDir - root of the mirrored tree

 @param entryId - entry whose page to read

 @returns Page, or that none stands

 @throws What the read raises for any failure but an absent file

 @example
 ```ts
 const page = await readPageIfThere({ publishDir, entryId, },);
 ```
 */
async function readPageIfThere(
  {
    publishDir,
    entryId,
  }: {
    readonly publishDir: string;
    readonly entryId: string;
  },
): Promise<PageRead> {
  try {
    return {
      kind: 'page',
      pageText: await readFile(
        fixedPagePath({
          publishDir,
          entryId,
        },),
        'utf8',
      ),
    };
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    return { kind: 'absent', };
  }
}

//endregion Pass entry page standing
