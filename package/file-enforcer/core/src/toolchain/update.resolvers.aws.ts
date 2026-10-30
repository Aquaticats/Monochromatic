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
const TRACKED_MAJOR = 2;

/**
 Reads the major, minor, and patch numbers of a stable v2 release tag.

 @param tag - tag name from the tag listing (`2.39.2`, no `v` prefix)

 @returns the three numbers, or `undefined` for any other tag shape or major version

 @example
 ```ts
 releaseNumbers({ tag: '2.39.2', },); // [2, 39, 2]
 releaseNumbers({ tag: '1.44.0', },); // undefined
 ```
 */
export function releaseNumbers(
  { tag, }: { readonly tag: string; },
): readonly number[] | undefined {
  /**
   Dot-separated parts of the tag.
   */
  const parts = tag.split('.',);
  if ((parts.length !== TAG_NUMBERS)
    || (!parts.every(function isNumber(part,): boolean {
      return isDigits({ text: part, },);
    },)))
    return undefined;
  /**
   Parts as numbers.
   */
  const numbers = parts.map(Number,);
  return numbers[0] === TRACKED_MAJOR
    ? numbers
    : undefined;
}

/**
 Whether one release number triple is newer than another.

 @param candidate - triple to test
 @param current - triple to beat

 @returns whether `candidate` sorts after `current` by major, then minor, then patch

 @example
 ```ts
 isNewerRelease({ candidate: [2, 39, 2,], current: [2, 38, 6,], },); // true
 ```
 */
export function isNewerRelease(
  {
    candidate,
    current,
  }: {
    readonly candidate: readonly number[];
    readonly current: readonly number[];
  },
): boolean {
  /**
   First position where the two triples differ.
   */
  const at = candidate.findIndex(function differs(
    part,
    index,
  ): boolean {
    return part !== current[index];
  },);
  return (at !== (-1)) && ((candidate[at] ?? 0) > (current[at] ?? 0));
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
   Newest stable v2 release tag, empty when the listing holds none.
   */
  const newVersion = tags.reduce(
    function newest(
      best: string,
      tag: string,
    ): string {
      /**
       Numbers of the tag under test.
       */
      const candidate = releaseNumbers({ tag, },);
      /**
       Numbers of the best tag so far.
       */
      const current = releaseNumbers({ tag: best, },);
      if (candidate === undefined)
        return best;
      return (current === undefined) || isNewerRelease({
          candidate,
          current,
        },)
        ? tag
        : best;
    },
    '',
  );
  if (newVersion === '')
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
  const urls = Object.entries(entry.artifacts,)
    .map(function renewed([platform, row,],): readonly [string, string,] {
      if (!row.url
        .includes(entry.version,))
        throw new Error(`${row.url} does not carry the recorded version ${entry.version}`,);
      return [
        platform,
        row.url
          .replaceAll(
          entry.version,
          newVersion,
          ),
      ];
    },);
  /**
   SHA-256 of each distinct new archive, keyed by URL.
   */
  const digests = new Map(await Promise.all([
    ...new Set(urls.map(function urlOf([, url,],): string {
      return url;
    },),),
  ].map(async function hashed(url,): Promise<readonly [string, string,]> {
    return [
      url,
      await hashDownload({
        url,
        fetcher,
      },),
    ];
  },),),);
  /**
   Mapped rows keyed by platform id.
   */
  const mapped: MutableArtifacts = {};
  for (const [platform, url,] of urls) {
    /**
     Digest of this row's archive.
     */
    const sha256 = digests.get(url,);
    if (sha256 === undefined)
      throw new Error(`no digest was computed for ${url}`,);
    mapped[platform] = {
      url,
      sha256,
    };
  }
  return {
    version: newVersion,
    artifacts: mapped,
  };
}
