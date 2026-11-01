//region Cited-reference fetch
// One page's text off the Exa contents endpoint, the same key and transport
// shape as `work-title-search.ts`. Raw `fetch`, no SDK (the owner's rule).
// The endpoint (read on 2026-09-16 at exa.ai/docs/reference/get-contents):
// POST https://api.exa.ai/contents with `urls` and `text.maxCharacters`
// (1 to 10,000), answering `results[]` with `url`, `title`, `text` and
// `statuses[]` with `status` success or error and an `error.tag`.

import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { readMaskedJsonBody, } from '@monochromatic-dev/translation-repair-provider-stream/ts';

/**
 Where the contents endpoint lives.
 */
export const EXA_CONTENTS_URL = 'https://api.exa.ai/contents';

/**
 Longest page text carried per reference, in characters. Four thousand holds
 the whole of the Mio blog's passages about the sister and the accident and
 keeps eight references under a third of a sheet.
 */
export const REFERENCE_TEXT_CHARACTERS = 4_000;

/**
 Raised when the endpoint refuses or answers in a shape the reader cannot
 use.

 NAMES THE HTTP STATUS OR THE CHECK THAT FAILED AND NOTHING THE ENDPOINT
 WROTE: the sentence is built here from a phrase each throw site authors, so
 a log line repeats it whole and an operator reads the status, where an
 endpoint's body never reaches one.

 @example
 ```ts
 throw new CitedReferenceFetchError({ detail: 'responded 401', },);
 ```
 */
