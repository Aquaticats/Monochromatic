/**
 GitHub URL parsing and argument validation helpers.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  DASH_PREFIX,
  EMPTY_SEGMENT,
  HIGHEST_DIGIT,
  HOST_DOT,
  HTTP_SCHEME,
  HTTPS_SCHEME,
  LOWEST_DIGIT,
  MAPPED_GITHUB_HOSTS,
  MAXIMUM_REFERENCE_NUMBER_DIGITS,
  SPACE_CHARACTER,
  TILDE_CHARACTER,
  TOKEN_ALLOWED_CHARACTERS,
  URL_SLASH,
  WWW_HOST_PREFIX,
} from './github-url-plan-constants.ts';
import type {
  ParsedGitHubUrl,
  SegmentValidation,
  TokenValidation,
  UrlParseResult,
} from './github-fetch-types.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: validationLogger, },);
 ```
 */
const validationLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'github-url-validation',
  l: validationLogger,
},);

//region URL parsing

/**
 Parse one URL and normalize its host and path segments for planning.
 
 @param url - absolute URL requested by the fetch tool
 
 @returns parsed planning input, or a safe rejection reason
 
 @example
 ```ts
 parseGitHubUrl('https://github.com/cli/cli');
 ```
 */
function parseGitHubUrl(url: string,): ParsedGitHubUrl {
  /**
   URL parse attempt result.
   */
  const parsed = tryParseUrl(url,);
  if (!parsed.parsed)
    return {
      parsed: false,
      reason: parsed.reason,
    };

  if ((parsed.url
    .protocol
    !== HTTPS_SCHEME) && (parsed.url
      .protocol
      !== HTTP_SCHEME))
    return {
      parsed: false,
      reason: `scheme ${parsed.url
        .protocol} is not an HTTP GitHub URL`,
    };
  if (parsed.url
    .port
    !== EMPTY_SEGMENT)
    return {
      parsed: false,
      reason: `port ${parsed.url
        .port} is not a mapped GitHub host`,
    };

  /**
   Lowercase host without one www label or one trailing root dot.
   */
  const host = normalizeGitHubHost(parsed.url
    .hostname,);
  if (!MAPPED_GITHUB_HOSTS.includes(host,))
    return {
      parsed: false,
      reason: `host ${host} has no gh mapping`,
    };

  return {
    parsed: true,
    url: parsed.url,
    host,
    segments: repositoryPathSegments(parsed.url,),
  };
}

/**
 Attempt one URL parse without a mutable function-root binding.
 
 @param url - absolute URL requested by the fetch tool
 
 @returns parsed URL, or a safe parse failure reason
 
 @example
 ```ts
 tryParseUrl('https://github.com/cli/cli');
 ```
 */
function tryParseUrl(url: string,): UrlParseResult {
  try {
    return {
      parsed: true,
      url: new URL(url,),
    };
  }
  catch (error: unknown) {
    return {
      parsed: false,
      reason: `unparsable URL: ${caughtValueText(error,)}`,
    };
  }
}

/**
 Normalize one hostname before GitHub host matching.
 
 @param host - URL hostname
 
 @returns lowercase host without one trailing root dot or one www label
 
 @example
 ```ts
 normalizeGitHubHost('WWW.GitHub.COM.');
 ```
 */
function normalizeGitHubHost(host: string,): string {
  /**
   Trimmed lowercase host.
   */
  const lowered = host
    .trim()
    .toLowerCase();
  /**
   Host without one trailing root dot.
   */
  const dotted = lowered.endsWith(HOST_DOT,)
    ? lowered.slice(
      0,
      -1,
    )
    : lowered;
  return dotted.startsWith(WWW_HOST_PREFIX,)
    ? dotted.slice(WWW_HOST_PREFIX.length,)
    : dotted;
}

/**
 Extract path segments without leading or trailing empty entries.
 
 @param url - parsed GitHub URL
 
 @returns path segments, keeping interior empty segments for rejection
 
 @example
 ```ts
 repositoryPathSegments(new URL('https://github.com/cli/cli/'));
 ```
 */
