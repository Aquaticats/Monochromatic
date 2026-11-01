import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { LookupHit, } from './lookup-cache.ts';
import { readMaskedJsonBody, } from '@monochromatic-dev/translation-repair-provider-stream/ts';

//region Work-title search
// One call to the Exa search endpoint for one title.
//
// NO SDK. The workspace bans `exa-js`; this is one `fetch` against `/search`,
// the request shape read off the reference on 2026-09-02
// (https://exa.ai/docs/reference/search): POST, `x-api-key` header, `query`,
// `type`, `numResults`, `contents.highlights` with its own `query` and
// `maxCharacters`; the response carries `results[]` with `title`, `url` and
// `highlights[]`. Exercised by hand the same day: 《活着》 answered "To Live",
// 《魔法少女小圆》 "Puella Magi Madoka Magica", about 1.5 s and $0.007 a query.

/**
 Environment variable carrying the Exa API key, injected by mise from the
 encrypted secrets file. Never logged, never written.
 */
export const EXA_API_KEY_VAR = 'TRANSLATION_REPAIR_EXA_API_KEY';

/**
 Where the search endpoint lives.
 */
export const EXA_SEARCH_URL = 'https://api.exa.ai/search';

/**
 Results asked for per title: enough to show an official edition beside a
 fan rendering, few enough to read.
 */
export const RESULTS_PER_TITLE = 5;

/**
 Longest highlight carried per result, in characters.
 */
export const HIGHLIGHT_CHARACTERS = 300;

/**
 Raised when the search endpoint refuses or answers in a shape the reader
 cannot use.

 NAMES THE HTTP STATUS OR THE CHECK THAT FAILED AND NOTHING THE ENDPOINT
 WROTE: the sentence is built here from a phrase each throw site authors, so
 a log line repeats it whole and an operator reads the status, where an
 endpoint's body (which can quote a key's owner or a page) never reaches one.

 @example
 ```ts
 throw new WorkTitleLookupError({ detail: 'responded 401', },);
 ```
 */
export class WorkTitleLookupError extends Error {
  /**
   Declares this message safe to forward: the status or the check that
   failed, never a body, a query or the key.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal.

   @param detail - phrase the throw site authors, an HTTP status, the check the answer failed or the network failing, never text read from the answer

   @param cause - failure the transport raised, kept for whoever inspects the refusal and never repeated in the message

   @example
   ```ts
   throw new WorkTitleLookupError({ detail: 'responded 401', },);
   ```
   */
  constructor({
    detail,
    cause,
  }: {
    readonly detail: string;
    readonly cause?: unknown;
  },) {
    super(
      `search ${detail}`,
      // Conditional spread keeps cause absent when none was supplied.
      ...((cause === undefined) ? [] : [{ cause, },]),
    );
    this.name = 'WorkTitleLookupError';
  }
}

/**
 One search result as the endpoint returns it, the fields read here.
 */
type SearchResultWire = {
  readonly title?: unknown;
  readonly url?: unknown;
  readonly highlights?: unknown;
};

/**
 Hits one wire result yields: one when it carries a url, none otherwise.

 @param value - element of the response's results

 @returns Zero or one hit, so callers flatten instead of filtering absence

 @example
 ```ts
 hitsOf({ value: { title: 'To Live', url: 'https://x', highlights: ['...'], }, },);
 ```
 */
export function hitsOf(
  { value, }: { readonly value: unknown; },
): readonly LookupHit[] {
  if (!isJsonRecord(value,))
    return [];
  /**
   Fields read.
   */
  const wire = value as SearchResultWire;
  if ((typeof wire.url) !== 'string')
    return [];
  /**
   Highlights as unknowns when the endpoint returned any.
   */
  const highlights: readonly unknown[] = Array.isArray(wire.highlights,) ? wire.highlights : [];
  /**
   First highlight when it is text.
   */
  const [first,] = highlights;
  return [{
    title: ((typeof wire.title) === 'string') ? wire.title : '',
    url: wire.url,
    highlight: ((typeof first) === 'string') ? first : '',
  },];
}

/**
 Sends one request, a refusal naming the endpoint where the transport rejects
 before any answer comes back: the network failing, or a request the
 transport will not send.

 AN ABORT STAYS AN ABORT. A transport rejects with the signal's reason when
 the caller stopped it, and that is the caller's decision, not a network
 failure, so it passes on unchanged.

 @param fetchFn - transport, `fetch` in production and a stub in tests

 @param init - request the transport is given, its signal the caller's

 @returns The endpoint's answer

 @throws The transport's rejection unchanged when the caller's signal aborted

 @throws {@link WorkTitleLookupError} when the transport rejected with the
 signal still standing (DNS failing, a connection resetting and a header
 value the transport will not send among the causes), the rejection kept as
 its cause

 @example
 ```ts
 const response = await sendSearch({ fetchFn: fetch, init: { method: 'POST', signal, }, },);
 ```
 */
