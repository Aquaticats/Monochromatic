/**
 Construction inputs for one exact-snapshot demand-driven effect index.
 
 @module
 */

import type { SourceFile, } from 'typescript/ast';
import type { Project, } from 'typescript/sync';

import type { EffectAnalysisBudget, } from './effect-analysis-budget.ts';
import type { EffectSummaryOmissionReason, } from './effect-cache-envelope.ts';
import type { ExternalOmissionRecord, } from './effect-external-omission.ts';
import type { EffectProjectFingerprint, } from './effect-project-fingerprint.ts';
import type { MutableEffectSummary, } from './effect-summary-model.ts';
import type { ExternalEffectIndexBuilder, } from './external-callable-effect.ts';

/**
 Fresh source scan awaiting complete dependency closure before persistence.
 */
export type PendingEffectSummaryStore = {
  readonly sourceFile: SourceFile;
  readonly summaries: ReadonlyMap<string, MutableEffectSummary>;
  readonly omittedCallableKeys: readonly string[];
  readonly omissionReasons: readonly EffectSummaryOmissionReason[];
  readonly externalOmissions: readonly ExternalOmissionRecord[];
};

/**
 Inputs needed to construct one exact-snapshot demand index.
 */
export type DemandDrivenEffectIndexOptions = {
  readonly project: Project;
  readonly indexedSourceFiles: ReadonlyMap<string, SourceFile>;
  readonly projectFingerprint: EffectProjectFingerprint;
  readonly scopeKey: string;
  readonly projectDigest: string;
  readonly cacheRootOverride?: string;
  readonly analysisRoot?: string;
  readonly buildIndex: ExternalEffectIndexBuilder;
  readonly analysisBudget: EffectAnalysisBudget;
};
