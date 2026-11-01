/**
 Fills the trust-history cache from the crates.io database dump.

 crates.io's data-access policy lists its API last, after the sparse index,
 static.crates.io, and the database dump (`https://static.crates.io/db-dump.tar.gz`,
 rebuilt daily, no rate limit), and asks bulk readers to use the dump. A cold
 verification of every locked crate is bulk access (1,024 crates measured on
 2026-10-29), so the owner chose the dump for it (2026-10-29).

 The dump exports `versions.published_by`, `versions.created_at`, and the
 per-version `repository`, but keeps every Trusted Publishing column private
 (`versions.trustpub_data` and the `trustpub_*` tables are "private" in
 crates.io's `dump-db.toml` at commit 55d3c7bf0d5d0ffc2fc6217d7ce0f97cc798ae02).
 A Trusted Publishing release has no `published_by`: across 785 crates of API
 history (37,822 releases, 1,470 of them Trusted Publishing) none had one. So a
 crate with a publisher-less release since Trusted Publishing began cannot be
 rated from the dump and is left to the API; every other crate's history is
 written from the dump in the API's cache shape.

 @module
 */

import {
  access,
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  columnIndexes,
  createCsvParser,
  DumpFormatError,
} from './csv.ts';
import type { VersionRecord, } from './policy.ts';
import {
  createTarReader,
  SKIP_ENTRY,
  STOP_ARCHIVE,
  type TarEntry,
  type TarEntryHandler,
} from './tar.ts';

//region Constants

/**
 Earliest time a release can be a Trusted Publishing release.
 crates.io's oldest Trusted Publishing release in the cached API histories is
 2025-07-12T16:49:05Z (785 crates, measured 2026-10-29); the cutoff sits a few
 days earlier, so a release it misses is sent to the API rather than misread.
 */
export const TRUSTED_PUBLISHING_START = '2025-07-01T00:00:00Z';

/**
 Archive path suffix of the crate table.
 */
const CRATES_ENTRY = '/data/crates.csv';

/**
 Archive path suffix of the version table.
 */
const VERSIONS_ENTRY = '/data/versions.csv';

//endregion Constants

//region Records

/**
 ISO form of a dump timestamp such as `2019-02-22 19:31:11.109728+00`.

 @param value - PostgreSQL text timestamp in UTC

 @returns ISO 8601 timestamp the policy can `Date.parse`

 @throws DumpFormatError when the value is not a UTC timestamp

 @example
 ```ts
 dumpTimestamp('2019-02-22 19:31:11.109728+00'); // '2019-02-22T19:31:11.109728Z'
 ```
 */
export function dumpTimestamp(value: string,): string {
  /**
   Date and time parts.
   */
  const [date = '', time = '',] = value.split(' ',);
  /**
   ISO form; the dump writes UTC as `+00`.
   */
  const iso = time.endsWith('+00',) ? `${date}T${time.slice(
    0,
    -'+00'.length,
  )}Z` : value;
  if (Number.isNaN(Date.parse(iso,),))
    throw new DumpFormatError(`the crates.io dump has a created_at value that is not a UTC timestamp: ${JSON.stringify(value,)}`,);
  return iso;
}

/**
 Whether a crate's history has a release the dump cannot rate:
 no publisher at a time when it may be a Trusted Publishing release.

 @param history - the crate's releases

 @returns whether the API must supply this crate's history

 @example
 ```ts
 needsApi([{ num: '1.0.0', createdAt: '2026-01-01T00:00:00Z', publisherId: '', trustpubRepository: '', repository: '', },]); // true
 ```
 */
export function needsApi(history: readonly VersionRecord[],): boolean {
  /**
   Cutoff in milliseconds.
   */
  const start = Date.parse(TRUSTED_PUBLISHING_START,);
  return history.some(function isAmbiguous(record: VersionRecord,): boolean {
    return (record.publisherId === '') && (Date.parse(record.createdAt,) >= start);
  },);
}

//endregion Records

//region Fill

/**
 Outcome of a fill.

 @example
 ```ts
 const result: DumpFillResult = { found: 1024, written: 883, leftToApi: ['oxc_parser',], kept: 0, };
 ```
 */
export type DumpFillResult = {
  /**
   Requested crates the dump has.
   */
  readonly found: number;
  /**
   Cache files written.
   */
  readonly written: number;
  /**
   Crates left to the API because of a possible Trusted Publishing release.
   */
  readonly leftToApi: readonly string[];
  /**
   Crates skipped because a cache file already existed (API histories are richer).
   */
  readonly kept: number;
};

/**
 Whether a file exists.

 @param path - file path

 @returns whether it can be accessed

 @example
 ```ts
 await fileExists('.cache/rust-catalog-update/crates-io/anyhow.json');
 ```
 */
async function fileExists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  } catch (error) {
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT'))
      return false;
    throw error;
  }
}

/**
 Handler for one table: parses rows and hands each to `onRow` after the header.

 @param columns - columns the table must have

 @param onRow - receives each data row and the column positions

 @param onEnd - runs after the last row

 @returns tar entry handler

 @example
 ```ts
 tableHandler({ columns: ['id',], onRow, onEnd, });
 ```
 */
