import { json, } from 'node:stream/consumers';

import { lookupAsnNetworks, } from '@monochromatic-dev/wg-core-allowedips/ts/asn-networks.ts';

/**
 OpenTofu `data.external` query carrying ASN text.
 
 @example
 ```ts
 const query: ExternalAsnQuery = { asn: 'AS41231' };
 ```
 */
type ExternalAsnQuery = {
  readonly asn: string;
};

/**
 Unknown JSON object shape used before field validation.
 
 @example
 ```ts
 const object: UnknownRecord = { asn: 'AS41231' };
 ```
 */
type UnknownRecord = Record<PropertyKey, unknown>;

/**
 Checks whether unknown value is object-like.
 
 @param value - Candidate JSON value.
 
 @returns Whether value supports property checks.
 
 @example
 ```ts
 isRecord({ asn: 'AS41231' }); // true
 ```
 */
function isRecord(value: unknown,): value is UnknownRecord {
  return ((typeof value) === 'object')
    && (value !== null);
}

/**
 Checks whether unknown value is OpenTofu ASN query input.
 
 @param value - Candidate OpenTofu query value.
 
 @returns Whether value carries ASN text.
 
 @example
 ```ts
 isExternalAsnQuery({ asn: 'AS41231' }); // true
 ```
 */
function isExternalAsnQuery(value: unknown,): value is ExternalAsnQuery {
  return isRecord(value,)
    && ((typeof value.asn) === 'string');
}

/**
 Parses OpenTofu query input or throws with context.
 
 @param value - Candidate OpenTofu query value.
 
 @returns Validated ASN query.
 
 @throws When value lacks ASN text.
 
 @example
 ```ts
 parseExternalAsnQuery({ asn: 'AS41231' });
 ```
 */
function parseExternalAsnQuery(value: unknown,): ExternalAsnQuery {
  if (isExternalAsnQuery(value,))
    return value;
  throw new Error('OpenTofu external query must include string asn',);
}

/**
 Raw OpenTofu `data.external` payload read from stdin.
 */
const rawInput: unknown = await json(process.stdin,);

/**
 Validated OpenTofu query payload.
 */
const input = parseExternalAsnQuery(rawInput,);

/**
 Token inherited from OpenTofu execution environment, or empty when only cache fallback is available.
 */
const { IPINFO_TOKEN = '', } = process.env;

/**
 Networks resolved through shared IPinfo Lite cache and streaming implementation.
 */
const networks = await lookupAsnNetworks({
  asn: input.asn,
  cacheDirectory: import.meta.dirname,
  token: IPINFO_TOKEN,
},);

process
  .stdout
  .write(JSON.stringify({ ips: networks.join(',',), },),);
