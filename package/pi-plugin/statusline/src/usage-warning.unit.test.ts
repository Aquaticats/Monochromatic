import { spawnSync, } from 'node:child_process';
import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { parseRateLimitSnapshots, } from './rate-limit-headers.ts';
import {
  PLAIN_USAGE_WARNING_STYLE,
  RATE_LIMIT_WINDOW_SECONDS,
  type RateLimitSnapshot,
  type UsageWarningStyle,
} from './rate-limit-types.ts';
import {
  formatRelativeTime,
  formatUsageWarningSegment,
  formatUsageWarningStatus,
  projectUsagePercent,
} from './usage-warning.ts';

const NOW_MS = Date.parse('2026-06-01T12:00:00Z',);
const SECOND_MS = 1_000;
const MINUTE_MS = 60 * SECOND_MS;
const HOUR_MS = 60 * MINUTE_MS;
const DAY_MS = 24 * HOUR_MS;

function resetAt(offsetMs: number,): string {
  return new Date(NOW_MS + offsetMs,).toISOString();
}

function resetAtEpochSeconds(offsetMs: number,): string {
  return String((NOW_MS + offsetMs) / SECOND_MS,);
}

function tokenHeaders({
  limit,
  remaining,
  resetOffsetMs,
}: Readonly<{
  limit: number;
  remaining: number;
  resetOffsetMs: number;
}>): Record<string, string> {
  return {
    'anthropic-ratelimit-tokens-limit': String(limit,),
    'anthropic-ratelimit-tokens-remaining': String(remaining,),
    'anthropic-ratelimit-tokens-reset': resetAt(resetOffsetMs,),
  };
}

function codexHeaders({
  usedPercent,
  windowMinutes,
  resetOffsetMs,
}: Readonly<{
  usedPercent: number;
  windowMinutes: number;
  resetOffsetMs: number;
}>): Record<string, string> {
  return {
    'x-codex-primary-used-percent': String(usedPercent,),
    'x-codex-primary-window-minutes': String(windowMinutes,),
    'x-codex-primary-reset-at': resetAtEpochSeconds(resetOffsetMs,),
  };
}

function syntheticQuotasHeader(quotas: unknown,): Record<string, string> {
  return {
    'x-synthetic-quotas': JSON.stringify(quotas,),
  };
}

function weeklySnapshots(quotas: unknown,): readonly RateLimitSnapshot[] {
  return parseRateLimitSnapshots({
    headers: syntheticQuotasHeader(quotas,),
    nowMs: NOW_MS,
  },);
}

function weeklyResetSnapshots(nextRegenAt: string,): readonly RateLimitSnapshot[] {
  return weeklySnapshots({
    planWeeklyLimit: {
      nextRegenAt,
      percentRemaining: 80,
    },
  },);
}

/**
 Text carried by the header the log case feeds in, which no log line may repeat.
 */
const HEADER_VALUE_MARKER = 'MARKER-NOT-FOR-LOGS';

function unreadableFieldChildSource(): string {
  return `
    import { parseSyntheticRateLimitSnapshots, } from './src/synthetic-quota-headers.ts';
    import { logger, } from '@monochromatic-dev/logger-core/ts';

    parseSyntheticRateLimitSnapshots({
      headers: { 'x-synthetic-quotas': ${JSON.stringify(JSON.stringify({
    planWeeklyLimit: { percentRemaining: HEADER_VALUE_MARKER, nextRegenAt: HEADER_VALUE_MARKER, },
  },),)}, },
      nowMs: 0,
    },);
    await logger.flush();
  `;
}

/**
 Longest a child process may run before it is killed: a kill switch, not an elapsed-time assertion.
 
 `spawnSync` blocks the event loop, so the harness's own timer cannot interrupt a child that never exits (for example
 a `logger.flush()` that never settles). The child normally ends in about a second; sixty seconds is far above that even
 on a loaded host, and a child that reaches it fails the case instead of hanging the suite.
 */
const CHILD_TIMEOUT_MS = 60_000;

/**
 Exit status reported for a child that ended by a signal and so has none of its own.
 */
const SIGNALED_EXIT_STATUS = -1;

/**
 A throwaway directory and the disposer that removes it on every path out of the case, including a failed spawn.
 */
