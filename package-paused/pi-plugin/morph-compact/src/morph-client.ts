/**
 Direct fetch client for the Morph Compact API (`/v1/compact`).
 
 Replaces `@morphllm/morphsdk`'s `CompactClient` to drop the indirect
 dependency on `@morphllm/morphsdk` and `diff` (which has a low DoS
 advisory pinned outside the SDK's `^7.0.0` range).
 
 Wire format mirrors `morphsdk@0.2.167`'s compact module
 (`tools/compact/core.ts`). Endpoint: `POST {apiUrl}/v1/compact` with
 `Authorization: Bearer {apiKey}` and JSON body.
 
 @module
 */

//region Types

/**
 Range of removed lines in the compacted output (1-indexed, inclusive).
 
 @example
 ```typescript
 const range: CompactedRange = { start: 12, end: 18 };
 ```
 */
export type CompactedRange = {
  /**
   First line in range (1-indexed, inclusive).
   */
  start: number;
  /**
   Last line in range (1-indexed, inclusive).
   */
  end: number;
};

/**
 Per-message compaction metadata returned by `/v1/compact`.
 
 @example
 ```typescript
 const msg: CompactMessage = {
   role: 'user',
   content: 'compressed text',
   compacted_line_ranges: [],
   kept_line_ranges: [],
 };
 ```
 */
export type CompactMessage = {
  /**
   Role of message ("user", "assistant", etc.).
   */
  role: string;
  /**
   Compressed message body.
   */
  content: string;
  /**
   Optional message name as supplied in request.
   */
  name?: string;
  /**
   Line ranges removed during compaction.
   */
  compacted_line_ranges: CompactedRange[];
  /**
   Line ranges force-preserved via `<keepContext>` tags.
   */
  kept_line_ranges: CompactedRange[];
};

/**
 Single message accepted as input to `/v1/compact`.
 
 @example
 ```typescript
 const msg: CompactInputMessage = { role: 'user', content: 'hello' };
 ```
 */
export type CompactInputMessage = {
  /**
   Role of message ("user", "assistant", etc.).
   */
  readonly role: string;
  /**
   Raw message body.
   */
  readonly content: string;
  /**
   Optional message name forwarded to API.
   */
  readonly name?: string;
};

/**
 Input shape for {@link MorphCompactClient.compact}.
 
 Either `input` or `messages` must be provided; `messages` takes priority
 when both are set.
 
 @example
 ```typescript
 const params: CompactInput = { input: 'long text', query: 'auth' };
 ```
 */
export type CompactInput = {
  /**
   String input or array of messages; ignored when `messages` is set.
   */
  readonly input?: string | readonly CompactInputMessage[];
  /**
   Array of messages to compact; takes priority over `input`.
   */
  readonly messages?: readonly CompactInputMessage[];
  /**
   Query the compactor conditions on; auto-detected from last user message when omitted.
   */
  readonly query?: string;
  /**
   Fraction of content to keep (0.05 to 1.0); default 0.5.
   */
  readonly compressionRatio?: number;
  /**
   Number of recent messages to keep uncompressed; default 2.
   */
  readonly preserveRecent?: number;
  /**
   Whether to include `compacted_line_ranges` in response; default `true`.
   */
  readonly includeLineRanges?: boolean;
  /**
   Whether to include "(filtered N lines)" markers; default `true`.
   */
  readonly includeMarkers?: boolean;
  /**
   Compactor model identifier; default "morph-compactor".
   */
  readonly model?: string;
  /**
   Optional cancellation signal forwarded to `fetch`; combined with the configured timeout.
   */
  readonly signal?: AbortSignal;
};

/**
 Response shape from `/v1/compact`.
 
 @example
 ```typescript
 const result: CompactResult = await client.compact({ input: 'text' });
 ```
 */
export type CompactResult = {
  /**
   Response identifier from Morph API.
   */
  id: string;
  /**
   All compacted messages joined by newline.
   */
  output: string;
  /**
   Per-message compaction metadata.
   */
  messages: CompactMessage[];
  /**
   Token usage and compression statistics.
   */
  usage: {
    /**
     Tokens in input payload.
     */
    input_tokens: number;
    /**
     Tokens in compressed output.
     */
    output_tokens: number;
    /**
     Realized compression ratio (output/input).
     */
    compression_ratio: number;
    /**
     Server-side processing time in milliseconds.
     */
    processing_time_ms: number;
  };
  /**
   Compactor model that produced response.
   */
  model: string;
};

/**
 Configuration for {@link MorphCompactClient}.
 
 @example
 ```typescript
 const config: CompactConfig = { morphApiKey: 'sk-...' };
 ```
 */
