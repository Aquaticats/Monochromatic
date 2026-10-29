/**
 GitHub release resolver: maps the recorded artifact rows onto one release's
 assets, taking digests from the asset `digest` field or hashing the download.

 @module
 */

import { createHash, } from 'node:crypto';
import { basename, } from 'node:path';
import type { ToolArtifacts, } from './plan.artifacts.data.ts';
import type { ToolPlanEntry, } from './types.ts';
import {
  githubReleaseFromJson,
  githubRepoFromRows,
  githubTagFromRows,
  type GithubAsset,
  type GithubRelease,
  versionFromTag,
} from './update.resolvers.github-release.ts';
import {
  type Fetcher,
  type MutableArtifacts,
  parseJson,
  stripPrefix,
  type ToolResolution,
} from './update.resolvers.shared.ts';

//region Asset selection

/**
 Marker for a glob scan that already failed to match.
 */
const NO_MATCH = -1;

/**
 Whether an asset name matches a glob in which `*` matches any run of characters
 and every other character matches itself, so `llama-*-ubuntu-rocm-gfx110X-x64.zip`
 selects assets named in that shape (`options.assetPattern` in the plan).

 @param pattern - glob to match against

 @param name - asset name

 @returns whether the whole name matches

 @example
 ```ts
 globMatches({ pattern: 'demo_*', name: 'demo_notes.txt', },); // true
 ```
 */
export function globMatches(
  {
    pattern,
    name,
  }: {
    readonly pattern: string;
    readonly name: string;
  },
): boolean {
  /**
   Literal runs between the stars.
   */
  const parts = pattern.split('*',);
  if (parts.length === 1)
    return name === pattern;
  /**
   Literal run that must open the name.
   */
  const first = parts.at(0,) ?? '';
  /**
   Literal run that must close the name.
   */
  const last = parts.at(-1,) ?? '';
  if ((name.length < (first.length + last.length))
    || (!name.startsWith(first,))
    || (!name.endsWith(last,)))
    return false;
  /**
   End of the leftmost in-order placement of the inner runs, or the miss marker.
   */
  const reachedAt = parts.slice(
    1,
    -1,
  )
    .reduce(
    function advance(
      cursor: number,
      part: string,
    ): number {
      if (cursor === NO_MATCH)
        return NO_MATCH;
      /**
       Where this inner run next occurs.
       */
      const at = name.indexOf(
        part,
        cursor,
      );
      return at === (-1)
        ? NO_MATCH
        : at + part.length;
    },
    first.length,
    );
  return (reachedAt !== NO_MATCH) && (reachedAt <= (name.length - last.length));
}

/**
 Picks the release asset one recorded row maps onto: the version-substituted
 name, then the recorded name, then (with a pattern) the single pattern match.

 @param release - release whose assets to search

 @param byName - assets indexed by file name

 @param oldName - recorded asset file name

 @param oldVersion - version recorded in the old name

 @param newVersion - version to put in the new name

 @param platform - platform id for error messages

 @param assetPattern - glob selecting the asset when the names do not line up

 @param patternMatches - assets whose names the glob selects, computed once for all rows

 @returns matching asset

 @throws Error when no asset matches, or when `assetPattern` selects zero or several assets
 */
function pickAsset(
  {
    release,
    byName,
    oldName,
    oldVersion,
    newVersion,
    platform,
    assetPattern,
    patternMatches,
  }: {
    readonly release: GithubRelease;
    readonly byName: ReadonlyMap<string, GithubAsset>;
    readonly oldName: string;
    readonly oldVersion: string;
    readonly newVersion: string;
    readonly platform: string;
    readonly assetPattern?: string;
    readonly patternMatches: readonly GithubAsset[];
  },
): GithubAsset {
  for (const candidate of [
    oldName.replaceAll(
      oldVersion,
      newVersion,
    ),
    oldName,
  ]) {
    /**
     Release asset with this candidate name.
     */
    const named = byName.get(candidate,);
    if (named !== undefined)
      return named;
  }
  if (assetPattern !== undefined) {
    /**
     The single selection, absent while the pattern selects zero or several assets.
     */
    const [selected,] = patternMatches;
    if ((patternMatches.length === 1) && (selected !== undefined))
      return selected;
    throw new Error(`asset pattern ${
      assetPattern
    } matches ${
      String(patternMatches.length,)
    } release assets for ${
      platform
    } in ${
      release.tag_name ?? 'the release'
    }; expected exactly one`,);
  }
  throw new Error(`no release asset matches ${
    oldName
  } in ${
    release.tag_name ?? 'the release'
  }`,);
}

/**
 SHA-256 of a release asset: the asset digest field when the release records
 one, else the downloaded bytes (releases predating asset digests).

 @param name - asset file name for error messages

 @param digest - digest field of the asset, absent for old releases

 @param downloadUrl - asset download URL

 @param fetcher - download source, absent when hashing is not possible

 @returns lowercase hex SHA-256

 @throws Error when the asset has no digest and no download can hash it
 */
