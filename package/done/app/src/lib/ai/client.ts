/**
 HTTP client for an OpenAI-compatible chat completions endpoint (llama.cpp, Ollama, etc.).
 
 Reads `CHAT_COMPLETIONS_URL` from the environment.
 Enforces an in-memory sliding-window rate limit so a single Done instance
 cannot overwhelm the shared inference server.
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type {
  ChatCompletionResponse,
  ChatMessage,
} from '@monochromatic-dev/llm-llm-type/ts';

export type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

//region Configuration

/**
 Default URL for the chat completions endpoint.
 */
const DEFAULT_COMPLETIONS_URL = 'http://localhost:8080/v1/chat/completions';

/**
 Resolved endpoint URL, evaluated once at import time.
 */
const completionsUrl = process.env
  .CHAT_COMPLETIONS_URL
  ?? DEFAULT_COMPLETIONS_URL;

//endregion Configuration

//region Rate limiter (sliding-window counter)

/**
 Maximum requests allowed within the sliding rate-limit bucket.
 */
const MAX_REQUESTS_PER_WINDOW = 30;

/**
 Sliding window duration in milliseconds (60 seconds).
 */
const WINDOW_DURATION_MS = 60_000;

/**
 Request timeout in milliseconds (30 seconds).
 */
const REQUEST_TIMEOUT_MS = 30_000;

/**
 Timestamps of requests within the current rate-limit bucket.
 Mutable array because the sliding window requires `.shift()` to discard expired
 entries and `.push()` to record new ones; a functional approach would allocate
 a new array on every request for no benefit in this module-scoped singleton.
 */
const requestTimestamps: number[] = [];

/**
 Removes expired entries and checks whether the limit has been reached.
 
 @returns `true` when the request should be rejected
 */
function isRateLimited(): boolean {
  /**
   Lower bound of the sliding window; older timestamps are evicted below.
   */
  const cutoff = Date.now()
    - WINDOW_DURATION_MS;

  // Discard entries older than the window
  /* oxlint-disable typescript/no-unsafe-type-assertion -- array index 0 is checked via length guard */
  while ((requestTimestamps.length
    > 0) && ((requestTimestamps[0] as number)
      < cutoff))
    requestTimestamps.shift();
  /* oxlint-enable typescript/no-unsafe-type-assertion */

  return requestTimestamps.length
    >= MAX_REQUESTS_PER_WINDOW;
}

/**
 Records the current timestamp in the sliding rate-limit bucket.
 */
function recordRequest(): void {
  requestTimestamps.push(Date.now(),);
}

//endregion Rate limiter

//region Types

/**
 Options for a chat completion request.
 */
export type ChatCompletionOptions = {
  /**
   Messages in the conversation.
   */
  readonly messages: readonly ChatMessage[];
  /**
   Sampling temperature (0 = deterministic).
   */
  readonly temperature?: number;
  /**
   Maximum tokens to generate.
   */
  readonly maxTokens?: number;
  /**
   When `true`, request `response_format: \{ type: "json_object" \}`.
   */
  readonly jsonMode?: boolean;
};

/**
 Discriminated union result of a chat completion attempt.
 */
export type ChatCompletionResult =
  | {
    ok: true;
    content: string;
  }
    | {
      ok: false;
      error: string;
    };

//endregion Types

//region Public API

/**
 Sends a chat completion request to the configured endpoint.
 
 Rejects when {@link isRateLimited}, and otherwise calls {@link recordRequest}
 to track the attempt. Returns an error result rather than throwing so callers
 can degrade gracefully (e.g. skip autofill when the AI is unavailable).
 
 @param options - Messages, temperature, and format controls
 
 @returns Completion text on success, or a descriptive error string
 
 @example
 ```ts
 const result = await chatCompletion({ messages, temperature: 0.3 });
 ```
 */
export async function chatCompletion(
  options: ChatCompletionOptions,
): Promise<ChatCompletionResult> {
  if (isRateLimited()) {
    return {
      ok: false,
      error: 'Rate limit exceeded: try again in a moment',
    };
  }

  recordRequest();

  /**
   Mutable request body so optional fields can be appended conditionally below.
   */
  const body: Record<string, unknown> = {
    messages: options.messages,
    temperature: options.temperature
      ?? 0,
  };

  if (options.maxTokens
    !== undefined)
    body.max_tokens = options.maxTokens;

  if (options.jsonMode
    === true)
    body.response_format = { type: 'json_object', };

  try {
    /**
     Network response from the OpenAI-compatible endpoint.
     */
    const response = await fetch(
      completionsUrl,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', },
        body: JSON.stringify(body,),
        signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS,),
      },
    );

    if (!response.ok) {
      /**
       Best-effort body text; reassigned in the try block, default surfaces on parse failure.
       */
      let errorText = 'unknown error';
      try {
        errorText = await response.text();
      }
      catch (bodyTextError: unknown) {
        // Response body text could not be read; log it and keep the default message.
        console.error(
          'chatCompletion could not read error response body text:',
          bodyTextError,
        );
      }
      return {
        ok: false,
        error: `AI endpoint returned ${String(response.status,)}: ${errorText}`,
      };
    }

    /* oxlint-disable typescript/no-unsafe-type-assertion -- API response shape matches ChatCompletionResponse */
    /**
     Parsed response payload; only `choices[0]` is consumed below.
     */
    const data = (await response.json()) as ChatCompletionResponse;
    /* oxlint-enable typescript/no-unsafe-type-assertion */
    /**
     First completion choice; absent on malformed responses.
     */
    const [firstChoice,] = data.choices;
    if (firstChoice === undefined) {
      return {
        ok: false,
        error: 'AI returned no choices',
      };
    }

    return {
      ok: true,
      content: firstChoice.message
        .content,
    };
  }
  catch (caughtError: unknown) {
    /**
     Human-readable error text extracted whether the cause is an Error or a raw throw.
     */
    const message = caughtValueText(caughtError,);
    console.error(
      'AI chat completion failed:',
      caughtError,
    );
    return {
      ok: false,
      error: `AI request failed: ${message}`,
    };
  }
}

//endregion Public API
