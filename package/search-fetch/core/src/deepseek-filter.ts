/**
 Local DeepSeek result filtering and deduplication.
 
 DeepSeek cannot receive the full blocklist as query operators, so filters that
 do not fit the query limit are enforced here after the provider responds.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { LinkupWebSearchInput, } from './client.ts';
import { dateBoundary, } from './deepseek-date.ts';
import { normalizeIncludeDomains, } from './deepseek-query.ts';
import {
  findBlockedHostMatch,
  normalizeHostForPolicy,
} from './domain-policy.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 */
const deepseekLogger = tagged({ tag: 'pi-search-fetch', },);

//region Types

/**
 Resolution of a result URL into a deduplication key.
 */
type UrlPathKey = {
  /**
   Whether the result carried a parsable URL.
   */
  readonly found: false;
} | {
  /**
   Whether the result carried a parsable URL.
   */
  readonly found: true;
  /**
   Host plus path key.
   */
  readonly key: string;
};

//endregion Types

/**
 Module logger.
 */
const l = tagged({
  tag: 'deepseek-filter',
  l: deepseekLogger,
},);

/**
 Read the results array from a DeepSeek search response envelope.
 
 @param response - parsed provider response
 
 @returns raw results array
 
 @throws when the response is not an object with a results array
 
 @example
 ```ts
 searchResultsFromResponse({ response: { results: [] } });
 ```
 */
function searchResultsFromResponse(
  {
    response,
  }: {
    readonly response: unknown;
  },
): readonly unknown[] {
  if (!isRecord(response,))
    throw new Error(
      'DeepSeek /v1/tools/search returned an unexpected response shape',
    );
  /**
   Results field from the response envelope.
   */
  const { results, } = response;
  if (!Array.isArray(results,))
    throw new Error(
      'DeepSeek /v1/tools/search returned an unexpected response shape',
    );
  return results;
}

/**
 Apply local provider filters and deduplicate DeepSeek search results.
 
 @param results - raw DeepSeek results
 
 @param input - model-facing search input supplying the filters
 
 @returns filtered and deduplicated results in provider order
 
 @example
 ```ts
 filterDeepseekResults({ results: [], input: { query: 'docs' } });
 ```
 */
function filterDeepseekResults(
  {
    results,
    input,
  }: {
    readonly results: readonly unknown[];
    readonly input: LinkupWebSearchInput;
  },
): readonly unknown[] {
  /**
   Normalized include domains.
   */
  const domains = normalizeIncludeDomains((input.includeDomains === undefined ? {} : { includeDomains: input.includeDomains, }),);
  /**
   Filtered results in provider order.
   */
  const filtered = results.filter(function keepResult(result,) {
    return keepsIncludeDomain({
      result,
      domains,
    },)
      && keepsDateRange({
        result,
        ...(input.fromDate === undefined ? {} : { fromDate: input.fromDate, }),
        ...(input.toDate === undefined ? {} : { toDate: input.toDate, }),
      },);
  },);
  return dedupeByUrlPath(filtered,);
}

/**
 Return whether a result matches at least one include domain.
 
 @param result - DeepSeek result candidate
 
 @param domains - normalized include domains
 
 @returns whether the result survives the include-domain filter
 
 @example
 ```ts
 keepsIncludeDomain({ result: { url: 'https://example.com' }, domains: ['example.com'] });
 ```
 */
function keepsIncludeDomain(
  {
    result,
    domains,
  }: {
    readonly result: unknown;
    readonly domains: readonly string[];
  },
): boolean {
  if (domains.length === 0)
    return true;
  if ((!isRecord(result,)) || ((typeof result.url) !== 'string'))
    return false;
  try {
    return findBlockedHostMatch({
      host: new URL(result.url,).hostname,
      blocklist: domains,
    },)
      .blocked;
  }
  catch (error: unknown) {
    l.debug(`dropping DeepSeek result with unparsable URL for include domains: ${String(error,)}`,);
    return false;
  }
}

/**
 Return whether a result date stays inside the requested window.
 
 Undated and unparsable result dates are kept; only a known date outside the
 window removes the result.
 
 @param result - DeepSeek result candidate
 
 @param fromDate - optional lower date bound
 
 @param toDate - optional upper date bound
 
 @returns whether the result survives the date filter
 
 @example
 ```ts
 keepsDateRange({ result: { date: '2026-01-01T00:00:00Z' }, fromDate: '2026-01-01' });
 ```
 */
function keepsDateRange(
  {
    result,
    fromDate,
    toDate,
  }: {
    readonly result: unknown;
    readonly fromDate?: string;
    readonly toDate?: string;
  },
): boolean {
  if ((fromDate === undefined) && (toDate === undefined))
    return true;
  if (!isRecord(result,))
    return true;
  /**
   Publication date from the result.
   */
  const {date} = result;
  if ((typeof date) !== 'string')
    return true;
  /**
   Publication instant in milliseconds.
   */
  const time = Date.parse(date,);
  if (Number.isNaN(time,))
    return true;
  /**
   Lower bound resolution, when a start filter is present.
   */
  const lowerBound = dateBoundary({
    ...(fromDate === undefined ? {} : { value: fromDate, }),
    endOfRange: false,
  },);
  if (lowerBound.found && (time < lowerBound.instant))
    return false;
  /**
   Upper bound resolution, when an end filter is present.
   */
  const upperBound = dateBoundary({
    ...(toDate === undefined ? {} : { value: toDate, }),
    endOfRange: true,
  },);
  if (upperBound.found && (time > upperBound.instant))
    return false;
  return true;
}

/**
 Deduplicate results by URL host plus path, keeping the first occurrence.
 
 @param results - filtered results in provider order
 
 @returns results without repeated host plus path pairs
 
 @example
 ```ts
 dedupeByUrlPath([{ url: 'https://example.com/a' }]);
 ```
 */
function dedupeByUrlPath(results: readonly unknown[],): readonly unknown[] {
  /**
   Host plus path keys already kept.
   */
  const seen = new Set<string>();
  return results.filter(function keepFirstResult(result,) {
    /**
     Deduplication key resolution for this result.
     */
    const resolved = resultUrlPathKey(result,);
    if (!resolved.found)
      return true;
    if (seen.has(resolved.key,)) {
      l.debug(`dropping duplicate DeepSeek result ${resolved.key}`,);
      return false;
    }
    seen.add(resolved.key,);
    return true;
  },);
}

/**
 Build a host plus path key for one result URL.
 
 @param result - result candidate
 
 @returns deduplication key resolution
 
 @example
 ```ts
 resultUrlPathKey({ url: 'https://Example.com/a#fragment' });
 ```
 */
function resultUrlPathKey(result: unknown,): UrlPathKey {
  if ((!isRecord(result,)) || ((typeof result.url) !== 'string'))
    return { found: false, };
  try {
    /**
     Parsed result URL.
     */
    const parsed = new URL(result.url,);
    return {
      found: true,
      key: `${normalizeHostForPolicy(parsed.hostname,)}${parsed.pathname}`,
    };
  }
  catch (error: unknown) {
    l.debug(`keeping DeepSeek result with unparsable URL: ${String(error,)}`,);
    return { found: false, };
  }
}

/**
 Return whether a value is a non-null object record.
 
 @param value - unknown value
 
 @returns whether the value can be read by string keys
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return (value !== null)
    && ((typeof value) === 'object')
    && (!Array.isArray(value,));
}

export {
  filterDeepseekResults,
  searchResultsFromResponse,
};