function repositoryPathSegments(url: URL,): readonly string[] {
  /**
   Raw slash-separated pathname parts, whose first entry is empty.
   */
  const rawSegments = url.pathname
    .split(URL_SLASH,);
  /**
   Parts without the leading empty entry produced by the root slash.
   */
  const withoutLeadingEmpty = (rawSegments[0] === EMPTY_SEGMENT)
    ? rawSegments.slice(1,)
    : rawSegments;
  /**
   Index of the last segment carrying content, negative when every segment is empty.
   */
  const lastContentIndex = lastContentSegmentIndex(withoutLeadingEmpty,);
  return withoutLeadingEmpty.slice(
    0,
    lastContentIndex + 1,
  );
}

/**
 Find the index of the last non-empty segment.
 
 @param segments - path segments
 
 @returns last non-empty index, or negative one when no segment carries content
 
 @example
 ```ts
 lastContentSegmentIndex(['cli', 'cli', '']);
 ```
 */
function lastContentSegmentIndex(segments: readonly string[],): number {
  return segments.reduce(
    function keepLastContentIndex(
      lastIndex: number,
      segment: string,
      position: number,
    ): number {
      return segment === EMPTY_SEGMENT
        ? lastIndex
        : position;
    },
    -1,
  );
}

//endregion URL parsing

//region Segment validation

/**
 Validate every path segment before any of them reaches gh.
 
 @param segments - URL path segments
 
 @returns validation result naming the first rejected segment
 
 @example
 ```ts
 validatePathSegments(['cli', 'cli', 'README.md']);
 ```
 */
function validatePathSegments(segments: readonly string[],): SegmentValidation {
  /**
   First segment rejected by path safety rules.
   */
  const unsafeSegment = segments.find(function isUnsafe(segment,) {
    return !isSafePathSegment(segment,);
  },);
  return unsafeSegment === undefined
    ? { valid: true, }
    : {
      valid: false,
      reason: `path segment ${JSON.stringify(unsafeSegment,)} is not a safe gh argument`,
    };
}

/**
 Return whether one path segment can be forwarded inside a gh endpoint.
 
 @param segment - one URL path segment
 
 @returns whether the segment is non-empty and printable ASCII
 
 @example
 ```ts
 isSafePathSegment('README.md');
 ```
 */
function isSafePathSegment(segment: string,): boolean {
  return (segment !== EMPTY_SEGMENT) && isPrintableAscii(segment,);
}

/**
 Decode one percent-encoded URL text value, keeping the raw text when decoding fails.
 
 URL path segments stay percent-encoded,
 while gh arguments and query values need decoded text,
 and a bare percent sign makes decoding impossible.
 
 @param value - percent-encoded text taken from a URL path
 
 @returns decoded text, or the raw value when it carries a malformed escape
 
 @example
 ```ts
 decodeUrlText('topic%2Bone');
 ```
 */
function decodeUrlText(value: string,): string {
  /**
   Logger tagged for this decode call.
   */
  const innerL = tagged({
    tag: decodeUrlText.name,
    l,
  },);
  try {
    return decodeURIComponent(value,);
  }
  catch (error: unknown) {
    innerL.debug(`keeping raw URL text ${JSON.stringify(value,)} after a malformed escape: ${caughtValueText(error,)}`,);
    return value;
  }
}

/**
 Return whether one value stays inside printable ASCII.
 
 URL parsing already percent-encodes every non-printable character,
 so this guard holds that invariant for any caller building a gh argument without URL parsing.
 
 @param value - argument candidate
 
 @returns whether every character sits between exclusive space and inclusive tilde
 
 @example
 ```ts
 isPrintableAscii('docs/README.md');
 ```
 */
function isPrintableAscii(value: string,): boolean {
  // Code point iteration is correct here: the rule is a per-code-point ASCII range test, so any
  // non-ASCII code point, including combining marks and emoji, must fail as one whole unit.
  for (const character of value) {
    if ((character <= SPACE_CHARACTER) || (character > TILDE_CHARACTER))
      return false;
  }
  return true;
}

//endregion Segment validation

//region Argument validation

