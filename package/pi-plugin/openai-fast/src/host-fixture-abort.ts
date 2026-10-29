/**
 Real host cancellation with a bounded, signal-owned synthetic native stream.
 
 @module
 */
import {
  createAssistantMessageEventStream,
  type Provider,
  type Api,
  type Model,
  type StreamOptions,
  type TranscriptContext,
  type AssistantMessageEventStream,
} from '@earendil-works/pi-ai';
import type { AgentSession, } from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { expect, } from '@monochromatic-dev/test-harness-core/ts';
import { fixtureAssistant, } from './host-fixture-model.ts';
import { fixtureProvider, } from './host-fixture-provider.ts';
import {
  fixtureHome,
  fixtureHost,
  requireCompanion,
} from './host-fixture-session.ts';

//region Cancellation: no timers, background work, fallback, or external transport.

/**
 Run actual session abort and verify finalized canonical native history.
 
 @example
 ```ts
 await verifyHostAbort();
 ```
 */
export async function verifyHostAbort(): Promise<void> {
  /**
   Independently disposable paths isolate the actual host cancellation lifecycle.
   */
  await using home = await fixtureHome();
  /**
   Native source owns request observations without remote refresh capability.
   */
  const source = fixtureProvider({ dynamic: false, },);
  /**
   Stream-start barrier prevents abort assertions from racing initial dispatch.
   */
  const started = Promise.withResolvers<void>();
  /**
   Synthetic native stream settles only when the real host signal is aborted.
   */
  const provider: Provider = {
    ...source.provider,
    stream: function stream(
      model: ForeignBorrowed<Model<Api>>,
      context: ForeignBorrowed<TranscriptContext>,
      options?: ForeignBorrowed<StreamOptions>,
    ): AssistantMessageEventStream {
      source.state
        .calls
        .push({
          kind: 'full',
          model,
          context,
          ...(options === undefined ? {} : { options, }),
        },);
    /**
     Native event queue remains owned by this single cancellation probe.
     */
      const events = createAssistantMessageEventStream();
    /**
     Initial native partial preserves the original model and provider identity.
     */
      const partial = fixtureAssistant({ model, },);
      events.push({
        type: 'start',
        partial,
      },);
    /**
     Actual host signal must own cancellation instead of a fixture timer.
     */
      const signal = options?.signal;
      if (signal === undefined)
        throw new Error('Actual host did not pass its cancellation signal.',);
    /**
     Cancellation emits the native error event and settles the same stream.
     */
      function aborted(): void {
      /**
       Terminal native abort response is preserved in canonical session history.
       */
        const message = fixtureAssistant({
          model,
          overrides: {
            stopReason: 'aborted',
            errorMessage: 'Synthetic request aborted.',
          },
        },);
        events.push({
          type: 'error',
          reason: 'aborted',
          error: message,
        },);
        events.end(message,);
      }
      if (signal.aborted)
        aborted();
      else
        signal.addEventListener(
          'abort',
          aborted,
          { once: true, },
        );
      started.resolve();
      return events;
    },
  };
  /**
   Actual SDK host drives prompt, cancellation, and final history reconciliation.
   */
  using host = await fixtureHost({
    home,
    source: {
      provider,
      state: source.state,
    },
  },);
  /**
   Genuine virtual selection must remain selected after the native abort.
   */
  const companion = requireCompanion({
    runtime: host.runtime,
    id: host.base
      .id,
  },);
  await host.session
    .setModel(companion,);
  /**
   In-flight host prompt remains owned until asynchronous cleanup drains it.
   */
  const pending = host.session
    .prompt('Abort this synthetic request.',);
  {
    /**
     Abort and drain host work even if an in-flight assertion fails.
     */
    await using completion = {
      [Symbol.asyncDispose]: async function dispose(): Promise<void> {
        await host.session
          .abort();
        await pending;
      },
    };
    await Promise.race([
      started.promise,
      pending,
    ],);
    expect(source.state
      .calls,)
      .toHaveLength(1,);
    await host.session
      .abort();
  }
  /**
   Finalized native abort history is inspected after host work has completed.
   */
  const assistant = host.session
    .messages
    .findLast(function assistantMessage(message: ForeignBorrowed<AgentSession['messages'][number]>) {
      return message.role === 'assistant';
    },);
  expect(assistant,)
    .toMatchObject({
      stopReason: 'aborted',
      model: host.base
        .id,
      provider: 'openai-codex',
      api: 'openai-codex-responses',
    },);
  expect(source.state
    .calls,)
    .toHaveLength(1,);
  expect(source.state
    .calls[0]
    ?.options,)
    .toHaveProperty(
    'serviceTier',
    'priority',
    );
  expect(host.session
    .model,)
    .toMatchObject({
      api: 'pi-virtual',
      provider: 'openai-codex-fast',
      id: host.base
        .id,
    },);
}

//endregion Cancellation
