import { mkdir, } from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  CACHE_ABSENT,
  readAsnCache,
  writeAsnCache,
} from './asn-cache.ts';
import { fetchAsnNetworks, } from './asn-fetch.ts';
import {
  AsnDatabaseError,
  normalizeAsn,
} from './asn-network.ts';
import type { LookupAsnNetworks, } from './networks.ts';

export { AsnDatabaseError, } from './asn-network.ts';

/**
 Inputs for low-level ASN database lookup.
 */
export type LookupAsnNetworksOptions = {
  readonly asn: string;
  readonly cacheDirectory: string;
  readonly token: string;
};

/**
 Inputs for invocation-scoped ASN adapter.
 */
export type CreateAsnLookupOptions = {
  readonly cacheDirectory: string;
  readonly token: string;
};

/**
 Environment variable overriding default ASN cache directory.
 */
const CACHE_DIRECTORY_ENVIRONMENT = 'WG_ALLOWEDIPS_CACHE_DIRECTORY';

/**
 Cache subdirectory beneath XDG or home cache root.
 */
const CACHE_SUBDIRECTORY = join(
  'wg-allowedips',
  'asn',
);

/* oxlint-disable eslint/no-magic-numbers -- Cache lifetime is clearest as one named policy duration with explicit unit conversion. */
/**
 Cache lifetime matching existing OpenTofu ASN refresh policy.
 */
const CACHE_LIFETIME_MS = 30 * 24
  * 60
  * 60
  * 1_000;
/* oxlint-enable eslint/no-magic-numbers */

/**
 Module logger for ASN lookup lifecycle.
 */
const l = tagged({ tag: 'asn-networks', },);

/**
 Resolves default process-owned ASN cache directory.
 
 Caller override wins,
 then XDG cache root,
 then user home cache.
 
 @returns Cache directory independent of workspace source layout.
 
 @example
 ```ts
 defaultAsnCacheDirectory();
 ```
 */
export function defaultAsnCacheDirectory(): string {
  /**
   Explicit application cache path when caller configured one.
   */
  const configured = process.env[CACHE_DIRECTORY_ENVIRONMENT];
  if ((configured !== undefined) && (configured !== ''))
    return configured;
  /**
   Standard user cache root when present.
   */
  const { XDG_CACHE_HOME: xdgCacheHome, } = process.env;
  if ((xdgCacheHome !== undefined) && (xdgCacheHome !== '')) {
    return join(
      xdgCacheHome,
      CACHE_SUBDIRECTORY,
    );
  }
  return join(
    homedir(),
    '.cache',
    CACHE_SUBDIRECTORY,
  );
}

/**
 Resolves every IPinfo Lite network assigned to one ASN with per-ASN caching.
 
 Fresh cache avoids network access.
 Refresh failure falls back to stale validated cache.
 Cache directory is explicit and created by this boundary.
 
 @param asn - ASN in case-insensitive `AS<number>` syntax.
 
 @param cacheDirectory - Directory containing per-ASN snapshots.
 
 @param token - IPinfo Lite token used only when refresh is required.
 
 @returns Network or single-address strings assigned to ASN.
 
 @throws {@link AsnDatabaseError} when ASN is invalid or refresh fails without stale cache.
 
 @example
 ```ts
 await lookupAsnNetworks({
   asn: 'AS64500',
   cacheDirectory: '/tmp/asn-cache',
   token: process.env.IPINFO_TOKEN ?? '',
 });
 ```
 */
export async function lookupAsnNetworks(
  {
    asn,
    cacheDirectory,
    token,
  }: LookupAsnNetworksOptions,
): Promise<readonly string[]> {
  /**
   Function-scoped logger for one normalized ASN.
   */
  const fl = tagged({
    tag: lookupAsnNetworks.name,
    l,
  },);
  /**
   Normalized ASN used for filtering and cache naming.
   */
  const targetAsn = normalizeAsn(asn,);
  await mkdir(
    cacheDirectory,
    { recursive: true, },
  );
  /**
   Per-ASN cache path under explicit owner directory.
   */
  const cachePath = join(
    cacheDirectory,
    `cache_${targetAsn}.txt`,
  );
  /**
   Fresh cache result when snapshot is inside lifetime policy.
   */
  const fresh = await readAsnCache({
    cachePath,
    targetAsn,
    earliestMtimeMs: Date.now() - CACHE_LIFETIME_MS,
  },);
  if (fresh !== CACHE_ABSENT) {
    fl.debug(`using fresh cache for ${targetAsn}`,);
    return fresh;
  }
  try {
    /**
     Current database records after cache expiration or absence.
     */
    const networks = await fetchAsnNetworks({
      targetAsn,
      token,
    },);
    await writeAsnCache({
      cacheDirectory,
      cachePath,
      targetAsn,
      text: networks.join(',',),
    },);
    return networks;
  }
  catch (error) {
    fl.error(`live fetch failed for ${targetAsn}; attempting stale cache: ${String(error,)}`,);
    /**
     Valid stale cache used only after live refresh failure.
     */
    const stale = await readAsnCache({
      cachePath,
      targetAsn,
    },);
    if (stale === CACHE_ABSENT) {
      throw new AsnDatabaseError(
        `IPinfo Lite lookup failed for ${targetAsn} and no cached fallback is available.`,
        { cause: error, },
      );
    }
    return stale;
  }
}

/**
 Creates invocation-scoped ASN adapter coalescing duplicate lookups.
 
 @param cacheDirectory - Explicit cache directory owned by caller.
 
 @param token - IPinfo token used for refreshes.
 
 @returns ASN resolver seam sharing pending and fulfilled work by ASN.
 
 @example
 ```ts
 const lookup = createAsnLookup({ cacheDirectory: '/tmp/asn', token: '' });
 await lookup({ asn: 'AS64500' });
 ```
 */
export function createAsnLookup(
  {
    cacheDirectory,
    token,
  }: CreateAsnLookupOptions,
): LookupAsnNetworks {
  /**
   Pending and fulfilled lookups keyed by normalized caller input.
   */
  const lookupPromises = new Map<string, Promise<readonly string[]>>();
  return async function lookupAsnNetworksForGeneration(
    { asn, }: { readonly asn: string; },
  ): Promise<readonly string[]> {
    /**
     Existing coalesced lookup when ASN already appeared in this generation.
     */
    const existing = lookupPromises.get(asn,);
    if (existing !== undefined)
      return await existing;
    /**
     New lookup registered before awaiting so concurrent duplicates share it.
     */
    const pending = lookupAsnNetworks({
      asn,
      cacheDirectory,
      token,
    },);
    lookupPromises.set(
      asn,
      pending,
    );
    return await pending;
  };
}
