/**
 awscli.amazonaws.com resolver for `aws`, which reads the release tags of the
 `aws/aws-cli` repository and hashes the vendor zips they name.

 The repository publishes no `releases/latest` pointer, and the vendor host
 publishes only GPG signatures, so the version comes from the tag list and the
 digests come from the downloaded bytes.

 @module
 */

import { createHash, } from 'node:crypto';
import type { ToolPlanEntry, } from './types.ts';
import {
  type Fetcher,
  isDigits,
  jsonObjectsOf,
  type MutableArtifacts,
  parseJson,
  stringField,
  type ToolResolution,
} from './update.resolvers.shared.ts';

/**
 Tag listing of the repository whose tags name the published AWS CLI v2 releases.

 The API lists the newest tags first, and one page of that listing reaches far
 past the newest release.
 */
const TAGS_URL = 'https://api.github.com/repos/aws/aws-cli/tags?per_page=100';

/**
 Dot-separated numbers in a release tag: major, minor, patch.
 */
const TAG_NUMBERS = 3;

/**
 Major version the plan tracks; the v1 line is a different install.
 */
const TRACKED_MAJOR = '2';

/**
 Whether a tag names a stable v2 release: three dot-separated numbers, the
 first being the tracked major version.

 @param tag - tag name from the tag listing (`2.39.2`, no `v` prefix)

 @returns whether the tag is `2.<minor>.<patch>` with numeric parts only

 @example
 ```ts
 isStableV2Tag({ tag: '2.39.2', },); // true
 isStableV2Tag({ tag: '1.44.0', },); // false
 isStableV2Tag({ tag: '2.40.0-rc1', },); // false
 ```
 */
function isStableV2Tag(
  { tag, }: { readonly tag: string; },
): boolean {
  /**
   Dot-separated parts of the tag.
   */
  const parts = tag.split('.',);
  return (parts.length === TAG_NUMBERS)
    && (parts[0] === TRACKED_MAJOR)
    && parts.every(function isNumber(part,): boolean {
      return isDigits({ text: part, },);
    },);
}

/**
 Orders two stable release tags by major, then minor, then patch number.

 @param left - first stable tag

 @param right - second stable tag

 @returns negative when `left` is older, positive when newer, zero when equal

 @example
 ```ts
 compareReleases({ left: '2.39.10', right: '2.39.2', },); // 8
 ```
 */
function compareReleases(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): number {
  /**
   Numbers of the first tag, with absent parts read as zero.
   */
  const [leftMajor = 0, leftMinor = 0, leftPatch = 0,] = left.split('.',)
    .map(Number,);
  /**
   Numbers of the second tag, with absent parts read as zero.
   */
  const [rightMajor = 0, rightMinor = 0, rightPatch = 0,] = right.split('.',)
    .map(Number,);
  return (leftMajor - rightMajor)
    || (leftMinor - rightMinor)
    || (leftPatch - rightPatch);
}

/**
 Downloads one archive and hashes it.

 @param url - archive URL

 @param fetcher - injected GET

 @returns lowercase hexadecimal SHA-256 of the archive

 @throws Error when the download fails or the fetcher returns no bytes
 */
async function hashDownload(
  {
    url,
    fetcher,
  }: {
    readonly url: string;
    readonly fetcher: Fetcher;
  },
): Promise<string> {
  /**
   Downloaded archive, hashed because the vendor publishes no SHA-256 for it.
   */
  const response = await fetcher(url,);
  if (!response.ok) {
    throw new Error(`cannot download ${
      url
    } to hash it: HTTP ${
      String(response.status,)
    }`,);
  }
  if (response.bytes === undefined)
    throw new Error(`the fetcher returned no bytes to hash for ${url}`,);
  return createHash('sha256',)
    .update(response.bytes,)
    .digest('hex',);
}

/**
 Resolves `aws` from the `aws/aws-cli` tag list and the vendor zips.

 Downloads and hashes the zips only when the version moves; rows that share a
 URL (the musl rows reuse the glibc archive) are downloaded once.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns new version and rows

 @throws Error when the tag listing holds no stable v2 release, a recorded URL
 does not carry the recorded version, or a zip cannot be downloaded

 @example
 ```ts
 await resolveAws({ entry, fetcher, },); // { version: '2.39.2', artifacts }
 ```
 */
export async function resolveAws(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Tag names in listing order, v1 and pre-release shapes included.
   */
  const tags = jsonObjectsOf({
    value: parseJson({
      response: await fetcher(TAGS_URL,),
      label: 'github.com/aws/aws-cli tags',
    },),
  },)
    .map(function nameOf(tag,): string {
      return stringField({
        doc: tag,
        path: ['name',],
      },);
    },);
  /**
   Newest stable v2 release tag, absent when the listing holds none.
   */
  const newVersion = tags.filter(function isStable(tag,): boolean {
    return isStableV2Tag({ tag, },);
  },)
    .toSorted(function byRelease(
      left,
      right,
    ): number {
      return compareReleases({
        left,
        right,
      },);
    },)
    .at(-1,);
  if (newVersion === undefined)
    throw new Error('github.com/aws/aws-cli lists no stable v2 release',);
  if (newVersion === entry.version) {
    return {
      version: newVersion,
      artifacts: entry.artifacts,
    };
  }
  /**
   New URL of each platform row, keyed by platform id.
   */
  const renewed: Record<string, string> = {};
  for (const [platform, row,] of Object.entries(entry.artifacts,)) {
    if (!row.url
      .includes(entry.version,))
      throw new Error(`${row.url} does not carry the recorded version ${entry.version}`,);
    renewed[platform] = row.url
      .replaceAll(
      entry.version,
      newVersion,
      );
  }
  /**
   Distinct new archive URLs, so rows that share an archive download it once.
   */
  const distinctUrls = new Set(Object.values(renewed,),);
  /**
   SHA-256 of each distinct new archive.
   */
  const hashes = await Promise.all([...distinctUrls,].map(
    async function hashed(url,): Promise<{
      readonly url: string;
      readonly sha256: string;
    }> {
      return {
        url,
        sha256: await hashDownload({
          url,
          fetcher,
        },),
      };
    },
  ),);
  /**
   Mapped rows keyed by platform id.
   */
  const mapped: MutableArtifacts = {};
  for (const [platform, url,] of Object.entries(renewed,)) {
    /**
     Hash computed for this row's archive.
     */
    const found = hashes.find(function isRowArchive(candidate,): boolean {
      return candidate.url === url;
    },);
    if (found === undefined)
      throw new Error(`no digest was computed for ${url}`,);
    mapped[platform] = {
      url,
      sha256: found.sha256,
    };
  }
  return {
    version: newVersion,
    artifacts: mapped,
  };
}
