//region Stack edge
// RUNS A FUNCTION WITH ITS CALLER ALMOST OUT OF STACK, so a parse of a text
// well under the nesting bound still exhausts the stack, in the process that is
// running the test and with no flag.
//
// Why not a child process with a small `--stack-size`: Node's own module loader
// fails to start under 50 KiB, and the regular-expression engine raises a
// `SyntaxError` of its own where the stack runs short at 80 KiB, so a child can
// only be tuned to one build of the engine. Here the edge is found by bisecting
// the depth the test itself stands at, so it holds on any engine and any frame
// size.

/**
 Depth the search for a ceiling starts at, and doubles from.
 */
const FIRST_CEILING_DEPTH = 1_024;

/**
 Calls a function from a given number of frames down.

 @param depth - frames to stand below the call

 @param run - function to call at that depth

 @returns What the function returns

 @example
 ```ts
 const value = callFromDepth({ depth: 10, run: function one(): number { return 1; }, },);
 ```
 */
function callFromDepth(
  {
    depth,
    run,
  }: {
    readonly depth: number;
    readonly run: () => unknown;
  },
): unknown {
  if (depth === 0)
    return run();
  return callFromDepth({
    depth: depth - 1,
    run,
  },);
}

/**
 Whether a function runs from a given depth without throwing.

 @param depth - frames to stand below the call

 @param run - function to call at that depth

 @returns The value it threw, absent value marked by the empty array when it
 returned

 @example
 ```ts
 const thrown = thrownFromDepth({ depth: 10, run, },);
 ```
 */
function thrownFromDepth(
  {
    depth,
    run,
  }: {
    readonly depth: number;
    readonly run: () => unknown;
  },
): readonly unknown[] {
  try {
    callFromDepth({
      depth,
      run,
    },);
    return [];
  }
  catch (error) {
    return [error,];
  }
}

/**
 Does nothing, so the depth a caller can stand at is measured without a task
 of its own.

 @returns Nothing, the empty value

 @example
 ```ts
 doNothing();
 ```
 */
function doNothing(): unknown {
  return undefined;
}

/**
 Finds a depth no function can start from, by doubling.

 @returns A depth at which even a function that does nothing overflows

 @example
 ```ts
 const ceiling = ceilingDepth();
 ```
 */
function ceilingDepth(): number {
  for (let depth = FIRST_CEILING_DEPTH; ; depth *= 2) {
    /**
     What doing nothing from this depth threw, if it threw.
     */
    const thrown = thrownFromDepth({
      depth,
      run: doNothing,
    },);
    if (thrown.length > 0)
      return depth;
  }
}

/**
 Runs a function from the shallowest depth at which it throws, the deepest
 stack it can still start from, so what it throws is what a nearly spent stack
 makes of it.

 @param run - function that returns from a shallow depth and throws from a deep
 one, because it needs more stack than the deep depth leaves

 @returns What it threw at that depth

 @throws {@link Error} when it throws from no depth, or throws from the
 shallowest, which would leave the edge unfound

 @example
 ```ts
 const refusal = caughtAtStackEdge({ run: function parseDeep(): unknown { return parseMdxBody({ body, },); }, },);
 ```
 */
export function caughtAtStackEdge({ run, }: { readonly run: () => unknown; },): unknown {
  /**
   What the function threw from the shallowest depth, if it threw.
   */
  const shallow = thrownFromDepth({
    depth: 0,
    run,
  },);
  if (shallow.length > 0)
    throw new Error('the function throws from the shallowest depth, so it has no stack edge',);
  /**
   Depths the function runs from and does not, and what it threw at the
   shallowest that threw, narrowed by halving.
   */
  const search: {
    ok: number;
    failing: number;
    found: readonly unknown[];
  } = {
    ok: 0,
    failing: ceilingDepth(),
    found: [],
  };
  while ((search.failing - search.ok) > 1) {
    /**
     Depth halfway between the two.
     */
    const middle = Math.floor((search.ok + search.failing) / 2,);
    /**
     What the function threw there, if it did.
     */
    const thrown = thrownFromDepth({
      depth: middle,
      run,
    },);
    if (thrown.length > 0) {
      search.failing = middle;
      search.found = thrown;
    }
    else
      search.ok = middle;
  }
  /**
   What it threw at the shallowest depth that threw, absent for none.
   */
  const { found, } = search;
  if (found.length === 0)
    throw new Error('the function throws from no depth, so it has no stack edge',);
  return found[0];
}

//endregion Stack edge
