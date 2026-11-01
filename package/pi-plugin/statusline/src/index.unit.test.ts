import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import statusline, { STATUS_KEY, } from './index.ts';
import {
  createAfterProviderResponseEvent,
  createExtensionContext,
  createSessionStartEvent,
  fakePiApi,
  getAfterProviderResponseHandler,
  getSessionStartHandler,
} from './pi-test-harness.ts';

const NOW_MS = Date.parse('2026-06-01T12:00:00Z',);
const SECOND_MS = 1_000;

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
    'anthropic-ratelimit-tokens-reset': new Date(NOW_MS + resetOffsetMs,).toISOString(),
  };
}

type DateNowRestore = {
  readonly [Symbol.dispose]: () => void;
};

function freezeDateNow(nowMs: number,): DateNowRestore {
  const originalDateNow = Date.now;
  Date.now = function now(): number {
    return nowMs;
  };

  return {
    [Symbol.dispose]: function restoreDateNow(): void {
      Date.now = originalDateNow;
    },
  };
}

async function withFrozenNow(fn: () => Promise<void>,): Promise<void> {
  using frozenDateNow = freezeDateNow(NOW_MS,);
  void frozenDateNow;
  await fn();
}

await describe({
  name: statusline.name,
  children: [
    it({
      name: 'registers provider and session handlers',
      fn: async function testRegistersHandlers() {
        const harness = fakePiApi();
        statusline(harness.api,);

        expect(harness.registrations,).toContain('event:session_start',);
        expect(harness.registrations,).toContain('event:session_shutdown',);
        expect(harness.registrations,).toContain('event:after_provider_response',);
      },
    },),
    it({
      name: 'sets themed status for projected overflow usage',
      fn: async function testSetsWarningStatus() {
        await withFrozenNow(async function runWithFrozenNow() {
          const harness = fakePiApi();
          const { ctx, statuses, } = createExtensionContext();
          statusline(harness.api,);

          const handler = getAfterProviderResponseHandler(harness.afterProviderResponseHandlers,);
          await handler(
            createAfterProviderResponseEvent(
              tokenHeaders({ limit: 100, remaining: 60, resetOffsetMs: 40 * SECOND_MS, },),
            ),
            ctx,
          );

          expect(statuses.get(STATUS_KEY,),).toBe('anthropic tokens <error>60% left →120%</error> (40s)',);
        },);
      },
    },),
    it({
      name: 'clears status when later usage is comfortable',
      fn: async function testClearsComfortableStatus() {
        await withFrozenNow(async function runWithFrozenNow() {
          const harness = fakePiApi();
          const { ctx, statuses, } = createExtensionContext();
          statusline(harness.api,);

          const handler = getAfterProviderResponseHandler(harness.afterProviderResponseHandlers,);
          await handler(
            createAfterProviderResponseEvent(
              tokenHeaders({ limit: 100, remaining: 60, resetOffsetMs: 40 * SECOND_MS, },),
            ),
            ctx,
          );
          await handler(
            createAfterProviderResponseEvent(
              tokenHeaders({ limit: 100, remaining: 80, resetOffsetMs: 10 * SECOND_MS, },),
            ),
            ctx,
          );

          expect(statuses.get(STATUS_KEY,),).toBe(undefined,);
        },);
      },
    },),
    it({
      name: 'does not write footer status when UI is unavailable',
      fn: async function testNoUiNoStatusWrite() {
        await withFrozenNow(async function runWithFrozenNow() {
          const harness = fakePiApi();
          const { ctx, statuses, } = createExtensionContext(false,);
          statusline(harness.api,);

          const handler = getAfterProviderResponseHandler(harness.afterProviderResponseHandlers,);
          await handler(
            createAfterProviderResponseEvent(
              tokenHeaders({ limit: 100, remaining: 60, resetOffsetMs: 40 * SECOND_MS, },),
            ),
            ctx,
          );

          expect(statuses.size,).toBe(0,);
        },);
      },
    },),
    it({
      name: 'clears status on session start',
      fn: async function testSessionStartClearsStatus() {
        await withFrozenNow(async function runWithFrozenNow() {
          const harness = fakePiApi();
          const { ctx, statuses, } = createExtensionContext();
          statusline(harness.api,);

          const responseHandler = getAfterProviderResponseHandler(harness.afterProviderResponseHandlers,);
          await responseHandler(
            createAfterProviderResponseEvent(
              tokenHeaders({ limit: 100, remaining: 60, resetOffsetMs: 40 * SECOND_MS, },),
            ),
            ctx,
          );

          const sessionStartHandler = getSessionStartHandler(harness.sessionStartHandlers,);
          await sessionStartHandler(
            createSessionStartEvent(),
            ctx,
          );

          expect(statuses.get(STATUS_KEY,),).toBe(undefined,);
        },);
      },
    },),
  ],
},);
