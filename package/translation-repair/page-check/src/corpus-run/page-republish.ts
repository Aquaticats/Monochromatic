import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  rethrowUnlessMissingPath,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { readRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { artifactFileNameOf, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type ParsedTwoLaneArtifact,
  parseSettledTwoLaneArtifact,
  rebuildPreparation,
  type TwoLaneArtifactGeneration,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import { pageAgreement, } from './page-agreement.ts';
import {
  fixedPagePath,
  publishFixedPage,
} from './publish-fixed.ts';

//region Page republish
// A PASS REWRITES FROM ITS ARTIFACT EVERY PAGE THAT IS MISSING OR DISAGREES
// (ledger A16c; the owner, 2026-09-27: a run always ships, and a check repairs
// what it can). The page is written before the artifact, so a crash between the
// two leaves a page no artifact records, never the reverse; but an artifact
// whose page was lost, or a page an earlier build wrote before a later reader
// fix (the census of 2026-09-27: 77 of 214 stored pages, 69 only by the class
// 181 and 147 typography), otherwise ships as it stands.
//
// THE PAGE IS REBUILT, NOT RE-SETTLED. The artifact records every reading both
// deciders approved; what it lacks is the original, so the pair is read again
// at the artifact's own corpus commit and carved with the recipe the artifact
// records. A carve that does not reproduce the recorded rows is not the carve
// the readings were decided over, and the page is left and reported.
//
// NOTHING HERE STOPS THE PASS. Every refusal (an unreadable artifact, a moved
// carve, a publish-time page check) leaves the page as it was and is logged by
// class name only, since a refusal message quotes the corpus text it disagrees
// about.

/**
 One entry's pair as it stood at a corpus commit.

 @example
 ```ts
 const pair: CorpusPairText = { sourceText: '# 小猫', targetText: '# Kitten', };
 ```
 */
export type CorpusPairText = Readonly<{
  /**
   The original page.
   */
  sourceText: string;

  /**
   The archive's English page.
   */
  targetText: string;

  /**
   The original's entry `info.yml` as parsed at the same commit, absent where
   the entry has none or the reader reads none; its recorded death joins the
   identity block of the preparation the page is rebuilt from.
   */
  sourceEntryInfo?: unknown;
}>;

/**
 Reads one entry's pair at a corpus commit; the pass reads the pinned clone,
 a test hands fixture text.

 @example
 ```ts
 const readPair: CorpusPairReader = async function read({ entryId, corpusSha, },) { return pair; };
 ```
 */
export type CorpusPairReader = (
  request: Readonly<{
    /**
     Entry whose pair to read.
     */
    entryId: string;

    /**
     Commit the artifact's texts were read at.
     */
    corpusSha: string;

    /**
     Generation of the artifact, which decides whether the entry's `info.yml`
     is read too (`readSettledEntryInfo`): only an artifact of a generation
     whose run told its models the recorded death needs it, so a missing or
     damaged `info.yml` changes nothing about the replay of an earlier one.
     */
    generation: TwoLaneArtifactGeneration;
  }>,
) => Promise<CorpusPairText>;

/**
 What became of one settled entry's page.

 @example
 ```ts
 const outcome: RepublishOutcome = { kind: 'republished', why: 'missing', };
 ```
 */
export type RepublishOutcome =
  | Readonly<{
    /**
     The page carries what its artifact ships; nothing was written.
     */
    kind: 'agreed';
  }>
    | Readonly<{
    /**
     The page was rewritten from its artifact.
     */
      kind: 'republished';

    /**
     Why it needed writing.
     */
      why: 'disagreed' | 'missing';
    }>
    | Readonly<{
    /**
     The page needed writing and was left as it was.
     */
      kind: 'left';

    /**
     Why it needed writing, or that the artifact could not be read at all.
     */
      why: 'disagreed' | 'missing' | 'unreadable';

    /**
     What refused, through `refusalText`: a marked refusal in its own words
     (a corpus read names its kind and remedy, so a commit the clone lacks
     reads apart from a page absent at one it holds), any other class by name
     alone, or where the carve moved. Until 2026-10-06 every refusal was its
     class name, and every kind of corpus read read `CorpusReadError`.
     */
      because: string;
    }>;

/**
 One entry's outcome, named.

 @example
 ```ts
 const row: RepublishRow = { entryId: 'tabby', outcome: { kind: 'agreed', }, };
 ```
 */
export type RepublishRow = Readonly<{
  /**
   Entry the artifact settled.
   */
  entryId: string;

  /**
   What became of its page.
   */
  outcome: RepublishOutcome;
}>;

/**
 One entry's page as the disk holds it.

 @example
 ```ts
 const page: PageOnDisk = { kind: 'missing', };
 ```
 */
type PageOnDisk =
  | Readonly<{
    /**
     The page is there.
     */
    kind: 'read';

    /**
     Its text.
     */
    text: string;
  }>
    | Readonly<{
    /**
     No page was written, or it was removed.
     */
      kind: 'missing';
    }>;

/**
 Reads one entry's page, or that there is none.

 @param path - page path

 @returns Its text, or `missing`

 @throws Whatever the read raised other than ENOENT

 @example
 ```ts
 const page = await pageOnDisk({ path, },);
 ```
 */
async function pageOnDisk(
  { path, }: { readonly path: string; },
): Promise<PageOnDisk> {
  try {
    return {
      kind: 'read',
      text: await readFile(
        path,
        'utf8',
      ),
    };
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    return { kind: 'missing', };
  }
}

/**
 Rewrites one page from its artifact, or says why it could not.

 @param artifact - settled artifact the page is rebuilt from

 @param why - why the page needs writing

 @param publishDir - root of the mirrored tree

 @param readPair - reads the pair at the artifact's commit

 @param l - pass logger

 @returns Republished, or left with the reason

 @example
 ```ts
 const outcome = await rebuildPage({ artifact, why: 'missing', publishDir, readPair, l, },);
 ```
 */
async function rebuildPage(
  {
    artifact,
    why,
    publishDir,
    readPair,
    l,
  }: {
    readonly artifact: ParsedTwoLaneArtifact;
    readonly why: 'disagreed' | 'missing';
    readonly publishDir: string;
    readonly readPair: CorpusPairReader;
    readonly l: Logger;
  },
): Promise<RepublishOutcome> {
  try {
    /**
     The pair as it stood when the artifact was settled.
     */
    const {
      sourceText,
      targetText: corpusTarget,
      sourceEntryInfo,
    } = await readPair({
      entryId: artifact.id,
      corpusSha: artifact.corpusSha,
      generation: artifact.artifactSchemaVersion,
    },);
    /**
     The carve the artifact's recipe gives, over the archive the run carved:
     the one the artifact stored where it kept one (ledger A18), since the
     pass reshapes the archive before it carves and shihai4h's page came out 9
     characters short, every wording in place, spliced into the corpus copy.
     */
    const {
      prepared,
      reproduction,
    } = rebuildPreparation({
      artifact,
      sourceText,
      targetText: corpusTarget,
      ...((sourceEntryInfo === undefined) ? {} : { sourceEntryInfo, }),
    },);
    if (reproduction.kind === 'moved') {
      return {
        kind: 'left',
        why,
        because: `carve moved: ${reproduction.detail}`,
      };
    }
    await publishFixedPage({
      artifact,
      slices: prepared.slices,
      archiveText: prepared.targetText,
      sourceText,
      entryId: artifact.id,
      publishDir,
      l,
      ...((prepared.archiveOriginalSpans === undefined)
        ? {}
        : { archiveOriginalSpans: prepared.archiveOriginalSpans, }),
    },);
    return {
      kind: 'republished',
      why,
    };
  }
  catch (error) {
    return {
      kind: 'left',
      why,
      because: refusalText({ error, },),
    };
  }
}

/**
 Judges one settled entry's page and rewrites it when it needs it.

 @param entryId - entry whose artifact to read

 @param artifactsDir - settled artifact root

 @param publishDir - root of the mirrored tree

 @param readPair - reads the pair at the artifact's commit

 @param l - pass logger

 @returns What became of the page

 @example
 ```ts
 const outcome = await republishOne({ entryId, artifactsDir, publishDir, readPair, l, },);
 ```
 */
async function republishOne(
  {
    entryId,
    artifactsDir,
    publishDir,
    readPair,
    l,
  }: {
    readonly entryId: string;
    readonly artifactsDir: string;
    readonly publishDir: string;
    readonly readPair: CorpusPairReader;
    readonly l: Logger;
  },
): Promise<RepublishOutcome> {
  try {
    /**
     The settled artifact.
     */
    const artifact = parseSettledTwoLaneArtifact({
      value: await readRunJson({
        path: join(
          artifactsDir,
          artifactFileNameOf({ entryId, },),
        ),
      },),
    },);
    /**
     The page it produced, or that there is none.
     */
    const page = await pageOnDisk({
      path: fixedPagePath({
        publishDir,
        entryId: artifact.id,
      },),
    },);
    if (page.kind === 'missing') {
      return await rebuildPage({
        artifact,
        why: 'missing',
        publishDir,
        readPair,
        l,
      },);
    }
    /**
     The page judged against its artifact by this build's reading.
     */
    const { agreement, } = pageAgreement({
      artifact,
      pageText: page.text,
    },);
    return (agreement === 'disagreed')
      ? await rebuildPage({
        artifact,
        why: 'disagreed',
        publishDir,
        readPair,
        l,
      },)
      : { kind: 'agreed', };
  }
  catch (error) {
    return {
      kind: 'left',
      why: 'unreadable',
      because: refusalText({ error, },),
    };
  }
}

/**
 Judges every settled entry's page in a runs directory and rewrites from its
 artifact each one that is missing or disagrees, logging what became of each.

 @param entryIds - entries with an artifact

 @param artifactsDir - settled artifact root

 @param publishDir - root of the mirrored tree

 @param readPair - reads the pair at an artifact's commit

 @param l - pass logger

 @returns One row per entry, in the order given

 @example
 ```ts
 const rows = await republishSettledPages({ entryIds, artifactsDir, publishDir, readPair, l, },);
 ```
 */
export async function republishSettledPages(
  {
    entryIds,
    artifactsDir,
    publishDir,
    readPair,
    l,
  }: {
    readonly entryIds: readonly string[];
    readonly artifactsDir: string;
    readonly publishDir: string;
    readonly readPair: CorpusPairReader;
    readonly l: Logger;
  },
): Promise<readonly RepublishRow[]> {
  /**
   Each entry's outcome, judged side by side.
   */
  const rows = await Promise.all(entryIds.map(async function judgeOne(entryId,): Promise<RepublishRow> {
    return {
      entryId,
      outcome: await republishOne({
        entryId,
        artifactsDir,
        publishDir,
        readPair,
        l,
      },),
    };
  },),);
  for (const {
    entryId,
    outcome,
  } of rows) {
    if (outcome.kind === 'republished')
      l.info(`REPUBLISHED entry=${entryId} why=${outcome.why}: page rewritten from its artifact`,);
    if (outcome.kind === 'left')
      l.warn(`REPUBLISH LEFT entry=${entryId} why=${outcome.why}: page left as it was; because=${outcome.because}`,);
  }
  /**
   Pages rewritten.
   */
  const rewritten = rows.filter(function wasRepublished({ outcome, },): boolean {
    return outcome.kind === 'republished';
  },);
  /**
   Pages that needed writing and were left.
   */
  const left = rows.filter(function wasLeft({ outcome, },): boolean {
    return outcome.kind === 'left';
  },);
  l.info(
    `republish: ${String(rows.length,)} settled ${
      wordForCount({
        count: rows.length,
        one: 'page',
        many: 'pages',
      },)
    } judged, ${String(rewritten.length,)} rewritten, ${
      String(left.length,)
    } left`,
  );
  return rows;
}

//endregion Page republish
