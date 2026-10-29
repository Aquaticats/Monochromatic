/**
 Rust resolvers: the `rust` toolchain from its rustup channel manifest and
 `rustup` from the versioned channel archive on `static.rust-lang.org`.

 @module
 */

import type { ArtifactRow, } from './plan.artifacts.data.ts';
import type { ToolPlanEntry, } from './types.ts';
import {
  type Fetcher,
  firstWord,
  isSha256Hex,
  type MutableArtifacts,
  quotedAssignment,
  type ToolResolution,
} from './update.resolvers.shared.ts';

//region Channels

/**
 Quote characters a manifest value may use.
 */
const BOTH_QUOTES = `'"`;

/**
 First non-empty string of a list.

 @param values - candidate strings in priority order

 @returns first non-empty string, or empty when all are empty
 */
function firstPresent(values: readonly string[],): string {
  return values.find(function isPresent(value,): boolean {
    return value !== '';
  },) ?? '';
}

/**
 First non-empty quoted value that `key = "value"` lines carry.

 @param text - manifest body

 @param key - key the line must start with

 @param quotes - quote characters that may open and close the value

 @returns value of the first matching line, or empty when no line matches
 */
function firstAssignment(
  {
    text,
    key,
    quotes,
  }: {
    readonly text: string;
    readonly key: string;
    readonly quotes: string;
  },
): string {
  return firstPresent(text.split('\n',)
    .map(function valueOfLine(line,): string {
      return quotedAssignment({
        line,
        key,
        quotes,
      },);
    },),);
}

/**
 Whether a line is the exact `[pkg.rust]` section header, not a target subsection.

 @param line - one manifest line

 @returns whether the line is the header followed only by blanks
 */
function isPkgRustHeader(
  { line, }: { readonly line: string; },
): boolean {
  /**
   Text after the header name.
   */
  const afterName = line.slice('[pkg.rust]'.length,)
    .trim();
  return line.startsWith('[pkg.rust]',)
    && (afterName === '');
}

/**
 `version` of the `[pkg.rust]` section: the field on the line right after an
 exact `[pkg.rust]` header, before the target subsections.

 @param text - channel manifest body

 @returns version text, or empty when the section carries none
 */
function pkgRustVersion(
  { text, }: { readonly text: string; },
): string {
  /**
   Manifest lines.
   */
  const lines = text.split('\n',);
  return firstPresent(lines.map(
    function versionAfterHeader(
      line,
      at,
    ): string {
      return isPkgRustHeader({ line, },)
        ? quotedAssignment({
          line: lines[at + 1] ?? '',
          key: 'version',
          quotes: '"',
        },)
        : '';
    },
  ),);
}

/**
 Resolves the `rust` toolchain from its rustup channel manifest, with two
 behaviors dispatched on the channel name in `entry.request`: `nightly`
 returns the `nightly-<date>` stamp of the top-level `date` line in
 `channel-rust-nightly.toml`, while every other channel (`stable`, ...)
 returns the `[pkg.rust]` section's `version` value in
 `channel-rust-<channel>.toml`. A request carrying no channel name keeps
 the nightly behavior.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns channel version with no rows

 @throws Error when the manifest carries no date or `[pkg.rust]` version

 @example
 ```ts
 await resolveRust({ entry, fetcher, },); // { version: 'nightly-2026-10-26', artifacts: {} }
 ```
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
  const { text, } = await fetcher(`https://static.rust-lang.org/dist/channel-rust-${
    channel
  }.toml`,);
  if (channel !== 'nightly') {
    /**
     `[pkg.rust]` version field.
     */
    const version = pkgRustVersion({ text, },);
    if (version === '') {
      throw new Error(`the rustup ${
        channel
      } manifest carries no [pkg.rust] version`,);
    }
    return {
      version,
      artifacts: {},
    };
  }
  /**
   Manifest date.
   */
  const date = firstAssignment({
    text,
    key: 'date',
    quotes: '"',
  },);
  if (date === '')
    throw new Error('the rustup nightly manifest carries no date',);
  return {
    version: `nightly-${date}`,
    artifacts: {},
  };
}

//endregion Channels

//region rustup channel archive

