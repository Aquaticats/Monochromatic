import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';

//region Archive-original publication completeness
// THE PAGE CARRIES EVERY SEALED SPAN AS THE ARCHIVE HAS IT, checked at
// publication rather than trusted from the preparation, for the same reason
// the front-matter guard recomputes its answer: a preparation that sealed a
// span and a lane that wrote over it anyway would otherwise ship the rewrite
// under a record saying it was sealed. Decided by the owner on 2026-09-08
// after the twelfth hakureico pass rewrote Hanasaka's letter in five places
// under a note saying the English was the original.
//
// BYTE FOR BYTE, since the seal's whole meaning is that no lane, no typography
// pass and no wrap touched the span. A page that carries the span's bytes at
// least once passes; it need not carry them at the recorded offsets, since
// the slices before the span may have grown or shrunk.

/**
 Refusal when a sealed span does not reach the page as the archive has it.

 CARRIES WHAT IT NAMES (ledger F-12): the entry, the span's index and the
 span's archive offsets are fields, so a caller can find the span without
 parsing the message, as the sibling completeness errors allow.

 @example
 ```ts
 throw new ArchiveOriginalCompletenessError({ entryId: 'hakureico', spanIndex: 0, startOffset: 120, endOffset: 480, });
 ```
 */
export class ArchiveOriginalCompletenessError extends Error {
  /**
   Message names entry, span index and offsets only.
   */
  readonly messageNamesOnly: true = true;

  /**
   Entry whose page lost the span.
   */
  readonly entryId: string;

  /**
   Which recorded span the page does not carry.
   */
  readonly spanIndex: number;

  /**
   Where the span starts in the archive page.
   */
  readonly startOffset: number;

  /**
   Where the span ends in the archive page, exclusive.
   */
  readonly endOffset: number;

  /**
   Builds refusal.

   @param entryId - entry refused

   @param spanIndex - which recorded span the page does not carry

   @param startOffset - span's first offset in the archive page

   @param endOffset - span's exclusive end in the archive page
   */
  public constructor(
    {
      entryId,
      spanIndex,
      startOffset,
      endOffset,
    }: {
      readonly entryId: string;
      readonly spanIndex: number;
      readonly startOffset: number;
      readonly endOffset: number;
    },
  ) {
    super(
      `entry ${entryId} page does not carry archive-original span ${String(spanIndex,)} (archive offsets `
        + `${String(startOffset,)} to ${String(endOffset,)}) as the archive has it`,
    );
    this.name = 'ArchiveOriginalCompletenessError';
    this.entryId = entryId;
    this.spanIndex = spanIndex;
    this.startOffset = startOffset;
    this.endOffset = endOffset;
  }
}

/**
 Refuses a page that does not carry every sealed span byte for byte.

 @param entryId - entry being published

 @param archiveText - complete archive page the spans index into

 @param pageText - assembled page candidate

 @param spans - spans the preparation sealed, empty when none

 @throws {@link ArchiveOriginalCompletenessError} when a span's archive
 bytes appear nowhere on the page

 @example
 ```ts
 assertArchiveOriginalComplete({ entryId, archiveText, pageText, spans: prepared.archiveOriginalSpans ?? [], },);
 ```
 */
export function assertArchiveOriginalComplete(
  {
    entryId,
    archiveText,
    pageText,
    spans,
  }: {
    readonly entryId: string;
    readonly archiveText: string;
    readonly pageText: string;
    readonly spans: readonly ArchiveOriginalSpan[];
  },
): void {
  for (const [spanIndex, span,] of spans.entries()) {
    /**
     The archive's bytes the seal covers.
     */
    const sealed = archiveText.slice(
      span.startOffset,
      span.endOffset,
    );
    if (!pageText.includes(sealed,))
      throw new ArchiveOriginalCompletenessError({
        entryId,
        spanIndex,
        startOffset: span.startOffset,
        endOffset: span.endOffset,
      },);
  }
}

//endregion Archive-original publication completeness
