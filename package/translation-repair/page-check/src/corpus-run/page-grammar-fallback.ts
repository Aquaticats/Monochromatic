import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  type SliceReplacement,
  spliceSlices,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type PageGrammarReading,
  readPageGrammar,
} from './page-grammar.ts';
import type { PublishDefect, } from './publish-defects.ts';

//region Page grammar fallback
// A PAGE THE GRAMMAR REFUSES STILL SHIPS (B1207). `publishFixedPage` used to end in `assertPageParses`, which threw after
// the defects were logged and before the page was written, and `entryErrorOutcome` files that error as a stop: an entry
// whose every slice had settled, after hours of work, ended with no page and no artifact. The owner's rules are "no
// matter what, we must ship" and "No valid wording: keep the archive at that slice and ship" (2026-09-27).
//
// THE ARCHIVE IS KEPT AT THE SLICES THAT BREAK THE PAGE, and no more of them than that. Every slice is read under the
// strict MDX grammar before it is admitted, so a page that fails as a whole fails through the way slices stand beside one
// another (a rewrite of the would-ship reading, an original, a quotation that crosses a slice). Slices go back to the
// archive's text one by one, in order, until the page parses; each one that was not needed is then put back. If even
// the archive's page does not parse, the page ships as composed with the defect `page-unparseable`, since a missing
// page costs the entry more than a page the site cannot build, which the defect line says.

/**
 What the page came to once the grammar had read it.
 */
export type ParsingPage = {
  /**
   The page to write.
   */
  readonly pageText: string;

  /**
   The replacements the page was spliced from, without those the archive was kept for.
   */
  readonly replacements: readonly SliceReplacement[];

  /**
   Slices the archive was kept at, none for a page that parsed as composed.
   */
  readonly keptArchiveAt: readonly number[];

  /**
   The defects the fallback reports, none for a page that parsed as composed.
   */
  readonly defects: readonly PublishDefect[];
};

/**
 Reads a composed page under the strict MDX grammar and, where it is refused, keeps the archive at the slices that break it.

 @param entryId - entry whose page is read

 @param pageText - page as composed

 @param replacements - replacements the page was spliced from

 @param slices - preparation defining the replacement spans

 @param archiveText - archive the page was spliced into

 @param l - logger the grammar's refusal is written to, which a defect's message does not hold

 @param readGrammar - grammar's reading of a page, the package's by default

 @returns Page to write, the replacements it holds and the defects to report

 @throws Whatever `spliceSlices` raises for replacements the composed page was itself spliced from, which none does

 @example
 ```ts
 const parsing = pageThatParses({ entryId, pageText, replacements, slices, archiveText, l, },);
 ```
 */
export function pageThatParses(
  {
    entryId,
    pageText,
    replacements,
    slices,
    archiveText,
    l,
    readGrammar = readPageGrammar,
  }: {
    readonly entryId: string;
    readonly pageText: string;
    readonly replacements: readonly SliceReplacement[];
    readonly slices: readonly ChunkPair[];
    readonly archiveText: string;
    readonly l: Logger;
    readonly readGrammar?: (reading: { readonly pageText: string; },) => PageGrammarReading;
  },
): ParsingPage {
  /**
   What the grammar says of the page as composed.
   */
  const composed = readGrammar({ pageText, },);
  if (composed.kind === 'parses') {
    return {
      pageText,
      replacements,
      keptArchiveAt: [],
      defects: [],
    };
  }

  /**
   Page with the archive kept at the given slices.

   @param kept - slices whose replacement is left out, so the archive's text stands there

   @returns Page and the replacements it holds

   @example
   ```ts
   const page = pageKeepingArchiveAt([1, 4,],);
   ```
   */
  function pageKeepingArchiveAt(
    kept: readonly number[],
  ): Pick<ParsingPage, 'pageText' | 'replacements'> {
    /**
     Replacements that stay.
     */
    const staying = replacements.filter(function stays({ sliceIndex, },): boolean {
      return !kept.includes(sliceIndex,);
    },);
    return {
      pageText: spliceSlices({
        targetText: archiveText,
        slices,
        replacements: staying,
      },),
      replacements: staying,
    };
  }

  /**
   Whether the page keeping the archive at the given slices parses.

   @param kept - slices whose replacement is left out

   @returns True when the grammar accepts that page

   @example
   ```ts
   const parses = parsesKeepingArchiveAt([1,],);
   ```
   */
  function parsesKeepingArchiveAt(kept: readonly number[],): boolean {
    /**
     The page keeping the archive at those slices.
     */
    const page = pageKeepingArchiveAt(kept,);
    return readGrammar({ pageText: page.pageText, },)
      .kind
      === 'parses';
  }

  /**
   Puts slices back to the archive, in order, until the page parses.

   @returns Slices put back, none where the page did not parse even with the archive at every slice

   @example
   ```ts
   const putBack = slicesPutBackUntilTheyParse();
   ```
   */
  function slicesPutBackUntilTheyParse(): readonly number[] {
    /**
     Slices put back so far.
     */
    const putBack: number[] = [];
    for (const { sliceIndex, } of replacements) {
      putBack.push(sliceIndex,);
      if (parsesKeepingArchiveAt(putBack,))
        return putBack;
    }
    return [];
  }

  l.warn(`entry ${entryId}: the page does not parse (${composed.refusal}); keeping the archive at the slices that break it`,);

  /**
   Slices put back to the archive, in order, until the page parses; none means the archive's page was refused too.
   */
  const reverted = slicesPutBackUntilTheyParse();
  if (reverted.length === 0) {
    l.warn(`entry ${entryId}: the archive's own page does not parse either; shipping the page as composed`,);
    return {
      pageText,
      replacements,
      keptArchiveAt: [],
      defects: [{
        check: 'page-unparseable',
        message: `entry ${entryId} ships a page the MDX grammar refuses to parse, and the archive kept at every slice `
          + 'is refused too, so the site build will not take this page',
      },],
    };
  }

  /**
   The slices the page needs the archive at: those put back that the page still does not parse without.
   */
  const needed = reverted.reduce(
    function keepNeeded(
      kept: readonly number[],
      sliceIndex,
    ): readonly number[] {
      /**
       The slices put back, this one given its replacement again.
       */
      const without = kept.filter(function other(candidate,): boolean {
        return candidate !== sliceIndex;
      },);
      return parsesKeepingArchiveAt(without,) ? without : kept;
    },
    reverted,
  );
  return {
    ...pageKeepingArchiveAt(needed,),
    keptArchiveAt: needed,
    defects: [{
      check: 'page-grammar-archive-kept',
      message: `entry ${entryId} kept the archive at ${
        wordForCount({
          count: needed.length,
          one: 'slice',
          many: 'slices',
        },)
      } ${needed.join(', ',)}, since the page did not parse with ${
        (needed.length === 1) ? 'its' : 'their'
      } wording`,
    },],
  };
}

//endregion Page grammar fallback
