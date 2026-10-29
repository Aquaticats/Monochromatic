//region Reference cache
// Where a fetched page is kept, once per url, for every later run. The
// owner's rule of 2026-09-16 with the decision to fetch cited references:
// "Such Exa fetched results need to be cached too to avoid hitting reference
// links too much." A subdirectory of the work-title lookup cache, keyed by
// the url's digest, never expiring (the owner, same day: "Cache Exa results
// semi-permanently on disk since the reference links rarely change their
// content"); a record is refreshed only by deleting its file.
//
// DAMAGED, UNREADABLE AND HALF-WRITTEN FILES are handled as the lookup cache
// handles them, through its reader (`lookup-cache.ts`): a file holding no
// record is a miss said on a warning naming the file, so the page is bought
// again and the file replaced; a file that is there and cannot be read is
// refused (`CacheFileUnreadableError`); and the write is atomic. The parse
// refusal of a file cut short used to reject the whole reference block, and
// with it the entry citing the page, on every run until the file was deleted.

import { createHash, } from 'node:crypto';
import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { FetchedReference, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { writeFileAtomic, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  CACHE_FILE_ABSENT,
  cacheFileText,
  lookupCacheDir,
} from './lookup-cache.ts';
import { parseModelJson, } from '@monochromatic-dev/translation-repair-provider-stream/ts';

/**
 Logger root for the cache.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Subdirectory of the lookup cache the references live in.
 */
export const REFERENCE_CACHE_SUBDIR = 'reference';

/**
 What the cache stores for one url: what the fetch yielded and when.

 @example
 ```ts
 const record: ReferenceRecord = { url: 'https://a.example/post', fetchedAt: '2026-09-16T16:00:00.000Z', status: 'success', title: 'Post', text: '...', };
 ```
 */
export type ReferenceRecord = {
  /**
   Page as the original links it.
   */
  readonly url: string;
  /**
   When it was bought.
   */
  readonly fetchedAt: string;
} & FetchedReference;

/**
 What a cache read answers: the record, or that there is none.

 @example
 ```ts
 const answer: CachedReference = { kind: 'miss', };
 ```
 */
export type CachedReference =
  | {
    readonly kind: 'hit';
    readonly record: ReferenceRecord;
  }
    | { readonly kind: 'miss'; };

/**
 Cache directory: the lookup cache's reference subdirectory, so the lookup
 override and the XDG rule both carry over.

 @param env - environment to read

 @returns Directory references are cached in

 @example
 ```ts
 referenceCacheDir({ env: process.env, },);
 ```
 */
export function referenceCacheDir(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): string {
  return join(
    lookupCacheDir({ env, },),
    REFERENCE_CACHE_SUBDIR,
  );
}

/**
 Cache file for one url.

 @param dir - cache directory

 @param url - page the record answers for

 @returns Path named by the url's digest

 @example
 ```ts
 referenceCachePath({ dir, url: 'https://a.example/post', },);
 ```
 */
export function referenceCachePath(
  {
    dir,
    url,
  }: {
    readonly dir: string;
    readonly url: string;
  },
): string {
  /**
   Digest naming the file, so any url is a safe file name.
   */
  const digest = createHash('sha256',)
    .update(
    url,
    'utf8',
    )
    .digest('hex',);
  return join(
    dir,
    `${digest}.json`,
  );
}

/**
 Whether a parsed value is a cache record.

 @param value - parsed JSON

 @returns Whether it carries a url, a time, a status, a title and text of
 the right shapes, and a failure tag exactly when its status is error

 @example
 ```ts
 if (isReferenceRecord(JSON.parse(text,),)) { }
 ```
 */
export function isReferenceRecord(value: unknown,): value is ReferenceRecord {
  if (!isJsonRecord(value,))
    return false;
  /**
   Whether the failure tag is text on an error record and absent on any
   other, which every record this package writes holds.
   */
  const failureShaped = (value.status === 'error')
    ? ((typeof value.failure) === 'string')
    : (value.failure === undefined);
  return ((typeof value.url) === 'string')
    && ((typeof value.fetchedAt) === 'string')
    && ((value.status === 'success') || (value.status === 'error'))
    && ((typeof value.title) === 'string')
    && ((typeof value.text) === 'string')
    && failureShaped;
}

/**
 Reads the cached record for a url.

 @param dir - cache directory

 @param url - page to look for

 @returns Record, or a miss when no file is there or the file holds no
 record: text that is not JSON (a file cut short or left empty), or JSON of
 another shape, each said on a warning naming the file

 @throws CacheFileUnreadableError when the file is there and could not be
 read

 @example
 ```ts
 const cached = await readCachedReference({ dir, url, },);
 ```
 */
export async function readCachedReference(
  {
    dir,
    url,
  }: {
    readonly dir: string;
    readonly url: string;
  },
): Promise<CachedReference> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: readCachedReference.name,
    l,
  },);
  /**
   File the record would be in.
   */
  const path = referenceCachePath({
    dir,
    url,
  },);
  /**
   File text, or that no file is there.
   */
  const text = await cacheFileText({ path, },);
  if (text === CACHE_FILE_ABSENT)
    return { kind: 'miss', };
  /**
   Parse attempt over the file's text, its failure taken as data and its
   detail never repeated, since V8 quotes the text it refused.
   */
  const attempt = parseModelJson({ text, },);
  if (!attempt.parsed) {
    rl.warn(`cached reference at ${path} does not parse as JSON; ignoring it`,);
    return { kind: 'miss', };
  }
  if (!isReferenceRecord(attempt.value,)) {
    rl.warn(`cached reference at ${path} is not a record; ignoring it`,);
    return { kind: 'miss', };
  }
  return {
    kind: 'hit',
    record: attempt.value,
  };
}

/**
 Writes a record for its url so no reader finds it half-written.

 @param dir - cache directory, created when missing

 @param record - record to keep

 @example
 ```ts
 await writeCachedReference({ dir, record, },);
 ```
 */
export async function writeCachedReference(
  {
    dir,
    record,
  }: {
    readonly dir: string;
    readonly record: ReferenceRecord;
  },
): Promise<void> {
  await mkdir(
    dir,
    { recursive: true, },
  );
  /**
   Pretty JSON, so a reader can open the file.
   */
  const text = JSON.stringify(
    record,
    null,
    2,
  );
  await writeFileAtomic({
    path: referenceCachePath({
      dir,
      url: record.url,
    },),
    text: `${text}\n`,
  },);
}

//endregion Reference cache
