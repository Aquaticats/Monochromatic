/**
 Reads and validates the clearnet host lists of one data file.

 The lists are data and not code, and both lists are read together:
 a file that installed one list without the other would remove the other list's rules as unwanted.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Module logger for reading the recorded host data.
 */
const l = tagged({ tag: 'clearnet-hosts-data-values', },);

/**
 Schema version this build reads.
 A file of another version is refused rather than guessed at,
 so a newer build cannot make this one install rules it does not understand.
 */
export const CLEARNET_HOSTS_DATA_SCHEMA_VERSION = 1;

/**
 Longest hostname a rule may name, in bytes.
 */
const MAX_HOST_LENGTH = 253;

/**
 Longest label of a hostname, in bytes.
 */
const MAX_HOST_LABEL_LENGTH = 63;

/**
 Fewest labels a hostname a rule may name has:
 one label names the search domain,
 which no rule can carry.
 */
const MIN_HOST_LABELS = 2;

/**
 Error raised when the clearnet host data of the installed build cannot be read as a list.

 The caller keeps the rules it already has,
 because a list that cannot be read says nothing about which hosts stay outside the tunnel.
 It carries a marker property instead of relying on `instanceof`,
 so it is recognized across separately built bundles.

 @example
 ```ts
 throw new ClearnetHostsDataError({ path: 'data.json', reason: 'is not JSON' });
 ```
 */
export class ClearnetHostsDataError extends Error {
  /**
   Stable error type name rendered by Node.
   */
  override name = 'ClearnetHostsDataError';

  /**
   Marker read by {@link isClearnetHostsDataError}.
   */
  readonly clearnetHostsData = true as const;

  /**
   Path of the data that could not be read.
   */
  readonly path: string;

  /**
   Builds the failure that names the file and what is wrong with it.

   @param path - Path of the data file.

   @param reason - What the file fails,
   phrased to follow the path.

   @example
   ```ts
   new ClearnetHostsDataError({ path: 'data.json', reason: 'is not JSON' });
   ```
   */
  public constructor(
    {
      path,
      reason,
    }: {
      readonly path: string;
      readonly reason: string;
    },
  ) {
    super(`the clearnet host list at ${path} ${reason}`,);
    this.path = path;
  }
}

/**
 Tells whether a caught value is the error of clearnet host data that cannot be read.

 @param error - Caught value.

 @returns Whether it carries the marker.

 @example
 ```ts
 isClearnetHostsDataError({ error: new ClearnetHostsDataError({ path: 'x', reason: 'y' }) }); // true
 ```
 */
export function isClearnetHostsDataError({ error, }: { readonly error: unknown; },): boolean {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('clearnetHostsData' in error)
    && (error.clearnetHostsData === true);
}

/**
 Reports whether a parsed JSON value is a non-null object.

 @param value - Parsed JSON.

 @returns Whether the value has named fields.

 @example
 ```ts
 isFieldRecord({ schemaVersion: 1 });
 ```
 */
function isFieldRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reports whether a text is a bare lowercase hostname a rule may name:
 at least two labels of letters,
 digits and inner dashes,
 each within the DNS length limits.

 A single label is refused because it names a search domain and not a host,
 and every other character is refused because a rule carries a name and not a scheme,
 port or path.

 @param host - Candidate host.

 @returns Whether the text can be a host rule's DNS name.

 @example
 ```ts
 isBareHostname({ host: 'api.deepseek.com' }); // true
 ```
 */
export function isBareHostname({ host, }: { readonly host: string; },): boolean {
  if ((host.length === 0) || (host.length > MAX_HOST_LENGTH))
    return false;
  /**
   Labels the name is made of.
   */
  const labels = host
    .split('.',);
  if (labels.length < MIN_HOST_LABELS)
    return false;
  for (const label of labels) {
    if ((label.length === 0) || (label.length > MAX_HOST_LABEL_LENGTH))
      return false;
    if (label.startsWith('-',) || label.endsWith('-',))
      return false;
    for (const character of label) {
      if (!(((character >= 'a') && (character <= 'z'))
        || ((character >= '0') && (character <= '9'))
        || (character === '-')))
        return false;
    }
  }
  return true;
}

/**
 Reads the host of one recorded provider base URL.

 @param baseUrl - Base URL as the data file records it.

 @returns Host the URL names,
 or the empty string when the URL is not an https URL with a bare host.

 @example
 ```ts
 providerHost({ baseUrl: 'https://api.deepseek.com/v1' }); // 'api.deepseek.com'
 ```
 */
