import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  AsnDatabaseError,
  validateNetwork,
} from './asn-network.ts';

/**
 Minimal IPinfo Lite record shape consumed by ASN filtering.
 */
type IpinfoLiteRecord = {
  readonly asn: string;
  readonly network: string;
};

/**
 Unknown JSON object shape used before field guards run.
 */
type UnknownRecord = Record<PropertyKey, unknown>;

/**
 Sentinel returned when NDJSON line has no matching network.
 */
const NO_MATCHING_NETWORK = Symbol('IPinfo line has no matching network',);

/**
 Type of {@link NO_MATCHING_NETWORK}.
 */
type NoMatchingNetwork = typeof NO_MATCHING_NETWORK;

/**
 IPinfo Lite database download endpoint.
 */
const IPINFO_LITE_URL = 'https://ipinfo.io/data/ipinfo_lite.json.gz';

/**
 Module logger for streamed IPinfo access.
 */
const l = tagged({ tag: 'asn-fetch', },);

/**
 Checks whether unknown value supports property guards.
 
 @param value - Candidate JSON value.
 
 @returns Whether value is object-like.
 
 @example
 ```ts
 isRecord({ asn: 'AS64500' }); // true
 ```
 */
function isRecord(value: unknown,): value is UnknownRecord {
  return ((typeof value) === 'object')
    && (value !== null);
}

/**
 Checks whether unknown value carries consumed IPinfo fields.
 
 @param value - Candidate parsed record.
 
 @returns Whether value has ASN and network text.
 
 @example
 ```ts
 isIpinfoLiteRecord({ asn: 'AS64500', network: '192.0.2.0/24' }); // true
 ```
 */
function isIpinfoLiteRecord(value: unknown,): value is IpinfoLiteRecord {
  return isRecord(value,)
    && ((typeof value.asn) === 'string')
    && ((typeof value.network) === 'string');
}

/**
 Parses one IPinfo NDJSON line when ASN matches.
 
 @param line - NDJSON line.
 
 @param targetAsn - Normalized ASN to retain.
 
 @returns Matching validated network or non-match sentinel.
 
 @example
 ```ts
 parseMatchingNetwork({
   line: '{"asn":"AS64500","network":"192.0.2.0/24"}',
   targetAsn: 'AS64500',
 });
 ```
 */
function parseMatchingNetwork(
  {
    line,
    targetAsn,
  }: {
    readonly line: string;
    readonly targetAsn: string;
  },
): string | NoMatchingNetwork {
  if (!line.includes(targetAsn,))
    return NO_MATCHING_NETWORK;
  /**
   Parsed NDJSON value before runtime shape validation.
   */
  const entry: unknown = JSON.parse(line,);
  if (!isIpinfoLiteRecord(entry,)) {
    if (isRecord(entry,) && (entry.asn === targetAsn)) {
      throw new AsnDatabaseError(
        `IPinfo Lite record for ${targetAsn} lacks network text.`,
      );
    }
    return NO_MATCHING_NETWORK;
  }
  if (entry.asn !== targetAsn)
    return NO_MATCHING_NETWORK;
  return validateNetwork({
    network: entry.network,
    targetAsn,
  },);
}

/**
 Creates authenticated IPinfo Lite database URL.
 
 @param token - IPinfo access token.
 
 @returns Database URL carrying encoded token parameter.
 
 @throws {@link AsnDatabaseError} when token is empty.
 
 @example
 ```ts
 databaseUrl('token-value');
 ```
 */
function databaseUrl(token: string,): URL {
  if (token === '')
    throw new AsnDatabaseError('IPINFO_TOKEN is required to refresh ASN network caches.',);
  /**
   URL object encoding query-component syntax.
   */
  const url = new URL(IPINFO_LITE_URL,);
  url
    .searchParams
    .set(
    'token',
    token,
    );
  return url;
}

/**
 Collects matching networks from complete decoded NDJSON lines.
 
 @param lines - Complete lines to inspect.
 
 @param targetAsn - Normalized ASN to retain.
 
 @param networks - Matching network accumulator.
 
 @example
 ```ts
 collectLines({ lines: [], targetAsn: 'AS64500', networks: [] });
 ```
 */
function collectLines(
  {
    lines,
    targetAsn,
    networks,
  }: {
    readonly lines: readonly string[];
    readonly targetAsn: string;
    readonly networks: string[];
  },
): void {
  for (const line of lines) {
    /**
     Network when line belongs to target ASN.
     */
    const network = parseMatchingNetwork({
      line,
      targetAsn,
    },);
    if (network !== NO_MATCHING_NETWORK)
      networks.push(network,);
  }
}

/**
 Streams IPinfo Lite and collects networks assigned to one ASN.
 
 @param targetAsn - Normalized ASN to retain.
 
 @param token - IPinfo database token.
 
 @returns Matching network strings in database order.
 
 @throws {@link AsnDatabaseError} for rejected responses and absent bodies.
 
 @example
 ```ts
 await fetchAsnNetworks({ targetAsn: 'AS64500', token: 'token-value' });
 ```
 */
export async function fetchAsnNetworks(
  {
    targetAsn,
    token,
  }: {
    readonly targetAsn: string;
    readonly token: string;
  },
): Promise<readonly string[]> {
  /**
   Function-scoped logger for one streamed database request.
   */
  const fl = tagged({
    tag: fetchAsnNetworks.name,
    l,
  },);
  fl.debug(`requesting IPinfo Lite database for ${targetAsn}`,);
  /**
   HTTP response carrying gzip-encoded NDJSON.
   */
  const response = await fetch(databaseUrl(token,),);
  if (!response.ok) {
    throw new AsnDatabaseError(
      `IPinfo Lite database request failed with HTTP ${String(response.status,)}.`,
    );
  }
  /**
   Nullable response body checked before decompression.
   */
  const { body, } = response;
  if (body === null)
    throw new AsnDatabaseError('IPinfo Lite database response did not include a body.',);
  /**
   Pull reader over decompressed NDJSON bytes.
   */
  const reader = body
    .pipeThrough(new DecompressionStream('gzip',),)
    .getReader();
  /**
   Decoder retained between chunks so split codepoints remain intact.
   */
  const decoder = new TextDecoder();
  /**
   Matching network accumulator.
   */
  const networks: string[] = [];
  /**
   Partial trailing line carried between chunks.
   */
  const textState = { leftover: '', };
  /**
   Current sequential stream result.
   */
  const readState = { current: await reader.read(), };
  while (!(readState.current
    .done)) {
    /**
     Complete decoded text available for splitting.
     */
    const chunk = textState.leftover + decoder.decode(
      readState.current
        .value,
      { stream: true, },
    );
    /**
     Complete lines plus trailing partial line.
     */
    const lines = chunk.split('\n',);
    textState.leftover = lines.pop() ?? '';
    collectLines({
      lines,
      targetAsn,
      networks,
    },);
    // oxlint-disable-next-line eslint/no-await-in-loop -- ReadableStream readers are sequential pull sources.
    readState.current = await reader.read();
  }
  /**
   Final decoder flush and unterminated trailing line.
   */
  const finalLine = textState.leftover + decoder.decode();
  if (finalLine !== '') {
    collectLines({
      lines: [finalLine,],
      targetAsn,
      networks,
    },);
  }
  fl.debug(`IPinfo Lite returned ${String(networks.length,)} network(s) for ${targetAsn}`,);
  return networks;
}
