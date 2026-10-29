import {
  type FootnoteGraphFinding,
  type FootnoteReferenceHit,
  normalizeFootnoteIdentifier,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
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

//region Footnote restoration
// A footnote reference a repair puts back is not a break, though inside one chunk it can look like one.
//
// The count is chunk-scoped, and a footnote's definition often lives in another chunk: run XingZ6022 slice 44's
// archive dropped `[^3]`, the repair put it back, and the definition of 3 sits in slice 115. Inside slice 44 the
// restored reference has no definition, so the count rose by one and the repair lost, though it had fixed real
// damage. The owner ruled on 2026-10-29 that a reference the slice's own source also carries counts as a
// restoration. Only an unresolved reference qualifies: an orphan or duplicated definition has no counterpart in
// the source's references, and a reference the source lacks is an invention whatever the page's definitions say.
//
// The reading is per KEY (convention plus folded label), not per mention. The signal counts classes of structural
// damage, so a patch that mentions a cited footnote more often than the source does is content fidelity for the
// checkers to judge. A key with no definition anywhere on the page is the page-level guard's to catch.

/**
 Key one footnote label is compared by: the syntax family and the label folded the way Markdown folds it.

 @param convention - syntax family of the reference

 @param identifier - label as the graph holds it

 @returns Key two references share exactly when they name one footnote in one syntax

 @example
 ```ts
 const key = footnoteReferenceKey({ convention: 'gfm', identifier: 'Note', },);
 ```
 */
function footnoteReferenceKey(
  {
    convention,
    identifier,
  }: {
    readonly convention: FootnoteReferenceHit['convention'];
    readonly identifier: string;
  },
): string {
  return `${convention}\u0000${normalizeFootnoteIdentifier({ identifier, },)}`;
}

/**
 Unresolved references of the patched chunk that restore one the slice's source carries and the translation lacked.

 Leave these out of the patched count when comparing it with the translation's: they are the repair undoing a
 drop, with the definition in another chunk. Every other finding stays counted.

 @param patchedDocument - parsed patched chunk

 @param targetDocument - parsed chunk before the patch

 @param sourceReferences - footnote references of the slice's own source, in the same syntax families

 @returns Findings of the patched chunk that count as restorations, in graph order

 @example
 ```ts
 const restored = restoredFootnoteReferences({ patchedDocument, targetDocument, sourceReferences, },);
 ```
 */
export function restoredFootnoteReferences(
  {
    patchedDocument,
    targetDocument,
    sourceReferences,
  }: {
    readonly patchedDocument: RepairDocument;
    readonly targetDocument: RepairDocument;
    readonly sourceReferences: readonly FootnoteReferenceHit[];
  },
): readonly FootnoteGraphFinding[] {
  /**
   Keys the slice's source cites.
   */
  const carriedBySource = new Set(sourceReferences.map(footnoteReferenceKey,),);

  /**
   Keys the translation cited before the patch, resolved or not.
   */
  const carriedByTarget = new Set(
    targetDocument.footnoteGraph
      .references
      .map(footnoteReferenceKey,),
  );

  return patchedDocument.footnoteGraph
    .findings
    .filter(function isRestoration(finding,): boolean {
      if (finding.kind !== 'unresolved-reference')
        return false;

      /**
       Key of the reference this finding reports.
       */
      const key = footnoteReferenceKey(finding,);
      return carriedBySource.has(key,) && (!carriedByTarget.has(key,));
    },);
}

//endregion Footnote restoration
