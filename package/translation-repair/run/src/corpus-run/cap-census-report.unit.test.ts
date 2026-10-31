/**
 Tests for the cap census's report and for the built command at its boundary.

 The built command is run as a child process whose environment carries no
 provider key, over logs the cases write into a scratch directory, so nothing
 here reads a real run log and nothing can spend.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  chmod,
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  COMPLETION_CAP,
  MODEL_CARDS,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { reportCapCensus, } from '../../dist/final/node/index.mjs';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { RUN_ENTRY, } from './built-entry.test-fixture.ts';
import { SEAT_HYPER_OPENROUTER_UNMEASURED, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { writeLog as writeRecordLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/report-log-write.test-fixture.ts';
import {
  CAP_CENSUS_CLOSING_NOTE,
  spendLine,
  streamLine,
} from '@monochromatic-dev/translation-repair-provider-stream/ts/corpus-run/cap-census-log.test-fixture.ts';
import { lineOf, } from './command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Seat the fixture calls went to.
 */
const SEAT = SEAT_HYPER_OPENROUTER_UNMEASURED;

/**
 The seat's Hyper id, as its lines name it.
 */
const HYPER_ID = MODEL_CARDS[SEAT].hyper?.id ?? 'no hyper block';

/**
 The START line of a real pass: a commit id a repository could give, the digest of the pipeline it ran and the figures
 a pass prints (`corpus-pass-lines.ts`).
 */
const PASS_START = 'START tip=2a336603bf5d1f0a8c7e94b1d62e0a3f77c9d815 '
  + 'pipeline=sha256-tree-v1:5d4265e1c0a37b8f9d2e6a4c1b07f3e58d9a2c4b6e1f0a7d3c5b8e92f4a6d1c0 files=156 pending=2 '
  + 'done=0 soft=259200000ms hard=25200000ms hard-scope=per-attempt call=360000ms claude-call=460000ms '
  + 'at=2026-09-28T09:59:00.000Z';

/**
 The START line a test suite prints from a fixture pass, whose commit id is one digit repeated forty times
 (ledger B963).
 */
const FIXTURE_PASS_START = `START tip=${'f'.repeat(40,)} `
  + 'pipeline=sha256-tree-v1:9ae510cf8a857e770845eabe23f4b2b24d4a2b36a317fbc6cec8d995a7fb834f files=1 pending=1 '
  + 'done=0 soft=259200000ms hard=25200000ms hard-scope=per-attempt call=360000ms claude-call=460000ms '
  + 'at=2026-09-28T09:59:00.000Z';

/**
 Lines of a pass-run log holding two completed Hyper calls of the seat, one
 far under its cap and one at it, whose stream delivered content.

 @returns Log's lines, opened by the marker a pass run opens with

 @example
 ```ts
 await writeFile(path, `${passRunLines().join('\n',)}\n`,);
 ```
 */
function passRunLines(): readonly string[] {
  return [
    PASS_START,
    streamLine({ stamp: '2026-09-28T10:00:00.000Z', label: HYPER_ID, outcome: 'completed', content: 7, },),
    spendLine({
      stamp: '2026-09-28T10:00:00.020Z',
      tail: `provider=hyper model=${HYPER_ID} prompt=10 completion=100`,
    },),
    streamLine({ stamp: '2026-09-28T10:01:00.000Z', label: HYPER_ID, outcome: 'completed', content: 9, },),
    spendLine({
      stamp: '2026-09-28T10:01:00.020Z',
      tail: `provider=hyper model=${HYPER_ID} prompt=10 completion=${String(COMPLETION_CAP[SEAT],)}`,
    },),
  ];
}

/**
 Writes a named record file of the given lines into a scratch directory.

 A line that opens `START tip=` is written as the record a pass makes of its START line, the way the console prints it
 (`[info] [stamp] START ...`); a line in the console's form is written as the record the logger writes for it, and any
 other line as a foreign line.

 @param directory - scratch directory

 @param name - file name, which gains `.jsonl` where it ends `.log`

 @param lines - log's lines

 @example
 ```ts
 await writeLog({ directory: scratch.path, name: 'nap.log', lines: [PASS_START,], },);
 ```
 */
async function writeLog(
  {
    directory,
    name,
    lines,
  }: {
    readonly directory: string;
    readonly name: string;
    readonly lines: readonly string[];
  },
): Promise<void> {
  await writeRecordLog({
    dir: directory,
    name,
    lines: lines.map(function asRecordLine(line,): string {
      return line.startsWith('START tip=',) ? `[info] [2026-09-28T09:59:00.000Z] ${line}` : line;
    },),
  },);
}

