/**
 Version re-resolution sources for `toolchain:update`, one resolver per
 family (the sources `doc/planning/mpm-toolchain-migration.md` names).

 Every resolver is pure over a {@link Fetcher}, so unit tests replay
 recorded responses and never touch the network. Digest sources per family:
 GitHub release-asset `digest` fields, nodejs.org and unofficial-builds
 `SHASUMS256.txt`, PyPI file digests, the Adoptium package checksum, and the
 anaconda.org file `sha256`. Google's repository publishes only a SHA-1 for
 cmdline-tools, so that one resolver downloads the zip and hashes the bytes
 when the version actually moves.
 */

import { createHash, } from 'node:crypto';
import { basename, } from 'node:path';
import type { ToolArtifacts, } from './plan.artifacts.data.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Fetching

/**
 Minimal HTTP response a resolver needs.
 */
export type FetchedResponse = {
  readonly ok: boolean;
  readonly status: number;
  readonly text: string;
  /**
   Raw bytes for binary payloads (the Android archive hash).
   */
  readonly bytes?: Uint8Array | undefined;
};

/**
 Injectable GET used by every resolver (bound to `GITHUB_TOKEN` at runtime).
 */
export type Fetcher = (
  url: string,
  init?: {
    readonly headers?: Readonly<Record<string, string>>;
  },
) => Promise<FetchedResponse>;

/**
 One tool's re-resolved version and artifact rows.
 */
export type ToolResolution = {
  readonly version: string;
  readonly artifacts: ToolArtifacts;
};

/**
 Parses one fetched JSON document.
 
 @param options - Response to parse and its source label
 
 @returns Parsed document
 
 @throws Error when the body is not JSON
 */
function parseJson(
  {
    response,
    label,
  }: {
    readonly response: FetchedResponse;
    readonly label: string;
  },
): unknown {
  try {
    return JSON.parse(response.text,) as unknown;
  }
  catch (error) {
    throw new Error(
      `${
      label
    } returned unparseable JSON: ${
      (error as Error)
        .message
    }`,
      { cause: error },
    );
  }
}

//endregion Fetching

//region GitHub releases

/**
 Reads the owner/name slug out of one artifact URL.
 
 @param options - Artifact rows to inspect
 
 @returns `owner/name` of the GitHub repository
 */
export function githubRepoFromRows(
  { rows, }: { readonly rows: ToolArtifacts; },
): string {
  for (const row of Object.values(rows,)) {
    /**
     Slug captured from a release download URL.
     */
    const match = /^https:\/\/github\.com\/([^/]+\/[^/]+)\/releases\/download\//u.exec(row.url);
    if ((match !== null) && (match[1] !== undefined))
      return match[1];
  }
  throw new Error('no GitHub release URL found in the artifact rows',);
}

/**
 Reads the release tag out of one artifact URL.
 
 @param options - Artifact rows to inspect
 
 @returns URL-decoded release tag
 */
export function githubTagFromRows(
  { rows, }: { readonly rows: ToolArtifacts; },
): string {
  for (const row of Object.values(rows,)) {
    /**
     Tag captured from a release download URL.
     */
    const match = /^https:\/\/github\.com\/[^/]+\/[^/]+\/releases\/download\/([^/]+)\//u.exec(row.url);
    if ((match !== null) && (match[1] !== undefined))
      return decodeURIComponent(match[1],);
  }
  throw new Error('no GitHub release tag found in the artifact rows',);
}

/**
 Derives the new version from the new tag using the prefix the old pair
 shows (`bun-v1.3.14` with version `1.3.14` keeps the `bun-v` prefix).
 
 @param options - Old tag and version, plus the new tag
 
 @returns New version string, or null when the old version is not in the tag
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
): string | null {
  /**
   Index of the old version inside the old tag.
   */
  const at = oldTag.indexOf(oldVersion,);
  if (at === (-1))
    return null;
  /**
   Tag prefix carried over every release (`''`, `'v'`, `'bun-v'`, ...).
   */
  const prefix = oldTag.slice(
    0,
    at,
  );
  return newTag.startsWith(prefix,)
    ? newTag.slice(prefix.length,)
    : null;
}

