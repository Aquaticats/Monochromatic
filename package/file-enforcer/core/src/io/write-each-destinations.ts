import type { GlobResults, } from './cat.ts';
import { mirrorGlobPath, } from './glob.ts';
import type { StalenessDestination, } from './staleness.ts';
import type { WriteIfChanged, } from './write-lazy.ts';

/**
 Destination mapped from a glob result.
 */
export type GlobDestination = StalenessDestination & {
  /**
   Source file path used in write logs.
   */
  readonly sourcePath: string;
};

/**
 Maps glob results to concrete destination paths using {@link mirrorGlobPath}.
 
 @param destGlob - Destination glob pattern.
 
 @param files - Source glob results.
 
 @returns Concrete destinations with source paths for logging.
 
 @example
 ```ts
 const destinations = destinationsForFiles({ destGlob, files });
 ```
 */
export function destinationsForFiles(
  {
    destGlob,
    files,
  }: {
    readonly destGlob: string;
    readonly files: GlobResults;
  },
): readonly GlobDestination[] {
  return files.map(function destinationForFile(file,): GlobDestination {
    return {
      path: mirrorGlobPath({
        sourcePattern: files.sourceGlob,
        destPattern: destGlob,
        sourcePath: file.path,
      },),
      content: file.content,
      sourcePath: file.path,
    };
  },);
}

/**
 Asserts glob mirroring produced one writer per destination path.
 
 @param destinations - Concrete destinations to inspect before writing.
 
 @throws When multiple sources map to the same destination.
 
 @example
 ```ts
 assertUniqueDestinations([{ path: './out/a.txt', content: 'a', sourcePath: './src/a.txt' }]);
 ```
 */
function assertUniqueDestinations(destinations: readonly GlobDestination[],): void {
  /**
   First source path seen for each destination path.
   */
  const sourcePathByDestination = new Map<string, string>();
  destinations.forEach(function assertUniqueDestination(destination,): void {
    /**
     Source path that already claimed this destination, when present.
     */
    const existingSourcePath = sourcePathByDestination.get(destination.path,);
    if (existingSourcePath !== undefined) {
      throw new Error(
        `Duplicate overwriteEach destination "${destination.path}" from "${existingSourcePath}" and "${destination.sourcePath}"`,
      );
    }
    sourcePathByDestination.set(
      destination.path,
      destination.sourcePath,
    );
  },);
}

/**
 Reconciles concrete glob destinations after rejecting duplicates with
 {@link assertUniqueDestinations}.
 
 @param destinations - Concrete destinations with content.
 
 @param writeIfChanged - {@link WriteIfChanged} reconciliation function.
 
 @param manifestPath - Resolved staleness manifest path.
 
 @param recordStaleness - Whether each destination write should record eager staleness metadata.
 
 @example
 ```ts
 await writeGlobDestinations({ destinations, writeIfChanged, manifestPath });
 ```
 */
export async function writeGlobDestinations(
  {
    destinations,
    writeIfChanged,
    manifestPath,
    recordStaleness,
  }: {
    readonly destinations: readonly GlobDestination[];
    readonly writeIfChanged: WriteIfChanged;
    readonly manifestPath: string;
    readonly recordStaleness?: boolean;
  },
): Promise<void> {
  assertUniqueDestinations(destinations,);
  await Promise.all(
    destinations.map(async function writeOneDestination(destination,): Promise<void> {
      await writeIfChanged(recordStaleness === undefined
        ? {
          dest: destination.path,
          content: destination.content,
          manifestPath,
          sourcePath: destination.sourcePath,
        }
        : {
          dest: destination.path,
          content: destination.content,
          manifestPath,
          sourcePath: destination.sourcePath,
          recordStaleness,
        },);
    },),
  );
}
