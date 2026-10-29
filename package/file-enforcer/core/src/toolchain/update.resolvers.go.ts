/**
 Go resolvers: the `go` runtime from go.dev's download index and `go-install`
 modules from the Go module proxy.

 @module
 */

import type { ToolPlanEntry, } from './types.ts';
import {
  type Fetcher,
  type JsonObject,
  jsonObjectsOf,
  type MutableArtifacts,
  parseJson,
  stringField,
  stripPrefix,
  type ToolResolution,
} from './update.resolvers.shared.ts';

/**
 One file of a go.dev download-index release.
 */
type GoFile = {
  readonly filename: string;
  readonly os: string;
  readonly arch: string;
  readonly kind: string;
  readonly sha256: string;
};

/**
 One go.dev download-index release record.
 */
type GoRelease = {
  readonly version: string;
  readonly stable: boolean;
  readonly files: readonly GoFile[];
};

/**
 Reads one file record of a release.

 @param file - parsed file record

 @returns file with absent fields empty
 */
function goFileFromJson(file: JsonObject,): GoFile {
  return {
    filename: stringField({
      doc: file,
      path: ['filename',],
    },),
    os: stringField({
      doc: file,
      path: ['os',],
    },),
    arch: stringField({
      doc: file,
      path: ['arch',],
    },),
    kind: stringField({
      doc: file,
      path: ['kind',],
    },),
    sha256: stringField({
      doc: file,
      path: ['sha256',],
    },),
  };
}

/**
 Reads one release record of the download index.

 @param release - parsed release record

 @returns release with absent fields empty
 */
function goReleaseFromJson(release: JsonObject,): GoRelease {
  /**
   Raw file list.
   */
  const { files, } = release;
  return {
    version: stringField({
      doc: release,
      path: ['version',],
    },),
    stable: release.stable === true,
    files: jsonObjectsOf({ value: files, },)
      .map(goFileFromJson,),
  };
}

/**
 Resolves the `go` runtime from go.dev's download index: the release whose
 `go`-prefixed version equals `entry.version`, else the newest stable
 release, with rows recorded from that release's linux `archive` files.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns bare `go` version (the `go` prefix stripped) and linux archive rows

 @throws Error when the index carries neither the recorded version nor a
 stable release, or when the chosen release lists no linux archive

 @example
 ```ts
 await resolveGoRuntime({ entry, fetcher, },); // { version: '1.27.3', artifacts }
 ```
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
   Parsed download index.
   */
  const doc = parseJson({
    response: await fetcher('https://go.dev/dl/?mode=json&include=all',),
    label: 'go.dev download index',
  },);
  /**
   Every go.dev release, newest first.
   */
  const releases = jsonObjectsOf({ value: doc, },)
    .map(goReleaseFromJson,);
  /**
   Release to record: the recorded version, else the newest stable release.
   */
  const chosen = releases.find(
    function carriesRecordedVersion(release,): boolean {
      return stripPrefix({
        text: release.version,
        prefix: 'go',
      },)
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
  const mapped: MutableArtifacts = {};
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
        : '';
    if (platform === '')
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
    version: stripPrefix({
      text: chosen.version,
      prefix: 'go',
    },),
    artifacts: mapped,
  };
}

/**
 Resolves a `go-install` tool from the Go module proxy's `@latest` record,
 keeping the build-from-source semantics `go install <module>@v<version>`.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns bare module version (one leading `v` stripped) with no rows

 @throws Error when the proxy rejects the query or reports no version

 @example
 ```ts
 await resolveGoModule({ entry, fetcher, },); // { version: '0.98.1', artifacts: {} }
 ```
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
    ?? stripPrefix({
      text: entry.id,
      prefix: 'go:',
    },);
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
  /**
   Version tag the proxy reports (`v0.98.1`).
   */
  const tag = stringField({
    doc: parseJson({
      response,
      label: `proxy.golang.org/${module}`,
    },),
    path: ['Version',],
  },);
  if (tag === '') {
    throw new Error(`proxy.golang.org reports no version for ${
      module
    }`,);
  }
  return {
    version: stripPrefix({
      text: tag,
      prefix: 'v',
    },),
    artifacts: {},
  };
}
