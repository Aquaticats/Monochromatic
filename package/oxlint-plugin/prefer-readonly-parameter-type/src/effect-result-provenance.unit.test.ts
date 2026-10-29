import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  FRESH_CONTAINER_MEMBER_NAMES,
  RESULT_PROVENANCE_BY_INTERFACE,
  RESULT_RELATION_OBSERVER_RETURN,
  RESULT_RELATION_RECEIVER_ELEMENTS_PAIRED,
  RESULT_RELATION_RECEIVER_VALUE,
  VERIFIED_RESULT_RELATION_COUNT,
} from '../dist/final/node/index.mjs';

/**
 Sentinel placed in a receiver, recognised only by identity.
 
 A value equal to nothing else, so a member handing back a structurally identical
 copy fails the probe. Comparing shapes would pass for `structuredClone`.
 */
type Sentinel = { readonly marker: 'receiver-held'; };

/**
 Value only an observer can put into a result, recognised by identity.
 
 Distinct from the receiver's sentinel on purpose. A member claiming the observer-return
 relation has to hand back what the observer produced and nothing the receiver held, and
 one marker could not tell those apart.
 */
const OBSERVER_MARKER: { readonly marker: 'observer-returned'; } = {
  marker: 'observer-returned',
};

/**
 Sentinel for a result that cannot be drained because it is not iterable.
 */
const NOT_ITERABLE: unique symbol = Symbol(
  'member result is neither an array nor iterable',
);

/**
 Drains an iterable result into the elements it yields.
 
 Separate from the array case because a member claiming the element relation may hand
 back an iterator rather than a container, and the claim is the same either way: the
 object is fresh and what it yields is what the receiver holds. A non-iterable result
 cannot satisfy that claim at all, which is why it gets a sentinel rather than an empty
 array; an empty array would fail the membership half for a reason that reads as a
 missing sentinel instead of a wrong shape.
 
 @param result - Value the member handed back.
 
 @returns yielded elements, or sentinel when result cannot be iterated.
 
 @example
 ```ts
 iterableElements(new Map([['k', value,],],).values());
 ```
 */
function iterableElements(
  result: unknown,
): readonly unknown[] | typeof NOT_ITERABLE {
  if ((result === undefined) || (result === null))
    return NOT_ITERABLE;
  /**
   Iteration method the result exposes, when it exposes one.
   */
  const iterate = (result as {
    readonly [Symbol.iterator]?: unknown;
  })[Symbol.iterator];
  if ((typeof iterate) !== 'function')
    return NOT_ITERABLE;
  return [...(result as Iterable<unknown>),];
}

/**
 Builds one receiver of the named interface holding the sentinel.
 
 @param ownerName - Declaring default-library interface name.
 
 @param sentinel - Value to place inside the receiver.
 
 @returns receiver holding sentinel, and arguments reaching it.
 
 @example
 ```ts
 receiverHolding({ ownerName: 'Map', sentinel });
 ```
 */
function receiverHolding({
  ownerName,
  memberName,
  sentinel,
}: {
  readonly ownerName: string;
  readonly memberName: string;
  readonly sentinel: Sentinel;
},): {
  readonly receiver: unknown;
  readonly argumentsByMember: Readonly<Record<string, readonly unknown[]>>;
} {
  if (ownerName.endsWith('Map',))
    return {
      /* Which side of the entry holds the sentinel depends on the member, because a map
       * has two of them and its relations name different ones. `get`, `values` and
       * `entries` are claims about the value at position 1, and `keys` is a claim about
       * the key at position 0. A single receiver could only probe one of those, and the
       * other would pass or fail for a reason that has nothing to do with its entry. */
      receiver: (memberName === 'keys')
        ? new Map([[sentinel, 'held',],],)
        : new Map([['key', sentinel,],],),
      argumentsByMember: { get: ['key',], },
    };
  return {
    receiver: [sentinel,],
    argumentsByMember: {
      at: [0,],
      /* A predicate that accepts, so the probe measures what the member hands back on a
       * hit. The miss path returns `undefined` for both, which is not the sentinel and
       * would fail the comparison for a reason that says nothing about the relation. */
      filter: [function keepsEvery(): boolean {
        return true;
      },],
      /* An observer returning something the receiver never held, so a member that hands
       * back receiver elements instead fails the probe rather than passing it by
       * coincidence. `flatMap` returns it wrapped, since it flattens one level. */
      map: [function projectsMarker(): unknown {
        return OBSERVER_MARKER;
      },],
      flatMap: [function projectsMarkerList(): readonly unknown[] {
        return [OBSERVER_MARKER,];
      },],
      find: [function acceptsFirst(): boolean {
        return true;
      },],
      reduce: [
        function foldsToMarker(): unknown {
          return OBSERVER_MARKER;
        },
        'seed',
      ],
      reduceRight: [
        function foldsRightToMarker(): unknown {
          return OBSERVER_MARKER;
        },
        'seed',
      ],
      findLast: [function acceptsLast(): boolean {
        return true;
      },],
      pop: [],
      shift: [],
      slice: [],
    },
  };
}

