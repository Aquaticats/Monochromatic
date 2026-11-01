import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  PLAIN_RATE_LIMIT_STYLE,
  formatProjectionMarker,
  formatRateLimitSegment,
  formatRateLimitStatus,
  formatRelativeTime,
  identityStyle,
  isNonEmptySegment,
  projectUsagePercent,
  rateLimitSeverity,
  remainingPercent,
  styleBySeverity,
  type RateLimitSnapshot,
  type RateLimitStyle,
} from './index.ts';

/**
 Stable sample time used by rate-limit projection tests.
 */
const SAMPLED_AT_MS = Date.parse('2026-06-01T12:00:00Z',);

/**
 Milliseconds in one second for test fixtures.
 */
const SECOND_MS = 1_000;

/**
 Milliseconds in one minute for test fixtures.
 */
const MINUTE_MS = 60 * SECOND_MS;

/**
 Milliseconds in one hour for test fixtures.
 */
const HOUR_MS = 60 * MINUTE_MS;

/**
 Milliseconds in one day for test fixtures.
 */
const DAY_MS = 24 * HOUR_MS;

/**
 Five-hour test window in seconds.
 */
const FIVE_HOUR_WINDOW_SECONDS = 5 * 3_600;

/**
 Angle-bracket style used to verify selected severity.
 */
const ANGLE_STYLE: RateLimitStyle = {
  green: function green(text,): string {
    return `<green>${text}</green>`;
  },
  yellow: function yellow(text,): string {
    return `<yellow>${text}</yellow>`;
  },
  red: function red(text,): string {
    return `<red>${text}</red>`;
  },
};

/**
 Builds a rate-limit snapshot for shared formatter tests.
 
 @param label - statusline label prefix
 
 @param usedPercent - current used capacity percentage
 
 @param resetOffsetMs - reset offset from {@link SAMPLED_AT_MS}
 
 @returns rate-limit snapshot fixture
 
 @example
 ```ts
 snapshot({ label: 'demo', usedPercent: 60, resetOffsetMs: HOUR_MS });
 ```
 */
function snapshot({
  label,
  usedPercent,
  resetOffsetMs,
}: Readonly<{
  label: string;
  usedPercent: number;
  resetOffsetMs: number;
}>,): RateLimitSnapshot {
  return {
    key: label.length > 0 ? label : 'unlabeled',
    label,
    resetAtMs: SAMPLED_AT_MS + resetOffsetMs,
    windowSeconds: FIVE_HOUR_WINDOW_SECONDS,
    paceScale: 1,
    sampledAtMs: SAMPLED_AT_MS,
    usedPercent,
  };
}

