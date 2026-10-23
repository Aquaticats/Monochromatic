/**
 Tests for the report over a persisted rendering audit, and for both CLIs at
 their boundary.

 NO CASE REACHED THESE BEFORE. `readRunRows` decides what a report is a
 report of, `newestRun` decides which run is meant when none is named, and
 `printAcross` is the only across-run reading; each is exported through the
 barrel for exactly this. The two commands are then run as built, against
 throwaway runs and an empty archive, so the refusal policy rendering-7 set
 (a stated refusal exits 6 with its line and no frames) is proved at the
 boundary an operator meets rather than at the throw.

 DISPOSABLE FIXTURES ONLY: every run is written under its own `mkdtemp`
 directory, in the shape the probe store writes, and nothing here reads a
 real run.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { RUN_MODELS, } from '@monochromatic-dev/translation-repair-run';
import { readRunnerClosure, } from '@monochromatic-dev/translation-repair-insertion';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  digestPipeline,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary';
import {
  newestRun,
  printAcross,
  readRunRows,
  type SettledAuditRow,
} from '../../dist/final/node/index.mjs';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  ARCHIVE,
  OTHER_TEXTS,
  PROBE_NAME,
  ROSTER,
  rowFor,
  runOver,
  SAME_TEXTS,
  writeRun,
} from './settled-run-file.test-fixture.ts';
import {
  ENTRY_ID,
  makeCorpus,
  SOURCE_PAGE,
  TARGET_PAGE,
  writeArtifact,
} from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-archive.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Name of the built report command under test.
 */
const REPORT_COMMAND = 'rendering-audit-settled-report';

/**
 Name of the built audit command under test.
 */
const AUDIT_COMMAND = 'rendering-audit-settled';

/**
 Builds one audited slice typed as the report reads it.

 @param sliceIndex - slice index

 @param texts - what the audit was shown, omitted to leave it unrecorded

 @returns Row shaped as the probe persists it

 @example
 ```ts
 const row = auditRowFor({ sliceIndex: 0, texts: SAME_TEXTS, },);
 ```
 */
function auditRowFor(
  {
    sliceIndex,
    texts,
  }: {
    readonly sliceIndex: number;
    readonly texts?: Parameters<typeof rowFor>[0]['texts'];
  },
): SettledAuditRow {
  return rowFor({
    sliceIndex,
    ...((texts === undefined) ? {} : { texts, }),
  },) as unknown as SettledAuditRow;
}

/**
 What a built command wrote and how it exited.
 */
type CommandRun = {
  /**
   Exit code, or -1 when the process was signalled.
   */
  readonly code: number;

  /**
   Everything written to stdout.
   */
  readonly stdout: string;

  /**
   Everything written to stderr.
   */
  readonly stderr: string;
};

/**
 Runs a built command with every provider key withheld, so the child can
 neither refuse for the wrong reason nor spend.

 @param command - built command's name

 @param args - arguments after it

 @returns Exit code and both streams

 @example
 ```ts
 const run = await runBuilt({ command: REPORT_COMMAND, args: ['--run', path,], },);
 ```
 */
async function runBuilt(
  {
    command,
    args,
  }: {
    readonly command: string;
    readonly args: readonly string[];
  },
): Promise<CommandRun> {
  /**
   Throwaway runs directory the child points at, removed once this function's
   `await using` scope ends (after the child's streams close).
   */
  await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);

  return await runBuiltCommand({
    command,
    packageEntry: PROBE_ENTRY,
    args,
    env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
  },);
}

/**
 Directory of the built commands, which a run digests and reads its closure from.
 */
const BUILT_DIR = join(
  import.meta.dirname,
  '..',
  '..',
  'dist',
  'final',
  'node',
);

/**
 What the built audit did over an archive of one settled artifact, with the places it was pointed at.
 */
type AuditOverArchive = {
  /**
   Exit code, then both streams whole.
   */
  readonly run: CommandRun;

  /**
   Runs directory the child kept its run in.
   */
  readonly runsDir: string;

  /**
   Archive the child read, written for the throwaway corpus.
   */
  readonly archiveDir: string;

  /**
   Throwaway corpus clone the child read the artifact's commit from.
   */
  readonly cloneDir: string;
};

