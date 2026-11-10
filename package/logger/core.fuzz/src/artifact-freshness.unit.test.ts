/**
 Guards what the property suite compares against.

 Every property file here imports the built runtime artifact through the
 package name, never the runtime package's source, so the properties cross the
 consumer boundary. That build is not committed: `dist` is ignored, and only a
 build step puts it there. A build older than the runtime source therefore runs
 the reference model against code the source no longer has, and the failure
 reads as a model or logger defect instead of the stale artifact it is: on
 2026-11-10 a build from before the change that keeps a slow sink's records made
 `the real sinks and the model agree on attempts, deliveries, flush deadlines,
 breadcrumbs, and throws` report `no-backend throw agreement` for a sink whose
 probe never answers.

 This file fails first, naming the build task, whenever that build predates any
 runtime source file it was made from.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { readdir, stat, } from 'node:fs/promises';
import { dirname, join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

//region The two paths the properties resolve through

/**
 Built entry point the property files import as
 `@monochromatic-dev/logger-core`.
 */
const artifactEntry = fileURLToPath(import.meta.resolve('@monochromatic-dev/logger-core',),);

/**
 TypeScript entry point behind the package's `/ts` subpath, whose directory
 holds every file the build bundles.
 */
const sourceEntry = fileURLToPath(import.meta.resolve('@monochromatic-dev/logger-core/ts',),);

/**
 Directory the build reads.
 */
const sourceDirectory = dirname(sourceEntry,);

//endregion The two paths the properties resolve through

//region Stale-build detection

/**
 Whether a file is one the build bundles.

 Test files live beside the runtime sources and are never part of the artifact,
 so a freshly added test must not read as a stale build.

 @param name - Base name of a file under the source directory.

 @returns Whether the build reads it.

 @example
 ```ts
 expect(isBundled({ name: 'create-logger.ts' })).toBe(true);
 expect(isBundled({ name: 'create-logger.unit.test.ts' })).toBe(false);
 ```
 */
function isBundled({ name, }: { readonly name: string; },): boolean {
  return name.endsWith('.ts',)
    && (!name.endsWith('.test.ts',));
}

/**
 One source file and the modification time that decides whether the build
 predates it.
 */
type SourceStamp = {
  readonly modifiedAt: number;
  readonly name: string;
};

/**
 Modification time of the newest file the build bundles, with its name so a
 failure names the file to rebuild from.

 @param directory - Directory to walk.

 @returns Newest source stamp.

 @example
 ```ts
 const newest = await newestSource({ directory: sourceDirectory });
 ```
 */
async function newestSource({ directory, }: { readonly directory: string; },): Promise<SourceStamp> {
  /**
   Every file under the runtime source directory, relative to it.
   */
  const relativePaths: string[] = await readdir(
    directory,
    {
      recursive: true,
    },
  );
  /**
   Bundled files among them, as absolute paths.
   */
  const bundled = relativePaths
    .filter(function isSourceFile(relativePath,) {
      return isBundled({ name: relativePath, },);
    },)
    .map(function toAbsolutePath(relativePath,) {
      return join(
        directory,
        relativePath,
      );
    },);
  /**
   Modification times of those files.
   */
  const stamps: SourceStamp[] = await Promise.all(
    bundled.map(async function toStamp(absolutePath,) {
      /**
       Size and times of one source file.
       */
      const details = await stat(absolutePath,);
      return {
        modifiedAt: details.mtimeMs,
        name: absolutePath,
      };
    },),
  );
  /**
   Newest stamp; the walk always finds the source directory's own entry point.
   */
  const newest = stamps.reduce(function later(
    left,
    right,
  ) {
    return (right.modifiedAt > left.modifiedAt) ? right : left;
  },);
  return newest;
}

/**
 Whether the build the properties import is current, or why it is not.
 */
type BuildFreshness =
  | { readonly kind: 'current'; }
    | {
      readonly kind: 'stale';
      /**
       Sentence naming both times and the build task that refreshes the build.
       */
      readonly report: string;
    };

/**
 Reports a build that predates the runtime source it came from, or that it is
 current when it does not.

 @param artifactModifiedAt - Modification time of the artifact entry point.

 @param newest - Newest bundled source file.

 @returns Tagged freshness, carrying the sentence to fail with when stale.

 @example
 ```ts
 buildFreshness({ artifactModifiedAt: 1, newest: { modifiedAt: 2, name: 'src/index.ts' } });
 ```
 */
function buildFreshness(
  {
    artifactModifiedAt,
    newest,
  }: {
    readonly artifactModifiedAt: number;
    readonly newest: SourceStamp;
  },
): BuildFreshness {
  if (artifactModifiedAt >= newest.modifiedAt)
    return { kind: 'current', };
  return {
    kind: 'stale',
    report: `the built logger this suite imports (${artifactEntry}, modified ${
      new Date(artifactModifiedAt,).toISOString()
    }) is older than the runtime source it is built from (${newest.name}, modified ${
      new Date(newest.modifiedAt,).toISOString()
    }), so the properties would compare the reference model with code that build does not contain. Build it before running this suite: mise run //package/logger/core:build`,
  };
}

//endregion Stale-build detection

await describe({
  name: 'property suite subject',
  children: [
    it({
      name: 'the built logger the properties import is not older than the runtime source it came from',
      fn: async () => {
        /**
         Times of the artifact entry point the properties import.
         */
        const artifact = await stat(artifactEntry,);
        /**
         Newest runtime source file the build bundles.
         */
        const newest = await newestSource({ directory: sourceDirectory, },);
        expect(buildFreshness({
          artifactModifiedAt: artifact.mtimeMs,
          newest,
        },),)
          .toEqual({ kind: 'current', },);
      },
    },),
  ],
},);
