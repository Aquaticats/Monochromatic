/**
 Position bookkeeping for the graph oracle in `./alias-bisim.ts`: which input
 contributed each value, which ancestors a position has, how upstream's cycle
 handling resolves a position, and whether the known false-cycle defect is
 involved.

 Upstream's cycle rule, read from `mergeUnknowns` and the default
 `mergeCircularReferences` in `src/defaults/vanilla.ts`: once any same-kind
 container value at a position is cyclic (identical to a value on its own
 path above), the position is not merged. It resolves to the ancestor result
 at the shared cyclic depth, or at the last value's depth, or to the last
 value with its references resolved when the last value is not cyclic.

 @module
 */

import { kindOf, } from './model.ts';

/**
 One input's value at a merge position, tagged with its input index.
 */
export type Sourced = {
  readonly input: number;
  readonly value: unknown;
};

/**
 An ancestor position: the values merged there and the result node built.
 */
export type Frame = {
  readonly sources: readonly Sourced[];
  readonly result: unknown;
};

/**
 How upstream's cycle handling resolves one position.
 */
export type CycleRule =
  | { readonly kind: 'merge'; }
    | {
      readonly kind: 'ancestor';
      readonly result: unknown
    }
    | {
      readonly kind: 'resolve-last';
      readonly last: Sourced
    };

/**
 Whether a record-style key can index an object.

 @param key - Record or Map key.

 @returns Whether `key` is a string, number, or symbol.

 @example
 ```ts
 isPropertyKey('a'); // true
 ```
 */
function isPropertyKey(key: unknown,): key is PropertyKey {
  return [
    'string',
    'number',
    'symbol',
  ].includes(typeof key,);
}

/**
 Values of one key (or Map key) across the sources that have it.

 @param sources - Container values at the parent position.
 
 @param key - Record or Map key.

 @returns Child sources, input indices preserved.

 @example
 ```ts
 childSources({ sources, key: 'a', });
 ```
 */
export function childSources({
  sources,
  key,
}: {
  readonly sources: readonly Sourced[];
  readonly key: unknown
},): readonly Sourced[] {
  return sources.flatMap(function childOf(source,): readonly Sourced[] {
    /**
     Parent container.
     */
    const container = source.value;
    if (container instanceof Map)
      return container.has(key,) ? [{
        input: source.input,
        value: container.get(key,) as unknown,
      },] : [];
    if (((typeof container) !== 'object') || (container === null)
      || (!isPropertyKey(key,)))
      return [];
    return Object.prototype
      .propertyIsEnumerable
      .call(
      container,
      key,
      )
      ? [{
        input: source.input,
        value: Reflect.get(
          container,
          key,
        ) as unknown,
      },]
      : [];
  },);
}

/**
 Whether a value from one input equals an ancestor that is not on its own
 path: the trigger of the known false-cycle defect.

 @param source - Container value at the current position and its input.
 
 @param ancestors - Frames from the root down.

 @returns Whether some ancestor holds the value only for other inputs.

 @example
 ```ts
 triggersFalseCycle({ source, ancestors, });
 ```
 */
export function triggersFalseCycle({
  source,
  ancestors,
}: {
  readonly source: Sourced;
  readonly ancestors: readonly Frame[]
},): boolean {
  return ancestors.some(function holdsForeign(frame,) {
    /**
     Inputs whose value at this ancestor is the checked value.
     */
    const holders = frame.sources
      .filter(function holds(candidate,) {
        return candidate.value === source.value;
      },);
    return (holders.length > 0) && (!holders.some(function ownPath(candidate,) {
      return candidate.input === source.input;
    },));
  },);
}

/**
 Distance up to the nearest ancestor where the value is the same input's
 value, as upstream's `getCyclicReferenceDepth` counts it on the value's own
 path; `0` when there is none.

 @param source - Value at this position and its input.
 
 @param ancestors - Frames from the root down.

 @returns Cyclic depth, `0` for a non-cyclic value.

 @example
 ```ts
 ownCycleDepth({ source, ancestors, });
 ```
 */
export function ownCycleDepth({
  source,
  ancestors,
}: {
  readonly source: Sourced;
  readonly ancestors: readonly Frame[]
},): number {
  /**
   Index of the deepest frame holding the value for the same input.
   */
  const index = ancestors.findLastIndex(function ownAncestor(frame,) {
    return frame.sources
      .some(function sameInput(candidate,) {
        return (candidate.input === source.input) && (candidate.value === source.value);
      },);
  },);
  return index === (-1) ? 0 : ancestors.length - index;
}

/**
 Apply upstream's cycle rule to one position.

 @param present - Present sources at this position.
 
 @param ancestors - Frames from the root down.

 @returns Rule outcome; `merge` when no cycle is involved.

 @example
 ```ts
 cycleRule({ present, ancestors, });
 ```
 */
export function cycleRule({
  present,
  ancestors,
}: {
  readonly present: readonly Sourced[];
  readonly ancestors: readonly Frame[]
},): CycleRule {
  /**
   Cyclic depth of every present value.
   */
  const depths = present.map(function depthOf(source,) {
    return ownCycleDepth({
      ancestors,
      source,
    },);
  },);
  /**
   Last present source and its depth.
   */
  const last = present.at(-1,);
  /**
   Depth of the last present value.
   */
  const lastDepth = depths.at(-1,) ?? 0;
  /**
   Outcome resolving to the ancestor at a cyclic depth.

   @param depth - Positive cyclic depth.

   @returns Ancestor outcome.
   */
  function toAncestor(depth: number,): CycleRule {
    return {
      kind: 'ancestor',
      result: ancestors[ancestors.length - depth]
        ?.result,
    };
  }
  if ((last === undefined) || depths.every(function acyclic(depth,) {
    return depth === 0;
  },))
    return { kind: 'merge', };
  if (present.length === 1)
    return toAncestor(lastDepth,);
  if (!present.every(function sameKind(source,) {
    return (kindOf(source.value,) === kindOf(last.value,)) && (kindOf(source.value,) !== 'other');
  },))
    return { kind: 'merge', };
  if (depths.every(function sameDepth(depth,) {
    return depth === depths[0];
  },))
    return toAncestor(depths[0] ?? 0,);
  return lastDepth === 0
    ? {
      kind: 'resolve-last',
      last,
    }
    : toAncestor(lastDepth,);
}
