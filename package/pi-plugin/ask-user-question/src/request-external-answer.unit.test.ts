import { once, } from 'node:events';
import { writeFile, } from 'node:fs/promises';
import { createConnection, } from 'node:net';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createRequestRegistry,
  type HelperCompletion,
  readHelperRequest,
  requestExternalAnswer,
  serializeHelperCompletion,
  type AnswerTerminalLauncher,
} from '../dist/final/node/index.mjs';

//region Fake helper

/**
 Builds terminal launcher that completes private helper protocol in-process.
 
 @param answer - answer text written before completion
 
 @param completion - helper completion frame;
 undefined disconnects after authentication
 
 @returns injectable terminal launcher
 */
function respondingLauncher(
  {
    answer,
    completion,
    expectedEditorCommand,
    precedeWithWrongToken = false,
  }: {
    readonly answer: string;
    readonly completion?: HelperCompletion;
    readonly expectedEditorCommand?: readonly string[];
    readonly precedeWithWrongToken?: boolean;
  },
): AnswerTerminalLauncher {
  return async function launchFakeTerminal({ command, },): Promise<void> {
    /**
     Request path is final helper argument.
     */
    const requestPath = command.at(-1,);
    if (requestPath === undefined)
      throw new Error('Fake terminal did not receive helper request path.',);
    /**
     Private helper request produced by extension.
     */
    const request = await readHelperRequest({ requestPath, },);
    if (expectedEditorCommand !== undefined)
      expect(request.editorCommand,)
        .toEqual(expectedEditorCommand,);
    await writeFile(
      request.answerPath,
      answer,
      'utf8',
    );
    if (precedeWithWrongToken) {
      /**
       Unauthenticated local candidate rejected before real helper.
       */
      const rejected = createConnection({
        host: request.host,
        port: request.port,
      },);
      await once(
        rejected,
        'connect',
      );
      /**
       Rejected-socket close subscribed before ending stream.
       */
      const rejectedClosed = once(
        rejected,
        'close',
      );
      rejected.end('wrong-token\n',);
      await rejectedClosed;
    }
    /**
     Authenticated helper socket.
     */
    const socket = createConnection({
      host: request.host,
      port: request.port,
    },);
    await once(
      socket,
      'connect',
    );
    // Drain the requester's acknowledgement while sending fixture completion.
    socket.resume();
    /**
     Helper close subscribed before ending stream.
     */
    const closed = once(
      socket,
      'close',
    );
    socket.end([
      `${request.token}\n`,
      completion === undefined
        ? ''
        : serializeHelperCompletion({ completion, }),
    ].join('',),);
    await closed;
  };
}

//endregion Fake helper

