import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  type PairedPreparation,
  prepareDocumentPairWithRoster,
} from '@monochromatic-dev/translation-repair-archive/ts';
import { keepBench, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { corpusNameLines, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { pageIdentityLines, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  parseDocument,
  photoReferences,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  openPairingCache,
  openSectionPairingCache,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';
import { entryInfoFindingsOf, } from '@monochromatic-dev/translation-repair-insertion/ts';
import {
  archiveBlockSourceContexts,
  frontMatterAuthorityOf,
  repairArchiveBlocks,
} from '@monochromatic-dev/translation-repair-page-check/ts';
import { passArchiveText, } from './pass-archive.ts';
import { attestPassReferences, } from './pass-attest-references.ts';
import {
  openPageTitleCache,
  passPageTitles,
} from './pass-page-titles.ts';
import { relabelArchiveFootnotes, } from './pass-footnote-relabel.ts';
import type { PairedReading, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { PassPrepareRequest, } from './pass-prepare-request.ts';

//region Pass preparation
// Corpus-specific shell owns pairing cache namespaces and reviews inherited
// blocks outside source claims before any later quality-stage purchase.
//
// LINEAR BY DESIGN: one preparation, at most one relabel of the archive's
// footnote labels to the original's with its re-preparation, at most one
// archive correction round, one re-preparation over the corrected archive.
// Each re-preparation is the structural consequence of having edited the
// archive, not a rejection-driven re-ask; blocks still unclaimed after the
// last become findings (doc/planning/translation-repair-no-loop-design.md).

/**
 What the pass's preparation hands the entry: the slicing both lanes run
 over, with every finding the preparation reported on its
 `alignmentFindings`, and the definition pairs the relabel read.

 NO FINDINGS LIST BESIDE IT (ledger B54). One was returned here, and the
 entry pipeline never read it, so no artifact carried a finding of the
 relabel, the evidence rounds or the archive review; only the pairing
 round's, which ride `alignmentFindings` (`prepare-with-pairing.ts`),
 reached one.

 @example
 ```ts
 const { prepared, } = await preparePassEntry({ client, entryId, entryCacheDir, pipelineDigest, modelIds, sourceText, targetText, signal, exchangeTimeoutMs, l, outsideReads, stageSettings, },);
 ```
 */
export type PassPreparation = Omit<PairedPreparation, 'findings'>;

/**
 A preparation carrying the pass's own findings on the channel the artifact
 records, after the pairing round's already there.

 @param paired - preparation the lanes run over

 @param findings - what the relabel, the evidence rounds and the archive
 review reported, which the artifact would otherwise never see

 @returns The preparation with them on its `alignmentFindings`

 @example
 ```ts
 return carryingPassFindings({ paired: sightedPaired, findings: passFindings, },);
 ```
 */
function carryingPassFindings(
  {
    paired,
    findings,
  }: {
    readonly paired: PairedPreparation;
    readonly findings: readonly string[];
  },
): PassPreparation {
  return {
    prepared: {
      ...paired.prepared,
      alignmentFindings: [
        ...paired.prepared
          .alignmentFindings,
        ...findings,
      ],
    },
    footnoteDefinitionPairs: paired.footnoteDefinitionPairs,
  };
}

/**
 Prepares one pass entry with cached roster pairing and publication safety.

 @param client - shared provider client

 @param entryId - corpus entry being settled

 @param entryCacheDir - entry cache root

 @param pipelineDigest - cache generation

 @param modelIds - pairing roster

 @param sourceText - source page

 @param targetText - archive page

 @param signal - entry deadline

 @param exchangeTimeoutMs - per-call ceiling

 @param l - entry logger

 @param readPictures - shared entry reader supplying picture support before archive review

 @param beforeItem - one per-item hook for every stage that asks the roster, so
 once a hold re-seats it the later stages stay re-seated (ledger X12)

 @param outsideReads - what the preparation reads from outside the
 pipeline, required so no caller inherits the run's keys and caches by
 leaving it out (ledger X19): the readers the pass's entry file builds over
 its environment in a run

 @param fanOut - seats a pairing round asks: the window of quorum plus one
 by default, or the whole bench a fixture scripting every seat asks for,
 since the window is rotated by the sheet text and a sheet edit elsewhere
 moves which seats it holds

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Prepared slices, every finding on their `alignmentFindings`

 @example
 ```ts
 const paired = await preparePassEntry({ client, entryId, entryCacheDir, pipelineDigest, modelIds, sourceText, targetText, signal, exchangeTimeoutMs, l, outsideReads, stageSettings, });
 ```
 */
export async function preparePassEntry(
  {
    client,
    entryId,
    entryCacheDir,
    pipelineDigest,
    modelIds,
    sourceText,
    targetText,
    signal,
    exchangeTimeoutMs,
    l,
    readPictures,
    beforeItem,
    outsideReads,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<PassPrepareRequest>,
): Promise<PassPreparation> {
  l.debug(`${preparePassEntry.name}: preparing entry ${entryId}`,);
  /**
   Archive bytes both deciders judge, normalized before preparation so
   spans, candidates, artifact and published page all describe the same
   visible text (`pass-archive.ts`).
   */
  const archiveText = passArchiveText({
    text: targetText,
    l,
  },);
  /**
   Whose front matter the page carries, decided on the archive as inherited
   and shared by both preparations, since the block correction round never
   touches metadata.
   */
  const frontMatterAuthority = frontMatterAuthorityOf({
    entryId,
    sourceText,
    archiveText,
  },);
  l.info(
    `FRONT MATTER entry=${entryId} authority=${frontMatterAuthority}: ${
      (frontMatterAuthority === 'archive')
        ? 'the archive translated it, so it ships as it stands and no lane writes it'
        : 'the archive never translated it, so the lanes render slice zero'
    }`,
  );
  /**
   Cache for block-pairing rounds across revised archive preparations.
   */
  const pairingCache = await openPairingCache({
    dir: entryCacheDir,
    generation: pipelineDigest,
  },);
  /**
   Cache for section-pairing rounds across revised archive preparations.
   */
  const sectionCache = await openSectionPairingCache({
    dir: entryCacheDir,
    generation: pipelineDigest,
  },);
  /**
   Web-lookup evidence for the works the original names, bought once per
   title and cached durably (the owner's rule of 2026-09-02), the same lines
   for both preparations so a corrected archive does not change what the
   sheets are told about a title.
   */
  const workTitleLines = await outsideReads.workTitles({
    sourceText,
    signal,
    l,
  },);
  /**
   How the corpus renders the people of other entries this original names
   (class seventy-eight, `corpus-name-index.ts`), read off every entry's
   front matter at the pin, so a handle another entry declares is never
   transliterated by guess.
   */
  const corpusNameContext = corpusNameLines({
    text: sourceText,
    names: await outsideReads.corpusNames(),
    ownId: entryId,
  },);
  /**
   Name lines under the heading, none when the block is empty.
   */
  const corpusNameEntries = corpusNameContext.slice(1,);
  l.info(
    `CORPUS NAMES entry=${entryId} named=${corpusNameEntries.length}: ${
      (corpusNameEntries.length === 0)
        ? 'the original names nobody another entry declares'
        : corpusNameEntries.join('; ',)
    }`,
  );
  /**
   The entry's `info.yml` at the pin, which records the date of death; absent
   when the entry has none, which records no death, and unreadable when the file
   refuses to parse, which records none either and is said on the artifact.
   */
  const entryInfo = await outsideReads.entryInfo({
    entryId,
    l,
  },);

  /**
   Parsed `info.yml` every preparation of this entry reads, absent when the
   entry has no such file or one that refuses to parse.
   */
  const sourceEntryInfo = (entryInfo.kind === 'read')
    ? entryInfo.info
    : undefined;
  /**
   The page's declared identity without the lexicon's own lines, assembled
   as preparation assembles it (ledger B28): the lexicon asks for a work's
   official English title, which a web lookup or a note establishes.
   */
  const lexiconIdentityLines = pageIdentityLines({
    sourceDocument: parseDocument({ text: sourceText, },),
    targetDocument: parseDocument({ text: archiveText, },),
    sourceText,
    targetText: archiveText,
    contextLines: [
      ...workTitleLines,
      ...corpusNameContext,
    ],
    ...((sourceEntryInfo === undefined) ? {} : { sourceEntryInfo, }),
  },);
  l.debug(
    `${preparePassEntry.name}: the title lexicon reads ${String(lexiconIdentityLines.length,)} identity ${
      wordForCount({
        count: lexiconIdentityLines.length,
        one: 'line',
        many: 'lines',
      },)
    }`,
  );
  /**
   One rendering of each title the original repeats that the archive leaves
   unpaired (ledger H16), settled once by the bench on the roster the hook
   hands over and cached durably, the same lines for every preparation so a
   corrected archive does not change what the sheets are told about a title.
   */
  const pageTitles = await passPageTitles({
    client,
    modelIds,
    beforeItem: beforeItem ?? keepBench,
    sourceText,
    targetText: archiveText,
    identityContext: lexiconIdentityLines.join('\n',),
    cache: await openPageTitleCache({
      dir: entryCacheDir,
      generation: pipelineDigest,
    },),
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  },);
  /**
   Evidence lines bought or read outside preparation, the same for every
   preparation.
   */
  const contextLines = [
    ...workTitleLines,
    ...corpusNameContext,
    ...pageTitles.lines,
  ];
  /**
   What the pages the original links say (class thirty-five, the owner's
   decision of 2026-09-16), bought once per page and cached durably, the
   same block for both preparations; the critic and panel sheets read it so
   a detail the archive took from a cited reference is not deleted as an
   addition.
   */
  const referenceLines = await outsideReads.references({
    sourceText,
    signal,
    l,
  },);
  /**
   Archive details a reference states, attested by the bench with quotes
   checked word for word (class thirty-seven, 2026-09-16), on the roster the
   hook hands over (ledger X12); the pairing and the review take the hook
   itself and re-seat per section and per block.
   */
  const attestation = await attestPassReferences({
    client,
    modelIds,
    beforeItem: beforeItem ?? keepBench,
    sourceText,
    archiveText,
    referenceLines,
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  },);
  /**
   Reference lines with the attested lines under them.
   */
  const referenceContext = [
    referenceLines,
    ...attestation.lines,
  ]
    .filter(function isLine(line,): boolean {
      return line !== '';
    },)
    .join('\n',);
  /**
   Preparation over one archive text, the same roster, caches, context and
   authority each time: once over the archive as inherited, once more where
   the relabel rewrote it, and once more where the block correction round
   did.

   @param targetText - archive text to prepare over

   @param pictureReadings - what reading produced per picture, once they are
   read, so the pairing sheets see them (class thirty-four)

   @returns Prepared slices and pairing findings

   @example
   ```ts
   const paired = await prepareOver({ targetText: archiveText, },);
   ```
   */
  function prepareOver(
    {
      targetText: over,
      pictureReadings,
    }: {
      readonly targetText: string;
      readonly pictureReadings?: ReadonlyMap<string, PairedReading>;
    },
  ): Promise<PairedPreparation> {
    return prepareDocumentPairWithRoster({
      client,
      modelIds,
      pairingCache,
      sectionCache,
      sourceText,
      targetText: over,
      signal,
      exchangeTimeoutMs,
      l,
      contextLines,
      ...((referenceContext === '') ? {} : { referenceContext, }),
      ...((attestation.details
        .length
        === 0) ? {} : { attestedDetails: attestation.details, }),
      frontMatterAuthority,
      sealArchiveOriginal: true,
      ...((sourceEntryInfo === undefined) ? {} : { sourceEntryInfo, }),
      ...((pictureReadings === undefined) ? {} : { pictureReadings, }),
      ...((beforeItem === undefined) ? {} : { beforeSection: beforeItem, }),
      // Conditional spread keeps the knob absent instead of undefined.
      ...((fanOut === undefined) ? {} : { fanOut, }),
      stageSettings,
    },);
  }
  /**
   Preparation over the archive as inherited.
   */
  const firstPaired = await prepareOver({ targetText: archiveText, },);
  /**
   The archive under the original's footnote labels, read off the first
   preparation's slices (the nineteenth class, 2026-09-08).
   */
  const relabel = relabelArchiveFootnotes({
    entryId,
    slices: firstPaired.prepared
      .slices,
    definitionPairs: firstPaired.footnoteDefinitionPairs,
    sourceText,
    archiveText,
    l,
  },);
  /**
   Preparation the block correction round starts from: over the relabelled
   archive where the relabel changed it, since the labels moved the offsets.
   */
  const labelled = relabel.changed
    ? await prepareOver({ targetText: relabel.archiveText, },)
    : firstPaired;
  /**
   Spans the archive's translators' note sealed as the English original,
   which no slice covers and no lane writes (the owner's rule of 2026-09-08).
   */
  const sealedSpans = labelled.prepared
    .archiveOriginalSpans
    ?? [];
  for (const [at, span,] of sealedSpans.entries()) {
    l.info(
      `ARCHIVE ORIGINAL entry=${entryId} span=${String(at,)} [${String(span.startOffset,)}, ${
        String(span.endOffset,)
      }) of the archive ships as it stands under the note: ${span.note}`,
    );
  }
  /**
   Picture evidence precedes the pairing that decides which archive blocks are
   unclaimed, and so precedes any verdict that could remove a picture's
   translation: read off the labelled slices, which name every picture. A
   source naming no picture reads nothing and seats nobody for it.
   Pictures the source names, in document order.
   */
  const pictureNames = photoReferences({ text: sourceText, },);
  /**
   What reading produced per picture, absent for a source naming none.
   */
  const pictureReadings = (pictureNames.length === 0)
    ? undefined
    : await readPictures?.({ slices: labelled.prepared
      .slices, },);
  /**
   Readings worth showing a sheet, absent when nothing was read.
   */
  const sighted = ((pictureReadings === undefined) || (pictureReadings.size === 0))
    ? {}
    : { pictureReadings, };
  /**
   Preparation the block correction round starts from: over the same archive,
   paired again with the pictures in the sheets where there are any (class
   thirty-four, 2026-09-16), since a pairing bought blind sets an archive
   block translating a picture against the original block standing where the
   picture stands, and that block then never reaches the review as unclaimed.
   */
  const sightedPaired = ('pictureReadings' in sighted)
    ? await prepareOver({
      targetText: relabel.archiveText,
      ...sighted,
    },)
    : labelled;
  /**
   What the evidence rounds reported, the same for every preparation and so
   carried on every way out (ledger X18: the way out through a corrected
   archive once rebuilt its findings and dropped the attestation's).
   */
  const evidenceFindings = [
    ...attestation.findings,
    ...pageTitles.findings,
  ];
  /**
   The pass's own findings so far, the relabel's and the evidence rounds';
   the pairing round's already ride the preparation (`prepare-with-pairing.ts`).
   */
  const passFindings = [
    ...relabel.findings,
    ...evidenceFindings,
    ...entryInfoFindingsOf({ entryInfo, },),
  ];
  /**
   Unclaimed blocks not already licensed unchanged.
   */
  const pending = sightedPaired.prepared
    .unclaimedTargetBlocks;
  if (pending.length === 0) {
    return carryingPassFindings({
      paired: sightedPaired,
      findings: passFindings,
    },);
  }
  /**
   Declared names this preparation built, which the archive review reads.
   */
  const preparedIdentity = sightedPaired.prepared
    .identityContext;
  /**
   Selected corrections and retained licenses from the single review round.
   */
  const repaired = await repairArchiveBlocks({
    client,
    modelIds,
    targetText: relabel.archiveText,
    sourceContexts: archiveBlockSourceContexts({
      prepared: sightedPaired.prepared,
      ...sighted,
    },),
    blocks: pending,
    // WHAT PREPARATION ALREADY HOLDS (ledger B28): the review judges archive
    // wording against the original, and a detail a cited page states is the
    // translator's knowledge; the house rules it carries read a DECLARED NAMES
    // block too.
    ...((preparedIdentity === undefined) ? {} : { identityContext: preparedIdentity, }),
    ...((referenceContext === '') ? {} : { referenceContext, }),
    signal,
    exchangeTimeoutMs,
    l,
    ...((beforeItem === undefined) ? {} : { beforeBlock: beforeItem, }),
    stageSettings,
  },);
  if (repaired.targetText === relabel.archiveText) {
    return carryingPassFindings({
      paired: sightedPaired,
      findings: [
        ...passFindings,
        ...repaired.findings,
      ],
    },);
  }
  /**
   Re-preparation over the corrected archive, whose offsets the correction
   moved, with the same pictures in the sheets.
   */
  const secondPaired = await prepareOver({
    targetText: repaired.targetText,
    ...sighted,
  },);
  /**
   Blocks the single correction round could not claim.
   */
  const remaining = secondPaired.prepared
    .unclaimedTargetBlocks;
  return carryingPassFindings({
    paired: secondPaired,
    findings: [
      ...passFindings,
      ...repaired.findings,
      ...(remaining.length === 0
        ? []
        : [`unclaimed archive blocks remain after the single correction round: ${String(remaining.length,)}`,]),
    ],
  },);
}

//endregion Pass preparation
