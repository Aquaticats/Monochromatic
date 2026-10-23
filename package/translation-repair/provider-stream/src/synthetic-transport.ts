import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  credentialsOfHeaders,
  maskCredentials,
} from './credential-mask.ts';
import { drainBody, } from './stream-drain.ts';
import { capturedHeadersOf, } from './response-header-capture.ts';
import { StreamCutShortError, } from './stream-cut.ts';
import type { StreamWireFormat, } from './stream-wire-format.ts';
import {
  armIdleGuard,
  type IdleGuard,
} from './stream-idle-guard.ts';
import { TransportRequestFailedError, } from './transport-request-error.ts';

//region Transport abstraction
// The one seam between the client and the network: tests inject a fake transport
// with recorded replies, drivers use the fetch-backed default. Keeping the seam at
// raw status-plus-body level means recorded fixtures stay copy-pasteable from real
// traffic and every parsing branch above the seam is exercised by unit tests.

/**
 Raw reply from one HTTP exchange, before any parsing.

 @example
 ```ts
 const reply: TransportReply = { status: 200, bodyText: '{"choices":[]}', };
 ```
 */
export type TransportReply = {
  /**
   HTTP status code.
   */
  readonly status: number;

  /**
   Response body as text; JSON parsing happens above the transport seam.
   */
  readonly bodyText: string;

  /**
   The response headers the exchange asked for (`captureHeaders`), by lower-case name; absent where none was asked
   for or none came back, so a reply keeps the shape older callers read.
   */
  readonly headers?: Readonly<Record<string, string>>;
};

/**
 One HTTP exchange the client asks a transport to perform.
 `signal` is mandatory so no call can be constructed that user steering
 cannot abort.

 @example
 ```ts
 const exchange: TransportExchange = {
   url: SYNTHETIC_QUOTAS_URL,
   method: 'GET',
   headers: { Authorization: 'Bearer test-key', },
   signal: AbortSignal.timeout(30_000,),
 };
 ```
 */
export type TransportExchange = {
  /**
   Absolute request URL.
   */
  readonly url: string;

  /**
   What this call is FOR, in the caller's own vocabulary, which for a chat
   exchange is the model id.

   REQUIRED RATHER THAN OPTIONAL, so no call site can quietly fall back to the
   endpoint. Every chat exchange goes to one URL, so labelling by URL made
   per-model latency unreadable, and reasoning from abandon counts instead is
   what produced the retracted conclusion that one vendor's models were slow.
   */
  readonly label: string;

  /**
   Provider this exchange goes to, named by its client, which the stream guard
   puts in the line that reports a cut stream (ledger B804). The model id in
   `label` does not say it: `deepseek-v4.1-flash` is served by three providers,
   and a silent stream means a different thing on each. Absent where the
   caller does not know it.
   */
  readonly provider?: ProviderName;

  /**
   HTTP method; the Synthetic surface needs only these two.
   */
  readonly method: 'GET' | 'POST';

  /**
   Request headers, auth included.
   */
  readonly headers: Readonly<Record<string, string>>;

  /**
   Serialized JSON request body; absent on GET exchanges.
   */
  readonly bodyJson?: string;

  /**
   Abort signal honored for the whole exchange.
   */
  readonly signal: AbortSignal;

  /**
   Characters the answer channel may produce before the drain ends the
   call, when the caller knows its own input size well enough to bound
   the reply against it.
   */
  readonly maxAnswerChars?: number;

  /**
   Event grammar this endpoint's stream speaks, which is a property of the
   PROVIDER rather than of the request, and absent means the older one.

   NAMED BY THE CALLER because only the client knows which provider it is
   addressing; the transport is one function serving both.
   */
  readonly wireFormat?: StreamWireFormat;

  /**
   Silence allowed before the first generated text, in place of the shared
   window (`STREAM_FIRST_CONTENT_MS`), for a model whose own card measured a
   shorter one (ledger B809). Absent means the shared window.

   NAMED BY THE CALLER because only the client knows the card of the model it
   addresses; the transport is one function serving every client. It reaches
   the guard of every attempt of the retry ladder, since the ladder hands each
   attempt this same exchange.
   */
  readonly firstContentMs?: number;

  /**
   Response headers to hand back on the reply: exact lower-case names, or prefixes ending in an asterisk
   (`anthropic-ratelimit-*`). Absent asks for none.

   NAMED BY THE CALLER because only the client knows which headers it reads (`retry-after` for the ladder, a
   provider's rate-limit family for a log line); the transport is one function serving every client.
   */
  readonly captureHeaders?: readonly string[];
};

