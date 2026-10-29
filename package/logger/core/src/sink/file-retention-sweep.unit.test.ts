/**
 Age-window retention sweeps stay inside disposable fixtures and touch only owned names. @module
 */
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  _cutoffDay as cutoffDay,
  _sweepLogDirectory as sweepLogDirectory,
  type LogRetentionPolicy,
} from '@monochromatic-dev/logger-core/node';

//region Fixture constants and helpers

/**
 Sweep instant shared by the fixtures: 2026-10-22T12:00:00Z.
 */
const NOW = Date.UTC(
  2_026,
  9,
  22,
  12,
  0,
  0,
);

/**
 Throttle window asserted against the documented five-minute default.
 */
const DEFAULT_STAMP_INTERVAL_MS = 300_000;

/**
 Modification time two hours before {@link NOW}, beyond the one-hour fresh-file grace.
 */
const AGED = NOW - (2 * 3_600_000);

/**
 Modification time at {@link NOW}, inside the fresh-file grace.
 */
const FRESH = NOW;

/**
 Retention bounds that expire nothing by age or budget,
 so a test controls only its own knob.
 */
const WIDE_POLICY: LogRetentionPolicy = {
  maxAgeDays: 14,
  maxBytes: 536_870_912,
  maxFiles: 50_000,
};

/**
 Stamp name a completed sweep leaves in the destination root.
 */
const STAMP_NAME = '.logger-retention-sweep';

/**
 Create a disposable destination directory outside every shared log tree.

 @returns Async-disposable fixture root under the agent scratch directory
 */
