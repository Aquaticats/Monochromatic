import {
  resolve4,
  resolve6,
} from 'node:dns/promises';
import { json, } from 'node:stream/consumers';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 OpenTofu `data.external` query carrying comma-separated Storage Box hostnames.
 
 @example
 ```ts
 const query: StorageboxHostsQuery = { hostnames: 'u123456.your-storagebox.de' };
 ```
 */
type StorageboxHostsQuery = {
  readonly hostnames: string;
};

/**
 DNS record family supported by Storage Box hostname resolution.
 
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
 const object: UnknownRecord = { hostnames: 'u123456.your-storagebox.de' };
 ```
 */
type UnknownRecord = Record<PropertyKey, unknown>;

/**
 Logger root for this script's diagnostics, tagged with the script's own name.
 OpenTofu reads this script's stdout as external-data JSON, so records stay at
 `warn` or above: that is the only level the console sink routes to stderr while
 keeping the record visible by default.
 */
const moduleLogger = tagged({ tag: 'resolve_storagebox_hosts', },);

/**
 Checks whether unknown value is object-like enough for property guards.
 
 @param value - Candidate JSON value.
 
 @returns Whether value supports property checks.
 
 @example
 ```ts
 isRecord({ hostnames: 'u123456.your-storagebox.de' }); // true
 ```
 */
function isRecord(value: unknown,): value is UnknownRecord {
  return ((typeof value) === 'object')
    && (value !== null);
}

/**
 Checks whether unknown value is OpenTofu Storage Box hostname query input.
 
 @param value - Candidate OpenTofu query value.
 
 @returns Whether value carries hostname text.
 
 @example
 ```ts
 isStorageboxHostsQuery({ hostnames: 'u123456.your-storagebox.de' }); // true
 ```
 */
function isStorageboxHostsQuery(value: unknown,): value is StorageboxHostsQuery {
  return isRecord(value,)
    && ((typeof value.hostnames) === 'string');
}

/**
 Parses OpenTofu query input or throws with context.
 
 @param value - Candidate OpenTofu query value.
 
 @returns Validated Storage Box hostname query.
 
 @throws When value lacks hostname text.
 
 @example
 ```ts
 parseStorageboxHostsQuery({ hostnames: 'u123456.your-storagebox.de' });
 ```
 */
function parseStorageboxHostsQuery(value: unknown,): StorageboxHostsQuery {
  if (isStorageboxHostsQuery(value,))
    return value;

  throw new Error('OpenTofu external query must include string hostnames',);
}

/**
 Validates and normalizes one configured Storage Box hostname.
 
 @param hostname - Candidate hostname from tfvars.
 
 @returns Hostname accepted for DNS resolution.
 
 @throws When hostname is wildcard or outside `your-storagebox.de`.
 
 @example
 ```ts
 assertStorageboxHostname('u123456.your-storagebox.de');
 ```
 */
function assertStorageboxHostname(hostname: string,): string {
  if (hostname.includes('*',))
    throw new Error('Storage Box wildcard hostnames cannot be resolved; configure concrete hostnames',);

  if (!hostname.endsWith('.your-storagebox.de',))
    throw new Error(`Storage Box hostname must end with .your-storagebox.de: ${hostname}`,);

  return hostname;
}

/**
 Resolves one DNS address family, returning empty list when family has no records.
 
 @param hostname - Concrete Storage Box hostname.
 
 @param family - DNS address family to resolve.
 
 @returns Address records for family, or empty list when resolver reports none.
 
 @example
 ```ts
 await resolveAddressFamily({ hostname: 'u123456.your-storagebox.de', family: 'A' });
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
    // Drain the record here: a hostname that resolves to nothing throws straight
    // after this helper returns, and a record still buffered during sink
    // verification would never reach stderr.
    await moduleLogger.flush();
    return [];
  }
}

/**
 Resolves one Storage Box hostname into `/32` and `/128` CIDRs, after
 validating it with {@link assertStorageboxHostname}.
 
 @param hostname - Concrete Storage Box hostname.
 
 @returns CIDRs suitable for hcloud `destination_ips`.
 
 @throws When hostname resolves to no A or AAAA records.
 
 @example
 ```ts
 await resolveStorageboxHostname('u123456.your-storagebox.de');
 ```
 */
async function resolveStorageboxHostname(hostname: string,): Promise<readonly string[]> {
  /**
   Hostname after suffix and wildcard validation.
   */
  const storageboxHostname = assertStorageboxHostname(hostname,);

  /**
   IPv4 and IPv6 DNS records for hostname.
   */
  const [
    ipv4Addresses,
    ipv6Addresses,
  ] = await Promise.all([
    resolveAddressFamily({
      hostname: storageboxHostname,
      family: 'A',
    },),
    resolveAddressFamily({
      hostname: storageboxHostname,
      family: 'AAAA',
    },),
  ],);

  if ((ipv4Addresses.length === 0)
    && (ipv6Addresses.length === 0))
    throw new Error(`Storage Box hostname resolved to no A or AAAA records: ${storageboxHostname}`,);

  /**
   IPv4 CIDR destinations for hcloud firewall rule.
   */
  const ipv4Cidrs = ipv4Addresses.map(function ipv4ToCidr(address,): string {
    return `${address}/32`;
  },);

  /**
   IPv6 CIDR destinations for hcloud firewall rule.
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
 Raw OpenTofu `data.external` payload read from stdin.
 */
const rawInput: unknown = await json(process.stdin,);

/**
 Validated OpenTofu query payload.
 */
const input = parseStorageboxHostsQuery(rawInput,);

/**
 Concrete Storage Box hostnames to resolve.
 */
const hostnames = input.hostnames
  .split(',',)
  .map(function normalizeHost(hostname,): string {
    return hostname
      .trim()
      .toLowerCase();
  },)
  .filter(function keepNonEmptyHost(hostname,): boolean {
    return hostname !== '';
  },);

/**
 Resolves all configured Storage Box hostnames and writes comma-separated CIDRs.
 */
const cidrLists = await Promise.all(hostnames.map(function resolveHostname(hostname,): Promise<readonly string[]> {
  return resolveStorageboxHostname(hostname,);
},),);

/**
 Comma-joined CIDR list ready to stream out to OpenTofu.
 */
const result = cidrLists
  .flat()
  .join(',',);

process.stdout
  .write(JSON.stringify({ ips: result, },),);
