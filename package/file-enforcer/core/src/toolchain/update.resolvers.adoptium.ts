/**
 Adoptium resolver for `java`.

 @module
 */

import type { ToolPlanEntry, } from './types.ts';
import {
  githubReleaseFromJson,
  githubTagFromRows,
} from './update.resolvers.github-release.ts';
import { mapGithubRows, } from './update.resolvers.github.ts';
import {
  type Fetcher,
  parseJson,
  stringField,
  stripPrefix,
  type ToolResolution,
} from './update.resolvers.shared.ts';

/**
 Resolves `java` from the Adoptium API (the `temurin-21` channel) and maps
 the rows onto the Adoptium GitHub release assets, which also carry the
 musl builds the API's `os=alpine` filter no longer lists.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns new `temurin-<core>.0.LTS` version and rows

 @throws Error when the release cannot be mapped onto the recorded rows

 @example
 ```ts
 await resolveJava({ entry, fetcher, },); // { version: 'temurin-21.0.13+9.0.LTS', artifacts }
 ```
 */
export async function resolveJava(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Adoptium records for the newest 21 hotspot jdk on linux.
   */
  const records = parseJson({
    response: await fetcher('https://api.adoptium.net/v3/assets/latest/21/hotspot?architecture=x64&image_type=jdk&os=linux',),
    label: 'api.adoptium.net',
  },);
  /**
   Newest record.
   */
  const newest: unknown = Array.isArray(records,)
    ? records[0]
    : undefined;
  /**
   Release name of the newest record (`jdk-21.0.13+9`).
   */
  const releaseName = stringField({
    doc: newest,
    path: ['release_name',],
  },);
  if (releaseName === '') {
    throw new Error('api.adoptium.net reports no jdk release for temurin-21',);
  }
  /**
   Release core of the new resolution (`21.0.13+9`).
   */
  const newCore = stripPrefix({
    text: releaseName,
    prefix: 'jdk-',
  },);
  /**
   Release core of the recorded resolution.
   */
  const oldCore = stripPrefix({
    text: githubTagFromRows({ rows: entry.artifacts, },),
    prefix: 'jdk-',
  },);
  /**
   Release payload carrying the per-platform assets and their digests.
   */
  const release = githubReleaseFromJson({
    doc: parseJson({
      response: await fetcher(`https://api.github.com/repos/adoptium/temurin21-binaries/releases/tags/jdk-${
        encodeURIComponent(newCore,)
      }`,),
      label: 'api.adoptium.net release assets',
    },),
  },);
  return {
    version: `temurin-${
      newCore
    }.0.LTS`,
    artifacts: await mapGithubRows({
      rows: entry.artifacts,
      release,
      // The asset names spell the build with an underscore (21.0.13_9).
      oldVersion: oldCore.replaceAll(
        '+',
        '_',
      ),
      newVersion: newCore.replaceAll(
        '+',
        '_',
      ),
      fetcher,
    },),
  };
}