async function assetDigest(
  {
    name,
    digest,
    downloadUrl,
    fetcher,
  }: {
    readonly name: string;
    readonly digest?: string;
    readonly downloadUrl: string;
    readonly fetcher?: Fetcher;
  },
): Promise<string> {
  if (digest !== undefined) {
    return stripPrefix({
      text: digest,
      prefix: 'sha256:',
    },);
  }
  if (fetcher === undefined) {
    throw new Error(`release asset ${
      name
    } carries no digest and no fetcher is available to hash it`,);
  }
  /**
   Downloaded asset bytes to hash.
   */
  const downloaded = await fetcher(downloadUrl,);
  if ((downloaded.bytes === undefined) || (!downloaded.ok)) {
    throw new Error(`cannot download release asset ${
      name
    } to hash it`,);
  }
  return createHash('sha256',)
    .update(downloaded.bytes,)
    .digest('hex',);
}

//endregion Asset selection

//region Mapping

/**
 Maps the recorded artifact rows onto one GitHub release's assets.

 @param rows - recorded artifact rows

 @param release - release payload whose assets to map onto

 @param oldVersion - version recorded in the asset names

 @param newVersion - version to put in the asset names

 @param fetcher - download source for assets that record no digest

 @param assetPattern - glob selecting the asset when the names do not line up

 @returns rows with the new URLs and SHA-256 digests

 @throws Error when no asset matches a row, or when `assetPattern` selects
 zero or several assets for a row

 @example
 ```ts
 await mapGithubRows({ rows, release, oldVersion: '1.0.0', newVersion: '1.1.0', },);
 ```
 */
export async function mapGithubRows(
  {
    rows,
    release,
    oldVersion,
    newVersion,
    fetcher,
    assetPattern,
  }: {
    readonly rows: ToolArtifacts;
    readonly release: GithubRelease;
    readonly oldVersion: string;
    readonly newVersion: string;
    readonly fetcher?: Fetcher;
    readonly assetPattern?: string;
  },
): Promise<ToolArtifacts> {
  /**
   Assets indexed by file name.
   */
  const byName = new Map<string, GithubAsset>();
  for (const asset of release.assets ?? []) {
    if (asset.name !== undefined)
      byName.set(
        asset.name,
        asset,
      );
  }
  /**
   Assets whose names the pattern selects, matched once so each row only reads the result.
   */
  const patternMatches = (release.assets ?? []).filter(
    function matchesPattern(candidate,): boolean {
      return (assetPattern !== undefined) && globMatches({
        pattern: assetPattern,
        name: candidate.name ?? '',
      },);
    },
  );
  /**
   Mapped rows keyed by platform id.
   */
  const mapped: MutableArtifacts = {};
  for (const [platform, row,] of Object.entries(rows,)) {
    /**
     Recorded asset name.
     */
    const oldName = basename(row.url,);
    /**
     Matching release asset.
     */
    const asset = pickAsset({
      release,
      byName,
      oldName,
      oldVersion,
      newVersion,
      platform,
      patternMatches,
      ...(assetPattern === undefined
        ? {}
        : { assetPattern, }),
    },);
    if ((asset.name === undefined) || (asset.browser_download_url === undefined)) {
      throw new Error(`release asset ${
        asset.name ?? oldName
      } carries no download URL`,);
    }
    mapped[platform] = {
      url: asset.browser_download_url,
      // oxlint-disable-next-line no-await-in-loop -- assets without a digest are large downloads; hashing them one at a time bounds memory and request rate
      sha256: await assetDigest({
        name: asset.name,
        downloadUrl: asset.browser_download_url,
        ...(asset.digest === undefined
          ? {}
          : { digest: asset.digest, }),
        ...(fetcher === undefined
          ? {}
          : { fetcher, }),
      },),
    };
  }
  return mapped;
}

/**
 Resolves a GitHub-hosted tool from the repository's latest release.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns new version and rows

 @throws Error when the release cannot be mapped onto the recorded rows

 @example
 ```ts
 await resolveGithubFamily({ entry, fetcher, },);
 ```
 */
export async function resolveGithubFamily(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Glob selecting the release asset when the names do not line up, if the plan sets one.
   */
  const { assetPattern, } = entry.options;
  /**
   Repository slug of the recorded release.
   */
  const repo = githubRepoFromRows({ rows: entry.artifacts, },);
  /**
   Recorded release tag.
   */
  const oldTag = githubTagFromRows({ rows: entry.artifacts, },);
  /**
   Latest release response, with the release list as the fallback when the
   repository publishes no `latest` pointer.
   */
  const latestResponse = await fetcher(
    `https://api.github.com/repos/${
      repo
    }/releases/latest`,
  );
  /**
   Release payload to map against.
   */
  const record = githubReleaseFromJson({
    doc: parseJson({
      response: latestResponse.ok
        ? latestResponse
        : await fetcher(
          `https://api.github.com/repos/${
            repo
          }/releases?per_page=1`,
        ),
      label: `github.com/${repo}`,
    },),
  },);
  /**
   Tag of the release to move to.
   */
  const newTag = record.tag_name;
  if (newTag === undefined)
    throw new Error(`github.com/${repo} reports no release tag`,);
  /**
   New bare version carried by the new tag.
   */
  const newVersion = versionFromTag({
    oldTag,
    oldVersion: entry.version,
    newTag,
  },);
  if (newVersion === entry.version) {
    // Nothing moved: keep the recorded rows instead of re-mapping them.
    return {
      version: entry.version,
      artifacts: entry.artifacts,
    };
  }
  return {
    version: newVersion,
    artifacts: await mapGithubRows({
      rows: entry.artifacts,
      release: record,
      oldVersion: entry.version,
      newVersion,
      fetcher,
      ...(assetPattern === undefined
        ? {}
        : { assetPattern, }),
    },),
  };
}

//endregion Mapping
