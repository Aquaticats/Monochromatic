/**
 Shard composition: grouping mutants into container-sized batches.
 
 Shards never span source files, keeping each shard's selected-test set
 tight; oversized per-file mutant lists chunk into consecutive shards.
 Bisection re-shards use the same chunker at half size.
 
 @example
 ```ts
 composeShards({ groups, shardSize: 16, timeoutFloorMs: 5000, timeoutFactor: 3, packagePath });
 ```
 */

import { sanitizeShardTag, } from './shard-tag.ts';
import {
  SHARD_SCHEMA_VERSION,
  type ShardManifest,
} from '../shard-schema.ts';
import type { Mutant, } from '../engine/types.ts';

/**
 One source file's mutants with their selected tests.
 */
export type MutantGroup = {
  readonly file: string;
  readonly mutants: readonly Mutant[];
  readonly tests: readonly string[];
};

/**
 Splits one array into consecutive chunks of at most size.
 
 @param options - Items and maximum chunk size.
 
 @returns Non-empty chunks preserving order.
 
 @example
 ```ts
 chunk({ items: [1, 2, 3], size: 2 });
 // [[1, 2], [3]]
 ```
 */
export function chunk<const T,>(options: {
  readonly items: readonly T[];
  readonly size: number;
},): readonly (readonly T[])[] {
  if (options.size === 0)
    throw new Error(`chunk size must be positive, received ${String(options.size,)}`,);

  return Array.from(
    { length: Math.ceil(options.items
      .length
      / options.size,), },
    function slice(
      _unusedElement,
      index,
    ): readonly T[] {
      return options.items
        .slice(
        index * options.size,
        (index + 1) * options.size,
        );
    },
  );
}

/**
 Composes shard manifests from per-file mutant groups.
 
 @param options - Groups, shard size, timeouts, and package identity.
 
 @returns Manifests ready to mount into containers.
 
 @example
 ```ts
 const manifests = composeShards({ groups, shardSize: 16, timeoutFloorMs: 5000, timeoutFactor: 3, packagePath: 'package/module/fs-path' });
 ```
 */
export function composeShards(options: {
  readonly groups: readonly MutantGroup[];
  readonly shardSize: number;
  readonly timeoutFloorMs: number;
  readonly timeoutFactor: number;
  readonly packagePath: string;
},): readonly ShardManifest[] {
  return options.groups
    .flatMap(function shardGroup(group,): readonly ShardManifest[] {
      return chunk({
        items: group.mutants,
        size: options.shardSize,
      },)
        .map(function toManifest(
          mutants,
          index,
        ): ShardManifest {
          return {
            schemaVersion: SHARD_SCHEMA_VERSION,
            shardId: `${sanitizeShardTag(group.file,)}-${String(index,)}`,
            packagePath: options.packagePath,
            mutants,
            tests: group.tests,
            timeoutFloorMs: options.timeoutFloorMs,
            timeoutFactor: options.timeoutFactor,
          };
        },);
    },);
}

/**
 Rebuilds manifests for a set of mutant ids at a given shard size,
 regrouping by source file; used for taint resharding and confirmation.
 
 @param options - Ids, size, lookup, timeouts, and package identity.
 
 @returns Manifests grouped per source file.
 
 @example
 ```ts
 composeReshard({ ids, size: 1, entryOf, timeoutFloorMs: 5000, timeoutFactor: 3, packagePath });
 ```
 */
export function composeReshard(options: {
  readonly ids: readonly string[];
  readonly size: number;
  readonly entryOf: (id: string,) => {
    readonly mutant: Mutant;
    readonly tests: readonly string[];
  };
  readonly timeoutFloorMs: number;
  readonly timeoutFactor: number;
  readonly packagePath: string;
},): readonly ShardManifest[] {
  /**
   Reshard groups keyed by source file.
   */
  const byFile = new Map<string, {
    readonly mutants: Mutant[];
    readonly tests: readonly string[];
  }>();

  for (const id of options.ids) {
    /**
     Lookup entry for this reshard id.
     */
    const entry = options.entryOf(id,);
    /**
     Existing bucket for this mutant's file.
     */
    const bucket = byFile.get(entry.mutant
      .file,);

    if (bucket === undefined)
      byFile.set(
        entry.mutant
          .file,
        {
          mutants: [entry.mutant,],
          tests: entry.tests,
        },
      );
    else
      bucket.mutants
        .push(entry.mutant,);
  }

  return composeShards({
    groups: [...byFile.entries(),]
      .map(function toGroup(entry,): MutantGroup {
        return {
          file: entry[0],
          mutants: entry[1]
            .mutants,
          tests: entry[1]
            .tests,
        };
      },),
    shardSize: options.size,
    timeoutFloorMs: options.timeoutFloorMs,
    timeoutFactor: options.timeoutFactor,
    packagePath: options.packagePath,
  },);
}
