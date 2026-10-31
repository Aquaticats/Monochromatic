/**
 Tests for reading the log files a report command was handed.

 A LOG THAT WILL NOT READ IS REFUSED IN WORDS. A path mistyped on the command
 line is the operator's to mend, so the refusal names the path they typed and
 the filesystem's code and says what to name instead; it quotes nothing the
 filesystem said, since a failure's own message repeats a path.

 A LOG NAMED TWICE IS REFUSED, including where two spellings reach one file,
 since totalling a log twice reads as a run twice as large.

 THE CLOSING NEWLINE IS NOT A LINE, and a blank line inside a log is one.

 Every file is written under a scratch directory. Fixtures are cat-themed
 invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  readdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  linesOfLog,
  readLogTexts,
  refuseRepeatedLogs,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { writeRunFolder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/run-log-write.test-fixture.ts';

await describe({
  name: 'report log read',
  concurrency: 1,
  children: [
    describe({
      name: readLogTexts.name,
      concurrency: 1,
      children: [
        it({
          name: 'READS every named record file whole, a line to a line and each closed by a line feed, in the order named',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'report-log-read-', },);
            /**
             Two record files, named in the reverse of their file names' order.
             */
            const first = join(
              scratch.path,
              'b.log.jsonl',
            );
            const second = join(
              scratch.path,
              'a.log.jsonl',
            );
            await writeFile(
              first,
              'purr\nnap\n',
            );
            await writeFile(
              second,
              'stretch',
            );

            expect(await readLogTexts({
              paths: [
                first,
                second,
              ],
            },),).toEqual([
              'purr\nnap\n',
              'stretch\n',
            ],);
          },
        },),

        it({
          name: 'READS nothing for no named log',
          fn: async () => {
            expect(await readLogTexts({ paths: [], },),).toEqual([],);
          },
        },),

        it({
          name: 'REFUSES a log no file stands at, naming the path as typed and the code, with the cause kept',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'report-log-read-', },);
            /**
             Path where nothing stands.
             */
            const missing = join(
              scratch.path,
              'missing.log.jsonl',
            );

            /**
             What the read threw.
             */
            const refusal = await rejectionOf(async function readsAbsent(): Promise<unknown> {
              return await readLogTexts({ paths: [missing,], },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              `StatedRefusalError: the log ${missing} could not be read (ENOENT), so nothing was counted. `
                + 'Name a run folder a pass wrote, or one of its .log.jsonl record files, by a path that exists.',
            );
            expect(Error.isError(refusal,) && Error.isError(refusal.cause,),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES a directory that holds no record file, in words of its own and not with a filesystem code',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'report-log-read-', },);
            /**
             Empty directory named where a run folder belongs.
             */
            const directory = join(
              scratch.path,
              'logs',
            );
            await mkdir(directory,);

            /**
             What the read threw.
             */
            const refusal = await rejectionOf(async function readsDirectory(): Promise<unknown> {
              return await readLogTexts({ paths: [directory,], },);
            },);

            expect(String(refusal,),).toBe(
              `StatedRefusalError: ${directory} holds no record file (log/<day>/*.log.jsonl), so there is nothing to count. `
                + 'Name the run folder of a pass that ran, or one of its record files. Nothing was counted.',
            );
          },
        },),
      ],
    },),

    describe({
      name: refuseRepeatedLogs.name,
      concurrency: 1,
      children: [
        it({
          name: 'ACCEPTS no log, one log and several different logs',
          fn: async ctx => {
            await using scratch = await scratchDir({ prefix: 'report-log-read-', },);
            /**
             Two run folders of two passes.
             */
            const [tabby, calico,] = await Promise.all([
              writeRunFolder({
                sinon: ctx.sinon,
                dir: scratch.path,
                name: 'corpus-pass-tabby',
                day: '2026-10-23',
                lines: ['purr',],
              },),
              writeRunFolder({
                sinon: ctx.sinon,
                dir: scratch.path,
                name: 'corpus-pass-calico',
                day: '2026-10-23',
                lines: ['nap',],
              },),
            ],);

            await refuseRepeatedLogs({ paths: [], },);
            await refuseRepeatedLogs({ paths: [tabby,], },);
            await refuseRepeatedLogs({
              paths: [
                tabby,
                calico,
              ],
            },);
          },
        },),

        it({
          name: 'REFUSES a log named a second time, naming it as typed the second time',
          fn: async () => {
            /**
             What the check threw.
             */
            const refusal = await rejectionOf(async function namesTwice(): Promise<unknown> {
              return await refuseRepeatedLogs({
                paths: [
                  '/tmp/tabby.log',
                  '/tmp/calico.log',
                  '/tmp/tabby.log',
                ],
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: the log /tmp/tabby.log is named more than once, so its lines would be counted '
                + 'twice. Name each log once.',
            );
          },
        },),

        it({
          name: 'REFUSES two spellings of one path, since they reach one file',
          fn: async () => {
            /**
             What the check threw.
             */
            const refusal = await rejectionOf(async function namesTwoSpellings(): Promise<unknown> {
              return await refuseRepeatedLogs({
                paths: [
                  '/tmp/tabby.log',
                  '/tmp/cats/../tabby.log',
                ],
              },);
            },);

            expect(String(refusal,),).toBe(
              'StatedRefusalError: the log /tmp/cats/../tabby.log is named more than once, so its lines would be '
                + 'counted twice. Name each log once.',
            );
          },
        },),

        it({
          name: 'REFUSES a run folder named beside its own log directory, a day directory or one of its files, and a '
            + 'link to a run folder already named, since each reaches the same record files by another path '
            + '(ledger B1119)',
          fn: async ctx => {
            await using scratch = await scratchDir({ prefix: 'report-log-read-', },);
            /**
             A run folder of one pass.
             */
            const runDir = await writeRunFolder({
              sinon: ctx.sinon,
              dir: scratch.path,
              name: 'corpus-pass-tabby',
              day: '2026-10-23',
              lines: ['purr',],
            },);
            /**
             The record file inside it.
             */
            const [file = 'no record file',] = await readdir(join(
              runDir,
              'log',
              '2026-10-23',
            ),);
            /**
             A link to the run folder.
             */
            const link = join(
              scratch.path,
              'link-to-tabby',
            );
            await symlink(
              runDir,
              link,
            );
            /**
             The pairs that name one run's files twice, each with the log that is named second.
             */
            const overlaps: readonly (readonly [readonly string[], string])[] = [
              [
                [
                  runDir,
                  join(
                    runDir,
                    'log',
                  ),
                ],
                join(
                  runDir,
                  'log',
                ),
              ],
              [
                [
                  join(
                    runDir,
                    'log',
                  ),
                  runDir,
                ],
                runDir,
              ],
              [
                [
                  runDir,
                  join(
                    runDir,
                    'log',
                    '2026-10-23',
                    file,
                  ),
                ],
                join(
                  runDir,
                  'log',
                  '2026-10-23',
                  file,
                ),
              ],
              [
                [
                  join(
                    runDir,
                    'log',
                    '2026-10-23',
                  ),
                  join(
                    runDir,
                    'log',
                    '2026-10-23',
                    file,
                  ),
                ],
                join(
                  runDir,
                  'log',
                  '2026-10-23',
                  file,
                ),
              ],
              [
                [
                  runDir,
                  link,
                ],
                link,
              ],
            ];
            for (const [paths, second,] of overlaps) {
              /**
               What the check threw for this pair.
               */
              // oxlint-disable-next-line no-await-in-loop -- one pair after the other, so a failing pair is the one named
              const refusal = await rejectionOf(async function namesOneRunTwice(): Promise<unknown> {
                return await refuseRepeatedLogs({ paths, },);
              },);

              expect(refusal,).toBeInstanceOf(StatedRefusalError,);
              expect(String(refusal,),).toBe(
                `StatedRefusalError: the log ${second} reaches the record file ${file}, which the log ${
                  paths[0]
                } reaches too, so its lines would be counted twice. Name each run folder or record file once, and `
                  + 'not a run folder together with its log directory or one of its files.',
              );
            }
          },
        },),

        it({
          name: 'REFUSES a log that will not list as the readers do, before it compares anything',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'report-log-read-', },);
            /**
             Folder where no run was kept.
             */
            const empty = join(
              scratch.path,
              'empty',
            );
            await mkdir(empty,);

            /**
             What the check threw.
             */
            const refusal = await rejectionOf(async function namesAnEmptyFolder(): Promise<unknown> {
              return await refuseRepeatedLogs({ paths: [empty,], },);
            },);

            expect(String(refusal,),).toBe(
              `StatedRefusalError: ${empty} holds no record file (log/<day>/*.log.jsonl), so there is nothing to count. `
                + 'Name the run folder of a pass that ran, or one of its record files. Nothing was counted.',
            );
          },
        },),
      ],
    },),

    describe({
      name: linesOfLog.name,
      concurrency: 1,
      children: [
        it({
          name: 'SPLITS a log into its lines whether or not a newline closes it',
          fn: async () => {
            expect(linesOfLog({ text: 'purr\nnap', },),).toEqual([
              'purr',
              'nap',
            ],);
            expect(linesOfLog({ text: 'purr\nnap\n', },),).toEqual([
              'purr',
              'nap',
            ],);
          },
        },),

        it({
          name: 'FINDS no line in an empty log and one empty line in a log of one newline',
          fn: async () => {
            expect(linesOfLog({ text: '', },),).toEqual([],);
            expect(linesOfLog({ text: '\n', },),).toEqual([''],);
          },
        },),

        it({
          name: 'KEEPS a blank line inside a log and a blank line before the closing newline',
          fn: async () => {
            expect(linesOfLog({ text: 'purr\n\nnap\n\n', },),).toEqual([
              'purr',
              '',
              'nap',
              '',
            ],);
          },
        },),
      ],
    },),
  ],
},);
