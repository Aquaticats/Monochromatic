import { createHash, } from 'node:crypto';

import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  BenchSeating,
  SliceCache,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  hashContent,
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  PAGE_TITLE_CACHE_VERSION,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  pageTitleLines,
  type SettledPageTitle,
  settlePageTitles,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  type RepeatedTitleSpan,
  repeatedTitleSpans,
} from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  openNamespacedCache,
  PAGE_TITLES_NAMESPACE,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Pass page titles
// THE PREPARATION'S TITLE LEXICON (ledger H16, 2026-09-28): the titles a page
// repeats and the archive leaves unpaired, settled once and carried as
// evidence lines after the page-name block. CACHED because the answer is
// wording: rebought on a resume, it would word the titles afresh, the identity
// context every slice key folds would move, and every settled slice of the
// page would be bought again.

/**
 What one settled lexicon round stores.

 @example
 ```ts
 const record: PageTitleLexiconRecord = { titles: [], findings: [], };
 ```
 */
export type PageTitleLexiconRecord = {
  /**
   Titles settled.
   */
  readonly titles: readonly SettledPageTitle[];
  /**
   What the round reported.
   */
  readonly findings: readonly string[];
};

/**
 Lines and findings the lexicon adds to a preparation.

 @example
 ```ts
 const added: PassPageTitles = { lines: [], findings: [], };
 ```
 */
export type PassPageTitles = {
  /**
   Identity-context lines, empty when nothing settled.
   */
  readonly lines: readonly string[];
  /**
   What the round reported, or what a resumed round had reported.
   */
  readonly findings: readonly string[];
};

/**
 Whether a stored value is one settled title.

 @param value - parsed stored item

 @returns Whether it reads as a settled title

 @example
 ```ts
 isSettledPageTitle({ source: '猫之歌', occurrences: 2, rendering: 'Song of the Cat', voices: 2, heard: 3, },);
 ```
 */
function isSettledPageTitle(value: unknown,): value is SettledPageTitle {
  return isJsonRecord(value,)
    && ((typeof value.source) === 'string')
    && ((typeof value.rendering) === 'string')
    && Number.isInteger(value.occurrences,)
    && Number.isInteger(value.voices,)
    && Number.isInteger(value.heard,);
}

/**
 Whether a stored value is a settled lexicon round.

 @param value - parsed stored record

 @returns Whether it can be republished as one

 @example
 ```ts
 isPageTitleLexiconRecord({ titles: [], findings: [], },);
 ```
 */
function isPageTitleLexiconRecord(value: unknown,): value is PageTitleLexiconRecord {
  return isJsonRecord(value,)
    && isJsonArray(value.titles,)
    && value.titles
      .every(isSettledPageTitle,)
    && isJsonArray(value.findings,)
    && value.findings
      .every(function isFinding(finding,): boolean {
        return (typeof finding) === 'string';
      },);
}

/**
 Opens an entry's page title lexicon cache.

 @param dir - per-entry slice-cache directory

 @param generation - digest of the built pipeline this pass runs

 @returns Cache resuming a settled lexicon and persisting a new one

 @example
 ```ts
 const pageTitleCache = await openPageTitleCache({ dir: entryCacheDir, generation, },);
 ```
 */
export async function openPageTitleCache(
  {
    dir,
    generation,
  }: {
    readonly dir: string;
    readonly generation: string;
  },
): Promise<SliceCache<PageTitleLexiconRecord>> {
  return await openNamespacedCache({
    dir,
    generation,
    namespace: PAGE_TITLES_NAMESPACE,
    isValue: isPageTitleLexiconRecord,
  },);
}

/**
 Names one lexicon question: the original it was asked over, the titles, the
 declared identity the sheet showed and the roster that answers, as one JSON
 value so no two questions share a key (the X15 lesson).

 @param sourceText - original document

 @param spans - titles asked about

 @param identityContext - declared identity the sheet showed, empty for none,
 so a page whose lookups or notes changed is asked again (ledger B28)

 @param modelIds - roster that answers

 @returns Cache key

 @example
 ```ts
 const key = pageTitleKey({ sourceText, spans, identityContext: '', modelIds, },);
 ```
 */
export function pageTitleKey(
  {
    sourceText,
    spans,
    identityContext,
    modelIds,
  }: {
    readonly sourceText: string;
    readonly spans: readonly RepeatedTitleSpan[];
    readonly identityContext: string;
    readonly modelIds: readonly RosterModelId[];
  },
): string {
  return createHash('sha256',)
    .update(
    JSON.stringify({
      version: PAGE_TITLE_CACHE_VERSION,
      source: hashContent({ content: sourceText, },),
      titles: spans,
      identity: hashContent({ content: identityContext, },),
      roster: modelIds,
    },),
    'utf8',
    )
    .digest('hex',);
}

