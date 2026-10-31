import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Host-suffix blocklist normalization and matching for Pi Search Fetch.
 
 @module
 */

/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: linkupLogger, },);
 ```
 */
const linkupLogger = tagged({ tag: 'pi-search-fetch', },);

//region Constants

/**
 Characters allowed in normalized host suffix entries.
 */
const HOST_SUFFIX_ALLOWED_CHARS = 'abcdefghijklmnopqrstuvwxyz0123456789-.';

/**
 Scheme delimiter rejected in blocklist entries.
 */
const SCHEME_DELIMITER = '://';

/**
 Slash rejected in blocklist entries.
 */
const SLASH = '/';

/**
 Colon rejected in blocklist entries.
 */
const COLON = ':';

/**
 Wildcard rejected in blocklist entries.
 */
const WILDCARD = '*';

/**
 Dot used as host label separator.
 */
const DOT = '.';

/**
 Boundary suffix separator for subdomain matching.
 */
const DOT_PREFIX = '.';

//endregion Constants

//region Types

/**
 Result of filtering Linkup search results with the local host blocklist.
 */
type SearchResultFilterResult = {
  /**
   Linkup-shaped response visible to the model after local filtering.
   */
  readonly linkupResponse: unknown;
  /**
   Untouched upstream Linkup response.
   */
  readonly rawLinkupResponse: unknown;
  /**
   Blocked result URLs removed from the model-visible response.
   */
  readonly removedBlockedUrls: readonly string[];
};

/**
 Host or URL blocklist match result.
 */
type BlocklistMatch = {
  /**
   Whether input matched the blocklist.
   */
  readonly blocked: false;
} | {
  /**
   Whether input matched the blocklist.
   */
  readonly blocked: true;
  /**
   Matching blocklist entry.
   */
  readonly entry: string;
};

/**
 Optional URL extracted from a Linkup search result.
 */
type SearchResultUrl = {
  /**
   Whether a URL was present.
   */
  readonly found: false;
} | {
  /**
   Whether a URL was present.
   */
  readonly found: true;
  /**
   Search result URL.
   */
  readonly url: string;
};

//endregion Types

/**
 Module logger.
 */
const l = tagged({
  tag: 'domain-policy',
  l: linkupLogger,
},);

//region Blocklist normalization

/**
 Normalize one configured host suffix.
 
 @param entry - raw blocklist entry from config
 
 @returns normalized host suffix
 
 @throws when entry is empty or not a strict host suffix
 
 @example
 ```ts
 normalizeBlocklistEntry(' Example.COM. ');
 ```
 */
function normalizeBlocklistEntry(entry: string,): string {
  /**
   Local value for innerL.
   */
  const innerL = tagged({
    tag: normalizeBlocklistEntry.name,
    l,
  },);
  /**
   Local value for trimmed.
   */
  const trimmed = entry.trim();
  if (trimmed === '')
    throw new Error('blocklist entry is empty');

  /**
   Local value for lowered.
   */
  const lowered = trimmed.toLowerCase();
  /**
   Local value for normalized.
   */
  const normalized = stripOneTrailingDot(lowered,);
  if (normalized === '')
    throw new Error(`blocklist entry ${JSON.stringify(entry,)} is empty after normalization`);
  if (normalized.includes(SCHEME_DELIMITER,))
    throw new Error(`blocklist entry ${JSON.stringify(entry,)} must not include a scheme`);
  if (normalized.includes(SLASH,))
    throw new Error(`blocklist entry ${JSON.stringify(entry,)} must not include a slash`);
  if (normalized.includes(COLON,))
    throw new Error(`blocklist entry ${JSON.stringify(entry,)} must not include a port`);
  if (normalized.includes(WILDCARD,))
    throw new Error(`blocklist entry ${JSON.stringify(entry,)} must not include a wildcard`);

  /**
   Local value for labels.
   */
  const labels = normalized.split(DOT,);
  if (labels.some(function isEmptyLabel(label,) {
    return label === '';
  },))
    throw new Error(`blocklist entry ${JSON.stringify(entry,)} must not include empty labels`);
  if (!hasOnlyAllowedHostSuffixChars(normalized,))
    throw new Error(`blocklist entry ${JSON.stringify(entry,)} contains invalid host characters`);

  innerL.debug(`normalized blocklist entry ${entry} to ${normalized}`,);
  return normalized;
}

/**
 Normalize all configured host suffixes and remove duplicates.
 
 @param entries - raw config blocklist entries
 
 @returns normalized blocklist
 
 @throws when any entry is invalid
 
 @example
 ```ts
 normalizeBlocklist(['Example.com.', 'example.com']);
 ```
 */
function normalizeBlocklist(entries: readonly string[],): readonly string[] {
  /**
   Local value for normalizedEntries.
   */
  const normalizedEntries = entries.map(function normalizeEntry(entry,) {
    return normalizeBlocklistEntry(entry,);
  },);
  return [...new Set(normalizedEntries,),];
}

/**
 Strip one optional trailing root dot.
 
 @param value - host suffix candidate
 
 @returns host suffix without one trailing dot
 
 @example
 ```ts
 stripOneTrailingDot('example.com.');
 ```
 */
function stripOneTrailingDot(value: string,): string {
  if (!value.endsWith(DOT,))
    return value;
  return value.slice(
    0,
    -1,
  );
}

/**
 Return whether a normalized host suffix character is allowed.
 
 @param char - one string iterator character
 
 @returns whether char is allowed in a host suffix
 
 @example
 ```ts
 isAllowedHostSuffixChar('a');
 ```
 */
function isAllowedHostSuffixChar(char: string,): boolean {
  return HOST_SUFFIX_ALLOWED_CHARS.includes(char,);
}

/**
 Return whether host suffix text uses only allowed ASCII host characters.
 
 @param value - normalized host suffix text
 
 @returns whether every character is allowed
 */
function hasOnlyAllowedHostSuffixChars(value: string,): boolean {
  for (const char of value) {
    if (!isAllowedHostSuffixChar(char,))
      return false;
  }
  return true;
}

//endregion Blocklist normalization

//region Host matching

/**
 Normalize URL hostnames before local policy matching.
 
 @param host - URL hostname
 
 @returns lowercase host without one optional trailing root dot
 
 @example
 ```ts
 normalizeHostForPolicy('WWW.Example.COM.');
 ```
 */
function normalizeHostForPolicy(host: string,): string {
  /**
   Local value for trimmed.
   */
  const trimmed = host.trim();
  /**
   Local value for lowered.
   */
  const lowered = trimmed.toLowerCase();
  return stripOneTrailingDot(lowered,);
}

/**
 Find blocklist entry matching a host.
 
 @param host - candidate URL host
 
 @param blocklist - normalized host suffix blocklist
 
 @returns match result
 
 @example
 ```ts
 findBlockedHostMatch({ host: 'www.example.com', blocklist: ['example.com'] });
 ```
 */
function findBlockedHostMatch(
  {
    host,
    blocklist,
  }: {
    readonly host: string;
    readonly blocklist: readonly string[];
  },
): BlocklistMatch {
  /**
   Local value for normalizedHost.
   */
  const normalizedHost = normalizeHostForPolicy(host,);
  /**
   Local value for matchedEntry.
   */
  const matchedEntry = blocklist.find(function matchesEntry(entry,) {
    return (normalizedHost === entry)
      || normalizedHost.endsWith(`${DOT_PREFIX}${entry}`,);
  },);
  return matchedEntry === undefined
    ? { blocked: false, }
    : {
      blocked: true,
      entry: matchedEntry,
    };
}

/**
 Return whether a host is blocked.
 
 @param host - candidate URL host
 
 @param blocklist - normalized host suffix blocklist
 
 @returns whether host matches blocklist exactly or as a subdomain suffix
 
 @example
 ```ts
 isBlockedHost({ host: 'www.example.com', blocklist: ['example.com'] });
 ```
 */
function isBlockedHost(
  {
    host,
    blocklist,
  }: {
    readonly host: string;
    readonly blocklist: readonly string[];
  },
): boolean {
  return findBlockedHostMatch({
    host,
    blocklist,
  },)
    .blocked;
}

/**
 Find blocklist entry matching a URL.
 
 @param url - candidate absolute URL
 
 @param blocklist - normalized host suffix blocklist
 
 @returns match result
 
 @throws when url cannot be parsed
 
 @example
 ```ts
 findBlockedUrlMatch({ url: 'https://www.example.com/a', blocklist: ['example.com'] });
 ```
 */
function findBlockedUrlMatch(
  {
    url,
    blocklist,
  }: {
    readonly url: string;
    readonly blocklist: readonly string[];
  },
): BlocklistMatch {
  /**
   Local value for parsedUrl.
   */
  const parsedUrl = parsePolicyUrl(url,);
  return findBlockedHostMatch({
    host: parsedUrl.hostname,
    blocklist,
  },);
}

/**
 Return whether a URL is blocked.
 
 @param url - candidate absolute URL
 
 @param blocklist - normalized host suffix blocklist
 
 @returns whether URL host is blocked
 
 @throws when url cannot be parsed
 
 @example
 ```ts
 isBlockedUrl({ url: 'https://example.com', blocklist: ['example.com'] });
 ```
 */
function isBlockedUrl(
  {
    url,
    blocklist,
  }: {
    readonly url: string;
    readonly blocklist: readonly string[];
  },
): boolean {
  return findBlockedUrlMatch({
    url,
    blocklist,
  },)
    .blocked;
}

/**
 Parse a URL for blocklist policy checks.
 
 @param url - candidate absolute URL
 
 @returns parsed URL
 
 @throws when url cannot be parsed
 
 @example
 ```ts
 parsePolicyUrl('https://example.com');
 ```
 */
function parsePolicyUrl(url: string,): URL {
  try {
    return new URL(url,);
  }
  catch (error: unknown) {
    /**
     Local value for detail.
     */
    const detail = caughtValueText(error,);
    throw new Error(
      `Invalid URL for pi-search-fetch blocklist check: ${url}. ${detail}`,
      { cause: error, },
    );
  }
}

//endregion Host matching

//region Search result filtering

/**
 Filter blocked Linkup search result URLs out of a Linkup-shaped response.
 
 @param response - upstream Linkup search response
 
 @param blocklist - normalized host suffix blocklist
 
 @returns raw response plus model-visible response after local filtering
 
 @example
 ```ts
 filterBlockedSearchResults({ response: { results: [] }, blocklist: [] });
 ```
 */
function filterBlockedSearchResults(
  {
    response,
    blocklist,
  }: {
    readonly response: unknown;
    readonly blocklist: readonly string[];
  },
): SearchResultFilterResult {
  if (blocklist.length === 0)
    return unfilteredSearchResponse(response,);
  if (!isRecord(response,))
    return unfilteredSearchResponse(response,);

  /**
   Local destructured value.
   */
  const { results: rawResults, } = response;
  if (!Array.isArray(rawResults,))
    return unfilteredSearchResponse(response,);

  /**
   Local value for removedBlockedUrls.
   */
  const removedBlockedUrls: string[] = [];
  /**
   Local value for filteredResults.
   */
  const filteredResults = rawResults.filter(function keepAllowedResult(result,) {
    /**
     Local value for resultUrl.
     */
    const resultUrl = searchResultUrl(result,);
    if (!resultUrl.found)
      return true;

    /**
     Local value for blockedEntry.
     */
    const blockedEntry = blockedEntryForPossiblyInvalidUrl({
      url: resultUrl.url,
      blocklist,
    },);
    if (!blockedEntry.blocked)
      return true;

    removedBlockedUrls.push(resultUrl.url,);
    return false;
  },);

  if (removedBlockedUrls.length === 0)
    return unfilteredSearchResponse(response,);

  l.warn(`removed ${String(removedBlockedUrls.length,)} blocked Linkup search result(s)`,);
  return {
    linkupResponse: {
      ...response,
      results: filteredResults,
    },
    rawLinkupResponse: response,
    removedBlockedUrls,
  };
}

/**
 Build an unfiltered search filter result.
 
 @param response - upstream response
 
 @returns unfiltered filter result
 */
function unfilteredSearchResponse(response: unknown,): SearchResultFilterResult {
  return {
    linkupResponse: response,
    rawLinkupResponse: response,
    removedBlockedUrls: [],
  };
}

/**
 Return blocklist match for a Linkup result URL or unblocked for unparsable URLs.
 
 @param url - Linkup result URL
 
 @param blocklist - normalized host suffix blocklist
 
 @returns matching blocklist entry, when URL parses and is blocked
 
 @example
 ```ts
 blockedEntryForPossiblyInvalidUrl({ url: 'https://example.com', blocklist: ['example.com'] });
 ```
 */
function blockedEntryForPossiblyInvalidUrl(
  {
    url,
    blocklist,
  }: {
    readonly url: string;
    readonly blocklist: readonly string[];
  },
): BlocklistMatch {
  try {
    return findBlockedUrlMatch({
      url,
      blocklist,
    },);
  }
  catch (error: unknown) {
    l.warn(`skipping local blocklist filtering for unparsable Linkup result URL: ${url}: ${String(error,)}`,);
    return { blocked: false, };
  }
}

/**
 Return a Linkup result URL from an arbitrary result value.
 
 @param value - Linkup result candidate
 
 @returns URL extraction result
 
 @example
 ```ts
 searchResultUrl({ url: 'https://example.com' });
 ```
 */
function searchResultUrl(value: unknown,): SearchResultUrl {
  if (!isRecord(value,))
    return { found: false, };
  /**
   Local destructured value.
   */
  const { url, } = value;
  return (typeof url) === 'string'
    ? {
      found: true,
      url,
    }
    : { found: false, };
}

/**
 Return whether value is a non-null object record.
 
 @param value - unknown value
 
 @returns whether value can be read by string keys
 
 @example
 ```ts
 isRecord({});
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return (value !== null)
    && ((typeof value) === 'object')
    && (!Array.isArray(value,));
}

//endregion Search result filtering

export {
  filterBlockedSearchResults,
  findBlockedHostMatch,
  findBlockedUrlMatch,
  isBlockedHost,
  isBlockedUrl,
  normalizeBlocklist,
  normalizeBlocklistEntry,
  normalizeHostForPolicy,
};
export type {
  BlocklistMatch,
  SearchResultFilterResult,
};
