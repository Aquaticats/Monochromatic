import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  resolve4,
  resolve6,
} from 'node:dns/promises';
import { join, } from 'node:path';
import { json, } from 'node:stream/consumers';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 OpenTofu `data.external` query carrying comma-separated service hostnames.
 
 @example
 ```ts
 const query: ResolveHostsQuery = { hostnames: 'nginx.org,palant.info' };
 ```
 */
type ResolveHostsQuery = {
  readonly hostnames: string;
};

/**
 DNS record family resolved for each hostname.
 
 @example
 ```ts
 const family: AddressFamily = 'A';
 ```
 */
type AddressFamily = 'A' | 'AAAA';

/**
 Unknown JSON object shape used before field-specific type guards run.
 
 @example
 ```ts
 const object: UnknownRecord = { 'nginx.org': [] };
 ```
 */
type UnknownRecord = Record<PropertyKey, unknown>;

/**
 CIDR lists keyed by hostname, used for both seed and accumulation cache files.
 
 @example
 ```ts
 const map: CidrsByHost = { 'nginx.org': ['3.147.99.198/32'] };
 ```
 */
type CidrsByHost = Record<string, readonly string[]>;

/**
 Comma-joined CIDR strings keyed by hostname, matching OpenTofu's `map(string)` result shape.
 
 @example
 ```ts
 const result: ResultByHost = { 'nginx.org': '3.147.99.198/32,2a05:d014:5c0:2600::6/128' };
 ```
 */
type ResultByHost = Record<string, string>;

/**
 One resolved hostname paired with its accumulated CIDR list, used to rebuild the map.
 
 @example
 ```ts
 const entry: ResolvedEntry = ['nginx.org', ['3.147.99.198/32']];
 ```
 */
type ResolvedEntry = readonly [
  string,
  readonly string[],
];

/**
 One resolved hostname paired with its comma-joined CIDR string for OpenTofu output.
 
 @example
 ```ts
 const entry: ResultEntry = ['nginx.org', '3.147.99.198/32'];
 ```
 */
type ResultEntry = readonly [
  string,
  string,
];

/**
 Logger root for this script's diagnostics, tagged with the script's own name.
 OpenTofu reads this script's stdout as external-data JSON, so records stay at
 `warn` or above: that is the only level the console sink routes to stderr while
 keeping the record visible by default.
 */
const moduleLogger = tagged({ tag: 'resolve_hosts', },);

/**
 Checks whether unknown value is object-like enough for property guards.
 
 @param value - Candidate JSON value.
 
 @returns Whether value supports property checks.
 
 @example
 ```ts
 isRecord({ hostnames: 'nginx.org' }); // true
 ```
 */
function isRecord(value: unknown,): value is UnknownRecord {
  return ((typeof value) === 'object')
    && (value !== null);
}

/**
 Checks whether unknown value is OpenTofu hostname query input.
 
 @param value - Candidate OpenTofu query value.
 
 @returns Whether value carries hostname text.
 
 @example
 ```ts
 isResolveHostsQuery({ hostnames: 'nginx.org' }); // true
 ```
 */
function isResolveHostsQuery(value: unknown,): value is ResolveHostsQuery {
  return isRecord(value,)
    && ((typeof value.hostnames) === 'string');
}

/**
 Parses OpenTofu query input or throws with context.
 
 @param value - Candidate OpenTofu query value.
 
 @returns Validated hostname query.
 
 @throws When value lacks hostname text.
 
 @example
 ```ts
 parseResolveHostsQuery({ hostnames: 'nginx.org' });
 ```
 */
function parseResolveHostsQuery(value: unknown,): ResolveHostsQuery {
  if (isResolveHostsQuery(value,))
    return value;

  throw new Error('OpenTofu external query must include string hostnames',);
}

/**
 Checks whether unknown value is an array of strings.
 
 @param value - Candidate parsed JSON value.
 
 @returns Whether value is a string array.
 
 @example
 ```ts
 isStringArray(['3.147.99.198/32']); // true
 ```
 */
function isStringArray(value: unknown,): value is readonly string[] {
  return Array.isArray(value,)
    && value.every(function isStringElement(element,): element is string {
      return (typeof element) === 'string';
    },);
}

/**
 Validates and normalizes one configured hostname.
 
 @param hostname - Candidate hostname from `local.resolvable_hostnames`.
 
 @returns Hostname accepted for DNS resolution.
 
 @throws When hostname is a wildcard or empty, since neither resolves to a concrete address.
 
 @example
 ```ts
 assertHostname('nginx.org'); // 'nginx.org'
 ```
 */
function assertHostname(hostname: string,): string {
  if (hostname === '')
    throw new Error('Cannot resolve an empty hostname',);

  if (hostname.includes('*',))
    throw new Error(`Wildcard hostnames cannot be resolved; configure concrete hostnames: ${hostname}`,);

  return hostname;
}

