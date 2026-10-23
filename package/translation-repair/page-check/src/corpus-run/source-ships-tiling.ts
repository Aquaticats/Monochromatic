import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type ComposedPage,
  composeSourcePage,
} from './source-ships-compose.ts';
import type { GrammarReader, } from './source-ships-runs.ts';
import type { WouldShipSource, } from './would-ship-text.ts';

//region Source ships tiling
// THE COMPOSITION RUN OVER WHAT AN ARTIFACT ALONE RECORDS. A page on disk is judged without the slicing of its entry
// (the artifact records each slice's archive wording and no offsets), so the archive is taken as the slices' archive
// wordings end to end, in the artifact's comparison order, and every slice as a span or an insertion at the offset the
// archive stands at when it is reached. That is the archive of an entry whose slices tile it; text the slicing left
// between slices is not recorded and not here. The composition is then the one `publishFixedPage` runs, so which runs a
// page is shipped without is decided in one place, and the verdict on a page read back is the verdict of the
// composition on the same text up to that gap.

/**
 A logger that writes nothing, for a composition run to judge a page and not to ship one.
 */
const SILENT: Logger = {
  debug: ignore,
  error: ignore,
  fatal: ignore,
  flush: async function flushNothing(): Promise<void> {
    // Nothing is buffered.
  },
  info: ignore,
  trace: ignore,
  warn: ignore,
};

/**
 Discards a message.

 @example
 ```ts
 ignore();
 ```
 */
function ignore(): void {
  // The composition is run to judge a page, which writes no line.
}

/**
 Slices and archive an artifact's comparison rows tile.

 @param artifact - settled entry, read for each slice's archive wording

 @returns The pairs and the archive end to end

 @example
 ```ts
 const { slices, archiveText, } = tilingOf({ artifact, },);
 ```
 */
export function tilingOf(
  { artifact, }: { readonly artifact: WouldShipSource; },
): {
  readonly slices: readonly ChunkPair[];
  readonly archiveText: string;
} {
  /**
   Offset in the archive the next slice starts at, advanced by every span.
   */
  const cursor = { at: 0, };
  /**
   The comparison rows, in slice order.
   */
  const { comparison, } = artifact;
  /**
   One pair per comparison row.
   */
  const slices = comparison.map(function toPair({
    sliceIndex,
    incumbentKind,
    incumbentText,
  },): ChunkPair {
    /**
     Where this slice's span starts.
     */
    const startOffset = cursor.at;
    cursor.at += incumbentText.length;
    return {
      source: {
        kind: 'content',
        sliceIndex,
        nodes: [],
        startOffset: 0,
        endOffset: 0,
        text: '',
      },
      target: (incumbentKind === 'absent')
        ? makeInsertionChunk({
          sliceIndex,
          offset: startOffset,
        },)
        : {
          kind: 'content',
          sliceIndex,
          nodes: [],
          startOffset,
          endOffset: cursor.at,
          text: incumbentText,
        },
    };
  },);
  return {
    slices,
    archiveText: comparison
      .map(function archiveOf({ incumbentText, },): string {
        return incumbentText;
      },)
      .join('',),
  };
}

/**
 Composes the page an artifact alone says would ship, by the composition the publisher runs.

 @param artifact - settled entry, read for what each slice would carry

 @param readGrammar - what the grammar says of a text, the strict MDX grammar the site compiles with by default

 @returns The composition: the step taken, the slices it left off and the artifact it weighs against

 @example
 ```ts
 const { step, slices, } = composeOverTiling({ artifact, },);
 ```
 */
export function composeOverTiling(
  {
    artifact,
    readGrammar,
  }: {
    readonly artifact: WouldShipSource;
    readonly readGrammar?: GrammarReader;
  },
): ComposedPage {
  /**
   Slices and archive the comparison rows tile.
   */
  const {
    slices,
    archiveText,
  } = tilingOf({ artifact, },);
  return composeSourcePage({
    artifact,
    slices,
    archiveText,
    entryId: 'the entry judged',
    l: SILENT,
    ...((readGrammar === undefined) ? {} : { readGrammar, }),
  },);
}

//endregion Source ships tiling
