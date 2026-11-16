/**
 Waits for what a call answers, and stops waiting when the bound runs out.

 Why: a filesystem call that has stopped answering is not something this process can cancel, and the work that asks for one happens
 outside any unit of the run's own, so nothing else ends it. A bound turns a wait that would never end into a refusal that names
 what did not answer, and it is stopped the moment the answer arrives, so nothing of it keeps the process alive afterwards.

 What the bound does not reach: work the kernel has already been asked for and will not answer. When the bound fires, the request it
 interrupted is left with the kernel, which holds it until the filesystem answers or the process ends; this process stops waiting
 for it and nothing later waits on it.

 @module
 */

import { setTimeout as wait, } from 'node:timers/promises';

/**
 How long a call may take before this process stops waiting for it, and what to say when it does.
 */
export type BoundOnAnAnswer<Answer> = {
  /**
   The call, already started.
   */
  readonly call: Promise<Answer>;

  /**
   How long the call may take, in milliseconds.
   */
  readonly milliseconds: number;

  /**
   The refusal to throw when the bound has passed, made when it is thrown so that it names the input and the bound.
   */
  readonly refusal: () => Error;
};

/**
 Answers a call within a bound, or refuses it in the words the caller gave.

 @param call - the call, already started

 @param milliseconds - how long the call may take

 @param refusal - the refusal to throw when the bound has passed, made when it is thrown

 @returns what the call answered

 @throws the error the call threw

 @throws the error `refusal` makes when the call did not answer inside the bound

 @example
 ```ts
 const text = await answerWithinABound({ call: readFile('/runs/a-1/summary.json', 'utf8'), milliseconds: 30_000, refusal: () => new ReadThatTookTooLongError('reading the summary took longer than 30000 ms'), });
 ```
 */
export async function answerWithinABound<Answer>({
  call,
  milliseconds,
  refusal,
}: BoundOnAnAnswer<Answer>,): Promise<Answer> {
  /**
   The wait this call is given, stopped when the call answers so that nothing of it keeps this process alive after the answer is in
   hand.
   */
  using bound = aBoundOnTheWait({
    milliseconds,
    refusal,
  },);
  /**
   What the call answered, either way it ends, so a call the bound has answered for never rejects unhandled.
   */
  const settled = await Promise.race([
    settleBy({ call, },),
    bound.promise,
  ],);
  if (settled.answered) {
    return settled.value;
  }
  throw settled.failed;
}

/**
 Makes the bound an answer is given: a refusal once the bound has passed, and a way to stop it when the answer arrives first.

 @param milliseconds - the bound

 @param refusal - the refusal to throw, made when the bound passes

 @returns the refusal the call is raced against, and the way to stop the timer behind it

 @example
 ```ts
 using bound = aBoundOnTheWait({ milliseconds: 30_000, refusal: () => new Error('it took too long') });
 ```
 */
function aBoundOnTheWait({
  milliseconds,
  refusal,
}: {
  readonly milliseconds: number;
  readonly refusal: () => Error;
},): {
  readonly promise: Promise<never>;
  readonly [Symbol.dispose]: () => void;
} {
  /**
   The signal that stops the timer, and makes the promise never settle, which is what `using` does when the answer arrives first.
   */
  const stopping = new AbortController();
  return {
    promise: waitOutTheBound({
      milliseconds,
      refusal,
      signal: stopping.signal,
    },),
    /**
     Stops the timer, so nothing of it keeps this process alive after the answer is in hand.
     */
    [Symbol.dispose]: function dispose() {
      stopping.abort();
    },
  };
}

/**
 Waits out the bound, then refuses the answer that did not come inside it.

 @param milliseconds - the bound it waits out

 @param refusal - the refusal to throw when the bound has passed

 @param signal - the signal that stops the wait when the answer arrives first

 @throws the error `refusal` makes when the bound has passed

 @example
 ```ts
 const refusal = waitOutTheBound({ milliseconds: 30_000, refusal: () => new Error('it took too long'), signal: new AbortController().signal, });
 ```
 */
async function waitOutTheBound({
  milliseconds,
  refusal,
  signal,
}: {
  readonly milliseconds: number;
  readonly refusal: () => Error;
  readonly signal: AbortSignal;
},): Promise<never> {
  await wait(
    milliseconds,
    undefined,
    { signal, },
  );
  throw refusal();
}

/**
 What a call answered, or what it threw, so a call that may refuse is read as a value.

 @param call - the call, already started

 @returns its answer, or the error it threw

 @example
 ```ts
 const settled = await settleBy({ call: readFile('/runs/a-1/summary.json', 'utf8'), });
 ```
 */
async function settleBy<Answer>({ call, }: { readonly call: Promise<Answer> },): Promise<
  | {
    readonly answered: true;
    readonly value: Answer
  }
    | {
      readonly answered: false;
      readonly failed: Error
    }
  > {
  try {
    return {
      answered: true,
      value: await call,
    };
  }
  catch (error) {
    // Anything a call throws is reported as an error, so the caller has one shape to throw onward.
    return {
      answered: false,
      failed: (Error.isError(error,)) ? error : new Error(String(error,)),
    };
  }
}
