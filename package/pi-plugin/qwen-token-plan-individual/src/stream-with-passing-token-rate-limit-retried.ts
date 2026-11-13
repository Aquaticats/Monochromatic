/**
 The provider's answer stream, with this provider's passing token-rate limit
 handed to pi in a form pi's own automatic retry takes.

 pi retries a failed turn from the message the stream ended on
 (`isRetryableAssistantError` over the settled `AssistantMessage`),
 so the translation happens where this provider's stream is consumed.
 The events are forwarded as they arrive,
 because the caller reads a reply while it streams,
 and only the terminal error event of a passing limit is rebuilt.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import type {
  Api,
  AssistantMessage,
  AssistantMessageEvent,
  AssistantMessageEventStream,
  Model,
  Usage,
} from '@earendil-works/pi-ai';
import { createAssistantMessageEventStream, } from '@earendil-works/pi-ai/utils/event-stream';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  isPassingTokenRateLimitAnswer,
  messagePiRetriesInPlaceOfPassingTokenRateLimit,
} from './passing-token-rate-limit.ts';

//region Types

/**
 What one forwarded stream needs:
 the stream to read, the stream to fill, the model the answer belongs to, and
 the logger both the rewrite and the guard report through.
 */
type ForwardedStream = {
  /**
   Stream the provider implementation produced.
   */
  readonly inner: AssistantMessageEventStream;
  /**
   Stream this module hands the caller.
   */
  readonly outer: AssistantMessageEventStream;
  /**
   Model the answer belongs to,
   so a stream that reports nothing can still say what it answered for.
   */
  readonly model: ForeignBorrowed<Model<Api>>;
  /**
   Logger tagged for the forwarding.
   */
  readonly l: Logger;
};

//endregion Types

//region Usage of a stream that reported nothing

/**
 Usage an answer that never arrived carries:
 nothing was read, written or charged.

 @returns four zero token counts and five zero cost rates

 @example
 ```typescript
 const usage = noUsage();
 ```
 */
function noUsage(): Usage {
  return {
    input: 0,
    output: 0,
    cacheRead: 0,
    cacheWrite: 0,
    totalTokens: 0,
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
      total: 0,
    },
  };
}

//endregion Usage of a stream that reported nothing

//region Forwarding

/**
 The message this module reports when the provider's stream ends without a
 terminal event.

 A caller waits on the stream's own result forever otherwise
 (`AssistantMessageEventStream.result` in `@earendil-works/pi-ai`
 never settles without a terminal event),
 and pi's own stream implementation always pushes one,
 so this path exists to keep a broken stream from hanging a run.

 @param model - model the answer belongs to

 @returns an error message the caller can settle on

 @example
 ```typescript
 const answer = answerThatNeverArrived({ model });
 ```
 */
function answerThatNeverArrived({
  model,
}: {
  readonly model: ForeignBorrowed<Model<Api>>;
},): AssistantMessage {
  return {
    role: 'assistant',
    content: [],
    api: model.api,
    provider: model.provider,
    model: model.id,
    usage: noUsage(),
    stopReason: 'error',
    errorMessage:
      'the answer stream ended before this provider reported an answer',
    timestamp: Date.now(),
  };
}

/**
 The event to forward:
 the passing limit's terminal error rebuilt so pi retries it,
 and every other event untouched.

 @param event - event the provider's stream pushed

 @param l - logger tagged for the forwarding

 @returns the event the caller sees

 @example
 ```typescript
 const forwarded = forwardedEvent({ event, l });
 ```
 */
function forwardedEvent({
  event,
  l,
}: {
  readonly event: ForeignBorrowed<AssistantMessageEvent>;
  readonly l: Logger;
},): AssistantMessageEvent {
  if (event.type !== 'error')
    return event;
  /**
   Error text the provider's answer arrived as.
   */
  const providerAnswer = event.error
    .errorMessage;
  if (providerAnswer === undefined)
    return event;
  if (!isPassingTokenRateLimitAnswer({
    errorMessage: providerAnswer,
    logger: l,
  },))
    return event;
  // The provider's words cannot ride in the error text pi classifies:
  // its not-retried list reads `insufficient_quota` and `quota exceeded` there.
  // They are logged whole here, and the message carries them in a diagnostic.
  l.warn(
    `the provider answered with its passing token-rate limit, so pi's own automatic retry takes it: ${providerAnswer}`,
  );
  return {
    ...event,
    error: messagePiRetriesInPlaceOfPassingTokenRateLimit({
      message: event.error,
      providerAnswer,
    },),
  };
}

/**
 Forward every event of the provider's stream to the caller's stream,
 and end that stream once the provider's has ended.

 @param inner - stream the provider implementation produced

 @param outer - stream this module hands the caller

 @param model - model the answer belongs to

 @param l - logger tagged for the forwarding

 @example
 ```typescript
 void forwardEvents({ inner, outer, model, l });
 ```
 */
async function forwardEvents({
  inner,
  outer,
  model,
  l,
}: ForwardedStream,): Promise<void> {
  try {
    for await (const event of inner) {
      outer.push(forwardedEvent({
        event,
        l,
      },),);
    }
  }
  catch (error) {
    l.error(
      `the provider's answer stream failed before it reported an answer: ${caughtValueText(error,)}`,
    );
  }
  // A stream that already reported an answer ignores this push
  // (`EventStream.push` in `@earendil-works/pi-ai` returns once the stream is
  // done), so the caller settles on the provider's own answer whenever there
  // was one, and gets an answer here when the provider reported none.
  outer.push({
    type: 'error',
    reason: 'error',
    error: answerThatNeverArrived({ model, },),
  },);
  outer.end();
}

/**
 Wrap one provider answer stream so this provider's passing token-rate limit
 reaches pi as an answer pi's own automatic retry takes.

 The caller reads the returned stream exactly as it read the provider's own:
 every event arrives as it happens,
 and the result is the message the stream ended on.

 @param inner - stream the provider implementation produced

 @param model - model the answer belongs to

 @param logger - package logger, re-tagged for this seam

 @returns the stream the caller drains

 @example
 ```typescript
 const stream = streamWithPassingTokenRateLimitRetried({ inner, model, logger });
 ```
 */
export function streamWithPassingTokenRateLimitRetried({
  inner,
  model,
  logger,
}: {
  readonly inner: AssistantMessageEventStream;
  readonly model: ForeignBorrowed<Model<Api>>;
  readonly logger: Logger;
},): AssistantMessageEventStream {
  /**
   Logger tagged for this seam, so records name the stream that was forwarded.
   */
  const l = tagged({
    tag: streamWithPassingTokenRateLimitRetried.name,
    l: logger,
  },);
  /**
   Stream handed to the caller, filled as the provider's own fills.
   */
  const outer = createAssistantMessageEventStream();
  void forwardEvents({
    inner,
    outer,
    model,
    l,
  },);
  return outer;
}

//endregion Forwarding
