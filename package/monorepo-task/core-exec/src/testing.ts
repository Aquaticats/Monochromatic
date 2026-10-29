/**
 Built testing seam for task utility implementation helpers.
 
 This sub-entry exists only so package tests exercise bundled code. Every
 re-exported declaration is internal and carries no compatibility promise.
 
 @module
 */

export {
  firstGlobMetaIndex,
} from './depends-resolve-glob.ts';
export {
  augmentOxlintOutput,
  extractRuleName,
  formatGuidanceLine,
  isHelpLine,
  NO_RULE,
  stripAnsi,
} from './oxlint-augment.ts';
export {
  fixUntilStable,
  MAX_AUTOFIX_PASSES,
  normalizeForConvergence,
  type OxlintRunResult,
} from './oxlint-fix-loop.ts';
export { RULE_GUIDANCE, } from './oxlint-guidance.ts';
export {
  filterPnpmOutput,
  isAllowedCycleWarning,
} from './pnpm-output-filter.ts';
export { buildTscArgs, } from './tsc-args.ts';
export {
  filterTscOutput,
  isContinuationLine,
  isDiagnosticLine,
  isNodeModulesDiagnostic,
} from './tsc-output-filter.ts';
