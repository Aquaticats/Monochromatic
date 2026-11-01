/**
 Tests for the audit sensitivity run over scripted clients, in which no model
 is ever called.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { RUN_MODELS, } from '@monochromatic-dev/translation-repair-run';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure';
import {
  AUDIT_ARMS,
  runAuditSensitivity,
} from '../../dist/final/node/index.mjs';
import {
  auditScriptedClient,
  POLARITY_FINDING,
  QUIET_AUDIT_REPORT,
} from '@monochromatic-dev/translation-repair-provider/ts/audit-scripted-client.test-fixture.ts';
import { successiveClients, } from '@monochromatic-dev/translation-repair-run/ts/introduced-defect-scripted-client.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Digest the run stamps, whose last eight characters name the file.
 */
const DIGEST = `sha256-tree-v1:${'c'.repeat(64,)}`;

/**
 Closure the run records.
 */
const CLOSURE = {
  kind: 'read',
  entry: 'audit-sensitivity.mjs',
  chunks: [],
} as const;

/**
 Builds a client in which every auditor casts the same report.

 @param flipped - whether every auditor claims the planted polarity defect

 @returns Scripted client

 @example
 ```ts
 const client = everyAuditorCasts({ flipped: true, },);
 ```
 */
function everyAuditorCasts({ flipped, }: { readonly flipped: boolean; },): ReturnType<typeof auditScriptedClient> {
  return auditScriptedClient({
    reportFor: function same() {
      return flipped
        ? {
          verdict: 'defects-found',
          findings: [POLARITY_FINDING,],
        }
        : QUIET_AUDIT_REPORT;
    },
    asked: [],
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runAuditSensitivity.name,
      concurrency: 1,
      children: [
        it({
          name: 'AUDITS both arms with one fresh client each, prints each arm and keeps the rows under the runs directory',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'audit-sensitivity-run-', },);
            /**
             Builder over one client per arm and the count it handed out.
             */
            const { newClient, built, } = successiveClients({
              clients: [
                everyAuditorCasts({ flipped: true, },),
                everyAuditorCasts({ flipped: false, },),
              ],
            },);

            await runAuditSensitivity({
              arms: AUDIT_ARMS,
              newClient,
              runsDir: scratch.path,
              startedAt: '2026-10-05T01:02:03.000Z',
              now: function finished(): string {
                return '2026-10-05T01:02:09.000Z';
              },
              pipelineDigest: DIGEST,
              runnerClosure: CLOSURE,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            /**
             File the rows were kept in, named for the start and the digest's tail.
             */
            const keptAt = `${scratch.path}/audit-sensitivity/2026-10-05T01-02-03.000Z-cccccccc.json`;
            expect(built(),).toBe(2,);
            expect(printed.lines.filter(function isTally(line,): boolean {
              return line.startsWith('SENSITIVITY ',);
            },),).toEqual([
              'SENSITIVITY arm=flipped expected=agreement at either tier on the oracle span heard=3/3 '
                + 'corroborated=1 agreed=1 agreedVoices=3 near=0 oracleVoices=3 findings=0',
              'SENSITIVITY arm=clean expected=agreement at neither tier heard=3/3 corroborated=0 agreed=0 '
                + 'agreedVoices=0 near=0 oracleVoices=0 findings=0',
              `SENSITIVITY kept 2 arms at ${keptAt}`,
            ],);

            /**
             What was kept, read back.
             */
            const kept: unknown = JSON.parse(await readFile(
              keptAt,
              'utf8',
            ),);
            if (!isJsonRecord(kept,))
              throw new Error('the kept run is not a record',);
            if (!isJsonArray(kept.rows,))
              throw new Error('the kept run holds no rows',);
            expect({
              startedAt: kept.startedAt,
              finishedAt: kept.finishedAt,
              pipelineDigest: kept.pipelineDigest,
              runnerClosure: kept.runnerClosure,
              roster: kept.roster,
              subject: kept.subject,
              rows: kept.rows.map(function brief(row,) {
                if (!isJsonRecord(row,))
                  throw new Error('a kept row is not a record',);
                return {
                  arm: row.arm,
                  oracleVoices: row.oracleVoices,
                };
              },),
            },).toEqual({
              startedAt: '2026-10-05T01:02:03.000Z',
              finishedAt: '2026-10-05T01:02:09.000Z',
              pipelineDigest: DIGEST,
              runnerClosure: CLOSURE,
              roster: RUN_MODELS.checkerModelIds,
              subject: {
                fixtures: 'audit-sensitivity-input.ts',
                arms: [
                  'flipped',
                  'clean',
                ],
              },
              rows: [
                {
                  arm: 'flipped',
                  oracleVoices: 3,
                },
                {
                  arm: 'clean',
                  oracleVoices: 0,
                },
              ],
            },);
          },
        },),

        it({
          name: 'KEEPS one arm in the singular and reads the finish instant from the clock after the last arm',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'audit-sensitivity-run-', },);

            await runAuditSensitivity({
              arms: AUDIT_ARMS.slice(
                1,
                2,
              ),
              newClient: successiveClients({ clients: [everyAuditorCasts({ flipped: false, },),], },).newClient,
              runsDir: scratch.path,
              startedAt: '2026-10-05T01:02:03.000Z',
              now: function finished(): string {
                return '2026-10-05T04:05:06.000Z';
              },
              pipelineDigest: DIGEST,
              runnerClosure: CLOSURE,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(printed.lines.at(-1,),).toBe(
              `SENSITIVITY kept 1 arm at ${scratch.path}/audit-sensitivity/2026-10-05T01-02-03.000Z-cccccccc.json`,
            );
            /**
             What was kept, read back.
             */
            const kept: unknown = JSON.parse(await readFile(
              `${scratch.path}/audit-sensitivity/2026-10-05T01-02-03.000Z-cccccccc.json`,
              'utf8',
            ),);
            if (!isJsonRecord(kept,))
              throw new Error('the kept run is not a record',);
            expect(
              kept.finishedAt,
            ).toBe('2026-10-05T04:05:06.000Z',);
          },
        },),
      ],
    },),
  ],
},);
