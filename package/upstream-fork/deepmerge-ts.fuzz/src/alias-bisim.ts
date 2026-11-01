/**
 Graph-aware oracle: does a merge result unfold to the merge of its inputs?

 The reference model in `./model.ts` recurses into values and so only
 terminates on trees. Cyclic inputs denote infinite regular trees; merging
 them position by position (records key-wise, arrays concatenated, Sets
 unioned, Maps key-wise, last value otherwise, `undefined` filtered) yields
 another infinite regular tree. A result graph is correct when it unfolds to
 that tree, which this module checks coinductively: a worklist of
 (result node, input values) pairs with a visited set, so a result cycle that
 revisits a pair already under check is accepted.

 In `upstream-rule` mode, cyclic positions are held to upstream's own cycle
 rule (`./alias-cycle-rule.ts`) instead of the unfolding, so only departures
 from upstream's design are reported; `unfolding` mode reports the design
 divergence itself.

 @module
 */

import {
  childSources,
  cycleRule,
  triggersFalseCycle,
  type Frame,
  type Sourced,
} from './alias-cycle-rule.ts';
import {
  childrenOf,
  isContainer,
} from './alias-walk.ts';
import { kindOf, } from './model.ts';

/**
 Verdict of one check.
 */
export type Verdict = {
  /**
   First mismatch, empty when the result is correct.
   */
  readonly mismatch: string;
  /**
   Whether the known false-cycle region was reached.
   */
  readonly falseCycle: boolean;
  /**
   Whether upstream's cycle rule governed some position (`upstream-rule` only).
   */
  readonly cycleRemap: boolean;
};

/**
 One pending comparison.
 */
type Pending = {
  readonly result: unknown;
  readonly sources: readonly Sourced[];
  readonly ancestors: readonly Frame[];
  readonly path: string;
};

/**
 Flags gathered while walking, returned with the verdict.
 */
type Flags = {
  falseCycle: boolean;
  cycleRemap: boolean;
};

/**
 Same values by identity, in order.

 @param actual - Actual elements.
 
 @param expected - Expected elements.

 @returns Whether both lists match element-wise with `Object.is`.

 @example
 ```ts
 sameElements({ actual: [1,], expected: [1,], }); // true
 ```
 */
function sameElements({
  actual,
  expected,
}: {
  readonly actual: readonly unknown[];
  readonly expected: readonly unknown[]
},): boolean {
  return (actual.length === expected.length) && actual.every(function matches(
    element,
    index,
  ) {
    return Object.is(
      element,
      expected[index],
    );
  },);
}

/**
 Elements of array or Set sources, concatenated in input order.

 @param sources - Array or Set values.

 @returns Their elements in iteration order.

 @example
 ```ts
 elementsOf([{ input: 0, value: [1,], },]); // [1]
 ```
 */
function elementsOf(sources: readonly Sourced[],): readonly unknown[] {
  return sources.flatMap(function elements(source,) {
    return isContainer(source.value,)
      ? childrenOf(source.value,)
        .map(function valueOf([, element,],) {
          return element;
        },)
      : [];
  },);
}

/**
 Keys a merged record or Map must have, in first-seen order: Map keys, or
 enumerable own record keys.

 @param sources - Record or Map values.

 @returns Union of their keys.

 @example
 ```ts
 keysOf([{ input: 0, value: { a: 1, }, },]); // ['a']
 ```
 */
function keysOf(sources: readonly Sourced[],): readonly unknown[] {
  return [...new Set(sources.flatMap(function keys(source,) {
    /**
     Container of this source.
     */
    const container = source.value;
    if (!isContainer(container,))
      return [];
    return childrenOf(container,)
      .filter(function enumerable([key,],) {
        return (container instanceof Map)
          || ((((typeof key) === 'string') || ((typeof key) === 'symbol'))
            && Object.prototype
              .propertyIsEnumerable
              .call(
              container,
              key,
              ));
      },)
      .map(function keyOf([key,],) {
        return key;
      },);
  },),),];
}

/**
 Check one pending pair, queueing its children.

 @param item - Pair under check.
 
 @param cycles - Oracle mode.
 
 @param flags - Walk flags, updated in place.
 
 @param enqueue - Adds a child pair to the worklist.
 
 @param firstVisit - Whether this (result node, merged values) pair is new.

 @returns Mismatch for this pair, or an empty string.

 @example
 ```ts
 checkPair({ item, cycles, flags, enqueue, firstVisit, });
 ```
 */