export type CompactConfig = {
  /**
   Morph API key; falls back to `MORPH_API_KEY` env var when undefined.
   */
  readonly morphApiKey?: string;
  /**
   Override base URL (e.g. for staging or proxies).
   */
  readonly morphApiUrl?: string;
  /**
   Per-request timeout in milliseconds; default 120000.
   */
  readonly timeout?: number;
};

//endregion

//region Constants

/**
 Default Morph API base URL.
 */
export const DEFAULT_API_URL = 'https://api.morphllm.com';

/**
 Default per-request timeout in milliseconds.
 */
export const DEFAULT_TIMEOUT_MS = 120_000;

/**
 Default compactor model identifier.
 */
export const DEFAULT_MODEL = 'morph-compactor';

/**
 Default compression ratio (fraction of content to keep).
 */
export const DEFAULT_COMPRESSION_RATIO = 0.5;

/**
 Default number of recent messages preserved uncompressed.
 */
export const DEFAULT_PRESERVE_RECENT = 2;

//endregion

//region Errors

/**
 Thrown when no API key is available from config or environment.
 
 @example
 ```typescript
 throw new MorphApiKeyMissingError();
 ```
 */
export class MorphApiKeyMissingError extends Error {
  /**
   Build error with fixed user-facing message.
   */
  constructor() {
    super(
      'Morph API key not found. Set MORPH_API_KEY environment variable or pass morphApiKey in config.',
    );
    this.name = 'MorphApiKeyMissingError';
  }
}

/**
 Thrown when the Morph API responds with a non-2xx status.
 
 @example
 ```typescript
 throw new MorphApiError({ status: 401, body: 'unauthorized' });
 ```
 */
export class MorphApiError extends Error {
  /**
   HTTP status from response.
   */
  readonly status: number;
  /**
   Raw response body returned by API.
   */
  readonly body: string;

  /**
   Build error from HTTP status and raw response body.
   */
  constructor({
    status,
    body,
  }: {
    readonly status: number;
    readonly body: string;
  },) {
    super(`Morph compact API error ${status}: ${body}`,);
    this.name = 'MorphApiError';
    this.status = status;
    this.body = body;
  }
}

/**
 Thrown when caller supplies neither `input` nor `messages`.
 
 @example
 ```typescript
 throw new MorphInvalidInputError();
 ```
 */
export class MorphInvalidInputError extends Error {
  /**
   Build error with fixed user-facing message.
   */
  constructor() {
    super("Either 'input' or 'messages' must be provided",);
    this.name = 'MorphInvalidInputError';
  }
}

//endregion

//region Helpers

/**
 Resolve API key from explicit config or `MORPH_API_KEY` env var.
 
 @param explicit - API key from config; takes precedence over env var
 
 @returns resolved API key string
 
 @throws {@link MorphApiKeyMissingError} when no key is found in config or environment.
 
 @example
 ```typescript
 const key = resolveApiKey('sk-...');
 ```
 */
function resolveApiKey(explicit?: string,): string {
  if ((explicit !== undefined) && (explicit !== ''))
    return explicit;
  /**
   Browser-safe fallback to process env when the runtime exposes one.
   */
  const envKey = ((typeof process) !== 'undefined')
    ? process.env
      .MORPH_API_KEY
    : undefined;
  if ((envKey !== undefined) && (envKey !== ''))
    return envKey;
  throw new MorphApiKeyMissingError();
}

/**
 Build JSON body for `/v1/compact` from a {@link CompactInput}.
 
 @param input - compaction request shape mirroring SDK input
 
 @returns wire-format JSON body ready for `fetch`
 
 @throws {@link MorphInvalidInputError} when neither `input` nor `messages` is supplied.
 
 @example
 ```typescript
 const body = buildRequestBody({ input: 'text', query: 'q' });
 ```
 */
function buildRequestBody(input: CompactInput,): Record<string, unknown> {
  /**
   Mutable accumulator filled with API-shape keys; one of input/messages added below.
   */
  const body: Record<string, unknown> = {
    compression_ratio: input.compressionRatio
      ?? DEFAULT_COMPRESSION_RATIO,
    preserve_recent: input.preserveRecent
      ?? DEFAULT_PRESERVE_RECENT,
    model: input.model
      ?? DEFAULT_MODEL,
    include_line_ranges: input.includeLineRanges
      ?? true,
    include_markers: input.includeMarkers
      ?? true,
  };
  if (input.query
    !== undefined)
    body.query = input.query;
  if (input.messages
    !== undefined) {
    body.messages = input.messages;
    return body;
  }
  if ((typeof input.input) === 'string') {
    body.input = input.input;
    return body;
  }
  if (Array.isArray(input.input,)) {
    body.messages = input.input;
    return body;
  }
  throw new MorphInvalidInputError();
}

