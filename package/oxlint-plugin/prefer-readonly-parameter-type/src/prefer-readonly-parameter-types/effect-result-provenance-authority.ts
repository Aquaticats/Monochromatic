/**
 Where a collection member's result came from, for members verified by probe.
 
 Separate from `effect-member-channel-authority.ts` on purpose. That table proves
 which user-code channels run during a call; this one proves where the returned
 value came from. They are independent facts: `Array.prototype.join` reaches user
 code and returns nothing of the receiver, while `Array.prototype.at` reaches the
 same indexed channel and returns the receiver's own element. Neither answer
 implies the other, so neither table may be read as evidence for the other.
 
 @module
 */

/**
 Result is identically a value the receiver held, not a copy of one.
 
 `Map.prototype.get` hands back the stored value itself, so mutating the result
 mutates what the receiver holds. Established by identity comparison against a
 sentinel placed in the receiver, never by return-type shape: a member returning
 `T` may equally return a fresh `T`.
 */
export const RESULT_RELATION_RECEIVER_VALUE: unique symbol = Symbol(
  'collection member returns a value the receiver holds, by identity',
);

/**
 Result relation no probe has established, so it stays failing closed.
 
 Absence from the table is never a claim that a result is fresh, only that
 nothing here has shown it is receiver state.
 */
export const RESULT_RELATION_UNPROVEN: unique symbol = Symbol(
  'collection member has no verified result relation',
);

/**
 Result is a freshly built container that may hold values the receiver holds.
 
 `values.filter(kept)` hands back a new array whose elements are the receiver's own, so
 `copy[0].label = 'x'` reaches the receiver while `copy.push(row)` reaches nothing the
 caller shared. Those are different answers about one value, which is why this is a
 second relation rather than a wider reading of the first.
 
 "May hold" rather than "holds", deliberately. A predicate can reject every element, and
 a member outside this increment can mix receiver elements with argument elements. The
 relation therefore bounds what the result can carry and never asserts that it does.
 
 Established by a probe with two halves, since either alone is satisfied by the wrong
 value: the result is not the receiver, and an element of the result is the sentinel the
 receiver held.
 */
export const RESULT_RELATION_RECEIVER_ELEMENTS: unique symbol = Symbol(
  'collection member returns a fresh container that may hold receiver values',
);

/**
 Result is a freshly built container holding what the observer returned.
 
 `values.map(project)` holds no element the receiver held, whatever the receiver holds:
 every element of its result came out of the observer. So what the result carries is a
 question about the observer rather than about the receiver, and
 `propagateElementApplications` answers it from the observer's own summary, recording
 receiver opacity when the observer hands its element back.
 
 Separate from the container relation for exactly that reason. Reading `map` as a
 container of receiver elements would report every projection, including the fresh one
 issue #414 is about; reading `filter` as observer-derived would lose the elements it
 really does carry. The two are opposite answers and the members are not interchangeable.
 
 Established by a probe placing one sentinel in the receiver and returning a different one
 from the observer, so a member handing back receiver elements fails it.
 */
export const RESULT_RELATION_OBSERVER_RETURN: unique symbol = Symbol(
  'collection member returns a fresh container holding what its observer returned',
);

/**
 Result is a fresh container of fresh tuples, one position of which is a receiver element.
 
 `items.entries()` is the whole reason this exists. It yields `[index, item]` pairs which
 are freshly allocated, so no element of the result is ever identical to anything the
 receiver holds, and the container relation's probe fails for it however the relation is
 worded. The receiver's element is one level deeper, inside the pair.
 
 The recorded position proves the claim rather than bounding it. Once a pair is known to
 contain a receiver element, everything reachable through that pair can reach the
 receiver, so `callResultElementReceiver` answers with the receiver and origins flow to
 the whole pair. That distinction is load-bearing for `Map`, whose pairs hold a
 caller-owned key at position 0 and a caller-owned value at position 1: verifying either
 one establishes the claim, and flow then covers both. Reading the position as a bound
 instead would lose a write through the key of a `Map<Labelled, string>`.
 
 Established by a probe with three parts, since the first two are the container probe's
 and neither reaches the nesting: the result is not the receiver, an element of the result
 is not the sentinel, and that element's own recorded position is.
 */
export const RESULT_RELATION_RECEIVER_ELEMENTS_PAIRED: unique symbol = Symbol(
  'collection member yields newly allocated pairs, each carrying one value from the receiver',
);

/**
 Which receiver-state relation one default-library member's result satisfies.
 */
export type MemberResultRelation =
  | typeof RESULT_RELATION_RECEIVER_VALUE
    | typeof RESULT_RELATION_RECEIVER_ELEMENTS
    | typeof RESULT_RELATION_RECEIVER_ELEMENTS_PAIRED
    | typeof RESULT_RELATION_OBSERVER_RETURN;