async function fixture() {
  /**
   Agent scratch root, made private before any fixture is created inside it.
   */
  const scratch = join(
    homedir(),
    'temp',
    'agent',
  );
  await mkdir(
    scratch,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  await chmod(scratch, 0o700,);
  const path = await mkdtemp(join(
    scratch,
    'logger-sweep-',),
  );
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Write one fixture entry with an exact modification time,
 so age and grace behavior never depend on the wall clock.

 @param directory - fixture root holding the entry

 @param name - entry name, optionally one level below the root

 @param content - byte payload counted by the budget pass

 @param modified - modification instant the budget pass measures
 */
async function writeEntry({
  directory,
  name,
  content = 'x',
  modified = AGED,
}: {
  readonly directory: string;
  readonly name: string;
  readonly content?: string;
  readonly modified?: number;
},): Promise<void> {
  /**
   Absolute fixture path of the entry under construction.
   */
  const path = join(
    directory,
    name,
  );
  await mkdir(
    dirname(path,),
    {
      recursive: true,
      mode: 0o700,
    },
  );
  await writeFile(path, content,);
  await utimes(
    path,
    new Date(modified,),
    new Date(modified,),
  );
}

/**
 List every fixture entry one and two levels deep,
 so assertions see both root names and day-directory contents.

 @param directory - fixture root to list

 @returns Sorted `name` and `day/name` entry paths
 */
async function tree(directory: string,): Promise<string[]> {
  /**
   Root entries, expanded one level into their directories.
   */
  const entries = await readdir(
    directory,
    { withFileTypes: true, },
  );
  const names = await Promise.all(entries.map(async function listEntry(entry,) {
    if (!entry.isDirectory())
      return [entry.name,];
    /**
     Names directly inside one root directory.
     */
    const inner = await readdir(join(
      directory,
      entry.name,
    ),);
    return [
      entry.name,
      ...inner.map(function withParent(name: string,) {
        return `${entry.name}/${name}`;
      },),
    ];
  },),);
  return names.flat()
    .toSorted();
}

/**
 Collect one sweep's reported faults instead of writing to the host console.

 @returns Collector function and the reports it received
 */
function reportCollector(): {
  readonly reports: { readonly context: string; readonly error: unknown, }[];
  readonly reportError: (report: { readonly context: string; readonly error: unknown, },) => void;
} {
  /**
   Reports captured from the injected sweep reporter.
   */
  const reports: { readonly context: string; readonly error: unknown, }[] = [];
  return {
    reports,
    reportError: function collect(report,) {
      reports.push(report,);
    },
  };
}

//endregion Fixture constants and helpers

await describe({
  name: '',
  children: [
    describe({
      name: cutoffDay.name,
      children: [
        it({
          name: 'returns the UTC day key of the age boundary',
          fn: async () => {
            expect(cutoffDay({
              now: NOW,
              maxAgeDays: 14,
            },),).toBe('2026-10-08',);
          },
        },),
        it({
          name: 'keys the boundary by whole days, including across month and year edges',
          fn: async () => {
            expect(cutoffDay({
              now: NOW,
              maxAgeDays: 0,
            },),).toBe('2026-10-22',);
            expect(cutoffDay({
              now: NOW,
              maxAgeDays: 1,
            },),).toBe('2026-10-21',);
            expect(cutoffDay({
              now: Date.UTC(
                2_026,
                0,
                2,
              ),
              maxAgeDays: 5,
            },),).toBe('2025-12-28',);
          },
        },),
      ],
    },),

    describe({
      name: sweepLogDirectory.name,
      children: [
        //region Age expiry

        it({
          name: 'expires old day directories wholesale and keeps newer day contents wholesale',
          fn: async () => {
            await using root = await fixture();
            // The expired day directory owns everything inside it, including a
            // file whose own name carries a newer day key.
            await writeEntry({
              directory: root.path,
              name: '2026-10-01/2026-10-01T01-00-00.a.log.jsonl',
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-01/2026-10-20T01-00-00.b.log.jsonl',
            },);
            // The kept day directory owns its contents just as wholesale, so a
            // file named with an older day key stays with its directory.
            await writeEntry({
              directory: root.path,
              name: '2026-10-20/2026-10-01T02-00-00.c.log.jsonl',
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: WIDE_POLICY,
              now: NOW,
            },);
            expect(await tree(root.path,),).toEqual([
              STAMP_NAME,
              '2026-10-20',
              '2026-10-20/2026-10-01T02-00-00.c.log.jsonl',
            ].toSorted(),);
          },
        },),

        it({
          name: 'expires root log and probe files whose day key predates the cutoff',
          fn: async () => {
            await using root = await fixture();
            await writeEntry({
              directory: root.path,
              name: '2026-10-01T14-00-00.x.log.jsonl',
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-01T14-00-00.x.probe.tmp',
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-22T14-00-00.y.log.jsonl',
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: WIDE_POLICY,
              now: NOW,
            },);
            expect(await tree(root.path,),).toEqual([
              STAMP_NAME,
              '2026-10-22T14-00-00.y.log.jsonl',
            ].toSorted(),);
          },
        },),

        //endregion Age expiry

        //region Foreign names survive every pass

        it({
          name: 'never deletes foreign root entries while an owned expired file proves the sweep ran',
          fn: async () => {
            await using root = await fixture();
            /**
             Foreign neighbors sharing the destination directory,
             all old enough that misclassification would delete them.
             */
            const foreign = [
              'translation-repair-runs-2026-10-01',
              'translation-repair-runs-2026-09-30/inner-notes.txt',
              'notes.txt',
              'arbitrary-dir/whatever.bin',
              '2026-1-22T14-00-01.x.log.jsonl',
              '2026-10-2234.log.jsonl',
              '2026-10-22x.log.jsonl',
              '2026--22-01.log.jsonl',
            ];
            for (const name of foreign) {
              // oxlint-disable-next-line no-await-in-loop -- sequential fixture writes keep setup readable
              await writeEntry({
                directory: root.path,
                name,
              },);
            }
            // A pre-existing stamp is overwritten, never treated as a victim.
            await writeEntry({
              directory: root.path,
              name: STAMP_NAME,
              content: 'stale stamp',
            },);
            // Positive control: an owned expired root file must actually go.
            await writeEntry({
              directory: root.path,
              name: '2026-10-01T00-00-00.z.log.jsonl',
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: {
                maxAgeDays: 14,
                // Tiny caps would trim any misclassified foreign candidate.
                maxBytes: 1,
                maxFiles: 1,
              },
              now: NOW,
            },);
            expect(await tree(root.path,),).toEqual([
              STAMP_NAME,
              // Foreign directories appear with their untouched contents.
              'arbitrary-dir',
              'translation-repair-runs-2026-09-30',
              ...foreign,
            ].toSorted(),);
          },
        },),

        it({
          name: 'excludes foreign files from budget measurement',
          fn: async () => {
            await using root = await fixture();
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T01-00-00.a.log.jsonl',
              content: 'x'.repeat(10,),
            },);
            // Foreign bytes inside the kept day directory stay uncounted.
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/notes.txt',
              content: 'x'.repeat(10_000,),
            },);
            await writeEntry({
              directory: root.path,
              name: 'translation-repair-runs-2026-10-22',
              content: 'x'.repeat(10_000,),
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: {
                maxAgeDays: 14,
                maxBytes: 100,
                maxFiles: 5,
              },
              now: NOW,
            },);
            expect(await tree(root.path,),).toEqual([
              STAMP_NAME,
              '2026-10-22',
              '2026-10-22/2026-10-22T01-00-00.a.log.jsonl',
              '2026-10-22/notes.txt',
              'translation-repair-runs-2026-10-22',
            ].toSorted(),);
          },
        },),

        //endregion Foreign names survive every pass

        //region Budget pass trims the oldest kept files

        it({
          name: 'deletes the oldest kept files until under the byte budget',
          fn: async () => {
            await using root = await fixture();
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T01-00-00.a.log.jsonl',
              content: 'x'.repeat(10,),
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T02-00-00.b.log.jsonl',
              content: 'x'.repeat(10,),
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T03-00-00.c.log.jsonl',
              content: 'x'.repeat(10,),
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: {
                maxAgeDays: 14,
                maxBytes: 15,
                maxFiles: 100,
              },
              now: NOW,
            },);
            expect(await tree(root.path,),).toEqual([
              STAMP_NAME,
              '2026-10-22',
              '2026-10-22/2026-10-22T03-00-00.c.log.jsonl',
            ].toSorted(),);
          },
        },),

        it({
          name: 'deletes the oldest kept files until under the file budget',
          fn: async () => {
            await using root = await fixture();
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T01-00-00.a.log.jsonl',
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T02-00-00.b.log.jsonl',
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T03-00-00.c.log.jsonl',
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: {
                maxAgeDays: 14,
                maxBytes: 536_870_912,
                maxFiles: 1,
              },
              now: NOW,
            },);
            expect(await tree(root.path,),).toEqual([
              STAMP_NAME,
              '2026-10-22',
              '2026-10-22/2026-10-22T03-00-00.c.log.jsonl',
            ].toSorted(),);
          },
        },),

        it({
          name: 'trims by day key first across kept day directories',
          fn: async () => {
            await using root = await fixture();
            await writeEntry({
              directory: root.path,
              name: '2026-10-20/2026-10-20T01-00-00.a.log.jsonl',
              content: 'x'.repeat(10,),
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T01-00-00.b.log.jsonl',
              content: 'x'.repeat(10,),
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: {
                maxAgeDays: 14,
                maxBytes: 15,
                maxFiles: 100,
              },
              now: NOW,
            },);
            // Deletion removes the aged file; the emptied day directory
            // itself stays, since only owned files are sweep victims.
            expect(await tree(root.path,),).toEqual([
              STAMP_NAME,
              '2026-10-20',
              '2026-10-22',
              '2026-10-22/2026-10-22T01-00-00.b.log.jsonl',
            ].toSorted(),);
          },
        },),

        //endregion Budget pass trims the oldest kept files

        //region Fresh files stay out of every deletion

        it({
          name: 'never deletes files modified within the fresh-file grace even over budget',
          fn: async () => {
            await using root = await fixture();
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T01-00-00.a.log.jsonl',
              modified: FRESH,
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T02-00-00.b.log.jsonl',
              modified: FRESH,
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: {
                maxAgeDays: 14,
                maxBytes: 1,
                maxFiles: 1,
              },
              now: NOW,
            },);
            expect(await tree(root.path,),).toEqual([
              STAMP_NAME,
              '2026-10-22',
              '2026-10-22/2026-10-22T01-00-00.a.log.jsonl',
              '2026-10-22/2026-10-22T02-00-00.b.log.jsonl',
            ].toSorted(),);
          },
        },),

        it({
          name: 'keeps fresh files over budget while deleting aged files behind them',
          fn: async () => {
            await using root = await fixture();
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T01-00-00.a.log.jsonl',
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T02-00-00.b.log.jsonl',
              modified: FRESH,
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T03-00-00.c.log.jsonl',
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: {
                maxAgeDays: 14,
                maxBytes: 1,
                maxFiles: 1,
              },
              now: NOW,
            },);
            // Selection skips the fresh file but keeps taking aged files
            // behind it until the caps hold.
            expect(await tree(root.path,),).toEqual([
              STAMP_NAME,
              '2026-10-22',
              '2026-10-22/2026-10-22T02-00-00.b.log.jsonl',
            ].toSorted(),);
          },
        },),

        it({
          name: 'stops selecting deletions once both caps hold',
          fn: async () => {
            await using root = await fixture();
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T01-00-00.a.log.jsonl',
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T02-00-00.b.log.jsonl',
              modified: FRESH,
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-22/2026-10-22T03-00-00.c.log.jsonl',
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: {
                maxAgeDays: 14,
                maxBytes: 100,
                maxFiles: 2,
              },
              now: NOW,
            },);
            // One oldest deletion satisfies both caps, so the aged file
            // behind the fresh one is left alone.
            expect(await tree(root.path,),).toEqual([
              STAMP_NAME,
              '2026-10-22',
              '2026-10-22/2026-10-22T02-00-00.b.log.jsonl',
              '2026-10-22/2026-10-22T03-00-00.c.log.jsonl',
            ].toSorted(),);
          },
        },),

        //endregion Fresh files stay out of every deletion

        //region Stamp throttles the walk

        it({
          name: 'leaves a stamp and walks at most once per default stamp interval',
          fn: async () => {
            await using root = await fixture();
            await writeEntry({
              directory: root.path,
              name: '2026-10-01T00-00-00.a.log.jsonl',
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: WIDE_POLICY,
              now: NOW,
            },);
            expect(await tree(root.path,),).toEqual([STAMP_NAME,],);
            expect(await readFile(join(
              root.path,
              STAMP_NAME,
            ), 'utf8',),).toBe(`${new Date(NOW,)
              .toISOString()}\n`,);
            // A second expired file arrives while the stamp is still fresh.
            await writeEntry({
              directory: root.path,
              name: '2026-10-01T00-00-00.b.log.jsonl',
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: WIDE_POLICY,
              now: NOW + (DEFAULT_STAMP_INTERVAL_MS - 1),
            },);
            expect((await tree(root.path,)).includes('2026-10-01T00-00-00.b.log.jsonl',),).toBe(true,);
            await sweepLogDirectory({
              directory: root.path,
              policy: WIDE_POLICY,
              now: NOW + DEFAULT_STAMP_INTERVAL_MS,
            },);
            expect(await tree(root.path,),).toEqual([STAMP_NAME,],);
            expect(await readFile(join(
              root.path,
              STAMP_NAME,
            ), 'utf8',),).toBe(`${new Date(NOW + DEFAULT_STAMP_INTERVAL_MS,)
              .toISOString()}\n`,);
          },
        },),

        it({
          name: 'honors an injected stampIntervalMs instead of the default window',
          fn: async () => {
            await using root = await fixture();
            await sweepLogDirectory({
              directory: root.path,
              policy: WIDE_POLICY,
              now: NOW,
              stampIntervalMs: 5,
            },);
            await writeEntry({
              directory: root.path,
              name: '2026-10-01T00-00-00.a.log.jsonl',
            },);
            await sweepLogDirectory({
              directory: root.path,
              policy: WIDE_POLICY,
              now: NOW + 4,
              stampIntervalMs: 5,
            },);
            expect((await tree(root.path,)).includes('2026-10-01T00-00-00.a.log.jsonl',),).toBe(true,);
            await sweepLogDirectory({
              directory: root.path,
              policy: WIDE_POLICY,
              now: NOW + 5,
              stampIntervalMs: 5,
            },);
            expect(await tree(root.path,),).toEqual([STAMP_NAME,],);
          },
        },),

        it({
          name: 'stamps an empty destination without walking into trouble',
          fn: async () => {
            await using root = await fixture();
            const { reports, reportError, } = reportCollector();
            await sweepLogDirectory({
              directory: root.path,
              policy: WIDE_POLICY,
              now: NOW,
              reportError,
            },);
            expect(await tree(root.path,),).toEqual([STAMP_NAME,],);
            expect(reports,).toEqual([],);
          },
        },),

        //endregion Stamp throttles the walk

        //region Faults report and never break the caller

        it({
          name: 'reports a stamp fault once and returns without walking',
          fn: async () => {
            await using root = await fixture();
            /**
             A regular file cannot hold the stamp, so the stamp write faults.
             */
            const occupied = join(
              root.path,
              'occupied',
            );
            await writeFile(occupied, 'occupied',);
            const { reports, reportError, } = reportCollector();
            await sweepLogDirectory({
              directory: occupied,
              policy: WIDE_POLICY,
              now: NOW,
              reportError,
            },);
            expect(reports,).toHaveLength(1,);
            expect(reports[0]?.context ?? '',)
              .toContain('log retention could not stamp',);
            expect(await tree(root.path,),).toEqual(['occupied',],);
          },
        },),

        //endregion Faults report and never break the caller
      ],
    },),
  ],
},);