await describe({
  name: requestExternalAnswer.name,
  children: [
    it({
      name: 'returns multiline answer after rejecting wrong token and removing one CRLF',
      fn: async () => {
        /**
         Session-scoped registry for test request.
         */
        const registry = createRequestRegistry();
        expect(await requestExternalAnswer({
          cwd: process.cwd(),
          registry,
          editorCommand: ['nano',],
          launch: respondingLauncher({
            answer: 'first\r\nsecond\r\n',
            completion: { status: 'submitted', },
            expectedEditorCommand: ['nano',],
            precedeWithWrongToken: true,
          },),
        },),)
          .toEqual({
            status: 'answered',
            answer: 'first\r\nsecond',
          },);
      },
    },),
    it({
      name: 'warns when resolved terminal and editor are Ghostty with Helix',
      fn: async () => {
        /**
         Captured compatibility warning messages.
         */
        const warnings: string[] = [];
        await requestExternalAnswer({
          cwd: process.cwd(),
          registry: createRequestRegistry(),
          editorCommand: ['hx',],
          resolveTerminalEntryId: async () => 'com.mitchellh.ghostty.desktop',
          warn: function captureWarning(message: string,): void {
            warnings.push(message,);
          },
          launch: respondingLauncher({
            answer: 'answer',
            completion: { status: 'submitted', },
          },),
        },);
        expect(warnings,)
          .toHaveLength(1,);
        expect(warnings[0],)
          .toContain('Ghostty',);
        expect(warnings[0],)
          .toContain('Helix',);
      },
    },),
    it({
      name: 'maps blank submitted file to cancellation',
      fn: async () => {
        /**
         Session-scoped registry for test request.
         */
        const registry = createRequestRegistry();
        expect(await requestExternalAnswer({
          cwd: process.cwd(),
          registry,
          launch: respondingLauncher({
            answer: ' \n',
            completion: { status: 'submitted', },
          },),
        },),)
          .toEqual({ status: 'cancelled', },);
      },
    },),
    it({
      name: 'maps authenticated disconnect to cancellation',
      fn: async () => {
        /**
         Session-scoped registry for test request.
         */
        const registry = createRequestRegistry();
        expect(await requestExternalAnswer({
          cwd: process.cwd(),
          registry,
          launch: respondingLauncher({
            answer: '',
          },),
        },),)
          .toEqual({ status: 'cancelled', },);
      },
    },),
    it({
      name: 'maps explicit helper cancellation',
      fn: async () => {
        /**
         Session-scoped registry for test request.
         */
        const registry = createRequestRegistry();
        expect(await requestExternalAnswer({
          cwd: process.cwd(),
          registry,
          launch: respondingLauncher({
            answer: '',
            completion: { status: 'cancelled', },
          },),
        },),)
          .toEqual({ status: 'cancelled', },);
      },
    },),
    it({
      name: 'throws helper operational error',
      fn: async () => {
        /**
         Captured helper error.
         */
        const caught: { value?: unknown; } = {};
        try {
          await requestExternalAnswer({
            cwd: process.cwd(),
            registry: createRequestRegistry(),
            launch: respondingLauncher({
              answer: '',
              completion: {
                status: 'error',
                message: 'editor unavailable',
              },
            },),
          },);
        }
        catch (error: unknown) {
          caught.value = error;
        }
        expect(caught.value,)
          .toBeInstanceOf(Error,);
        if (!Error.isError(caught.value,))
          throw new Error('Expected helper operational Error.',);
        expect(caught.value.message,)
          .toContain('editor unavailable',);
      },
    },),
    it({
      name: 'propagates tool cancellation while waiting for helper',
      fn: async () => {
        /**
         Tool cancellation controlled by fake launcher.
         */
        const controller = new AbortController();
        /**
         Captured cancellation failure.
         */
        const caught: { value?: unknown; } = {};
        try {
          await requestExternalAnswer({
            cwd: process.cwd(),
            registry: createRequestRegistry(),
            signal: controller.signal,
            launch: async () => {
              controller.abort();
            },
          },);
        }
        catch (error: unknown) {
          caught.value = error;
        }
        expect(caught.value,)
          .toBeInstanceOf(Error,);
      },
    },),
    it({
      name: 'session shutdown during terminal resolution prevents a later launch',
      fn: async () => {
        const registry = createRequestRegistry();
        const state = { launched: false, };
        let caught: unknown;
        try {
          await requestExternalAnswer({
            cwd: process.cwd(),
            registry,
            resolveTerminalEntryId: async () => {
              registry.abortAll();
              return 'fixture-terminal';
            },
            launch: async () => {
              state.launched = true;
              throw new Error('An already-cancelled request launched a terminal.',);
            },
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect(state.launched,).toBe(false,);
      },
    },),
    it({
      name: 'propagates terminal launch failure without hanging channel',
      fn: async () => {
        /**
         Captured launch failure.
         */
        const caught: { value?: unknown; } = {};
        try {
          await requestExternalAnswer({
            cwd: process.cwd(),
            registry: createRequestRegistry(),
            launch: async () => {
              throw new Error('terminal launch refused',);
            },
          },);
        }
        catch (error: unknown) {
          caught.value = error;
        }
        expect(caught.value,)
          .toBeInstanceOf(Error,);
        if (!Error.isError(caught.value,))
          throw new Error('Expected terminal launch Error.',);
        expect(caught.value.message,)
          .toContain('terminal launch refused',);
      },
    },),
  ],
},);
