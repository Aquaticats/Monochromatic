import {
  credentialsOfHeaders,
  maskCredentials,
  type ModelTransport,
  type TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Provider listing
// Reads one provider's model listing over the transport the caller hands in,
// as parsed JSON, and refuses a listing that did not come back whole.
//
// A CREDENTIAL THE REQUEST CARRIED NEVER LEAVES THIS READ IN A BODY. A provider
// may echo the key it was sent, an authentication failure's body being the
// plausible place, and both listing commands print what they read. The mask is
// applied here, to the text, before it is parsed, so no caller can parse,
// print, store or throw an unmasked body: the transport the caller hands in
// may be the live one, which masks too, or any other.

/**
 How long one listing may take before it is abandoned, in milliseconds.

 A CHOICE, NOT A MEASUREMENT. The reads this replaced waited as long as the
 provider cared to, and the transport's own idle guard ends a stalled stream;
 this bounds a provider that answers slowly all the same, at the freshness
 window the budget meters use.
 */
export const LISTING_TIMEOUT_MS = 60_000;

/**
 Lowest status that answers a request with a listing.
 */
const FIRST_SUCCESS = 200;

/**
 Highest status that answers a request with a listing.
 */
const LAST_SUCCESS = 299;

/**
 Reads a provider's listing.

 @param url - absolute listing URL

 @param apiKey - key sent as a bearer token, never printed

 @param transport - transport the request goes over: `fetchTransport` in a run

 @param timeoutMs - how long the request may run before it is abandoned

 @param statusRefusal - sentence the command refuses with when the provider
 answers with a status outside 200 to 299, built from the status alone since
 the reason phrase and the body are the provider's wording

 @returns The listing as parsed JSON, read from a body with every credential
 the request carried masked out of it

 @throws {@link StatedRefusalError} When the provider answers with a status
 outside 200 to 299, or with a body that is not JSON (neither refusal repeats
 the body)

 @example
 ```ts
 const body = await readProviderListing({
   url,
   apiKey,
   transport: fetchTransport,
   timeoutMs: LISTING_TIMEOUT_MS,
   statusRefusal: function says({ status, }): string { return `${url} answered ${String(status,)}`; },
 },);
 ```
 */
export async function readProviderListing(
  {
    url,
    apiKey,
    transport,
    timeoutMs,
    statusRefusal,
  }: {
    readonly url: string;
    readonly apiKey: string;
    readonly transport: ModelTransport;
    readonly timeoutMs: number;
    readonly statusRefusal: (input: { readonly status: number; },) => string;
  },
): Promise<unknown> {
  /**
   Request headers, which name the credential the reply body must not repeat.
   */
  const headers = { authorization: `Bearer ${apiKey}`, };

  /**
   What the provider answered.
   */
  const reply: TransportReply = await transport({
    url,
    label: url,
    method: 'GET',
    headers,
    signal: AbortSignal.timeout(timeoutMs,),
  },);
  if ((reply.status < FIRST_SUCCESS) || (reply.status > LAST_SUCCESS))
    throw new StatedRefusalError({ says: statusRefusal({ status: reply.status, },), },);

  /**
   The body with every credential of the request masked, before anything reads it.
   */
  const bodyText = maskCredentials({
    text: reply.bodyText,
    credentials: credentialsOfHeaders({ headers, },),
  },);
  try {
    return JSON.parse(bodyText,);
  } catch (error) {
    // The parse failure's own words quote the text it refused, so they are
    // kept as the cause and never repeated.
    throw new StatedRefusalError({
      says: `${url} answered with a body that is not JSON; the body is the provider's wording and is not repeated`,
      cause: error,
    },);
  }
}

//endregion Provider listing