/**
 GitHub release JSON fields the resolvers use.
 */
type GithubAsset = NonNullable<GithubRelease['assets']>[number];

export type GithubRelease = {
  readonly tag_name?: string;
  readonly assets?: readonly {
    readonly name?: string;
    readonly digest?: string | null;
    readonly browser_download_url?: string;
  }[];
};

/**
 Translates a release-asset glob into a whole-name regular expression:
 `*` matches any run of characters, every other character matches itself,
 so `llama-*-ubuntu-rocm-gfx110X-x64.zip` selects assets named in that
 shape (`options.assetPattern` in the plan).
 
 @param options - Glob to translate
 
 @returns Regular expression anchored to the whole asset name
 */
function assetPatternRegex(
  { pattern, }: { readonly pattern: string; },
): RegExp {
  return new RegExp(
    `^${
    pattern.split('*',)
      .map(
        function escapeLiteral(part,): string {
          return part.replaceAll(
            /[\\^$.*+?()[\]{}|]/gu,
            String.raw`\$&`,
          );
        },
      )
      .join('.*',)
  }$`,
    'u',
  );
}

/**
 Maps the recorded artifact rows onto one GitHub release's assets.
 
 @param options - Rows, release payload, old/new versions, and asset pattern
 
 @returns Rows with the new URLs and SHA-256 digests
 
 @throws Error when no asset matches a row, or when `assetPattern` selects
 zero or several assets for a row
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
    readonly fetcher?: Fetcher | undefined;
    readonly assetPattern?: string | undefined;
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
   Mapped rows keyed by platform id.
   */
  const mapped: Record<string, {
    url: string;
    sha256: string;
  }> = {};
  for (const [platform, row,] of Object.entries(rows,)) {
    /**
     Recorded asset name and its two candidate names in the new release.
     */
    const oldName = basename(row.url,);
    const candidates = [
      oldName.replaceAll(
        oldVersion,
        newVersion,
      ),
      oldName,
    ];
    /**
     Matching release asset.
     */
    let asset: GithubAsset | undefined;
    for (const candidate of candidates) {
      asset = byName.get(candidate,);
      if (asset !== undefined)
        break;
    }
    if ((asset === undefined) && (assetPattern !== undefined)) {
      /**
       Assets whose names the pattern selects and the single selection
       (undefined while the pattern selects zero or several assets).
       */
      const matches = (release.assets ?? [])
        .filter(
        function matchesPattern(candidate,): boolean {
          return assetPatternRegex({ pattern: assetPattern, },)
            .test(candidate.name ?? '',);
        },
        );
      const [selected] = matches.length === 1
        ? matches
        : [];
      if (selected === undefined) {
        throw new Error(`asset pattern ${
          assetPattern
        } matches ${
          String(matches.length,)
        } release assets for ${
          platform
        } in ${
          release.tag_name ?? 'the release'
        }; expected exactly one`,);
      }
      asset = selected;
    }
    if (asset === undefined) {
      throw new Error(`no release asset matches ${
        oldName
      } in ${
        release.tag_name ?? 'the release'
      }`,);
    }
    if ((asset.name === undefined) || (asset.browser_download_url === undefined)) {
      throw new Error(`release asset ${
        asset.name ?? oldName
      } carries no download URL`,);
    }
    /**
     SHA-256: the asset digest field when the release records one, else the
     downloaded bytes (releases predating asset digests).
     */
    let digest = (asset.digest === null) || (asset.digest === undefined)
      ? undefined
      : asset.digest
        .replace(
        /^sha256:/u,
        '',
        );
    if (digest === undefined) {
      if (fetcher === undefined) {
        throw new Error(`release asset ${
          asset.name
        } carries no digest and no fetcher is available to hash it`,);
      }
      /**
       Downloaded asset bytes to hash.
       */
      const downloaded = await fetcher(asset.browser_download_url,);
      if ((downloaded.bytes === undefined) || (!downloaded.ok)) {
        throw new Error(`cannot download release asset ${
          asset.name
        } to hash it`,);
      }
      digest = createHash('sha256',)
        .update(downloaded.bytes,)
        .digest('hex',);
    }
    mapped[platform] = {
      url: asset.browser_download_url,
      sha256: digest,
    };
  }
  return mapped;
}