/**
 Segments below the archive marker: version, target triple, `rustup-init`.
 */
const ARCHIVE_SEGMENTS = 3;

/**
 Reads the target triple out of a rustup archive URL.

 @param url - recorded row URL (`.../rustup/archive/<version>/<target>/rustup-init`)

 @returns target triple, or empty when the URL is not a rustup archive URL
 */
function rustupTarget(
  { url, }: { readonly url: string; },
): string {
  /**
   Marker between the host and the archive path.
   */
  const marker = '/rustup/archive/';
  /**
   Index of the marker in the URL.
   */
  const at = url.indexOf(marker,);
  if (at === (-1))
    return '';
  /**
   Path below the marker.
   */
  const segments = url.slice(at + marker.length,)
    .split('/',);
  /**
   Version, target triple, and file name.
   */
  const [version = '', target = '', file,] = segments;
  return (segments.length === ARCHIVE_SEGMENTS)
    && (version !== '')
    && (file === 'rustup-init')
    ? target
    : '';
}

/**
 Platform id with its recorded artifact row, as `Object.entries` lists them.
 */
type RowEntry = readonly [
  string,
  ArtifactRow,
];

/**
 Platform id with its rebuilt artifact row.
 */
type RebuiltRow = readonly [
  string,
  {
    url: string;
    sha256: string;
  },
];

/**
 Rebuilds one row against the new channel archive and reads its published checksum.

 @param platform - platform id of the row

 @param row - recorded row whose URL names the target triple

 @param newVersion - channel version to rebuild the URL for

 @param fetcher - injected GET

 @returns platform id with the rebuilt row

 @throws Error when the row URL carries no target triple or the checksum is not a sha256 line
 */
async function rebuildRustupRow(
  {
    platform,
    row,
    newVersion,
    fetcher,
  }: {
    readonly platform: string;
    readonly row: ArtifactRow;
    readonly newVersion: string;
    readonly fetcher: Fetcher;
  },
): Promise<RebuiltRow> {
  /**
   Target triple recorded in the row URL.
   */
  const target = rustupTarget({ url: row.url, },);
  if (target === '') {
    throw new Error(
      `rustup artifact url ${
        row.url
      } carries no target triple`,
    );
  }
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
  const digest = firstWord({ text: checksum, },);
  if (!isSha256Hex({ text: digest, },)) {
    throw new Error(
      `rustup archive checksum for ${
        target
      } is not a sha256 line`,
    );
  }
  return [
    platform,
    {
      url,
      sha256: digest,
    },
  ];
}

/**
 Resolves `rustup` from the versioned channel archive on
 `static.rust-lang.org`
 (rust-lang/rustup publishes no GitHub release assets),
 rebuilding each row's URL and reading the published `rustup-init.sha256`
 beside it.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns new version with rebuilt rows

 @throws Error when the manifest, a row URL or a checksum is malformed

 @example
 ```ts
 await resolveRustup({ entry, fetcher, },); // { version: '1.29.3', artifacts }
 ```
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
  const { text, } = await fetcher('https://static.rust-lang.org/rustup/release-stable.toml',);
  /**
   New channel version from the manifest `version = '1.29.2'` line.
   */
  const newVersion = firstAssignment({
    text,
    key: 'version',
    quotes: BOTH_QUOTES,
  },);
  if (newVersion === '')
    throw new Error('the rustup release manifest carries no version',);
  if (newVersion === entry.version) {
    // Nothing moved: keep the recorded rows instead of re-mapping them.
    return {
      version: entry.version,
      artifacts: entry.artifacts,
    };
  }
  /**
   Rows rebuilt against the new channel archive, one checksum fetch per row.
   */
  const rebuilt = await Promise.all(Object.entries(entry.artifacts,)
    .map(function rebuild(pair: RowEntry,) {
      return rebuildRustupRow({
        platform: pair[0],
        row: pair[1],
        newVersion,
        fetcher,
      },);
    },),);
  /**
   Rebuilt rows keyed by platform id.
   */
  const mapped: MutableArtifacts = Object.fromEntries(rebuilt,);
  return {
    version: newVersion,
    artifacts: mapped,
  };
}

//endregion rustup channel archive
