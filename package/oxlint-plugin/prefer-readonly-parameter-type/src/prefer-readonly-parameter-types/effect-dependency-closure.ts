/**
 Transitive dependency-closure snapshots for incremental cache entries.
 
 @module
 */

import type { Project, } from 'typescript/sync';

import {
  directModuleDependencies,
  MODULE_DEPENDENCIES_UNRESOLVED,
} from './effect-module-dependencies.ts';
import type { EffectDependencyClosure, } from './effect-summary-persistent-cache.ts';

/**
 Recorded closure edges for one source, shared by cache layers.
 */
export type EffectClosureEdges = {
  readonly resolved: boolean;
  readonly directDependencies: readonly string[];
};

/**
 Closure resolver over indexed sources with seeded and fresh edges.
 */
export type EffectDependencyClosureResolver = {
  readonly seedEdges: (options: {
    readonly fileName: string;
    readonly edges: EffectClosureEdges;
  }) => void;
  readonly includeDirectDependencies: (options: {
    readonly fileName: string;
    readonly dependencies: readonly string[];
  }) => void;
  readonly closureFor: (fileName: string) => EffectDependencyClosure;
};

/**
 Builds closure resolver for one effect-summary index build.
 
 Edges seed from validated persistent entries and resolve freshly through
 module references otherwise, memoized per resolver. Closure walks stay
 inside indexed sources: declaration and external files are excluded
 because whole-scope surface digests already bind them.
 
 @param project - Configured project resolving fresh module references.
 
 @param indexedFileNames - Non-declaration owned sources in current scope.
 
 @param sourceDigests - Current per-source content digests.
 
 @returns seedable memoized closure resolver.
 
 @throws Error when an indexed closure node has no current source digest.
 
 @example
 ```ts
 const resolver = createDependencyClosureResolver({
   project,
   indexedFileNames,
   sourceDigests,
 });
 ```
 */
export function createDependencyClosureResolver({
  project,
  indexedFileNames,
  sourceDigests,
}: {
  readonly project: Project;
  readonly indexedFileNames: ReadonlySet<string>;
  readonly sourceDigests: ReadonlyMap<string, string>;
},): EffectDependencyClosureResolver {
  /**
   Memoized closure edges by source path.
   */
  const edgesByFile = new Map<string, EffectClosureEdges>();

  /**
   Returns memoized or freshly resolved closure edges for one source.
   
   @param fileName - Indexed source path.
   
   @returns resolved or unresolved closure edges.
   */
  function edgesFor(fileName: string,): EffectClosureEdges {
    /**
     Previously seeded or resolved edges.
     */
    const memoized = edgesByFile.get(fileName,);
    if (memoized !== undefined)
      return memoized;
    /**
     Program source for fresh module-reference resolution.
     */
    const sourceFile = project.program
      .getSourceFile(fileName,);
    /**
     Freshly resolved dependencies, unresolved when source or references fail.
     */
    const resolved = sourceFile === undefined
      ? MODULE_DEPENDENCIES_UNRESOLVED
      : directModuleDependencies({
        project,
        sourceFile,
      },);
    /**
     Normalized closure edges retained for later walks.
     */
    const edges: EffectClosureEdges = ((typeof resolved) === 'symbol')
      ? {
        resolved: false,
        directDependencies: [],
      }
      : {
        resolved: true,
        directDependencies: resolved,
      };
    edgesByFile.set(
      fileName,
      edges,
    );
    return edges;
  }

  /**
   Snapshots whole indexed scope when exact closure cannot be proven.
   
   @returns unresolved whole-scope closure snapshot.
   */
  function wholeScopeClosure(): EffectDependencyClosure {
    /**
     Digest snapshot over every indexed source.
     */
    const dependencyDigests: Record<string, string> = {};
    indexedFileNames.forEach(function snapshotIndexed(fileName,): void {
      /**
       Current content digest for indexed source.
       */
      const digest = sourceDigests.get(fileName,);
      if (digest === undefined)
        throw new Error(`Indexed source ${fileName} has no current content digest.`,);
      dependencyDigests[fileName] = digest;
    },);
    return {
      resolved: false,
      directDependencies: [],
      dependencyDigests,
    };
  }

  return {
    seedEdges({
      fileName,
      edges,
    },): void {
      edgesByFile.set(
        fileName,
        edges,
      );
    },
    includeDirectDependencies({
      fileName,
      dependencies,
    },): void {
      /**
       Module-reference edges resolved for current source.
       */
      const current = edgesFor(fileName,);
      if (!current.resolved)
        return;
      edgesByFile.set(
        fileName,
        {
          resolved: true,
          directDependencies: [...new Set([
            ...current.directDependencies,
            ...dependencies,
          ],),].toSorted(),
        },
      );
    },
    closureFor(fileName: string,): EffectDependencyClosure {
      /**
       Own first edges, unresolved forcing whole-scope fallback.
       */
      const ownEdges = edgesFor(fileName,);
      if (!ownEdges.resolved)
        return wholeScopeClosure();
      /**
       Digest snapshot accumulated over walked closure nodes.
       */
      const dependencyDigests: Record<string, string> = {};
      /**
       Walked closure nodes.
       */
      const visited = new Set<string>([fileName,],);
      /**
       Work stack of indexed nodes whose edges remain to walk.
       */
      const pending = ownEdges.directDependencies
        .filter(function indexedEdge(edge,): boolean {
          return indexedFileNames.has(edge,);
        },);
      while (pending.length > 0) {
        /**
         Next closure node.
         */
        const current = pending.pop();
        if ((current === undefined) || visited.has(current,))
          continue;
        visited.add(current,);
        /**
         Current node's closure edges.
         */
        const edges = edgesFor(current,);
        if (!edges.resolved)
          return wholeScopeClosure();
        /**
         Current content digest bound into snapshot.
         */
        const digest = sourceDigests.get(current,);
        if (digest === undefined)
          throw new Error(`Closure node ${current} has no current content digest.`,);
        dependencyDigests[current] = digest;
        edges.directDependencies
          .forEach(function pushIndexedEdge(edge,): void {
            if (indexedFileNames.has(edge,) && (!visited.has(edge,)))
              pending.push(edge,);
          },);
      }
      return {
        resolved: true,
        directDependencies: ownEdges.directDependencies
          .filter(function indexedEdge(edge,): boolean {
            return indexedFileNames.has(edge,);
          },),
        dependencyDigests,
      };
    },
  };
}