/**
 One verified result relation, with the receiver position its result comes from.
 
 The position matters and a bare relation would lose it. `Map<K, V>.get` returns
 the `V` at position 1, and treating "any type argument of the receiver" as the
 source would let a `Map<Labelled, string>` lookup claim its `string` result
 aliases the `Labelled` key. Recording the position keeps the claim member-specific.
 */
export type MemberResultProvenance =
  | {
    readonly relation:
      | typeof RESULT_RELATION_RECEIVER_VALUE
        | typeof RESULT_RELATION_OBSERVER_RETURN;
    readonly receiverTypeArgumentIndex: number;
    /**
     Whether the relation holds only when the call supplies a starting accumulator.
     */
    readonly seededOnly?: boolean;
  }
    | {
      readonly relation: typeof RESULT_RELATION_RECEIVER_ELEMENTS;
      readonly receiverTypeArgumentIndex: number;
    /**
     Position in the result's own type arguments holding what the receiver held.
     
     Separate from the receiver's position because the two coincide only by accident.
     They agree for `filter` and `slice`, whose result is another array, and they
     disagree for `Map<K, V>.values`, whose receiver holds its values at position 1
     and whose result is a `MapIterator<V>` holding them at position 0. Reading one
     index on both sides made that entry unverifiable: the result has no position 1,
     so the comparison read `undefined` against `V` and answered with the sentinel
     for a relation that is true.
     */
      readonly resultTypeArgumentIndex: number;
    }
    | {
      readonly relation: typeof RESULT_RELATION_RECEIVER_ELEMENTS_PAIRED;
      readonly receiverTypeArgumentIndex: number;
      readonly resultTypeArgumentIndex: number;
    /**
     Position inside each yielded tuple holding what the receiver held.
     
     Proves the relation rather than bounding it, which the relation's own
     documentation explains: flow reaches the whole tuple once any position is
     shown to carry a receiver element.
     */
      readonly pairedElementIndex: number;
    };

/**
 Members whose result provenance is verified, by declaring interface.
 
 This is an authority, and it exists under the same amendment that permits
 `effect-member-channel-authority.ts`: which receiver value a member returns is a
 fact about ECMA-262 that no declaration exposes. `at` and `slice` have return
 types differing only in arity, and one aliases the receiver's element while the
 other builds a fresh array holding the same elements.
 
 That distinction is the whole reason this table is narrow. A fresh container
 holding receiver elements is not itself receiver state: crediting `values.slice()`
 to `values` would attribute `copy.push(x)` to a caller's array that never saw it.
 So only the direct-value relation is represented, and container results stay
 unproven rather than being described by a second relation nothing needs yet.
 
 Every entry is enforced by `effect-result-provenance.unit.test.ts`, which places a
 sentinel in a real receiver and compares result identity. An entry added without a
 passing identity probe is a defect.
 */
const PROVENANCE_BY_OWNER: Readonly<
  Record<string, Readonly<Record<string, MemberResultProvenance>>>
  > = {
    Array: {
      reduce: {
        relation: RESULT_RELATION_OBSERVER_RETURN,
        receiverTypeArgumentIndex: 0,
        seededOnly: true,
      },
      reduceRight: {
        relation: RESULT_RELATION_OBSERVER_RETURN,
        receiverTypeArgumentIndex: 0,
        seededOnly: true,
      },
      at: {
        relation: RESULT_RELATION_RECEIVER_VALUE,
        receiverTypeArgumentIndex: 0,
      },
      flatMap: {
        relation: RESULT_RELATION_OBSERVER_RETURN,
        receiverTypeArgumentIndex: 0,
      },
      map: {
        relation: RESULT_RELATION_OBSERVER_RETURN,
        receiverTypeArgumentIndex: 0,
      },
      filter: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
      },
      find: {
        relation: RESULT_RELATION_RECEIVER_VALUE,
        receiverTypeArgumentIndex: 0,
      },
      findLast: {
        relation: RESULT_RELATION_RECEIVER_VALUE,
        receiverTypeArgumentIndex: 0,
      },
      pop: {
        relation: RESULT_RELATION_RECEIVER_VALUE,
        receiverTypeArgumentIndex: 0,
      },
      shift: {
        relation: RESULT_RELATION_RECEIVER_VALUE,
        receiverTypeArgumentIndex: 0,
      },
      slice: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
      },
      toSorted: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
      },
      toReversed: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
      },
      entries: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS_PAIRED,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
        pairedElementIndex: 1,
      },
      values: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
      },
    },
    ReadonlyArray: {
      reduce: {
        relation: RESULT_RELATION_OBSERVER_RETURN,
        receiverTypeArgumentIndex: 0,
        seededOnly: true,
      },
      reduceRight: {
        relation: RESULT_RELATION_OBSERVER_RETURN,
        receiverTypeArgumentIndex: 0,
        seededOnly: true,
      },
      at: {
        relation: RESULT_RELATION_RECEIVER_VALUE,
        receiverTypeArgumentIndex: 0,
      },
      flatMap: {
        relation: RESULT_RELATION_OBSERVER_RETURN,
        receiverTypeArgumentIndex: 0,
      },
      map: {
        relation: RESULT_RELATION_OBSERVER_RETURN,
        receiverTypeArgumentIndex: 0,
      },
      filter: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
      },
      find: {
        relation: RESULT_RELATION_RECEIVER_VALUE,
        receiverTypeArgumentIndex: 0,
      },
      findLast: {
        relation: RESULT_RELATION_RECEIVER_VALUE,
        receiverTypeArgumentIndex: 0,
      },
      slice: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
      },
      toSorted: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
      },
      toReversed: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
      },
      entries: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS_PAIRED,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
        pairedElementIndex: 1,
      },
      values: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
      },
    },
    Map: {
      get: {
        relation: RESULT_RELATION_RECEIVER_VALUE,
        receiverTypeArgumentIndex: 1,
      },
      entries: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS_PAIRED,
        receiverTypeArgumentIndex: 1,
        resultTypeArgumentIndex: 0,
        pairedElementIndex: 1,
      },
      keys: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
      },
      values: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 1,
        resultTypeArgumentIndex: 0,
      },
    },
    ReadonlyMap: {
      get: {
        relation: RESULT_RELATION_RECEIVER_VALUE,
        receiverTypeArgumentIndex: 1,
      },
      entries: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS_PAIRED,
        receiverTypeArgumentIndex: 1,
        resultTypeArgumentIndex: 0,
        pairedElementIndex: 1,
      },
      keys: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 0,
        resultTypeArgumentIndex: 0,
      },
      values: {
        relation: RESULT_RELATION_RECEIVER_ELEMENTS,
        receiverTypeArgumentIndex: 1,
        resultTypeArgumentIndex: 0,
      },
    },
  };