export function providerHost({ baseUrl, }: { readonly baseUrl: string; },): string {
  /**
   Parsed URL,
   or nothing when the text is not a URL.
   */
  const parsed = parsedUrlText({ baseUrl, },);
  if (parsed === PARSE_ABSENT)
    return '';
  if (parsed.protocol !== 'https:')
    return '';
  if (!isBareHostname({ host: parsed.hostname, },))
    return '';
  return parsed.hostname;
}

/**
 Sentinel representing a text that is not a URL.
 */
const PARSE_ABSENT: unique symbol = Symbol('the text is not a URL',);

/**
 Parses a recorded base URL.

 @param baseUrl - Base URL as the data file records it.

 @returns The parsed URL,
 or the absence sentinel.

 @example
 ```ts
 parsedUrlText({ baseUrl: 'https://api.deepseek.com' });
 ```
 */
function parsedUrlText({ baseUrl, }: { readonly baseUrl: string; },): URL | typeof PARSE_ABSENT {
  try {
    return new URL(baseUrl,);
  }
  catch (error) {
    l.debug(`not a URL: ${JSON.stringify(baseUrl,)}: ${String(error,)}`,);
    return PARSE_ABSENT;
  }
}

/**
 Reads the strings of one recorded host list.

 @param value - Parsed field.

 @param path - Data path named by a failure.

 @param field - Field name named by a failure.

 @returns The hosts.

 @throws {@link ClearnetHostsDataError} when the field is not a list of bare hostnames.

 @example
 ```ts
 hostList({ value: ['crates.io'], path: 'data.json', field: 'updateHosts.plan' });
 ```
 */
function hostList(
  {
    value,
    path,
    field,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly field: string;
  },
): readonly string[] {
  if (!Array.isArray(value,)) {
    throw new ClearnetHostsDataError({
      path,
      reason: `has no list of hosts at ${field}`,
    },);
  }
  return value.map(function checkedHost(
    entry: unknown,
    index: number,
  ): string {
    if ((typeof entry) !== 'string') {
      throw new ClearnetHostsDataError({
        path,
        reason: `names a host at ${field}[${String(index,)}] that is not a string`,
      },);
    }
    if (!isBareHostname({ host: entry, },)) {
      throw new ClearnetHostsDataError({
        path,
        reason: `names a host at ${field}[${String(index,)}] that is not a bare lowercase hostname: ${
           JSON.stringify(entry,)}`,
      },);
    }
    return entry;
  },);
}

/**
 Reads the host of one recorded provider entry.

 @param entry - Parsed provider entry.

 @param path - Data path named by a failure.

 @param index - Entry position named by a failure.

 @returns Host the entry's base URL names.

 @throws {@link ClearnetHostsDataError} when the entry is not a provider record or its URL is unusable.

 @example
 ```ts
 providerHostAt({ entry: { provider: 'deepseek', baseUrl: 'https://api.deepseek.com' }, path: 'data.json', index: 0 });
 ```
 */
function providerHostAt(
  {
    entry,
    path,
    index,
  }: {
    readonly entry: unknown;
    readonly path: string;
    readonly index: number;
  },
): string {
  if (!isFieldRecord(entry,)) {
    throw new ClearnetHostsDataError({
      path,
      reason: `names a provider at providerBaseUrls[${String(index,)}] without a base URL`,
    },);
  }
  /**
   Recorded provider id and base URL.
   */
  const {
    provider,
    baseUrl,
  } = entry;
  if (((typeof provider) !== 'string') || (provider === '')) {
    throw new ClearnetHostsDataError({
      path,
      reason: `names a provider at providerBaseUrls[${String(index,)}] without an id`,
    },);
  }
  if ((typeof baseUrl) !== 'string') {
    throw new ClearnetHostsDataError({
      path,
      reason: `names a provider at providerBaseUrls[${String(index,)}] without a base URL`,
    },);
  }
  /**
   Host its requests are sent to.
   */
  const host = providerHost({ baseUrl, },);
  if (host === '') {
    throw new ClearnetHostsDataError({
      path,
      reason: `names a provider at providerBaseUrls[${String(index,)}] whose base URL is not an https URL `
        + `with a bare host: ${JSON.stringify(baseUrl,)}`,
    },);
  }
  return host;
}