/**
 Settles, or resumes, the lexicon of one entry's repeated unpaired titles.

 THE HOOK IS READ ONLY WHEN THE PAGE REPEATS SUCH A TITLE, since a page that
 repeats none asks nobody and so has no bench to re-seat.

 A ROUND IS STORED ONLY WHEN EVERY TITLE IT ASKED IS SETTLED (the lead's ruling
 of 2026-10-25 on re-check 3, Q1). A round that leaves any title unsettled is
 not stored, so the next run asks again on whatever bench it has; the lines it
 returns for this run still carry the titles it did settle.

 @param client - provider client

 @param modelIds - roster the preparation started on

 @param beforeItem - hook handing the round its roster (ledger X12), `keepBench`
 for a caller with none; required so no caller drops it by omission

 @param sourceText - original document

 @param targetText - archive as the preparation reads it

 @param identityContext - page's declared identity without the lexicon's
 own lines, empty for none: the sheet asks for a work's official English
 title, which a web lookup or a note establishes (ledger B28)

 @param cache - store a settled round is republished from

 @param signal - entry deadline and caller abort

 @param exchangeTimeoutMs - per-call bound

 @param l - entry logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Lines for the identity context and the round's findings

 @example
 ```ts
 const { lines, findings, } = await passPageTitles({ client, modelIds, beforeItem, sourceText, targetText, identityContext: '', cache, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function passPageTitles(
  {
    client,
    modelIds,
    beforeItem,
    sourceText,
    targetText,
    identityContext,
    cache,
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly beforeItem: () => Promise<BenchSeating>;
    readonly sourceText: string;
    readonly targetText: string;
    readonly identityContext: string;
    readonly cache: SliceCache<PageTitleLexiconRecord>;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<PassPageTitles> {
  /**
   Titles the page repeats and the archive leaves unpaired.
   */
  const spans = repeatedTitleSpans({
    sourceText,
    targetText,
  },);
  if (spans.length === 0) {
    l.debug(`${passPageTitles.name}: the page repeats no unpaired title`,);
    return {
      lines: [],
      findings: [],
    };
  }
  /**
   Seating the hook hands the round: a roster read under a hold, or none,
   which keeps the one the preparation started on.
   */
  const seating = await beforeItem();
  /**
   Roster the round is asked of.
   */
  const roster = seating.modelIds ?? modelIds;
  /**
   This question's identity.
   */
  const key = pageTitleKey({
    sourceText,
    spans,
    identityContext,
    modelIds: roster,
  },);
  /**
   A round an earlier run settled for this question.
   */
  const cached = cache.resumed
    .get(key,);
  if (cached !== undefined) {
    l.info(
      `PAGE TITLES resumed: ${String(cached.titles
        .length,)} of ${String(spans.length,)} repeated ${
        wordForCount({
          count: spans.length,
          one: 'title',
          many: 'titles',
        },)
      }`,
    );
    return {
      lines: pageTitleLines({ titles: cached.titles, },),
      findings: cached.findings,
    };
  }
  /**
   What the bench settled.
   */
  const lexicon = await settlePageTitles({
    client,
    modelIds: roster,
    sourceText,
    spans,
    ...((identityContext === '') ? {} : { identityContext, }),
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  },);
  // A ROUND IS STORED ONLY WHEN EVERY TITLE IT ASKED IS SETTLED (the lead's
  // ruling of 2026-10-25 on re-check 3, Q1, on the owner's rule that one judge
  // alone never decides). A round that heard fewer than two voices is the bench
  // unreachable, not undecided, and settles nothing; a round that heard two or
  // more and left any title unsettled (a split of one voice apiece, a title no
  // voice rendered) would freeze that gap under the key of this bench, and a
  // later run on a fuller bench would resume the gap instead of asking again.
  // A round that settled every title heard at least two voices, since a title
  // settles on a rendering two voices gave, so this one test covers both.
  /**
   Titles the round settled, one for each title it asked when none was left out.
   */
  const { titles, } = lexicon;
  if (titles.length === spans.length) {
    await cache.persist({
      key,
      serialized: JSON.stringify({
        titles: lexicon.titles,
        findings: lexicon.findings,
      } satisfies PageTitleLexiconRecord,),
    },);
  }
  return {
    lines: pageTitleLines({ titles: lexicon.titles, },),
    findings: lexicon.findings,
  };
}

//endregion Pass page titles