/**
 Combine optional caller-supplied signal with request timeout signal so
 either source can cancel the in-flight `fetch`.
 
 @param options - Optional caller signal and hard timeout.
 
 @returns single signal that aborts on first source to fire
 
 @mutates options - DOM commit 5796f716 AbortSignal.any dependent-signal relations can retain `options.caller`.
 
 @example
 ```typescript
 const signal = buildSignal({ caller: userSignal, timeoutMs: 60000 });
 ```
 */
function buildSignal(options: {
  readonly caller?: AbortSignal;
  readonly timeoutMs: number;
},): AbortSignal {
  /**
   Signal and primitive timeout extracted after naming effect boundary.
   */
  const {
    caller,
    timeoutMs,
  } = options;
  /**
   Hard ceiling so a hung request cannot block compaction indefinitely.
   */
  const timeout = AbortSignal.timeout(timeoutMs,);
  if (caller === undefined)
    return timeout;
  return AbortSignal.any([
    caller,
    timeout,
  ],);
}

//endregion

//region Client

/**
 Public surface of a Morph Compact client: a single `compact` method.
 A frozen object literal (factory output) rather than a class, per the
 no-class rule.
 */
export type MorphCompactClient = {
  /**
   Compact messages or text via `POST {apiUrl}/v1/compact`.
   */
  readonly compact: (input: CompactInput,) => Promise<CompactResult>;
};

/**
 Build a Morph Compact client with optional config overrides; defaults
 applied for `morphApiUrl` and `timeout`. The API key is resolved per-request
 to allow late-binding via env var.
 
 Returns per-message `compacted_line_ranges` showing which lines were
 removed. Caller may pass an `AbortSignal` to cancel mid-flight; it is
 combined with the configured timeout via `AbortSignal.any`.
 
 @param config - optional overrides for API URL, timeout, and key
 
 @returns frozen client exposing {@link MorphCompactClient.compact}
 
 @throws {@link MorphApiKeyMissingError} when no API key is available, {@link MorphInvalidInputError} when neither
   `input` nor `messages` is set, or {@link MorphApiError} when the API responds with a non-2xx status.
 
 @example
 ```typescript
 const client = createMorphCompactClient({ morphApiKey: 'sk-...' });
 const result = await client.compact({
   input: longText,
   query: 'auth flow',
   compressionRatio: 0.5,
 });
 console.log(result.output);
 ```
 */
export function createMorphCompactClient(
  config: Readonly<CompactConfig> = {},
): MorphCompactClient {
  /**
   Resolved base URL; falls back to the public Morph endpoint.
   */
  const morphApiUrl = config.morphApiUrl
    ?? DEFAULT_API_URL;
  /**
   Resolved per-request timeout in milliseconds.
   */
  const timeout = config.timeout
    ?? DEFAULT_TIMEOUT_MS;
  /**
   Explicit key override resolved late so env overrides still surface.
   */
  const explicitKey = config.morphApiKey;

  return Object.freeze({
    async compact(input: CompactInput,): Promise<CompactResult> {
      /**
       Late-bound key resolution so env overrides can surface per call.
       */
      const apiKey = resolveApiKey(explicitKey,);
      /**
       Fully qualified compact endpoint resolved against the configured base URL.
       */
      const url = `${morphApiUrl}/v1/compact`;
      /**
       Wire-shape JSON payload built from caller input.
       */
      const body = buildRequestBody(input,);
      /**
       Composite signal so either caller-cancel or timeout aborts fetch.
       */
      const signal = buildSignal({
        ...((input.signal
          !== undefined) ? { caller: input.signal, } : {}),
        timeoutMs: timeout,
      },);
      /**
       Raw fetch response inspected for status and parsed below.
       */
      const response = await fetch(
        url,
        {
          method: 'POST',
          headers: {
            Authorization: `Bearer ${apiKey}`,
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(body,),
          signal,
        },
      );
      if (!response.ok) {
        /**
         Captured error body forwarded into MorphApiError for diagnostics.
         */
        const text = await response.text();
        throw new MorphApiError({
          status: response.status,
          body: text,
        },);
      }
      // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- response.json() returns any; shape is the documented /v1/compact contract mirrored from morphsdk@0.2.167
      return await response.json() as CompactResult;
    },
  },);
}

//endregion
