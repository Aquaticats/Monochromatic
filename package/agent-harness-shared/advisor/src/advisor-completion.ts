/**
 Bounded Advisor provider-attempt state machine.
 
 @module
 */

import type {
  AssistantMessage,
  SimpleStreamOptions,
} from '@earendil-works/pi-ai';
import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { AdvisorCompletionError, } from './advisor-completion-error.ts';
import {
  advisorDeadlineEndError as deadlineEndError,
  type AdvisorDeadline,
  createAdvisorDeadline,
  remainingAdvisorDeadlineMs as remainingDeadlineMs,
  throwIfAdvisorDeadlineEnded as throwIfDeadlineEnded,
} from './advisor-deadline.ts';

//region Constants

/**
 Maximum provider attempts allowed for one Advisor operation.
 */
const MAX_ADVISOR_ATTEMPTS = 2;

//endregion Constants

//region Types

/**
 Inputs for one bounded Advisor completion sequence.
 */
export type CompleteAdvisorAttemptsOptions = {
  /**
   Canonical selected model slug used in diagnostics.
   */
  readonly modelSlug: string;
  /**
   Configured total operation timeout.
   */
  readonly timeoutMs: number;
  /**
   Operation start time, including context preparation.
   */
  readonly operationStartedAtMs?: number;
  /**
   Caller cancellation signal from Pi.
   */
  readonly signal?: ForeignHostCapability<AbortSignal>;
  /**
   Provider options shared by every attempt.
   */
  readonly providerOptions: Readonly<Omit<SimpleStreamOptions, 'signal' | 'timeoutMs'>>;
  /**
   Provider invocation boundary supplied by Advisor client.
   */
  readonly complete: ForeignHostCapability<(options: {
    readonly providerOptions: ForeignHostCapability<SimpleStreamOptions>;
  }) => Promise<AssistantMessage>>;
};

//endregion Types

//region Public API

/**
 Complete Advisor through at most one no-text retry under a shared deadline.
 
 @param options - provider boundary, deadline, and diagnostic identity
 
 @returns successful Advisor response containing user-visible text
 
 @mutates options - provider callback consumes supplied host capabilities and `AbortSignal.any` stores dependent-signal relations
 
 @throws {@link AdvisorCompletionError} when provider fails, aborts, times out, requests a tool, or returns no text twice
 
 @example
 ```typescript
 await completeAdvisorAttempts({ modelSlug, timeoutMs, providerOptions, complete });
 ```
 */
export async function completeAdvisorAttempts(
  options: ForeignHostCapability<CompleteAdvisorAttemptsOptions>,
): Promise<AssistantMessage> {
  /**
   Shared operation deadline for every attempt.
   */
  const deadline = createAdvisorDeadline({
    timeoutMs: options.timeoutMs,
    ...(options.operationStartedAtMs === undefined
      ? {}
      : { operationStartedAtMs: options.operationStartedAtMs, }),
    ...(options.signal === undefined ? {} : { callerSignal: options.signal, }),
  },);
  /**
   First terminal provider response.
   */
  const firstResponse = await completeAdvisorAttempt({
    complete: options.complete,
    sharedProviderOptions: options.providerOptions,
    deadline,
    modelSlug: options.modelSlug,
    attempt: 1,
  },);
  if (responseHasText(firstResponse,))
    return firstResponse;
  /**
   Second terminal provider response after one successful no-text response.
   */
  const secondResponse = await completeAdvisorAttempt({
    complete: options.complete,
    sharedProviderOptions: options.providerOptions,
    deadline,
    modelSlug: options.modelSlug,
    attempt: MAX_ADVISOR_ATTEMPTS,
  },);
  if (responseHasText(secondResponse,))
    return secondResponse;

  throwIfDeadlineEnded({
    deadline,
    modelSlug: options.modelSlug,
    attempt: MAX_ADVISOR_ATTEMPTS,
  },);
  throw new AdvisorCompletionError(
    `advisor: provider ${options.modelSlug} returned no text after ${String(MAX_ADVISOR_ATTEMPTS,)} attempts`,
  );
}

//endregion Public API

//region Attempt execution

/**
 Execute and classify one provider attempt under shared deadline.
 
 @param complete - provider invocation capability
 
 @param sharedProviderOptions - options shared by every attempt
 
 @param deadline - shared operation deadline
 
 @param modelSlug - selected model identity
 
 @param attempt - current attempt number
 
 @returns successful terminal response, which may contain no text
 
 @mutates complete - provider callback can consume or retain supplied host capabilities
 
 @mutates deadline - provider callback can consume or retain combined signal capability
 
 @throws {@link AdvisorCompletionError} when provider attempt fails
 */
async function completeAdvisorAttempt(
  {
    complete,
    sharedProviderOptions,
    deadline,
    modelSlug,
    attempt,
  }: ForeignBorrowed<Readonly<{
    complete: CompleteAdvisorAttemptsOptions['complete'];
    sharedProviderOptions: CompleteAdvisorAttemptsOptions['providerOptions'];
    deadline: AdvisorDeadline;
    modelSlug: string;
    attempt: number;
  }>>,
): Promise<AssistantMessage> {
  throwIfDeadlineEnded({
    deadline,
    modelSlug,
    attempt,
  },);
  /**
   Provider options with shared signal and remaining deadline.
   */
  const attemptOptions: SimpleStreamOptions = {
    ...sharedProviderOptions,
    signal: deadline.combinedSignal,
    timeoutMs: remainingDeadlineMs(deadline,),
  };
  /**
   Terminal provider response from abort-aware boundary.
   */
  const response = await (async function invokeProvider(): Promise<AssistantMessage> {
    try {
      return await complete({
        providerOptions: attemptOptions,
      },);
    }
    catch (error) {
      throw completionErrorFromCaught({
        error,
        deadline,
        modelSlug,
        attempt,
      },);
    }
  })();

  throwForFailedResponse({
    response,
    deadline,
    modelSlug,
    attempt,
  },);
  return response;
}