/**
 Resolves a GitHub-hosted tool from the repository's latest release.
 
 @param options - Plan entry and fetcher
 
 @returns New version and rows
 
 @throws Error when the release cannot be mapped onto the recorded rows
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
   Repository slug and recorded release tag.
   */
  const repo = githubRepoFromRows({ rows: entry.artifacts, },);
  const oldTag = githubTagFromRows({ rows: entry.artifacts, },);
  /**
   Latest release payload, falling back to the release list when the
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
  const release = (latestResponse.ok
    ? parseJson({
      response: latestResponse,
      label: `github.com/${repo}`,
    },)
    : parseJson({
      response: await fetcher(
        `https://api.github.com/repos/${
          repo
        }/releases?per_page=1`,
      ),
      label: `github.com/${repo}`,
    },)) as (GithubRelease | readonly GithubRelease[]);
  /**
   Single release record.
   */
  const record = (Array.isArray(release,)
    ? release[0]
    : release) ?? {};
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
  if (newVersion === null) {
    throw new Error(`cannot derive a version from tag ${
      newTag
    } given ${
      oldTag
    } -> ${
      entry.version
    }`,);
  }
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
      assetPattern: entry.options
        .assetPattern,
    },),
  };
}

//endregion GitHub releases

//region Adoptium

/**
 Resolves `java` from the Adoptium API (the `temurin-21` channel) and maps
 the rows onto the Adoptium GitHub release assets, which also carry the
 musl builds the API's `os=alpine` filter no longer lists.
 
 @param options - Plan entry and fetcher
 
 @returns New `temurin-<core>.0.LTS` version and rows
 
 @throws Error when the release cannot be mapped onto the recorded rows
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
   Adoptium record for the newest 21 hotspot jdk on linux.
   */
  const record = (parseJson({
    response: await fetcher('https://api.adoptium.net/v3/assets/latest/21/hotspot?architecture=x64&image_type=jdk&os=linux',),
    label: 'api.adoptium.net',
  },) as readonly {
    release_name?: string;
  }[])[0];
  const releaseName = record?.release_name;
  if (releaseName === undefined) {
    throw new Error('api.adoptium.net reports no jdk release for temurin-21',);
  }
  /**
   Release core of the new resolution (`21.0.13+9`) and of the recorded one.
   */
  const newCore = releaseName.replace(
    /^jdk-/u,
    '',
  );
  const oldCore = githubTagFromRows({ rows: entry.artifacts, },)
    .replace(
    /^jdk-/u,
    '',
    );
  /**
   Release payload carrying the per-platform assets and their digests.
   */
  const release = parseJson({
    response: await fetcher(`https://api.github.com/repos/adoptium/temurin21-binaries/releases/tags/jdk-${
      encodeURIComponent(newCore,)
    }`,),
    label: 'api.adoptium.net release assets',
  },) as GithubRelease;
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

//endregion Adoptium

//region nodejs.org

/**
 Parses a `SHASUMS256.txt` body into file name to digest pairs.
 
 @param options - Sums text
 
 @returns Digests keyed by file name
 */
export function parseShasums(
  { text, }: { readonly text: string; },
): ReadonlyMap<string, string> {
  /**
   Digests keyed by file name.
   */
  const sums = new Map<string, string>();
  for (const line of text.split('\n',)) {
    /**
     Digest and file name of one sums row.
     */
    const match = /^([0-9a-f]{64})\s+\*?(.+)$/u.exec(line);
    if ((match === null) || (match[1] === undefined)
      || (match[2] === undefined))
      continue;
    sums.set(
      match[2],
      match[1],
    );
  }
  return sums;
}

/**
 Resolves `node` from nodejs.org's release index and sums files.
 
 @param options - Plan entry and fetcher
 
 @returns New version and rows
 
 @throws Error when the index or a sums row is missing
 */
