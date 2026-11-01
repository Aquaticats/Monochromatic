/**
 Semantic effect cache lifecycle coordination.
 
 @module
 */

import { clearEffectSummaryCache, } from './effect-summary-cache.ts';
import { clearReportedExternalOmissions, } from './effect-summary-omission.ts';
import { clearFinalEffectIndexCache, } from './effect-final-index-cache.ts';
import { clearExternalCallableEffectCache, } from './external-callable-effect.ts';
import { closeExternalImplementationProjects, } from './external-implementation-project.ts';
import { clearLockfilePackageEligibilityCache, } from './lockfile-package-eligibility.ts';

/**
 Closes external projects and clears all process-local effect caches.
 
 @example
 ```ts
 resetSemanticEffectCaches();
 ```
 */
export function resetSemanticEffectCaches(): void {
  closeExternalImplementationProjects();
  clearExternalCallableEffectCache();
  clearLockfilePackageEligibilityCache();
  clearEffectSummaryCache();
  clearFinalEffectIndexCache();
  clearReportedExternalOmissions();
}
