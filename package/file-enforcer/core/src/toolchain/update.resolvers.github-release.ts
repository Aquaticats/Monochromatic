/**
 GitHub release facts the resolvers read: release download URLs, tag to
 version mapping, and the release JSON shape.

 @module
 */

import type { ToolArtifacts, } from './plan.artifacts.data.ts';
import {
  isJsonObject,
  type JsonObject,
  jsonObjectsOf,
} from './update.resolvers.shared.ts';

//region Release URLs

/**
 Host prefix of every GitHub release download URL.
 */
const GITHUB_PREFIX = 'https://github.com/';

/**
 Segments of a release download URL below the host: owner, name, `releases`,
 `download`, tag, and at least the file name.
 */
const MIN_DOWNLOAD_SEGMENTS = 6;

/**
 Repository slug and release tag read from one release download URL.
 */
type ReleaseDownload = {
  readonly repo?: string;
  readonly tag?: string;
};

/**
 Splits a release download URL (`https://github.com/<owner>/<name>/releases/download/<tag>/<file>`).

 @param url - artifact URL to inspect

 @returns slug and raw tag, or an empty record when the URL is not a release download
 */
function parseReleaseDownload(
  { url, }: { readonly url: string; },
): ReleaseDownload {
  if (!url.startsWith(GITHUB_PREFIX,))
    return {};
  /**
   Path segments below the host.
   */
  const segments = url.slice(GITHUB_PREFIX.length,)
    .split('/',);
  /**
   Owner, repository name, the two literal segments, and the tag.
   */
  const [owner = '', name = '', releases, download, tag = '',] = segments;
  if ((segments.length < MIN_DOWNLOAD_SEGMENTS)
    || (owner === '')
    || (name === '')
    || (releases !== 'releases')
    || (download !== 'download')
    || (tag === ''))
    return {};
  return {
    repo: `${owner}/${name}`,
    tag,
  };
}

/**
 Reads the owner/name slug out of one artifact URL.

 @param rows - artifact rows to inspect

 @returns `owner/name` of the GitHub repository

 @throws Error when no row is a release download URL

 @example
 ```ts
 githubRepoFromRows({ rows: { 'linux-x64': { url: 'https://github.com/a/b/releases/download/v1/x.tgz', sha256: '', }, }, },); // 'a/b'
 ```
 */
export function githubRepoFromRows(
  { rows, }: { readonly rows: ToolArtifacts; },
): string {
  for (const row of Object.values(rows,)) {
    /**
     Slug of this row's release download URL, absent for other URLs.
     */
    const { repo, } = parseReleaseDownload({ url: row.url, },);
    if (repo !== undefined)
      return repo;
  }
  throw new Error('no GitHub release URL found in the artifact rows',);
}

/**
 Reads the release tag out of one artifact URL.

 @param rows - artifact rows to inspect

 @returns URL-decoded release tag

 @throws Error when no row is a release download URL

 @example
 ```ts
 githubTagFromRows({ rows: { 'linux-x64': { url: 'https://github.com/a/b/releases/download/v1/x.tgz', sha256: '', }, }, },); // 'v1'
 ```
 */
export function githubTagFromRows(
  { rows, }: { readonly rows: ToolArtifacts; },
): string {
  for (const row of Object.values(rows,)) {
    /**
     Tag of this row's release download URL, absent for other URLs.
     */
    const { tag, } = parseReleaseDownload({ url: row.url, },);
    if (tag !== undefined)
      return decodeURIComponent(tag,);
  }
  throw new Error('no GitHub release tag found in the artifact rows',);
}

/**
 Derives the new version from the new tag using the prefix the old pair
 shows (`bun-v1.3.14` with version `1.3.14` keeps the `bun-v` prefix).

 @param oldTag - recorded release tag

 @param oldVersion - version recorded beside that tag

 @param newTag - tag of the release to move to

 @returns new version string

 @throws Error when the old version is not in the old tag or the new tag lacks the old prefix

 @example
 ```ts
 versionFromTag({ oldTag: 'bun-v1.3.14', oldVersion: '1.3.14', newTag: 'bun-v1.3.15', },); // '1.3.15'
 ```
 */
export function versionFromTag(
  {
    oldTag,
    oldVersion,
    newTag,
  }: {
    readonly oldTag: string;
    readonly oldVersion: string;
    readonly newTag: string;
  },
): string {
  /**
   Index of the old version inside the old tag.
   */
  const at = oldTag.indexOf(oldVersion,);
  /**
   Tag prefix carried over every release (`''`, `'v'`, `'bun-v'`, ...).
   */
  const prefix = oldTag.slice(
    0,
    at,
  );
  if ((at === (-1)) || (!newTag.startsWith(prefix,))) {
    throw new Error(`cannot derive a version from tag ${
      newTag
    } given ${
      oldTag
    } -> ${
      oldVersion
    }`,);
  }
  return newTag.slice(prefix.length,);
}

//endregion Release URLs

//region Release JSON

/**
 One release asset the resolvers use.
 */
export type GithubAsset = {
  readonly name?: string;
  readonly digest?: string;
  readonly browser_download_url?: string;
};

/**
 GitHub release JSON fields the resolvers use.
 */
export type GithubRelease = {
  readonly tag_name?: string;
  readonly assets?: readonly GithubAsset[];
};

/**
 Keeps the asset fields that are strings (the API reports `digest: null` for old assets).

 @param asset - parsed asset record

 @returns asset with the string fields only
 */
function githubAssetFromJson(asset: JsonObject,): GithubAsset {
  /**
   Raw field values.
   */
  const {
    name,
    digest,
    browser_download_url: downloadUrl,
  } = asset;
  return {
    ...((typeof name) === 'string'
      ? { name, }
      : {}),
    ...((typeof digest) === 'string'
      ? { digest, }
      : {}),
    ...((typeof downloadUrl) === 'string'
      ? { browser_download_url: downloadUrl, }
      : {}),
  };
}

/**
 Narrows a parsed release document (one release, or a release list whose first entry counts).

 @param doc - parsed `releases/latest` or `releases?per_page=1` document

 @returns release record with only the fields that are present and well-typed

 @example
 ```ts
 githubReleaseFromJson({ doc: { tag_name: 'v1.0.0', }, },); // { tag_name: 'v1.0.0' }
 ```
 */
export function githubReleaseFromJson(
  { doc, }: { readonly doc: unknown; },
): GithubRelease {
  /**
   Release record: the first element when the document is a release list.
   */
  const record: unknown = Array.isArray(doc,)
    ? doc[0]
    : doc;
  if (!isJsonObject(record,))
    return {};
  /**
   Raw tag and asset list.
   */
  const {
    tag_name: tag,
    assets,
  } = record;
  return {
    ...((typeof tag) === 'string'
      ? { tag_name: tag, }
      : {}),
    ...(Array.isArray(assets,)
      ? {
        assets: jsonObjectsOf({ value: assets, },)
          .map(githubAssetFromJson,),
      }
      : {}),
  };
}

//endregion Release JSON