async function sendSearch(
  {
    fetchFn,
    init,
  }: {
    readonly fetchFn: typeof fetch;
    readonly init: RequestInit;
  },
): Promise<Response> {
  try {
    return await fetchFn(
      EXA_SEARCH_URL,
      init,
    );
  } catch (error) {
    /**
     The caller's abort, absent when the request carried none.
     */
    const { signal, } = init;
    if (signal?.aborted === true)
      throw error;
    throw new WorkTitleLookupError({
      detail: `request to ${EXA_SEARCH_URL} failed before any answer came back: either the network could not be reached, or the transport refused to send the request, as it does for a key or header holding a line break; check the connection and the key`,
      cause: error,
    },);
  }
}

/**
 Reads the answer's body as JSON, a refusal naming the endpoint where the
 connection fails while the body arrives.

 A CREDENTIAL THE REQUEST CARRIED IS MASKED OUT OF THE TEXT before it is
 parsed (`masked-json-body.ts`), since what is read here is kept in a sheet
 and in a cache.

 A BODY THAT IS NOT JSON is the parser's `SyntaxError` and passes on
 unchanged: its message quotes the text it refused, so the log names its class
 alone (ledger B166). An abort passes on unchanged too.

 @param response - the endpoint's answer, status already checked

 @param signal - the call's abort

 @param headers - headers the request carried, which name the credentials
 the body must not repeat

 @returns The parsed body

 @throws The caller's abort, or a parse failure, unchanged

 @throws {@link WorkTitleLookupError} when the connection failed while the
 body was read, the failure kept as its cause

 @example
 ```ts
 const parsed = await readSearchBody({ response, signal, headers, },);
 ```
 */
async function readSearchBody(
  {
    response,
    signal,
    headers,
  }: {
    readonly response: Response;
    readonly signal: AbortSignal;
    readonly headers: Readonly<Record<string, string>>;
  },
): Promise<unknown> {
  try {
    return await readMaskedJsonBody({
      response,
      headers,
    },);
  } catch (error) {
    if (signal.aborted || (error instanceof SyntaxError))
      throw error;
    throw new WorkTitleLookupError({
      detail: `lost the network while ${EXA_SEARCH_URL} was answering`,
      cause: error,
    },);
  }
}

/**
 Asks the search endpoint about one title.

 @param apiKey - key sent as `x-api-key`, never logged

 @param query - search string

 @param signal - the call's abort

 @param fetchFn - transport, `fetch` in production and a stub in tests

 @returns Hits the endpoint returned

 @throws {@link WorkTitleLookupError} when the endpoint answers anything but
 2xx or a body without a results array, or the network fails before or while
 it answers; the caller's abort passes on as it came

 @example
 ```ts
 const hits = await searchWorkTitle({ apiKey, query, signal, fetchFn: fetch, },);
 ```
 */
export async function searchWorkTitle(
  {
    apiKey,
    query,
    signal,
    fetchFn,
  }: {
    readonly apiKey: string;
    readonly query: string;
    readonly signal: AbortSignal;
    readonly fetchFn: typeof fetch;
  },
): Promise<readonly LookupHit[]> {
  /**
   Headers of the request, which name the credential its answer must not repeat.
   */
  const headers = {
    'x-api-key': apiKey,
    'content-type': 'application/json',
    accept: 'application/json',
  };

  /**
   Endpoint's answer.
   */
  const response = await sendSearch({
    fetchFn,
    init: {
      method: 'POST',
      headers,
      body: JSON.stringify({
        query,
        type: 'auto',
        numResults: RESULTS_PER_TITLE,
        contents: {
          highlights: {
            query,
            maxCharacters: HIGHLIGHT_CHARACTERS,
          },
        },
      },),
      signal,
    },
  },);
  if (!response.ok) {
    // The body is not read: it is the endpoint's own words, which a refusal
    // must not carry (ledger B166).
    throw new WorkTitleLookupError({ detail: `responded ${String(response.status,)}`, },);
  }
  /**
   Parsed body.
   */
  const parsed = await readSearchBody({
    response,
    signal,
    headers,
  },);
  if (!isJsonRecord(parsed,)) {
    throw new WorkTitleLookupError({ detail: 'answered with a body that is not an object', },);
  }
  /**
   Results field.
   */
  const { results, } = parsed;
  if (!Array.isArray(results,)) {
    throw new WorkTitleLookupError({ detail: 'answered without a results array', },);
  }
  /**
   Results as unknowns, each narrowed.
   */
  const wires: readonly unknown[] = results;
  return wires.flatMap(function toHits(value,): readonly LookupHit[] {
    return hitsOf({ value, },);
  },);
}

//endregion Work-title search