/**
 Reads the host lists of one data value,
 deriving the model provider hosts from the recorded base URLs.

 Both lists are read together and the whole value is refused when any part of it is wrong,
 because a refresh that installed only one list would remove the other list's rules as unwanted.

 @param value - Parsed data file contents.

 @param path - Path of the data file, named by every failure.

 @returns Every host the file names,
 once each,
 update hosts before provider hosts.

 @throws {@link ClearnetHostsDataError} when the value is not a data file this build reads.

 @example
 ```ts
 clearnetHostsFromData({ value: { schemaVersion: 1, updateHosts: { plan: ['crates.io'], redirect: [], nonFetch: [] }, providerBaseUrls: [] }, path: 'data.json' });
 ```
 */
export function clearnetHostsFromData(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly string[] {
  if (!isFieldRecord(value,)) {
    throw new ClearnetHostsDataError({
      path,
      reason: 'is not an object',
    },);
  }
  if (value.schemaVersion !== CLEARNET_HOSTS_DATA_SCHEMA_VERSION) {
    throw new ClearnetHostsDataError({
      path,
      reason: `has schema version ${JSON.stringify(value.schemaVersion,)}, `
        + `and this build reads only version ${String(CLEARNET_HOSTS_DATA_SCHEMA_VERSION,)}`,
    },);
  }
  /**
   Recorded update-host lists and provider base URLs.
   */
  const {
    updateHosts,
    providerBaseUrls,
  } = value;
  if (!isFieldRecord(updateHosts,)) {
    throw new ClearnetHostsDataError({
      path,
      reason: 'has no update-host lists',
    },);
  }
  /**
   Documentation-only hosts of the update sources,
   which never get a rule and are checked so a damaged file is refused as a whole.
   */
  hostList({
    value: updateHosts.nonFetch,
    path,
    field: 'updateHosts.nonFetch',
  },);
  /**
   Update hosts the plan names,
   of which a file names at least one,
   because a list that names none would remove every update-host rule the tunnel has.
   */
  const planHosts = hostList({
    value: updateHosts.plan,
    path,
    field: 'updateHosts.plan',
  },);
  if (planHosts.length === 0) {
    throw new ClearnetHostsDataError({
      path,
      reason: 'names no update host at updateHosts.plan, '
        + 'and a list that names none would remove every update-host rule the tunnel has',
    },);
  }
  /**
   Hosts the plan never names but whose downloads reach them.
   */
  const redirectHosts = hostList({
    value: updateHosts.redirect,
    path,
    field: 'updateHosts.redirect',
  },);
  if (!Array.isArray(providerBaseUrls,)) {
    throw new ClearnetHostsDataError({
      path,
      reason: 'has no list of provider base URLs',
    },);
  }
  /**
   Hosts of the model providers,
   derived as each provider's requests reach them.
   */
  const providerHosts = providerBaseUrls.map(function mappedProvider(
    entry: unknown,
    index: number,
  ): string {
    return providerHostAt({
      entry,
      path,
      index,
    },);
  },);
  /**
   Every host the file names,
   once each,
   with the update hosts first because one refresh installs both lists together.
   */
  const hosts = [
    ...planHosts,
    ...redirectHosts,
    ...providerHosts,
  ];
  return [...new Set(hosts,),];
}

/**
 Parses one data file's text.

 @param text - File text.

 @param path - Path of the data file, named by every failure.

 @returns The parsed value.

 @throws {@link ClearnetHostsDataError} when the text is not JSON.

 @example
 ```ts
 parsedJsonText({ text: '{}', path: 'data.json' });
 ```
 */
function parsedJsonText(
  {
    text,
    path,
  }: {
    readonly text: string;
    readonly path: string;
  },
): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error) {
    throw new ClearnetHostsDataError({
      path,
      reason: `is not JSON: ${String(error,)}`,
    },);
  }
}

/**
 Reads one data file's text as the clearnet host list.

 @param text - File text.

 @param path - Path of the data file, named by every failure.

 @returns Every host the file names.

 @throws {@link ClearnetHostsDataError} when the text is not a data file this build reads.

 @example
 ```ts
 parseClearnetHostsData({ text: '{"schemaVersion":1}', path: 'data.json' });
 ```
 */
export function parseClearnetHostsData(
  {
    text,
    path,
  }: {
    readonly text: string;
    readonly path: string;
  },
): readonly string[] {
  return clearnetHostsFromData({
    value: parsedJsonText({
      text,
      path,
    },),
    path,
  },);
}
