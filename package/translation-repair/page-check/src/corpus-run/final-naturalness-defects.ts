import { NaturalnessCompletenessError, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { SettledArtifact, } from '@monochromatic-dev/translation-repair-artifact/ts';
import { assertFinalNaturalnessComplete, } from './final-naturalness-completeness.ts';
import {
  type PublishDefect,
  publishDefects,
} from './publish-defects.ts';

//region Final naturalness defects
// A BODY WITHOUT ABSOLUTE NATURALNESS APPROVAL SHIPS WITH THE SHORTFALL REPORTED (B1207). `assertFinalNaturalnessComplete`
// threw before the page was written, and `entryErrorOutcome` files its error as a stop: an entry whose every slice had
// settled ended with no page and no artifact because a review was missing or did not meet the floor. The owner's rules are
// "no matter what, we must ship" and that a check of the page ships with the defect reported, so the refusal is collected
// as a defect, like every other content check of `publish-defects.ts`. Anything else the guard raises (an artifact that
// records its contest or consolidation as never run, a review the parser cannot read) is a fault in the code and still
// stops the entry.

/**
 Reports the shortfall of a settled artifact's naturalness review as a defect instead of a stop.

 @param artifact - settled artifact, before the page or the artifact is persisted

 @returns Defect naming the first slice that lacks approval, or none

 @throws Whatever `assertFinalNaturalnessComplete` raises other than its own refusal

 @example
 ```ts
 const defects = finalNaturalnessDefects({ artifact, },);
 ```
 */
export function finalNaturalnessDefects(
  { artifact, }: { readonly artifact: Pick<SettledArtifact, 'consolidation' | 'laneSelection'>; },
): readonly PublishDefect[] {
  return publishDefects({
    steps: [{
      check: 'final-naturalness',
      refusal: NaturalnessCompletenessError,
      run: function naturalnessComplete(): void {
        assertFinalNaturalnessComplete({ artifact, },);
      },
    },],
  },);
}

//endregion Final naturalness defects