export class CitedReferenceFetchError extends Error {
  /**
   Declares this message safe to forward: the status or the check that
   failed, never a body, a url or the key.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal.

   @param detail - phrase the throw site authors, an HTTP status, the check the answer failed or the network failing, never text read from the answer

   @param cause - failure the transport raised, kept for whoever inspects the refusal and never repeated in the message

   @example
   ```ts
   throw new CitedReferenceFetchError({ detail: 'responded 401', },);
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
      `contents ${detail}`,
      // Conditional spread keeps cause absent when none was supplied.
      ...((cause === undefined) ? [] : [{ cause, },]),
    );
    this.name = 'CitedReferenceFetchError';
  }
}

/**
 What one fetch yields: the page's title and text on success, the endpoint's
 error tag on failure.

 A UNION ON `status`: the tag is present exactly when the fetch failed, as
 `fetchedOf` has always built it, so a reader needs no fallback for an error
 without one (ledger T8).

 @example
 ```ts
 const fetched: FetchedReference = { status: 'success', title: 'In Memory', text: '...', };
 ```
 */
export type FetchedReference =
  | {
    /**
     The endpoint read the page.
     */
    readonly status: 'success';
    /**
     Page title, empty when the endpoint gave none.
     */
    readonly title: string;
    /**
     Page text up to `REFERENCE_TEXT_CHARACTERS`.
     */
    readonly text: string;
  }
    | {
    /**
     The endpoint could not read the page.
     */
      readonly status: 'error';
    /**
     Page title, empty when the endpoint gave none.
     */
      readonly title: string;
    /**
     Page text, empty on failure.
     */
      readonly text: string;
    /**
     Endpoint's error tag, `error` when it named none, `no result` when it
     answered nothing at all.
     */
      readonly failure: string;
    };

/**
 String field of a record, empty when absent or not text.

 @param record - parsed object

 @param key - field to read

 @returns Field text or empty

 @example
 ```ts
 stringField({ record: { title: 'x', }, key: 'title', },);
 // => 'x'
 ```
 */
function stringField(
  {
    record,
    key,
  }: {
    readonly record: Readonly<Record<string, unknown>>;
    readonly key: string;
  },
): string {
  /**
   Raw field.
   */
  const value = record[key];
  return ((typeof value) === 'string') ? value : '';
}

/**
 Failure tag of the first status when it reports an error, empty otherwise.

 @param statuses - endpoint's `statuses`, unknown until read

 @returns Error tag, or empty for success or a missing status

 @example
 ```ts
 failureOf({ statuses: [{ status: 'error', error: { tag: 'CRAWL_NOT_FOUND', }, },], },);
 // => 'CRAWL_NOT_FOUND'
 ```
 */
function failureOf({ statuses, }: { readonly statuses: unknown; },): string {
  if (!isJsonArray(statuses,))
    return '';
  /**
   First status, the only one for a one-url ask.
   */
  const [first,] = statuses;
  if (!isJsonRecord(first,))
    return '';
  if (first.status !== 'error')
    return '';
  /**
   Error detail when the endpoint gave one.
   */
  const detail = isJsonRecord(first.error,) ? first.error : {};
  /**
   Tag naming the failure, or the bare word when there is none.
   */
  const tag = stringField({
    record: detail,
    key: 'tag',
  },);
  return (tag === '') ? 'error' : tag;
}

/**
 Reads the endpoint's answer for one url into a fetched reference.

 @param parsed - response body

 @returns What the body says about the page

 @throws When the body carries no results array

 @example
 ```ts
 const fetched = fetchedOf({ parsed: { results: [{ title: 't', text: 'x', },], statuses: [], }, },);
 ```
 */
export function fetchedOf({ parsed, }: { readonly parsed: unknown; },): FetchedReference {
  if (!isJsonRecord(parsed,))
    throw new CitedReferenceFetchError({ detail: 'answered with a body that is not an object', },);
  if (!isJsonArray(parsed.results,))
    throw new CitedReferenceFetchError({ detail: 'answered without a results array', },);
  /**
   Failure tag, empty on success.
   */
  const failure = failureOf({ statuses: parsed.statuses, },);
  /**
   First result, the only one for a one-url ask.
   */
  const [first,] = parsed.results;
  /**
   Result as a record, empty when the endpoint returned none.
   */
  const result = isJsonRecord(first,) ? first : {};
  /**
   Page text the endpoint read.
   */
  const text = stringField({
    record: result,
    key: 'text',
  },);
  /**
   Whether the endpoint returned no result at all.
   */
  const resultless = (text === '') && (first === undefined);
  if ((failure !== '') || resultless) {
    return {
      status: 'error',
      title: stringField({
        record: result,
        key: 'title',
      },),
      text: '',
      failure: (failure === '') ? 'no result' : failure,
    };
  }
  return {
    status: 'success',
    title: stringField({
      record: result,
      key: 'title',
    },),
    text,
  };
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

 @throws {@link CitedReferenceFetchError} when the transport rejected with the
 signal still standing (DNS failing, a connection resetting and a header
 value the transport will not send among the causes), the rejection kept as
 its cause

 @example
 ```ts
 const response = await sendContentsRequest({ fetchFn: fetch, init: { method: 'POST', signal, }, },);
 ```
 */
async function sendContentsRequest(
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
      EXA_CONTENTS_URL,
      init,
    );
  } catch (error) {
    /**
     The caller's abort, absent when the request carried none.
     */
    const { signal, } = init;
    if (signal?.aborted === true)
      throw error;
    throw new CitedReferenceFetchError({
      detail: `request to ${EXA_CONTENTS_URL} failed before any answer came back: either the network could not be reached, or the transport refused to send the request, as it does for a key or header holding a line break; check the connection and the key`,
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

 @throws {@link CitedReferenceFetchError} when the connection failed while the
 body was read, the failure kept as its cause

 @example
 ```ts
 const parsed = await readContentsBody({ response, signal, headers, },);
 ```
 */
async function readContentsBody(
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
    throw new CitedReferenceFetchError({
      detail: `lost the network while ${EXA_CONTENTS_URL} was answering`,
      cause: error,
    },);
  }
}

/**
 Asks the contents endpoint for one page's text.

 @param apiKey - key sent as `x-api-key`, never logged

 @param url - page to read

 @param signal - the call's abort

 @param fetchFn - transport, `fetch` in production and a stub in tests

 @returns What the endpoint read

 @throws When the endpoint answers anything but 2xx or a body without
 results, or the network fails before or while it answers; the caller's abort
 passes on as it came

 @example
 ```ts
 const fetched = await fetchCitedReference({ apiKey, url, signal, fetchFn: fetch, },);
 ```
 */
export async function fetchCitedReference(
  {
    apiKey,
    url,
    signal,
    fetchFn,
  }: {
    readonly apiKey: string;
    readonly url: string;
    readonly signal: AbortSignal;
    readonly fetchFn: typeof fetch;
  },
): Promise<FetchedReference> {
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
  const response = await sendContentsRequest({
    fetchFn,
    init: {
      method: 'POST',
      headers,
      body: JSON.stringify({
        urls: [url,],
        text: { maxCharacters: REFERENCE_TEXT_CHARACTERS, },
      },),
      signal,
    },
  },);
  if (!response.ok) {
    // The body is not read: it is the endpoint's own words, which a refusal
    // must not carry (ledger B166).
    throw new CitedReferenceFetchError({ detail: `responded ${String(response.status,)}`, },);
  }
  /**
   Parsed body.
   */
  const parsed = await readContentsBody({
    response,
    signal,
    headers,
  },);
  return fetchedOf({ parsed, },);
}

//endregion Cited-reference fetch
