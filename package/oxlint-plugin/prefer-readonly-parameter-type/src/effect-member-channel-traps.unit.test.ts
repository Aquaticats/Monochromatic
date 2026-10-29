import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  MEMBER_CHANNEL_RECEIVER_INDEX,
  MEMBER_CHANNELS_BY_INTERFACE,
} from '../dist/final/node/index.mjs';

/**
 Proxy operations a plain indexed access on a parameter already opens.
 
 Measured against the rule itself rather than assumed, because the own-index channel
 is defined as "no wider than what indexed access opens" and that set turned out to
 be wider than the `Get` and `Set` an earlier description named. Each entry
 corresponds to source this rule accepts on a parameter with no unresolved effect:
 `values[0]` opens `get`, `0 in values` opens `has`, `values[0] = x` opens `set`,
 `getOwnPropertyDescriptor` and `defineProperty`, and `delete values[0]` opens
 `deleteProperty`. The last two are reported as plain mutations, never as something
 the rule could not resolve.
 */
const INDEXED_ACCESS_TRAPS: ReadonlySet<string> = new Set([
  'get',
  'has',
  'set',
  'defineProperty',
  'getOwnPropertyDescriptor',
  'deleteProperty',
],);

/**
 Trap names one probed operation reached.
 */
type TrapHits = string[];

/**
 Arguments letting each probed member do its real work on a two-element receiver.
 
 Plain strings, because this probe measures which operations reach the receiver
 rather than what a member coerces. Element coercion has its own probe in
 `effect-member-channel-authority.unit.test.ts`.
 
 @param memberName - Member being invoked.
 
 @returns argument list for the call.
 
 @example
 ```ts
 trapProbeArguments({ memberName: 'with', });
 ```
 */
function trapProbeArguments(
  { memberName, }: { readonly memberName: string; },
): readonly unknown[] {
  /**
   Members needing an index or a value to reach past their first step.
   */
  const byMember: Record<string, readonly unknown[]> = {
    at: [0,],
    includes: ['a',],
    indexOf: ['a',],
    lastIndexOf: ['a',],
    with: [0, 'z',],
    toSpliced: [0, 1, 'z',],
    push: ['z',],
    unshift: ['z',],
    fill: ['z',],
    copyWithin: [0, 1,],
  };
  return byMember[memberName] ?? [];
}

/**
 Builds a handler recording every operation before forwarding it unchanged.
 
 Each trap is written out with its own signature rather than collected through rest
 parameters, so no forwarding call can silently receive the wrong argument at the
 wrong position: `get` takes the receiver third while `set` takes the value there.
 
 `ownKeys` and `getPrototypeOf` are instrumented precisely because they lie outside
 the indexed-access baseline. A member reaching either is doing something no source
 construct this rule accepts would do, and must leave the channel.
 
 @param hits - Accumulator recording every trap reached.
 
 @returns proxy handler forwarding every operation to the target.
 
 @example
 ```ts
 new Proxy(['a'], recordingHandler({ hits: [], },),);
 ```
 */
function recordingHandler(
  { hits, }: { readonly hits: TrapHits; },
): ProxyHandler<unknown[]> {
  return {
    get(target, key, receiver,) {
      hits.push('get',);
      // `Reflect.get` is typed `any`, and forwarding that unchanged is what the trap
      // has to do, so the value is narrowed to `unknown` at this boundary instead.
      return Reflect.get(target, key, receiver,) as unknown;
    },
    set(target, key, value, receiver,) {
      hits.push('set',);
      return Reflect.set(target, key, value, receiver,);
    },
    has(target, key,) {
      hits.push('has',);
      return Reflect.has(target, key,);
    },
    deleteProperty(target, key,) {
      hits.push('deleteProperty',);
      return Reflect.deleteProperty(target, key,);
    },
    defineProperty(target, key, descriptor,) {
      hits.push('defineProperty',);
      return Reflect.defineProperty(target, key, descriptor,);
    },
    getOwnPropertyDescriptor(target, key,) {
      hits.push('getOwnPropertyDescriptor',);
      return Reflect.getOwnPropertyDescriptor(target, key,);
    },
    ownKeys(target,) {
      hits.push('ownKeys',);
      return Reflect.ownKeys(target,);
    },
    getPrototypeOf(target,) {
      hits.push('getPrototypeOf',);
      return Reflect.getPrototypeOf(target,);
    },
  };
}