await describe({
  name: 'usage projection formatting',
  children: [
    describe({
      name: formatRelativeTime.name,
      children: [
        it({
          name: 'formats reset times across duration bands',
          fn: async function testRelativeTimeBands(): Promise<void> {
            expect(formatRelativeTime({ resetAtMs: SAMPLED_AT_MS, renderedAtMs: SAMPLED_AT_MS, },),)
              .toBe('now',);
            expect(formatRelativeTime({ resetAtMs: SAMPLED_AT_MS + (30 * SECOND_MS), renderedAtMs: SAMPLED_AT_MS, },),)
              .toBe('30s',);
            expect(formatRelativeTime({ resetAtMs: SAMPLED_AT_MS + (5 * MINUTE_MS), renderedAtMs: SAMPLED_AT_MS, },),)
              .toBe('5m',);
            expect(formatRelativeTime({ resetAtMs: SAMPLED_AT_MS + (2 * HOUR_MS) + (30 * MINUTE_MS), renderedAtMs: SAMPLED_AT_MS, },),)
              .toBe('2h30m',);
            expect(formatRelativeTime({ resetAtMs: SAMPLED_AT_MS + (3 * DAY_MS) + (2 * HOUR_MS), renderedAtMs: SAMPLED_AT_MS, },),)
              .toBe('3d2h',);
          },
        },),
      ],
    },),
    describe({
      name: projectUsagePercent.name,
      children: [
        it({
          name: 'uses sampledAtMs rather than render time for projection',
          fn: async function testProjectionUsesSampleTime(): Promise<void> {
            /**
             Snapshot sampled one hour into a five-hour window.
             */
            const current = snapshot({
              label: 'demo',
              usedPercent: 30,
              resetOffsetMs: 4 * HOUR_MS,
            },);

            expect(projectUsagePercent({ snapshot: current, },),).toBe(150,);
          },
        },),
        it({
          name: 'returns zero without stable elapsed window data',
          fn: async function testProjectionSuppression(): Promise<void> {
            const lowUsage = snapshot({
              label: 'low',
              usedPercent: 4,
              resetOffsetMs: 4 * HOUR_MS,
            },);
            const futureReset = snapshot({
              label: 'future',
              usedPercent: 20,
              resetOffsetMs: 6 * HOUR_MS,
            },);

            expect(projectUsagePercent({ snapshot: lowUsage, },),).toBe(0,);
            expect(projectUsagePercent({ snapshot: futureReset, },),).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: formatProjectionMarker.name,
      children: [
        it({
          name: 'floors projected percentage marker',
          fn: async function testProjectionMarker(): Promise<void> {
            expect(formatProjectionMarker(0,),).toBe('→0%',);
            expect(formatProjectionMarker(101.9,),).toBe('→101%',);
          },
        },),
      ],
    },),
    describe({
      name: remainingPercent.name,
      children: [
        it({
          name: 'computes remaining capacity and clamps overuse to zero',
          fn: async function testRemainingPercent(): Promise<void> {
            /**
             Snapshot whose provider reports credit above full capacity.
             */
            const overCredit = snapshot({
              label: 'over-credit',
              usedPercent: -5,
              resetOffsetMs: HOUR_MS,
            },);
            /**
             Snapshot with ordinary partial usage.
             */
            const partial = snapshot({
              label: 'partial',
              usedPercent: 35,
              resetOffsetMs: HOUR_MS,
            },);
            /**
             Snapshot whose provider reports usage past full capacity.
             */
            const overused = snapshot({
              label: 'overused',
              usedPercent: 140,
              resetOffsetMs: HOUR_MS,
            },);

            expect(remainingPercent(overCredit,),).toBe(105,);
            expect(remainingPercent(partial,),).toBe(65,);
            expect(remainingPercent(overused,),).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: rateLimitSeverity.name,
      children: [
        it({
          name: 'selects green, yellow, and red severity bands',
          fn: async function testSeverityBands(): Promise<void> {
            expect(rateLimitSeverity({ remaining: 50, projectedPercent: 0, },),).toBe('green',);
            expect(rateLimitSeverity({ remaining: 25, projectedPercent: 0, },),).toBe('yellow',);
            expect(rateLimitSeverity({ remaining: 10, projectedPercent: 0, },),).toBe('red',);
            expect(rateLimitSeverity({ remaining: 80, projectedPercent: 101, },),).toBe('red',);
          },
        },),
      ],
    },),
    describe({
      name: styleBySeverity.name,
      children: [
        it({
          name: 'selects matching style callback for every severity',
          fn: async function testStyleBySeverity(): Promise<void> {
            expect(styleBySeverity({
              text: 'ok',
              severity: 'green',
              style: ANGLE_STYLE,
            },),).toBe('<green>ok</green>',);
            expect(styleBySeverity({
              text: 'warn',
              severity: 'yellow',
              style: ANGLE_STYLE,
            },),).toBe('<yellow>warn</yellow>',);
            expect(styleBySeverity({
              text: 'bad',
              severity: 'red',
              style: ANGLE_STYLE,
            },),).toBe('<red>bad</red>',);
          },
        },),
      ],
    },),
    describe({
      name: identityStyle.name,
      children: [
        it({
          name: 'returns style text unchanged',
          fn: async function testIdentityStyle(): Promise<void> {
            expect(identityStyle('plain',),).toBe('plain',);
          },
        },),
      ],
    },),
    describe({
      name: isNonEmptySegment.name,
      children: [
        it({
          name: 'keeps only non-empty segments',
          fn: async function testIsNonEmptySegment(): Promise<void> {
            expect(['', 'visible',].filter(function keepSegment(segment,): boolean {
              return isNonEmptySegment(segment,);
            },),).toStrictEqual(['visible',],);
          },
        },),
      ],
    },),
    describe({
      name: formatRateLimitSegment.name,
      children: [
        it({
          name: 'hides comfortable remaining capacity without projected overrun',
          fn: async function testComfortableHidden(): Promise<void> {
            expect(formatRateLimitSegment({
              snapshot: snapshot({
                label: 'demo',
                usedPercent: 20,
                resetOffsetMs: 4 * HOUR_MS,
              },),
              renderedAtMs: SAMPLED_AT_MS,
              style: PLAIN_RATE_LIMIT_STYLE,
            },),).toBe('',);
          },
        },),
        it({
          name: 'formats labeled remaining-capacity warning with selected severity',
          fn: async function testLabeledWarning(): Promise<void> {
            expect(formatRateLimitSegment({
              snapshot: snapshot({
                label: 'demo',
                usedPercent: 80,
                resetOffsetMs: HOUR_MS,
              },),
              renderedAtMs: SAMPLED_AT_MS,
              style: ANGLE_STYLE,
            },),).toBe('demo <yellow>20% left</yellow> (1h)',);
          },
        },),
        it({
          name: 'formats projected overrun with red severity and marker',
          fn: async function testProjectedOverrun(): Promise<void> {
            expect(formatRateLimitSegment({
              snapshot: snapshot({
                label: '',
                usedPercent: 30,
                resetOffsetMs: 4 * HOUR_MS,
              },),
              renderedAtMs: SAMPLED_AT_MS,
              style: ANGLE_STYLE,
            },),).toBe('<red>70% left →150%</red> (4h)',);
          },
        },),
      ],
    },),
    describe({
      name: formatRateLimitStatus.name,
      children: [
        it({
          name: 'joins visible segments and drops hidden ones',
          fn: async function testJoinedSegments(): Promise<void> {
            const status = formatRateLimitStatus({
              snapshots: [
                snapshot({
                  label: 'hidden',
                  usedPercent: 20,
                  resetOffsetMs: 4 * HOUR_MS,
                },),
                snapshot({
                  label: 'visible',
                  usedPercent: 80,
                  resetOffsetMs: HOUR_MS,
                },),
              ],
              renderedAtMs: SAMPLED_AT_MS,
              style: PLAIN_RATE_LIMIT_STYLE,
            },);

            expect(status.statusText,).toBe('visible 20% left (1h)',);
          },
        },),
      ],
    },),
  ],
},);
