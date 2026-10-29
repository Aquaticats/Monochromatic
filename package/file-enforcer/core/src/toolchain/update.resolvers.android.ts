/**
 Android SDK resolver: Google's repository XML stamps the cmdline-tools
 revision and carries the archive build number.

 @module
 */

import { createHash, } from 'node:crypto';
import type { ToolPlanEntry, } from './types.ts';
import {
  type Fetcher,
  isDigits,
  type MutableArtifacts,
  type ToolResolution,
} from './update.resolvers.shared.ts';

/**
 One cmdline-tools package of the repository: revision and archive build.
 */
type CmdlinePackage = {
  readonly major: number;
  readonly minor: number;
  readonly build: number;
};

/**
 Opening of the linux cmdline-tools archive file name, inside a `<url>` element.
 */
const ARCHIVE_OPEN = 'url>commandlinetools-linux-';

/**
 Closing of the archive file name.
 */
const ARCHIVE_CLOSE = '_latest.zip';

/**
 Reads the digits of a numeric element token such as `major>23`.

 @param token - text after one `<`, with trailing blanks removed

 @param tag - element name the token must open

 @returns digit text, or empty when the token is not that element with digits
 */
function digitsOfToken(
  {
    token,
    tag,
  }: {
    readonly token: string;
    readonly tag: string;
  },
): string {
  /**
   Opening of the element, with its closing angle bracket.
   */
  const open = `${tag}>`;
  /**
   Text after the opening.
   */
  const digits = token.slice(open.length,);
  return token.startsWith(open,) && isDigits({ text: digits, },)
    ? digits
    : '';
}

/**
 Reads the revision and archive build out of one `<remotePackage` block.

 @param block - XML text between two `<remotePackage` openings

 @returns one package when the block is a linux cmdline-tools package, none otherwise
 */
function cmdlinePackageFromBlock(block: string,): readonly CmdlinePackage[] {
  if (!block.includes('commandlinetools-linux-',))
    return [];
  /**
   Text after each `<`, with blanks between elements dropped.
   */
  const tokens = block.split('<',)
    .map(function trimToken(token,): string {
      return token.trimEnd();
    },);
  /**
   Index of the `<revision>` token (a `<min-revision>` elsewhere does not count).
   */
  const revisionAt = tokens.indexOf('revision>',);
  /**
   Index of the archive `<url>` token whose element closes right after.
   */
  const archiveAt = tokens.findIndex(function isArchive(
    token,
    at,
  ): boolean {
    return token.startsWith(ARCHIVE_OPEN,)
      && token.endsWith(ARCHIVE_CLOSE,)
      && (tokens[at + 1] === '/url>');
  },);
  if ((revisionAt === (-1)) || (archiveAt === (-1)))
    return [];
  /**
   Revision elements in order: major, its closing tag, minor, its closing tag.
   */
  const [majorToken = '', majorClose = '', minorToken = '', minorClose = '',] = tokens.slice(revisionAt + 1,);
  /**
   Major revision digits.
   */
  const major = digitsOfToken({
    token: majorToken,
    tag: 'major',
  },);
  /**
   Minor revision digits.
   */
  const minor = digitsOfToken({
    token: minorToken,
    tag: 'minor',
  },);
  /**
   Build number digits inside the archive file name.
   */
  const build = (tokens[archiveAt] ?? '').slice(
    ARCHIVE_OPEN.length,
    -ARCHIVE_CLOSE.length,
  );
  if ((majorClose !== '/major>')
    || (minorClose !== '/minor>')
    || (major === '')
    || (minor === '')
    || (!isDigits({ text: build, },)))
    return [];
  return [
    {
      major: Number(major,),
      minor: Number(minor,),
      build: Number(build,),
    },
  ];
}

/**
 Orders packages oldest first by revision, then build.

 @param left - first package

 @param right - second package

 @returns negative when `left` is older, positive when newer, zero when equal
 */
function byRevision(
  {
    left,
    right,
  }: {
    readonly left: CmdlinePackage;
    readonly right: CmdlinePackage;
  },
): number {
  return (left.major - right.major)
    || (left.minor - right.minor)
    || (left.build - right.build);
}

/**
 Resolves `android-sdk` from Google's repository XML: the cmdline-tools
 revision stamps the version and the archive URL carries the build number.
 The repository publishes only a SHA-1, so the SHA-256 is computed from the
 downloaded zip when the URL moves.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns new version and rows with freshly hashed digests when the URL moved

 @throws Error when the repository lists no cmdline-tools archive

 @example
 ```ts
 await resolveAndroid({ entry, fetcher, },); // { version: '24.0', artifacts }
 ```
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
  const { text, } = await fetcher('https://dl.google.com/android/repository/repository2-3.xml',);
  /**
   Newest cmdline-tools package.
   */
  const newest = text.split('<remotePackage',)
    .flatMap(cmdlinePackageFromBlock,)
    .toSorted(function oldestFirst(
      left,
      right,
    ): number {
      return byRevision({
        left,
        right,
      },);
    },)
    .at(-1,);
  if (newest === undefined)
    throw new Error('the Android repository lists no cmdline-tools archive',);
  /**
   New version stamp.
   */
  const version = `${
    newest.major
  }.${
    newest.minor
  }`;
  /**
   New archive URL.
   */
  const url = `https://dl.google.com/android/repository/commandlinetools-linux-${
    newest.build
  }_latest.zip`;
  /**
   The single archive every platform row shares.
   */
  const [current,] = Object.values(entry.artifacts,);
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
  /**
   SHA-256 of the archive.
   */
  const sha256 = createHash('sha256',)
    .update(response.bytes,)
    .digest('hex',);
  /**
   Mapped rows keyed by platform id.
   */
  const mapped: MutableArtifacts = {};
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
