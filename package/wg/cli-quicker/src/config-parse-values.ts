import { homedir, } from 'node:os';

import { ConfigError, } from './errors.ts';
import {
  splitWords,
  trimLinear,
} from './text.ts';

/**
 Highest valid UDP port number.
 */
const MAX_UDP_PORT = 65_535;

/**
 Paths parsed from one peer's `AllowedIPsFromFiles` value.
 */
export type AllowedFromFilesPaths = {
  /**
   File containing allowed address-set entries.
   */
  readonly allowed: string;

  /**
   File containing disallowed address-set entries.
   */
  readonly disallowed: string;
};

/**
 Strips inline comment and trims without regular expression.
 
 @param line - Raw config line.
 
 @returns Content before first comment introducer.
 
 @example
 ```ts
 stripComment({ line: 'Address = 10.0.0.1/32 # tunnel' });
 ```
 */
export function stripComment(
  { line, }: { readonly line: string; },
): string {
  /**
   First comment introducer or absent index.
   */
  const hash = line.indexOf('#',);
  return trimLinear({ value: hash === (-1) ? line : line.slice(
    0,
    hash,
  ), },);
}

/**
 Parses the `AllowedIPsFromFiles` value into its two file paths.
 
 @param value - Raw value text, expected `allowed.txt disallowed.txt`.
 
 @returns The two file paths.
 
 @throws {@link ConfigError} when the value does not name exactly two paths.
 
 @example
 ```ts
 parseAllowedFromFiles({ value: '~/allowed.txt ~/disallowed.txt' });
 ```
 */
export function parseAllowedFromFiles(
  { value, }: { readonly value: string; },
): AllowedFromFilesPaths {
  /**
   Whitespace-separated path tokens.
   */
  const parts = splitWords({ line: value, },);
  /**
   Destructured allowed path, disallowed path, and any rejected extra tokens.
   */
  const [allowed, disallowed, ...rest] = parts;
  if ((allowed === undefined) || (disallowed === undefined)
    || (rest.length > 0)) {
    throw new ConfigError(
      `Invalid \`AllowedIPsFromFiles' value \`${value}': expected exactly two paths (allowed, disallowed)`,
    );
  }
  return {
    allowed: expandHome({ path: allowed, },),
    disallowed: expandHome({ path: disallowed, },),
  };
}

/**
 Expands a leading `~` in a file path to the current user's home directory.
 
 @param path - Path possibly starting with `~` or `~/`.
 
 @returns The absolute path with the home directory substituted.
 
 @example
 ```ts
 expandHome({ path: '~/allowed.txt' });
 ```
 */
export function expandHome({ path, }: { readonly path: string; },): string {
  if (path === '~')
    return homedir();
  if (path.startsWith('~/',))
    return `${homedir()}${path.slice(1,)}`;
  return path;
}

/**
 Reports whether one character is an ASCII digit or a dot.
 
 @param char - Candidate character from a DNS token.
 
 @returns True for `0` to `9` and `.` only.
 
 @example
 ```ts
 isDigitOrDotChar('4');
 ```
 */
function isDigitOrDotChar(char: string,): boolean {
  return ((char >= '0') && (char <= '9')) || (char === '.');
}

/**
 Reports whether DNS token is IP literal rather than search domain.
 
 The digits-and-dots branch is a lexical run check, not an IPv4 validator;
 it accepts any non-empty run of ASCII digits and dots.
 
 @param token - One comma-separated DNS token.
 
 @returns True when token is digits/dots or contains IPv6 colon.
 
 @example
 ```ts
 isIpLiteral({ token: '198.245.51.147' });
 ```
 */
export function isIpLiteral(
  { token, }: { readonly token: string; },
): boolean {
  if (token.includes(':',))
    return true;
  if (token.length === 0)
    return false;
  for (const char of token) {
    if (!isDigitOrDotChar(char,))
      return false;
  }
  return true;
}

/**
 Parses a positive integer interface option, rejecting non-numeric input.
 
 @param key - Option name used in the error message.
 
 @param value - Raw value text from the config file.
 
 @returns The parsed positive integer.
 
 @throws {@link ConfigError} when the value is not a positive integer.
 
 @example
 ```ts
 parsePositiveInt({ key: 'MTU', value: '1420' });
 ```
 */
export function parsePositiveInt(
  {
    key,
    value,
  }: {
    readonly key: string;
    readonly value: string;
  },
): number {
  /**
   Numeric value before the integer check.
   */
  const parsed = Number(value,);
  if ((!Number.isSafeInteger(parsed,)) || (parsed <= 0)) {
    throw new ConfigError(`Invalid \`${key}' value \`${value}': expected a positive integer`,);
  }
  return parsed;
}

/**
 Extracts UDP destination port from one WireGuard peer endpoint.
 
 Hostnames and IPv4 use `host:port`;
 IPv6 must use WireGuard's bracketed `[address]:port` form.
 
 @param value - Comment-stripped peer endpoint value.
 
 @returns Valid UDP port from endpoint.
 
 @throws {@link ConfigError} when endpoint lacks valid host or port.
 
 @example
 ```ts
 parseEndpointPort({ value: '[2001:db8::1]:51820' });
 ```
 */
export function parseEndpointPort(
  { value, }: { readonly value: string; },
): number {
  /**
   Final colon separates host from port for every valid endpoint form.
   */
  const separator = value.lastIndexOf(':',);
  /**
   Final endpoint index used to reject missing port text.
   */
  const finalIndex = value.length - 1;
  if ((separator <= 0) || (separator === finalIndex))
    throw new ConfigError(`Invalid \`Endpoint' value \`${value}': expected host:port`,);
  /**
   Host section before final separator.
   */
  const host = value.slice(
    0,
    separator,
  );
  if (host.includes(':',) && ((!host.startsWith('[',)) || (!host.endsWith(']',)))) {
    throw new ConfigError(
      `Invalid \`Endpoint' value \`${value}': IPv6 address must use [address]:port`,
    );
  }
  /**
   Validated positive port candidate.
   */
  const port = parsePositiveInt({
    key: 'Endpoint port',
    value: value.slice(separator + 1,),
  },);
  if (port > MAX_UDP_PORT)
    throw new ConfigError(`Invalid \`Endpoint' value \`${value}': port exceeds 65535`,);
  return port;
}
