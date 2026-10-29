/**
 Synthetic.new quota response header parsing.
 
 The `x-synthetic-quotas` header shape mirrors `@aliou/pi-synthetic`.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  PERCENT_BASE,
  SECONDS_PER_DAY,
  SECONDS_PER_HOUR,
  SECONDS_PER_WEEK,
  type RateLimitSnapshot,
} from './rate-limit-types.ts';
import {
  INVALID_RATE_LIMIT_SNAPSHOT,
  INVALID_VALUE,
  clampNumber,
  createRateLimitSnapshot,
  isRateLimitSnapshot,
  isUnknownRecord,
  readNumberProperty,
  readRecordProperty,
  readResetProperty,
  usedPercentFromLimit,
  type InvalidRateLimitSnapshot,
  type InvalidValue,
  type UnknownRecord,
} from './rate-limit-parse-helpers.ts';

/**
 Logger root for Synthetic quota header parsing.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: syntheticQuotaHeadersLogger, },);
 ```
 */
const syntheticQuotaHeadersLogger = tagged({ tag: 'pi-statusline:synthetic-quota-headers', },);

/**
 Synthetic quotas response header name.
 */
const SYNTHETIC_QUOTAS_HEADER = 'x-synthetic-quotas';

/**
 Pace multiplier used by Synthetic's weekly token quota.
 
 The Synthetic extension source models weekly credits as daily regeneration
 normalized across seven days.
 */
const SYNTHETIC_WEEKLY_PACE_SCALE = SECONDS_PER_DAY / SECONDS_PER_WEEK;

/**
 Parses Synthetic quotas header as an object.
 
 @param headers - lowercase provider response headers
 
 @returns {@link UnknownRecord}, or invalid sentinel when missing or invalid
 
 @example
 ```ts
 parseSyntheticQuotasHeader({ 'x-synthetic-quotas': '{}' });
 ```
 */
function parseSyntheticQuotasHeader(
  headers: Readonly<Record<string, string>>,
): UnknownRecord | InvalidValue {
  /**
   Function-scoped logger tagged by function name.
   */
  const log = tagged({
    tag: parseSyntheticQuotasHeader.name,
    l: syntheticQuotaHeadersLogger,
  },);
  /**
   Raw quotas header JSON string.
   */
  const rawValue = headers[SYNTHETIC_QUOTAS_HEADER];
  if (rawValue === undefined)
    return INVALID_VALUE;

  try {
    /**
     Parsed quotas JSON value.
     */
    const parsedValue: unknown = JSON.parse(rawValue,);
    if (!isUnknownRecord(parsedValue,))
      return INVALID_VALUE;

    return parsedValue;
  } catch (error: unknown) {
    /**
     Rendered caught value; malformed quotas JSON is treated as a missing header.
     */
    const detail = caughtValueText(error,);
    log.debug(`ignoring invalid quotas header JSON: ${detail}`,);
    return INVALID_VALUE;
  }
}

/**
 Weekly block key names Synthetic has delivered, in the order they were first seen.

 The `GET /v2/quotas` body carried `weeklyTokenLimit` at 19:32:44 UTC on 2026-10-22 and `planWeeklyLimit` at
 03:28:16 UTC on 2026-10-23; the rename happened after 22:20:53 UTC and before 22:21:58 UTC on 2026-10-22, as the
 sibling reader in `package/translation-repair/provider-stream/src/synthetic-quota.ts` records. The
 `x-synthetic-quotas` header this module reads carried `planWeeklyLimit` at 05:06:33 UTC on 2026-10-23, with about 2%
 of the weekly credit left, so the rename is not tied to a spent credit. Whether it is permanent is not known, so both
 names are read.
 */
const SYNTHETIC_WEEKLY_BLOCK_NAMES = [
  'weeklyTokenLimit',
  'planWeeklyLimit',
] as const;

/**
 Weekly credit fields this extension projects from one weekly block.
 */
type SyntheticWeeklyReading = Readonly<{
  percentRemaining: number;
  resetAtMs: number;
}>;

/**
 Reads the projected fields of one weekly block.
 
 @param block - weekly block object under either name
 
 @returns {@link SyntheticWeeklyReading}, or invalid sentinel when a field is missing or mistyped
 
 @example
 ```ts
 readSyntheticWeeklyBlock({ percentRemaining: 80, nextRegenAt: '2026-10-23T05:03:47.000Z' });
 ```
 */
function readSyntheticWeeklyBlock(block: UnknownRecord,): SyntheticWeeklyReading | InvalidValue {
  /**
   Remaining weekly credits as percentage.
   */
  const percentRemaining = readNumberProperty({
    record: block,
    key: 'percentRemaining',
  },);
  /**
   Next daily regeneration timestamp.
   */
  const resetAtMs = readResetProperty({
    record: block,
    key: 'nextRegenAt',
  },);
  if ((percentRemaining === INVALID_VALUE) || (resetAtMs === INVALID_VALUE))
    return INVALID_VALUE;

  return {
    percentRemaining,
    resetAtMs,
  };
}