/**
 Transport function the client is parameterized over.

 @example
 ```ts
 const recorded: ModelTransport = async () => ({ status: 200, bodyText: '{}', });
 ```
 */
export type ModelTransport = (
  exchange: ForeignBorrowed<TransportExchange>,
) => Promise<TransportReply>;

/**
 Sends the request, naming a rejection the runtime raised with no abort standing.

 @param url - absolute request URL

 @param label - what the call is for, named by the failure

 @param dependentSignal - the signal the request carries, whose abort is steering or a stall

 @param init - request init handed to `fetch`

 @returns The platform response, headers read and body unread

 @throws {@link TransportRequestFailedError} when the runtime rejects the request
 and no abort stands, the rejection as its cause; an abort's rejection passes on

 @example
 ```ts
 const response = await sendRequest({ url, label, dependentSignal, init, },);
 ```
 */
async function sendRequest(
  {
    url,
    label,
    dependentSignal,
    init,
  }: {
    readonly url: string;
    readonly label: string;
    readonly dependentSignal: AbortSignal;
    readonly init: RequestInit;
  },
): Promise<Response> {
  try {
    return await fetch(
      url,
      init,
    );
  }
  catch (error) {
    // An abort, the caller's or the idle guard's, passes on as it came: the
    // retry ladder and the stream bound read it by identity.
    if (dependentSignal.aborted)
      throw error;

    throw new TransportRequestFailedError({
      label,
      cause: error,
    },);
  }
}

/**
 Drains a body and, where the stream was cut, masks the credentials out of the
 text the cut error carries.

 @param response - response whose body is drained

 @param guard - silence guard notified per chunk

 @param callerSignal - caller's own signal

 @param label - what the call is for

 @param credentials - secrets the request carried

 @param maxAnswerChars - bound for this call, when known

 @param wireFormat - event grammar of the endpoint

 @mutates response - the drain consumes its body

 @returns Whole decoded body, unmasked

 @throws {@link StreamCutShortError} rebuilt with masked partial text, the
 same label, progress and cause

 @example
 ```ts
 const text = await drainWithMaskedCut({ response, guard, callerSignal, label, credentials, },);
 ```
 */
async function drainWithMaskedCut(
  {
    response,
    guard,
    callerSignal,
    label,
    credentials,
    maxAnswerChars,
    wireFormat,
  }: {
    readonly response: Response;
    readonly guard: IdleGuard;
    readonly callerSignal: AbortSignal;
    readonly label: string;
    readonly credentials: readonly string[];
    readonly maxAnswerChars?: number;
    readonly wireFormat?: StreamWireFormat;
  },
): Promise<string> {
  try {
    return await drainBody({
      response,
      guard,
      callerSignal,
      label,
      credentials,
      ...((maxAnswerChars === undefined) ? {} : { maxAnswerChars, }),
      ...((wireFormat === undefined) ? {} : { wireFormat, }),
    },);
  }
  catch (error) {
    if (!(error instanceof StreamCutShortError))
      throw error;

    throw new StreamCutShortError({
      label: error.label,
      partialText: maskCredentials({
        text: error.partialText,
        credentials,
      },),
      progress: error.progress,
      ...((error.generatedChars === undefined) ? {} : { generatedChars: error.generatedChars, }),
      cause: error.cause,
    },);
  }
}