/**
 Narrows an unknown caught value to {@link NodeJS.ErrnoException} so callers can
 branch on `error.code` without an unsafe `as` assertion (oxlint bans that cast).
 
 @param error - Caught value, which `try` lifts to `unknown`.
 
 @returns Whether value is an {@link Error} carrying a `code` property.
 
 @example
 ```ts
 if (isErrnoException(error,) && (error.code === 'ENOENT')) return ABSENT;
 ```
 */
function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error);
}

/**
 Sentinel for "the requested file does not exist". A unique `Symbol` rather than
 `null`/`undefined` so the absent case stays out of a `no-nullish-union`-banned union.
 */
const ABSENT: unique symbol = Symbol('cidr map file missing on disk',);

/**
 Reads a file's UTF-8 contents, collapsing a missing file to {@link ABSENT};
 every other read error propagates.
 
 @param path - Absolute path to read.
 
 @returns File contents, or {@link ABSENT} on `ENOENT`.
 
 @throws When the read fails for any reason other than a missing file.
 
 @example
 ```ts
 const text = await readTextIfExists('/abs/cache_resolved_hosts.json');
 ```
 */
async function readTextIfExists(path: string,): Promise<string | typeof ABSENT> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (isErrnoException(error,)
      && (error.code
        === 'ENOENT'))
      return ABSENT;
    throw error;
  }
}

/**
 Reads a hostname-to-CIDR map file, returning an empty map when absent or malformed.
 
 @param path - Absolute path to a seed or cache JSON file.
 
 @returns Parsed map, or empty map when the file is missing or fails validation.
 
 @example
 ```ts
 await loadCidrMap('/abs/seed_resolved_hosts.json');
 ```
 */
async function loadCidrMap(path: string,): Promise<CidrsByHost> {
  /**
   Raw file contents, or {@link ABSENT} when the seed/cache file does not exist yet.
   */
  const contents = await readTextIfExists(path,);
  if (contents === ABSENT)
    return {};

  /**
   Parsed file contents before runtime shape validation.
   */
  const parsed: unknown = JSON.parse(contents,);
  if (!isRecord(parsed,))
    return {};

  /**
   Validated hostname map rebuilt from JSON-owned entries so no caller-owned
   property access capability crosses this function boundary.
   */
  const cidrsByHost: CidrsByHost = {};
  for (const [hostname, value,] of Object.entries(parsed,)) {
    if (!isStringArray(value,))
      return {};
    cidrsByHost[hostname] = value;
  }
  return cidrsByHost;
}

/**
 Resolves one DNS address family, returning an empty list when the family has no records.
 
 @param hostname - Concrete service hostname.
 
 @param family - DNS address family to resolve.
 
 @returns Address records for the family, or empty list when the resolver reports none or fails.
 
 @example
 ```ts
 await resolveAddressFamily({ hostname: 'nginx.org', family: 'A' });
 ```
 */
async function resolveAddressFamily({
  hostname,
  family,
}: {
  readonly hostname: string;
  readonly family: AddressFamily;
},): Promise<readonly string[]> {
  try {
    if (family === 'A')
      return await resolve4(hostname,);

    return await resolve6(hostname,);
  }
  catch (error) {
    // A host with only A records throws on the AAAA lookup (and vice versa); that
    // absence is expected, so log the cause to stderr and treat the family as empty.
    tagged({
      l: moduleLogger,
      tag: resolveAddressFamily.name,
    },)
      .warn(`${family} lookup for ${hostname} failed: ${String(error,)}`,);
    // Drain the record here: a cold total-resolution failure throws straight after
    // this helper returns, and a record still buffered during sink verification
    // would never reach stderr.
    await moduleLogger.flush();
    return [];
  }
}

/**
 Resolves one hostname into `/32` and `/128` CIDRs, tolerating hosts with no records.
 
 @param hostname - Concrete service hostname.
 
 @returns Freshly resolved CIDRs, or empty list when the host resolves to nothing.
 
 @example
 ```ts
 await resolveHostCidrs('nginx.org');
 ```
 */
async function resolveHostCidrs(hostname: string,): Promise<readonly string[]> {
  /**
   IPv4 and IPv6 DNS records for the hostname.
   */
  const [
    ipv4Addresses,
    ipv6Addresses,
  ] = await Promise.all([
    resolveAddressFamily({
      hostname,
      family: 'A',
    },),
    resolveAddressFamily({
      hostname,
      family: 'AAAA',
    },),
  ],);

  /**
   IPv4 CIDR destinations for the hcloud firewall rule.
   */
  const ipv4Cidrs = ipv4Addresses.map(function ipv4ToCidr(address,): string {
    return `${address}/32`;
  },);

  /**
   IPv6 CIDR destinations for the hcloud firewall rule.
   */
  const ipv6Cidrs = ipv6Addresses.map(function ipv6ToCidr(address,): string {
    return `${address}/128`;
  },);

  return [
    ...ipv4Cidrs,
    ...ipv6Cidrs,
  ];
}