/**
 Steps a drain may take before the probe treats the iterator as non-terminating.
 */
const DRAIN_STEP_LIMIT = 16;

/**
 Applies one member to a trapped receiver and exhausts any iterator it returns.
 
 Drainage is where an iterator member reaches its receiver, so applying without
 draining would report a clean run for `keys`, `values` and `entries` while measuring
 only the producer. The terminal step reporting `done` is included, since that step
 reads the receiver's length one last time.
 
 @param receiver - Trapped receiver the member is applied to.
 
 @param memberName - Member being invoked.
 
 @throws Error when a result keeps yielding past the runaway guard.
 
 @example
 ```ts
 applyAndDrain({ receiver, memberName: 'entries', });
 ```
 */
function applyAndDrain({
  receiver,
  memberName,
}: {
  readonly receiver: unknown;
  readonly memberName: string;
},): void {
  /**
   Value the member handed back, drained when it is an iterator.
   */
  const result = (
    (receiver as Record<string, unknown>)[memberName] as (
      this: unknown,
      ...args: readonly unknown[]
    ) => unknown
  ).apply(receiver, [...trapProbeArguments({ memberName, },),],);
  if ((result === null) || ((typeof result) !== 'object'))
    return;
  /**
   Advancing member, present only when the result is an iterator.
   */
  const advance = (result as { readonly next?: unknown; }).next;
  if ((typeof advance) !== 'function')
    return;
  /**
   Drain position, tracking exhaustion and guarding against a non-terminating result.
   */
  const drain = {
    done: false,
    steps: 0,
  };
  while (!drain.done) {
    drain.steps++;
    if (drain.steps > DRAIN_STEP_LIMIT)
      throw new Error(
        `A probed iterator did not finish within ${DRAIN_STEP_LIMIT} steps, so the drain measured an unbounded operation rather than one array's worth of reads.`,
      );
    drain.done = (advance as (this: unknown,) => { readonly done?: boolean; })
      .call(result,)
      .done === true;
  }
}

/**
 Runs one operation against a fully trapped array and reports what it reached.
 
 @param operate - Operation performed on the trapped receiver.
 
 @returns trap names reached, deduplicated and sorted.
 
 @example
 ```ts
 reachedTraps({ operate(receiver,) { void (receiver as readonly unknown[])[0]; }, });
 ```
 */
function reachedTraps({
  operate,
}: {
  readonly operate: (receiver: unknown,) => void;
},): readonly string[] {
  /**
   Traps reached by this single operation.
   */
  const hits: TrapHits = [];
  operate(new Proxy([
    'a',
    'b',
  ], recordingHandler({ hits, },),),);
  return [...new Set(hits,),].toSorted();
}

