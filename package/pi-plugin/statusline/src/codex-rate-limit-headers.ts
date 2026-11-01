/**
 OpenAI Codex subscription rate-limit response header parsing.
 
 Header names mirror OpenAI Codex CLI `codex-api/src/rate_limits.rs`.
 
 @module
 */

import {
  SECONDS_PER_DAY,
  SECONDS_PER_MINUTE,
  type RateLimitSnapshot,
} from './rate-limit-types.ts';
import {
  INVALID_RATE_LIMIT_SNAPSHOT,
  INVALID_VALUE,
  createRateLimitSnapshot,
  isRateLimitSnapshot,
  parseEpochSecondsHeader,
  parseNumberHeader,
  parseStringHeader,
  type InvalidRateLimitSnapshot,
  type InvalidValue,
} from './rate-limit-parse-helpers.ts';

/**
 Codex header prefix used for the default ChatGPT subscription limit family.
 */
const CODEX_DEFAULT_PREFIX = 'x-codex';

/**
 Codex primary used-percent suffix used to discover dynamic limit families.
 */
const CODEX_PRIMARY_USED_PERCENT_SUFFIX = '-primary-used-percent';

/**
 Codex secondary used-percent suffix used to discover dynamic limit families.
 */
const CODEX_SECONDARY_USED_PERCENT_SUFFIX = '-secondary-used-percent';

/**
 Lower duration tolerance matching Codex CLI approximate-window checks.
 */
const CODEX_DURATION_LOWER_MULTIPLIER = 0.95;

/**
 Upper duration tolerance matching Codex CLI approximate-window checks.
 */
const CODEX_DURATION_UPPER_MULTIPLIER = 1.05;

/**
 Codex five-hour window duration in minutes.
 */
const CODEX_FIVE_HOURS_MINUTES = 300;

/**
 Codex daily window duration in minutes.
 */
const CODEX_DAY_MINUTES = SECONDS_PER_DAY / SECONDS_PER_MINUTE;

/**
 Codex weekly window duration in minutes.
 */
const CODEX_WEEK_MINUTES = 10_080;

/**
 Codex monthly window duration in minutes.
 */
const CODEX_MONTH_MINUTES = 43_200;

/**
 Codex window kinds parsed for each limit prefix.
 */
const CODEX_WINDOW_KINDS = [
  'primary',
  'secondary',
] as const;

/**
 Codex window kind reflected in response header names.
 */
type CodexWindowKind = typeof CODEX_WINDOW_KINDS[number];

/**
 Parsed Codex window data before generic snapshot conversion.
 */
type CodexWindow = {
  /**
   Provider-used percentage.
   */
  readonly usedPercent: number;
  /**
   Window duration in minutes.
   */
  readonly windowMinutes: number;
  /**
   Reset timestamp in epoch milliseconds.
   */
  readonly resetAtMs: number;
};

/**
 Detects approximate duration match.
 
 @param minutes - actual minute count
 
 @param expectedMinutes - expected minute count
 
 @returns whether actual minutes fall inside five-percent tolerance
 
 @example
 ```ts
 isApproximateMinutes({ minutes: 301, expectedMinutes: 300 });
 ```
 */
function isApproximateMinutes({
  minutes,
  expectedMinutes,
}: Readonly<{
  minutes: number;
  expectedMinutes: number;
}>,): boolean {
  /**
   Lower bound matching Codex CLI duration classification.
   */
  const lowerBound = expectedMinutes * CODEX_DURATION_LOWER_MULTIPLIER;
  /**
   Upper bound matching Codex CLI duration classification.
   */
  const upperBound = expectedMinutes * CODEX_DURATION_UPPER_MULTIPLIER;

  return (minutes >= lowerBound) && (minutes <= upperBound);
}

/**
 Converts Codex header prefix to stable limit id.
 
 @param prefix - lowercase header prefix beginning with `x-`
 
 @returns stable limit id
 
 @example
 ```ts
 codexLimitIdFromPrefix('x-codex-secondary');
 ```
 */
function codexLimitIdFromPrefix(prefix: string,): string {
  /**
   Prefix after the transport-level `x-` namespace.
   */
  const withoutHeaderNamespace = prefix.startsWith('x-',)
    ? prefix.slice(2,)
    : prefix;

  return withoutHeaderNamespace.replaceAll(
    '-',
    '_',
  );
}