/**
 Runs the built audit against an archive holding one settled artifact and a throwaway corpus, with every
 provider key withheld by the fixture and every location the command reads named here.

 @param typed - flags after the archive and the clone

 @param check - what the case reads while the throwaway directories still exist

 @returns Nothing; the check holds the assertions

 @example
 ```ts
 await auditOverArchive({ typed: ['--cap', '0',], check: async function read({ run, },) { expect(run.code,).toBe(0,); }, },);
 ```
 */
async function auditOverArchive(
  {
    typed,
    check,
  }: {
    readonly typed: readonly string[];
    readonly check: (seen: AuditOverArchive,) => Promise<void>;
  },
): Promise<void> {
  await using corpus = await makeCorpus();
  await using archive = await scratchDir({ prefix: 'rendering-audit-settled-archive-', },);
  await using runs = await scratchDir({ prefix: 'rendering-audit-settled-runs-', },);
  await writeArtifact({
    archiveDir: archive.path,
    runSet: 'first-run',
    prepared: prepareDocumentPair({
      sourceText: SOURCE_PAGE,
      targetText: TARGET_PAGE,
    },),
    corpusSha: corpus.commitSha,
    entryId: ENTRY_ID,
  },);
  await check({
    run: await runBuiltCommand({
      command: AUDIT_COMMAND,
      packageEntry: PROBE_ENTRY,
      args: [
        '--archive',
        archive.path,
        '--clone',
        corpus.cloneDir,
        ...typed,
      ],
      env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
    },),
    runsDir: runs.path,
    archiveDir: archive.path,
    cloneDir: corpus.cloneDir,
  },);
}

/**
 The line the population report prints for the one artifact every archive of these cases holds.
 */
const POPULATION_LINE = 'first-run/mittens.json  subjects=3 retained=2 replaced=1 displaced=0 undecided=3 verification=verified';

/**
 Paths of the two runs a report case leaves in its runs directory.
 */
type TwoRuns = {
  /**
   Runs directory both live in, which the child is pointed at.
   */
  readonly runsDir: string;

  /**
   The older run, which bought one row.
   */
  readonly older: string;

  /**
   The newer run, which bought two rows.
   */
  readonly newer: string;
};

/**
 Writes an older run of one row and a newer run of two rows, runs the built report against them, and hands
 the result to the case while the throwaway directory still exists.

 @param typed - flags the report is run with, given where the two runs are

 @param check - what the case reads of the finished child

 @returns Nothing; the check holds the assertions

 @example
 ```ts
 await reportOverTwoRuns({ typed: function none() { return []; }, check: async function read({ run, },) { expect(run.code,).toBe(0,); }, },);
 ```
 */
async function reportOverTwoRuns(
  {
    typed,
    check,
  }: {
    readonly typed: (runs: TwoRuns,) => readonly string[];
    readonly check: (seen: { readonly run: CommandRun; readonly runs: TwoRuns; },) => Promise<void>;
  },
): Promise<void> {
  await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
  /**
   Where the two runs are.
   */
  const runs: TwoRuns = {
    runsDir: scratch.path,
    older: await writeRun({
      runsDir: scratch.path,
      stamp: '2026-08-25T01-00-00.000Z',
      body: runOver({
        rows: [auditRowFor({
          sliceIndex: 0,
          texts: SAME_TEXTS,
        },),],
        roster: ROSTER,
      },),
    },),
    newer: await writeRun({
      runsDir: scratch.path,
      stamp: '2026-08-26T01-00-00.000Z',
      body: runOver({
        rows: [
          auditRowFor({
            sliceIndex: 0,
            texts: OTHER_TEXTS,
          },),
          auditRowFor({
            sliceIndex: 1,
            texts: SAME_TEXTS,
          },),
        ],
        roster: ROSTER,
      },),
    },),
  };
  await check({
    run: await runBuiltCommand({
      command: REPORT_COMMAND,
      packageEntry: PROBE_ENTRY,
      args: typed(runs,),
      env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
    },),
    runs,
  },);
}

/**
 What the built report prints over the newer run, which bought two fresh rows nobody audited twice, up to
 and including the band line, then the closing lines.

 @param path - run the report read, as it prints it

 @param between - lines between the band and the closing lines, none unless the report was asked to compare

 @returns Whole stdout

 @example
 ```ts
 expect(run.stdout,).toBe(newerRunReport({ path: runs.newer, between: '', },),);
 ```
 */
