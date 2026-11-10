import { prepareRun, } from './prepare-run.ts';

import type { RunArgs, } from './types.ts';

/**
 Applies synchronous pipeline steps with wide implementation types.
 
 Its ladder and the one in {@link runPipeAsync} are hand-written twins, on purpose: both dispatch
 the same arities with the same step labels, and the asynchronous side differs only by awaiting
 each application, which no shared ladder can express without hiding an `await` behind a callback.
 {@link prepareRun} owns the front half both sides share, so a change to the logger, the widening,
 or the validation lands once; a change to an arity branch must be mirrored in {@link runPipeAsync}.
 
 Dispatches on the first absent `fnN` through an explicit if-chain of nested calls; no array
 indexing, no recursion. Argument validation runs before step execution so a gap or overflow logs
 under `invalid pipe arguments`, distinct from the `step failed` log reserved for a throwing step.
 
 @param args - value, contiguous step functions, and optional logger
 
 @returns final pipeline output
 
 @throws {@link PipeStepGapError} when step keys are not contiguous
 
 @throws {@link PipeStepOverflowError} when `fn10` is present
 
 @throws whatever pipeline step throws; the throw propagates unchanged
 
 @example
 ```ts
 const value = runPipe({ value: 1, fn1: (input) => input });
 ```
 */
export function runPipe(args: RunArgs,): unknown {
  /**
   Prepared call: the tagged logger, the initial value, and the validated step slots.
   */
  const {
    l,
    value,
    fn1,
    fn2,
    fn3,
    fn4,
    fn5,
    fn6,
    fn7,
    fn8,
    fn9,
  } = prepareRun({
    args,
    tag: runPipe.name,
  },);

  try {
    if (fn2 === undefined) {
      l.debug('1 step',);
      return fn1(value,);
    }
    if (fn3 === undefined) {
      l.debug('2 steps',);
      return fn2(fn1(value,),);
    }
    if (fn4 === undefined) {
      l.debug('3 steps',);
      return fn3(
        fn2(fn1(value,),),
      );
    }
    if (fn5 === undefined) {
      l.debug('4 steps',);
      return fn4(
        fn3(
          fn2(fn1(value,),),
        ),
      );
    }
    if (fn6 === undefined) {
      l.debug('5 steps',);
      return fn5(
        fn4(
          fn3(
            fn2(fn1(value,),),
          ),
        ),
      );
    }
    if (fn7 === undefined) {
      l.debug('6 steps',);
      return fn6(
        fn5(
          fn4(
            fn3(
              fn2(fn1(value,),),
            ),
          ),
        ),
      );
    }
    if (fn8 === undefined) {
      l.debug('7 steps',);
      return fn7(
        fn6(
          fn5(
            fn4(
              fn3(
                fn2(fn1(value,),),
              ),
            ),
          ),
        ),
      );
    }
    if (fn9 === undefined) {
      l.debug('8 steps',);
      return fn8(
        fn7(
          fn6(
            fn5(
              fn4(
                fn3(
                  fn2(fn1(value,),),
                ),
              ),
            ),
          ),
        ),
      );
    }

    l.debug('9 steps',);
    return fn9(
      fn8(
        fn7(
          fn6(
            fn5(
              fn4(
                fn3(
                  fn2(fn1(value,),),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
  catch (error) {
    l.error(`step failed: ${String(error,)}`,);
    throw error;
  }
}

/**
 Applies asynchronous pipeline steps with wide implementation types.
 
 Its ladder and the one in {@link runPipe} are hand-written twins, and that declaration states the
 arrangement: both dispatch the same arities with the same step labels and must move together, and
 {@link prepareRun} owns the front half both sides share.
 
 Awaits the initial value first (so a `T | Promise<T>` is accepted), awaits every intermediate
 before the next step so steps run strictly sequentially, and awaits the final application so a
 last-step rejection stays inside the execution `try` and is logged before it propagates.
 Argument validation runs before execution under the `invalid pipe arguments` log.
 
 @param args - possibly promised value, contiguous step functions, and optional logger
 
 @returns promise resolving to final pipeline output
 
 @throws {@link PipeStepGapError} when step keys are not contiguous
 
 @throws {@link PipeStepOverflowError} when `fn10` is present
 
 @throws whatever pipeline step throws or rejects with; the failure propagates unchanged
 
 @example
 ```ts
 const value = await runPipeAsync({ value: Promise.resolve(1), fn1: (input) => input });
 ```
 */
export async function runPipeAsync(args: RunArgs,): Promise<unknown> {
  /**
   Prepared call: the tagged logger, the possibly promised value, and the validated step slots.
   */
  const {
    l,
    value,
    fn1,
    fn2,
    fn3,
    fn4,
    fn5,
    fn6,
    fn7,
    fn8,
    fn9,
  } = prepareRun({
    args,
    tag: runPipeAsync.name,
  },);

  try {
    if (fn2 === undefined) {
      l.debug('1 step',);
      return await fn1(await value,);
    }
    if (fn3 === undefined) {
      l.debug('2 steps',);
      return await fn2(await fn1(await value,),);
    }
    if (fn4 === undefined) {
      l.debug('3 steps',);
      return await fn3(
        await fn2(await fn1(await value,),),
      );
    }
    if (fn5 === undefined) {
      l.debug('4 steps',);
      return await fn4(
        await fn3(
          await fn2(await fn1(await value,),),
        ),
      );
    }
    if (fn6 === undefined) {
      l.debug('5 steps',);
      return await fn5(
        await fn4(
          await fn3(
            await fn2(await fn1(await value,),),
          ),
        ),
      );
    }
    if (fn7 === undefined) {
      l.debug('6 steps',);
      return await fn6(
        await fn5(
          await fn4(
            await fn3(
              await fn2(await fn1(await value,),),
            ),
          ),
        ),
      );
    }
    if (fn8 === undefined) {
      l.debug('7 steps',);
      return await fn7(
        await fn6(
          await fn5(
            await fn4(
              await fn3(
                await fn2(await fn1(await value,),),
              ),
            ),
          ),
        ),
      );
    }
    if (fn9 === undefined) {
      l.debug('8 steps',);
      return await fn8(
        await fn7(
          await fn6(
            await fn5(
              await fn4(
                await fn3(
                  await fn2(await fn1(await value,),),
                ),
              ),
            ),
          ),
        ),
      );
    }

    l.debug('9 steps',);
    return await fn9(
      await fn8(
        await fn7(
          await fn6(
            await fn5(
              await fn4(
                await fn3(
                  await fn2(await fn1(await value,),),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
  catch (error) {
    l.error(`step failed: ${String(error,)}`,);
    throw error;
  }
}
