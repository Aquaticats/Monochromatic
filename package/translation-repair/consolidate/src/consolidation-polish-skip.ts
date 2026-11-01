import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { ConsolidationPolish, } from './consolidation-polish-model.ts';

//region Unpolished baseline
// ONE ANSWER FOR EVERY BASE NO POLISH MAY RUN OVER, shared by
// `polishConsolidation` and by a settlement that kept the archive because no
// wording passed the deterministic rule (owner, 2026-09-27, "Keep archive,
// ship"). `assertFinalNaturalnessComplete` accepts a front-matter slice only
// under the front-matter reason and a body slice only under the unsafe-baseline
// one, so the two callers must never drift apart on which is which.
//
// NOT REACHED THROUGH `applyFinalPolish` by the archive-kept settlement,
// deliberately: that function first checks the settlement's text for dropped
// contributor names and stops the entry on a drop, and an archive-kept
// settlement's text is the refused standing, which never ships.

/**
 Records why no polish ran over a base it may not touch.

 @param syntax - explicit syntax role, absent for ordinary prose

 @returns Not-run record naming front matter, or an unadmitted baseline

 @example
 ```ts
 const polish = unpolishedBaseline({ syntax: 'front-matter', },);
 ```
 */
export function unpolishedBaseline(
  { syntax, }: { readonly syntax?: SliceSyntax; },
): ConsolidationPolish {
  return {
    kind: 'not-run',
    reason: (syntax === 'front-matter') ? 'front-matter' : 'unsafe-baseline',
  };
}

//endregion Unpolished baseline
