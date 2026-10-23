import type { CompletionUsage, } from '@monochromatic-dev/module-llm-type/ts';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

import { wholeOpening, } from '@monochromatic-dev/translation-repair-markup/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { StreamWireFormat, } from './stream-wire-format.ts';

//region Completion shape
// Provider protocol reading the streaming clients share. A 200 reply that fails
// these expectations is a provider defect and throws; what the model wrote is
// never judged here, that is chatJson's job and flows as data. The non-streaming
// body parser that once lived here went with the last non-streaming call
// (ledger B30).
//
// EVERY PROVIDER THROWS THESE CLASSES, so no message here names one. `ProviderHttpError`
// is the one class for a non-success HTTP status from any of them, and it carries the
// provider that served the call as a field (renamed from `SyntheticHttpError` on
// 2026-10-19, when a Bedrock refusal was logged under the Synthetic class name with no
// provider). The NAME and the MESSAGES were separate problems, because a name is read by
// whoever edits this file and a message is read by whoever is holding a broken run at
// two in the morning.
//
// FOUND IN A LIVE LOG, not by reading: `panel gemma-4-26b-a4b-it: MalformedCompletionError:
// Synthetic completion body violated the OpenAI-compatible contract`. Every one of that
// model's 51 billed calls that run went to Charm Hyper. The message sent the reader to
// the wrong provider's status page.
//
// SO A MESSAGE HERE STATES ONLY WHAT THIS FILE KNOWS: a status, a detail, an excerpt.
// The provider rides on the `provider` field and is rendered by `exchangeFailureText`
// (`exchange-failure-text.ts`), not interpolated into this message. A caller that wants
// the provider in the message itself says so through `summary`, which the meter endpoints
// do because each is bound to one provider by construction.
//
// THE CONTRACT A BODY BROKE IS THE READER'S TO NAME (ledger B90). The fix for that live
// log took the provider's name out and kept "the OpenAI-compatible contract", which the
// Hyper call it showed did not speak either: it was an Anthropic Messages stream. Every
// `MalformedCompletionError` now carries the wire format its reader read the body as.

/**
 Most UTF-16 units of the body excerpt embedded in thrown errors,
 bounding log size while keeping enough context to diagnose; the excerpt
 ends on a whole character (`wholeOpening`).
 */
const BODY_EXCERPT_LIMIT = 600;

/**
 Signals a non-success HTTP status from whichever provider served the call:
 Synthetic, Amazon Bedrock, Charm Hyper or OpenRouter.

 THE PROVIDER IS A REQUIRED FIELD, set by the client that made the call and never
 inferred from the status or the body, because no status or body names its sender.
 A log line that reports this error without the provider reads as the wrong
 provider's failure.

 @example
 ```ts
 throw new ProviderHttpError({ provider: 'bedrock', status: 429, bodyText: reply.bodyText, },);
 ```
 */
export class ProviderHttpError extends Error {
  /**
   Provider that served the call and answered with the failing status.
   */
  public readonly provider: ProviderName;

  /**
   HTTP status the API returned; drivers branch on 429 for backoff.
   */
  public readonly status: number;

  /**
   Opening of the response body for diagnosis.
   */
  public readonly bodyExcerpt: string;

  /**
   Wait the reply asked for in a header rather than in its body, in milliseconds, zero when it named none. Anthropic
   states a rate limit's wait in `retry-after` and its error body carries no wait text, so the body alone never
   shows a router how long to hold the provider out.
   */
  public readonly retryAfterMs: number;

  /**
   Builds failure carrying provider, status and body opening.

   @param provider - provider that served the call, so a failure read without its call site still says whose it was

   @param status - non-success HTTP status

   @param bodyText - raw response body, excerpted for the message

   @param summary - what went wrong, where a subclass knows something this
   class cannot infer from a status. Absent leaves the plain status line.
   The excerpt is appended either way, so no subclass has to know how long
   an excerpt runs or how to label whose words it quotes

   @param retryAfterMs - wait the reply's headers named, in milliseconds; absent when none was read

   @example
   ```ts
   new ProviderHttpError({ provider: 'hyper', status: 500, bodyText: 'upstream exploded', },);
   ```
   */
  public constructor(
    {
      provider,
      status,
      bodyText,
      summary,
      excerpt = 'quoted',
      retryAfterMs = 0,
    }: {
      readonly provider: ProviderName;
      readonly status: number;
      readonly bodyText: string;
      readonly summary?: string;
      readonly excerpt?: 'quoted' | 'withheld';
      readonly retryAfterMs?: number;
    },
  ) {
    // THE EXCERPT IS QUOTED INTO THE MESSAGE ON PURPOSE for this class, which
    // stays unmarked for that reason; a subclass that declares its message
    // quote-free withholds it and keeps it on `bodyExcerpt`, which
    // `exchangeFailureLogText` (`exchange-failure-text.ts`) reads for log
    // lines alone; `provider-budget-refusal.ts` and
    // `decision-context-refusal.ts` read it to classify a refusal.
    super(
      (excerpt === 'withheld')
        ? (summary ?? `provider API returned HTTP ${String(status,)}`)
        : `${summary ?? `provider API returned HTTP ${String(status,)}:`} ${
          wholeOpening({
            text: bodyText,
            units: BODY_EXCERPT_LIMIT,
          },)
        }`,
    );
    this.name = 'ProviderHttpError';
    this.provider = provider;
    this.status = status;
    this.bodyExcerpt = wholeOpening({
      text: bodyText,
      units: BODY_EXCERPT_LIMIT,
    },);
    this.retryAfterMs = retryAfterMs;
  }
}