export async function resolveNode(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Release index listing the newest release first.
   */
  const index = parseJson({
    response: await fetcher('https://nodejs.org/dist/index.json',),
    label: 'nodejs.org index',
  },) as readonly { version?: string; }[];
  const newTag = index[0]
    ?.version;
  if (newTag === undefined)
    throw new Error('nodejs.org reports no release',);
  /**
   New bare version (`26.12.1`).
   */
  const newVersion = newTag.replace(
    /^v/u,
    '',
  );
  /**
   Sums bodies for the official and the unofficial (musl) host.
   */
  const sums = new Map<string, string>();
  for (const host of [
    'https://nodejs.org/dist',
    'https://unofficial-builds.nodejs.org/download/release',
  ]) {
    /**
     Sums response for this host.
     */
    const response = await fetcher(`${
      host
    }/v${
      newVersion
    }/SHASUMS256.txt`,);
    if (!response.ok)
      continue;
    for (const [name, digest,] of parseShasums({ text: response.text, },))
      sums.set(
        name,
        digest,
      );
  }
  /**
   Mapped rows keyed by platform id.
   */
  const mapped: Record<string, {
    url: string;
    sha256: string;
  }> = {};
  for (const [platform, row,] of Object.entries(entry.artifacts,)) {
    /**
     New file name and its sums row.
     */
    const newName = basename(row.url,)
      .replaceAll(
      entry.version,
      newVersion,
      );
    const digest = sums.get(newName,);
    if (digest === undefined)
      throw new Error(`SHASUMS256.txt has no row for ${newName}`,);
    mapped[platform] = {
      url: row.url
        .replaceAll(
        entry.version,
        newVersion,
        ),
      sha256: digest,
    };
  }
  return {
    version: newVersion,
    artifacts: mapped,
  };
}

//endregion nodejs.org

//region Version-only registries

/**
 Reads one JSON field path out of a parsed document.
 
 @param options - Document and field path segments
 
 @returns String value, or undefined when absent
 */
function stringField(
  {
    doc,
    path,
  }: {
    readonly doc: unknown;
    readonly path: readonly string[];
  },
): string | undefined {
  /**
   Current position while walking the path.
   */
  let cursor: unknown = doc;
  for (const key of path) {
    if (((typeof cursor) !== 'object') || (cursor === null))
      return undefined;
    cursor = (cursor as Record<string, unknown>)[key];
  }
  return (typeof cursor) === 'string'
    ? cursor
    : undefined;
}

/**
 Resolves a `cargo:` tool from crates.io's max stable version.
 
 @param options - Plan entry and fetcher
 
 @returns New version with the recorded (empty) rows
 
 @throws Error when the crate is unknown
 */
export async function resolveCargo(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Crate name behind the entry.
   */
  const crate = entry.packageId
    ?? entry.id
      .replace(
      /^cargo:/u,
      '',
      );
  /**
   Crate document from the registry.
   */
  const doc = parseJson({
    response: await fetcher(`https://crates.io/api/v1/crates/${crate}`,),
    label: `crates.io/${crate}`,
  },);
  const version = stringField({
    doc,
    path: [
      'crate',
      'max_stable_version'
    ],
  },);
  if (version === undefined)
    throw new Error(`crates.io reports no version for ${crate}`,);
  return {
    version,
    artifacts: {},
  };
}

/**
 Resolves an `mpm-pnpm` tool from the npm registry's dist-tags: the `latest`
 tag for the floating track (and for pins, whose newer-availability note
 reads it), the named channel's tag for `{channel}` requests (`next`, ...).
 
 @param options - Plan entry and fetcher
 
 @returns Bare registry version with the recorded (empty) rows
 
 @throws Error when the registry reports no matching dist-tag
 */
export async function resolveNpm(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Package name behind the entry (the scoped names keep their slash).
   */
  const name = entry.packageId
    ?? entry.id
      .replace(
      /^npm:/u,
      '',
      );
  /**
   Dist-tag the request tracks (`latest`, `next`, ...).
   */
  const tag = 'channel' in entry.request
    ? entry.request
      .channel
    : 'latest';
  /**
   Packument from the registry.
   */
  const doc = parseJson({
    response: await fetcher(`https://registry.npmjs.org/${name}`,),
    label: `registry.npmjs.org/${name}`,
  },);
  const version = stringField({
    doc,
    path: [
      'dist-tags',
      tag
    ],
  },);
  if (version === undefined) {
    throw new Error(`registry.npmjs.org reports no ${
      tag
    } dist-tag for ${
      name
    }`,);
  }
  return {
    version,
    artifacts: {},
  };
}

