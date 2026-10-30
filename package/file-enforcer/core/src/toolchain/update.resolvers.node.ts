/**
 nodejs.org resolver for `node`, which reads the release index and the
 `SHASUMS256.txt` files of the official and the unofficial (musl) hosts.

 @module
 */

import { basename, } from 'node:path';
import type { ToolPlanEntry, } from './types.ts';
import {
  type Fetcher,
  isDigits,
  isSha256Hex,
  jsonObjectsOf,
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
 Dot-separated numbers in a stable release tag: major, minor, patch.
 */
const STABLE_TAG_NUMBERS = 3;

/**
 Whether a release tag is a stable release: `v` and three dot-separated numbers, nothing after them.

 The index lists pre-releases (`v27.0.0-alpha.0.0.0`) ahead of the stable releases, and a
 pre-release publishes no sums rows for the recorded artifacts.

 @param tag - release tag from the index `version` field

 @returns whether the tag matches `v<major>.<minor>.<patch>` exactly

 @example
 ```ts
 isStableTag({ tag: 'v26.13.0', },); // true
 isStableTag({ tag: 'v27.0.0-alpha.0.0.0', },); // false
 ```
 */
function isStableTag(
  { tag, }: { readonly tag: string; },
): boolean {
  /**
   Dot-separated numbers after the `v`.
   */
  const numbers = stripPrefix({
    text: tag,
    prefix: 'v',
  },)
    .split('.',);
  return tag.startsWith('v',)
    && (numbers.length === STABLE_TAG_NUMBERS)
    && numbers.every(function isNumber(number,): boolean {
      return isDigits({ text: number, },);
    },);
}

/**
 Resolves `node` from nodejs.org's release index and sums files.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns new version and rows

 @throws Error when the index lists no stable release or a sums row is missing

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
   Release index listing the newest release first, pre-releases included.
   */
  const index = parseJson({
    response: await fetcher('https://nodejs.org/dist/index.json',),
    label: 'nodejs.org index',
  },);
  /**
   Release tags in index order (`v27.0.0-alpha.0.0.0`, `v26.13.0`, ...).
   */
  const tags = jsonObjectsOf({ value: index, },)
    .map(function versionOf(release,): string {
      return stringField({
        doc: release,
        path: ['version',],
      },);
    },);
  /**
   Newest stable release tag (`v26.13.0`), empty when the index lists none.
   */
  const newTag = tags.find(function isStable(tag,): boolean {
    return isStableTag({ tag, },);
  },) ?? '';
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
