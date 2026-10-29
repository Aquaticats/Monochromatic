import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import { runPipe, } from './run.ts';

import type {
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
  SyncStep,
} from './types.ts';

export function pipe<const TInput, TStep1,>(
  args: {
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly l?: Logger;
  } & NoStepsAfter1,
): (value: TInput,) => TStep1;
export function pipe<const TInput, TStep1, TStep2,>(
  args: {
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly l?: Logger;
  } & NoStepsAfter2,
): (value: TInput,) => TStep2;
export function pipe<const TInput, TStep1, TStep2, TStep3,>(
  args: {
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly fn3: SyncStep<TStep2, TStep3>;
    readonly l?: Logger;
  } & NoStepsAfter3,
): (value: TInput,) => TStep3;
export function pipe<const TInput, TStep1, TStep2, TStep3, TStep4,>(
  args: {
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly fn3: SyncStep<TStep2, TStep3>;
    readonly fn4: SyncStep<TStep3, TStep4>;
    readonly l?: Logger;
  } & NoStepsAfter4,
): (value: TInput,) => TStep4;
export function pipe<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5,>(
  args: {
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly fn3: SyncStep<TStep2, TStep3>;
    readonly fn4: SyncStep<TStep3, TStep4>;
    readonly fn5: SyncStep<TStep4, TStep5>;
    readonly l?: Logger;
  } & NoStepsAfter5,
): (value: TInput,) => TStep5;
export function pipe<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5, TStep6,>(
  args: {
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly fn3: SyncStep<TStep2, TStep3>;
    readonly fn4: SyncStep<TStep3, TStep4>;
    readonly fn5: SyncStep<TStep4, TStep5>;
    readonly fn6: SyncStep<TStep5, TStep6>;
    readonly l?: Logger;
  } & NoStepsAfter6,
): (value: TInput,) => TStep6;
export function pipe<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5, TStep6, TStep7,>(
  args: {
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly fn3: SyncStep<TStep2, TStep3>;
    readonly fn4: SyncStep<TStep3, TStep4>;
    readonly fn5: SyncStep<TStep4, TStep5>;
    readonly fn6: SyncStep<TStep5, TStep6>;
    readonly fn7: SyncStep<TStep6, TStep7>;
    readonly l?: Logger;
  } & NoStepsAfter7,
): (value: TInput,) => TStep7;
export function pipe<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5, TStep6, TStep7, TStep8,>(
  args: {
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly fn3: SyncStep<TStep2, TStep3>;
    readonly fn4: SyncStep<TStep3, TStep4>;
    readonly fn5: SyncStep<TStep4, TStep5>;
    readonly fn6: SyncStep<TStep5, TStep6>;
    readonly fn7: SyncStep<TStep6, TStep7>;
    readonly fn8: SyncStep<TStep7, TStep8>;
    readonly l?: Logger;
  } & NoStepsAfter8,
): (value: TInput,) => TStep8;
export function pipe<
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
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly fn3: SyncStep<TStep2, TStep3>;
    readonly fn4: SyncStep<TStep3, TStep4>;
    readonly fn5: SyncStep<TStep4, TStep5>;
    readonly fn6: SyncStep<TStep5, TStep6>;
    readonly fn7: SyncStep<TStep6, TStep7>;
    readonly fn8: SyncStep<TStep7, TStep8>;
    readonly fn9: SyncStep<TStep8, TStep9>;
    readonly l?: Logger;
  } & NoStepsAfter9,
  ): (value: TInput,) => TStep9;
/**
 Creates a reusable synchronous left-to-right pipeline function, delegating
 each call to {@link runPipe}.
 
 @param args - contiguous step functions and optional logger
 
 @returns function that applies captured steps to each input value
 
 @throws {@link PipeStepGapError} or {@link PipeStepOverflowError} when
 runtime step keys are invalid
 
 @throws whatever pipeline step throws; the throw propagates unchanged
 
 @example
 ```ts
 import { pipe } from '\@monochromatic-dev/pipe-core';
 
 const render = pipe({
   fn1: (input: number) => input + 1,
   fn2: (input) => String(input),
 });
 render(1);
 ```
 */
export function pipe(args: DeferredArgs,): (value: unknown,) => unknown {
  /**
   Applies captured synchronous steps to a provided value.
   
   @param value - input value supplied to the reusable pipeline
   
   @returns final pipeline output
   */
  function pipeline(value: unknown,): unknown {
    /**
     Logger tagged at the deferred public API invocation boundary.
     */
    const l = tagged(args.l
      === undefined
      ? { tag: pipe.name, }
      : {
        tag: pipe.name,
        l: args.l,
      },);

    return runPipe({
      ...args,
      value,
      l,
    },);
  }

  return pipeline;
}