/**
 Resolves an `mpm-uvx` tool from PyPI's project metadata.
 
 @param options - Plan entry and fetcher
 
 @returns New version with the recorded rows (empty for tools)
 
 @throws Error when the project is unknown
 */
export async function resolvePypi(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Project name behind the entry.
   */
  const name = entry.packageId
    ?? entry.id
      .replace(
      /^pipx:/u,
      '',
      );
  /**
   Project metadata from PyPI.
   */
  const doc = parseJson({
    response: await fetcher(`https://pypi.org/pypi/${name}/json`,),
    label: `pypi.org/${name}`,
  },);
  const version = stringField({
    doc,
    path: [
      'info',
      'version'
    ],
  },);
  if (version === undefined)
    throw new Error(`pypi.org reports no version for ${name}`,);
  return {
    version,
    artifacts: entry.id === 'mpm'
      ? entry.artifacts
      : {},
  };
}

//endregion Version-only registries

//region Go

/**
 One go.dev download-index release record.
 */
type GoRelease = {
  readonly version: string;
  readonly stable: boolean;
  readonly files: readonly {
    readonly filename: string;
    readonly os: string;
    readonly arch: string;
    readonly kind: string;
    readonly sha256: string;
  }[];
};

/**
 Resolves the `go` runtime from go.dev's download index: the release whose
 `go`-prefixed version equals `entry.version`, else the newest stable
 release, with rows recorded from that release's linux `archive` files.
 
 @param options - Plan entry and fetcher
 
 @returns Bare `go` version (the `go` prefix stripped) and linux archive rows
 
 @throws Error when the index carries neither the recorded version nor a
 stable release, or when the chosen release lists no linux archive
 */
export async function resolveGoRuntime(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Every go.dev release, newest first, and the release to record.
   */
  const releases = parseJson({
    response: await fetcher('https://go.dev/dl/?mode=json&include=all',),
    label: 'go.dev download index',
  },) as readonly GoRelease[];
  const chosen = releases.find(
    function carriesRecordedVersion(release,): boolean {
      return release.version
        .replace(
        /^go/u,
        '',
        )
        === entry.version;
    },
  ) ?? releases.find(
    function isStable(release,): boolean {
      return release.stable;
    },
  );
  if (chosen === undefined) {
    throw new Error(`the go.dev download index carries neither ${
      entry.version
    } nor a stable release`,);
  }
  /**
   Mapped rows keyed by platform id.
   */
  const mapped: Record<string, {
    url: string;
    sha256: string;
  }> = {};
  for (const file of chosen.files) {
    if ((file.kind !== 'archive') || (file.os !== 'linux'))
      continue;
    /**
     Platform id of this linux archive (`linux-x64`, `linux-arm64`).
     */
    const platform = file.arch === 'amd64'
      ? 'linux-x64'
      : file.arch === 'arm64'
        ? 'linux-arm64'
        : null;
    if (platform === null)
      continue;
    mapped[platform] = {
      url: `https://go.dev/dl/${
        file.filename
      }`,
      sha256: file.sha256,
    };
  }
  if (Object.keys(mapped,)
    .length
    === 0) {
    throw new Error(`the go.dev release ${
      chosen.version
    } lists no linux archive`,);
  }
  return {
    version: chosen.version
      .replace(
      /^go/u,
      '',
      ),
    artifacts: mapped,
  };
}

/**
 Resolves a `go-install` tool from the Go module proxy's `@latest` record,
 keeping the build-from-source semantics `go install <module>@v<version>`.
 
 @param options - Plan entry and fetcher
 
 @returns Bare module version (one leading `v` stripped) with no rows
 
 @throws Error when the proxy rejects the query or reports no version
 */
