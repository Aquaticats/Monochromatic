/**
 Exact-model completion wrapper retaining bounded no-text recovery. @module
 */
import type {
  AssistantMessage,
  SimpleStreamOptions,
} from '@earendil-works/pi-ai';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  requestAdvisor,
  type CompleteAdvisorOptions,
} from './advisor-client.ts';
import { completeAdvisorAttempts, } from './advisor-completion.ts';

/**
 Complete one exact model through the existing bounded same-model recovery policy.
 
 @param options - selected model and request inputs
 
 @returns completed visible review
 
 @mutates options - authentication and provider callbacks consume supplied host capabilities
 
 @throws when selected model fails or returns no text twice
 
 @example
 ```ts
 const response = await completeAdvisor({ ctx, model, config, advisorContext });
 ```
 */
export async function completeAdvisor(options: ForeignHostCapability<CompleteAdvisorOptions>,): Promise<AssistantMessage> {
  /**
   Shared start retained across preparation and both allowed dispatches.
   */
  const operationStartedAtMs = options.operationStartedAtMs ?? Date.now();
  return await completeAdvisorAttempts({
    modelSlug: `${options.model
      .provider}/${options.model
        .id}`,
    timeoutMs: options.config
      .timeoutMs,
    operationStartedAtMs,
    ...(options.signal === undefined ? {} : { signal: options.signal, }),
    providerOptions: {},
    /**
     Dispatch selected model with the shared cancellation capability.
     @param attempt - provider options carrying combined caller and deadline signal
     @returns raw terminal response for exact-model classification
     @mutates attempt - provider consumes the supplied cancellation capability
     */
    complete: async function complete(attempt: ForeignHostCapability<{
      readonly providerOptions: ForeignHostCapability<SimpleStreamOptions>;
    }>,): Promise<AssistantMessage> {
      return await requestAdvisor({
        ...options,
        operationStartedAtMs,
        ...(attempt.providerOptions
          .signal
          === undefined ? {} : { signal: attempt.providerOptions
            .signal, }),
      },);
    },
  },);
}