/**
 Formats Codex duration label from window minutes.
 
 @param windowMinutes - provider window duration in minutes
 
 @returns compact duration label or fallback text
 
 @example
 ```ts
 codexDurationLabel(300);
 ```
 */
function codexDurationLabel(windowMinutes: number,): string {
  /**
   Window duration clamped to non-negative minutes.
   */
  const minutes = Math.max(
    0,
    windowMinutes,
  );

  if (isApproximateMinutes({
    minutes,
    expectedMinutes: CODEX_FIVE_HOURS_MINUTES,
  },))
    return '5h';
  if (isApproximateMinutes({
    minutes,
    expectedMinutes: CODEX_DAY_MINUTES,
  },))
    return 'daily';
  if (isApproximateMinutes({
    minutes,
    expectedMinutes: CODEX_WEEK_MINUTES,
  },))
    return 'weekly';
  if (isApproximateMinutes({
    minutes,
    expectedMinutes: CODEX_MONTH_MINUTES,
  },))
    return 'monthly';

  return `${Math.round(minutes,)}m`;
}

/**
 Formats Codex snapshot label.
 
 @param limitName - server-provided limit name or derived limit id
 
 @param kind - primary or secondary Codex window
 
 @param windowMinutes - provider window duration in minutes
 
 @returns footer label for Codex projected overflow
 
 @example
 ```ts
 codexSnapshotLabel({ limitName: 'codex', kind: 'primary', windowMinutes: 300 });
 ```
 */
function codexSnapshotLabel({
  limitName,
  kind,
  windowMinutes,
}: Readonly<{
  limitName: string;
  kind: CodexWindowKind;
  windowMinutes: number;
}>,): string {
  /**
   Duration label derived from provider window metadata.
   */
  const durationLabel = codexDurationLabel(windowMinutes,);
  /**
   Secondary suffix when Codex reports a secondary window.
   */
  const secondaryLabel = kind === 'secondary' ? ' secondary' : '';

  return `${limitName} ${durationLabel}${secondaryLabel}`;
}

/**
 Discovers Codex header prefixes from known window suffixes.
 
 @param headers - lowercase provider response headers
 
 @returns sorted header prefixes to parse
 
 @example
 ```ts
 codexPrefixes({ 'x-codex-primary-used-percent': '50' });
 ```
 */
function codexPrefixes(headers: Readonly<Record<string, string>>,): readonly string[] {
  /**
   Mutable unique prefix set seeded with the default Codex family.
   */
  const prefixes = new Set<string>([CODEX_DEFAULT_PREFIX,],);

  for (const headerName of Object.keys(headers,)) {
    if (headerName.endsWith(CODEX_PRIMARY_USED_PERCENT_SUFFIX,)) {
      prefixes.add(headerName.slice(
        0,
        -CODEX_PRIMARY_USED_PERCENT_SUFFIX.length,
      ),);
      continue;
    }
    if (headerName.endsWith(CODEX_SECONDARY_USED_PERCENT_SUFFIX,))
      prefixes.add(headerName.slice(
        0,
        -CODEX_SECONDARY_USED_PERCENT_SUFFIX.length,
      ),);
  }

  return [...prefixes].toSorted(function sortPrefix(
    left,
    right,
  ): number {
    return left.localeCompare(right,);
  },);
}

/**
 Reads Codex limit display name for a header prefix.
 
 @param headers - lowercase provider response headers
 
 @param prefix - Codex header prefix
 
 @returns display label for the limit family
 
 @example
 ```ts
 codexLimitName({ headers: {}, prefix: 'x-codex' });
 ```
 */
function codexLimitName({
  headers,
  prefix,
}: Readonly<{
  headers: Readonly<Record<string, string>>;
  prefix: string;
}>,): string {
  /**
   Optional server-provided display name header.
   */
  const parsedLimitName = parseStringHeader({
    headers,
    headerName: `${prefix}-limit-name`,
  },);

  if (parsedLimitName !== INVALID_VALUE)
    return parsedLimitName;
  if (prefix === CODEX_DEFAULT_PREFIX)
    return 'codex';

  return codexLimitIdFromPrefix(prefix,)
    .replaceAll(
    '_',
    ' ',
    );
}