/**
 Verified result relations indexed for lookup, by interface then member.
 */
export const RESULT_PROVENANCE_BY_INTERFACE: ReadonlyMap<
  string,
  ReadonlyMap<string, MemberResultProvenance>
  > = new Map(Object.entries(PROVENANCE_BY_OWNER,)
    .map(function ownerProvenance(
      [ownerName, members,],
    ): readonly [
      string,
      ReadonlyMap<string, MemberResultProvenance>,
    ] {
      return [
        ownerName,
        new Map(Object.entries(members,),),
      ];
    },),);

/**
 Total verified entries, pinned so silent table growth fails the guard.
 
 A literal in the architecture guard must match this. Adding a member therefore
 cannot pass unnoticed: the author must change a number in a second file, which is
 the point at which the identity probe becomes unavoidable.
 */
export const VERIFIED_RESULT_RELATION_COUNT = 36;

/**
 Fresh-container members still absent, each for a reason of its own.
 
 `slice` and `filter` left this list when the container relation arrived, and `toReversed`
 followed once that shape was proven. The rest stay, and not merely because nobody got to
 them:
 
 - `concat`, `with` and `toSpliced` mix receiver elements with argument elements, so
   `rows.with(0, replacement)` can hand back a container whose written element came from
   the argument. The relation would have to union two sources, and the probe would have to
   record that it is non-exclusive.
 - `flat` returns descendants rather than the receiver's immediate held type:
   `readonly (readonly Row[])[]` flattens to `Row[]`, so the type argument this authority
   records names the wrong level, and greater depths reach further.
 - `concat` additionally consults `Symbol.isConcatSpreadable` and traverses its arguments,
   which is a channel the stated trust baseline does not cover.
 `toReversed` used to be listed here, held back only until the container probe shape was
 proven. `filter` and `slice` proved it and `toSorted` repeated it, so the reason expired
 and the entry was added rather than left standing on a condition that no longer held.
 
 Their exclusion is asserted, so removing one without adding its relation fails a test.
 */
export const FRESH_CONTAINER_MEMBER_NAMES: ReadonlySet<string> = new Set([
  'concat',
  'toSpliced',
  'with',
  'flat',
],);

/**
 Resolves which result relation a collection member is verified to satisfy.
 
 @param ownerName - Declaring default-library interface name.
 
 @param memberName - Member being called.
 
 @returns verified provenance, or the unproven sentinel.
 
 @example
 ```ts
 memberResultProvenance({ ownerName: 'ReadonlyMap', memberName: 'get' });
 ```
 */
export function memberResultProvenance({
  ownerName,
  memberName,
}: {
  readonly ownerName: string;
  readonly memberName: string;
},): MemberResultProvenance | typeof RESULT_RELATION_UNPROVEN {
  return RESULT_PROVENANCE_BY_INTERFACE.get(ownerName,)
    ?.get(memberName,)
    ?? RESULT_RELATION_UNPROVEN;
}
