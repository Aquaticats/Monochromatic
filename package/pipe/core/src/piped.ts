import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import { runPipe, } from './run.ts';

import type {
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
  SyncStep,
} from './types.ts';

export function piped<const TInput, TStep1,>(
  args: {
    readonly value: TInput;
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly l?: Logger;
  } & NoStepsAfter1,
): TStep1;
export function piped<const TInput, TStep1, TStep2,>(
  args: {
    readonly value: TInput;
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly l?: Logger;
  } & NoStepsAfter2,
): TStep2;
export function piped<const TInput, TStep1, TStep2, TStep3,>(
  args: {
    readonly value: TInput;
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly fn3: SyncStep<TStep2, TStep3>;
    readonly l?: Logger;
  } & NoStepsAfter3,
): TStep3;
export function piped<const TInput, TStep1, TStep2, TStep3, TStep4,>(
  args: {
    readonly value: TInput;
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly fn3: SyncStep<TStep2, TStep3>;
    readonly fn4: SyncStep<TStep3, TStep4>;
    readonly l?: Logger;
  } & NoStepsAfter4,
): TStep4;
export function piped<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5,>(
  args: {
    readonly value: TInput;
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly fn3: SyncStep<TStep2, TStep3>;
    readonly fn4: SyncStep<TStep3, TStep4>;
    readonly fn5: SyncStep<TStep4, TStep5>;
    readonly l?: Logger;
  } & NoStepsAfter5,
): TStep5;
export function piped<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5, TStep6,>(
  args: {
    readonly value: TInput;
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly fn3: SyncStep<TStep2, TStep3>;
    readonly fn4: SyncStep<TStep3, TStep4>;
    readonly fn5: SyncStep<TStep4, TStep5>;
    readonly fn6: SyncStep<TStep5, TStep6>;
    readonly l?: Logger;
  } & NoStepsAfter6,
): TStep6;
export function piped<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5, TStep6, TStep7,>(
  args: {
    readonly value: TInput;
    readonly fn1: SyncStep<TInput, TStep1>;
    readonly fn2: SyncStep<TStep1, TStep2>;
    readonly fn3: SyncStep<TStep2, TStep3>;
    readonly fn4: SyncStep<TStep3, TStep4>;
    readonly fn5: SyncStep<TStep4, TStep5>;
    readonly fn6: SyncStep<TStep5, TStep6>;
    readonly fn7: SyncStep<TStep6, TStep7>;
    readonly l?: Logger;
  } & NoStepsAfter7,
): TStep7;
export function piped<const TInput, TStep1, TStep2, TStep3, TStep4, TStep5, TStep6, TStep7, TStep8,>(
  args: {
    readonly value: TInput;
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
): TStep8;
export function piped<
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
  ): TStep9;
/**
 Runs a synchronous value through one or more left-to-right pipeline steps,
 delegating to {@link runPipe}.
 
 @param args - input value, contiguous step functions, and optional logger
 
 @returns final pipeline output
 
 @throws {@link PipeStepGapError} or {@link PipeStepOverflowError} when
 runtime step keys are invalid
 
 @throws whatever pipeline step throws; the throw propagates unchanged
 
 @example
 ```ts
 import { piped } from '\@monochromatic-dev/pipe-core';
 
 const result = piped({
   value: 1,
   fn1: (input: number) => input + 1,
   fn2: (input) => String(input),
 });
 ```
 */
export function piped(args: RunArgs,): unknown {
  /**
   Logger tagged at the eager public API boundary.
   */
  const l = tagged(args.l
    === undefined
    ? { tag: piped.name, }
    : {
      tag: piped.name,
      l: args.l,
    },);

  return runPipe({
    ...args,
    l,
  },);
}
