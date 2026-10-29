import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import { runPipeAsync, } from './run.ts';

import type {
  AsyncStep,
  DeferredArgs,
  NoStepsAfter1,
  NoStepsAfter2,
  NoStepsAfter3,
  NoStepsAfter4,
  NoStepsAfter5,
  NoStepsAfter6,
  NoStepsAfter7,
  NoStepsAfter8,
  NoStepsAfter9,
} from './types.ts';

export function pipeAsync<const TInput, TStep1,>(
  args: {
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly l?: Logger;
  } & NoStepsAfter1,
): (value: TInput | Promise<TInput>,) => Promise<Awaited<TStep1>>;
export function pipeAsync<const TInput, TStep1, TStep2,>(
  args: {
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly l?: Logger;
  } & NoStepsAfter2,
): (value: TInput | Promise<TInput>,) => Promise<Awaited<TStep2>>;
export function pipeAsync<const TInput, TStep1, TStep2, TStep3,>(
  args: {
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly fn3: AsyncStep<TStep2, TStep3>;
    readonly l?: Logger;
  } & NoStepsAfter3,
): (value: TInput | Promise<TInput>,) => Promise<Awaited<TStep3>>;
export function pipeAsync<const TInput, TStep1, TStep2, TStep3, TStep4,>(
  args: {
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly fn3: AsyncStep<TStep2, TStep3>;
    readonly fn4: AsyncStep<TStep3, TStep4>;
    readonly l?: Logger;
  } & NoStepsAfter4,
): (value: TInput | Promise<TInput>,) => Promise<Awaited<TStep4>>;
export function pipeAsync<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5,>(
  args: {
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly fn3: AsyncStep<TStep2, TStep3>;
    readonly fn4: AsyncStep<TStep3, TStep4>;
    readonly fn5: AsyncStep<TStep4, TStep5>;
    readonly l?: Logger;
  } & NoStepsAfter5,
): (value: TInput | Promise<TInput>,) => Promise<Awaited<TStep5>>;
export function pipeAsync<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5, TStep6,>(
  args: {
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly fn3: AsyncStep<TStep2, TStep3>;
    readonly fn4: AsyncStep<TStep3, TStep4>;
    readonly fn5: AsyncStep<TStep4, TStep5>;
    readonly fn6: AsyncStep<TStep5, TStep6>;
    readonly l?: Logger;
  } & NoStepsAfter6,
): (value: TInput | Promise<TInput>,) => Promise<Awaited<TStep6>>;
export function pipeAsync<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5, TStep6, TStep7,>(
  args: {
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly fn3: AsyncStep<TStep2, TStep3>;
    readonly fn4: AsyncStep<TStep3, TStep4>;
    readonly fn5: AsyncStep<TStep4, TStep5>;
    readonly fn6: AsyncStep<TStep5, TStep6>;
    readonly fn7: AsyncStep<TStep6, TStep7>;
    readonly l?: Logger;
  } & NoStepsAfter7,
): (value: TInput | Promise<TInput>,) => Promise<Awaited<TStep7>>;
export function pipeAsync<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5, TStep6, TStep7, TStep8,>(
  args: {
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly fn3: AsyncStep<TStep2, TStep3>;
    readonly fn4: AsyncStep<TStep3, TStep4>;
    readonly fn5: AsyncStep<TStep4, TStep5>;
    readonly fn6: AsyncStep<TStep5, TStep6>;
    readonly fn7: AsyncStep<TStep6, TStep7>;
    readonly fn8: AsyncStep<TStep7, TStep8>;
    readonly l?: Logger;
  } & NoStepsAfter8,
): (value: TInput | Promise<TInput>,) => Promise<Awaited<TStep8>>;
export function pipeAsync<
  const TInput,
  TStep1,
  TStep2,
  TStep3,
  TStep4,
  TStep5,
  TStep6,
  TStep7,
  TStep8,
  TStep9,
  >(
  args: {
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly fn3: AsyncStep<TStep2, TStep3>;
    readonly fn4: AsyncStep<TStep3, TStep4>;
    readonly fn5: AsyncStep<TStep4, TStep5>;
    readonly fn6: AsyncStep<TStep5, TStep6>;
    readonly fn7: AsyncStep<TStep6, TStep7>;
    readonly fn8: AsyncStep<TStep7, TStep8>;
    readonly fn9: AsyncStep<TStep8, TStep9>;
    readonly l?: Logger;
  } & NoStepsAfter9,
  ): (value: TInput | Promise<TInput>,) => Promise<Awaited<TStep9>>;
/**
 Creates a reusable asynchronous left-to-right pipeline function, delegating
 each call to {@link runPipeAsync}.
 
 @param args - contiguous step functions and optional logger
 
 @returns function that applies captured steps to each possibly promised input value
 
 @throws {@link PipeStepGapError} or {@link PipeStepOverflowError} when
 runtime step keys are invalid
 
 @throws whatever pipeline step throws or rejects with; the failure propagates unchanged
 
 @example
 ```ts
 import { pipeAsync } from '\@monochromatic-dev/pipe-core';
 
 const render = pipeAsync({
   fn1: (input: number) => input + 1,
   fn2: async (input) => String(input),
 });
 await render(Promise.resolve(1));
 ```
 */
export function pipeAsync(args: DeferredArgs,): (value: unknown,) => Promise<unknown> {
  /**
   Applies captured asynchronous steps to a provided value.
   
   @param value - input value supplied to the reusable pipeline
   
   @returns promise resolving to final pipeline output
   */
  function pipelineAsync(value: unknown,): Promise<unknown> {
    /**
     Logger tagged at the deferred async public API invocation boundary.
     */
    const l = tagged(args.l
      === undefined
      ? { tag: pipeAsync.name, }
      : {
        tag: pipeAsync.name,
        l: args.l,
      },);

    return runPipeAsync({
      ...args,
      value,
      l,
    },);
  }

  return pipelineAsync;
}
