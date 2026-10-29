import { lookup, } from 'node:dns/promises';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  createAsnLookup,
  defaultAsnCacheDirectory,
} from './asn-networks.ts';
import { generateAllowedIpsWithLookup, } from './generate-with-lookup.ts';
import type { LookupAddress, } from './networks.ts';

export {
  createAsnLookup,
  defaultAsnCacheDirectory,
  lookupAsnNetworks,
} from './asn-networks.ts';
export type {
  CreateAsnLookupOptions,
  LookupAsnNetworksOptions,
} from './asn-networks.ts';
export { AsnDatabaseError, } from './asn-network.ts';
export { InputValidationError, } from './errors.ts';
export { generateAllowedIpsWithLookup, } from './generate-with-lookup.ts';
export type { GenerateAllowedIpsWithLookupOptions, } from './generate-with-lookup.ts';
export type {
  LookupAddress,
  LookupAddresses,
  LookupAsnNetworks,
} from './networks.ts';

/**
 Text inputs accepted by {@link generateAllowedIps}.
 */
export type GenerateAllowedIpsOptions = {
  readonly allowedText: string;
  readonly disallowedText: string;
};

/**
 Module logger for the production resolver boundary.
 */
const l = tagged({ tag: 'generate', },);

/**
 Operating-system lookup adapter contributing every address returned for one hostname.
 
 @param hostname - Domain name resolved through Node's operating-system lookup.
 
 @returns Every address from the point-in-time lookup.
 
 @example
 ```ts
 await lookupAddresses({ hostname: 'localhost' });
 ```
 */
async function lookupAddresses(
  { hostname, }: { readonly hostname: string; },
): Promise<readonly LookupAddress[]> {
  /**
   Function-scoped logger for operating-system resolution.
   */
  const fl = tagged({
    tag: lookupAddresses.name,
    l,
  },);
  fl.debug(`resolving ${hostname} through operating-system lookup`,);
  /**
   Complete lookup result from Node.
   */
  const addresses = await lookup(
    hostname,
    { all: true, },
  );
  fl.debug(`operating-system lookup returned ${String(addresses.length,)} address(es)`,);
  return addresses;
}

/**
 Generates a minimized WireGuard `AllowedIPs` value from allowed and disallowed file text.
 
 Domains resolve through the operating system once per input entry during this call.
 ASNs resolve through IPinfo Lite with a process-owned per-ASN cache.
 
 @param allowedText - Complete allowed-file text.
 
 @param disallowedText - Complete disallowed-file text.
 
 @returns Empty string for complete subtraction, otherwise comma-separated CIDRs and one trailing newline.
 
 @example
 ```ts
 await generateAllowedIps({
   allowedText: '10.0.0.0/8',
   disallowedText: '10.0.0.0/9',
 });
 // => '10.128.0.0/9\n'
 ```
 */
export async function generateAllowedIps(
  {
    allowedText,
    disallowedText,
  }: GenerateAllowedIpsOptions,
): Promise<string> {
  /**
   Function-scoped logger for public generation lifecycle.
   */
  const fl = tagged({
    tag: generateAllowedIps.name,
    l,
  },);
  fl.debug('generating AllowedIPs text',);
  /**
   Result generated through the internal resolver seam.
   */
  const result = await generateAllowedIpsWithLookup({
    allowedText,
    disallowedText,
    lookupAddresses,
    lookupAsnNetworks: createAsnLookup({
      cacheDirectory: defaultAsnCacheDirectory(),
      token: process.env
        .IPINFO_TOKEN
        ?? '',
    },),
  },);
  fl.debug('generated AllowedIPs text',);
  return result;
}
