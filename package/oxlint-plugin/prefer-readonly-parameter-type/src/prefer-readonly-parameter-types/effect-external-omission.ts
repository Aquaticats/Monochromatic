/**
 Omissions met inside installed package implementations, carried to the sources that depend on them.

 A source whose scan applied the effect of an installed package's shipped implementation keeps that
 effect in its cached summaries. A restore of the source therefore never opens the implementation, so
 the omission the implementation's own entry records is neither restored nor reported unless the
 depending source records it too.

 @module
 */

import {
  areOmissionReasons,
  type EffectSummaryOmissionReason,
} from './effect-omission-reason.ts';
import {
  isCacheString,
  isIndex,
  isRecord,
  MAX_CALLABLE_ARITY,
} from './effect-summary-cache-primitive.ts';

/**
 One installed-package source with callables left out of the effect index.
 */
export type ExternalOmissionRecord = {
  readonly sourceFileName: string;
  readonly omittedCount: number;
  readonly reasons: readonly EffectSummaryOmissionReason[];
};

/**
 Sentinel returned when a parsed record list cannot be proved valid.
 */
export const EXTERNAL_OMISSIONS_INVALID: unique symbol = Symbol(
  'external omission records failed validation',
);

/**
 Parses one value into a bounded record of an omitted source.
 
 @param value - Parsed record candidate.
 
 @returns the record when it names a source,
 a positive bounded count,
 and a nonempty set of known reason categories,
 or the invalid sentinel.
 
 @example
 ```ts
 parseExternalOmissionRecord({ sourceFileName: 'a.js', omittedCount: 1, reasons: ['direct-summary-construction-failed'] });
 ```
 */
function parseExternalOmissionRecord(
  value: unknown,
): ExternalOmissionRecord | typeof EXTERNAL_OMISSIONS_INVALID {
  if (!isRecord(value,))
    return EXTERNAL_OMISSIONS_INVALID;
  /**
   Candidate fields read once so each is narrowed before use.
   */
  const {
    sourceFileName,
    omittedCount,
    reasons,
  } = value;
  if ((!isCacheString(sourceFileName,))
    || ((typeof omittedCount) !== 'number')
    || (omittedCount === 0)
    || (!isIndex({
      value: omittedCount,
      upperBound: MAX_CALLABLE_ARITY,
    },))
    || (!Array.isArray(reasons,))
    || (reasons.length === 0)
    || (new Set(reasons,).size !== reasons.length)
    || (!areOmissionReasons(reasons,)))
    return EXTERNAL_OMISSIONS_INVALID;
  return {
    sourceFileName,
    omittedCount,
    reasons,
  };
}

/**
 Parses a list into bounded unique records of sources other than the entry's own.
 
 @param value - Parsed list candidate.
 
 @param fileName - Source the entry belongs to,
 whose own omissions are persisted as callable identities instead.
 
 @returns the records when every one is valid,
 names a distinct source,
 and none names the entry's own source,
 or the invalid sentinel.
 
 @example
 ```ts
 parseExternalOmissionRecords({ value: [], fileName: 'consumer.ts' });
 ```
 */
export function parseExternalOmissionRecords({
  value,
  fileName,
}: {
  readonly value: unknown;
  readonly fileName: string;
},): readonly ExternalOmissionRecord[] | typeof EXTERNAL_OMISSIONS_INVALID {
  if ((!Array.isArray(value,)) || (value.length > MAX_CALLABLE_ARITY))
    return EXTERNAL_OMISSIONS_INVALID;
  /**
   Records parsed so far.
   */
  const records: ExternalOmissionRecord[] = [];
  /**
   Source names already claimed by a parsed record.
   */
  const names = new Set<string>([fileName,],);
  for (const candidate of value) {
    /**
     Parsed candidate or the invalid sentinel.
     */
    const record = parseExternalOmissionRecord(candidate,);
    if ((record === EXTERNAL_OMISSIONS_INVALID) || names.has(record.sourceFileName,))
      return EXTERNAL_OMISSIONS_INVALID;
    names.add(record.sourceFileName,);
    records.push(record,);
  }
  return records;
}

/**
 Merges groups of records into one deterministic list with one record per source.

 Two records for one source come from one stored entry and agree,
 so the merge keeps the larger count and the union of reasons rather than choosing between them.

 @param groups - Record lists gathered from several external calls or reached sources.

 @returns records sorted by source name.

 @example
 ```ts
 mergeExternalOmissions([[record], [record]]);
 ```
 */
export function mergeExternalOmissions(
  groups: readonly (readonly ExternalOmissionRecord[])[],
): readonly ExternalOmissionRecord[] {
  /**
   Merged record per source name.
   */
  const bySource = new Map<string, ExternalOmissionRecord>();
  groups.forEach(function mergeGroup(group,): void {
    group.forEach(function mergeRecord(record,): void {
      /**
       Record already held for the same source.
       */
      const held = bySource.get(record.sourceFileName,);
      bySource.set(
        record.sourceFileName,
        held === undefined
          ? record
          : {
            sourceFileName: record.sourceFileName,
            omittedCount: Math.max(
              held.omittedCount,
              record.omittedCount,
            ),
            reasons: [...new Set([
              ...held.reasons,
              ...record.reasons,
            ],),].toSorted(),
          },
      );
    },);
  },);
  return [...bySource.values(),]
    .toSorted(function bySourceName(
      left,
      right,
    ): number {
      return left.sourceFileName < right.sourceFileName ? -1 : 1;
    },);
}
