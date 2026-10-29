/**
 conda resolver: anaconda.org's package record for `conda:` tools.

 @module
 */

import type { ToolPlanEntry, } from './types.ts';
import {
  type Fetcher,
  isJsonObject,
  type JsonObject,
  jsonObjectsOf,
  type MutableArtifacts,
  parseJson,
  stringField,
  stripPrefix,
  type ToolResolution,
} from './update.resolvers.shared.ts';

/**
 Build file of one version in a subdirectory.
 */
type CondaFile = {
  readonly basename: string;
  readonly sha256: string;
};

/**
 Indexes the files of one version by subdirectory, keeping the first build
 the record lists for each.

 @param files - parsed `files` entries of the package record

 @param version - version whose files to keep

 @returns files by subdirectory (`linux-64`, ...)
 */
function filesBySubdir(
  {
    files,
    version,
  }: {
    readonly files: readonly JsonObject[];
    readonly version: string;
  },
): ReadonlyMap<string, CondaFile> {
  /**
   Files of the newest version indexed by subdirectory.
   */
  const bySubdir = new Map<string, CondaFile>();
  for (const file of files) {
    /**
     Subdirectory of this build.
     */
    const subdir = stringField({
      doc: file,
      path: [
        'attrs',
        'subdir',
      ],
    },);
    /**
     File name of this build.
     */
    const basename = stringField({
      doc: file,
      path: ['basename',],
    },);
    /**
     Digest of this build.
     */
    const sha256 = stringField({
      doc: file,
      path: ['sha256',],
    },);
    /**
     Version this build belongs to.
     */
    const fileVersion = stringField({
      doc: file,
      path: ['version',],
    },);
    if ((fileVersion !== version)
      || (basename === '')
      || (sha256 === '')
      || (subdir === '')
      || bySubdir.has(subdir,))
      continue;
    bySubdir.set(
      subdir,
      {
        basename,
        sha256,
      },
    );
  }
  return bySubdir;
}

/**
 Resolves a `conda:` tool from anaconda.org's package record.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns new version and rows

 @throws Error when no file matches a recorded platform row

 @example
 ```ts
 await resolveConda({ entry, fetcher, },); // { version: '0.2.0', artifacts }
 ```
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
  const spec = stripPrefix({
    text: entry.id,
    prefix: 'conda:',
  },);
  /**
   Package record from anaconda.org.
   */
  const doc = parseJson({
    response: await fetcher(`https://api.anaconda.org/package/conda-forge/${spec}`,),
    label: `api.anaconda.org/${spec}`,
  },);
  /**
   Newest version the record lists.
   */
  const version = stringField({
    doc,
    path: ['latest_version',],
  },);
  if (version === '')
    throw new Error(`api.anaconda.org reports no version for ${spec}`,);
  /**
   Raw file list of the record.
   */
  const rawFiles: unknown = isJsonObject(doc,)
    ? doc.files
    : undefined;
  /**
   Files of the newest version indexed by subdirectory.
   */
  const bySubdir = filesBySubdir({
    files: jsonObjectsOf({ value: rawFiles, },),
    version,
  },);
  /**
   Mapped rows keyed by platform id.
   */
  const mapped: MutableArtifacts = {};
  for (const [platform, row,] of Object.entries(entry.artifacts,)) {
    /**
     Subdirectory the recorded row installs from (`linux-64`, ...).
     */
    const subdir = row.url
      .split('/',)
      .at(-2,)
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