function tableHandler(
  {
    columns,
    onRow,
    onEnd,
  }: {
    readonly columns: readonly string[];
    readonly onRow: (request: {
      readonly row: readonly string[];
      readonly at: ReadonlyMap<string, number>
    },) => void;
    readonly onEnd: () => void;
  },
): TarEntryHandler {
  /**
   Column positions, set by the header row.
   */
  const header = {
    at: new Map<string, number>() as ReadonlyMap<string, number>,
    read: false,
  };
  /**
   CSV parser for this table.
   */
  const parser = createCsvParser({
    onRow: function onCsvRow(row: readonly string[],): void {
      if (!header.read) {
        header.at = columnIndexes({
          header: row,
          columns,
        },);
        header.read = true;
        return;
      }
      onRow({
        row,
        at: header.at,
      },);
    },
  },);
  return {
    push: parser.push,
    end: function end(): void {
      parser.end();
      onEnd();
    },
  };
}

/**
 Field of a row by column name.

 @param row - data row

 @param at - column positions

 @param column - column name

 @returns field text

 @example
 ```ts
 field({ row, at, column: 'name', });
 ```
 */
function field({
  row,
  at,
  column,
}: {
  readonly row: readonly string[];
  readonly at: ReadonlyMap<string, number>;
  readonly column: string
},): string {
  return row[at.get(column,) ?? (-1)] ?? '';
}

/**
 Reads `crates.csv` and `versions.csv` from decompressed dump bytes and writes cache files.

 @param tarChunks - decompressed tar bytes of `db-dump.tar.gz`, in order

 @param names - crate names to fill

 @param cacheDir - directory of per-crate history files

 @returns counts and the crates left to the API

 @throws DumpFormatError when the archive or a table differs from the expected layout

 @example
 ```ts
 await fillHistoriesFromDump({ tarChunks: gunzipped, names: new Set(['anyhow',]), cacheDir, });
 ```
 */
export async function fillHistoriesFromDump(
  {
    tarChunks,
    names,
    cacheDir,
  }: {
    readonly tarChunks: AsyncIterable<Uint8Array>;
    readonly names: ReadonlySet<string>;
    readonly cacheDir: string;
  },
): Promise<DumpFillResult> {
  /**
   Crate id to name, for requested crates.
   */
  const idToName = new Map<string, string>();
  /**
   Releases per requested crate.
   */
  const histories = new Map<string, VersionRecord[]>();
  /**
   Which tables have been read.
   */
  const progress = {
    crates: false,
    versions: false,
  };

  /**
   Reader decision per archive entry.
   */
  const reader = createTarReader({
    decide: function decide(entry: TarEntry,) {
      if (entry.name
        .endsWith(CRATES_ENTRY,)) {
        return tableHandler({
          columns: [
            'id',
            'name',
          ],
          onRow: function onCrate({
            row,
            at,
          },): void {
            /**
             Crate name.
             */
            const name = field({
              row,
              at,
              column: 'name',
            },);
            if (names.has(name,))
              idToName.set(
                field({
                  row,
                  at,
                  column: 'id',
                },),
                name,
              );
          },
          onEnd: function onCratesEnd(): void {
            progress.crates = true;
          },
        },);
      }
      if (entry.name
        .endsWith(VERSIONS_ENTRY,)) {
        if (!progress.crates)
          throw new DumpFormatError('the crates.io dump lists versions.csv before crates.csv; the single-pass reader needs crates.csv first',);
        return tableHandler({
          columns: [
            'crate_id',
            'num',
            'created_at',
            'published_by',
            'repository',
          ],
          onRow: function onVersion({
            row,
            at,
          },): void {
            /**
             Requested crate this version belongs to, if any.
             */
            const name = idToName.get(field({
              row,
              at,
              column: 'crate_id',
            },),);
            if (name === undefined)
              return;
            /**
             Releases collected for this crate so far.
             */
            const releases = histories.get(name,) ?? [];
            if (releases.length === 0)
              histories.set(
                name,
                releases,
              );
            releases.push({
              num: field({
                row,
                at,
                column: 'num',
              },),
              createdAt: dumpTimestamp(field({
                row,
                at,
                column: 'created_at',
              },),),
              publisherId: field({
                row,
                at,
                column: 'published_by',
              },),
              trustpubRepository: '',
              repository: field({
                row,
                at,
                column: 'repository',
              },),
            },);
          },
          onEnd: function onVersionsEnd(): void {
            progress.versions = true;
          },
        },);
      }
      return (progress.crates && progress.versions) ? STOP_ARCHIVE : SKIP_ENTRY;
    },
  },);
  for await (const chunk of tarChunks) {
    if ((!reader.push(chunk,)) || (progress.crates && progress.versions))
      break;
  }
  if (!progress.versions)
    throw new DumpFormatError('the crates.io dump ended before versions.csv was read completely',);

  await mkdir(
    cacheDir,
    { recursive: true, },
  );
  /**
   Per-crate outcomes.
   */
  const outcomes = await Promise.all([...histories,].map(async function writeHistory([name, history,],): Promise<'written' | 'api' | 'kept'> {
    if (needsApi(history,))
      return 'api';
    /**
     Cache file for this crate.
     */
    const path = join(
      cacheDir,
      `${name}.json`,
    );
    if (await fileExists(path,))
      return 'kept';
    await writeFile(
      path,
      `${JSON.stringify(history,)}\n`,
    );
    return 'written';
  },),);
  return {
    found: histories.size,
    written: outcomes.filter(function isWritten(outcome,): boolean {
      return outcome === 'written';
    },)
      .length,
    leftToApi: [...histories.keys(),].filter(function isApi(
      _name: string,
      index: number,
    ): boolean {
      return outcomes[index] === 'api';
    },)
      .toSorted(),
    kept: outcomes.filter(function isKept(outcome,): boolean {
      return outcome === 'kept';
    },)
      .length,
  };
}

//endregion Fill
