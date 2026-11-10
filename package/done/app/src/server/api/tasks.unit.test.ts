/**
 Failure reporting of the task REST handlers.

 The handlers live in a module that opens the database while it evaluates, so
 this file points `DB_PATH` at an in-memory database before the built
 `testing` entry loads and never reaches the package's data file.

 @module
 */

import {
  setTimeout as delay,
} from 'node:timers/promises';
import { HTTP_INTERNAL_SERVER_ERROR, } from '@monochromatic-dev/const-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Error text only the requested body stream can produce, so a log record
 carrying it proves the handler's catch block reported the value it caught
 rather than a validation failure.

 @example
 ```ts
 const request = failingBodyRequest('/api/tasks');
 ```
 */
const INJECTED_READ_FAILURE = 'injected body read failure';

/**
 Text `String(error)` renders for {@link INJECTED_READ_FAILURE}, which is what
 the 500 payload carries to the client.
 */
const EXPECTED_PAYLOAD_ERROR = `Error: ${INJECTED_READ_FAILURE}`;

/**
 Logging tag the task handler module reports failures under.
 */
const EXPECTED_LOG_TAG = '[done-task-api]';

/**
 Delay allowed for the logger to reach the console before an assertion reads
 it, in milliseconds.

 The console sink batches records onto a microtask queue and replays the ones
 logged before its sinks verified, so every step between the handler's catch
 block and `console.error` is a microtask: the delay only has to let that
 queue drain, which completes before the timer fires.
 */
const CONSOLE_FLUSH_DELAY_MS = 10;

//region Artifact loading

/**
 `DB_PATH` the test process inherited, restored once the artifact has resolved
 and cached its database path.
 */
const inheritedDatabasePath = process.env.DB_PATH;

// Set before the import: the module the artifact pulls in resolves its
// database path, connects, and migrates while that import evaluates.
process.env.DB_PATH = ':memory:';

/**
 Handlers under test, loaded from the artifact this package builds.
 */
const {
  handleCreateTask,
  handleUpdateTask,
} = await import('../../../dist/final/node/testing.mjs');

if (inheritedDatabasePath === undefined) {
  delete process.env.DB_PATH;
}
else {
  process.env.DB_PATH = inheritedDatabasePath;
}

//endregion Artifact loading

//region Log capture

/**
 Structural view of the console-error capture a test installs, so this file
 needs no dependency on the spy library's types.
 */
type ConsoleErrorCalls = {
  readonly callCount: number;
  readonly getCalls: () => readonly { readonly args: readonly unknown[]; }[];
};

/**
 Request initialization this runtime honours, including the `duplex` field a
 streaming body requires that the DOM `RequestInit` declaration omits.
 */
type StreamingRequestInit = RequestInit & { readonly duplex: 'half'; };

/**
 Builds a POST request whose body stream fails while the handler reads it.
 The handler therefore fails after validation, in the same catch block a
 database failure reaches.

 @param path - API path the handler answers.

 @returns Request whose `json()` rejects with {@link INJECTED_READ_FAILURE}.
 */
function failingBodyRequest(path: string,): Request {
  /**
   Stream that errors on the first pull, standing in for a truncated upload.
   */
  const failingBody = new ReadableStream({
    pull(controller,) {
      controller.error(new Error(INJECTED_READ_FAILURE,),);
    },
  },);
  /**
   Initialization including the streaming `duplex` mode node requires.
   */
  const requestInit: StreamingRequestInit = {
    method: 'POST',
    body: failingBody,
    duplex: 'half',
  };
  return new Request(`http://localhost${path}`, requestInit,);
}

/**
 Renders one recorded console argument as text.

 @param value - First argument the sink recorded.

 @returns The string itself, or its JSON form for a non-string argument.
 */
function recordedArgumentText(value: unknown,): string {
  if ((typeof value) === 'string')
    return value;
  return JSON.stringify(value,) ?? '';
}

/**
 Renders every captured console-error call as one text blob.

 @param calls - Console-error capture to read.

 @returns First argument of each call, one per line.
 */
function consoleRecordText(calls: ConsoleErrorCalls,): string {
  return calls.getCalls()
    .map(function firstArgument(call,) {
      return recordedArgumentText(call.args[0],);
    },)
    .join('\n',);
}

//endregion Log capture

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: handleCreateTask.name,
      children: [
        it({
          name: 'answers 500 with the caught error and logs it under the module tag',
          fn: async (ctx,) => {
            /**
             Console-error capture standing in for the log sink.
             */
            const consoleError = ctx.sinon.spy(console, 'error',);
            /**
             Response the failing body produces.
             */
            const response = await handleCreateTask(failingBodyRequest('/api/tasks',),);
            expect(response.status,).toBe(HTTP_INTERNAL_SERVER_ERROR,);
            /**
             JSON payload the client receives; unchanged by the logging.
             */
            const payload: unknown = await response.json();
            expect(payload,).toEqual({ error: EXPECTED_PAYLOAD_ERROR, },);

            await delay(CONSOLE_FLUSH_DELAY_MS,);
            /**
             Logged text produced by the handler's catch block.
             */
            const loggedText = consoleRecordText(consoleError,);
            expect(consoleError.callCount,).toBe(1,);
            expect(loggedText,).toContain(INJECTED_READ_FAILURE,);
            expect(loggedText,).toContain(EXPECTED_LOG_TAG,);
          },
        },),
      ],
    },),
    describe({
      name: handleUpdateTask.name,
      children: [
        it({
          name: 'answers 500 with the caught error and logs it under the module tag',
          fn: async (ctx,) => {
            /**
             Task id the failing update names.
             */
            const taskId = 'aa11bb22-cc33-dd44-ee55-ff6677889900';
            /**
             Console-error capture standing in for the log sink.
             */
            const consoleError = ctx.sinon.spy(console, 'error',);
            const response = await handleUpdateTask({
              req: failingBodyRequest(`/api/tasks/${taskId}`,),
              id: taskId,
            },);
            expect(response.status,).toBe(HTTP_INTERNAL_SERVER_ERROR,);
            const payload: unknown = await response.json();
            expect(payload,).toEqual({ error: EXPECTED_PAYLOAD_ERROR, },);

            await delay(CONSOLE_FLUSH_DELAY_MS,);
            const loggedText = consoleRecordText(consoleError,);
            expect(consoleError.callCount,).toBe(1,);
            expect(loggedText,).toContain(INJECTED_READ_FAILURE,);
            expect(loggedText,).toContain(EXPECTED_LOG_TAG,);
            expect(loggedText,).toContain(taskId,);
          },
        },),
      ],
    },),
  ],
},);
