/**
 DeepSeek search query composition for the fixed filter policy.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { LinkupWebSearchInput, } from './client.ts';
import { isOperatorDate, } from './deepseek-date.ts';
import { normalizeBlocklistEntry, } from './domain-policy.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 */
const deepseekLogger = tagged({ tag: 'pi-search-fetch', },);

//region Constants

/**
 Unicode code point limit on one DeepSeek search query.
 */
// oxlint-disable-next-line eslint/no-magic-numbers -- DeepSeek documents 128 Unicode characters as the hard query cap; the constant name is the only readable place to record the API value.
const DEEPSEEK_QUERY_MAX_CODE_POINTS = 128 as const;

/**
 Domain separator used between repeated `site:` operators.
 */
const DOMAIN_OPERATOR_SEPARATOR = ' OR ' as const;

/**
 Separator between the base query and each operator clause.
 */
const QUERY_CLAUSE_SEPARATOR = ' ' as const;

/**
 Separator between a DeepSeek operator and its value.
 */
const OPERATOR_VALUE_SEPARATOR = ':' as const;

/**
 Include-domain operator name.
 */
const SITE_OPERATOR = 'site' as const;

/**
 Lower date bound operator name.
 */
const AFTER_OPERATOR = 'after' as const;

/**
 Upper date bound operator name.
 */
const BEFORE_OPERATOR = 'before' as const;

//endregion Constants

//region Types

/**
 Resolution of a raw include-domain filter into a host suffix.
 */
type IncludeDomainHost = {
  /**
   Whether the filter named a usable host.
   */
  readonly found: false;
} | {
  /**
   Whether the filter named a usable host.
   */
  readonly found: true;
  /**
   Normalized host suffix.
   */
  readonly host: string;
};

//endregion Types

/**
 Module logger.
 */
const l = tagged({
  tag: 'deepseek-query',
  l: deepseekLogger,
},);

/**
 Compose the DeepSeek query, adding filter operators while they fit the limit.
 
 @param input - model-facing search input
 
 @returns query with the operators that fit
 
 @throws when the base query alone exceeds 128 code points
 
 @example
 ```ts
 composeDeepseekQuery({ query: 'docs', includeDomains: ['example.com'] });
 ```
 */
function composeDeepseekQuery(input: LinkupWebSearchInput,): string {
  /**
   Code points in the base query.
   */
  const baseLength = countCodePoints(input.query,);
  if (baseLength > DEEPSEEK_QUERY_MAX_CODE_POINTS)
    throw new Error(
      `DeepSeek search query is ${String(baseLength,)} code points; the limit is ${String(DEEPSEEK_QUERY_MAX_CODE_POINTS,)}`,
    );
  /**
   Operator clauses in append order.
   */
  const clauses = filterOperatorClauses(input,);
  return clauses.reduce(
    function appendClause(
      query,
      clause,
    ) {
      /**
       Candidate query with this clause appended.
       */
      const candidate = `${query}${QUERY_CLAUSE_SEPARATOR}${clause}`;
      if (countCodePoints(candidate,) > DEEPSEEK_QUERY_MAX_CODE_POINTS) {
        l.debug(`dropping DeepSeek query operator ${clause}; query would exceed ${String(DEEPSEEK_QUERY_MAX_CODE_POINTS,)} code points`,);
        return query;
      }
      return candidate;
    },
    input.query,
  );
}

/**
 Build operator clauses for the filters present on one search input.
 
 @param input - model-facing search input
 
 @returns operator clauses DeepSeek accepts for the supplied filters
 
 @example
 ```ts
 filterOperatorClauses({ query: 'docs', fromDate: '2026-01-01' });
 ```
 */
function filterOperatorClauses(input: LinkupWebSearchInput,): readonly string[] {
  /**
   Clauses in stable append order: domains, start date, end date.
   */
  const clauses: string[] = [];
  /**
   Normalized include domains.
   */
  const domains = normalizeIncludeDomains((input.includeDomains === undefined ? {} : { includeDomains: input.includeDomains, }),);
  if (domains.length > 0) {
    /**
     Domain clause group joined by OR for DeepSeek.
     */
    const domainClause = domains.map(function siteClause(domain,) {
      return `${SITE_OPERATOR}${OPERATOR_VALUE_SEPARATOR}${domain}`;
    },)
      .join(DOMAIN_OPERATOR_SEPARATOR,);
    clauses.push(domainClause,);
  }
  if ((input.fromDate !== undefined) && isOperatorDate(input.fromDate,))
    clauses.push(`${AFTER_OPERATOR}${OPERATOR_VALUE_SEPARATOR}${input.fromDate}`,);
  if ((input.toDate !== undefined) && isOperatorDate(input.toDate,))
    clauses.push(`${BEFORE_OPERATOR}${OPERATOR_VALUE_SEPARATOR}${input.toDate}`,);
  return clauses;
}

/**
 Normalize include-domain filters into host suffixes DeepSeek operators accept.
 
 @param includeDomains - raw include-domain filters
 
 @returns unique normalized host suffixes in input order
 
 @example
 ```ts
 normalizeIncludeDomains({ includeDomains: ['https://Example.com/docs'] });
 ```
 */
function normalizeIncludeDomains(
  {
    includeDomains,
  }: {
    readonly includeDomains?: readonly string[];
  },
): readonly string[] {
  if (includeDomains === undefined)
    return [];
  /**
   Normalized domains in first-seen order.
   */
  const normalized: string[] = [];
  for (const domain of includeDomains) {
    /**
     Normalized host, when the filter names one.
     */
    const lookup = includeDomainHost(domain,);
    if (!lookup.found)
      continue;
    if (!normalized.includes(lookup.host,))
      normalized.push(lookup.host,);
  }
  return normalized;
}

/**
 Normalize one include-domain filter into a host suffix.
 
 @param value - raw include-domain filter
 
 @returns normalized host suffix resolution
 
 @example
 ```ts
 includeDomainHost('https://Example.com/docs');
 ```
 */
function includeDomainHost(value: string,): IncludeDomainHost {
  /**
   Trimmed filter text.
   */
  const trimmed = value.trim();
  if (trimmed === '')
    return { found: false, };
  try {
    /**
     Parsed host from the filter, accepting a bare domain or a URL.
     */
    const host = new URL(trimmed.includes('://') ? trimmed : `https://${trimmed}`,).hostname;
    return {
      found: true,
      host: normalizeBlocklistEntry(host,),
    };
  }
  catch (error: unknown) {
    l.debug(`ignoring invalid DeepSeek include domain ${JSON.stringify(value,)}: ${String(error,)}`,);
    return { found: false, };
  }
}

/**
 Count Unicode code points in one query string.
 
 DeepSeek documents its query limit in Unicode characters, so counting code
 points is deliberate; grapheme clusters are not what the API measures.
 
 @param value - query text
 
 @returns code point count
 
 @example
 ```ts
 countCodePoints('docs');
 ```
 */
function countCodePoints(value: string,): number {
  return Array.from(value,)
    .length;
}

export {
  composeDeepseekQuery,
  countCodePoints,
  normalizeIncludeDomains,
};