/**
 Reads the weekly reading from whichever key a quotas object carries it under.
 
 A key that is missing or holds `null` counts as absent. Both keys present and reading the same is accepted; both
 present and reading differently is refused, since which one to believe is then unknowable. Any other value that is no
 readable block is refused even beside a readable other, and neither key present is refused.
 
 @param quotas - Synthetic quotas response object
 
 @returns {@link SyntheticWeeklyReading}, or invalid sentinel when refused
 
 @example
 ```ts
 readSyntheticWeeklyReading({ planWeeklyLimit: { percentRemaining: 0, nextRegenAt: '2026-10-23T05:03:47.000Z' } });
 ```
 */
function readSyntheticWeeklyReading(quotas: UnknownRecord,): SyntheticWeeklyReading | InvalidValue {
  /**
   Function-scoped logger tagged by function name.
   */
  const log = tagged({
    tag: readSyntheticWeeklyReading.name,
    l: syntheticQuotaHeadersLogger,
  },);
  /**
   Weekly block names this quotas object carries.
   */
  const present = SYNTHETIC_WEEKLY_BLOCK_NAMES.filter(function carried(name,): boolean {
    return (quotas[name] !== undefined) && (quotas[name] !== null);
  },);
  if (present.length === 0) {
    log.debug(`refusing quotas without ${SYNTHETIC_WEEKLY_BLOCK_NAMES.join(' or ',)} as a non-null value`,);
    return INVALID_VALUE;
  }

  /**
   One reading per carried block, with the invalid sentinel for an unreadable block.
   */
  const readings = present.map(function readCarried(name,): SyntheticWeeklyReading | InvalidValue {
    /**
     Block under this name as delivered.
     */
    const block = readRecordProperty({
      record: quotas,
      key: name,
    },);
    if (block === INVALID_VALUE) {
      log.debug(`refusing quotas whose ${name} is not an object`,);
      return INVALID_VALUE;
    }

    /**
     Reading of this block, or the invalid sentinel when one of its fields is unreadable.
     */
    const reading = readSyntheticWeeklyBlock(block,);
    if (reading === INVALID_VALUE)
      log.debug(`refusing quotas whose ${name} has an unreadable percentRemaining or nextRegenAt`,);

    return reading;
  },);
  /**
   First carried block, and the second where the quotas object carries both names.
   */
  const [first, second,] = readings;
  if ((first === undefined) || (first === INVALID_VALUE))
    return INVALID_VALUE;

  if (second === undefined)
    return first;

  if (second === INVALID_VALUE)
    return INVALID_VALUE;

  if ((first.percentRemaining !== second.percentRemaining) || (first.resetAtMs !== second.resetAtMs)) {
    log.debug(`refusing quotas whose ${present.join(' and ',)} are both present and read differently`,);
    return INVALID_VALUE;
  }

  return first;
}

/**
 Parses Synthetic weekly credit quota into a projectable snapshot.
 
 @param quotas - Synthetic quotas response object
 
 @param nowMs - wall-clock sample time in epoch milliseconds
 
 @returns parsed weekly {@link RateLimitSnapshot} or invalid sentinel
 
 @example
 ```ts
 parseSyntheticWeeklySnapshot({ quotas, nowMs: Date.now() });
 ```
 */
function parseSyntheticWeeklySnapshot({
  quotas,
  nowMs,
}: Readonly<{
  quotas: UnknownRecord;
  nowMs: number;
}>,): RateLimitSnapshot | InvalidRateLimitSnapshot {
  /**
   Weekly credit reading from either block name.
   */
  const reading = readSyntheticWeeklyReading(quotas,);
  if (reading === INVALID_VALUE)
    return INVALID_RATE_LIMIT_SNAPSHOT;

  /**
   Used weekly credits percentage.
   */
  const usedPercent = PERCENT_BASE - clampNumber({
    value: reading.percentRemaining,
    min: 0,
    max: PERCENT_BASE,
  },);

  return createRateLimitSnapshot({
    key: 'synthetic:week',
    label: 'synthetic week',
    resetAtMs: reading.resetAtMs,
    windowSeconds: SECONDS_PER_DAY,
    paceScale: SYNTHETIC_WEEKLY_PACE_SCALE,
    sampledAtMs: nowMs,
    usedPercent,
  },);
}

/**
 Parses Synthetic hourly search quota into a projectable snapshot.
 
 @param quotas - Synthetic quotas response object
 
 @param nowMs - wall-clock sample time in epoch milliseconds
 
 @returns parsed search snapshot or invalid sentinel
 
 @example
 ```ts
 parseSyntheticSearchSnapshot({ quotas, nowMs: Date.now() });
 ```
 */