type LogDirectory = {
  /**
   Absolute path of the directory, handed to the child's logger so its files stay out of the project.
   */
  readonly path: string;
  /**
   Removes the directory with everything in it; `await using` calls it so a failing case leaves nothing behind.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates the throwaway log directory a child process writes into.
 
 @returns directory that removes itself when its `await using` scope ends, on every path out of the case
 
 @example
 ```ts
 await using logDirectory = await makeLogDirectory();
 ```
 */
async function makeLogDirectory(): Promise<LogDirectory> {
  const path = await mkdtemp(join(scratchRoot(), 'statusline-log-',),);

  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(path, { force: true, recursive: true, },);
    },
  };
}

/**
 Runs a TypeScript module source in a keyless child node process in the package directory and captures its streams.
 
 @param executable - node binary to run
 
 @param source - module source passed to `--eval`
 
 @param logDirectory - directory the child's logger writes into
 
 @param timeoutMs - milliseconds after which the child is killed
 
 @returns exit status and both streams
 
 @throws {Error} naming the timeout when the child outlives it, or the spawn error's message when it cannot be spawned
 
 @example
 ```ts
 runNodeChild({ executable: process.execPath, source: '', logDirectory: '/tmp/x', timeoutMs: 1000 });
 ```
 */
function runNodeChild({
  executable,
  source,
  logDirectory,
  timeoutMs,
}: Readonly<{
  executable: string;
  source: string;
  logDirectory: string;
  timeoutMs: number;
}>,): Readonly<{ status: number; stdout: string; stderr: string; }> {
  const child = spawnSync(
    executable,
    [
      '--input-type=module-typescript',
      '--eval',
      source,
    ],
    {
      cwd: join(import.meta.dirname, '..',),
      encoding: 'utf8',
      env: {
        PATH: process.env.PATH ?? '',
        HOME: process.env.HOME ?? '',
        MONOCHROMATIC_VERBOSE: 'true',
        MONOCHROMATIC_LOG_DIRECTORY: logDirectory,
      },
      killSignal: 'SIGKILL',
      timeout: timeoutMs,
    },
  );
  if (child.error !== undefined) {
    if (('code' in child.error) && (child.error.code === 'ETIMEDOUT'))
      throw new Error(`the child process timed out after ${timeoutMs} ms and was killed`,);

    throw new Error(`the child process could not run: ${child.error.message}`,);
  }

  return {
    status: child.status ?? SIGNALED_EXIT_STATUS,
    stdout: child.stdout,
    stderr: child.stderr,
  };
}

const REFUSED_RESET_TEXTS = [
  { text: '2026', why: 'a year alone, which Date.parse reads', },
  { text: 'Oct 23 2026 05:03', why: 'a month-name form, which Date.parse reads in local time', },
  { text: '2026-10-23T05:03:47.000Z\n', why: 'a trailing newline', },
  { text: '2026-10-23T05:03:47', why: 'no zone', },
  { text: '2026-10-23T05:03:47+0000', why: 'an offset without its colon', },
  { text: '2026-04-31T05:03:47Z', why: 'April has 30 days', },
  { text: '2026-02-29T05:03:47Z', why: '2026 is no leap year', },
  { text: '2026-02-30T05:03:47Z', why: 'February never has 30 days', },
  { text: '2100-02-29T05:03:47Z', why: '2100 is divisible by 100 and not by 400', },
  { text: '2026-10-23T24:00:00Z', why: 'hour 24 is an end-of-day spelling, not an instant', },
] as const;

/**
 Shape-valid texts with a field outside its range, which the reader refuses by its own range checks.
 
 The engine's `Date.parse` refuses each of these too; {@link parseWithEngineAcceptingAll} takes that second line away.
 */
const OUT_OF_RANGE_RESET_TEXTS = [
  { text: '2026-13-23T05:03:47Z', why: 'month 13', },
  { text: '2026-00-23T05:03:47Z', why: 'month 00', },
  { text: '2026-10-23T05:60:47Z', why: 'minute 60', },
  { text: '2026-10-23T05:03:60Z', why: 'second 60, a leap second is not an instant here', },
  { text: '2026-10-23T05:03:47+24:00', why: 'offset hours 24', },
  { text: '2026-10-23T05:03:47+00:60', why: 'offset minutes 60', },
  { text: '2026-10-23T05:03:47-23:60', why: 'offset minutes 60 beside valid offset hours', },
] as const;

