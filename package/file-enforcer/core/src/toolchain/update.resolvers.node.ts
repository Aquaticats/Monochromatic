/**
 nodejs.org resolver for `node`, which reads the release index and the
 `SHASUMS256.txt` files of the official and the unofficial (musl) hosts.

 @module
 */

import { basename, } from 'node:path';
import type { ToolPlanEntry, } from './types.ts';
import {
  type Fetcher,
  isSha256Hex,
  type MutableArtifacts,
  parseJson,
  stringField,
  stripPrefix,
  type ToolResolution,
} from './update.resolvers.shared.ts';

/**
 Hex characters in a SHA-256 digest at the start of a sums row.
 */
const DIGEST_LENGTH = 64;

/**
 Parses a `SHASUMS256.txt` body into file name to digest pairs.

 @param text - sums text

 @returns digests keyed by file name

 @example
 ```ts
 parseShasums({ text: `${'a'.repeat(64,)}  node-v26.12.1-linux-x64.tar.gz\n`, },); // Map { 'node-v26.12.1-linux-x64.tar.gz' => 'aaa...' }
 ```
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
     Candidate digest at the start of the row.
     */
    const digest = line.slice(
      0,
      DIGEST_LENGTH,
    );
    /**
     Row remainder: blanks, an optional `*` binary marker, then the file name.
     */
    const rest = line.slice(DIGEST_LENGTH,);
    /**
     Remainder without the blanks that separate digest and name.
     */
    const named = rest.trimStart();
    /**
     File name without the binary marker.
     */
    const name = stripPrefix({
      text: named,
      prefix: '*',
    },)
      .trimEnd();
    if ((!isSha256Hex({ text: digest, },))
      || (named.length === rest.length)
      || (name === ''))
      continue;
    sums.set(
      name,
      digest,
    );
  }
  return sums;
}

/**
 Hosts whose `SHASUMS256.txt` files cover the recorded platforms.
 */
const SUMS_HOSTS = [
  'https://nodejs.org/dist',
  'https://unofficial-builds.nodejs.org/download/release',
] as const;

/**
 Resolves `node` from nodejs.org's release index and sums files.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns new version and rows

 @throws Error when the index or a sums row is missing

 @example
 ```ts
 await resolveNode({ entry, fetcher, },); // { version: '26.12.1', artifacts }
 ```
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
  },);
  /**
   Newest release entry.
   */
  const newest: unknown = Array.isArray(index,)
    ? index[0]
    : undefined;
  /**
   Newest release tag (`v26.12.1`).
   */
  const newTag = stringField({
    doc: newest,
    path: ['version',],
  },);
  if (newTag === '')
    throw new Error('nodejs.org reports no release',);
  /**
   New bare version (`26.12.1`).
   */
  const newVersion = stripPrefix({
    text: newTag,
    prefix: 'v',
  },);
  /**
   Sums responses for the official and the unofficial (musl) host.
   */
  const responses = await Promise.all(SUMS_HOSTS.map(
    function fetchSums(host,) {
      return fetcher(`${
        host
      }/v${
        newVersion
      }/SHASUMS256.txt`,);
    },
  ),);
  /**
   Digests of both hosts keyed by file name (the unofficial host wins ties).
   */
  const sums = new Map<string, string>();
  for (const response of responses) {
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
  const mapped: MutableArtifacts = {};
  for (const [platform, row,] of Object.entries(entry.artifacts,)) {
    /**
     New file name and its sums row.
     */
    const newName = basename(row.url,)
      .replaceAll(
      entry.version,
      newVersion,
      );
    /**
     Digest of the new file.
     */
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
