/**
 Object-graph generator for aliasing and cycle properties.

 The tree generators in `./arbitraries.ts` never share a container, so they
 cannot reach deepmerge-ts's cycle detection with anything but the shallow
 chains in `./merge-invariant.property.unit.test.ts`. Here a merge input is a
 root of an arbitrary small graph: a fixed set of container nodes (records,
 arrays, Sets, Maps) whose entries are leaves or references to any node,
 including itself and nodes reachable from other inputs. That covers shared
 subtrees within and across inputs, self-loops, cycles through every
 container kind, and interlocking cycles between inputs.

 Graphs stay small (a handful of nodes), so every cycle is shallow; deep
 cyclic inputs belong to the embargoed findings and are out of scope here.

 @module
 */

import {
  array,
  boolean,
  constant,
  constantFrom,
  integer,
  oneof,
  record,
  tuple,
  type Arbitrary,
} from 'fast-check';


/**
 Container kind of one graph node.
 */
export type NodeKind = 'array' | 'map' | 'record' | 'set';

/**
 Target of one node entry: a leaf value or another node, by index.
 */
export type EntryTarget =
  | { readonly leaf: unknown; }
    | { readonly node: number; };

/**
 One node entry; `key` is ignored by arrays and Sets.
 */
export type NodeEntry = {
  readonly key: string;
  readonly target: EntryTarget;
};

/**
 Generated graph: its nodes and the node index of each merge input root.
 */
export type GraphSpec = {
  readonly nodes: readonly {
    readonly kind: NodeKind;
    readonly entries: readonly NodeEntry[];
  }[];
  readonly roots: readonly number[];
};

/**
 Switches for the graph generator.
 */
export type GraphOptions = {
  /**
   Whether `undefined` and dates appear as leaves; off for `deepmergeInto`,
   whose first-value typing defect they trigger (`./known-defect.unit.test.ts`).
   */
  readonly exoticLeaves: boolean;
  /**
   Whether every root must be a record, as `deepmergeInto` targets are.
   */
  readonly recordRoots: boolean;
};

/**
 Most nodes in one graph; cycles through them stay shallow.
 */
const MAX_NODES = 5;

/**
 Most entries per node.
 */
const MAX_ENTRIES = 3;

/**
 Most merge inputs drawn from one graph.
 */
const MAX_ROOTS = 3;

/**
 Leaf generator.

 @param exoticLeaves - Whether `undefined` and dates are included.

 @returns Leaf values.

 @example
 ```ts
 const leaves = leafOf(true);
 ```
 */
function leafOf(exoticLeaves: boolean,): Arbitrary<unknown> {
  return oneof(
    constant(null,),
    boolean(),
    integer({
      min: 0,
      max: 3,
    },),
    constantFrom(
      'p',
      'q',
    ),
    ...(exoticLeaves
      ? [
        constant(undefined,),
        integer({
          min: 0,
          max: 2,
        },)
          .map(function toDate(offset,) {
            return new Date(offset,);
          },),
      ]
      : []),
  );
}

/**
 Graph spec generator.

 @param options - Leaf and root switches.

 @returns Generator of graph specs with in-range node references.

 @example
 ```ts
 const graphs = graphSpecArbitrary({ exoticLeaves: true, recordRoots: false, });
 ```
 */
export function graphSpecArbitrary(options: GraphOptions,): Arbitrary<GraphSpec> {
  return integer({
    min: 1,
    max: MAX_NODES,
  },)
    .chain(function withSize(size,) {
    /**
     Entry target: a leaf, or any node of this graph.
     */
      const target: Arbitrary<EntryTarget> = oneof(
        leafOf(options.exoticLeaves,)
          .map(function asLeaf(leaf,) {
            return { leaf, };
          },),
        integer({
          min: 0,
          max: size - 1,
        },)
          .map(function asNode(node,) {
            return { node, };
          },),
      );
    /**
     Node generator; the first node is forced to a record when roots must be.
     */
      const node = record({
        kind: constantFrom<NodeKind>(
          'record',
          'record',
          'array',
          'set',
          'map',
        ),
        entries: array(
          record({
            key: constantFrom(
              'a',
              'b',
              'c',
            ),
            target,
          },),
          { maxLength: MAX_ENTRIES, },
        ),
      },);
      return tuple(
        array(
          node,
          {
            minLength: size,
            maxLength: size,
          },
        ),
        array(
          integer({
            min: 0,
            max: size - 1,
          },),
          {
            minLength: 1,
            maxLength: MAX_ROOTS,
          },
        ),
      )
        .map(function toSpec([nodes, roots,],) {
          return {
            nodes: options.recordRoots
              ? nodes.map(function recordsOnly(entry: {
                readonly kind: NodeKind;
                readonly entries: readonly NodeEntry[]
              },) {
                return {
                  ...entry,
                  kind: 'record' as const,
                };
              },)
              : nodes,
            roots,
          };
        },);
    },);
}

/**
 Build the containers of a spec, then wire their entries, so every
 reference (including cycles) resolves to the one shared container.

 @param spec - Graph to build.

 @returns Merge inputs: the root containers in `spec.roots` order.

 @example
 ```ts
 const inputs = materialize(spec);
 ```
 */
export function materialize(spec: GraphSpec,): readonly unknown[] {
  /**
   Empty container per node.
   */
  const containers: readonly object[] = spec.nodes
    .map(function emptyOf(node,) {
      if (node.kind === 'array')
        return [];
      if (node.kind === 'set')
        return new Set();
      if (node.kind === 'map')
        return new Map();
      return {};
    },);
  for (const [index, node,] of spec.nodes
    .entries()) {
    /**
     Container being filled.
     */
    const container = containers[index];
    for (const entry of node.entries) {
      /**
       Resolved entry value.
       */
      const value = 'node' in entry.target ? containers[entry.target
        .node] : entry.target
          .leaf;
      if (Array.isArray(container,))
        container.push(value,);
      else if (container instanceof Set)
        container.add(value,);
      else if (container instanceof Map)
        container.set(
          entry.key,
          value,
        );
      else if (container !== undefined)
        Reflect.defineProperty(
          container,
          entry.key,
          {
            configurable: true,
            enumerable: true,
            value,
            writable: true,
          },
        );
    }
  }
  return spec.roots
    .map(function rootOf(root,) {
      return containers[root];
    },);
}
