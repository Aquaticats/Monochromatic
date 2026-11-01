import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { RepairTranslationResult, } from '@monochromatic-dev/translation-repair-assembly/ts';
import { cappedFailureText, } from '@monochromatic-dev/translation-repair-slice-cache/ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Sentinel probe line
// What one PROBE line says about one probed entry, as text and nothing else.
//
// Split from the walk over the entries so every arm of the line (an entry
// with no accepted issue, one with several dispositions, a failure with a long
// message) is reached from values a case builds, with no corpus read and no
// model asked.

/**
 Most UTF-16 units of an error message's opening kept in a PROBE line, ending
 on a whole character (`wholeOpening`); a corpus read's kind and remedy follow
 it uncut (`cappedFailureText`).
 */
const ERROR_MESSAGE_CAP = 200;

/**
 The parts of a repair result a PROBE line reads, so a case builds only
 those.

 @example
 ```ts
 const result: ProbedResult = { status: 'unchanged', issues: [], findings: [], };
 ```
 */
export type ProbedResult = Pick<RepairTranslationResult, 'status' | 'findings'> & {
  /**
   Every adjudicated issue, of which a line reads the status, the repair
   disposition and whether the naturalness lane refined it.
   */
  readonly issues: readonly {
    readonly issue: { readonly status: string; };
    readonly repairDisposition: string;
    readonly refined: boolean;
  }[];
};

/**
 Counts accepted issues per repair disposition, printed sorted so two probe
 lines compare directly.

 @param accepted - accepted issues of one result

 @returns `name:count` pairs joined by commas, `none` where nothing was accepted

 @example
 ```ts
 dispositionCounts({ accepted: [], },); // 'none'
 ```
 */
function dispositionCounts(
  { accepted, }: { readonly accepted: ProbedResult['issues']; },
): string {
  /**
   Accepted issues counted per repair disposition; a map, as every record
   filled by a key is (ledger B77).
   */
  const counts = new Map<string, number>();
  for (const record of accepted) {
    counts.set(
      record.repairDisposition,
      (counts.get(record.repairDisposition,) ?? 0) + 1,
    );
  }

  if (counts.size === 0)
    return 'none';

  // Those counts as pairs, so a probe shows whether repair provenance is
  // actually recorded rather than only whether issues were found.
  return [...counts,]
    .toSorted(function byName(
      left: ReadonlyEntry<string, number>,
      right: ReadonlyEntry<string, number>,
    ): number {
      return compareCodePoints({
        left: left[0],
        right: right[0],
      },);
    },)
    .map(function toPair(entry: ReadonlyEntry<string, number>,): string {
      return `${entry[0]}:${String(entry[1],)}`;
    },)
    .join(',',);
}

/**
 The PROBE line of an entry whose repair ran to a result.

 @param id - corpus entry probed

 @param result - what the repair returned

 @param elapsedMs - duration of this entry's probe

 @returns One line, without a trailing newline

 @example
 ```ts
 console.log(probeResultLine({ id: 'mittens', result, elapsedMs: 12, },),);
 ```
 */
export function probeResultLine(
  {
    id,
    result,
    elapsedMs,
  }: {
    readonly id: string;
    readonly result: ProbedResult;
    readonly elapsedMs: number;
  },
): string {
  /**
   Accepted issues among all adjudicated.
   */
  const accepted = result.issues
    .filter(function isAccepted(record,): boolean {
      return record.issue
        .status
        === 'accepted';
    },);

  /**
   Issues the naturalness lane refined.
   */
  const refined = result.issues
    .filter(function wasRefined(record,): boolean {
      return record.refined;
    },);
  return `PROBE ${id} status=${result.status} issues=${String(result.issues
    .length,)} accepted=${String(accepted.length,)} repairs=${
    dispositionCounts({ accepted, },)
  } refinedIssues=${String(refined.length,)} findings=${String(result.findings
    .length,)} ms=${String(elapsedMs,)}`;
}

/**
 The PROBE line of an entry whose probe failed.

 @param id - corpus entry probed

 @param error - what was thrown, of unknown type by construction

 @param elapsedMs - duration of this entry's probe

 @returns One line: a marked class in its own words, anything else by name
 only, capped at {@link ERROR_MESSAGE_CAP} units on a whole character, a
 corpus read's kind and remedy kept whole after the cut

 @example
 ```ts
 console.log(probeErrorLine({ id: 'mittens', error, elapsedMs: 12, },),);
 ```
 */
export function probeErrorLine(
  {
    id,
    error,
    elapsedMs,
  }: {
    readonly id: string;
    readonly error: unknown;
    readonly elapsedMs: number;
  },
): string {
  /**
   Failure text for the PROBE line.
   */
  const message = cappedFailureText({
    error,
    units: ERROR_MESSAGE_CAP,
  },);
  return `PROBE ${id} status=ERROR ms=${String(elapsedMs,)} error=${message}`;
}

//endregion Sentinel probe line
