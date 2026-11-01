import type { RepairDocument, } from './parse-document.ts';

//region Footnote break count
// The footnote integrity signal, kept beside `downgradeCount` because the two
// answer the same question about different damage.
//
// A repair that breaks a footnote leaves the grammar perfectly valid, so the
// downgrade signal cannot see it. Measured over 56 settled entries, four
// shipped with footnote damage the pipeline had already detected and never
// consulted: a definition invented, a definition duplicated, a reference and a
// definition dropped from different footnotes, and a reference invented in a
// document that carried no footnotes at all.
//
// Counted rather than compared in detail, because the gate asks only whether a
// patch made things WORSE. An input translation is free to arrive with dangling
// references, and one does; that is a defect to repair, not a reason to refuse
// every repair of the chunk holding it.

/**
 Count of footnote integrity findings a document carries.

 Every kind counts alike: an unresolved reference, an orphaned definition and
 a duplicated definition are all damage when a patch introduces one.

 @param document - parsed document under inspection

 @returns Footnote finding count

 @example
 ```ts
 footnoteBreakCount({ document, },);
 ```
 */
export function footnoteBreakCount(
  { document, }: { readonly document: RepairDocument; },
): number {
  /**
   Graph the parse already built for this document.
   */
  const { footnoteGraph, } = document;

  /**
   Integrity findings it carries.
   */
  const { findings, } = footnoteGraph;

  return findings.length;
}

//endregion Footnote break count