/**
 Runs the weekly reset reader while the engine's `Date.parse` accepts every text as epoch zero, so a refusal can only
 come from the reader's own checks.
 
 @param nextRegenAt - reset text to read
 
 @returns snapshots the reader yields for that text
 
 @example
 ```ts
 parseWithEngineAcceptingAll('2026-13-23T05:03:47Z');
 ```
 */
function parseWithEngineAcceptingAll(nextRegenAt: string,): readonly RateLimitSnapshot[] {
  /**
   The engine's own parser, kept so the disposer can put it back.
   */
  const engineParse = Date.parse;
  /**
   Disposer that restores the engine's parser on every path out of this function, including a throw.
   */
  const restoreEngineParse = {
    [Symbol.dispose](): void {
      Date.parse = engineParse;
    },
  };
  /**
   Holds the disposer so the restore runs when this function returns.
   */
  using restore = restoreEngineParse;
  Date.parse = function acceptEverything(): number {
    return 0;
  };

  return weeklyResetSnapshots(nextRegenAt,);
}

const READ_RESET_TEXTS = [
  { text: '2026-10-23T05:03:47.000Z', instant: '2026-10-23T05:03:47.000Z', },
  { text: '2026-10-23T01:03:47-04:00', instant: '2026-10-23T05:03:47.000Z', },
  { text: '2024-02-29T00:00:00Z', instant: '2024-02-29T00:00:00.000Z', },
  { text: '2000-02-29T00:00:00Z', instant: '2000-02-29T00:00:00.000Z', },
] as const;

function firstSnapshot(headers: Readonly<Record<string, string>>,): RateLimitSnapshot {
  /**
   First parsed snapshot from supplied header fixture.
   */
  const [snapshot,] = parseRateLimitSnapshots({
    headers,
    nowMs: NOW_MS,
  },);
  if (snapshot === undefined)
    throw new Error('Expected one parsed snapshot',);

  return snapshot;
}

const ANGLE_STYLE: UsageWarningStyle = {
  green: function green(text: string,): string {
    return `<green>${text}</green>`;
  },
  yellow: function yellow(text: string,): string {
    return `<yellow>${text}</yellow>`;
  },
  red: function red(text: string,): string {
    return `<red>${text}</red>`;
  },
};

