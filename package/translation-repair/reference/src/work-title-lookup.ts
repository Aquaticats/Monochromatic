import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type LookupHit,
  type LookupRecord,
  readCachedLookup,
  writeCachedLookup,
} from './lookup-cache.ts';
import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  lookupQueryFor,
  workTitlesOf,
} from './work-title-scan.ts';
import {
  EXA_API_KEY_VAR,
  searchWorkTitle,
} from './work-title-search.ts';
import { carriesWord, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Work-title lookup
// Official English titles of the works an original names, looked up on the
// web once and carried to every sheet as evidence lines.
//
// WHY. The owner's rule of 2026-09-02 (`doc/decision/translation-repair-work-titles-established-vocabulary.md`):
// "If something has an official English translation, use that." A translator
// working one slice cannot know whether 《活着》 has an English edition (it
// does: "To Live"), and the Toka_ls reading found the archive's "Life of Aiden"
// re-rendered by a judge that missed the play on "Life of Pi". The owner then
// added `TRANSLATION_REPAIR_EXA_API_KEY` and said: "Looking up official
// translations and the like can and should be cached."
//
// EVIDENCE, NOT AUTHORITY. A search result can be wrong; the lines say what
// the web returned and the house rule says what an official title is. The
// sheets read a "web lookup" line as evidence to weigh.

/**
 Logger root for the lookup.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Record for one title, from the cache or bought and cached.

 @param title - title with its marks

 @param apiKey - key for the endpoint

 @param dir - cache directory

 @param signal - call's abort

 @param fetchFn - transport

 @param now - clock, for the record's time

 @returns Record, cached from now on

 @throws CacheFileUnreadableError when the title's cache file is there and
 cannot be read, before anything is bought (`lookup-cache.ts`)

 @example
 ```ts
 const record = await lookupWorkTitle({ title: '《活着》', apiKey, dir, signal, fetchFn: fetch, now: () => new Date(), },);
 ```
 */
export async function lookupWorkTitle(
  {
    title,
    apiKey,
    dir,
    signal,
    fetchFn,
    now,
  }: {
    readonly title: string;
    readonly apiKey: string;
    readonly dir: string;
    readonly signal: AbortSignal;
    readonly fetchFn: typeof fetch;
    readonly now: () => Date;
  },
): Promise<LookupRecord> {
  /**
   Query the cache and the endpoint are keyed on.
   */
  const query = lookupQueryFor({ title, },);
  /**
   What the cache holds for it.
   */
  const cached = await readCachedLookup({
    dir,
    query,
  },);
  if (cached.kind === 'hit')
    return cached.record;
  /**
   When this record was bought.
   */
  const bought = now();
  /**
   Fresh record.
   */
  const record: LookupRecord = {
    query,
    fetchedAt: bought.toISOString(),
    hits: await searchWorkTitle({
      apiKey,
      query,
      signal,
      fetchFn,
    },),
  };
  await writeCachedLookup({
    dir,
    record,
  },);
  return record;
}

/**
 Lines a record contributes to the identity context.

 @param title - title with its marks

 @param record - record for it

 @returns One line per hit, or one line saying the web returned nothing

 @example
 ```ts
 lookupLinesOf({ title: '《活着》', record, },);
 // => ['- web lookup for 《活着》: "To Live (novel) - Wikipedia" https://en.wikipedia.org/wiki/To_Live_(novel): To Live is a novel by Yu Hua...']
 ```
 */
export function lookupLinesOf(
  {
    title,
    record,
  }: {
    readonly title: string;
    readonly record: LookupRecord;
  },
): readonly string[] {
  /**
   Hits to render.
   */
  const { hits, } = record;
  if (hits.length === 0)
    return [`- web lookup for ${title}: nothing found`,];
  /**
   The title as the text writes it, marks removed, which a result naming
   this work would carry somewhere.
   */
  const bare = bareTitleOf({ title, },);
  /**
   Hits naming this work first, so the sheet reads the relevant ones before
   the neighbours.
   */
  const ordered = [...hits,].toSorted(function namingFirst(
    left,
    right,
  ): number {
    return Number(namesWork({
      hit: right,
      bare,
    },),) - Number(namesWork({
      hit: left,
      bare,
    },),);
  },);
  return ordered.map(function toLine(hit,): string {
    /**
     Highlight on one line, or nothing.
     */
    const highlight = hit.highlight
      .split('\n',)
      .join(' ',)
      .trim();
    /**
     Highlight part of the line, absent when there is none.
     */
    const tail = (highlight === '') ? '' : `: ${highlight}`;
    /**
     Warning where the result never names the work asked about.

     THE TOKA_LS RERUN OF 2026-09-02: the five results for 《奇妙漂流》 were
     奇幻漂流 neighbours (喵的奇幻漂流, the film released in English as "Flow"),
     none naming the work, and the judges renamed the person's own work "Flow".
     */
    const warning = namesWork({
      hit,
      bare,
    },)
      ? ''
      : ' (this result does NOT name the work asked about; it is a neighbour, not its title)';
    return `- web lookup for ${title}${warning}: "${hit.title}" ${hit.url}${tail}`;
  },);
}

/**
 The title without its 《…》 marks.

 @param title - title with its marks

 @returns Title as a plain string

 @example
 ```ts
 bareTitleOf({ title: '《活着》', },);
 // => '活着'
 ```
 */
export function bareTitleOf(
  { title, }: { readonly title: string; },
): string {
  return title
    .replaceAll(
    '《',
    '',
    )
    .replaceAll(
    '》',
    '',
    );
}

/**
 Whether a result names the work asked about, in its title or its
 highlight.

 AS WORDS WHERE THE TITLE IS LATIN (ledger B23): a result titled
 "Catcraftopia" does not name 《Catcraft》. A Han edge needs no boundary, as
 Chinese writes none, so a Han title is still found inside a longer run.

 @param hit - one result

 @param bare - title without its marks

 @returns Whether the bare title occurs in the result

 @example
 ```ts
 namesWork({ hit, bare: '活着', },);
 ```
 */
export function namesWork(
  {
    hit,
    bare,
  }: {
    readonly hit: LookupHit;
    readonly bare: string;
  },
): boolean {
  /**
   Result fields the work's title could occur in.
   */
  const {
    title,
    highlight,
  } = hit;
  return [
    title,
    highlight,
  ].some(function carries(text,): boolean {
    return carriesWord({
      text,
      needle: bare,
      end: 'word',
    },);
  },);
}

/**
 What one title's lookup came to: its evidence lines, a failure to write, or
 the caller's stop it ended in.

 WRITTEN BY THE CALLER, NOT THE LOOKUP (ledger B414). The lookups run at
 once, and a failure written inside one landed in the order the lookups
 ended.

 @example
 ```ts
 const outcome: TitleOutcome = { kind: 'failed', note: 'lookup for 《猫》 failed and contributes no line: search responded 401', };
 ```
 */
type TitleOutcome = {
  readonly kind: 'lines';

  /**
   Evidence lines for the title.
   */
  readonly lines: readonly string[];
} | {
  readonly kind: 'failed';

  /**
   The warning naming the title and its failure.
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
 What a title's failed lookup comes to: the caller's stop, unwritten, or a
 lookup that contributes no line, with the failure as `refusalText` renders
 it.

 THE CALLER'S OWN STOP IS NO FAILED LOOKUP (ledger B406): written as one, it
 read as a search that failed and was answered as a title with no evidence.
 A search's own failure, such as a transport dropping as the stop landed,
 keeps its line.

 @param error - what the lookup threw

 @param signal - entry's abort

 @param title - work looked up

 @returns Stop, or the failure's warning

 @example
 ```ts
 const outcome = failedTitleOutcome({ error, signal, title: '《猫》', },);
 ```
 */
function failedTitleOutcome(
  {
    error,
    signal,
    title,
  }: {
    readonly error: unknown;
    readonly signal: AbortSignal;
    readonly title: string;
  },
): TitleOutcome {
  if (signal.aborted && (error === signal.reason)) {
    return {
      kind: 'stopped',
      reason: error,
    };
  }
  return {
    kind: 'failed',
    // `refusalText`, never the caught value's own words: a search body that
    // is not JSON fails as V8's `SyntaxError`, whose message quotes it.
    note: `lookup for ${title} failed and contributes no line: ${refusalText({ error, },)}`,
  };
}

/**
 Evidence lines for every work an original names, each title looked up once
 and cached; a failed lookup contributes no line and is logged as
 `refusalText` renders it, its message where its class declares the message
 free of quoted text and its class name otherwise. Every failure is logged
 once every lookup has ended, in title order, whichever ended first.

 @param sourceText - original document

 @param apiKey - key for the endpoint; empty means no lookups at all

 @param dir - cache directory

 @param signal - entry's abort

 @param fetchFn - transport

 @param now - clock

 @param logger - entry logger

 @returns Lines in title order, empty when the original names no work or no
 key is set

 @throws The caller's reason, unchanged and unwritten, when `signal` aborts and
 a lookup ends in it, once every lookup has ended and every other failure is
 written, a lookup that failed of its own as the stop landed among them

 @example
 ```ts
 const lines = await workTitleLookupLines({ sourceText, apiKey, dir, signal, fetchFn: fetch, now: () => new Date(), logger: l, },);
 ```
 */
export async function workTitleLookupLines(
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
): Promise<readonly string[]> {
  /**
   Logger pre-tagged with this function's name.
   */
  const wl = tagged({
    tag: workTitleLookupLines.name,
    l: logger,
  },);
  /**
   Works the original names.
   */
  const titles = workTitlesOf({ text: sourceText, },);
  if (titles.length === 0)
    return [];
  if (apiKey === '') {
    wl.warn(`${EXA_API_KEY_VAR} is not set; ${String(titles.length,)} work ${
      wordForCount({
        count: titles.length,
        one: 'title',
        many: 'titles',
      },)
    } ${
      wordForCount({
        count: titles.length,
        one: 'goes',
        many: 'go',
      },)
    } unlooked-up`,);
    return [];
  }
  /**
   What each title's lookup came to, bought together; no lookup rejects.
   */
  const outcomes = await Promise.all(titles.map(async function outcomeFor(title,): Promise<TitleOutcome> {
    try {
      /**
       Record for this title, cached or fresh.
       */
      const record = await lookupWorkTitle({
        title,
        apiKey,
        dir,
        signal,
        fetchFn,
        now,
      },);
      return {
        kind: 'lines',
        lines: lookupLinesOf({
          title,
          record,
        },),
      };
    } catch (error) {
      return failedTitleOutcome({
        error,
        signal,
        title,
      },);
    }
  },),);
  // IN TITLE ORDER, from what each lookup returned, whichever ended first
  // (`mistake-prevention.md`, "Output decided by arrival order"); a lookup
  // that failed before the stop keeps its line.
  for (const outcome of outcomes) {
    if (outcome.kind === 'failed')
      wl.warn(outcome.note,);
  }
  // The caller's reason goes on unchanged and unwritten: every lookup the stop
  // ended returned the same reason, so the lookups refuse with it whichever
  // ended first.
  for (const outcome of outcomes) {
    if (outcome.kind === 'stopped')
      throw outcome.reason;
  }
  /**
   Every line, in title order; a failed lookup contributes none.
   */
  const lines = outcomes.flatMap(function linesOf(outcome,): readonly string[] {
    if (outcome.kind === 'stopped')
      throw new Error('unreachable: a stopped lookup was left after the lookups refused with the caller\'s reason',);
    return (outcome.kind === 'lines') ? outcome.lines : [];
  },);
  wl.info(`${String(titles.length,)} work ${
    wordForCount({
      count: titles.length,
      one: 'title',
      many: 'titles',
    },)
  } looked up, ${String(lines.length,)} ${
    wordForCount({
      count: lines.length,
      one: 'line',
      many: 'lines',
    },)
  }`,);
  return lines;
}

//endregion Work-title lookup