export async function resolveGoModule(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Go module path behind the entry.
   */
  const module = entry.packageId
    ?? entry.id
      .replace(
      /^go:/u,
      '',
      );
  /**
   `@latest` record from the module proxy.
   */
  const response = await fetcher(`https://proxy.golang.org/${
    module
  }/@latest`,);
  if (!response.ok) {
    throw new Error(`proxy.golang.org reports no latest version for ${
      module
    }: HTTP ${
      String(response.status,)
    }`,);
  }
  const doc = parseJson({
    response,
    label: `proxy.golang.org/${module}`,
  },);
  /**
   Version tag the proxy reports (`v0.98.1`).
   */
  const tag = stringField({
    doc,
    path: ['Version'],
  },);
  if (tag === undefined) {
    throw new Error(`proxy.golang.org reports no version for ${
      module
    }`,);
  }
  return {
    version: tag.replace(
      /^v/u,
      '',
    ),
    artifacts: {},
  };
}

//endregion Go

//region Channels

/**
 Resolves the `rust` toolchain from its rustup channel manifest, with two
 behaviors dispatched on the channel name in `entry.request`: `nightly`
 returns the `nightly-<date>` stamp of the top-level `date` line in
 `channel-rust-nightly.toml`, while every other channel (`stable`, ...)
 returns the `[pkg.rust]` section's `version` value in
 `channel-rust-<channel>.toml`. A request carrying no channel name keeps
 the nightly behavior.
 
 @param options - Plan entry and fetcher
 
 @returns Channel version with no rows
 
 @throws Error when the manifest carries no date or `[pkg.rust]` version
 */
export async function resolveRust(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Channel name the entry requests (`nightly`, `stable`, ...).
   */
  const channel = 'channel' in entry.request
    ? entry.request
      .channel
    : 'nightly';
  /**
   Channel manifest body.
   */
  const {text} = (await fetcher(`https://static.rust-lang.org/dist/channel-rust-${
    channel
  }.toml`,));
  if (channel !== 'nightly') {
    /**
     `[pkg.rust]` version field, the first field of the section block that
     precedes the target subsections.
     */
    const match = /^\[pkg\.rust\][ \t]*\r?\nversion[ \t]*=[ \t]*"([^"]+)"/mu.exec(text);
    if ((match === null) || (match[1] === undefined)) {
      throw new Error(`the rustup ${
        channel
      } manifest carries no [pkg.rust] version`,);
    }
    return {
      version: match[1],
      artifacts: {},
    };
  }
  /**
   Manifest date line.
   */
  const match = /^date\s*=\s*"([^"]+)"/mu.exec(text);
  if ((match === null) || (match[1] === undefined))
    throw new Error('the rustup nightly manifest carries no date',);
  return {
    version: `nightly-${match[1]}`,
    artifacts: {},
  };
}

//endregion Channels

//region rustup channel archive

/**
 Resolves `rustup` from the versioned channel archive on
 `static.rust-lang.org`
 (rust-lang/rustup publishes no GitHub release assets),
 rebuilding each row's URL and reading the published `rustup-init.sha256`
 beside it.
 
 @param options - Plan entry and fetcher
 
 @returns New version with rebuilt rows
 
 @throws Error when the manifest, a row URL or a checksum is malformed
 */
