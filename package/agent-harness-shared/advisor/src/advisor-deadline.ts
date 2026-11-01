/**
 Shared deadline for Advisor provider attempts.
 
 @module
 */

import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { AdvisorCompletionError, } from './advisor-completion-error.ts';

//region Constants

/**
 Sentinel indicating shared Advisor deadline remains active.
 */
export const ADVISOR_DEADLINE_ACTIVE: unique symbol = Symbol(
  'advisor/deadline-remains-active-before-attempt',
);

//endregion Constants

//region Types

/**
 Deadline shared by every provider attempt in one Advisor operation.
 */
export type AdvisorDeadline = {
  /**
   Absolute operation deadline in Unix milliseconds.
   */
  readonly deadlineAtMs: number;
  /**
   Configured total operation timeout.
   */
  readonly timeoutMs: number;
  /**
   Caller cancellation signal, when supplied by Pi.
   */
  readonly callerSignal?: ForeignHostCapability<AbortSignal>;
  /**
   Timeout signal owned by Advisor.
   */
  readonly timeoutSignal: AbortSignal;
  /**
   Combined caller and timeout signal handed to providers.
   */
  readonly combinedSignal: ForeignHostCapability<AbortSignal>;
};

//endregion Types

//region Public API

/**
 Create one deadline shared by all provider attempts.
 
 @param timeoutMs - configured total timeout
 
 @param operationStartedAtMs - operation start time before context preparation
 
 @param callerSignal - caller cancellation signal
 
 @returns shared deadline state
 
 @mutates callerSignal - `AbortSignal.any` can retain dependent-signal relations
 
 @example
 ```typescript
 createAdvisorDeadline({ timeoutMs: 1_000 });
 ```
 */
export function createAdvisorDeadline(
  {
    timeoutMs,
    operationStartedAtMs = Date.now(),
    callerSignal,
  }: ForeignHostCapability<Readonly<{
    timeoutMs: number;
    operationStartedAtMs?: number;
    callerSignal?: ForeignHostCapability<AbortSignal>;
  }>>,
): AdvisorDeadline {
  /**
   Absolute deadline inherited by every attempt.
   */
  const deadlineAtMs = operationStartedAtMs + timeoutMs;
  /**
   Initial timeout delay, clamped for AbortSignal.timeout.
   */
  const initialRemainingMs = Math.max(
    1,
    Math.ceil(deadlineAtMs - Date.now(),),
  );
  /**
   Advisor-owned signal identifying deadline expiry.
   */
  const timeoutSignal = AbortSignal.timeout(initialRemainingMs,);
  return {
    deadlineAtMs,
    timeoutMs,
    ...(callerSignal === undefined ? {} : { callerSignal, }),
    timeoutSignal,
    combinedSignal: callerSignal === undefined
      ? timeoutSignal
      : AbortSignal.any([
        callerSignal,
        timeoutSignal,
      ],),
  };
}

/**
 Return positive provider timeout remaining under shared deadline.
 
 @param deadline - shared Advisor deadline
 
 @returns remaining timeout rounded up to one millisecond minimum
 
 @example
 ```typescript
 remainingAdvisorDeadlineMs(deadline);
 ```
 */
export function remainingAdvisorDeadlineMs(
  deadline: ForeignBorrowed<AdvisorDeadline>,
): number {
  return Math.max(
    1,
    Math.ceil(deadline.deadlineAtMs - Date.now(),),
  );
}

/**
 Throw when caller cancellation or Advisor deadline already ended operation.
 
 @param options - shared deadline and current attempt identity
 
 @throws {@link AdvisorCompletionError} when operation ended
 
 @example
 ```typescript
 throwIfAdvisorDeadlineEnded({ deadline, modelSlug, attempt: 1 });
 ```
 */
export function throwIfAdvisorDeadlineEnded(
  options: ForeignBorrowed<Readonly<{
    deadline: AdvisorDeadline;
    modelSlug: string;
    attempt: number;
  }>>,
): void {
  /**
   Classified cancellation or timeout, when operation ended.
   */
  const error = advisorDeadlineEndError(options,);
  if ((typeof error) !== 'symbol')
    throw error;
}

/**
 Classify caller cancellation or Advisor deadline expiry.
 
 @param deadline - shared operation deadline
 
 @param modelSlug - selected model diagnostic identity
 
 @param attempt - current provider attempt
 
 @returns completion error when operation ended, otherwise active sentinel
 
 @example
 ```typescript
 advisorDeadlineEndError({ deadline, modelSlug, attempt: 1 });
 ```
 */
export function advisorDeadlineEndError(
  {
    deadline,
    modelSlug,
    attempt,
  }: ForeignBorrowed<Readonly<{
    deadline: AdvisorDeadline;
    modelSlug: string;
    attempt: number;
  }>>,
): AdvisorCompletionError | typeof ADVISOR_DEADLINE_ACTIVE {
  if (deadline
    .callerSignal
    ?.aborted
    === true) {
    return new AdvisorCompletionError(
      `advisor: call cancelled for ${modelSlug} on attempt ${String(attempt,)}`,
    );
  }
  if (deadline
    .timeoutSignal
    .aborted
    || (Date.now() >= deadline.deadlineAtMs)) {
    return new AdvisorCompletionError(
      `advisor: call timed out after ${String(deadline.timeoutMs,)}ms for ${modelSlug} on attempt ${String(attempt,)}`,
    );
  }
  return ADVISOR_DEADLINE_ACTIVE;
}

//endregion Public API