function parseSyntheticSearchSnapshot({
  quotas,
  nowMs,
}: Readonly<{
  quotas: UnknownRecord;
  nowMs: number;
}>,): RateLimitSnapshot | InvalidRateLimitSnapshot {
  /**
   Search quota object.
   */
  const search = readRecordProperty({
    record: quotas,
    key: 'search',
  },);
  if (search === INVALID_VALUE)
    return INVALID_RATE_LIMIT_SNAPSHOT;

  /**

   Hourly search quota object.

   */
  const hourly = readRecordProperty({
    record: search,
    key: 'hourly',
  },);
  if (hourly === INVALID_VALUE)
    return INVALID_RATE_LIMIT_SNAPSHOT;

  /**

   Search request limit.

   */
  const limit = readNumberProperty({
    record: hourly,
    key: 'limit',
  },);
  /**
   Search requests already used in this window.
   */
  const requests = readNumberProperty({
    record: hourly,
    key: 'requests',
  },);
  /**
   Hourly reset timestamp.
   */
  const resetAtMs = readResetProperty({
    record: hourly,
    key: 'renewsAt',
  },);
  if ((limit === INVALID_VALUE)
    || (requests === INVALID_VALUE)
    || (resetAtMs === INVALID_VALUE))
    return INVALID_RATE_LIMIT_SNAPSHOT;

  /**

   Used search percentage.

   */
  const usedPercent = usedPercentFromLimit({
    used: requests,
    limit,
  },);
  if (usedPercent === INVALID_VALUE)
    return INVALID_RATE_LIMIT_SNAPSHOT;

  return createRateLimitSnapshot({
    key: 'synthetic:search',
    label: 'synthetic search',
    resetAtMs,
    windowSeconds: SECONDS_PER_HOUR,
    paceScale: 1,
    sampledAtMs: nowMs,
    usedPercent,
  },);
}

/**
 Parses Synthetic daily free-tool quota into a projectable snapshot.
 
 @param quotas - Synthetic quotas response object
 
 @param nowMs - wall-clock sample time in epoch milliseconds
 
 @returns parsed tool snapshot or invalid sentinel
 
 @example
 ```ts
 parseSyntheticToolsSnapshot({ quotas, nowMs: Date.now() });
 ```
 */
function parseSyntheticToolsSnapshot({
  quotas,
  nowMs,
}: Readonly<{
  quotas: UnknownRecord;
  nowMs: number;
}>,): RateLimitSnapshot | InvalidRateLimitSnapshot {
  /**
   Free tool calls quota object.
   */
  const freeToolCalls = readRecordProperty({
    record: quotas,
    key: 'freeToolCalls',
  },);
  if (freeToolCalls === INVALID_VALUE)
    return INVALID_RATE_LIMIT_SNAPSHOT;

  /**

   Free tool call limit.

   */
  const limit = readNumberProperty({
    record: freeToolCalls,
    key: 'limit',
  },);
  /**
   Free tool calls already used.
   */
  const requests = readNumberProperty({
    record: freeToolCalls,
    key: 'requests',
  },);
  /**
   Daily reset timestamp.
   */
  const resetAtMs = readResetProperty({
    record: freeToolCalls,
    key: 'renewsAt',
  },);
  if ((limit === INVALID_VALUE)
    || (requests === INVALID_VALUE)
    || (resetAtMs === INVALID_VALUE))
    return INVALID_RATE_LIMIT_SNAPSHOT;

  /**

   Used free-tool percentage.

   */
  const usedPercent = usedPercentFromLimit({
    used: requests,
    limit,
  },);
  if (usedPercent === INVALID_VALUE)
    return INVALID_RATE_LIMIT_SNAPSHOT;

  return createRateLimitSnapshot({
    key: 'synthetic:tools',
    label: 'synthetic tools',
    resetAtMs,
    windowSeconds: SECONDS_PER_DAY,
    paceScale: 1,
    sampledAtMs: nowMs,
    usedPercent,
  },);
}

/**
 Parses all projectable Synthetic quota windows from response headers.
 
 @param headers - lowercase provider response headers
 
 @param nowMs - wall-clock sample time in epoch milliseconds
 
 @returns parsed Synthetic {@link RateLimitSnapshot} entries
 
 @example
 ```ts
 parseSyntheticRateLimitSnapshots({ headers, nowMs: Date.now() });
 ```
 */
function parseSyntheticRateLimitSnapshots({
  headers,
  nowMs,
}: Readonly<{
  headers: Readonly<Record<string, string>>;
  nowMs: number;
}>,): readonly RateLimitSnapshot[] {
  /**
   Parsed Synthetic quotas header.
   */
  const quotas = parseSyntheticQuotasHeader(headers,);
  if (quotas === INVALID_VALUE)
    return [];

  /**
   Candidate Synthetic snapshots before invalid sentinels are removed.
   */
  const snapshots: readonly (RateLimitSnapshot | InvalidRateLimitSnapshot)[] = [
    parseSyntheticWeeklySnapshot({
      quotas,
      nowMs,
    },),
    parseSyntheticSearchSnapshot({
      quotas,
      nowMs,
    },),
    parseSyntheticToolsSnapshot({
      quotas,
      nowMs,
    },),
  ];

  return snapshots.filter(function keepSyntheticSnapshot(snapshot,): snapshot is RateLimitSnapshot {
    return isRateLimitSnapshot(snapshot,);
  },);
}

export {
  SYNTHETIC_QUOTAS_HEADER,
  parseSyntheticRateLimitSnapshots,
};
