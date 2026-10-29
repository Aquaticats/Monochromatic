/**
 Direct effect-summary omission recording and visible lifecycle logging.
 
 @module
 */

import { caughtValueStack, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  DIRECT_SUMMARY_FAILURE_REASON,
  type EffectSummaryOmissionReason,
} from './effect-cache-envelope.ts';
import type { ExternalOmissionRecord, } from './effect-external-omission.ts';

/**
 Omission lifecycle logger.
 */
const l = tagged({ tag: 'effect-summary-omission', },);

/**
 Restored external omissions already reported by this process,
 keyed by source,
 count and reasons.

 A fresh scan reports an installed package's omission once per process because the package's index is
 built once. Every source depending on that package restores the same record, so without this set a
 warm run would repeat the line once per dependent source.
 */
const reportedExternalOmissions = new Set<string>();

/**
 Records one fresh direct-summary omission without exposing stack noise at warning level.
 
 @param allOmittedKeys - Build-wide identities accepted by completeness checks.
 
 @param sourceOmittedKeys - Source-local identities persisted with direct summaries.
 
 @param sourceOmissionReasons - Source-local bounded failure categories.
 
 @param key - Stable callable identity whose summary failed.
 
 @param error - Caught summary construction failure retained at debug level.
 
 @mutates allOmittedKeys - Adds omitted callable identity.
 
 @mutates sourceOmittedKeys - Adds omitted callable identity.
 
 @mutates sourceOmissionReasons - Adds bounded caught failure category.
 
 @example
 ```ts
 recordDirectSummaryOmission({
   allOmittedKeys,
   sourceOmittedKeys,
   sourceOmissionReasons,
   key,
   error,
 });
 ```
 */
export function recordDirectSummaryOmission({
  allOmittedKeys,
  sourceOmittedKeys,
  sourceOmissionReasons,
  key,
  error,
}: {
  readonly allOmittedKeys: Set<string>;
  readonly sourceOmittedKeys: Set<string>;
  readonly sourceOmissionReasons: Set<EffectSummaryOmissionReason>;
  readonly key: string;
  readonly error: unknown;
}): void {
  /**
   Complete caught detail retained for the debug log.
   */
  const detail = caughtValueStack(error,);
  allOmittedKeys.add(key,);
  sourceOmittedKeys.add(key,);
  sourceOmissionReasons.add(DIRECT_SUMMARY_FAILURE_REASON,);
  l.debug(`omitting ${key} from the effect index: ${detail}`,);
}

/**
 Emits one concise warning for fresh source-local omissions.
 
 @param omittedKeys - Source-local identities omitted during scan.
 
 @param sourceFileName - Exact source whose scan was incomplete.
 
 @param reasons - Bounded failure categories encountered during scan.
 
 @example
 ```ts
 reportDirectSummaryOmissions({ omittedKeys, sourceFileName, reasons });
 ```
 */
export function reportDirectSummaryOmissions({
  omittedKeys,
  sourceFileName,
  reasons,
}: {
  readonly omittedKeys: ReadonlySet<string>;
  readonly sourceFileName: string;
  readonly reasons: ReadonlySet<EffectSummaryOmissionReason>;
}): void {
  if (omittedKeys.size === 0)
    return;
  /**
   Deterministic comma-separated reason categories for one-line warning.
   */
  const renderedReasons = [...reasons,]
    .toSorted()
    .join(',',);
  l.warn(
    `omitted ${String(omittedKeys.size,)} callable summaries for ${sourceFileName}: ${renderedReasons}; debug logging contains causes`,
  );
}

/**
 Restores cached omission identities and emits one concise warning.
 
 @param allOmittedKeys - Build-wide identities accepted by completeness checks.
 
 @param restoredKeys - Validated source-local identities restored from cache.
 
 @param sourceFileName - Exact source whose cached scan was incomplete.
 
 @param reasons - Validated bounded reason categories restored from cache.
 
 @mutates allOmittedKeys - Adds every restored callable identity.
 
 @example
 ```ts
 restoreCachedSummaryOmissions({
   allOmittedKeys,
   restoredKeys,
   sourceFileName,
   reasons,
 });
 ```
 */
export function restoreCachedSummaryOmissions({
  allOmittedKeys,
  restoredKeys,
  sourceFileName,
  reasons,
}: {
  readonly allOmittedKeys: Set<string>;
  readonly restoredKeys: readonly string[];
  readonly sourceFileName: string;
  readonly reasons: readonly EffectSummaryOmissionReason[];
}): void {
  restoredKeys.forEach(function restoreKey(key,): void {
    allOmittedKeys.add(key,);
  },);
  if (restoredKeys.length === 0)
    return;
  l.warn(
    `restored ${String(restoredKeys.length,)} omitted callable summaries for ${sourceFileName} from effect cache: ${reasons.join(',')}`,
  );
}

/**
 Forgets which restored external omissions this process has reported.
 
 @example
 ```ts
 clearReportedExternalOmissions();
 ```
 */
export function clearReportedExternalOmissions(): void {
  reportedExternalOmissions.clear();
}

/**
 Reports omissions recorded from installed package implementations that a restored source depends on.
 
 A restored source never opens those implementations, so the report a fresh scan made when it did is
 made here from the record the source persisted.
 
 @param records - Validated records restored with the source entry.
 
 @example
 ```ts
 restoreExternalOmissions(hit.externalOmissions);
 ```
 */
export function restoreExternalOmissions(records: readonly ExternalOmissionRecord[],): void {
  records.forEach(function restoreRecord(record,): void {
    /**
     Deterministic comma-separated reason categories for one-line warning.
     */
    const renderedReasons = record.reasons
      .join(',',);
    /**
     Identity of what this record would report.
     */
    const reportKey = `${record.sourceFileName}\0${String(record.omittedCount,)}\0${renderedReasons}`;
    if (reportedExternalOmissions.has(reportKey,))
      return;
    reportedExternalOmissions.add(reportKey,);
    l.warn(
      `restored ${String(record.omittedCount,)} omitted callable summaries for ${record.sourceFileName} from effect cache: ${renderedReasons}`,
    );
  },);
}
