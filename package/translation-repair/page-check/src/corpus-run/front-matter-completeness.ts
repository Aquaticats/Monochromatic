import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  leadingMarkWidth,
  splitFrontMatter,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { validateFrontMatterTranslation, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { archiveFrontMatterStands, } from './archive-front-matter.ts';
import {
  directoryIdNameStands,
  namesDirectoryId,
} from '@monochromatic-dev/translation-repair-glossary/ts';

//region Front matter publication completeness
// Final page must retain parseable metadata under explicit reviewed slice.
//
// STRUCTURAL CHECKS ONLY, by the owner's decision of 2026-09-02. The rule of
// 2026-08-28 ("review visible front matter", written for archives whose
// metadata was never translated) refused a page whose metadata equalled the
// archive's while the source's differed, reading that as nobody having
// reviewed the slice. Chinese and English metadata always differ, so the
// trigger was a proxy for nothing: it discarded the Carena0442 pass, whose
// judges had split, and would have discarded Toka_ls, whose gate kept the
// archive six ballots to two, and one night of machinery reading which panel
// had chosen the keep (commits daaf0ffa0, 6f70a2085, 1160ebb4c) answered a
// question that only existed because of the proxy. Metadata is judged like
// every other slice by the lanes, the contest and the gate, and the artifact
// keeps their records. What stays here is what is structural: the metadata
// slice sits where the preparation put it, the page parses, the identity and
// attribution rules hold, and the visible name is not the directory id where
// the source names the person differently.

/**
 Refusal when published front matter fails a structural check.

 @example
 ```ts
 throw new FrontMatterCompletenessError({ entryId: 'Cat', reason: 'missing-slice', });
 ```
 */
export class FrontMatterCompletenessError extends Error {
  /**
   Message names entry and structural reason only.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds refusal.

   @param entryId - entry refused

   @param reason - structural check that failed: `missing-slice` when the
   preparation carries no metadata slice where it must, `invalid-page` when
   the page's metadata does not parse or breaks the identity or attribution
   rules, `directory-id-name` when the page's visible name is the directory id
   while the source's is not, `archive-front-matter` when the archive's
   front matter stands and the page or the preparation did not leave it alone
   */
  public constructor(
    {
      entryId,
      reason,
    }: {
      readonly entryId: string;
      readonly reason: 'missing-slice' | 'invalid-page' | 'directory-id-name' | 'archive-front-matter';
    },
  ) {
    super(`entry ${entryId} front matter is not publishable (${reason})`,);
    this.name = 'FrontMatterCompletenessError';
  }
}

/**
 Refuses page whose metadata fails a structural check.

 @param entryId - entry being published

 @param sourceText - complete original page

 @param archiveText - complete archive page before lane changes

 @param pageText - assembled page candidate

 @param slices - preparation carrying explicit syntax role

 @throws FrontMatterCompletenessError when metadata role or syntax differs,
 when the page's metadata does not parse or breaks the identity or
 attribution rules, or when its visible name is still the directory id while
 the source names the person differently

 @example
 ```ts
 assertFrontMatterComplete({ entryId, sourceText, archiveText, pageText, slices, });
 ```
 */
export function assertFrontMatterComplete(
  {
    entryId,
    sourceText,
    archiveText,
    pageText,
    slices,
  }: {
    readonly entryId: string;
    readonly sourceText: string;
    readonly archiveText: string;
    readonly pageText: string;
    readonly slices: readonly ChunkPair[];
  },
): void {
  /**
   Parsed source page.
   */
  const source = splitFrontMatter({ text: sourceText, });
  /**
   Parsed archive page.
   */
  const archive = splitFrontMatter({ text: archiveText, });
  /**
   Presence of source metadata.
   */
  const sourcePresent = source.frontMatter !== undefined;
  /**
   Presence of archive metadata.
   */
  const archivePresent = archive.frontMatter !== undefined;
  /**
   Parsed assembled page.
   */
  const page = splitFrontMatter({ text: pageText, });
  /**
   Archive metadata when target declares it.
   */
  const { frontMatter: archiveMetadata, } = archive;
  /**
   Assembled metadata when final page declares it.
   */
  const { frontMatter: pageMetadata, } = page;
  if (!sourcePresent) {
    if (!archivePresent) {
      if (pageMetadata !== undefined) {
        throw new FrontMatterCompletenessError({
          entryId,
          reason: 'invalid-page',
        },);
      }
      return;
    }
    if ((pageMetadata === undefined)
      || (archiveMetadata === undefined)
      || (pageMetadata.raw !== archiveMetadata.raw)) {
      throw new FrontMatterCompletenessError({
        entryId,
        reason: 'invalid-page',
      },);
    }
    return;
  }

  /**
   Metadata slices, which must be exactly slice zero where the lanes render
   the front matter and none at all where the archive's stands.
   */
  const metadataSlices = slices.filter(function isFrontMatter(slice,): boolean {
    return slice.syntax === 'front-matter';
  },);
  // THE ARCHIVE'S FRONT MATTER STANDS (the owner's rule of 2026-09-08): the
  // page carries it byte for byte and the preparation made no metadata slice,
  // since nothing was to render it. Recomputed here from the two documents
  // rather than read off the preparation, so a preparation that rendered a
  // standing archive is refused instead of trusted.
  if (archiveFrontMatterStands({
    entryId,
    sourceText,
    archiveText,
  },)) {
    if ((metadataSlices.length > 0)
      || (pageMetadata === undefined)
      || (archiveMetadata === undefined)
      || (pageMetadata.raw !== archiveMetadata.raw)) {
      throw new FrontMatterCompletenessError({
        entryId,
        reason: 'archive-front-matter',
      },);
    }
    return;
  }
  /**
   Sole metadata slice when count is valid.
   */
  const [metadataSlice,] = metadataSlices;
  if ((metadataSlices.length !== 1) || (metadataSlice === undefined)) {
    throw new FrontMatterCompletenessError({
      entryId,
      reason: 'missing-slice',
    },);
  }

  if ((pageMetadata === undefined)
    || (source.frontMatter === undefined)) {
    throw new FrontMatterCompletenessError({
      entryId,
      reason: 'invalid-page',
    },);
  }

  /**
   Exact source metadata bytes preparation must have reviewed.
   */
  const { raw: sourceFrontMatter, } = source.frontMatter;
  /**
   Exact archive metadata bytes preparation reviewed,
   empty for source-only insertion.
   */
  const archiveFrontMatter = archiveMetadata?.raw ?? '';
  /**
   Where the metadata slice has to start in the original: at its opening
   fence, after any byte order mark the page opens with.
   */
  const sourceStart = source.frontMatter
    .startOffset;
  /**
   Where the metadata slice has to start in the archive: at its opening fence,
   or, where it has none, after the byte order mark its page opens with, which
   is where the preparation inserts.
   */
  const targetStart = archiveMetadata?.startOffset ?? leadingMarkWidth({ text: archiveText, },);
  /**
   Source span metadata slice claims.
   */
  const { source: sourceSlice, } = metadataSlice;
  /**
   Target span metadata slice claims.
   */
  const { target: targetSlice, } = metadataSlice;
  /**
   Whether explicit role points anywhere but exact metadata spans.
   */
  const misplaced = (slices.at(0,) !== metadataSlice)
    || (sourceSlice.sliceIndex !== 0)
    || (targetSlice.sliceIndex !== 0)
    || (sourceSlice.startOffset !== sourceStart)
    || (targetSlice.startOffset !== targetStart)
    || (sourceSlice.endOffset !== (sourceStart + sourceFrontMatter.length))
    || (targetSlice.endOffset !== (targetStart + archiveFrontMatter.length))
    || (sourceSlice.text !== sourceFrontMatter)
    || (targetSlice.text !== archiveFrontMatter)
    || ((archivePresent) && isInsertionChunk(targetSlice,))
    || ((!archivePresent) && (!isInsertionChunk(targetSlice,)));
  if (misplaced) {
    throw new FrontMatterCompletenessError({
      entryId,
      reason: 'missing-slice',
    },);
  }

  /**
   Exact page metadata bytes.
   */
  const { raw: pageFrontMatter, } = pageMetadata;
  /**
   Structural validation before persistence.
   */
  const validation = validateFrontMatterTranslation({
    sourceText: sourceFrontMatter,
    pageText: archiveFrontMatter,
    candidateText: pageFrontMatter,
  },);
  if (validation.kind !== 'valid') {
    throw new FrontMatterCompletenessError({
      entryId,
      reason: 'invalid-page',
    },);
  }
  // THE PAGE'S OWN VISIBLE NAME, whatever the archive carried and whether or
  // not any lane changed the slice: a person published under the folder name is
  // the one metadata defect bytes alone ever caught, and it stays refused.
  //
  // ONLY WHERE THE SOURCE NAMES THE PERSON DIFFERENTLY. A census of the pinned
  // corpus on 2026-09-02 found 23 of 92 archives naming the directory; 8 of
  // those (Anilovr, Arita, ArtsEpiphany, Hangmster, keyword233, Mio, mone,
  // s5ehfr9) do so because the handle IS the person's name in the source too,
  // and the other 15 show the directory id where the source has a name of its
  // own. Refusing on the page alone would have refused the 8 forever.
  //
  // AND NOT WHERE THE ID IS A RENDERING OF THE NAME, the owner's decision of
  // 2026-09-07 after the Huasheng page was refused for naming 椛笙 by its
  // pinyin: the id stands when it spells the source's name, when the source
  // carries it among its aliases, or when the page or the archive carries a
  // Latin-script alias other than the id (`directory-id-name.ts`).
  /**
   Whether the assembled page shows the directory id as the person's name.
   */
  const pageNamesDirectory = namesDirectoryId({
    metadata: pageMetadata,
    entryId,
  },);
  /**
   Whether the source does too, which makes the handle the person's name.
   */
  const sourceNamesDirectory = namesDirectoryId({
    metadata: source.frontMatter,
    entryId,
  },);
  if (!pageNamesDirectory)
    return;
  if (sourceNamesDirectory)
    return;
  if (directoryIdNameStands({
    entryId,
    source: source.frontMatter,
    page: pageMetadata,
    archives: (archiveMetadata === undefined) ? [] : [archiveMetadata,],
  },))
    return;

  throw new FrontMatterCompletenessError({
    entryId,
    reason: 'directory-id-name',
  },);
}

//endregion Front matter publication completeness
