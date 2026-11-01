/**
 Cycle-safe walks over object graphs: copying, tree detection, and
 isomorphism.

 `./shape.ts` walks trees only; these helpers track visited containers so
 shared subtrees stay shared and cycles terminate, which the aliasing and
 cycle properties (`./alias-graph.property.unit.test.ts`) need.

 @module
 */

import { kindOf, } from './model.ts';

/**
 Whether a value is a merge container (record, array, Set, or Map).

 @param value - Any graph value.

 @returns Whether `value` is an object deepmerge-ts merges into.

 @example
 ```ts
 isContainer([]); // true
 ```
 */
export function isContainer(value: unknown,): value is object {
  return kindOf(value,) !== 'other';
}

/**
 Children of a container in a fixed order.

 @param container - Array, Set, Map, or record.

 @returns Keys (or indices) and child values, in iteration order.

 @example
 ```ts
 childrenOf({ a: 1, }); // [['a', 1]]
 ```
 */
export function childrenOf(container: object,): readonly (readonly [
  unknown,
  unknown,
])[] {
  if (container instanceof Map)
    return [...container.entries(),];
  if ((container instanceof Set) || Array.isArray(container,)) {
    return [...(container as Iterable<unknown>),].map(function indexed(
      element,
      index,
    ) {
      return [
        index,
        element,
      ] as const;
    },);
  }
  return Reflect.ownKeys(container,)
    .map(function keyed(key,) {
      return [
        key,
        Reflect.get(
          container,
          key,
        ),
      ] as const;
    },);
}

/**
 Copy a graph, sharing and cycles included, keeping leaves by identity.

 @param value - Graph node to copy.
 
 @param copies - Originals already copied, so shared nodes stay shared.

 @returns Copy with the same shape and no container in common with `value`.

 @example
 ```ts
 const copy = cloneGraph({ value: looped, copies: new Map(), });
 ```
 */
export function cloneGraph({
  value,
  copies,
}: {
  readonly value: unknown;
  readonly copies: Map<object, object>
},): unknown {
  if (!isContainer(value,))
    return value;
  /**
   Copy made earlier, when this node is shared or cyclic.
   */
  const existing = copies.get(value,);
  if (existing !== undefined)
    return existing;
  if (value instanceof Map) {
    /**
     Map copy, registered before its values for cycles.
     */
    const copy = new Map<unknown, unknown>();
    copies.set(
      value,
      copy,
    );
    for (const [key, entry,] of value.entries())
      copy.set(
        key,
        cloneGraph({
          copies,
          value: entry,
        },),
      );
    return copy;
  }
  if (value instanceof Set) {
    /**
     Set copy, registered before its elements for cycles.
     */
    const copy = new Set<unknown>();
    copies.set(
      value,
      copy,
    );
    for (const element of value.values())
      copy.add(cloneGraph({
        copies,
        value: element,
      },),);
    return copy;
  }
  /**
   Array or record copy with the original prototype, registered first.
   */
  const copy: object = Array.isArray(value,) ? [] : {};
  Reflect.setPrototypeOf(
    copy,
    Reflect.getPrototypeOf(value,),
  );
  copies.set(
    value,
    copy,
  );
  for (const key of Reflect.ownKeys(value,)) {
    /**
     Original descriptor; data values are copied through the graph.
     */
    const descriptor = Reflect.getOwnPropertyDescriptor(
      value,
      key,
    );
    if (descriptor !== undefined) {
      Reflect.defineProperty(
        copy,
        key,
        ('value' in descriptor) ? {
          ...descriptor,
          value: cloneGraph({
            copies,
            value: Reflect.get(
              value,
              key,
            ),
          },),
        } : descriptor,
      );
    }
  }
  return copy;
}

/**
 Whether no container is reachable twice from a value: no sharing, no cycle.

 `deepmergeInto` merges into the target's own containers in place, so a
 container the target reaches at two positions changes at both; properties
 comparing an into target against a merge therefore use tree targets.

 @param value - Graph root.

 @returns Whether every container under `value` is reached exactly once.

 @example
 ```ts
 isTree({ a: {}, }); // true
 ```
 */
export function isTree(value: unknown,): boolean {
  /**
   Containers reached so far.
   */
  const reached = new Set<object>();
  /**
   Containers still to visit.
   */
  const pending: object[] = isContainer(value,) ? [value,] : [];
  for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
    if (reached.has(next,))
      return false;
    reached.add(next,);
    for (const [, child,] of childrenOf(next,)) {
      if (isContainer(child,))
        pending.push(child,);
    }
  }
  return true;
}

/**
 Whether two graphs have the same shape: same container kinds and keys at
 every position, identical leaves, and sharing and cycles in the same places.

 @param left - Graph after an operation.
 
 @param right - Copy taken before it with {@link cloneGraph}.

 @returns Mismatch location, or an empty string when isomorphic.

 @example
 ```ts
 isomorphismMismatch({ left: input, right: copy, });
 ```
 */
export function isomorphismMismatch({
  left,
  right,
}: {
  readonly left: unknown;
  readonly right: unknown
},): string {
  /**
   Pairing of left nodes to right nodes found so far.
   */
  const paired = new Map<object, object>();
  /**
   Pairs still to compare, with their path.
   */
  const pending: (readonly [
    unknown,
    unknown,
    string,
  ])[] = [[
    left,
    right,
    '$',
  ],];
  for (let item = pending.pop(); item !== undefined; item = pending.pop()) {
    /**
     Nodes and path of this pair.
     */
    const [actual, expected, path,] = item;
    if ((!isContainer(actual,)) || (!isContainer(expected,))) {
      if (!Object.is(
        actual,
        expected,
      ))
        return `${path}: ${String(actual,)} instead of ${String(expected,)}`;
      continue;
    }
    if (kindOf(actual,) !== kindOf(expected,))
      return `${path}: kind ${kindOf(actual,)} instead of ${kindOf(expected,)}`;
    /**
     Right node this left node was paired with before.
     */
    const partner = paired.get(actual,);
    if (partner !== undefined) {
      if (partner !== expected)
        return `${path}: sharing differs`;
      continue;
    }
    paired.set(
      actual,
      expected,
    );
    /**
     Children of both sides.
     */
    const leftChildren = childrenOf(actual,);
    /**
     Children of the pre-operation copy.
     */
    const rightChildren = childrenOf(expected,);
    if (leftChildren.length !== rightChildren.length)
      return `${path}: ${String(leftChildren.length,)} children instead of ${String(rightChildren.length,)}`;
    for (const [index, [key, child,],] of leftChildren.entries()) {
      /**
       Matching right child.
       */
      const other = rightChildren[index];
      if ((other === undefined) || (!Object.is(
        key,
        other[0],
      )))
        return `${path}: key ${String(key,)} differs`;
      pending.push([
        child,
        other[1],
        `${path}.${String(key,)}`,
      ],);
    }
  }
  return '';
}
