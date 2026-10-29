import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { alignHeadingsForced, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  ContentChunk,
  RepairDocument,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  chunkByHeadings,
  chunkLabel,
  pairingQuestionKey,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type FanOutMode,
  type NumberedSection,
  type PairedDocumentRecord,
  pairSectionsWithRoster,
  readSectionPairing,
  requireSectionPairingRefusal,
  type SectionPair,
  type SectionPairingError,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Section pairing round
// BUYS A SECTION PAIRING, BUT ONLY WHERE THE DETERMINISTIC ALIGNER REFUSED.
//
// Measured over the pinned corpus: 85 of 92 entries have equal section shape
// and never reach the aligner at all, and 5 of the remaining 7 align with no
// refusal. Two entries are ever asked, and on those two the aligner's affinity
// grid is almost entirely zeros, because it scores headings by token overlap
// and Chinese headings share no tokens with English ones.
//
// ASKING WHERE IT DID NOT REFUSE WOULD BE WORSE, not merely wasteful. A forced
// pairing lies on every optimal path, which is a stronger claim than a roster
// vote, and replacing one with the other would trade a proof for an opinion.

/**
 What one document's section round settled, and what it cost.

 @example
 ```ts
 const round: SectionRoundOutcome = { pairing: [], findings: [], };
 ```
 */
export type SectionRoundOutcome = {
  /**
   Correspondences to align on, empty when nobody was asked or nobody agreed.

   EMPTY MEANS KEEP THE DETERMINISTIC ALIGNER, and the caller must read it
   that way rather than as "align nothing": a document handed an empty pairing
   would come out with every section unpaired on both sides, which is the
   failure already recorded one scale down.
   */
  readonly pairing: readonly SectionPair[];

  /**
   What the round reported, in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Nothing bought and nothing to say, which is most entries.
 */
const NO_ROUND: SectionRoundOutcome = {
  pairing: [],
  findings: [],
};

/**
 Presents heading-bounded chunks as the numbered sections a sheet shows.

 @param chunks - one side's sections in document order

 @returns Same sections, numbered

 @example
 ```ts
 const numbered = asNumbered({ chunks, },);
 ```
 */
function asNumbered(
  { chunks, }: { readonly chunks: readonly ContentChunk[]; },
): readonly NumberedSection[] {
  return chunks.map(function toNumbered(
    chunk,
    index,
  ): NumberedSection {
    return {
      index,
      text: chunk.text,
    };
  },);
}

/**
 Whether the deterministic aligner leaves any section without a partner.

 @param sourceChunks - original sections in document order

 @param targetChunks - translation sections in document order

 @returns Whether asking a roster could add anything

 @example
 ```ts
 const refused = deterministicRefuses({ sourceChunks, targetChunks, },);
 ```
 */
function deterministicRefuses(
  {
    sourceChunks,
    targetChunks,
  }: {
    readonly sourceChunks: readonly ContentChunk[];
    readonly targetChunks: readonly ContentChunk[];
  },
): boolean {
  /**
   Whether both sides have equal counts and matching leading node kinds.

   EQUAL SHAPE PAIRS BY INDEX without the aligner being consulted, so there is
   no refusal to repair. The fast path stays by a measured decision (a roster
   agreed with index order on every equal-count entry), and checking it here
   would reopen that decision by a side effect.
   */
  const equalShape = (sourceChunks.length === targetChunks.length)
    && sourceChunks.every(function leadingKindMatches(
      chunk,
      index,
    ): boolean {
      return chunk.nodes[0]
        ?.kind
        === targetChunks[index]
          ?.nodes[0]
          ?.kind;
    },);
  if (equalShape)
    return false;

  return alignHeadingsForced({
    sourceHeadings: sourceChunks.map(function sourceLabel(chunk,): string {
      return chunkLabel(chunk,);
    },),
    targetHeadings: targetChunks.map(function targetLabel(chunk,): string {
      return chunkLabel(chunk,);
    },),
  },)
    .some(function unpaired(step,): boolean {
      return step.kind !== 'paired';
    },);
}

/**
 Names this question by the text it is about.

 OVER BOTH SIDES WHOLE, because the pairing is a claim about these two
 documents and any edit to either makes the stored answer a claim about
 something else.

 @param sourceSections - original sections as the sheet numbers them

 @param targetSections - translation sections as the sheet numbers them

 @param modelIds - roster that answers the question, so a round one bench
 settled is never resumed for another (ledger X13)

 @returns Cache key for this pairing question, one per question (ledger X15)

 @example
 ```ts
 const key = roundKey({ sourceSections, targetSections, modelIds, },);
 ```
 */
function roundKey(
  {
    sourceSections,
    targetSections,
    modelIds,
  }: {
    readonly sourceSections: readonly NumberedSection[];
    readonly targetSections: readonly NumberedSection[];
    readonly modelIds: readonly RosterModelId[];
  },
): string {
  return pairingQuestionKey({
    question: 'section',
    sourceTexts: sourceSections.map(function toText(section,): string {
      return section.text;
    },),
    targetTexts: targetSections.map(function toText(section,): string {
      return section.text;
    },),
    pictureContext: '',
    modelIds,
  },);
}

/**
 The refusal of a cached pairing read against the document's sections, or
 nothing when the record is a pairing a fresh reply could have been.

 READ THROUGH `readSectionPairing`, the whole reader a fresh reply goes
 through (indices, then strictly increasing on both sides), since the cache
 reader checks a record's shape only and the key names the sections' text. A
 record a fresh reply would be refused for was not written by this pipeline
 for this question: a damaged or hand-edited file. Handed on as stored, a
 translation section the document lacks stopped the alignment with a
 missing-value error naming nothing, an original section it lacks left the
 document with no slice, a repeated or reversed pairing stopped the slicer
 hours later, and one original paired with two translations aligned the
 later section's pair silently.

 @param pairs - pairs the cached record holds

 @param sourceCount - original sections of the document

 @param targetCount - translation sections of the document

 @returns Refusal in a one-element list, or an empty list when the
 record fits

 @throws The caught value unchanged when the reader fails other than by
 refusing the pairing

 @example
 ```ts
 const [misfit,] = cachedSectionPairingMisfit({ pairs: cached.pairs, sourceCount: 3, targetCount: 2, },);
 ```
 */
function cachedSectionPairingMisfit(
  {
    pairs,
    sourceCount,
    targetCount,
  }: {
    readonly pairs: PairedDocumentRecord['pairs'];
    readonly sourceCount: number;
    readonly targetCount: number;
  },
): readonly SectionPairingError[] {
  try {
    readSectionPairing({
      value: { pairs, },
      sourceCount,
      targetCount,
    },);
    return [];
  }
  catch (error) {
    return [requireSectionPairingRefusal({ error, },),];
  }
}

/**
 Buys a section pairing when, and only when, the aligner refused something.

 @param client - injected model client

 @param modelIds - roster to ask

 @param source - parsed original document

 @param target - parsed translation document

 @param signal - caller's steering

 @param exchangeTimeoutMs - per-call bound

 @param l - driver logger

 @param sectionCache - store a settled round is republished from, so a resumed
 entry buys nothing; a stored round the section reader refuses is a miss

 @param fanOut - seats a pairing round asks: the window of quorum plus one
 by default, or the whole bench a fixture scripting every seat asks for,
 since the window is rotated by the sheet text and a sheet edit elsewhere
 moves which seats it holds

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Pairing to align on, empty to keep the deterministic aligner

 @example
 ```ts
 const round = await buySectionPairing({ client, modelIds, source, target, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function buySectionPairing(
  {
    client,
    modelIds,
    source,
    target,
    signal,
    exchangeTimeoutMs,
    l,
    sectionCache,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly source: RepairDocument;
    readonly target: RepairDocument;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly sectionCache?: SliceCache<PairedDocumentRecord>;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<SectionRoundOutcome> {
  /**
   Original-side sections.
   */
  const sourceChunks = chunkByHeadings({ document: source, },);

  /**
   Translation-side sections.
   */
  const targetChunks = chunkByHeadings({ document: target, },);

  if ((sourceChunks.length === 0) || (targetChunks.length === 0))
    return NO_ROUND;
  if (!deterministicRefuses({
    sourceChunks,
    targetChunks,
  },))
    return NO_ROUND;

  /**
   Original sections as the sheet numbers them.
   */
  const sourceSections = asNumbered({ chunks: sourceChunks, },);

  /**
   Translation sections as the sheet numbers them.
   */
  const targetSections = asNumbered({ chunks: targetChunks, },);

  /**
   This question's identity.
   */
  const key = roundKey({
    sourceSections,
    targetSections,
    modelIds,
  },);

  /**
   A round an earlier run already bought for these two documents.
   */
  const cached = sectionCache?.resumed
    .get(key,);

  /**
   Why the cached round does not fit these documents' sections, absent when
   it fits or nothing was cached.
   */
  const [misfit,] = (cached === undefined)
    ? []
    : cachedSectionPairingMisfit({
      pairs: cached.pairs,
      sourceCount: sourceChunks.length,
      targetCount: targetChunks.length,
    },);
  if (misfit !== undefined)
    l.warn(
      `the section pairing cache misses: the record under ${key} does not fit its sections (${misfit.message}), `
        + 'so the roster is asked again',
    );
  if ((cached !== undefined) && (misfit === undefined)) {
    // REPUBLISHED BEFORE ANYTHING IS DECIDED, for the reason the block round
    // gives: this run asks nobody, so every finding the first run reported is
    // reported by nothing at all unless it comes back off disk.
    /**
     The two halves of a stored round: what it agreed, and what it reported.
     */
    const {
      pairs: cachedPairs,
      findings: cachedFindings,
    } = cached;
    l.info(`section pairing resumed: ${String(cachedPairs.length,)} ${
      wordForCount({
        count: cachedPairs.length,
        one: 'correspondence',
        many: 'correspondences',
      },)
    }`,);
    return {
      pairing: cachedPairs,
      findings: cachedFindings,
    };
  }

  l.warn(
    `the aligner refused sections on this document: asking ${String(modelIds.length,)} ${
      wordForCount({
        count: modelIds.length,
        one: 'voice',
        many: 'voices',
      },)
    } `
      + `which of ${String(sourceChunks.length,)} original ${
        wordForCount({
          count: sourceChunks.length,
          one: 'section',
          many: 'sections',
        },)
      } render as which of ${
        String(targetChunks.length,)
      } translation ${
        wordForCount({
          count: targetChunks.length,
          one: 'section',
          many: 'sections',
        },)
      }`,
  );

  /**
   What the roster agreed on.
   */
  const outcome = await pairSectionsWithRoster({
    client,
    modelIds,
    sourceSections,
    targetSections,
    signal,
    exchangeTimeoutMs,
    l,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
    stageSettings,
  },);

  /**
   What the round agreed, beside how many voices stood behind it.
   */
  const {
    pairs,
    usable,
    heard,
  } = outcome;

  /**
   Everything this round reported, gathered before any of it is stored.
   */
  const findings = [...outcome.findings,];
  if (usable > 0)
    findings.push(
      `section-pairing paired ${String(pairs.length,)} of ${
        String(sourceChunks.length,)
      } original and ${String(targetChunks.length,)} translation ${
        wordForCount({
          count: targetChunks.length,
          one: 'section',
          many: 'sections',
        },)
      }, from ${
        String(usable,)
      } usable ${
        wordForCount({
          count: usable,
          one: 'voice',
          many: 'voices',
        },)
      } of ${String(heard,)} heard`,
    );
  if (pairs.length === 0)
    findings.push('section-pairing fell back to the deterministic aligner',);

  // A ROUND NOBODY ANSWERED IS NOT AN ANSWER, and must not be cached: the
  // roster was unreachable, not undecided, and caching that would make one bad
  // minute permanent for this entry. A round that WAS answered caches even when
  // it agreed on nothing, because that is a stable fact about these documents.
  if (usable > 0)
    await sectionCache?.persist({
      key,
      serialized: JSON.stringify({
        pairs,
        findings,
      },),
    },);

  if (pairs.length === 0)
    l.warn('no agreed section pairing, keeping the deterministic aligner',);

  return {
    pairing: pairs,
    findings,
  };
}

//endregion Section pairing round
