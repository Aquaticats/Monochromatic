import type { RepairDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Downgrade count
// The `mdx-downgraded` integrity signal, split from repair-chunk.ts to keep
// that orchestration file under its line budget.

/**
 Count of `mdx-downgraded` findings, the integrity signal:
 a patch that forces markdown fallback broke document grammar.

 @param document - parsed document under inspection

 @returns Downgrade finding count

 @example
 ```ts
 downgradeCount({ document, },);
 ```
 */
export function downgradeCount(
  { document, }: { readonly document: RepairDocument; },
): number {
  return document
    .parseFindings
    .filter(function isDowngrade(finding,) {
      return finding.kind === 'mdx-downgraded';
    },)
    .length;
}

//endregion Downgrade count
