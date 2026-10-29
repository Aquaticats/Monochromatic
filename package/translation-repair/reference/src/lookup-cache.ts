import { createHash, } from 'node:crypto';
import {
  mkdir,
  readFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { writeFileAtomic, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { failureName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  contextRoot,
  isMissingPathError,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { parseModelJson, } from '@monochromatic-dev/translation-repair-provider-stream/ts';

//region Lookup cache
// Durable cache of web lookups, across runs and entries.
//
// WHY DURABLE. The owner, 2026-09-02: "Looking up official translations and
// the like can and should be cached." A title is bought once; a resumed run
// reads the same lines and so keeps its preparation identity, which hashes the
// identity context the lines join. The per-launch runs directory is exactly
// the place that would re-buy every title per launch, so the cache lives under
// the user's cache home instead, overridable for tests.
//
// A FILE THAT HOLDS NO RECORD IS A MISS, AND SAID. A file cut short or emptied
// by a write that never finished, and one whose JSON is some other shape,
// each cost one lookup: the read warns, naming the file and none of its text,
// the caller buys the record again, and the write replaces the file. The
// parse refusal used to reach the caller instead and leave the file where it
// was, so every later run met it at the same title.
//
// A FILE THAT IS THERE AND CANNOT BE READ IS A REFUSAL
// (`CacheFileUnreadableError`). Only a path nothing stands at is absence
// (`missing-path-error.ts`); a directory in the file's place or a permission
// refused read as a miss buys the record on every run and never keeps it.
//
// THE WRITE IS ATOMIC (`corpus-run/atomic-write.ts`): a reader finds the
// earlier record, the whole new one, or no file, and a write that fails part
// way leaves the path as it was.

/**
 Logger root for the cache.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Environment variable overriding where lookups are cached.
 */
export const LOOKUP_CACHE_DIR_VAR = 'TRANSLATION_REPAIR_LOOKUP_CACHE_DIR';

/**
 One result the lookup keeps.

 @example
 ```ts
 const hit: LookupHit = { title: 'To Live (novel) - Wikipedia', url: 'https://en.wikipedia.org/wiki/To_Live_(novel)', highlight: 'To Live is a novel by Yu Hua...', };
 ```
 */
export type LookupHit = {
  readonly title: string;
  readonly url: string;
  readonly highlight: string;
};

/**
 What the cache stores for one query.

 @example
 ```ts
 const record: LookupRecord = { query: '《活着》 official English title', fetchedAt: '2026-09-02T10:00:00.000Z', hits: [], };
 ```
 */
export type LookupRecord = {
  readonly query: string;
  readonly fetchedAt: string;
  readonly hits: readonly LookupHit[];
};

/**
 What a cache read answers: the record, or that there is none.

 @example
 ```ts
 const answer: CachedLookup = { kind: 'miss', };
 ```
 */
export type CachedLookup =
  | {
    readonly kind: 'hit';
    readonly record: LookupRecord;
  }
    | { readonly kind: 'miss'; };

/**
 The package's directory under the user's cache home, which the lookup cache
 and the coverage census each keep a directory in.

 @param env - environment to read

 @returns `translation-repair` under the XDG cache home when set, under `~/.cache` otherwise

 @example
 ```ts
 packageCacheDir({ env: process.env, },);
 ```
 */
export function packageCacheDir(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): string {
  /**
   XDG cache home when set, the conventional default otherwise.
   */
  const cacheHome = env.XDG_CACHE_HOME ?? '';
  /**
   Base the cache sits under.
   */
  const base = (cacheHome === '')
    ? join(
      homedir(),
      '.cache',
    )
    : cacheHome;
  return join(
    base,
    'translation-repair',
  );
}

/**
 Cache directory, from the override or the user's cache home.

 @param env - environment to read

 @returns Directory lookups are cached in

 @example
 ```ts
 lookupCacheDir({ env: process.env, },);
 ```
 */
export function lookupCacheDir(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): string {
  /**
   Explicit override when set and non-empty.
   */
  const override = env[LOOKUP_CACHE_DIR_VAR] ?? '';
  if (override !== '')
    return override;
  return join(
    packageCacheDir({ env, },),
    'lookup',
  );
}

/**
 Cache file for one query.

 @param dir - cache directory

 @param query - query the record answers

 @returns Path named by the query's digest

 @example
 ```ts
 lookupCachePath({ dir, query: '《活着》 official English title', },);
 ```
 */
export function lookupCachePath(
  {
    dir,
    query,
  }: {
    readonly dir: string;
    readonly query: string;
  },
): string {
  /**
   Digest naming the file, so any query is a safe file name.
   */
  const digest = createHash('sha256',)
    .update(
    query,
    'utf8',
    )
    .digest('hex',);
  return join(
    dir,
    `${digest}.json`,
  );
}

/**
 Whether a parsed value is one hit.

 @param value - element of a record's hits

 @returns Whether it carries the three strings

 @example
 ```ts
 isLookupHit({ title: 'a', url: 'https://x', highlight: '', },);
 // => true
 ```
 */
export function isLookupHit(value: unknown,): value is LookupHit {
  if (!isJsonRecord(value,))
    return false;
  /**
   Candidate fields.
   */
  const hit = value as {
    readonly title?: unknown;
    readonly url?: unknown;
    readonly highlight?: unknown;
  };
  return ((typeof hit.title) === 'string')
    && ((typeof hit.url) === 'string')
    && ((typeof hit.highlight) === 'string');
}

/**
 Whether a parsed value is a cache record.

 @param value - parsed JSON

 @returns Whether it carries a query, a time and hits of the right shape

 @example
 ```ts
 if (isLookupRecord(JSON.parse(text,),)) { }
 ```
 */
export function isLookupRecord(value: unknown,): value is LookupRecord {
  if (!isJsonRecord(value,))
    return false;
  /**
   Candidate fields.
   */
  const record = value as {
    readonly query?: unknown;
    readonly fetchedAt?: unknown;
    readonly hits?: unknown;
  };
  if (!Array.isArray(record.hits,))
    return false;
  /**
   Hits as unknowns, each checked.
   */
  const hits: readonly unknown[] = record.hits;
  return ((typeof record.query) === 'string')
    && ((typeof record.fetchedAt) === 'string')
    && hits.every(isLookupHit,);
}

/**
 A cache file that is there and could not be read: a directory standing in
 its place, a permission refused, a failing disk.

 A REFUSAL, NOT A MISS. A miss buys the record again and then meets the same
 path at the write, so every run would pay for a record it can neither read
 nor keep, with only a debug line to say why.

 WHAT TO DO ABOUT ONE. The message names the filesystem code and the file.
 Make the file readable, or remove whatever stands at its path, and the next
 run reads the record or buys it once.

 @example
 ```ts
 throw new CacheFileUnreadableError({ path, failure: 'EACCES', cause: error, },);
 ```
 */
export class CacheFileUnreadableError extends Error {
  /**
   Declares this message safe to forward: it names a filesystem code or a
   class name and the cache file's path, and repeats nothing the file holds.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param path - cache file that could not be read

   @param failure - filesystem code where there was one, class name otherwise, never a message

   @param cause - what the read raised
   */
  constructor(
    {
      path,
      failure,
      cause,
    }: {
      readonly path: string;
      readonly failure: string;
      readonly cause: unknown;
    },
  ) {
    super(
      `cache file is there and could not be read (${failure}): ${path}`,
      { cause, },
    );
    this.name = 'CacheFileUnreadableError';
  }
}

/**
 Answer for a cache path nothing stands at.
 */
export const CACHE_FILE_ABSENT: unique symbol = Symbol('the cache path holds no file (ENOENT)',);

/**
 Reads a cache file's text, telling a file that is not there from one that
 is there and empty: an empty file is what a write cut short before its
 first byte leaves, and its reader says so instead of reading it as absent.

 @param path - cache file to read

 @returns Its text, or `CACHE_FILE_ABSENT` where nothing stands at the path

 @throws {@link CacheFileUnreadableError} when something stands at the path
 and could not be read

 @example
 ```ts
 const text = await cacheFileText({ path, },);
 ```
 */
export async function cacheFileText(
  { path, }: { readonly path: string; },
): Promise<string | typeof CACHE_FILE_ABSENT> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: cacheFileText.name,
    l,
  },);
  try {
    return await readFile(
      path,
      'utf8',
    );
  } catch (error) {
    if (!isMissingPathError({ error, },)) {
      throw new CacheFileUnreadableError({
        path,
        failure: failureName({ error, },),
        cause: error,
      },);
    }
    rl.debug(`no cache file at ${path}`,);
    return CACHE_FILE_ABSENT;
  }
}