/**
 Parses one Codex window from a header prefix.
 
 @param headers - lowercase provider response headers
 
 @param prefix - Codex header prefix
 
 @param kind - primary or secondary Codex window
 
 @returns parsed {@link CodexWindow}, or invalid sentinel when incomplete
 
 @example
 ```ts
 parseCodexWindow({ headers, prefix: 'x-codex', kind: 'primary' });
 ```
 */
function parseCodexWindow({
  headers,
  prefix,
  kind,
}: Readonly<{
  headers: Readonly<Record<string, string>>;
  prefix: string;
  kind: CodexWindowKind;
}>,): CodexWindow | InvalidValue {
  /**
   Used percentage reported by Codex.
   */
  const usedPercent = parseNumberHeader({
    headers,
    headerName: `${prefix}-${kind}-used-percent`,
  },);
  /**
   Window duration in minutes reported by Codex.
   */
  const windowMinutes = parseNumberHeader({
    headers,
    headerName: `${prefix}-${kind}-window-minutes`,
  },);
  /**
   Reset timestamp reported by Codex as epoch seconds.
   */
  const resetAtMs = parseEpochSecondsHeader({
    headers,
    headerName: `${prefix}-${kind}-reset-at`,
  },);

  if ((usedPercent === INVALID_VALUE)
    || (windowMinutes === INVALID_VALUE)
    || (resetAtMs === INVALID_VALUE)
    || (windowMinutes <= 0))
    return INVALID_VALUE;

  return {
    usedPercent,
    windowMinutes,
    resetAtMs,
  };
}

/**
 Converts one parsed Codex window into generic snapshot shape.
 
 @param window - parsed {@link CodexWindow}
 
 @param prefix - Codex header prefix
 
 @param limitName - Codex display limit name
 
 @param kind - primary or secondary Codex window
 
 @param nowMs - wall-clock sample time in epoch milliseconds
 
 @returns generic {@link RateLimitSnapshot}
 
 @example
 ```ts
 codexWindowSnapshot({ window, prefix: 'x-codex', limitName: 'codex', kind: 'primary', nowMs: Date.now() });
 ```
 */
function codexWindowSnapshot({
  window,
  prefix,
  limitName,
  kind,
  nowMs,
}: Readonly<{
  window: CodexWindow;
  prefix: string;
  limitName: string;
  kind: CodexWindowKind;
  nowMs: number;
}>,): RateLimitSnapshot | InvalidRateLimitSnapshot {
  return createRateLimitSnapshot({
    key: `${codexLimitIdFromPrefix(prefix,)}:${kind}`,
    label: codexSnapshotLabel({
      limitName,
      kind,
      windowMinutes: window.windowMinutes,
    },),
    resetAtMs: window.resetAtMs,
    windowSeconds: window.windowMinutes * SECONDS_PER_MINUTE,
    paceScale: 1,
    sampledAtMs: nowMs,
    usedPercent: window.usedPercent,
  },);
}

/**
 Parses all Codex subscription usage windows from response headers.
 
 @param headers - lowercase provider response headers
 
 @param nowMs - wall-clock sample time in epoch milliseconds
 
 @returns parsed Codex usage snapshots
 
 @example
 ```ts
 parseCodexRateLimitSnapshots({ headers, nowMs: Date.now() });
 ```
 */
function parseCodexRateLimitSnapshots({
  headers,
  nowMs,
}: {
  readonly headers: Record<string, string>;
  readonly nowMs: number;
},): readonly RateLimitSnapshot[] {
  /**
   Mutable Codex snapshots collected from all discovered prefixes.
   */
  const snapshots: RateLimitSnapshot[] = [];

  for (const prefix of codexPrefixes(headers,)) {
    /**
     Display limit name for this Codex prefix.
     */
    const limitName = codexLimitName({
      headers,
      prefix,
    },);

    for (const kind of CODEX_WINDOW_KINDS) {
      /**
       Parsed Codex window for this prefix and kind.
       */
      const window = parseCodexWindow({
        headers,
        prefix,
        kind,
      },);
      if (window === INVALID_VALUE)
        continue;

      /**

       Generic snapshot converted from Codex window data.

       */
      const snapshot = codexWindowSnapshot({
        window,
        prefix,
        limitName,
        kind,
        nowMs,
      },);
      if (isRateLimitSnapshot(snapshot,))
        snapshots.push(snapshot,);
    }
  }

  return snapshots;
}

export {
  CODEX_DEFAULT_PREFIX,
  parseCodexRateLimitSnapshots,
};