export async function resolveRustup(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Release manifest body.
   */
  const {text} = (await fetcher('https://static.rust-lang.org/rustup/release-stable.toml',));
  /**
   Manifest version line (`version = '1.29.2'`).
   */
  const match = /^version\s*=\s*['"]([^'"]+)['"]/mu.exec(text);
  if ((match === null) || (match[1] === undefined))
    throw new Error('the rustup release manifest carries no version',);
  /**
   New channel version.
   */
  const newVersion = match[1];
  if (newVersion === entry.version) {
    // Nothing moved: keep the recorded rows instead of re-mapping them.
    return {
      version: entry.version,
      artifacts: entry.artifacts,
    };
  }
  /**
   Rows rebuilt against the new channel archive.
   */
  const mapped: Record<string, {
    url: string;
    sha256: string;
  }> = {};
  for (const [platform, row,] of Object.entries(entry.artifacts,)) {
    /**
     Target triple recorded in the row URL.
     */
    const targetMatch = /\/rustup\/archive\/[^/]+\/([^/]+)\/rustup-init$/u.exec(row.url);
    if ((targetMatch === null) || (targetMatch[1] === undefined)) {
      throw new Error(
        `rustup artifact url ${
          row.url
        } carries no target triple`,
      );
    }
    /**
     Target triple of this row.
     */
    const target = targetMatch[1];
    /**
     New channel archive URL.
     */
    const url = `https://static.rust-lang.org/rustup/archive/${
      newVersion
    }/${
      target
    }/rustup-init`;
    /**
     Published checksum line (`<hex> *./rustup-init`).
     */
    const checksum = (await fetcher(`${url}.sha256`,)).text;
    /**
     Hex digest from the checksum line.
     */
    const digest = checksum.trim()
      .split(/\s/u)[0];
    if ((digest === undefined) || (!/^[0-9a-f]{64}$/u.test(digest,))) {
      throw new Error(
        `rustup archive checksum for ${
          target
        } is not a sha256 line`,
      );
    }
    mapped[platform] = {
      url,
      sha256: digest,
    };
  }
  return {
    version: newVersion,
    artifacts: mapped,
  };
}

//endregion rustup channel archive

//region zig

/**
 Reads the newest stable zig version from the download index, for pin
 reporting only (zig's pin moves only through a plan edit).
 
 @param options - Fetcher
 
 @returns Newest stable index key, or null when only dev builds are listed
 */
export async function resolveZigLatest(
  {
    fetcher,
  }: {
    readonly fetcher: Fetcher;
  },
): Promise<string | null> {
  /**
   Download index keyed by version.
   */
  const doc = parseJson({
    response: await fetcher('https://ziglang.org/download/index.json',),
    label: 'ziglang.org index',
  },) as Record<string, unknown>;
  /**
   Stable keys, sorted descending.
   */
  const stable = Object.keys(doc,)
    .filter(
    function isStable(key,): boolean {
      return (key !== 'master') && (!key.includes('-',));
    },
    )
    .sort(
    function byVersion(
      a,
      b,
    ) {
        /**
         Numeric segments of the two keys.
         */
      const left = a.split('.',)
        .map(Number,);
      const right = b.split('.',)
        .map(Number,);
      for (let at = 0; at < Math.max(
        left.length,
        right.length,
      ); at += 1) {
          /**
           Segment difference at this position.
           */
        const delta = (left[at] ?? 0) - (right[at] ?? 0);
        if (delta !== 0)
          return delta;
      }
      return 0;
    },
    )
    .reverse();
  return stable[0] ?? null;
}

//endregion zig

//region Android SDK

/**
 Resolves `android-sdk` from Google's repository XML: the cmdline-tools
 revision stamps the version and the archive URL carries the build number.
 The repository publishes only a SHA-1, so the SHA-256 is computed from the
 downloaded zip when the URL moves.
 
 @param options - Plan entry and fetcher
 
 @returns New version and rows with freshly hashed digests when the URL moved
 
 @throws Error when the repository lists no cmdline-tools archive
 */
