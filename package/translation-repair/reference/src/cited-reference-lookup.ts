//region Cited-reference lookup
// The pages an original links, bought once each through the Exa contents
// endpoint and kept in the reference cache, rendered as the lines the
// critic and panel sheets read (class thirty-five, the owner's decision of
// 2026-09-16: "3, and we have Exa. Such Exa fetched results need to be
// cached too to avoid hitting reference links too much").

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  allInInputOrder,
  refusalText,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { fetchCitedReference, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { citedReferenceUrlsOf, } from './cited-reference-scan.ts';
import { foldedLine, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  REFERENCE_UNFETCHED,
  REFERENCE_UNREADABLE,
  referenceLineHead,
} from './reference-line-head.ts';
import {
  type CachedReference,
  readCachedReference,
  type ReferenceRecord,
  writeCachedReference,
} from './reference-cache.ts';
import { EXA_API_KEY_VAR, } from './work-title-search.ts';

/**
 Logger root for the lookup.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Record for one url, from the cache or bought and cached.

 @param url - page as the original links it

 @param apiKey - key for the endpoint

 @param dir - cache directory

 @param signal - call's abort

 @param fetchFn - transport

 @param now - clock, for the record's time

 @returns Record, cached from now on

 @throws CacheFileUnreadableError when the page's cache file is there and
 cannot be read, before anything is bought (`lookup-cache.ts`)

 @example
 ```ts
 const record = await lookupCitedReference({ url, apiKey, dir, signal, fetchFn: fetch, now: () => new Date(), },);
 ```
 */
export async function lookupCitedReference(
  {
    url,
    apiKey,
    dir,
    signal,
    fetchFn,
    now,
  }: {
    readonly url: string;
    readonly apiKey: string;
    readonly dir: string;
    readonly signal: AbortSignal;
    readonly fetchFn: typeof fetch;
    readonly now: () => Date;
  },
): Promise<ReferenceRecord> {
  /**
   What the cache holds for it.
   */
  const cached = await readCachedReference({
    dir,
    url,
  },);
  if (cached.kind === 'hit')
    return cached.record;
  /**
   When this record was bought.
   */
  const bought = now();
  /**
   What the endpoint read.
   */
  const fetched = await fetchCitedReference({
    apiKey,
    url,
    signal,
    fetchFn,
  },);
  /**
   Fresh record.
   */
  const record: ReferenceRecord = {
    url,
    fetchedAt: bought.toISOString(),
    ...fetched,
  };
  await writeCachedReference({
    dir,
    record,
  },);
  return record;
}

/**
 Line a record contributes to the sheets: the page's text on one line, or
 why there is none.

 @param index - position among the entry's references, from one

 @param record - record for the page

 @returns One line

 @example
 ```ts
 referenceLineOf({ index: 1, record, },);
 // => '- reference 1 https://a.example/post ("Post"): the page text...'
 ```
 */
export function referenceLineOf(
  {
    index,
    record,
  }: {
    readonly index: number;
    readonly record: ReferenceRecord;
  },
): string {
  /**
   Line head naming the page.
   */
  const head = referenceLineHead({
    index,
    url: record.url,
  },);
  // The failure tag is the endpoint's text, folded like the rest so the
  // block stays one line per reference (ledger B25).
  if (record.status === 'error')
    return `${head}: ${REFERENCE_UNFETCHED} (${foldedLine({ text: record.failure, },)})`;
  /**
   Text on one line.
   */
  const text = foldedLine({ text: record.text, },);
  if (text === '')
    return `${head}: ${REFERENCE_UNREADABLE}`;
  /**
   Title part, absent when the page has none.
   */
  const title = (record.title === '') ? '' : ` ("${foldedLine({ text: record.title, },)}")`;
  return `${head}${title}: ${text}`;
}

/**
 What one cited page's lookup came to: its line in the block beside the log
 line saying how it was come by, or the caller's stop it ended in.

 WRITTEN BY THE BLOCK, NOT THE LOOKUP (ledger B414). The lookups run at once,
 and a log line written inside one landed in the order the lookups ended.

 @example
 ```ts
 const page: PageOutcome = { kind: 'page', line, level: 'info', note: 'REFERENCE 1 https://cats.example/: success, 9 chars, bought', };
 ```
 */
type PageOutcome = {
  readonly kind: 'page';

  /**
   The page's line in the block.
   */
  readonly line: string;

  /**
   Level of the log line: a page read is information, a page that could not
   be fetched a warning.
   */
  readonly level: 'info' | 'warn';

  /**
   The log line.
   */
  readonly note: string;
} | {
  readonly kind: 'stopped';

  /**
   The caller's reason the lookup ended in.
   */
  readonly reason: unknown;
};

/**
 What a page's failed lookup comes to: the caller's stop, unwritten, or a
 page that could not be fetched, with the failure as `refusalText` renders it.

 THE CALLER'S OWN STOP IS NO PAGE THAT COULD NOT BE FETCHED (ledger B406):
 written as one, it read as a fetch failure and the page was answered as
 unfetched in a block the sheets read. A fetch's own failure, such as a
 transport dropping as the stop landed, keeps its line.

 @param error - what the lookup threw

 @param signal - entry's abort

 @param index - page's position among the references, from one

 @param url - page as the original links it

 @returns Stop, or the unfetched page and its warning

 @example
 ```ts
 const outcome = failedPageOutcome({ error, signal, index: 1, url, },);
 ```
 */
function failedPageOutcome(
  {
    error,
    signal,
    index,
    url,
  }: {
    readonly error: unknown;
    readonly signal: AbortSignal;
    readonly index: number;
    readonly url: string;
  },
): PageOutcome {
  if (signal.aborted && (error === signal.reason)) {
    return {
      kind: 'stopped',
      reason: error,
    };
  }
  return {
    kind: 'page',
    line: `${referenceLineHead({
      index,
      url,
    },)}: ${REFERENCE_UNFETCHED}`,
    level: 'warn',
    // `refusalText`, never the caught value's own words: a contents body
    // that is not JSON fails as V8's `SyntaxError`, whose message quotes it.
    note: `reference ${String(index,)} ${url} could not be fetched: ${refusalText({ error, },)}`,
  };
}

/**
 Block of reference lines for one original: what every page it links says,
 bought once and cached, one line per page in order of citation. Empty when
 the original links nowhere or no key is set. A page that could not be
 fetched keeps a line saying so, and its failure is logged as `refusalText`
 renders it, its message where its class declares the message free of quoted
 text and its class name otherwise. Every page's log line is written once
 every lookup has ended, in order of citation, whichever ended first.

 @param sourceText - original document

 @param apiKey - key for the endpoint; empty means no references at all

 @param dir - cache directory

 @param signal - entry's abort

 @param fetchFn - transport

 @param now - clock

 @param logger - entry logger

 @returns Lines joined by newlines, or an empty string

 @throws CacheFileUnreadableError when a cited page's cache file is there and
 cannot be read: every page's file is read before anything is bought, outside
 the catch that turns one page's failed fetch into a line, so a cache
 directory that cannot be read stops the entry and buys nothing

 @throws The caller's reason, unchanged and unwritten, when `signal` aborts and
 a page's fetch ends in it, once every lookup has ended and every other page's
 line is written, a fetch that failed of its own as the stop landed among them

 @example
 ```ts
 const referenceContext = await citedReferenceBlock({ sourceText, apiKey, dir, signal, fetchFn: fetch, now: () => new Date(), logger: l, },);
 ```
 */
export async function citedReferenceBlock(
  {
    sourceText,
    apiKey,
    dir,
    signal,
    fetchFn,
    now,
    logger,
  }: {
    readonly sourceText: string;
    readonly apiKey: string;
    readonly dir: string;
    readonly signal: AbortSignal;
    readonly fetchFn: typeof fetch;
    readonly now: () => Date;
    readonly logger: Logger;
  },
): Promise<string> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: citedReferenceBlock.name,
    l: logger,
  },);
  /**
   Pages the original links.
   */
  const urls = citedReferenceUrlsOf({ text: sourceText, },);
  if (urls.length === 0)
    return '';
  if (apiKey === '') {
    rl.warn(`${EXA_API_KEY_VAR} is not set; ${String(urls.length,)} cited ${
      wordForCount({
        count: urls.length,
        one: 'reference',
        many: 'references',
      },)
    } ${
      wordForCount({
        count: urls.length,
        one: 'goes',
        many: 'go',
      },)
    } unread`,);
    return '';
  }
  /**
   What the cache held for each page before the ask, so the log can say
   what was bought and what was read back.
   */
  const held = await allInInputOrder({
    members: urls.map(function heldFor(url,): Promise<CachedReference> {
      return readCachedReference({
        dir,
        url,
      },);
    },),
  },);
  /**
   Answers that were hits.
   */
  const hits = held.filter(function isHit(answer,): boolean {
    return answer.kind === 'hit';
  },);
  /**
   How many that is.
   */
  const cachedCount = hits.length;
  /**
   What each page's lookup came to, bought together; a failure yields a line
   saying so, so the sheet still names the page, and no lookup rejects.
   */
  const outcomes = await Promise.all(urls.map(async function outcomeFor(
    url,
    at,
  ): Promise<PageOutcome> {
    /**
     Position among the references, from one.
     */
    const index = at + 1;
    /**
     What the cache answered for this page before the ask.
     */
    const answer = held[at];
    /**
     Whether that answer was a hit.
     */
    const wasHeld = answer?.kind === 'hit';
    try {
      /**
       Record for this page, cached or fresh.
       */
      const record = await lookupCitedReference({
        url,
        apiKey,
        dir,
        signal,
        fetchFn,
        now,
      },);
      /**
       Page text, for its length.
       */
      const { text, } = record;
      return {
        kind: 'page',
        line: referenceLineOf({
          index,
          record,
        },),
        level: 'info',
        note: `REFERENCE ${String(index,)} ${url}: ${record.status}, ${String(text.length,)} ${
          wordForCount({
            count: text.length,
            one: 'char',
            many: 'chars',
          },)
        }${
          wasHeld ? ', cached' : ', bought'
        }`,
      };
    } catch (error) {
      return failedPageOutcome({
        error,
        signal,
        index,
        url,
      },);
    }
  },),);
  // IN ORDER OF CITATION, from what each lookup returned, whichever ended
  // first (`mistake-prevention.md`, "Output decided by arrival order"); a page
  // read or refused before the stop keeps its line.
  for (const outcome of outcomes) {
    if (outcome.kind === 'page') {
      if (outcome.level === 'warn')
        rl.warn(outcome.note,);
      else
        rl.info(outcome.note,);
    }
  }
  // The caller's reason goes on unchanged and unwritten: every lookup the stop
  // ended returned the same reason, so the block refuses with it whichever
  // ended first.
  for (const outcome of outcomes) {
    if (outcome.kind === 'stopped')
      throw outcome.reason;
  }
  /**
   Line per url, in order of citation.
   */
  const lines = outcomes.map(function lineOf(outcome,): string {
    if (outcome.kind === 'stopped')
      throw new Error('unreachable: a stopped lookup was left after the block refused with the caller\'s reason',);
    return outcome.line;
  },);
  rl.info(
    `REFERENCES cited=${String(urls.length,)} cached=${String(cachedCount,)} bought=${
      String(urls.length - cachedCount,)
    }`,
  );
  return lines.join('\n',);
}

//endregion Cited-reference lookup