/**
 Default fetch-backed transport.
 `fetch` receives only locally owned values:
 primitive strings, a fresh headers copy, and a dependent signal,
 so caller-owned objects are never retained by the platform request.

 @param exchange - request to perform

 @mutates exchange - DOM commit 5796f716 AbortSignal.any dependent-signal relations can retain the exchange signal, and undiciFetch retains the derived signal and may invoke abort listeners through it for the request lifetime.

 @returns Status and body text, whatever the status was, with every credential the
 request carried masked out of the body (`maskCredentials`)

 @throws {@link TransportRequestFailedError} when the runtime rejects the request
 with no abort standing, the rejection kept as its cause; an abort passes on
 unchanged

 @example
 ```ts
 const reply = await fetchTransport({
   url: SYNTHETIC_QUOTAS_URL,
   method: 'GET',
   headers: { Authorization: `Bearer ${apiKey}`, },
   signal,
 },);
 ```
 */
export async function fetchTransport(
  exchange: ForeignBorrowed<TransportExchange>,
): Promise<TransportReply> {
  /**
   Fields extracted after naming the effect boundary;
   url, method, and body are primitives.
   */
  const {
    url,
    label,
    provider,
    method,
    headers,
    bodyJson,
    signal,
    maxAnswerChars,
    wireFormat,
    firstContentMs,
    captureHeaders,
  } = exchange;

  /**
   Silence guard for this exchange. Armed before the request so its window
   also covers a provider that never sends response headers.
   */
  using guard = armIdleGuard({
    label,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((provider === undefined) ? {} : { provider, }),
    // Conditional spread keeps the window absent, so the shared one stands.
    ...((firstContentMs === undefined) ? {} : { firstContentMs, }),
  },);

  /**
   Dependent signal derived locally so the platform request never holds the
   caller's own signal handle. The guard rides along so a stalled stream
   tears the request down without touching the caller's signal, which is what
   lets the retry layer treat it as transient.
   */
  const dependentSignal = AbortSignal.any([
    signal,
    guard.signal,
  ],);

  /**
   Raw fetch response; body is read as text so callers decide how to parse.
   Chat exchanges stream (the provider is finicky without streaming, and
   headers on a stream arrive before fetch's default headers timeout);
   reading to text drains the whole event stream.
   */
  const response = await sendRequest({
    url,
    label,
    dependentSignal,
    init: {
      method,
      // Fresh copy: header values are primitive strings, so the platform
      // request holds no caller-owned object.
      headers: { ...headers, },
      // Conditional spread keeps body absent (not explicitly undefined) on GET.
      ...(bodyJson === undefined
        ? {}
        : { body: bodyJson, }),
      signal: dependentSignal,
    },
  },);

  /**
   Secrets this request carried, which no reply body may repeat.
   */
  const credentials = credentialsOfHeaders({ headers, },);

  /**
   Headers the caller asked for, none when it asked for none or none came back.
   */
  const captured = (captureHeaders === undefined)
    ? {}
    : capturedHeadersOf({
      headers: response.headers,
      wanted: captureHeaders,
    },);

  return {
    status: response.status,
    // MASKED AFTER THE WHOLE BODY IS ASSEMBLED, so a credential cut in two by
    // a chunk boundary is whole again when it is looked for, and on a
    // successful reply too, since an error event can arrive inside a 200 stream.
    bodyText: maskCredentials({
      text: await drainWithMaskedCut({
        response,
        guard,
        callerSignal: signal,
        label,
        credentials,
        ...((maxAnswerChars === undefined) ? {} : { maxAnswerChars, }),
        ...((wireFormat === undefined) ? {} : { wireFormat, }),
      },),
      credentials,
    },),
    // Conditional spread keeps the member absent when no header was captured.
    ...((Object.keys(captured,)
      .length
      === 0) ? {} : { headers: captured, }),
  };
}

//endregion Transport abstraction