function checkPair(
  {
    item,
    cycles,
    flags,
    enqueue,
    firstVisit,
  }: {
    readonly item: Pending;
    readonly cycles: 'unfolding' | 'upstream-rule';
    readonly flags: Flags;
    readonly enqueue: (pending: Pending,) => void;
    readonly firstVisit: (pair: {
      readonly node: object;
      readonly merged: readonly Sourced[]
    },) => boolean;
  },
): string {
  /**
   Sources left after the default `undefined` filter.
   */
  const present = item.sources
    .filter(function isPresent(source,) {
      return source.value !== undefined;
    },);
  if (present.some(function foreign(source,) {
    return isContainer(source.value,) && triggersFalseCycle({
      ancestors: item.ancestors,
      source,
    },);
  },))
    flags.falseCycle = true;
  /**
   Last present source, which wins whenever merging stops.
   */
  const last = present.at(-1,);
  if (last === undefined)
    return item.result === undefined ? '' : `${item.path}: expected undefined`;
  /**
   Upstream's cycle rule for this position, when checking against it.
   */
  const rule = cycles === 'upstream-rule' ? cycleRule({
    ancestors: item.ancestors,
    present,
  },) : { kind: 'merge', } as const;
  if (rule.kind !== 'merge')
    flags.cycleRemap = true;
  if (rule.kind === 'ancestor')
    return item.result === rule.result ? '' : `${item.path}: cyclic position not resolved to its ancestor's result`;
  /**
   Whether every present value shares one mergeable kind.
   */
  const mergeable = present.every(function sameKind(source,) {
    return (kindOf(source.value,) === kindOf(last.value,)) && isContainer(source.value,);
  },);
  /**
   Sources actually merged here.
   */
  const merged = (rule.kind === 'resolve-last') ? [rule.last,] : (mergeable ? present : [last,]);
  /**
   Value every merged source's kind must match.
   */
  const first = merged[0]
    ?.value;
  if ((merged.length === 1) && Object.is(
    item.result,
    first,
  ))
    return '';
  if (!isContainer(first,))
    return `${item.path}: expected the last value ${String(first,)}`;
  if ((!isContainer(item.result,)) || (kindOf(item.result,) !== kindOf(first,)))
    return `${item.path}: kind ${kindOf(item.result,)} instead of ${kindOf(first,)}`;
  if (!firstVisit({
    merged,
    node: item.result,
  },))
    return '';
  /**
   Result node as a source, for the shared key and element readers.
   */
  const self = [{
    input: 0,
    value: item.result,
  },];
  if ((item.result instanceof Set) || Array.isArray(item.result,)) {
    /**
     Elements the result must hold, Sets de-duplicated.
     */
    const expected = item.result instanceof Set ? [...new Set(elementsOf(merged,),),] : elementsOf(merged,);
    return sameElements({
      actual: elementsOf(self,),
      expected,
    },) ? '' : `${item.path}: elements differ`;
  }
  /**
   Keys the result must have.
   */
  const keys = keysOf(merged,);
  /**
   Keys the result node has.
   */
  const actualKeys = keysOf(self,);
  if (!sameElements({
    actual: actualKeys,
    expected: keys,
  },))
    return `${item.path}: keys ${actualKeys.map(String,)
      .join(',',)} instead of ${keys.map(String,)
        .join(',',)}`;
  /**
   Frames for this node's children.
   */
  const ancestors = [
    ...item.ancestors,
    {
      result: item.result,
      sources: merged,
    },
  ];
  for (const key of keys) {
    enqueue({
      ancestors,
      path: `${item.path}.${String(key,)}`,
      result: childSources({
        key,
        sources: self,
      },)[0]
        ?.value,
      sources: childSources({
        key,
        sources: merged,
      },),
    },);
  }
  return '';
}

/**
 Check a result against the merge of the inputs' unfoldings.

 @param result - Value returned by (or target mutated by) the merge.
 
 @param values - Merge inputs in argument order.
 
 @param cycles - `unfolding` holds cyclic positions to the unfolding;
   `upstream-rule` holds them to upstream's cycle rule instead.

 @returns First mismatch and the regions the walk reached.

 @example
 ```ts
 checkUnfolding({ result: deepmerge(a, b), values: [a, b,], cycles: 'upstream-rule', });
 ```
 */
export function checkUnfolding(
  {
    result,
    values,
    cycles,
  }: {
    readonly result: unknown;
    readonly values: readonly unknown[];
    readonly cycles: 'unfolding' | 'upstream-rule';
  },
): Verdict {
  /**
   Walk flags.
   */
  const flags: Flags = {
    cycleRemap: false,
    falseCycle: false,
  };
  /**
   Visited (result node, merged values) pairs, keyed by identity numbers.
   */
  const visited = new Map<object, Set<string>>();
  /**
   Identity numbers for visited keys.
   */
  const ids = new Map<unknown, number>();
  /**
   Pending comparisons, processed depth-first.
   */
  const pending: Pending[] = [{
    ancestors: [],
    path: '$',
    result,
    sources: values.map(function tag(
      value,
      input,
    ) {
      return {
        input,
        value,
      };
    },),
  },];
  /**
   Record a (node, merged values) pair, reporting whether it is new.

   @param node - Result node.
   
   @param merged - Values merged into it.

   @returns Whether the pair was not seen before.
   */
  function firstVisit({
    node,
    merged,
  }: {
    readonly node: object;
    readonly merged: readonly Sourced[]
  },): boolean {
    /**
     Visited keys of this node.
     */
    const seen = visited.get(node,) ?? new Set<string>();
    visited.set(
      node,
      seen,
    );
    /**
     Identity key of the merged tuple.
     */
    const key = merged
      .map(function idOf(source,) {
        if (!ids.has(source.value,))
          ids.set(
            source.value,
            ids.size,
          );
        return String(ids.get(source.value,),);
      },)
      .join(',',);
    if (seen.has(key,))
      return false;
    seen.add(key,);
    return true;
  }
  /**
   Queue a child pair.

   @param next - Pair to check later.
   */
  function enqueue(next: Pending,): void {
    pending.push(next,);
  }
  for (let item = pending.pop(); item !== undefined; item = pending.pop()) {
    /**
     Mismatch at this pair, if any.
     */
    const mismatch = checkPair({
      cycles,
      enqueue,
      firstVisit,
      flags,
      item,
    },);
    if (mismatch !== '')
      return {
        ...flags,
        mismatch,
      };
  }
  return {
    ...flags,
    mismatch: '',
  };
}
