import { isIP, } from 'node:net';

/**
 Error raised when ASN syntax or IPinfo Lite database access fails.
 
 @example
 ```ts
 throw new AsnDatabaseError('ASN must use AS<number> syntax: AS-example');
 ```
 */
export class AsnDatabaseError extends Error {
  /**
   Stable error type name.
   */
  override name = 'AsnDatabaseError';
}

/**
 Node family marker for IPv4.
 */
const IPV4_FAMILY = 4;

/**
 Maximum IPv4 prefix.
 */
const IPV4_PREFIX = 32;

/**
 Maximum IPv6 prefix.
 */
const IPV6_PREFIX = 128;

/**
 Checks whether text contains only ASCII decimal digits.
 
 @param text - Candidate decimal suffix.
 
 @returns Whether every character is an ASCII digit and text is nonempty.
 
 @example
 ```ts
 isDecimalDigits('41231'); // true
 ```
 */
function isDecimalDigits(text: string,): boolean {
  if (text === '')
    return false;
  for (const character of text) {
    if ((character < '0') || (character > '9'))
      return false;
  }
  return true;
}

/**
 Normalizes and validates conventional `AS<number>` text.
 
 @param asn - Candidate ASN text.
 
 @returns Uppercase normalized ASN.
 
 @throws {@link AsnDatabaseError} when syntax is not `AS<number>`.
 
 @example
 ```ts
 normalizeAsn('as41231'); // 'AS41231'
 ```
 */
export function normalizeAsn(asn: string,): string {
  /**
   Case-normalized trimmed ASN.
   */
  const normalized = asn
    .trim()
    .toUpperCase();
  if ((!normalized.startsWith('AS',))
    || (!isDecimalDigits(normalized.slice(2,),))) {
    throw new AsnDatabaseError(`ASN must use AS<number> syntax: ${asn}`,);
  }
  return normalized;
}

/**
 Validates one IPinfo network or single-address record without changing its text.
 
 @param network - Candidate database or cache entry.
 
 @param targetAsn - ASN named in validation diagnostics.
 
 @returns Validated original network text.
 
 @throws {@link AsnDatabaseError} when text is neither an IP address nor bounded CIDR.
 
 @example
 ```ts
 validateNetwork({ network: '91.189.88.0/24', targetAsn: 'AS41231' });
 ```
 */
export function validateNetwork(
  {
    network,
    targetAsn,
  }: {
    readonly network: string;
    readonly targetAsn: string;
  },
): string {
  if (isIP(network,) !== 0)
    return network;
  /**
   Only slash separating CIDR address and prefix.
   */
  const slashIndex = network.indexOf('/',);
  if ((slashIndex <= 0) || (slashIndex !== network.lastIndexOf('/',))) {
    throw new AsnDatabaseError(`Invalid network for ${targetAsn}: ${network}`,);
  }
  /**
   Original address text before CIDR prefix.
   */
  const address = network.slice(
    0,
    slashIndex,
  );
  /**
   Strict address family before prefix.
   */
  const family = isIP(address,);
  /**
   Decimal prefix text after slash.
   */
  const prefixText = network.slice(slashIndex + 1,);
  if ((family === 0) || (!isDecimalDigits(prefixText,))) {
    throw new AsnDatabaseError(`Invalid network for ${targetAsn}: ${network}`,);
  }
  /**
   Family-specific largest accepted prefix.
   */
  const maximumPrefix = family === IPV4_FAMILY
    ? IPV4_PREFIX
    : IPV6_PREFIX;
  if (Number(prefixText,) > maximumPrefix) {
    throw new AsnDatabaseError(`Invalid network for ${targetAsn}: ${network}`,);
  }
  return network;
}
