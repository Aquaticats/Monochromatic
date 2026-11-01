/**
 Presence of a source in the native TypeScript service, across every materialized project.
 
 @module
 */

import type { Snapshot, } from 'typescript/sync';

/**
 Reports whether the native service already holds a source under any materialized project.
 
 The question `openSemanticFile` has to answer before it announces a source is whether the
 service already holds that path, which is not the same question as whether the source is
 reusable from the project this bridge selected for it. The two diverge whenever the project
 cache answers absent while an outer project's program has already read the file, and a source
 announced as created is a source the service never rereads, so its overlay is ignored and its
 types come from whatever text was read first.
 
 `getProjects` reads the snapshot's own project map with no request to the native child, and
 `getSourceFileMetadata` asks per file without decoding the source, caching its answer on the
 program. Both take TypeScript's path canonicalization, so a normalized source path is accepted
 whatever the host spells paths like.
 
 @param snapshot - Snapshot the service is currently answering from.
 
 @param fileName - Normalized source path, per `normalizeSemanticFileName`.
 
 @returns whether any project in snapshot already holds source.
 
 @example
 ```ts
 snapshotHoldsSource({ snapshot, fileName });
 ```
 */
export function snapshotHoldsSource({
  snapshot,
  fileName,
}: {
  readonly snapshot: Snapshot;
  readonly fileName: string;
},): boolean {
  return snapshot
    .getProjects()
    .some(function projectHoldsSource(project,): boolean {
      /**
       Program-stored metadata for source, absent when project does not hold it.
       */
      const metadata = project
        .program
        .getSourceFileMetadata(fileName,);
      return metadata !== undefined;
    },);
}
