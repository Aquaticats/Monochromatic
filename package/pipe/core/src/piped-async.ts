import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import { runPipeAsync, } from './run.ts';

import type {
  AsyncStep,
  NoStepsAfter1,
  NoStepsAfter2,
  NoStepsAfter3,
  NoStepsAfter4,
  NoStepsAfter5,
  NoStepsAfter6,
  NoStepsAfter7,
  NoStepsAfter8,
  NoStepsAfter9,
  RunArgs,
} from './types.ts';

export function pipedAsync<const TInput, TStep1,>(
  args: {
    readonly value: TInput;
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly l?: Logger;
  } & NoStepsAfter1,
): Promise<Awaited<TStep1>>;
export function pipedAsync<const TInput, TStep1, TStep2,>(
  args: {
    readonly value: TInput;
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly l?: Logger;
  } & NoStepsAfter2,
): Promise<Awaited<TStep2>>;
export function pipedAsync<const TInput, TStep1, TStep2, TStep3,>(
  args: {
    readonly value: TInput;
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly fn3: AsyncStep<TStep2, TStep3>;
    readonly l?: Logger;
  } & NoStepsAfter3,
): Promise<Awaited<TStep3>>;
export function pipedAsync<const TInput, TStep1, TStep2, TStep3, TStep4,>(
  args: {
    readonly value: TInput;
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly fn3: AsyncStep<TStep2, TStep3>;
    readonly fn4: AsyncStep<TStep3, TStep4>;
    readonly l?: Logger;
  } & NoStepsAfter4,
): Promise<Awaited<TStep4>>;
export function pipedAsync<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5,>(
  args: {
    readonly value: TInput;
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly fn3: AsyncStep<TStep2, TStep3>;
    readonly fn4: AsyncStep<TStep3, TStep4>;
    readonly fn5: AsyncStep<TStep4, TStep5>;
    readonly l?: Logger;
  } & NoStepsAfter5,
): Promise<Awaited<TStep5>>;
export function pipedAsync<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5, TStep6,>(
  args: {
    readonly value: TInput;
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly fn3: AsyncStep<TStep2, TStep3>;
    readonly fn4: AsyncStep<TStep3, TStep4>;
    readonly fn5: AsyncStep<TStep4, TStep5>;
    readonly fn6: AsyncStep<TStep5, TStep6>;
    readonly l?: Logger;
  } & NoStepsAfter6,
): Promise<Awaited<TStep6>>;
export function pipedAsync<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5, TStep6, TStep7,>(
  args: {
    readonly value: TInput;
    readonly fn1: AsyncStep<TInput, TStep1>;
    readonly fn2: AsyncStep<TStep1, TStep2>;
    readonly fn3: AsyncStep<TStep2, TStep3>;
    readonly fn4: AsyncStep<TStep3, TStep4>;
    readonly fn5: AsyncStep<TStep4, TStep5>;
    readonly fn6: AsyncStep<TStep5, TStep6>;
    readonly fn7: AsyncStep<TStep6, TStep7>;
    readonly l?: Logger;
  } & NoStepsAfter7,
): Promise<Awaited<TStep7>>;
export function pipedAsync<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5, TStep6, TStep7, TStep8,>(
  args: {
    readonly value: TInput;
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
): Promise<Awaited<TStep8>>;
export function pipedAsync<
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
    readonly value: TInput;
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
  ): Promise<Awaited<TStep9>>;
/**
 Runs a possibly promised value through asynchronous left-to-right pipeline
 steps, delegating to {@link runPipeAsync}.
 
 @param args - possibly promised input value, contiguous step functions, and optional logger
 
 @returns promise resolving to final pipeline output
 
 @throws {@link PipeStepGapError} or {@link PipeStepOverflowError} when
 runtime step keys are invalid
 
 @throws whatever pipeline step throws or rejects with; the failure propagates unchanged
 
 @example
 ```ts
 import { pipedAsync } from '\@monochromatic-dev/pipe-core';
 
 const result = await pipedAsync({
   value: Promise.resolve(1),
   fn1: (input: number) => input + 1,
   fn2: async (input) => String(input),
 });
 ```
 */
export function pipedAsync(args: RunArgs,): Promise<unknown> {
  /**
   Logger tagged at the eager async public API boundary.
   */
  const l = tagged(args.l
    === undefined
    ? { tag: pipedAsync.name, }
    : {
      tag: pipedAsync.name,
      l: args.l,
    },);

  return runPipeAsync({
    ...args,
    l,
  },);
}