function newerRunReport(
  {
    path,
    between,
  }: {
    readonly path: string;
    readonly between: string;
  },
): string {
  return `${path}\n2 subjects\n\n`
    + 'THE TWO HALVES, READ APART\n'
    + '  ARCHIVE text  subjects=0  drew a claim=0  claims=0  corroborated=0  agreed=0  near=0  degraded=0\n'
    + '  FRESH   text  subjects=2  drew a claim=0  claims=0  corroborated=0  agreed=0  near=0  degraded=0\n\n'
    + 'WHAT A DOCUMENT WOULD CARRY AT THE SAME SLICES\n'
    + '  survives                                    subjects=2  claims=0\n'
    + '  A displaced subject was audited on wording no reader of a document would meet. '
    + 'An undecided one is waiting on a decision, not overruled.\n\n'
    + 'WHAT EACH AUDITOR THOUGHT WAS WORTH A CLAIM\n'
    + '  hf:cat/Tabby-1                                   asked=2 answered=2 lost=0 spoke on=0 claims=0 dropped=0\n'
    + '  hf:cat/Mouser-1                                  asked=2 answered=0 lost=2 spoke on=0 claims=0 dropped=0\n\n'
    + 'RELOCATION CANDIDATES: claim pairs=0 slice pairs=0\n\n'
    + 'INSTRUMENT BAND over texts this run audited twice\n'
    + '  NOTHING PAIRED. Either no text was audited twice, or the rows predate the recorded text identity '
    + 'that pairing needs. No band is quotable from this run, so no comparison in it resolves anything.\n'
    + `${between}\nTWO ENTRIES. Nothing here settles anything about a particular entry, and nothing here may gate `
    + 'what ships: the instrument\'s own error rate is unmeasured.\n'
    + `Archive that run read: ${ARCHIVE}\n`;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readRunRows.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the rows, the archive the run named and the roster it asked',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            /**
             One complete run.
             */
            const path = await writeRun({
              runsDir: scratch.path,
              stamp: '2026-08-25T01-00-00.000Z',
              body: runOver({
                rows: [auditRowFor({ sliceIndex: 0, },),],
                roster: ROSTER,
              },),
            },);

            /**
             What the report reads off it.
             */
            const read = await readRunRows({ path, },);

            expect(read.rows.length,).toBe(1,);
            expect(read.archiveDir,).toBe(ARCHIVE,);
            expect(read.roster,).toEqual(ROSTER,);
          },
        },),

        it({
          name: 'READS A RUN WRITTEN BEFORE THE ROSTER WAS KEPT as an empty roster, so its other '
            + 'readings still answer and the voice rates say only what the rows say',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            /**
             One run carrying no roster field.
             */
            const path = await writeRun({
              runsDir: scratch.path,
              stamp: '2026-08-25T01-00-00.000Z',
              body: runOver({ rows: [auditRowFor({ sliceIndex: 0, },),], },),
            },);

            expect((await readRunRows({ path, },)).roster,).toEqual([],);
          },
        },),

        it({
          name: 'REFUSES A FILE CARRYING NO ROWS ARRAY as a stated refusal, since it is not a run of '
            + 'this probe rather than a quiet one, and the remedy is the operator\'s',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            /**
             A file with the run's identity and nothing bought.
             */
            const path = await writeRun({
              runsDir: scratch.path,
              stamp: '2026-08-25T01-00-00.000Z',
              body: {
                startedAt: '2026-08-25T01:00:00.000Z',
                subject: { archiveDir: ARCHIVE, },
              },
            },);

            await expect(readRunRows({ path, },),).rejects.toThrow(StatedRefusalError,);
          },
        },),

        it({
          name: 'REFUSES A ROW THAT IS NOT AN OBJECT as a stated refusal naming its position and never its content, '
            + 'since every reading after this one reads fields off every row',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            /**
             A run whose second row is a bare number.
             */
            const path = await writeRun({
              runsDir: scratch.path,
              stamp: '2026-08-25T01-00-00.000Z',
              body: runOver({
                rows: [
                  rowFor({ sliceIndex: 0, },),
                  7,
                ],
                roster: ROSTER,
              },),
            },);

            /**
             What the reader said about the file.
             */
            const refusal = await rejectionOf(async function readBadRows() {
              return await readRunRows({ path, },);
            },);
            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              `StatedRefusalError: ${path} carries a row that is not an object, at position 1`,
            );
          },
        },),
      ],
    },),

    describe({
      name: newestRun.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PICKS THE NEWEST RUN BY NAME, since names sort by the instant they carry and no file '
            + 'has to be opened',
          fn: async () => {
            /**
             Two runs, a day apart.
             */
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            const runsDir = scratch.path;
            await writeRun({
              runsDir,
              stamp: '2026-08-25T01-00-00.000Z',
              body: runOver({ rows: [], },),
            },);
            /**
             The later one, which the report should read.
             */
            const later = await writeRun({
              runsDir,
              stamp: '2026-08-26T01-00-00.000Z',
              body: runOver({ rows: [], },),
            },);

            expect(await newestRun({ runsDir, },),).toBe(later,);
          },
        },),

        it({
          name: 'PICKS ONLY A FILE THE STORE FINISHED: a directory named like a later run, and a run still being '
            + 'written under its `.partial` name, are no run (ledger B65)',
          fn: async () => {
            /**
             Runs directory holding one finished run.
             */
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            const runsDir = scratch.path;

            /**
             The one run the store finished, which the report should read.
             */
            const finished = await writeRun({
              runsDir,
              stamp: '2026-08-25T01-00-00.000Z',
              body: runOver({ rows: [], },),
            },);
            await mkdir(join(
              runsDir,
              PROBE_NAME,
              '2026-08-26T01-00-00.000Z-cafef00d.json',
            ),);
            await writeFile(
              join(
                runsDir,
                PROBE_NAME,
                '2026-08-27T01-00-00.000Z-cafef00d.json.4242.partial',
              ),
              '{"rows":',
              'utf8',
            );

            expect(await newestRun({ runsDir, },),).toBe(finished,);
          },
        },),

        it({
          name: 'REFUSES A PROBE THAT HAS NEVER RUN as a stated refusal, since reporting nothing '
            + 'would look exactly like reporting a clean run',
          fn: async () => {
            /**
             A probe directory with no run in it.
             */
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            const runsDir = scratch.path;
            await mkdir(
              join(
                runsDir,
                PROBE_NAME,
              ),
              { recursive: true, },
            );

            await expect(newestRun({ runsDir, },),).rejects.toThrow(StatedRefusalError,);
          },
        },),
      ],
    },),

    describe({
      name: printAcross.name,
      children: [
        it({
          name: 'PRINTS A BAND over the subjects both runs bought on identical text, and no sentence '
            + 'about moved or unverifiable slots',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

            /**
             Earlier run over the same text.
             */
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            const against = await writeRun({
              runsDir: scratch.path,
              stamp: '2026-08-25T01-00-00.000Z',
              body: runOver({
                rows: [auditRowFor({
                  sliceIndex: 0,
                  texts: SAME_TEXTS,
                },),],
              },),
            },);

            await printAcross({
              rows: [auditRowFor({
                sliceIndex: 0,
                texts: SAME_TEXTS,
              },),],
              against,
            },);

            /**
             Everything printed, as one body to search.
             */
            const said = printed.lines.join('\n',);

            expect(said.includes('NOTHING PAIRED',),).toBe(false,);
            expect(said.includes('DISAGREES',),).toBe(false,);
            expect(said.includes('cannot be checked',),).toBe(false,);
          },
        },),

        it({
          name: 'COUNTS ONE UNVERIFIABLE SLOT in the singular and TWO in the plural, since the phrase opens a '
            + 'sentence a reader follows',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

            /**
             Earlier run that recorded the slot's text.
             */
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            const against = await writeRun({
              runsDir: scratch.path,
              stamp: '2026-08-25T01-00-00.000Z',
              body: runOver({
                rows: [auditRowFor({
                  sliceIndex: 0,
                  texts: SAME_TEXTS,
                },),],
              },),
            },);

            await printAcross({
              rows: [auditRowFor({
                sliceIndex: 0,
              },),],
              against,
            },);

            /**
             Everything printed, as one body to search.
             */
            const said = printed.lines.join('\n',);
            expect(said.includes('One slot that cannot be checked',),).toBe(true,);

            /**
             The same read with two slots unrecorded on one side.
             */
            const two = await writeRun({
              runsDir: scratch.path,
              stamp: '2026-08-25T02-00-00.000Z',
              body: runOver({
                rows: [auditRowFor({
                  sliceIndex: 0,
                  texts: SAME_TEXTS,
                },), auditRowFor({
                  sliceIndex: 1,
                  texts: SAME_TEXTS,
                },),],
              },),
            },);
            await printAcross({
              rows: [auditRowFor({
                sliceIndex: 0,
              },), auditRowFor({
                sliceIndex: 1,
              },),],
              against: two,
            },);
            /**
             Everything both reads printed.
             */
            const both = printed.lines.join('\n',);
            expect(both.includes('2 slots that cannot be checked',),).toBe(true,);
          },
        },),

        it({
          name: 'SAYS THE TEXT DISAGREES and leaves the slot out where both runs recorded it and the '
            + 'archive moved between them',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

            /**
             Earlier run over a different rendering of the slot.
             */
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            const against = await writeRun({
              runsDir: scratch.path,
              stamp: '2026-08-25T01-00-00.000Z',
              body: runOver({
                rows: [auditRowFor({
                  sliceIndex: 0,
                  texts: OTHER_TEXTS,
                },),],
              },),
            },);

            await printAcross({
              rows: [auditRowFor({
                sliceIndex: 0,
                texts: SAME_TEXTS,
              },),],
              against,
            },);

            expect(printed.lines.join('\n',)
              .includes('the text DISAGREES',),).toBe(true,);
          },
        },),

        it({
          name: 'SAYS A SLOT CANNOT BE CHECKED where one run recorded no text identity, which is a '
            + 'fact about the run and nothing about the archive',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

            /**
             Earlier run written before identities were recorded.
             */
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            const against = await writeRun({
              runsDir: scratch.path,
              stamp: '2026-08-25T01-00-00.000Z',
              body: runOver({ rows: [auditRowFor({ sliceIndex: 0, },),], },),
            },);

            await printAcross({
              rows: [auditRowFor({
                sliceIndex: 0,
                texts: SAME_TEXTS,
              },),],
              against,
            },);

            expect(printed.lines.join('\n',)
              .includes('cannot be checked',),).toBe(true,);
          },
        },),
      ],
      concurrency: 1,
    },),

    describe({
      name: 'rendering-audit-settled-report as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS A NAMED RUN and exits 0, printing both halves and the archive that run read',
          fn: async () => {
            /**
             One complete run to report.
             */
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            const path = await writeRun({
              runsDir: scratch.path,
              stamp: '2026-08-25T01-00-00.000Z',
              body: runOver({
                rows: [auditRowFor({
                  sliceIndex: 0,
                  texts: SAME_TEXTS,
                },),],
                roster: ROSTER,
              },),
            },);

            /**
             What the command wrote.
             */
            const run = await runBuilt({
              command: REPORT_COMMAND,
              args: [
                '--run',
                path,
              ],
            },);

            expect(run.code,).toBe(0,);
            expect(run.stdout.includes('THE TWO HALVES, READ APART',),).toBe(true,);
            expect(run.stdout.includes(`Archive that run read: ${ARCHIVE}`,),).toBe(true,);
            expect(run.stdout.includes('asked=1 answered=1 lost=0',),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES A FILE THAT IS NOT A RUN with its line and exit 6, no frames, which is the '
            + 'policy rendering-7 set for the four operator refusals',
          fn: async () => {
            /**
             A file with no rows.
             */
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-report-', },);
            const path = await writeRun({
              runsDir: scratch.path,
              stamp: '2026-08-25T01-00-00.000Z',
              body: {
                startedAt: '2026-08-25T01:00:00.000Z',
                subject: { archiveDir: ARCHIVE, },
              },
            },);

            /**
             What the command wrote.
             */
            const run = await runBuilt({
              command: REPORT_COMMAND,
              args: [
                '--run',
                path,
              ],
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stderr.includes('carries no rows array',),).toBe(true,);
            expect(run.stderr.includes('    at ',),).toBe(false,);
          },
        },),

        it({
          name: 'REPORTS THE NEWEST RUN when none is named, since the store keeps every run and a reader that '
            + 'merged them would undo that',
          fn: async () => {
            await reportOverTwoRuns({
              typed: function none(): readonly string[] {
                return [];
              },
              check: async function readReport({
                run,
                runs,
              },): Promise<void> {
                expect(run.code,).toBe(0,);
                expect(run.stderr,).toBe('',);
                expect(run.stdout,).toBe(newerRunReport({
                  path: runs.newer,
                  between: '',
                },),);
              },
            },);
          },
        },),

        it({
          name: 'PAIRS TWO RUNS when asked to compare, printing the across-run band and what it left out, '
            + 'beneath the run it reported',
          fn: async () => {
            await reportOverTwoRuns({
              typed: function comparing({
                older,
                newer,
              },): readonly string[] {
                return [
                  '--run',
                  newer,
                  '--against',
                  older,
                ];
              },
              check: async function readReport({
                run,
                runs,
              },): Promise<void> {
                expect(run.code,).toBe(0,);
                expect(run.stderr,).toBe('',);
                expect(run.stdout,).toBe(newerRunReport({
                  path: runs.newer,
                  between: `\nINSTRUMENT BAND over the same subjects in ${runs.older}\n`
                    + '  NOTHING PAIRED. Either no text was audited twice, or the rows predate the recorded text '
                    + 'identity that pairing needs. No band is quotable from this run, so no comparison in it '
                    + 'resolves anything.\n'
                    + '  One slot recorded by BOTH runs and the text DISAGREES, so the archive moved between '
                    + 'them and these are left out: naptime-20260825/mittens#0\n',
                },),);
              },
            },);
          },
        },),

        it({
          name: 'REFUSES A RUN FLAG WRITTEN WITHOUT A VALUE with its line and exit 6, and prints nothing on stdout, '
            + 'rather than reporting the newest run in silence',
          fn: async () => {
            await reportOverTwoRuns({
              typed: function valueless(): readonly string[] {
                return ['--run',];
              },
              check: async function readRefusal({ run, },): Promise<void> {
                expect(run.code,).toBe(REFUSED_AS_STATED,);
                expect(run.stdout,).toBe('',);
                expect(run.stderr,).toBe(
                  'rendering-audit-settled-report: --run needs a value written after it. Usage: '
                    + 'rendering-audit-settled-report [--run <run file>] [--against <run file>]\n',
                );
              },
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'rendering-audit-settled as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES AN EMPTY ARCHIVE with its line and exit 6 before any roster is woken, since '
            + 'the run was pointed somewhere wrong rather than at a clean archive',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'rendering-audit-settled-empty-', },);
            /**
             What the command wrote against an archive holding nothing.
             */
            const run = await runBuilt({
              command: AUDIT_COMMAND,
              args: [
                '--archive',
                scratch.path,
                '--cap',
                '0',
              ],
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stderr.includes('no artifacts under',),).toBe(true,);
            expect(run.stderr.includes('    at ',),).toBe(false,);
          },
        },),

        it({
          name: 'RUNS TO ITS END AT A ZERO CAP over an archive of one settled artifact: exit 0, the population, '
            + 'the buying line and where the run was kept, and a kept run naming what was read and holding no row',
          fn: async () => {
            await auditOverArchive({
              typed: [
                '--cap',
                '0',
              ],
              check: async function readKept({
                run,
                runsDir,
                archiveDir,
                cloneDir,
              },): Promise<void> {
                /**
                 Files the run left in its probe's directory.
                 */
                const kept = await readdir(join(
                  runsDir,
                  PROBE_NAME,
                ),);
                expect(kept.length,).toBe(1,);
                /**
                 Name of the one kept run, which carries the start and the build's tail.
                 */
                const keptName = nonNullishOrThrow(kept[0],);
                expect(run.code,).toBe(0,);
                expect(run.stderr,).toBe('',);
                expect(run.stdout,).toBe(
                  `${POPULATION_LINE}\n\nBUYING 0 of 3 selectable subjects\n\n\nkept at ${
                    join(
                      runsDir,
                      PROBE_NAME,
                      keptName,
                    )
                  }\n`,
                );
                /**
                 The kept run as written.
                 */
                const written = JSON.parse(await readFile(
                  join(
                    runsDir,
                    PROBE_NAME,
                    keptName,
                  ),
                  'utf8',
                ),) as Record<string, unknown>;
                /**
                 Build the child ran under, digested the way the child digests it.
                 */
                const { digest, } = await digestPipeline({ dir: BUILT_DIR, },);
                expect(written,).toEqual({
                  startedAt: written.startedAt,
                  finishedAt: written.finishedAt,
                  pipelineDigest: digest,
                  runnerClosure: await readRunnerClosure({
                    entryPath: join(
                      BUILT_DIR,
                      `${AUDIT_COMMAND}.mjs`,
                    ),
                  },),
                  roster: RUN_MODELS.checkerModelIds,
                  subject: {
                    archiveDir,
                    cloneDir,
                    cap: 0,
                    onlyIds: [],
                    artifacts: [
                      {
                        runSet: 'first-run',
                        artifactFile: 'mittens.json',
                        entryId: 'mittens',
                        artifactDigest: `sha256-tree-v1:${'c'.repeat(64,)}`,
                        subjects: 3,
                        verification: 'verified',
                      },
                    ],
                  },
                  rows: [],
                },);
                expect(keptName,).toBe(`${String(written.startedAt,)
                  .split(':',)
                  .join('-',)}-${digest.slice(-8,)}.json`,);
              },
            },);
          },
        },),

        it({
          name: 'REFUSES AN ENTRY THE ARCHIVE DOES NOT HOLD after the population it read, with its line and exit 6, '
            + 'and keeps nothing',
          fn: async () => {
            await auditOverArchive({
              typed: [
                '--only',
                'nobody',
              ],
              check: async function readRefusal({ run, runsDir, },): Promise<void> {
                expect(run.code,).toBe(REFUSED_AS_STATED,);
                expect(run.stdout,).toBe(`${POPULATION_LINE}\n`,);
                expect(run.stderr,).toBe(
                  'rendering-audit-settled: --only asks for "nobody", which the settled archive does not hold\n',
                );
                expect(await readdir(runsDir,),).toEqual([],);
              },
            },);
          },
        },),

        it({
          name: 'REFUSES TO BUY WITHOUT A KEY after naming what it would buy, with its line and exit 6, asking '
            + 'no model and keeping nothing',
          fn: async () => {
            await auditOverArchive({
              typed: [
                '--cap',
                '1',
              ],
              check: async function readRefusal({ run, runsDir, },): Promise<void> {
                expect(run.code,).toBe(REFUSED_AS_STATED,);
                expect(run.stdout,).toBe(`${POPULATION_LINE}\n\nBUYING 1 of 3 selectable subjects\n\n`,);
                expect(run.stderr,).toBe(
                  'rendering-audit-settled: TRANSLATION_REPAIR_NEURALWATT_API_KEY, TRANSLATION_REPAIR_SYNTHETIC_API_KEY, '
                    + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY, TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY, '
                    + 'DEEPSEEK_API_KEY, TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY, TRANSLATION_REPAIR_CLAUDE_API_KEY or '
                    + 'TRANSLATION_REPAIR_OPENROUTER_API_KEY is not set; run under mise so sops injects it\n',
                );
                expect(await readdir(runsDir,),).toEqual([],);
              },
            },);
          },
        },),

        it({
          name: 'REFUSES A CAP THAT IS NOT A WHOLE NUMBER before it reads the archive, with its line and exit 6, '
            + 'and prints nothing on stdout',
          fn: async () => {
            await auditOverArchive({
              typed: [
                '--cap',
                'kitten',
              ],
              check: async function readRefusal({ run, runsDir, },): Promise<void> {
                expect(run.code,).toBe(REFUSED_AS_STATED,);
                expect(run.stdout,).toBe('',);
                expect(run.stderr,).toBe(
                  'rendering-audit-settled: --cap needs a whole number written in digits, at most '
                    + '9007199254740991, and "kitten" is not one\n',
                );
                expect(await readdir(runsDir,),).toEqual([],);
              },
            },);
          },
        },),
      ],
    },),
  ],
},);
