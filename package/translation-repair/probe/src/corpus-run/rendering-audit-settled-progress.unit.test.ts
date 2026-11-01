/**
 Tests for the line a settled audit prints for each slice as it lands: which
 text was audited, what a later stage made of it, and the counts beside them.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  printSettledRow,
  type SettledAuditRow,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 Builds one audited slice as the probe persists it, carrying only what the line reads.

 @param auditsArchiveText - whether the slice audited the archive's own English

 @param pageRelation - what a document would carry at the slice, absent to build a row from before the field existed

 @param voices - how many claims each voice anchored, one entry per voice

 @param tiers - how many entries each agreement tier and the degradation hold

 @returns Row shaped as the probe persists it

 @example
 ```ts
 const row = rowOf({ auditsArchiveText: false, pageRelation: { kind: 'survives', }, voices: [2, 1,], tiers: { corroborated: 1, agreed: 2, near: 0, degraded: 0, }, },);
 ```
 */
function rowOf(
  {
    auditsArchiveText,
    pageRelation,
    voices,
    tiers,
  }: {
    readonly auditsArchiveText: boolean;
    readonly pageRelation?: unknown;
    readonly voices: readonly number[];
    readonly tiers: {
      readonly corroborated: number;
      readonly agreed: number;
      readonly near: number;
      readonly degraded: number;
    };
  },
): SettledAuditRow {
  /**
   One placeholder entry per count, since the line reads lengths only.
   */
  function entries({ count, }: { readonly count: number; },): readonly string[] {
    return Array.from(
      { length: count, },
      function purr(): string {
        return 'purr';
      },
    );
  }
  return {
    runSet: 'naptime-20260825',
    entryId: 'mittens',
    sliceIndex: 3,
    auditsArchiveText,
    ...((pageRelation === undefined) ? {} : { pageRelation, }),
    report: {
      corroborated: entries({ count: tiers.corroborated, },),
      agreed: entries({ count: tiers.agreed, },),
      near: entries({ count: tiers.near, },),
      findings: entries({ count: tiers.degraded, },),
      rows: voices.map(function voiceOf(claims,): { readonly findings: readonly string[]; } {
        return { findings: entries({ count: claims, },), };
      },),
    },
  } as unknown as SettledAuditRow;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: printSettledRow.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS A FRESH SLICE with its page relation, the claims every voice anchored added together, '
            + 'each tier\'s count, and no degraded count when nothing degraded',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSettledRow({
              row: rowOf({
                auditsArchiveText: false,
                pageRelation: { kind: 'survives', },
                voices: [
                  2,
                  1,
                ],
                tiers: {
                  corroborated: 1,
                  agreed: 2,
                  near: 0,
                  degraded: 0,
                },
              },),
            },);
            expect(printed.lines,).toEqual([
              'naptime-20260825/mittens#3 FRESH   survives claimed=3 corroborated=1 agreed=2 near=0',
            ],);
          },
        },),

        it({
          name: 'PRINTS AN ARCHIVE SLICE with the stage that displaced it and the count of degraded findings '
            + 'after the near misses',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSettledRow({
              row: rowOf({
                auditsArchiveText: true,
                pageRelation: {
                  kind: 'displaced',
                  decidedBy: 'consolidation',
                },
                voices: [0,],
                tiers: {
                  corroborated: 0,
                  agreed: 0,
                  near: 4,
                  degraded: 2,
                },
              },),
            },);
            expect(printed.lines,).toEqual([
              'naptime-20260825/mittens#3 ARCHIVE displaced:consolidation claimed=0 corroborated=0 agreed=0 near=4 degraded=2',
            ],);
          },
        },),

        it({
          name: 'PRINTS A ROW FROM BEFORE THE PAGE RELATION WAS KEPT as unrecorded, and a roster with no voice as '
            + 'no claims',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSettledRow({
              row: rowOf({
                auditsArchiveText: false,
                voices: [],
                tiers: {
                  corroborated: 0,
                  agreed: 0,
                  near: 0,
                  degraded: 0,
                },
              },),
            },);
            expect(printed.lines,).toEqual([
              'naptime-20260825/mittens#3 FRESH   unrecorded claimed=0 corroborated=0 agreed=0 near=0',
            ],);
          },
        },),
      ],
    },),
  ],
},);
