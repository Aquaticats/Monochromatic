import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  citedReferenceBlock,
  EXA_API_KEY_VAR,
  lookupCacheDir,
  referenceCacheDir,
  workTitleLookupLines,
} from '@monochromatic-dev/translation-repair-reference/ts';
import {
  type CorpusName,
  type CorpusPin,
  type listCorpusPeople,
  type readCorpusFile,
  readCorpusNames,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';

import {
  type EntryInfoRecord,
  readEntryInfoRecord,
} from '@monochromatic-dev/translation-repair-insertion/ts';

//region Pass outside reads
// WHAT A PREPARATION READS FROM OUTSIDE THE PIPELINE, named in one place and
// required of every caller (ledger X19). The run's readers take their keys
// and cache directories from the environment `mise` builds, and the unit
// suite runs under that same environment, decrypted keys included. On
// 2026-09-28 a unit test for the page title lexicon (ledger H16) prepared an
// original naming an invented title; the preparation's own lookup read the
// Exa key the suite had inherited, bought a web search for the invented
// title, and wrote the result into the real lookup cache. A seam a caller may
// leave out is a seam a test forgets; a required one is a type error until
// the caller chooses.
//
// BUILT BY THE ENTRY FILE THAT RUNS A PREPARATION, over its own environment,
// since 2026-10-10: `RUN_OUTSIDE_READS` was built here at module load over
// the live `process.env`, so a library module named the process's environment
// and every importer reached it without an entry file handing it on.
// `corpus-pass.ts` and `rendering-audit-settled.ts` now call
// `outsideReadsFrom` in their own functions.

/**
 Reads what the pages an original links say, empty when it links nowhere or
 nothing could be read.

 @example
 ```ts
 const references: PassReferenceReader = async () => '';
 ```
 */
export type PassReferenceReader = (
  input: {
    readonly sourceText: string;
    readonly signal: AbortSignal;
    readonly l: Logger;
  },
) => Promise<string>;

/**
 Reads the evidence lines for the works an original names, none when it
 names no work or nothing could be read.

 @example
 ```ts
 const workTitles: PassWorkTitleReader = async () => [];
 ```
 */
export type PassWorkTitleReader = (
  input: {
    readonly sourceText: string;
    readonly signal: AbortSignal;
    readonly l: Logger;
  },
) => Promise<readonly string[]>;

/**
 Reads the names every corpus entry declares.

 @example
 ```ts
 const corpusNames: PassCorpusNameReader = async () => [];
 ```
 */
export type PassCorpusNameReader = () => Promise<readonly CorpusName[]>;

/**
 Reads an entry's `info.yml` at the pin, which carries the date of death; a file
 that refuses to parse is answered as the unreadable record, never thrown.

 @example
 ```ts
 const entryInfo: PassEntryInfoReader = async () => ({ kind: 'absent', });
 ```
 */
export type PassEntryInfoReader = (
  input: {
    readonly entryId: string;
    readonly l: Logger;
  },
) => Promise<EntryInfoRecord>;

/**
 Everything a preparation reads from outside the pipeline.

 @example
 ```ts
 const reads: PassOutsideReads = outsideReadsFrom({ env: process.env, fetchFn: fetch, pin, listPeople: listCorpusPeople, readFile: readCorpusFile, },);
 ```
 */
export type PassOutsideReads = {
  /**
   Evidence for the works the original names (the owner's rule of
   2026-09-02).
   */
  readonly workTitles: PassWorkTitleReader;

  /**
   What the pages the original links say (class thirty-five, the owner's
   decision of 2026-09-16).
   */
  readonly references: PassReferenceReader;

  /**
   Names other entries declare (class seventy-eight).
   */
  readonly corpusNames: PassCorpusNameReader;

  /**
   The entry's `info.yml` at the pin, which records the date of death (the
   house tense rule's "life that has ended").
   */
  readonly entryInfo: PassEntryInfoReader;
};

/**
 The clock a lookup record is stamped with.

 @returns Now

 @example
 ```ts
 const stamped = wallClock().toISOString();
 ```
 */
function wallClock(): Date {
  return new Date();
}

/**
 Outside reads over one environment, transport and corpus: the web through
 the environment's Exa key and lookup cache, and the entries at a pin.

 Every input that reaches past the process is REQUIRED, as the readers
 themselves are of their callers (ledger X19, M68), so a test builds these
 readers only by naming its own transport, cache directory and corpus.

 @param env - environment the key and the cache directory are read from, at each read

 @param fetchFn - transport both web readers buy through

 @param pin - corpus checkout and commit the names are read at

 @param listPeople - entry lister at the pin

 @param readFile - document reader at the pin

 @param now - clock a new lookup record is stamped with; the wall clock unless a test fixes it

 @returns Readers for one preparation's outside reads

 @example
 ```ts
 const reads = outsideReadsFrom({ env: process.env, fetchFn: fetch, pin, listPeople: listCorpusPeople, readFile: readCorpusFile, },);
 ```
 */
export function outsideReadsFrom(
  {
    env,
    fetchFn,
    pin,
    listPeople,
    readFile,
    now = wallClock,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly fetchFn: typeof fetch;
    readonly pin: CorpusPin;
    readonly listPeople: typeof listCorpusPeople;
    readonly readFile: typeof readCorpusFile;
    readonly now?: () => Date;
  },
): PassOutsideReads {
  return {
    workTitles: function lookupWorkTitles(
      {
        sourceText,
        signal,
        l,
      },
    ): Promise<readonly string[]> {
      return workTitleLookupLines({
        sourceText,
        // No key means no lookup, which the reader logs and answers with no
        // evidence lines; a run without one is a run that buys no search.
        apiKey: env[EXA_API_KEY_VAR] ?? '',
        dir: lookupCacheDir({ env, },),
        signal,
        fetchFn,
        now,
        logger: l,
      },);
    },
    references: function readReferences(
      {
        sourceText,
        signal,
        l,
      },
    ): Promise<string> {
      return citedReferenceBlock({
        sourceText,
        // No key means no page is read, as for the work titles.
        apiKey: env[EXA_API_KEY_VAR] ?? '',
        dir: referenceCacheDir({ env, },),
        signal,
        fetchFn,
        now,
        logger: l,
      },);
    },
    corpusNames: function readNames(): Promise<readonly CorpusName[]> {
      return readCorpusNames({
        pin,
        listPeople,
        readFile,
      },);
    },
    entryInfo: function readEntryInfo(
      {
        entryId,
        l,
      },
    ): Promise<EntryInfoRecord> {
      return readEntryInfoRecord({
        pin,
        entryId,
        readFile,
        l,
      },);
    },
  };
}

//endregion Pass outside reads