//endregion Attempt execution

//region Response classification

/**
 Throw for unsuccessful terminal provider responses.
 
 @param response - terminal provider response
 
 @param deadline - shared operation deadline
 
 @param modelSlug - selected model identity
 
 @param attempt - provider attempt number
 
 @throws {@link AdvisorCompletionError} for error, abort, or unexpected tool use
 */
function throwForFailedResponse(
  {
    response,
    deadline,
    modelSlug,
    attempt,
  }: ForeignBorrowed<Readonly<{
    response: AssistantMessage;
    deadline: AdvisorDeadline;
    modelSlug: string;
    attempt: number;
  }>>,
): void {
  if (response.stopReason === 'error') {
    throw new AdvisorCompletionError(
      `advisor: provider call failed for ${modelSlug} on attempt ${String(attempt,)}: ${responseFailureText(response,)}`,
    );
  }
  if (response.stopReason === 'aborted') {
    throwIfDeadlineEnded({
      deadline,
      modelSlug,
      attempt,
    },);
    throw new AdvisorCompletionError(
      `advisor: provider aborted ${modelSlug} on attempt ${String(attempt,)}: ${responseFailureText(response,)}`,
    );
  }
  if (response.stopReason === 'toolUse') {
    throw new AdvisorCompletionError(
      `advisor: provider ${modelSlug} requested unavailable tool use on attempt ${String(attempt,)}`,
    );
  }
}

/**
 Convert thrown provider boundary value into classified Advisor failure.
 
 @param error - thrown provider boundary value
 
 @param deadline - shared operation deadline
 
 @param modelSlug - selected model identity
 
 @param attempt - provider attempt number
 
 @returns classified Advisor completion error
 */
function completionErrorFromCaught(
  {
    error,
    deadline,
    modelSlug,
    attempt,
  }: ForeignBorrowed<Readonly<{
    error: unknown;
    deadline: AdvisorDeadline;
    modelSlug: string;
    attempt: number;
  }>>,
): AdvisorCompletionError {
  /**
   Deadline failure taking precedence over provider boundary text.
   */
  const deadlineError = deadlineEndError({
    deadline,
    modelSlug,
    attempt,
  },);
  if ((typeof deadlineError) !== 'symbol')
    return deadlineError;
  /**
   Primitive provider failure text safe to retain.
   */
  const errorText = ((typeof error) === 'object')
    && (error !== null)
    && ('message' in error)
    && ((typeof error.message) === 'string')
    ? error.message
    : (typeof error) === 'string'
      ? error
      : 'unknown provider failure';
  return new AdvisorCompletionError(
    `advisor: provider call failed for ${modelSlug} on attempt ${String(attempt,)}: ${errorText}`,
  );
}

/**
 Extract provider error and redacted diagnostic text.
 
 @param response - failed provider response
 
 @returns visible diagnostic text
 */
function responseFailureText(
  response: ForeignBorrowed<AssistantMessage>,
): string {
  /**
   Trimmed provider error, when present.
   */
  const errorMessage = response.errorMessage
    ?.trim();
  /**
   Redacted provider diagnostic summary retaining primitive fields only.
   */
  const diagnostics = responseDiagnosticsText(response,);
  if ((errorMessage !== undefined) && (errorMessage !== ''))
    return diagnostics === ''
      ? errorMessage
      : `${errorMessage}; diagnostics=${diagnostics}`;
  return diagnostics === ''
    ? `stopReason=${response.stopReason}`
    : `diagnostics=${diagnostics}`;
}

/**
 Format provider diagnostics without handing borrowed object identity to unresolved code.
 
 @param response - failed provider response
 
 @returns compact primitive diagnostic summary
 */
function responseDiagnosticsText(
  response: ForeignBorrowed<AssistantMessage>,
): string {
  if (response.diagnostics === undefined)
    return '';
  return response
    .diagnostics
    .map(function formatDiagnostic(
      diagnostic: ForeignBorrowed<NonNullable<AssistantMessage['diagnostics']>[number]>,
    ) {
      /**
       Error message retained from current diagnostic.
       */
      const errorMessage = diagnostic
        .error
        ?.message;
      return errorMessage === undefined
        ? `${diagnostic.type}@${String(diagnostic.timestamp,)}`
        : `${diagnostic.type}@${String(diagnostic.timestamp,)}:${errorMessage}`;
    },)
    .join(', ',);
}

/**
 Test whether response contains user-visible Advisor text.
 
 @param response - terminal provider response
 
 @returns whether any non-empty text block exists
 */
function responseHasText(
  response: ForeignBorrowed<AssistantMessage>,
): boolean {
  return response
    .content
    .some(function hasTextContent(
      block: ForeignBorrowed<AssistantMessage['content'][number]>,
    ): boolean {
      return (block.type === 'text') && (block.text !== '');
    },);
}

//endregion Response classification
