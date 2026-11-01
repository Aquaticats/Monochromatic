import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { leadingMarkWidth, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { foldInvisibleVariants, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type ArchiveRetainedLine,
  stripStubMarkersWithOrigins,
} from '@monochromatic-dev/translation-repair-page-check/ts';

//region Pass archive
// Archive English enters every pass stage through one transform: the
// invisible-variant fold, then the stub-marker strip (`archive-stub.ts`, which
// removes a notice that the page is unfinished as it removes a placeholder), in
// that order so a marker spelled with an invisible variant is folded before it
// is read. A byte order mark opening the archive is no content and is not
// folded. Preparation, both lanes, the artifact's stored archive and the
// published page all describe the text this returns, so no later check has to
// know a marker was ever there.

/**
 Normalizes archive bytes before preparation and every downstream decision.

 @param text - archive English as stored in corpus

 @param l - entry logger, which records every stub marker removed so a run's
 log witnesses it

 @returns Normalized text and retained pinned-line coordinates

 @example
 ```ts
 const { text, lines, } = passArchiveWithOrigins({ text: 'non‑binary', l, });
 ```
 */
export function passArchiveWithOrigins(
  {
    text,
    l,
  }: {
    readonly text: string;
    readonly l: Logger;
  },
): {
  readonly text: string;
  readonly lines: readonly ArchiveRetainedLine[]
} {
  /**
   The byte order mark the archive opens with, which is no content of the
   page: the fold leaves it where it stands, as every reader of the page
   reads it, so an archive nothing else changes comes back byte for byte. A
   mark anywhere else is still folded away.
   */
  const mark = text.slice(
    0,
    leadingMarkWidth({ text, },),
  );
  /**
   Visible text from shared fold.
   */
  const { text: folded, } = foldInvisibleVariants({ text: text.slice(mark.length,), },);
  /**
   Folded text without placeholder paragraphs, and what was removed.
   */
  const {
    text: stripped,
    stripped: markers,
    lines,
  } = stripStubMarkersWithOrigins({ text: `${mark}${folded}`, },);
  for (const marker of markers) {
    l.warn(
      `archive: stripped stub marker ${JSON.stringify(marker.text,)} at line ${String(marker.lineNumber,)}: a `
        + 'placeholder, or a notice that the English page is unfinished, is not content the ORIGINAL carries '
        + '(doc/decision/translation-repair-good-result-over-bad-original.md)',
    );
  }
  return {
    text: stripped,
    lines,
  };
}

/**
 {@inheritDoc passArchiveWithOrigins}

 @returns Normalized archive text without exposing provenance metadata to existing callers
 */
export function passArchiveText({
  text,
  l,
}: {
  readonly text: string;
  readonly l: Logger;
},): string {
  return passArchiveWithOrigins({
    text,
    l,
  },)
    .text;
}

//endregion Pass archive
