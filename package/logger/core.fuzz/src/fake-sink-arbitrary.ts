/**
 fast-check arbitraries over the fake-sink scripts.

 Kept apart from `fake-sink.ts` so the sink stays a plain module the unit
 tests and the coverage driver can use without fast-check.

 @module
 */

import {
  type Arbitrary,
  array,
  boolean,
  constant,
  constantFrom,
  record,
} from 'fast-check';

import type {
  HookScript,
  SinkScript,
  VerifyOutcome,
  WriteOutcome,
} from './fake-sink.ts';

/**
 Longest head a generated hook script has; long enough for "fail, fail,
 recover" and short enough to keep counterexamples readable.
 */
const MAX_HEAD_LENGTH = 3;

/**
 Every verify outcome.
 */
const VERIFY_OUTCOMES: readonly VerifyOutcome[] = [
  'resolve-true',
  'resolve-false',
  'reject',
  'throw',
  'never',
];

/**
 Every write and flush outcome.
 */
const WRITE_OUTCOMES: readonly WriteOutcome[] = [
  'resolve',
  'reject',
  'throw',
  'never',
];

/**
 Write tails that settle, so a sink recovers after its scripted failures.
 */
const SETTLING_WRITE_TAILS: readonly WriteOutcome[] = [
  'resolve',
  'reject',
];

/**
 Generated parts of a sink script before the optional flush hook is folded
 in; `flushPresent` models absence without a nullish union.
 */
type GeneratedParts = {
  readonly flushPresent: boolean;
  readonly flushScript: HookScript<WriteOutcome>;
  readonly verify: HookScript<VerifyOutcome>;
  readonly write: HookScript<WriteOutcome>;
};

/**
 Builds a hook-script arbitrary over one outcome alphabet.

 @param outcomes - Alphabet the head draws from.

 @param tails - Alphabet the tail draws from, when narrower than `outcomes`.

 @returns Arbitrary over hook scripts.

 @example
 ```ts
 const writeScript = hookScript({ outcomes: WRITE_OUTCOMES });
 ```
 */
function hookScript<Outcome extends string,>(
  {
    outcomes,
    tails = outcomes,
  }: {
    readonly outcomes: readonly Outcome[];
    readonly tails?: readonly Outcome[];
  },
): Arbitrary<HookScript<Outcome>> {
  return record({
    head: array(
      constantFrom(...outcomes,),
      { maxLength: MAX_HEAD_LENGTH, },
    ),
    tail: constantFrom(...tails,),
  },);
}

/**
 Folds generated parts into a sink script, spreading the flush hook in only
 when present so the script satisfies `exactOptionalPropertyTypes`.

 @param parts - Generated parts.

 @returns Sink script.

 @example
 ```ts
 const script = toSinkScript({ flushPresent: false, flushScript, verify, write });
 ```
 */
function toSinkScript(parts: GeneratedParts,): SinkScript {
  return {
    ...(parts.flushPresent ? { flush: parts.flushScript, } : {}),
    verify: parts.verify,
    write: parts.write,
  };
}

/**
 Verify scripts. The logger calls `verify` once per sink, so only the first
 head entry (or the tail) ever matters; the full script shape is kept so a
 counterexample reads uniformly.

 @returns Arbitrary over verify scripts.

 @example
 ```ts
 const script = sample(verifyScript(), 1)[0];
 ```
 */
export function verifyScript(): Arbitrary<HookScript<VerifyOutcome>> {
  return hookScript({ outcomes: VERIFY_OUTCOMES, },);
}

/**
 Write scripts whose tail settles, so a sink recovers after its scripted
 failures and the exactly-once accounting has something to count.

 @returns Arbitrary over write scripts with a `resolve` or `reject` tail.

 @example
 ```ts
 const script = sample(writeScript(), 1)[0];
 ```
 */
export function writeScript(): Arbitrary<HookScript<WriteOutcome>> {
  return hookScript({
    outcomes: WRITE_OUTCOMES,
    tails: SETTLING_WRITE_TAILS,
  },);
}

/**
 Write scripts over the full alphabet, tails included, for the deadline
 properties that need a sink whose every write hangs.

 @returns Arbitrary over write scripts with any tail.

 @example
 ```ts
 const script = sample(anyWriteScript(), 1)[0];
 ```
 */
export function anyWriteScript(): Arbitrary<HookScript<WriteOutcome>> {
  return hookScript({ outcomes: WRITE_OUTCOMES, },);
}

/**
 Flush-hook scripts over the full alphabet.

 @returns Arbitrary over flush scripts.

 @example
 ```ts
 const script = sample(flushScript(), 1)[0];
 ```
 */
export function flushScript(): Arbitrary<HookScript<WriteOutcome>> {
  return hookScript({ outcomes: WRITE_OUTCOMES, },);
}

/**
 Whole sink scripts; roughly half of them expose a flush hook.

 @param write - Write-script arbitrary to use; defaults to the settling-tail
 one.

 @returns Arbitrary over sink scripts.

 @example
 ```ts
 const sinks = array(sinkScript(), { minLength: 1, maxLength: 4 });
 ```
 */
export function sinkScript(
  { write = writeScript(), }: { readonly write?: Arbitrary<HookScript<WriteOutcome>>; } = {},
): Arbitrary<SinkScript> {
  return record({
    flushPresent: boolean(),
    flushScript: flushScript(),
    verify: verifyScript(),
    write,
  },)
    .map(toSinkScript,);
}

/**
 Flush outcomes that never retire a sink.
 */
const NON_RETIRING_FLUSH_OUTCOMES: readonly WriteOutcome[] = [
  'resolve',
  'never',
];

/**
 Sink scripts that stay available for the whole run: verify answers true
 and no flush hook rejects or throws. Writes may still fail, since write
 failures never retire a sink.

 @returns Arbitrary over always-available sink scripts.

 @example
 ```ts
 const sinks = array(availableSinkScript(), { minLength: 1, maxLength: 3 });
 ```
 */
export function availableSinkScript(): Arbitrary<SinkScript> {
  return record({
    flushPresent: boolean(),
    flushScript: hookScript({ outcomes: NON_RETIRING_FLUSH_OUTCOMES, },),
    verify: constant<HookScript<VerifyOutcome>>({
      head: [],
      tail: 'resolve-true',
    },),
    write: writeScript(),
  },)
    .map(toSinkScript,);
}
