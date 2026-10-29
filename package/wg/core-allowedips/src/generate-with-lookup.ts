import { tagged, } from '@monochromatic-dev/module-logger/ts';
import { excludeCidr, } from 'cidr-tools';

import { InputValidationError, } from './errors.ts';
import {
  type LookupAddresses,
  type LookupAsnNetworks,
  textNetworks,
} from './networks.ts';

/**
 Input required to generate an `AllowedIPs` value with an injected resolver.
 */
export type GenerateAllowedIpsWithLookupOptions = {
  readonly allowedText: string;
  readonly disallowedText: string;
  readonly lookupAddresses: LookupAddresses;
  readonly lookupAsnNetworks: LookupAsnNetworks;
};

/**
 Module logger for address-set generation.
 */
const l = tagged({ tag: 'generate-with-lookup', },);

/**
 Complete IANA-designated IPv4 and IPv6 loopback address space.
 */
const LOOPBACK_NETWORKS = [
  '127.0.0.0/8',
  '::1/128',
] as const;

/**
 Generates exact `union(allowed) - union(disallowed)` output with an injected resolver.
 
 This is a built-artifact test seam and is not exported through the package export map.
 
 @param allowedText - Complete allowed-file text.
 
 @param disallowedText - Complete disallowed-file text.
 
 @param lookupAddresses - Deterministic test or operating-system resolver adapter.
 
 @param lookupAsnNetworks - Deterministic test or IPinfo Lite ASN adapter.
 
 @returns Empty string for complete subtraction, otherwise minimized sorted CIDRs joined by `, ` and one newline.
 
 @throws {@link InputValidationError} when allowed input contributes no addresses.
 
 @example
 ```ts
 await generateAllowedIpsWithLookup({
   allowedText: '10.0.0.0/8',
   disallowedText: '10.0.0.0/9',
   lookupAddresses: async () => [],
   lookupAsnNetworks: async () => [],
 });
 // => '10.128.0.0/9\n'
 ```
 */
export async function generateAllowedIpsWithLookup(
  {
    allowedText,
    disallowedText,
    lookupAddresses,
    lookupAsnNetworks,
  }: GenerateAllowedIpsWithLookupOptions,
): Promise<string> {
  /**
   Function-scoped logger for set-generation lifecycle.
   */
  const fl = tagged({
    tag: generateAllowedIpsWithLookup.name,
    l,
  },);
  fl.debug('parsing allowed and disallowed input',);
  /**
   Parsed allowed and disallowed networks, resolved concurrently.
   */
  const [allowedNetworks, disallowedNetworks,] = await Promise.all([
    textNetworks({
      text: allowedText,
      lookupAddresses,
      lookupAsnNetworks,
    },),
    textNetworks({
      text: disallowedText,
      lookupAddresses,
      lookupAsnNetworks,
    },),
  ],);
  /**
   Loopback remainder not semantically covered by resolved disallowed networks.
   */
  const uncoveredLoopbackNetworks = excludeCidr(
    LOOPBACK_NETWORKS,
    disallowedNetworks,
  );
  if (uncoveredLoopbackNetworks.length > 0) {
    fl.warn(
      `disallowed IPs do not cover all loopback ranges; uncovered: ${uncoveredLoopbackNetworks.join(', ',)}`,
    );
  }
  if (allowedNetworks.length === 0) {
    fl.error('allowed input contributed no addresses',);
    throw new InputValidationError('Allowed input must contain at least one address.',);
  }
  fl.debug(
    `subtracting ${String(disallowedNetworks.length,)} network(s) from ${String(allowedNetworks.length,)} network(s)`,
  );
  /**
   Minimized, sorted exact set difference produced by adopted dependency.
   */
  const result = excludeCidr(
    allowedNetworks,
    disallowedNetworks,
  );
  if (result.length === 0) {
    fl.debug('subtraction produced an empty set',);
    return '';
  }
  fl.debug(`subtraction produced ${String(result.length,)} CIDR(s)`,);
  return `${result.join(', ',)}\n`;
}