await describe({
  name: 'collection member proxy-visible operations',
  concurrency: 1,
  children: [
    it({
      name: 'opens no operation that indexed access on a parameter does not',
      fn: async () => {
        /**
         Members reaching an operation outside the indexed-access baseline.
         */
        const wider: string[] = [];
        for (const [ownerName, members,] of MEMBER_CHANNELS_BY_INTERFACE) {
          if (!ownerName.endsWith('Array',))
            continue;
          for (const [memberName, channel,] of members) {
            if (channel !== MEMBER_CHANNEL_RECEIVER_INDEX)
              continue;
            /**
             Operations this member reached through the trapped receiver.
             */
            const traps = reachedTraps({
              operate(receiver,): void {
                applyAndDrain({
                  receiver,
                  memberName,
                },);
              },
            },);
            /**
             Operations the baseline does not admit.
             */
            const outside = traps.filter(function notBaseline(trapName,): boolean {
              return !INDEXED_ACCESS_TRAPS.has(trapName,);
            },);
            if (outside.length > 0)
              wider.push(`${ownerName}.${memberName} reached ${outside.join(', ',)}`,);
          }
        }
        /* A non-empty list means a member reaches the receiver in a way no source
         * construct this rule accepts would, so the own-index channel no longer
         * describes it. Move the entry out rather than widen this baseline: the
         * baseline is measured against the rule's own behaviour, not chosen. */
        expect(wider,).toEqual([],);
      },
    },),
    it({
      name: 'derives the baseline from operations the rule accepts on a parameter',
      fn: async () => {
        /* Controls, and the derivation of `INDEXED_ACCESS_TRAPS` in executable form.
         * Every operation below is one the rule was measured to accept on a parameter
         * with no unresolved effect: reads and `in` keep a read-only offer, while the
         * write and the delete are reported as plain mutations. */
        expect(reachedTraps({
          operate(receiver,): void {
            void (receiver as readonly unknown[])[0];
          },
        },),).toEqual(['get',],);
        expect(reachedTraps({
          operate(receiver,): void {
            void (0 in (receiver as readonly unknown[]));
          },
        },),).toEqual(['has',],);
        expect(reachedTraps({
          operate(receiver,): void {
            (receiver as unknown[])[0] = 'z';
          },
        },),).toEqual([
          'defineProperty',
          'getOwnPropertyDescriptor',
          'set',
        ],);
        /* `Reflect.deleteProperty` rather than `delete values[0]`, which this repo
         * forbids on an array. Both perform the same `[[Delete]]`, and the source
         * construct whose acceptance defines this baseline entry is the `delete`
         * form, measured against the rule separately. */
        expect(reachedTraps({
          operate(receiver,): void {
            Reflect.deleteProperty(receiver as unknown[], 0,);
          },
        },),).toEqual(['deleteProperty',],);
        /* And the two traps deliberately instrumented but never admitted, so a member
         * reaching either fails the assertion above rather than passing unseen. */
        expect(INDEXED_ACCESS_TRAPS.has('ownKeys',),).toBe(false,);
        expect(INDEXED_ACCESS_TRAPS.has('getPrototypeOf',),).toBe(false,);
        expect(reachedTraps({
          operate(receiver,): void {
            Object.keys(receiver as readonly unknown[],);
          },
        },).includes('ownKeys',),).toBe(true,);
        /* The drain is instrumented, and this is the only probe that can show it.
         * `Array.prototype.keys` yields indices without fetching an element, so the
         * sibling accessor probe observes nothing while draining it. What it does read
         * is `length`, which surfaces here as `get` because a `Proxy` traps a property
         * that a real array will not let anyone instrument. Without this control, a
         * drain that silently stopped advancing would report a clean run for every
         * iterator member. */
        expect(reachedTraps({
          operate(receiver,): void {
            applyAndDrain({
              receiver,
              memberName: 'keys',
            },);
          },
        },),).toEqual(['get',],);
      },
    },),
    it({
      name: 'cannot probe internal-slot members this way, which is why they rest on the specification',
      fn: async () => {
        /**
         Reports whether applying a `Map` member to a proxied receiver throws.
         
         @returns outcome of the attempt.
         
         @example
         ```ts
         proxiedMapOutcome();
         ```
         */
        function proxiedMapOutcome(): string {
          try {
            Map.prototype.get
              .call(new Proxy(new Map(), {},), 'k',);
            return 'no throw';
          }
          catch (error: unknown) {
            return `threw:${caughtValueText(error,)}`;
          }
        }
        /**
         Outcome of applying `Map.prototype.get` to a proxied receiver.
         */
        const outcome = proxiedMapOutcome();
        /* A `Proxy` cannot stand in for a `Map` or `Set` receiver, because these
         * members require the internal slot and reject anything without it. So no
         * general property trap can watch them, and the internal-slot claim rests on
         * the specification while the `size` accessor in the sibling probe guards the
         * one channel that is observable. */
        expect(outcome.startsWith('threw:',),).toBe(true,);
      },
    },),
  ],
},);