/**
 Unions CIDR lists while preserving first-seen order and dropping duplicates.
 
 @param lists - CIDR lists to merge, earliest first.
 
 @returns Deduplicated CIDR list keeping the earliest occurrence of each entry.
 
 @example
 ```ts
 unionCidrs([['1.1.1.1/32'], ['1.1.1.1/32', '2.2.2.2/32']]); // ['1.1.1.1/32', '2.2.2.2/32']
 ```
 */
function unionCidrs(lists: readonly (readonly string[])[],): readonly string[] {
  /**
   CIDRs accumulated through audited array iteration while preserving insertion order.
   */
  const cidrs = new Set<string>();
  for (const list of lists) {
    for (const cidr of list)
      cidrs.add(cidr,);
  }
  return [...cidrs,];
}

/**
 Raw OpenTofu `data.external` payload read from stdin.
 */
const rawInput: unknown = await json(process.stdin,);

/**
 Validated OpenTofu query payload.
 */
const input = parseResolveHostsQuery(rawInput,);

/**
 Concrete, deduplicated, validated hostnames to resolve.
 */
const hostnames = [
  ...new Set(input.hostnames
    .split(',',)
    .map(function normalizeHost(hostname,): string {
      return hostname
        .trim()
        .toLowerCase();
    },)
    .filter(function keepNonEmptyHost(hostname,): boolean {
      return hostname !== '';
    },),),
].map(function validateHostname(hostname,): string {
  return assertHostname(hostname,);
},);

/**
 Committed baseline of known-good CIDRs per hostname, unioned in so a cold checkout
 (no cache) and transiently moved hosts never lose previously-allowed addresses.
 */
const SEED_FILE = join(
  import.meta.dirname,
  'seed_resolved_hosts.json',
);

/**
 Local, gitignored accumulation of every CIDR ever resolved for the configured hostnames.
 */
const CACHE_FILE = join(
  import.meta.dirname,
  'cache_resolved_hosts.json',
);

/**
 Seed and prior-cache CIDR maps, both used as accumulation inputs.
 */
const [
  seed,
  cache,
] = await Promise.all([
  loadCidrMap(SEED_FILE,),
  loadCidrMap(CACHE_FILE,),
],);

/**
 Per-host accumulation entries, each unioning seed, prior cache, and fresh DNS.
 */
const accumulatedEntries = await Promise.all(hostnames.map(
  async function accumulateHost(hostname,): Promise<ResolvedEntry> {
    /**
     Freshly resolved CIDRs for this hostname on this run.
     */
    const fresh = await resolveHostCidrs(hostname,);

    return [
      hostname,
      unionCidrs([
        seed[hostname] ?? [],
        cache[hostname] ?? [],
        fresh,
      ],),
    ];
  },
),);

/**
 Per-host union of seed, prior cache, and freshly resolved CIDRs.
 */
const resolvedByHost: CidrsByHost = Object.fromEntries(accumulatedEntries,);

/**
 Total CIDRs across every configured hostname, used to detect a cold total-resolution failure.
 */
const totalCidrs = Object.values(resolvedByHost,)
  .flat()
  .length;

if ((hostnames.length > 0)
  && (totalCidrs === 0))
  throw new Error('resolution failed and no seed or cache fallback available',);

// Persist the accumulation only when hostnames were requested, so the offline smoke test
// (empty hostname list) cannot prune an otherwise-useful cache to an empty map.
if (hostnames.length > 0) {
  await writeFile(
    CACHE_FILE,
    JSON.stringify(resolvedByHost,),
  );
}

/**
 Joins CIDRs without passing caller-owned array identity to an unresolved method.
 
 @param cidrs - CIDR strings to join in source order.
 
 @returns Comma-separated CIDR text.
 
 @example
 ```ts
 joinCidrs(['192.0.2.1/32', '2001:db8::1/128']);
 ```
 */
function joinCidrs(cidrs: readonly string[],): string {
  /**
   Output text accumulated from primitive CIDR strings.
   */
  let result = '';
  for (const cidr of cidrs)
    result = result === '' ? cidr : `${result},${cidr}`;
  return result;
}

/**
 Per-host output entries, each pairing a hostname with its comma-joined CIDR string.
 */
const resultEntries = Object.entries(resolvedByHost,)
  .map(function joinHost([
    hostname,
    cidrs,
  ],): ResultEntry {
    return [
      hostname,
      joinCidrs(cidrs,),
    ];
  },);

/**
 Comma-joined CIDR map ready to stream out to OpenTofu's `map(string)` result.
 */
const result: ResultByHost = Object.fromEntries(resultEntries,);

process.stdout
  .write(JSON.stringify(result,),);
