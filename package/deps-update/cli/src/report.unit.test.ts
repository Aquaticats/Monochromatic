/**
 Unit tests for the immature-pick report.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NO_MINIMUM_RELEASE_AGE, } from './exclude-list.ts';
import {
  formatImmatureReport,
  type ImmaturePick,
  matureAt,
} from './report.ts';

/**
 Configured age used by the repository (one day).
 */
const ONE_DAY_MINUTES = 1_440;

/**
 Pick pulled in transitively.
 */
const CHORD: ImmaturePick = {
  name: '@earendil-works/chord',
  version: '0.87.1',
  publishedAt: new Date('2026-09-22T19:38:00.606Z',),
  dependents: ['@earendil-works/pi-coding-agent@0.87.1',],
};

/**
 Pick declared directly by a workspace package, published later.
 */
const DIRECT: ImmaturePick = {
  name: 'left-pad',
  version: '2.0.0',
  publishedAt: new Date('2026-09-22T21:00:00.000Z',),
  dependents: [],
};

await describe({
  name: '',
  children: [
    describe({
      name: matureAt.name,
      children: [
        it({
          name: 'adds the configured minutes',
          fn: async () => {
            expect(matureAt({ publishedAt: CHORD.publishedAt, minutes: ONE_DAY_MINUTES, },).toISOString(),).toBe(
              '2026-09-23T19:38:00.606Z',
            );
          },
        },),
      ],
    },),
    describe({
      name: formatImmatureReport.name,
      children: [
        it({
          name: 'lists each pick with maturity and dependents',
          fn: async () => {
            /**
             Rendered report.
             */
            const report = formatImmatureReport({ picks: [CHORD, DIRECT,], minutes: ONE_DAY_MINUTES, },);
            expect(report,).toContain('pnpm update refused 2 version(s)',);
            expect(report,).toContain('@earendil-works/chord@0.87.1',);
            expect(report,).toContain('passes the age gate: 2026-09-23T19:38:00.606Z',);
            expect(report,).toContain('pulled in by: @earendil-works/pi-coding-agent@0.87.1',);
            expect(report,).toContain('pulled in by: a workspace package directly',);
          },
        },),
        it({
          name: 'suggests rerunning after the latest maturity',
          fn: async () => {
            expect(formatImmatureReport({ picks: [CHORD, DIRECT,], minutes: ONE_DAY_MINUTES, },),).toContain(
              'rerun after 2026-09-23T21:00:00.000Z',
            );
          },
        },),
        it({
          name: 'omits maturity times when the age is unset',
          fn: async () => {
            /**
             Rendered report without an age.
             */
            const report = formatImmatureReport({ picks: [CHORD,], minutes: NO_MINIMUM_RELEASE_AGE, },);
            expect(report,).toContain('passes the age gate: unknown',);
            expect(report,).toContain('Wait until every pick above passes the age gate',);
          },
        },),
        it({
          name: 'appends a follow-up failure',
          fn: async () => {
            expect(formatImmatureReport({
              picks: [CHORD,],
              minutes: ONE_DAY_MINUTES,
              followUp: '[ERR_PNPM_PEER_DEP_ISSUES] Unmet peer dependencies',
            },),).toContain('separate failure the scratch resolution also hit:\n  [ERR_PNPM_PEER_DEP_ISSUES]',);
          },
        },),
        it({
          name: 'explains an empty diagnosis',
          fn: async () => {
            expect(formatImmatureReport({ picks: [], minutes: ONE_DAY_MINUTES, },),).toContain(
              'appended no minimumReleaseAgeExclude entries',
            );
          },
        },),
      ],
    },),
  ],
},);