await describe({
  name: 'collection member result provenance',
  concurrency: 1,
  children: [
    it({
      name: 'returns the identical value the receiver held, for every listed member',
      fn: async () => {
        /**
         Entries whose result was not identically the sentinel.
         */
        const notIdentical: string[] = [];
        /**
         Container entries failing either half of their own probe.
         */
        const notFreshCarrier: string[] = [];
        /**
         Observer-return entries whose result failed either half of their probe.
         */
        const notObserverDerived: string[] = [];
        /**
         Seeded-only entries whose condition turned out not to be load-bearing.
         */
        const notConditional: string[] = [];
        /**
         Paired entries whose tuples did not hold the sentinel at their position.
         */
        const notPairedCarrier: string[] = [];
        for (const [ownerName, members,] of RESULT_PROVENANCE_BY_INTERFACE) {
          for (const [memberName, provenance,] of members) {
            /**
             Fresh sentinel per member, so one member cannot pass on another's value.
             */
            const sentinel: Sentinel = { marker: 'receiver-held', };
            const { receiver, argumentsByMember, } = receiverHolding({
              ownerName,
              memberName,
              sentinel,
            },);
            /**
             Value the member handed back.
             */
            const result = (
              (receiver as Record<string, unknown>)[memberName] as (
                this: unknown,
                ...args: readonly unknown[]
              ) => unknown
            ).apply(receiver, [...argumentsByMember[memberName] ?? [],],);
            if (provenance.relation === RESULT_RELATION_RECEIVER_VALUE) {
              if (result !== sentinel)
                notIdentical.push(`${ownerName}.${memberName}`,);
              continue;
            }
            if (provenance.relation === RESULT_RELATION_OBSERVER_RETURN) {
              /* Both halves again, and here they pull apart the two containers. The result
               * must hold what the observer produced, which a member ignoring its observer
               * fails, and must hold nothing the receiver held, which `filter` fails. That
               * second half is what keeps this relation from being applied to a member whose
               * result really does carry caller-owned elements. */
              /* A member handing back one observer return rather than a container of them
               * satisfies the relation by identity where a container satisfies it by
               * membership. */
              if (Array.isArray(result,)) {
                if ((!result.includes(OBSERVER_MARKER,))
                  || result.includes(sentinel,))
                  notObserverDerived.push(`${ownerName}.${memberName}`,);
                continue;
              }
              if (result !== OBSERVER_MARKER)
                notObserverDerived.push(`${ownerName}.${memberName}`,);
              /* The condition is probed, not asserted: a `seededOnly` entry must fail its own
               * relation unseeded, or the flag is decoration. */
              if (provenance.seededOnly === true) {
                /**
                 What the same member hands back with no starting accumulator.
                 */
                const unseeded = (
                  (receiver as Record<string, unknown>)[memberName] as (
                    this: unknown,
                    ...args: readonly unknown[]
                  ) => unknown
                ).apply(receiver, [
                  function foldsToMarker(): unknown {
                    return OBSERVER_MARKER;
                  },
                ],);
                if (unseeded !== sentinel)
                  notConditional.push(`${ownerName}.${memberName}`,);
              }
              continue;
            }
            if (provenance.relation
              === RESULT_RELATION_RECEIVER_ELEMENTS_PAIRED) {
              /* Three parts, because the container probe's two cannot see the nesting.
               * The result is fresh, no element of it is the sentinel, which is what
               * separates this relation from the container one rather than merely
               * restating it, and the recorded position inside an element is. A member
               * whose pairs happen to hold the sentinel somewhere else fails, which is
               * the point: the position is what the entry claims. */
              const yielded = iterableElements(result,);
              if ((result === receiver)
                || (yielded === NOT_ITERABLE)
                || yielded.includes(sentinel,)
                || (!yielded
                  .some(function holdsAtPosition(element,): boolean {
                    return Array.isArray(element,)
                      && (element[provenance.pairedElementIndex] === sentinel);
                  },)))
                notPairedCarrier.push(`${ownerName}.${memberName}`,);
              continue;
            }
            /* Both halves, because either alone passes for the wrong value. A member
             * returning the receiver itself satisfies the membership half, and a member
             * returning an empty fresh array satisfies the freshness half, and the
             * relation claims exactly the conjunction: a new container holding what the
             * receiver holds.
             *
             * Drained rather than indexed, because an iterator carries its elements the
             * same way an array does and exposes none of them to `Array.isArray`. The
             * drain is what the relation is about for `values`: the object handed back is
             * fresh, and advancing it yields what the receiver holds. A result that is
             * neither an array nor iterable drains to nothing and fails, which is the
             * intended answer rather than a gap. */
            const drained = Array.isArray(result,)
              ? result
              : iterableElements(result,);
            if ((result === receiver)
              || (drained === NOT_ITERABLE)
              || (!drained.includes(sentinel,)))
              notFreshCarrier.push(`${ownerName}.${memberName}`,);
          }
        }
        /* A non-empty list means a listed member returns something other than the
         * value the receiver held, so crediting its result to the receiver's
         * parameter would attribute mutations to state the caller never shared.
         * Remove the entry rather than weaken this comparison. */
        expect(notIdentical,).toEqual([],);
        expect(notFreshCarrier,).toEqual([],);
        /* A non-empty list means a member claiming to build its result out of observer
         * returns either ignored the observer or handed back receiver elements. Either way
         * the relation is wrong for it, and the entry goes rather than this comparison. */
        expect(notObserverDerived,).toEqual([],);
        expect(notConditional,).toEqual([],);
        /* A non-empty list means a member claiming to yield tuples carrying a receiver
         * element at a recorded position does not, so crediting everything reachable
         * through one of its tuples to the receiver would attribute writes to state the
         * caller never shared. Remove the entry rather than widen the search. */
        expect(notPairedCarrier,).toEqual([],);
      },
    },),
    it({
      name: 'excludes fresh containers, whose elements alias while the container does not',
      fn: async () => {
        /* The reason this table is narrow, checked for every excluded member rather
         * than argued for one. Each returns a new array whose elements are the
         * receiver's: element identity holds, container identity does not. Crediting
         * the container to the receiver would attribute `copy.push(x)` to an array
         * that never received it.
         *
         * Driven through dynamic dispatch, so the assertion covers the whole
         * exclusion list and no member is spot-checked by a hand-written call. */
        /**
         Arguments letting each excluded member return without throwing.
         */
        const containerArguments: Readonly<Record<string, readonly unknown[]>> = {
          slice: [],
          concat: [],
          filter: [
            function keepAll(): boolean {
              return true;
            },
          ],
          toReversed: [],
          toSpliced: [
            0,
            0,
          ],
          with: [
            0,
            { marker: 'receiver-held', },
          ],
          flat: [],
        };
        /**
         Excluded members whose result was the receiver, or lost its element.
         */
        const notFreshContainer: string[] = [];
        for (const memberName of FRESH_CONTAINER_MEMBER_NAMES) {
          /* Absent from every owner's table, which is what "excluded" has to mean. */
          for (const [, members,] of RESULT_PROVENANCE_BY_INTERFACE)
            expect(members.has(memberName,),).toBe(false,);
          /**
           Fresh sentinel, so no member can pass on a value another left behind.
           */
          const sentinel: Sentinel = { marker: 'receiver-held', };
          /**
           Receiver whose container identity is compared against the result's.
           */
          const values: Sentinel[] = [sentinel,];
          /**
           Container the member handed back.
           */
          const result = (
            (values as unknown as Record<string, unknown>)[memberName] as (
              this: unknown,
              ...args: readonly unknown[]
            ) => unknown
          ).apply(values, [...containerArguments[memberName] ?? [],],);
          if (!Array.isArray(result,)) {
            notFreshContainer.push(`${memberName} returned no array`,);
            continue;
          }
          if (result === values)
            notFreshContainer.push(`${memberName} returned the receiver`,);
        }
        /* A non-empty list means an excluded member is not the fresh container this
         * exclusion assumes, so the reason it is excluded is wrong even if excluding
         * it happens to be safe. */
        expect(notFreshContainer,).toEqual([],);
        /* And the list is not vacuous, which an empty set would make every loop above
         * pass without checking anything. It went from seven to five when `filter` and
         * `slice` gained the container relation, and from five to four when `toReversed`
         * did: it was held back only until that probe shape was proven, and `toSorted`
         * repeating it retired the condition rather than the member changing.
         *
         * Each name that remains is held back for a reason recorded beside the set, and
         * none of them is a timing reason: mixed element sources for `concat`, `with` and
         * `toSpliced`, and a descendant rather than an element for `flat`. */
        expect(FRESH_CONTAINER_MEMBER_NAMES.size,).toBe(4,);
      },
    },),
    it({
      name: 'keeps the pinned entry count matching the table',
      fn: async () => {
        expect(
          [...RESULT_PROVENANCE_BY_INTERFACE.values(),]
            .reduce(function sumEntries(total, members,): number {
              return total + members.size;
            }, 0,),
        ).toBe(VERIFIED_RESULT_RELATION_COUNT,);
      },
    },),
  ],
},);