/**
 Contract each wire format holds a body to, as a refusal names it.

 NAMED BY THE READER THAT REFUSED, never assumed here (ledger B90). The
 Anthropic readers throw this class too, and a message naming only the
 OpenAI-compatible contract told a reader holding a failed Hyper call that
 an OpenAI-shaped body was broken. The keys are the closed set of formats
 the stream readers declare, never text from a body.
 */
const CONTRACT_NAMES: Readonly<Record<StreamWireFormat, string>> = {
  openai: 'OpenAI-compatible',
  anthropic: 'Anthropic Messages',
};

/**
 Signals a success-status completion body that violates the contract of the
 wire format it was read as; always a provider defect, never a
 model-content defect.

 @example
 ```ts
 throw new MalformedCompletionError({ wireFormat: 'openai', detail: 'stream event is not valid JSON', },);
 ```
 */
export class MalformedCompletionError extends Error {
  /**
   Declares this message safe to forward: it names which part of the protocol the body broke, never the body.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the violated expectation.

   @param wireFormat - format the refusing reader read the body as, which names the contract

   @param detail - which contract expectation the body violated

   @param cause - underlying parse error when JSON itself failed

   @example
   ```ts
   new MalformedCompletionError({ wireFormat: 'anthropic', detail: 'stream event is not JSON', cause: error, },);
   ```
   */
  public constructor(
    {
      wireFormat,
      detail,
      cause,
    }: {
      readonly wireFormat: StreamWireFormat;
      readonly detail: string;
      readonly cause?: unknown;
    },
  ) {
    super(
      `completion body violated the ${CONTRACT_NAMES[wireFormat]} contract: ${detail}`,
      // Conditional spread keeps cause absent when none was supplied.
      ...(cause === undefined
        ? []
        : [{ cause, },]),
    );
    this.name = 'MalformedCompletionError';
  }
}

/**
 Text and optional usage extracted from one completion body.

 @example
 ```ts
 const extracted: ExtractedCompletion = { text: '{"issues":[]}', };
 ```
 */
export type ExtractedCompletion = {
  /**
   Verbatim `message.content` of the first choice;
   empty when the API refused and returned no content.
   Reasoning arrives in a separate field, so content is the answer channel.
   THAT FIELD'S NAME VARIES BY MODEL on this provider, `reasoning_content` on
   some and `reasoning` on others, measured 2026-08-21; `stream-delta-scan.ts`
   carries the per-model counts. Nothing here has to choose between them,
   because this path reads the answer and never the thinking.
   */
  readonly text: string;

  /**
   First-class refusal from the message `refusal` field;
   a stronger signal than any heuristic over content.
   */
  readonly refusal?: string;

  /**
   Token usage when the server reported it; feeds budget observability.
   Completion counts include thinking tokens,
   which dominate output on these models.
   */
  readonly usage?: CompletionUsage;

  /**
   Why the model stopped, verbatim from `choices[0].finish_reason`.

   READ BECAUSE A COMPLETION THAT STOPPED EARLY IS INDISTINGUISHABLE FROM A
   MALFORMED ONE WITHOUT IT. A model cut off mid-answer delivers a whole,
   well-formed stream with no unreadable frames, and the only symptom
   downstream is that its content does not parse. Reported as a schema
   mismatch, that sends a reader to the prompt and the schema; reported as
   `length`, it sends them to the token ceiling instead. Measured on a live
   lane-contest round where one voice stopped mid-string at 287 characters.

   ABSENT RATHER THAN DEFAULTED when the provider omits it, since guessing
   `stop` would assert the very thing this exists to establish.
   */
  readonly finishReason?: string;
};

/**
 Reads why one choice stopped, when the provider said.

 @param choice - one entry of the choices array

 @returns Spreadable fragment carrying the reason, or nothing

 @example
 ```ts
 const fragment = readFinishReason({ choice, },);
 ```
 */
export function readFinishReason(
  { choice, }: { readonly choice: Readonly<Record<string, unknown>>; },
): { readonly finishReason?: string; } {
  /**
   Reason as delivered, which providers may omit or send as null.
   */
  const reason = choice.finish_reason;
  return (((typeof reason) === 'string') && (reason !== ''))
    ? { finishReason: reason, }
    : {};
}

/**
 Reads optional usage block when both component counts are numbers.

 @param parsed - whole parsed completion body

 @returns Usage block, or nothing when absent or mistyped

 @example
 ```ts
 const usage = readUsage({ parsed, },);
 ```
 */
export function readUsage({ parsed, }: { readonly parsed: Readonly<Record<string, unknown>>; },): {
  readonly usage?: CompletionUsage;
} {
  /**
   Usage block as delivered, when any.
   */
  const { usage, } = parsed;
  if (!isJsonRecord(usage,))
    return {};

  /**
   Component token counts pulled out for numeric validation.
   */
  const {
    prompt_tokens: promptTokens,
    completion_tokens: completionTokens,
  } = usage;
  if (((typeof promptTokens) !== 'number') || ((typeof completionTokens) !== 'number'))
    return {};

  return {
    usage: {
      prompt_tokens: promptTokens,
      completion_tokens: completionTokens,
    },
  };
}

//endregion Completion shape
