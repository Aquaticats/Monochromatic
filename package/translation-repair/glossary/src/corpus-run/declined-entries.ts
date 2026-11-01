import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import { writeFileAtomic, } from './atomic-write.ts';
import {
  filesystemReason,
  presentNamesOfKind,
} from './directory-listing.ts';

//region Declined entries
// WHAT A PASS LEAVES BEHIND FOR AN ENTRY IT DECLINED, and how the next pass
// reads it. Decided by the owner on 2026-09-08: an entry whose archive note
// says the whole page is the author's own English (`cheonwoomaeng`, the one in
// the pinned corpus) is not repaired. The pipeline's standing rule is that it
// always yields output (`doc/decision/translation-repair-always-yields-output.md`),
// and a decline keeps to its letter: the archive page stands as the output,
// untouched, and the decline is RECORDED rather than silent, which is the
// failure that rule exists to stop.
//
// BESIDE THE ARTIFACTS, NOT AMONG THEM. Every artifact reader treats a file in
// `artifacts/` as a settled two-lane artifact and refuses anything else, and
// `verify-published` expects a page for each; a decline record is neither, so
// it lives in its own directory and the scheduler's skip set is the union of
// the two.

/**
 Directory under a runs dir holding one record per declined entry.
 */
export const DECLINED_DIR = 'declined';

/**
 File suffix of a decline record.
 */
const RECORD_SUFFIX = '.json';

/**
 Why the pipeline declined an entry; one reason exists today.
 */
export type DeclineReason = 'archive-original';

/**
 What a decline record carries.

 @example
 ```ts
 const record: DeclinedEntryRecord = {
   id: 'cheonwoomaeng',
   tip: 'abc123',
   pipelineDigest: 'sha256:...',
   corpusSha: 'a41fc60',
   timestamp: '2026-09-08T21:00:00.000Z',
   reason: 'archive-original',
   note: '这篇文章的原文即英文，作者的第一语言为英语，请翻译时不要动本篇。',
 };
 ```
 */
export type DeclinedEntryRecord = {
  /**
   Entry declined.
   */
  readonly id: string;

  /**
   Repository head the pass ran at.
   */
  readonly tip: string;

  /**
   Built pipeline that declined it.
   */
  readonly pipelineDigest: string;

  /**
   Corpus commit the archive page was read at.
   */
  readonly corpusSha: string;

  /**
   When the decline was recorded.
   */
  readonly timestamp: string;

  /**
   Why.
   */
  readonly reason: DeclineReason;

  /**
   The archive's note that decided it, folded onto one line.
   */
  readonly note: string;
};

/**
 Writes one decline record where the next pass will find it.

 @param declinedDir - directory of decline records, created if absent by the
 atomic writer

 @param record - what to record

 @example
 ```ts
 await writeDeclinedEntry({ declinedDir, record, },);
 ```
 */
export async function writeDeclinedEntry(
  {
    declinedDir,
    record,
  }: {
    readonly declinedDir: string;
    readonly record: DeclinedEntryRecord;
  },
): Promise<void> {
  // CREATED ON THE FIRST DECLINE rather than by the pass at startup: a runs
  // dir whose pass declined nothing carries no such directory, and the reader
  // treats its absence as no declines.
  await mkdir(
    declinedDir,
    { recursive: true, },
  );
  await writeFileAtomic({
    path: join(
      declinedDir,
      `${record.id}${RECORD_SUFFIX}`,
    ),
    text: `${JSON.stringify(
      record,
      undefined,
      2,
    )}\n`,
  },);
}

/**
 The decline directory is there and could not be listed.

 Read as no declines, it would send every declined entry back through the
 pipeline and drop it from `verify-published`'s count. The message names the
 filesystem code alone, never the path, which can name a person.

 @example
 ```ts
 throw new DeclinedEntriesUnreadableError({ filesystemCode: 'EACCES', cause: error, },);
 ```
 */
export class DeclinedEntriesUnreadableError extends Error {
  /**
   Declares this message safe to forward: it names a filesystem code alone.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the unlistable-directory failure.

   @param filesystemCode - code the listing failed with, or the class name
   where there is none

   @param cause - failure the listing raised
   */
  constructor(
    {
      filesystemCode,
      cause,
    }: {
      readonly filesystemCode: string;
      readonly cause: unknown;
    },
  ) {
    super(
      `the decline directory could not be listed (${filesystemCode}); a pass reading it as no declines would re-run every declined entry`,
      { cause, },
    );
    this.name = 'DeclinedEntriesUnreadableError';
  }
}

/**
 The files the decline directory holds, so a non-file is never a record.

 @param declinedDir - directory of decline records

 @returns Their names, none for an absent directory

 @throws {@link DeclinedEntriesUnreadableError} for any other listing failure

 @example
 ```ts
 const names = await recordFiles({ declinedDir, },);
 ```
 */
async function recordFiles(
  { declinedDir, }: { readonly declinedDir: string; },
): Promise<readonly string[]> {
  try {
    return await presentNamesOfKind({
      dir: declinedDir,
      kind: 'file',
    },);
  } catch (error) {
    throw new DeclinedEntriesUnreadableError({
      filesystemCode: filesystemReason({ error, },),
      cause: error,
    },);
  }
}

/**
 Entry ids a runs dir carries a decline record for.

 An absent directory is no declines, not an error: a runs dir written before
 declines existed, or one whose pass declined nothing, has none. Any other
 listing failure throws, and a name that is not a regular file is no record,
 as the artifact listing reads its own directory (ledger A9b).

 @param declinedDir - directory of decline records

 @returns Ids, sorted

 @example
 ```ts
 const declined = await declinedEntryIds({ declinedDir, },);
 ```
 */
export async function declinedEntryIds(
  { declinedDir, }: { readonly declinedDir: string; },
): Promise<ReadonlySet<string>> {
  return new Set(
    textsInCodePointOrder({ texts: (await recordFiles({ declinedDir, },))
      .filter(function isRecord(name,): boolean {
        return name.endsWith(RECORD_SUFFIX,);
      },)
      .map(function toId(name,): string {
        return name.slice(
          0,
          -RECORD_SUFFIX.length,
        );
      },), },),
  );
}

//endregion Declined entries
