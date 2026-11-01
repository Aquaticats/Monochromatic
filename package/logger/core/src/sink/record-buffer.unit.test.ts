import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  _createRecordBuffer as createRecordBuffer,
  type Level,
} from '@monochromatic-dev/logger-core';

/**
 Builds a capturing backend handoff so a test can assert exactly which
 batches left the buffer and in what shape.
 
 @returns Handoff function plus the captured batch list.
 */
function createCapturingFlush(): {
  readonly batches: string[];
  readonly onFlush: (batch: string,) => void;
} {
  const batches: string[] = [];
  return {
    batches,
    onFlush: function captureBatch(batch: string,): void {
      batches.push(batch,);
    },
  };
}

/**
 Installs a capturing `globalThis.addEventListener` (absent under Node, so
 the buffer's lifecycle registration is otherwise a no-op there), restoring
 the prior value when the returned guard leaves `using` scope.
 
 @returns Disposable exposing captured handlers by event type.
 */
function installFakeGlobalListeners(): Disposable & {
  readonly handlers: Map<string, () => void>;
} {
  const original = globalThis.addEventListener;
  const handlers = new Map<string, () => void>();
  globalThis.addEventListener = (function captureListener(
    type: string,
    handler: () => void,
  ): void {
    handlers.set(type, handler,);
  }) as unknown as typeof globalThis.addEventListener;
  return {
    handlers,
    [Symbol.dispose](): void {
      globalThis.addEventListener = original;
    },
  };
}

/**
 Installs a fake `globalThis.document` (absent under Node) whose
 `visibilityState` is controllable and whose `addEventListener` captures
 handlers, restoring the prior value when the returned guard leaves `using`
 scope.
 
 @returns Disposable exposing captured handlers and mutable visibility.
 */
function installFakeDocument(): Disposable & {
  readonly handlers: Map<string, () => void>;
  readonly visibility: { state: string; };
} {
  const original = globalThis.document;
  const handlers = new Map<string, () => void>();
  const visibility = { state: 'visible', };
  globalThis.document = {
    addEventListener: function captureListener(
      type: string,
      handler: () => void,
    ): void {
      handlers.set(type, handler,);
    },
    get visibilityState(): string {
      return visibility.state;
    },
  } as unknown as Document;
  return {
    handlers,
    visibility,
    [Symbol.dispose](): void {
      globalThis.document = original;
    },
  };
}

/**
 Severities the buffer must flush synchronously on `add`.
 */
const URGENT_LEVELS: readonly Level[] = ['warn', 'error', 'fatal',];

/**
 Severities the buffer must keep buffered on `add`.
 */
const ROUTINE_LEVELS: readonly Level[] = ['trace', 'debug', 'info',];