export async function resolveAndroid(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Repository XML body.
   */
  const {text} = (await fetcher('https://dl.google.com/android/repository/repository2-3.xml',));
  /**
   Every cmdline-tools package block with its revision, build and archive.
   */
  const packages: {
    readonly major: number;
    readonly minor: number;
    readonly build: number;
  }[] = [];
  for (const block of text.split('<remotePackage',)) {
    if (!block.includes('commandlinetools-linux-'))
      continue;
    /**
     Revision and archive of one cmdline-tools block.
     */
    const revision = /<revision>\s*<major>(\d+)<\/major>\s*<minor>(\d+)<\/minor>/u.exec(block);
    const archive = /<url>commandlinetools-linux-(\d+)_latest\.zip<\/url>/u.exec(block);
    if ((revision === null) || (archive === null))
      continue;
    if ((revision[1] === undefined) || (revision[2] === undefined)
      || (archive[1] === undefined))
      continue;
    packages.push({
      major: Number(revision[1],),
      minor: Number(revision[2],),
      build: Number(archive[1],),
    },);
  }
  if (packages.length === 0)
    throw new Error('the Android repository lists no cmdline-tools archive',);
  packages.sort(
    function byRevision(
      a,
      b,
    ) {
      return (a.major - b.major)
        || (a.minor - b.minor)
        || (a.build - b.build);
    },
  );
  /**
   Newest cmdline-tools package.
   */
  const newest = packages.at(-1);
  if (newest === undefined)
    throw new Error('the Android repository lists no cmdline-tools archive',);
  /**
   New version stamp and archive URL.
   */
  const version = `${
    newest.major
  }.${
    newest.minor
  }`;
  const url = `https://dl.google.com/android/repository/commandlinetools-linux-${
    newest.build
  }_latest.zip`;
  /**
   The single archive every platform row shares.
   */
  const current = Object.values(entry.artifacts,)[0];
  if (current === undefined)
    throw new Error('the android-sdk entry carries no artifact rows',);
  if (current.url === url) {
    return {
      version,
      artifacts: entry.artifacts,
    };
  }
  /**
   Downloaded zip bytes, hashed because the repository publishes no
   SHA-256 for the archive.
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
    throw new Error('the fetcher returned no bytes to hash for the Android archive',);
  const sha256 = createHash('sha256',)
    .update(response.bytes,)
    .digest('hex',);
  /**
   Mapped rows keyed by platform id.
   */
  const mapped: Record<string, {
    url: string;
    sha256: string;
  }> = {};
  for (const platform of Object.keys(entry.artifacts,))
    mapped[platform] = {
      url,
      sha256,
    };
  return {
    version,
    artifacts: mapped,
  };
}

//endregion Android SDK

//region conda

/**
 Resolves a `conda:` tool from anaconda.org's package record.
 
 @param options - Plan entry and fetcher
 
 @returns New version and rows
 
 @throws Error when no file matches a recorded platform row
 */
export async function resolveConda(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Package owner and name behind the entry.
   */
  const spec = entry.id
    .replace(
    /^conda:/u,
    '',
    );
  /**
   Package record from anaconda.org.
   */
  const doc = parseJson({
    response: await fetcher(`https://api.anaconda.org/package/conda-forge/${spec}`,),
    label: `api.anaconda.org/${spec}`,
  },) as {
    latest_version?: string;
    files?: readonly {
      basename?: string;
      version?: string;
      sha256?: string;
      attrs?: {
        build_number?: number;
        subdir?: string
      };
    }[];
  };
  const version = doc.latest_version;
  if (version === undefined)
    throw new Error(`api.anaconda.org reports no version for ${spec}`,);
  /**
   Files of the newest version indexed by subdirectory.
   */
  const bySubdir = new Map<string, {
    readonly basename: string;
    readonly sha256: string;
  }>();
  for (const file of doc.files ?? []) {
    if ((file.version !== version)
      || (file.basename === undefined)
      || (file.sha256 === undefined)
      || (file.attrs
        ?.subdir
        === undefined))
      continue;
    /**
     Previously recorded build for this subdirectory, kept when it is
     newer (the record lists every build of the version).
     */
    const previous = bySubdir.get(file.attrs
      .subdir,);
    if (previous === undefined) {
      bySubdir.set(
        file.attrs
          .subdir,
        {
          basename: file.basename,
          sha256: file.sha256,
        },
      );
    }
  }
  /**
   Mapped rows keyed by platform id.
   */
  const mapped: Record<string, {
    url: string;
    sha256: string;
  }> = {};
  for (const [platform, row,] of Object.entries(entry.artifacts,)) {
    /**
     Subdirectory the recorded row installs from (`linux-64`, ...).
     */
    const subdir = row.url
      .split('/')
      .at(-2)
      ?? '';
    /**
     Matching file of the newest version.
     */
    const file = bySubdir.get(subdir,);
    if (file === undefined) {
      throw new Error(`api.anaconda.org lists no ${version} file for ${subdir}`,);
    }
    mapped[platform] = {
      url: `https://conda.anaconda.org/conda-forge/${
        file.basename
      }`,
      sha256: file.sha256,
    };
  }
  return {
    version,
    artifacts: mapped,
  };
}

//endregion conda
