/**
 Explicitly guarded faux providers for disposable Pi host verification only. @module
 */
import { once, } from 'node:events';
import { appendFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  fauxAssistantMessage,
  fauxProvider,
  fauxToolCall,
  type AssistantMessage,
  type SimpleStreamOptions,
} from '@earendil-works/pi-ai';
import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { AdvisorCompletionError, } from '../dist/final/node/index.mjs';

/**
 Register finite scripted providers only under the dedicated verification environment.
 
 @param pi - disposable host registration capability
 
 @mutates pi - registers only faux providers with no network transport
 
 @throws when invoked outside the explicitly guarded host fixture
 
 @example
 ```ts
 // Loaded only by the verify:host task in a disposable home.
 ```
 */
export default function advisorHostFixture(pi: ForeignHostCapability<ExtensionAPI>,): void {
  /**
   Guard prevents accidental loading into an ordinary user session.
   */
  const enabled = process.env
    .PI_ADVISOR_VERIFY_HOST;
  /**
   Fixture runner creates this private directory before spawning Pi.
   */
  const root = process.env
    .PI_ADVISOR_FIXTURE_ROOT;
  /**
   Fixed scenario chosen by the runner, never by a model prompt.
   */
  const mode = process.env
    .PI_ADVISOR_FIXTURE_MODE;
  if ((enabled !== '1') || (root === undefined)
    || (![
      'serial-credit',
      'explicit-credit',
      'collect',
      'straggler',
      'slash',
      'slash-error',
    ].includes(mode ?? '',)))
    throw new AdvisorCompletionError('Advisor host fixtures require the dedicated verification environment',);
  /**
   Primitive validated trace destination retained by provider callbacks.
   */
  const tracePath = join(
    root,
    'dispatch.jsonl',
  );
  /**
   Releases the initial reviewer only after the alternate has actually dispatched.
   */
  const alternateStarted = Promise.withResolvers<void>();

  /**
   Record only synthetic dispatch metadata, never prompts or credentials.
   
   @param event - fixed fixture event label
   */
  async function trace(event: string,): Promise<void> {
    await appendFile(
      tracePath,
      `${JSON.stringify({ event, },)}\n`,
    );
  }

  /**
   Scripted primary agent can issue exactly one Advisor tool call, then finish.
   */
  const main = fauxProvider({
    provider: 'fixture-main',
    api: 'fixture-main-api',
    models: [{
      id: 'main',
      maxTokens: 1_000,
    },],
  },);
  main.setResponses([
    fauxAssistantMessage(fauxToolCall(
      'advisor',
      mode === 'explicit-credit' ? { model: 'fixture-credit/a', } : {},
      { id: 'fixture-advisor-call', },
    ),),
    fauxAssistantMessage('HOST_FIXTURE_COMPLETE',),
  ],);
  /**
   Initial reviewer and a sibling whose dispatch must be blocked after exhausted credits.
   */
  const initial = fauxProvider({
    provider: 'fixture-credit',
    api: 'fixture-credit-api',
    models: [
      {
        id: 'a',
        maxTokens: 1_000,
        cost: {
          input: 10,
          output: 10,
          cacheRead: 0,
          cacheWrite: 0,
        },
      },
      {
        id: 'b',
        maxTokens: 1_000,
        cost: {
          input: 9,
          output: 9,
          cacheRead: 0,
          cacheWrite: 0,
        },
      },
    ],
  },);
  initial.setResponses([
    /**
     Supply the initial review, explicit credit failure, or cancellable straggler.
     
     @param _context - unused synthetic request
     
     @param options - operation cancellation supplied by Advisor
     
     @returns scripted terminal provider response
     
     @mutates options - cancellation observation retains its signal until abort
     */
    async function initialResponse(
      _context: unknown,
      options?: ForeignHostCapability<SimpleStreamOptions>,
    ): Promise<AssistantMessage> {
      await trace('initial-dispatch',);
      if ((mode === 'serial-credit') || (mode === 'explicit-credit')
        || (mode === 'slash-error')) {
        return fauxAssistantMessage(
          '',
          {
            stopReason: 'error',
            errorMessage: '402: {"message":"You are out of credits. PRIVATE_PROVIDER_PAYLOAD","type":"billing_error"}',
          },
        );
      }
      if (mode === 'straggler') {
        if (options?.signal === undefined)
          throw new AdvisorCompletionError('fixture expected an operation cancellation signal',);
        if (!options.signal
          .aborted)
          await once(
            options.signal,
            'abort',
          );
        await trace('initial-cancelled',);
        return fauxAssistantMessage(
          '',
          {
            stopReason: 'aborted',
            errorMessage: 'fixture cancellation acknowledged',
          },
        );
      }
      if (mode === 'collect')
        await alternateStarted.promise;
      return fauxAssistantMessage('INITIAL_REVIEW',);
    },
  ],);
  /**
   Alternate reviewer is independently registered and has a distinct provider identity.
   */
  const alternate = fauxProvider({
    provider: 'fixture-alternate',
    api: 'fixture-alternate-api',
    models: [
      {
        id: 'c',
        maxTokens: 1_000,
        cost: {
          input: 1,
          output: 1,
          cacheRead: 0,
          cacheWrite: 0,
        },
      },
    ],
  },);
  alternate.setResponses([
    /**
     Return a usable review after confirming this provider was dispatched.

     @returns synthetic alternate review
     */
    async function alternateResponse(): Promise<AssistantMessage> {
      await trace('alternate-dispatch',);
      alternateStarted.resolve();
      return fauxAssistantMessage('ALTERNATE_REVIEW',);
    },
  ],);
  pi.registerProvider(main.provider,);
  pi.registerProvider(initial.provider,);
  pi.registerProvider(alternate.provider,);
}