// Serial because the lifecycle tests swap `globalThis` members, and because
// every buffer registers a process-wide pagehide listener, so a concurrent
// dispatch could drain a sibling test's buffer mid-assertion.
await describe({
  name: createRecordBuffer.name,
  concurrency: 1,
  children: [
    it({
      name: 'a routine record stays buffered until drain, then leaves verbatim',
      fn: async () => {
        const { batches, onFlush, } = createCapturingFlush();
        const buffer = createRecordBuffer({ onFlush, },);

        buffer.add({ level: 'info', serialized: 'r1', },);
        expect(batches.length,)
          .toBe(0,);

        buffer.drain();
        expect(batches.join('|',),)
          .toBe('r1',);
      },
    },),

    it({
      name: 'drain joins buffered records with newlines in add order',
      fn: async () => {
        const { batches, onFlush, } = createCapturingFlush();
        const buffer = createRecordBuffer({ onFlush, },);

        buffer.add({ level: 'info', serialized: 'r1', },);
        buffer.add({ level: 'debug', serialized: 'r2', },);
        buffer.add({ level: 'trace', serialized: 'r3', },);
        buffer.drain();

        expect(batches.join('|',),)
          .toBe('r1\nr2\nr3',);
      },
    },),

    ...URGENT_LEVELS.map(function mapUrgent(level,) {
      return it({
        name: `a ${level} record flushes itself and everything buffered ahead of it`,
        fn: async () => {
          const { batches, onFlush, } = createCapturingFlush();
          const buffer = createRecordBuffer({ onFlush, },);

          buffer.add({ level: 'info', serialized: 'ahead', },);
          buffer.add({ level, serialized: 'urgent', },);

          expect(batches.join('|',),)
            .toBe('ahead\nurgent',);
        },
      },);
    },),

    ...ROUTINE_LEVELS.map(function mapRoutine(level,) {
      return it({
        name: `a ${level} record does not flush on add`,
        fn: async () => {
          const { batches, onFlush, } = createCapturingFlush();
          const buffer = createRecordBuffer({ onFlush, },);

          buffer.add({ level, serialized: 'routine', },);
          expect(batches.length,)
            .toBe(0,);

          // Leave nothing armed behind for later tests.
          buffer.drain();
        },
      },);
    },),

    it({
      name: 'reaching the byte cap flushes synchronously from inside add',
      fn: async () => {
        const { batches, onFlush, } = createCapturingFlush();
        const buffer = createRecordBuffer({ onFlush, },);

        // A single record past the 32 KiB cap leaves immediately on add.
        buffer.add({ level: 'info', serialized: 'L'.repeat(40_000,), },);
        expect(batches.length,)
          .toBe(1,);
      },
    },),

    it({
      name: 'an addition that would breach the cap flushes existing entries first',
      fn: async () => {
        const { batches, onFlush, } = createCapturingFlush();
        const buffer = createRecordBuffer({ onFlush, },);

        buffer.add({ level: 'info', serialized: 'small', },);
        buffer.add({ level: 'info', serialized: 'L'.repeat(40_000,), },);

        // Two separate batches: the small record is never a batch-mate of the
        // cap-breaching one.
        expect(batches.length,)
          .toBe(2,);
        expect(batches[0],)
          .toBe('small',);
      },
    },),

    it({
      name: 'the quiet-period deadline drains without any trigger call',
      fn: async () => {
        const { batches, onFlush, } = createCapturingFlush();
        const buffer = createRecordBuffer({ onFlush, },);

        buffer.add({ level: 'debug', serialized: 'idle', },);

        /**
         Comfortably past the buffer's 250 ms quiet-period deadline.
         */
        const pastDeadlineMs = 400;
        await wait(pastDeadlineMs,);

        expect(batches.join('|',),)
          .toBe('idle',);
      },
      timeout: 5_000,
    },),

    it({
      name: 'drain on an empty buffer is a no-op',
      fn: async () => {
        const { batches, onFlush, } = createCapturingFlush();
        const buffer = createRecordBuffer({ onFlush, },);

        buffer.drain();
        buffer.drain();
        expect(batches.length,)
          .toBe(0,);
      },
    },),

    it({
      name: 'registers a pagehide listener that drains the buffer',
      fn: async () => {
        using listeners = installFakeGlobalListeners();
        const { batches, onFlush, } = createCapturingFlush();
        const buffer = createRecordBuffer({ onFlush, },);

        buffer.add({ level: 'info', serialized: 'leaving', },);
        /**
         Captured pagehide handler; the buffer must have registered one.
         */
        const onPagehide = listeners.handlers
          .get('pagehide',);
        expect((typeof onPagehide) === 'function',)
          .toBe(true,);

        onPagehide?.();
        expect(batches.join('|',),)
          .toBe('leaving',);
      },
    },),

    it({
      name: 'registers a visibilitychange listener that drains only when hidden',
      fn: async () => {
        using fakeDocument = installFakeDocument();
        const { batches, onFlush, } = createCapturingFlush();
        const buffer = createRecordBuffer({ onFlush, },);

        buffer.add({ level: 'info', serialized: 'tabbed away', },);
        /**
         Captured visibilitychange handler; the buffer must have registered one.
         */
        const onVisibilityChange = fakeDocument.handlers
          .get('visibilitychange',);
        expect((typeof onVisibilityChange) === 'function',)
          .toBe(true,);

        // Still visible: the handler must not drain.
        onVisibilityChange?.();
        expect(batches.length,)
          .toBe(0,);

        fakeDocument.visibility.state = 'hidden';
        onVisibilityChange?.();
        expect(batches.join('|',),)
          .toBe('tabbed away',);
      },
    },),
  ],
},);