/**
 Reads the cached record for a query.

 @param dir - cache directory

 @param query - query to look for

 @returns Record, or a miss when no file is there or the file holds no
 record: text that is not JSON (a file cut short or left empty), or JSON of
 another shape, each said on a warning naming the file

 @throws {@link CacheFileUnreadableError} when the file is there and could
 not be read

 @example
 ```ts
 const cached = await readCachedLookup({ dir, query, },);
 ```
 */
export async function readCachedLookup(
  {
    dir,
    query,
  }: {
    readonly dir: string;
    readonly query: string;
  },
): Promise<CachedLookup> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: readCachedLookup.name,
    l,
  },);
  /**
   File the record would be in.
   */
  const path = lookupCachePath({
    dir,
    query,
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
    rl.warn(`cached lookup at ${path} does not parse as JSON; ignoring it`,);
    return { kind: 'miss', };
  }
  if (!isLookupRecord(attempt.value,)) {
    rl.warn(`cached lookup at ${path} is not a record; ignoring it`,);
    return { kind: 'miss', };
  }
  return {
    kind: 'hit',
    record: attempt.value,
  };
}

/**
 Writes a record for its query so no reader finds it half-written.

 @param dir - cache directory, created when missing

 @param record - record to keep

 @example
 ```ts
 await writeCachedLookup({ dir, record, },);
 ```
 */
export async function writeCachedLookup(
  {
    dir,
    record,
  }: {
    readonly dir: string;
    readonly record: LookupRecord;
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
    path: lookupCachePath({
      dir,
      query: record.query,
    },),
    text: `${text}\n`,
  },);
}

//endregion Lookup cache