await describe({
  name: 'cap-census report',
  concurrency: 1,
  children: [
    describe({
      name: reportCapCensus.name,
      children: [
        it({
          name: 'PRINTS THE CENSUS OF EVERY PASS-RUN LOG under a named directory, the calls of all of them together, '
            + 'and leaves a log with no run marker out of the count of calls',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'cap-census-report-', },);
            await writeLog({
              directory: scratch.path,
              name: 'first.log',
              lines: passRunLines(),
            },);
            await writeLog({
              directory: scratch.path,
              name: 'second.log',
              lines: [
                PASS_START,
                streamLine({ stamp: '2026-09-29T10:00:00.000Z', label: HYPER_ID, outcome: 'completed', content: 3, },),
                spendLine({
                  stamp: '2026-09-29T10:00:00.020Z',
                  tail: `provider=hyper model=${HYPER_ID} prompt=10 completion=50`,
                },),
                streamLine({ stamp: '2026-09-29T10:01:00.000Z', label: HYPER_ID, outcome: 'completed', content: 4, },),
                spendLine({
                  stamp: '2026-09-29T10:01:00.020Z',
                  tail: `provider=hyper model=${HYPER_ID} prompt=10 completion=60`,
                },),
              ],
            },);
            await writeLog({
              directory: scratch.path,
              name: 'suite.log',
              lines: [passRunLines()[1] ?? '',],
            },);

            await reportCapCensus({ line: lineOf({ command: 'cap-census', typed: [scratch.path,], },), },);

            expect(printed.lines,).toEqual([
              'cap-census: 3 logs, 2 pass-run logs, 4 completed calls, 0 on ids no card names, 0 paths unreadable, '
                + '0 dangling links with no log name passed over; '
                + 'lines left out for a stamp the logger did not write: 0',
              `${SEAT}: card cap ${String(COMPLETION_CAP[SEAT],)}, rule reads nothing (too few calls)`,
              `  hyper: 4 calls, p99 ${String(COMPLETION_CAP[SEAT],)}; since the caps 4 calls, 1 at the cap (25.00%): `
                + '1 with content, 0 with none, 0 unpaired',
              CAP_CENSUS_CLOSING_NOTE,
            ],);
          },
        },),
        it({
          name: 'COUNTS A LOG IT CANNOT READ as an unreadable path and says so on stderr, rather than stopping the '
            + 'census of the logs it can',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            const stderr = ctx.sinon.stub(
              console,
              'error',
            );
            await using scratch = await scratchDir({ prefix: 'cap-census-report-', },);
            await writeLog({
              directory: scratch.path,
              name: 'first.log',
              lines: passRunLines(),
            },);
            await writeLog({
              directory: scratch.path,
              name: 'sealed.log',
              lines: passRunLines(),
            },);

            /**
             The log nobody may read.
             */
            const sealed = join(
              scratch.path,
              'sealed.log.jsonl',
            );
            await chmod(
              sealed,
              0,
            );
            await using _reopened = {
              [Symbol.asyncDispose]: async function reopen(): Promise<void> {
                await chmod(
                  sealed,
                  0o600,
                );
              },
            };

            await reportCapCensus({ line: lineOf({ command: 'cap-census', typed: [scratch.path,], },), },);

            expect({
              header: printed.lines[0],
              stderr: stderr.args,
            },).toEqual({
              header: 'cap-census: 2 logs, 1 pass-run log, 2 completed calls, 0 on ids no card names, 1 path unreadable, '
                + '0 dangling links with no log name passed over; '
                + 'lines left out for a stamp the logger did not write: 0',
              stderr: [[
                `cap-census: cannot read ${sealed}: Error: EACCES: permission denied, open '${sealed}'`,
              ],],
            },);
          },
        },),
        it({
          name: 'COUNTS A DANGLING LINK WITH NO LOG NAME in the summary, apart from the unreadable paths, and '
            + 'says nothing of it on stderr (ledger B991)',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            const stderr = ctx.sinon.stub(
              console,
              'error',
            );
            await using scratch = await scratchDir({ prefix: 'cap-census-report-', },);
            await writeLog({
              directory: scratch.path,
              name: 'first.log',
              lines: passRunLines(),
            },);
            await symlink(
              join(
                scratch.path,
                'no-such-nest',
              ),
              join(
                scratch.path,
                'SingletonLock',
              ),
            );

            await reportCapCensus({ line: lineOf({ command: 'cap-census', typed: [scratch.path,], },), },);

            expect({
              header: printed.lines[0],
              stderr: stderr.args,
            },).toEqual({
              header: 'cap-census: 1 log, 1 pass-run log, 2 completed calls, 0 on ids no card names, 0 paths unreadable, '
                + '1 dangling link with no log name passed over; '
                + 'lines left out for a stamp the logger did not write: 0',
              stderr: [],
            },);
          },
        },),
        it({
          name: 'COUNTS THE LINES A LOG LEFT OUT for a stamp the logger did not write, and prints a lone log and a '
            + 'lone call in the singular',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'cap-census-report-', },);
            await writeLog({
              directory: scratch.path,
              name: 'only.log',
              lines: [
                PASS_START,
                ...passRunLines().slice(1, 3),
                `SPEND provider=hyper model=${HYPER_ID} prompt=10 completion=13`,
              ],
            },);

            await reportCapCensus({ line: lineOf({ command: 'cap-census', typed: [scratch.path,], },), },);

            expect(printed.lines[0],).toBe(
              'cap-census: 1 log, 1 pass-run log, 1 completed call, 0 on ids no card names, 0 paths unreadable, '
                + '0 dangling links with no log name passed over; '
                + 'lines left out for a stamp the logger did not write: 1',
            );
          },
        },),
        it({
          name: 'LEAVES A TEST-SUITE LOG OUT OF THE PASS-RUN COUNT where it printed the START line a fixture pass '
            + 'prints, since the count of pass-run logs is a figure the report states (ledger B963)',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'cap-census-report-', },);
            await writeLog({
              directory: scratch.path,
              name: 'suite.log',
              lines: [
                FIXTURE_PASS_START,
                ...passRunLines().slice(1,),
              ],
            },);

            await reportCapCensus({ line: lineOf({ command: 'cap-census', typed: [scratch.path,], },), },);

            expect(printed.lines[0],).toBe(
              'cap-census: 1 log, 0 pass-run logs, 0 completed calls, 0 on ids no card names, 0 paths unreadable, '
                + '0 dangling links with no log name passed over; '
                + 'lines left out for a stamp the logger did not write: 0',
            );
          },
        },),
        it({
          name: 'PRINTS NO SEAT ROW where no log is a pass run, and counts the logs it passed over',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'cap-census-report-', },);
            await writeLog({
              directory: scratch.path,
              name: 'suite.log',
              lines: ['a suite log',],
            },);

            await reportCapCensus({ line: lineOf({ command: 'cap-census', typed: [scratch.path,], },), },);

            expect(printed.lines,).toEqual([
              'cap-census: 1 log, 0 pass-run logs, 0 completed calls, 0 on ids no card names, 0 paths unreadable, '
                + '0 dangling links with no log name passed over; '
                + 'lines left out for a stamp the logger did not write: 0',
              CAP_CENSUS_CLOSING_NOTE,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'cap-census as built',
      children: [
        it({
          name: 'PRINTS THE CENSUS of the pass-run logs under the named paths and exits 0, leaving out a log '
            + 'with no run marker, a file that is no log, a skipped directory and counting the path it cannot read',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'cap-census-built-', },);
            await using runs = await scratchDir({ prefix: 'cap-census-built-runs-', },);
            await writeLog({
              directory: scratch.path,
              name: 'pass.log',
              lines: passRunLines(),
            },);
            await writeLog({
              directory: scratch.path,
              name: 'suite.log',
              lines: ['a suite log',],
            },);
            await writeFile(
              join(
                scratch.path,
                'notes.txt',
              ),
              `${PASS_START}\n`,
              'utf8',
            );
            await mkdir(join(
              scratch.path,
              'slice-cache',
            ),);
            await writeLog({
              directory: join(
                scratch.path,
                'slice-cache',
              ),
              name: 'skipped.log',
              lines: passRunLines(),
            },);

            /**
             A path the walk cannot read.
             */
            const missing = join(
              scratch.path,
              'no-such-nest',
            );

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: 'cap-census',
              packageEntry: RUN_ENTRY,
              args: [
                scratch.path,
                missing,
              ],
              env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
            },);

            expect(run.code,).toBe(0,);
            expect(run.stderr,).toBe(
              `cap-census: cannot read ${missing}: Error: ENOENT: no such file or directory, stat '${missing}'\n`,
            );
            expect(run.stdout,).toBe([
              'cap-census: 2 logs, 1 pass-run log, 2 completed calls, 0 on ids no card names, 1 path unreadable, '
                + '0 dangling links with no log name passed over; '
                + 'lines left out for a stamp the logger did not write: 0',
              `${SEAT}: card cap ${String(COMPLETION_CAP[SEAT],)}, rule reads nothing (too few calls)`,
              `  hyper: 2 calls, p99 ${String(COMPLETION_CAP[SEAT],)}; since the caps 2 calls, 1 at the cap (50.00%): `
                + '1 with content, 0 with none, 0 unpaired',
              CAP_CENSUS_CLOSING_NOTE,
              '',
            ].join('\n',),);
          },
        },),
        it({
          name: 'REFUSES A COMMAND LINE NAMING NO PATH as stated and exits 6 with its line, before reading anything',
          fn: async () => {
            await using runs = await scratchDir({ prefix: 'cap-census-built-runs-', },);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: 'cap-census',
              packageEntry: RUN_ENTRY,
              args: [],
              env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'cap-census: this command needs <log file or directory>. '
                + 'Usage: cap-census <log file or directory> [<log file or directory> ...]\n',
            );
          },
        },),
        it({
          name: 'REFUSES A FLAG IT DOES NOT DECLARE as stated and exits 6 with its line',
          fn: async () => {
            await using runs = await scratchDir({ prefix: 'cap-census-built-runs-', },);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: 'cap-census',
              packageEntry: RUN_ENTRY,
              args: ['--tabby',],
              env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'cap-census: --tabby is not a flag this command reads; this command needs <log file or directory>. '
                + 'Usage: cap-census <log file or directory> [<log file or directory> ...]\n',
            );
          },
        },),
      ],
    },),
  ],
},);