await describe({
  name: 'usage warning formatting',
  children: [
    describe({
      name: formatRelativeTime.name,
      children: [
        it({
          name: 'formats reset times across duration bands',
          fn: async function testRelativeTimeBands() {
            expect(formatRelativeTime({ resetAtMs: NOW_MS, nowMs: NOW_MS, },),)
              .toBe('now',);
            expect(formatRelativeTime({ resetAtMs: NOW_MS + (30 * SECOND_MS), nowMs: NOW_MS, },),)
              .toBe('30s',);
            expect(formatRelativeTime({ resetAtMs: NOW_MS + (5 * MINUTE_MS), nowMs: NOW_MS, },),)
              .toBe('5m',);
            expect(formatRelativeTime({ resetAtMs: NOW_MS + (2 * HOUR_MS) + (30 * MINUTE_MS), nowMs: NOW_MS, },),)
              .toBe('2h30m',);
            expect(formatRelativeTime({ resetAtMs: NOW_MS + (3 * DAY_MS) + (2 * HOUR_MS), nowMs: NOW_MS, },),)
              .toBe('3d2h',);
          },
        },),
      ],
    },),
    describe({
      name: parseRateLimitSnapshots.name,
      children: [
        it({
          name: 'parses Anthropic headers case-insensitively',
          fn: async function testAnthropicSnapshots() {
            const snapshots = parseRateLimitSnapshots({
              headers: {
                'Anthropic-Ratelimit-Tokens-Limit': '100',
                'Anthropic-Ratelimit-Tokens-Remaining': '25',
                'Anthropic-Ratelimit-Tokens-Reset': resetAt(40 * SECOND_MS,),
              },
              nowMs: NOW_MS,
            },);

            expect(snapshots.length,).toBe(1,);
            expect(snapshots[0]?.label,).toBe('anthropic tokens',);
            expect(snapshots[0]?.usedPercent,).toBe(75,);
            expect(snapshots[0]?.windowSeconds,).toBe(RATE_LIMIT_WINDOW_SECONDS,);
          },
        },),
        it({
          name: 'parses Codex subscription headers',
          fn: async function testCodexSnapshots() {
            const snapshots = parseRateLimitSnapshots({
              headers: codexHeaders({
                usedPercent: 30,
                windowMinutes: 300,
                resetOffsetMs: 4 * HOUR_MS,
              },),
              nowMs: NOW_MS,
            },);

            expect(snapshots.length,).toBe(1,);
            expect(snapshots[0]?.label,).toBe('codex 5h',);
            expect(snapshots[0]?.usedPercent,).toBe(30,);
          },
        },),
        it({
          name: 'parses Codex secondary and dynamic limit headers',
          fn: async function testCodexDynamicSnapshots() {
            const snapshots = parseRateLimitSnapshots({
              headers: {
                'x-codex-secondary-used-percent': '40',
                'x-codex-secondary-window-minutes': '300',
                'x-codex-secondary-reset-at': resetAtEpochSeconds(4 * HOUR_MS,),
                'x-codex-bengalfox-primary-used-percent': '30',
                'x-codex-bengalfox-primary-window-minutes': '300',
                'x-codex-bengalfox-primary-reset-at': resetAtEpochSeconds(4 * HOUR_MS,),
                'x-codex-bengalfox-limit-name': 'gpt-5.2-codex-sonic',
              },
              nowMs: NOW_MS,
            },);

            expect(snapshots.length,).toBe(2,);
            expect(snapshots[0]?.label,).toBe('codex 5h secondary',);
            expect(snapshots[1]?.label,).toBe('gpt-5.2-codex-sonic 5h',);
          },
        },),
        it({
          name: 'skips Synthetic quota windows that are not pace-projectable',
          fn: async function testSyntheticUnsupportedWindows() {
            const snapshots = parseRateLimitSnapshots({
              headers: syntheticQuotasHeader({
                rollingFiveHourLimit: {
                  nextTickAt: resetAt(30 * MINUTE_MS,),
                  tickPercent: 0.1,
                  remaining: 5,
                  max: 100,
                  limited: false,
                },
                subscription: {
                  limit: 100,
                  requests: 95,
                  renewsAt: resetAt(2 * HOUR_MS,),
                },
              },),
              nowMs: NOW_MS,
            },);

            expect(snapshots.length,).toBe(0,);
          },
        },),
        it({
          name: 'parses Synthetic quotas header',
          fn: async function testSyntheticSnapshots() {
            const snapshots = parseRateLimitSnapshots({
              headers: syntheticQuotasHeader({
                search: {
                  hourly: {
                    limit: 100,
                    requests: 60,
                    renewsAt: resetAt(30 * MINUTE_MS,),
                  },
                },
              },),
              nowMs: NOW_MS,
            },);

            expect(snapshots.length,).toBe(1,);
            expect(snapshots[0]?.label,).toBe('synthetic search',);
            expect(snapshots[0]?.usedPercent,).toBe(60,);
          },
        },),
        it({
          name: 'reads the weekly block under planWeeklyLimit as measured on 2026-10-23 03:28:16 UTC',
          fn: async function testSyntheticRenamedWeeklyBlock() {
            // The body shape of the keyed `GET /v2/quotas` of 2026-10-23: no `weeklyTokenLimit`, no `freeToolCalls`.
            const snapshots = parseRateLimitSnapshots({
              headers: syntheticQuotasHeader({
                subscription: {
                  limit: 100,
                  requests: 0,
                  renewsAt: '2026-11-01T00:00:00.000Z',
                },
                search: { hourly: {}, },
                planWeeklyLimit: {
                  nextRegenAt: '2026-10-23T05:03:47.000Z',
                  percentRemaining: 0,
                  nextRegenPercent: 2,
                },
              },),
              nowMs: Date.parse('2026-10-23T03:28:16Z',),
            },);

            expect(snapshots.length,).toBe(1,);
            expect(snapshots[0]?.key,).toBe('synthetic:week',);
            expect(snapshots[0]?.usedPercent,).toBe(100,);
            expect(snapshots[0]?.resetAtMs,).toBe(Date.parse('2026-10-23T05:03:47.000Z',),);
          },
        },),
        it({
          name: 'still reads the weekly block under weeklyTokenLimit',
          fn: async function testSyntheticOldWeeklyBlock() {
            const snapshots = parseRateLimitSnapshots({
              headers: syntheticQuotasHeader({
                weeklyTokenLimit: {
                  nextRegenAt: resetAt(20 * HOUR_MS,),
                  percentRemaining: 80,
                },
              },),
              nowMs: NOW_MS,
            },);

            expect(snapshots.length,).toBe(1,);
            expect(snapshots[0]?.key,).toBe('synthetic:week',);
            expect(snapshots[0]?.usedPercent,).toBe(20,);
          },
        },),
        it({
          name: 'reads a body carrying both weekly names when the two read the same',
          fn: async function testSyntheticBothWeeklyNamesEqual() {
            const snapshots = parseRateLimitSnapshots({
              headers: syntheticQuotasHeader({
                weeklyTokenLimit: {
                  nextRegenAt: resetAt(20 * HOUR_MS,),
                  percentRemaining: 80,
                },
                planWeeklyLimit: {
                  nextRegenAt: resetAt(20 * HOUR_MS,),
                  percentRemaining: 80,
                  nextRegenPercent: 2,
                },
              },),
              nowMs: NOW_MS,
            },);

            expect(snapshots.length,).toBe(1,);
            expect(snapshots[0]?.usedPercent,).toBe(20,);
          },
        },),
        it({
          name: 'refuses a body carrying both weekly names when the two differ',
          fn: async function testSyntheticBothWeeklyNamesDiffer() {
            const snapshots = parseRateLimitSnapshots({
              headers: syntheticQuotasHeader({
                weeklyTokenLimit: {
                  nextRegenAt: resetAt(20 * HOUR_MS,),
                  percentRemaining: 80,
                },
                planWeeklyLimit: {
                  nextRegenAt: resetAt(20 * HOUR_MS,),
                  percentRemaining: 0,
                },
              },),
              nowMs: NOW_MS,
            },);

            expect(snapshots.length,).toBe(0,);
          },
        },),
        it({
          name: 'refuses a body carrying one weekly name that is no object beside a readable other',
          fn: async function testSyntheticWeeklyNameNotObject() {
            const snapshots = parseRateLimitSnapshots({
              headers: syntheticQuotasHeader({
                weeklyTokenLimit: {
                  nextRegenAt: resetAt(20 * HOUR_MS,),
                  percentRemaining: 80,
                },
                planWeeklyLimit: 'spent',
              },),
              nowMs: NOW_MS,
            },);

            expect(snapshots.length,).toBe(0,);
          },
        },),
        it({
          name: 'refuses a body carrying one weekly name that is no object beside a readable first name',
          fn: async function testSyntheticFirstWeeklyNameNotObject() {
            const snapshots = weeklySnapshots({
              weeklyTokenLimit: 'spent',
              planWeeklyLimit: {
                nextRegenAt: resetAt(20 * HOUR_MS,),
                percentRemaining: 80,
              },
            },);

            expect(snapshots.length,).toBe(0,);
          },
        },),
        it({
          name: 'reads planWeeklyLimit beside a null weeklyTokenLimit',
          fn: async function testSyntheticNullOldName() {
            const snapshots = weeklySnapshots({
              weeklyTokenLimit: null,
              planWeeklyLimit: {
                nextRegenAt: resetAt(20 * HOUR_MS,),
                percentRemaining: 80,
              },
            },);

            expect(snapshots.length,).toBe(1,);
            expect(snapshots[0]?.usedPercent,).toBe(20,);
          },
        },),
        it({
          name: 'reads weeklyTokenLimit beside a null planWeeklyLimit',
          fn: async function testSyntheticNullNewName() {
            const snapshots = weeklySnapshots({
              weeklyTokenLimit: {
                nextRegenAt: resetAt(20 * HOUR_MS,),
                percentRemaining: 80,
              },
              planWeeklyLimit: null,
            },);

            expect(snapshots.length,).toBe(1,);
            expect(snapshots[0]?.usedPercent,).toBe(20,);
          },
        },),
        it({
          name: 'refuses a body whose only weekly name is null',
          fn: async function testSyntheticOnlyNullWeeklyName() {
            const snapshots = weeklySnapshots({ weeklyTokenLimit: null, },);

            expect(snapshots.length,).toBe(0,);
          },
        },),
        it({
          name: 'refuses both weekly names that read the same percent and differ in nextRegenAt',
          fn: async function testSyntheticBothWeeklyNamesDifferInReset() {
            const snapshots = weeklySnapshots({
              weeklyTokenLimit: {
                nextRegenAt: resetAt(20 * HOUR_MS,),
                percentRemaining: 80,
              },
              planWeeklyLimit: {
                nextRegenAt: resetAt(21 * HOUR_MS,),
                percentRemaining: 80,
              },
            },);

            expect(snapshots.length,).toBe(0,);
          },
        },),
        it({
          name: 'refuses a weekly block whose percentRemaining is a string, under either name',
          fn: async function testSyntheticMistypedPercent() {
            const underNew = weeklySnapshots({
              planWeeklyLimit: { nextRegenAt: resetAt(20 * HOUR_MS,), percentRemaining: '80', },
            },);
            const underOld = weeklySnapshots({
              weeklyTokenLimit: { nextRegenAt: resetAt(20 * HOUR_MS,), percentRemaining: '80', },
            },);

            expect(underNew.length,).toBe(0,);
            expect(underOld.length,).toBe(0,);
          },
        },),
        it({
          name: 'refuses a weekly block whose nextRegenAt is a number, under either name',
          fn: async function testSyntheticMistypedReset() {
            const underNew = weeklySnapshots({
              planWeeklyLimit: { nextRegenAt: NOW_MS + HOUR_MS, percentRemaining: 80, },
            },);
            const underOld = weeklySnapshots({
              weeklyTokenLimit: { nextRegenAt: NOW_MS + HOUR_MS, percentRemaining: 80, },
            },);

            expect(underNew.length,).toBe(0,);
            expect(underOld.length,).toBe(0,);
          },
        },),
        it({
          name: 'ignores a mistyped nextRegenPercent',
          fn: async function testSyntheticMistypedRegenPercent() {
            const snapshots = weeklySnapshots({
              planWeeklyLimit: {
                nextRegenAt: resetAt(20 * HOUR_MS,),
                percentRemaining: 80,
                nextRegenPercent: 'two',
              },
            },);

            expect(snapshots.length,).toBe(1,);
            expect(snapshots[0]?.usedPercent,).toBe(20,);
          },
        },),
        it({
          name: 'keeps the search and free-tool windows when the weekly block is refused',
          fn: async function testSyntheticWeeklyRefusalKeepsOtherWindows() {
            const snapshots = weeklySnapshots({
              weeklyTokenLimit: { nextRegenAt: resetAt(20 * HOUR_MS,), percentRemaining: 80, },
              planWeeklyLimit: { nextRegenAt: resetAt(20 * HOUR_MS,), percentRemaining: 0, },
              search: {
                hourly: {
                  limit: 100,
                  requests: 60,
                  renewsAt: resetAt(30 * MINUTE_MS,),
                },
              },
              freeToolCalls: {
                limit: 50,
                requests: 10,
                renewsAt: resetAt(12 * HOUR_MS,),
              },
            },);

            expect(snapshots.map(function keyOf(snapshot,): string {
              return snapshot.key;
            },),).toEqual(['synthetic:search', 'synthetic:tools',],);
          },
        },),
        describe({
          name: 'refuses nextRegenAt that is not an ISO-8601 instant with a zone',
          children: REFUSED_RESET_TEXTS.map(function refusalCase(refused,) {
            return it({
              name: `refuses ${JSON.stringify(refused.text,)}: ${refused.why}`,
              fn: async function testSyntheticResetInstantRefused() {
                expect(weeklyResetSnapshots(refused.text,).length,).toBe(0,);
              },
            },);
          },),
        },),
        describe({
          name: 'refuses nextRegenAt with a field out of range, without help from Date.parse',
          children: OUT_OF_RANGE_RESET_TEXTS.map(function outOfRangeCase(refused,) {
            return it({
              name: `refuses ${JSON.stringify(refused.text,)}: ${refused.why}`,
              fn: async function testSyntheticResetFieldOutOfRange() {
                expect(parseWithEngineAcceptingAll(refused.text,).length,).toBe(0,);
              },
            },);
          },),
        },),
        describe({
          name: 'refuses nextRegenAt with a field out of range under the real Date.parse',
          children: OUT_OF_RANGE_RESET_TEXTS.map(function outOfRangeEngineCase(refused,) {
            return it({
              name: `refuses ${JSON.stringify(refused.text,)}: ${refused.why}`,
              fn: async function testSyntheticResetFieldOutOfRangeEngine() {
                expect(weeklyResetSnapshots(refused.text,).length,).toBe(0,);
              },
            },);
          },),
        },),
        describe({
          name: 'reads nextRegenAt spelled as an ISO-8601 instant with a zone',
          children: READ_RESET_TEXTS.map(function readCase(read,) {
            return it({
              name: `reads ${JSON.stringify(read.text,)} as ${read.instant}`,
              fn: async function testSyntheticResetInstantRead() {
                const snapshots = weeklyResetSnapshots(read.text,);

                expect(snapshots.length,).toBe(1,);
                expect(snapshots[0]?.resetAtMs,).toBe(Date.parse(read.instant,),);
              },
            },);
          },),
        },),
        it({
          name: 'applies the instant rule to the search and free-tool windows',
          fn: async function testSyntheticResetInstantOtherWindows() {
            const snapshots = weeklySnapshots({
              search: { hourly: { limit: 100, requests: 60, renewsAt: '2026', }, },
              freeToolCalls: { limit: 50, requests: 10, renewsAt: 'Oct 23 2026 05:03', },
            },);

            expect(snapshots.length,).toBe(0,);
          },
        },),
        it({
          name: 'logs an unreadable weekly field through the tagged debug logger and holds no value of the header',
          fn: async function testSyntheticUnreadableFieldLog() {
            await using logDirectory = await makeLogDirectory();
            const child = runNodeChild({
              executable: process.execPath,
              source: unreadableFieldChildSource(),
              logDirectory: logDirectory.path,
              timeoutMs: CHILD_TIMEOUT_MS,
            },);
            const lines = child.stderr.split('\n',).filter(function refusalLine(line,): boolean {
              return line.includes('refusing quotas',);
            },);

            expect(child.status,).toBe(0,);
            expect(lines.length,).toBe(1,);
            expect(lines[0]?.startsWith('[debug] [',),).toBe(true,);
            expect(lines[0]?.endsWith(
              '[pi-statusline:synthetic-quota-headers] [readSyntheticWeeklyReading] '
                + 'refusing quotas whose planWeeklyLimit has an unreadable percentRemaining or nextRegenAt',
            ),).toBe(true,);
            expect((child.stdout + child.stderr).includes(HEADER_VALUE_MARKER,),).toBe(false,);
          },
        },),
        it({
          name: 'fails a child that outlives its timeout with an error that names the timeout',
          fn: async function testChildTimeoutNamed() {
            await using logDirectory = await makeLogDirectory();

            expect(function runHungChild(): unknown {
              return runNodeChild({
                executable: process.execPath,
                source: 'setTimeout(function keepAlive() {}, 60_000);',
                logDirectory: logDirectory.path,
                timeoutMs: 500,
              },);
            },).toThrow('timed out after 500 ms',);
          },
        },),
        it({
          name: 'fails a child that cannot be spawned with the message of the spawn error',
          fn: async function testChildSpawnFailure() {
            await using logDirectory = await makeLogDirectory();

            expect(function runMissingExecutable(): unknown {
              return runNodeChild({
                executable: join(logDirectory.path, 'no-such-node',),
                source: '',
                logDirectory: logDirectory.path,
                timeoutMs: CHILD_TIMEOUT_MS,
              },);
            },).toThrow('ENOENT',);
          },
        },),
        it({
          name: 'ignores incomplete or invalid header groups',
          fn: async function testInvalidHeaders() {
            const snapshots = parseRateLimitSnapshots({
              headers: {
                'anthropic-ratelimit-tokens-limit': '100',
                'anthropic-ratelimit-tokens-remaining': 'not-a-number',
              },
              nowMs: NOW_MS,
            },);

            expect(snapshots.length,).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: projectUsagePercent.name,
      children: [
        it({
          name: 'returns zero without stable elapsed window data',
          fn: async function testProjectionSuppression() {
            const lowUsage = firstSnapshot(
              tokenHeaders({ limit: 100, remaining: 96, resetOffsetMs: 30 * SECOND_MS, },),
            );
            const futureReset = firstSnapshot(
              tokenHeaders({ limit: 100, remaining: 80, resetOffsetMs: 70 * SECOND_MS, },),
            );

            expect(projectUsagePercent({ snapshot: lowUsage, nowMs: NOW_MS, },),)
              .toBe(0,);
            expect(projectUsagePercent({ snapshot: futureReset, nowMs: NOW_MS, },),)
              .toBe(0,);
          },
        },),
        it({
          name: 'projects usage from current used percentage and reset time',
          fn: async function testProjectedUsage() {
            const snapshot = firstSnapshot(
              tokenHeaders({ limit: 100, remaining: 60, resetOffsetMs: 40 * SECOND_MS, },),
            );

            expect(projectUsagePercent({ snapshot, nowMs: NOW_MS, },),)
              .toBe(120,);
          },
        },),
      ],
    },),
    describe({
      name: formatUsageWarningSegment.name,
      children: [
        it({
          name: 'formats low remaining capacity when projection stays within the window',
          fn: async function testLowRemainingWarning() {
            const snapshot = firstSnapshot(
              tokenHeaders({ limit: 100, remaining: 20, resetOffsetMs: 10 * SECOND_MS, },),
            );

            expect(formatUsageWarningSegment({
              snapshot,
              nowMs: NOW_MS,
              style: PLAIN_USAGE_WARNING_STYLE,
            },),).toBe('anthropic tokens 20% left (10s)',);
          },
        },),
        it({
          name: 'formats projected overflow with overflow color',
          fn: async function testOverflowWarning() {
            const snapshot = firstSnapshot(
              tokenHeaders({ limit: 100, remaining: 60, resetOffsetMs: 40 * SECOND_MS, },),
            );

            expect(formatUsageWarningSegment({
              snapshot,
              nowMs: NOW_MS,
              style: ANGLE_STYLE,
            },),).toBe('anthropic tokens <red>60% left →120%</red> (40s)',);
          },
        },),
      ],
    },),
    describe({
      name: formatUsageWarningStatus.name,
      children: [
        it({
          name: 'renders Codex projected overflow',
          fn: async function testCodexOverflowStatus() {
            const current = formatUsageWarningStatus({
              headers: codexHeaders({
                usedPercent: 30,
                windowMinutes: 300,
                resetOffsetMs: 4 * HOUR_MS,
              },),
              nowMs: NOW_MS,
              style: PLAIN_USAGE_WARNING_STYLE,
            },);

            expect(current.statusText,).toBe('codex 5h 70% left →150% (4h)',);
          },
        },),
        it({
          name: 'renders Synthetic projected overflow',
          fn: async function testSyntheticOverflowStatus() {
            const current = formatUsageWarningStatus({
              headers: syntheticQuotasHeader({
                search: {
                  hourly: {
                    limit: 100,
                    requests: 60,
                    renewsAt: resetAt(30 * MINUTE_MS,),
                  },
                },
              },),
              nowMs: NOW_MS,
              style: PLAIN_USAGE_WARNING_STYLE,
            },);

            expect(current.statusText,).toBe('synthetic search 40% left →120% (30m)',);
          },
        },),
        it({
          name: 'renders Synthetic weekly projection using daily regen pace',
          fn: async function testSyntheticWeeklyOverflowStatus() {
            const current = formatUsageWarningStatus({
              headers: syntheticQuotasHeader({
                weeklyTokenLimit: {
                  nextRegenAt: resetAt(20 * HOUR_MS,),
                  percentRemaining: 80,
                  maxCredits: '$100.00',
                  remainingCredits: '$80.00',
                  nextRegenCredits: '$10.00',
                },
              },),
              nowMs: NOW_MS,
              style: PLAIN_USAGE_WARNING_STYLE,
            },);

            expect(current.statusText,).toBe('synthetic week 80% left →840% (20h)',);
          },
        },),
        it({
          name: 'joins multiple projected overflow groups',
          fn: async function testJoinedSegments() {
            const current = formatUsageWarningStatus({
              headers: {
                ...tokenHeaders({ limit: 100, remaining: 60, resetOffsetMs: 40 * SECOND_MS, },),
                ...codexHeaders({ usedPercent: 30, windowMinutes: 300, resetOffsetMs: 4 * HOUR_MS, },),
              },
              nowMs: NOW_MS,
              style: PLAIN_USAGE_WARNING_STYLE,
            },);

            expect(current.statusText,).toBe('anthropic tokens 60% left →120% (40s) · codex 5h 70% left →150% (4h)',);
          },
        },),
        it({
          name: 'returns empty status text when no projected overflow exists',
          fn: async function testEmptyStatus() {
            const current = formatUsageWarningStatus({
              headers: tokenHeaders({ limit: 100, remaining: 80, resetOffsetMs: 10 * SECOND_MS, },),
              nowMs: NOW_MS,
              style: PLAIN_USAGE_WARNING_STYLE,
            },);

            expect(current.statusText,).toBe('',);
          },
        },),
      ],
    },),
  ],
},);
