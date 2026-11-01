/**
 Per-node edge snapshots for detecting which input containers a merge wrote
 into, by identity rather than by shape.

 `./alias-graph.property.unit.test.ts` uses them to tell apart the known
 defect (a merge writing into source nodes the target shares after storing
 one by reference, pinned in `./known-defect-alias.unit.test.ts`) from any
 other source mutation.

 @module
 */

import {
  childrenOf,
  isContainer,
} from './alias-walk.ts';

/**
 Children of each container as captured before a merge.
 */
export type EdgeSnapshot = ReadonlyMap<object, readonly (readonly [
  unknown,
  unknown,
])[]>;

/**
 Every container reachable from `roots`, walked with an explicit work stack
 so deep or cyclic graphs never recurse.

 @param roots - Graph entry points; leaves are ignored.

 @returns Reachable containers, each once.

 @example
 ```ts
 const nodes = reachableContainers([looped,]);
 ```
 */
export function reachableContainers(roots: readonly unknown[],): ReadonlySet<object> {
  /**
   Containers already visited.
   */
  const seen = new Set<object>();
  /**
   Values still to visit.
   */
  const stack: unknown[] = [...roots,];
  while (stack.length > 0) {
    /**
     Next value to visit.
     */
    const value = stack.pop();
    if (isContainer(value,) && (!seen.has(value,))) {
      seen.add(value,);
      stack.push(...childrenOf(value,)
        .map(function childOf([, child,],) {
          return child;
        },),);
    }
  }
  return seen;
}

/**
 Capture each container's children before a merge.

 @param nodes - Containers to watch.

 @returns Children per container, compared later by identity.

 @example
 ```ts
 const edges = snapshotEdges(reachableContainers(sources,),);
 ```
 */
export function snapshotEdges(nodes: ReadonlySet<object>,): EdgeSnapshot {
  return new Map([...nodes,].map(function capture(node,) {
    return [
      node,
      childrenOf(node,),
    ] as const;
  },),);
}

/**
 Watched containers reachable from `root` after the merge: the input nodes a
 merge result or into target shares, whether it stores them directly or
 reaches them through a node it stores.

 @param edges - Snapshot of the input containers.

 @param root - Merge result or into target after the merge.

 @returns Input containers that are also part of the result's graph.

 @example
 ```ts
 const shared = sharedWithRoot({ edges, root: intoTarget, });
 ```
 */
export function sharedWithRoot(
  {
    edges,
    root,
  }: {
    readonly edges: EdgeSnapshot;
    readonly root: unknown;
  },
): ReadonlySet<object> {
  return new Set([...reachableContainers([root,],),]
    .filter(function isWatched(node,) {
      return edges.has(node,);
    },),);
}

/**
 Containers whose children changed since `edges` was captured: a key added,
 removed, reordered, or pointing at another value (`Object.is`).

 @param edges - Snapshot taken before the merge.

 @returns Changed containers, in snapshot order.

 @example
 ```ts
 rewiredNodes(edges).length; // 0 when no watched container was written
 ```
 */
export function rewiredNodes(edges: EdgeSnapshot,): readonly object[] {
  return [...edges,]
    .filter(function changed([node, before,],) {
      /**
       Children after the merge.
       */
      const after = childrenOf(node,);
      return (after.length !== before.length)
        || after.some(function differs(
          [key, child,],
          index,
        ) {
          /**
           Child at the same position before the merge.
           */
          const previous = before[index];
          return (previous === undefined)
            || (!Object.is(
              key,
              previous[0],
            ))
            || (!Object.is(
              child,
              previous[1],
            ));
        },);
    },)
    .map(function nodeOf([node,],) {
      return node;
    },);
}