/**
 Validate one positional gh argument restricted to GitHub naming characters.
 
 @param value - candidate argument value
 
 @param label - argument name used in rejection reasons
 
 @returns validation result
 
 @example
 ```ts
 validateTokenArgument({ value: 'cli', label: 'repository owner' });
 ```
 */
function validateTokenArgument(
  {
    value,
    label,
  }: {
    readonly value: string;
    readonly label: string;
  },
): TokenValidation {
  if (value === EMPTY_SEGMENT)
    return {
      safe: false,
      reason: `${label} is empty`,
    };
  if (value.startsWith(DASH_PREFIX,))
    return {
      safe: false,
      reason: `${label} ${JSON.stringify(value,)} would parse as a gh flag`,
    };
  for (const character of value) {
    if (!TOKEN_ALLOWED_CHARACTERS.includes(character,))
      return {
        safe: false,
        reason: `${label} ${JSON.stringify(value,)} contains characters GitHub does not allow in it`,
      };
  }
  return { safe: true, };
}

/**
 Validate one issue or pull request number.
 
 @param value - candidate reference number
 
 @param label - reference name used in rejection reasons
 
 @returns validation result
 
 @example
 ```ts
 validateReferenceNumber({ value: '13118', label: 'issue' });
 ```
 */
function validateReferenceNumber(
  {
    value,
    label,
  }: {
    readonly value: string;
    readonly label: string;
  },
): TokenValidation {
  if ((!hasOnlyDigits(value,)) || (value.length > MAXIMUM_REFERENCE_NUMBER_DIGITS))
    return {
      safe: false,
      reason: `${label} reference ${JSON.stringify(value,)} is not a number`,
    };
  return { safe: true, };
}

/**
 Return whether one value contains only decimal digits.
 
 @param value - candidate reference number
 
 @returns whether every character is a decimal digit
 
 @example
 ```ts
 hasOnlyDigits('13118');
 ```
 */
function hasOnlyDigits(value: string,): boolean {
  for (let index = 0; index < value.length; index += 1) {
    if ((value.charAt(index,) < LOWEST_DIGIT) || (value.charAt(index,) > HIGHEST_DIGIT))
      return false;
  }
  return true;
}

/**
 Validate one value forwarded inside a gh endpoint path or query.
 
 @param value - candidate endpoint fragment
 
 @param label - fragment name used in rejection reasons
 
 @returns validation result
 
 @example
 ```ts
 validateEndpointFragment({ value: 'v2.101.0', label: 'release tag' });
 ```
 */
function validateEndpointFragment(
  {
    value,
    label,
  }: {
    readonly value: string;
    readonly label: string;
  },
): TokenValidation {
  if (value === EMPTY_SEGMENT)
    return {
      safe: false,
      reason: `${label} is empty`,
    };
  if (!isPrintableAscii(value,))
    return {
      safe: false,
      reason: `${label} contains characters outside printable ASCII`,
    };
  return { safe: true, };
}

/**
 Validate one positional gh argument that may contain path separators.
 
 @param value - candidate argument value
 
 @param label - argument name used in rejection reasons
 
 @returns validation result
 
 @example
 ```ts
 validatePositionalPathArgument({ value: 'release/1.0', label: 'release tag' });
 ```
 */
function validatePositionalPathArgument(
  {
    value,
    label,
  }: {
    readonly value: string;
    readonly label: string;
  },
): TokenValidation {
  /**
   Printable ASCII and emptiness validation shared with endpoint fragments.
   */
  const fragmentValidation = validateEndpointFragment({
    value,
    label,
  },);
  if (!fragmentValidation.safe)
    return fragmentValidation;
  if (value.startsWith(DASH_PREFIX,))
    return {
      safe: false,
      reason: `${label} ${JSON.stringify(value,)} would parse as a gh flag`,
    };
  return { safe: true, };
}

//endregion Argument validation

export {
  decodeUrlText,
  isPrintableAscii,
  parseGitHubUrl,
  validateEndpointFragment,
  validatePathSegments,
  validatePositionalPathArgument,
  validateReferenceNumber,
  validateTokenArgument,
};
