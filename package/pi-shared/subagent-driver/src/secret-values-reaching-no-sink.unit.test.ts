/**
 The controls that drive Pi's own installed code, not a stand-in of this package's: a tool result, the message Pi keeps, the
 driver's feed record, and the compaction request Pi builds from the session.

 Every value here is a stand-in this file makes up, and every file it writes is under the scratch folder the test harness names.

 @module
 */

import { createRequire, } from 'node:module';
import { realpathSync, } from 'node:fs';
import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  fileURLToPath,
  pathToFileURL,
} from 'node:url';
import {
  AgentSession,
  ExtensionRunner,
  SessionManager,
} from '@earendil-works/pi-coding-agent';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  armSecretValueMasking,
  createSecretValueMasking,
  maskedValueMarker,
  MASKING_NOTHING,
  maskedToolResult,
  startRecordOfPiRun,
  type SecretValue,
  type TextMasking,
} from '../dist/final/node/index.mjs';

/**
 The stand-in value every test here arms with.
 */
const SECRET = 'stand-in-hook-value-0123456789';

/**
 The name a marker says in its place.
 */
const SECRET_NAME = 'STAND_IN_HOOK_API_KEY';

/**
 What Pi's own `message_end` answers when no handler changed the message, which this test turns into a failure.
 */
const NOTHING_CHANGED: unique symbol = Symbol('message kept unchanged',);

/**
 The one stand-in value every test arms with.
 */
const VALUES: readonly SecretValue[] = [{ name: SECRET_NAME, value: SECRET, },];

/**
 Pi's own tool-call loop, which is where a tool result is made.

 `@earendil-works/pi-agent-core` is Pi's own dependency, so it is resolved from Pi's entry rather than from this package.
 */
async function piToolCallLoop() {
  /**
   Where Pi's own session code is, which is what its dependencies resolve against.
   */
  const agentSessionEntry = new URL(
    '../node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session.js',
    import.meta.url,
  );
  /**
   A resolver that starts from Pi's own folder.
   */
  const requireFromPi = createRequire(
    realpathSync(fileURLToPath(agentSessionEntry,),),
  );
  /**
   Where Pi's tool-call loop is.
   */
  const corePackage = requireFromPi.resolve('@earendil-works/pi-agent-core/package.json',);
  return await import(new URL('dist/index.js', pathToFileURL(corePackage,),).href,) as {
    readonly runToolCall: (
      toolCall: unknown,
      options: unknown,
    ) => Promise<{ readonly result: { readonly content: readonly unknown[]; readonly details: unknown; readonly structuredContent?: unknown }; readonly isError: boolean }>;
  };
}

/**
 What one arming registered, and the runner Pi would build from it.
 */
function armedRunner() {
  /**
   The handlers the extension registered, by the event.
   */
  const handlers = new Map<string, ((event: never,) => unknown)[]>();
  /**
   The extension API Pi hands an extension, as far as this extension uses it.
   */
  const api = {
    on: function on(event: string, handler: (event: never,) => unknown,) {
      handlers.set(
        event,
        [...(handlers.get(event,) ?? []), handler,],
      );
      return function off(): void {
        handlers.delete(event,);
      };
    },
    registerCommand: function registerCommand() {
      // A command list is a start's business, not a hook's.
    },
  } as unknown as Parameters<typeof armSecretValueMasking>[0]['pi'];
  armSecretValueMasking({
    pi: api,
    values: VALUES,
    noteFailure: function note() {
      // A failure would come from a file this test names, and the assertions below are the ones that matter.
    },
  },);
  /**
   The runner Pi reads its handlers from, which is Pi's own class with the members its hooks read.
   */
  const runner: ExtensionRunner = Object.assign(
    Object.create(ExtensionRunner.prototype,) as ExtensionRunner,
    {
      extensions: [{ path: 'stand-in-extension', handlers, },],
      createContext: function createContext() {
        return {};
      },
      emitError: function emitError() {
        // Nothing to report: every event here is one the extension handles.
      },
    },
  );
  return { handlers, runner, };
}

/**
 A tool that returns one result, whatever it is given.

 @param result - the result the tool returns

 @returns the tool Pi's loop calls
 */
function toolReturning(result: unknown,) {
  return {
    name: 'stand_in_tool',
    description: 'stand-in tool',
    parameters: { type: 'object', properties: {}, },
    execute: async function execute() {
      return result;
    },
  };
}

/**
 Runs one tool call through Pi's own loop and Pi's own `_afterToolCall`.

 @param result - what the tool returns

 @returns what Pi finalized
 */
async function throughRealToolCall(result: unknown,) {
  /**
   Pi's own tool-call loop.
   */
  const { runToolCall, } = await piToolCallLoop();
  /**
   What the arming registered, whose runner Pi reads its hooks from.
   */
  const { runner, } = armedRunner();
  /**
   The session Pi's hook reads its limits and settings from, as far as `_afterToolCall` uses them.
   */
  const session = {
    _extensionRunner: runner,
    settingsManager: { getImageAutoResize: () => false, },
    _limitsModel: () => undefined,
  };
  /**
   The tool the call runs.
   */
  const tool = toolReturning(result,);
  return await runToolCall(
    {
      name: tool.name,
      id: 'stand-in-call',
      arguments: {},
    },
    {
      tools: [tool,],
      assistantMessage: {},
      context: { messages: [], tools: [tool,], },
      afterToolCall: async function afterToolCall(context: unknown,) {
        // Pi's own tool-result hook, which its session owns and its tool loop calls back into.
        const afterToolCallOfPi = (AgentSession.prototype as unknown as { _afterToolCall: (this: unknown, context: unknown) => Promise<unknown> })._afterToolCall;
        return await afterToolCallOfPi.call(
          session,
          context as never,
        );
      },
    },
  ) as { readonly result: { readonly content: readonly unknown[]; readonly details: unknown; readonly structuredContent?: unknown }; readonly isError: boolean };
}

/**
 Reads a session's file, which Pi names only once it has one.

 @param session - the session manager Pi keeps messages in

 @returns the session file's text
 */
async function textOfSession({ session, }: { readonly session: SessionManager },): Promise<string> {
  /**
   Where Pi keeps the session.
   */
  const file = session.getSessionFile();
  if (file === undefined) {
    throw new Error('the session Pi was given kept no file',);
  }
  return await readFile(
    file,
    'utf8',
  );
}

/**
 A folder this file makes, removed when it is done.

 @returns the folder's path and its disposer
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder, under the scratch root the test harness names.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'pi-hooks-',
  ),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(
        path,
        { recursive: true, force: true, },
      );
    },
  };
}

await describe({
  name: 'a value of this machine never reaching a sink Pi writes',
  children: [
    it({
      name: 'masks a tool result details through Pi own tool loop, the driver feed and the session file',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         What Pi finalized for a tool that put the value in its details.
         */
        const finalized = await throughRealToolCall({
          content: [{ type: 'text', text: 'ordinary result', },],
          details: { credential: SECRET, },
        },);
        expect(JSON.stringify(finalized.result.details,),).not.toContain(SECRET,);
        expect(JSON.stringify(finalized.result.content,),).not.toContain(SECRET,);
        /**
         The masking the driver writes its own files through.
         */
        const masking: TextMasking = createSecretValueMasking({ values: VALUES, },);
        /**
         The masking a run without values writes with, which is what the record defaults to.
         */
        expect(MASKING_NOTHING.maskText('unchanged',),).toBe('unchanged',);
        /**
         The run's feed, which reads the same event Pi keeps.
         */
        const record = startRecordOfPiRun({
          feedFile: join(folder.path, 'feed.jsonl',),
          mask: masking,
        },);
        record.observe({
          event: {
            type: 'tool_execution_end',
            toolCallId: 'stand-in-call',
            toolName: 'stand_in_tool',
            result: finalized.result,
            isError: false,
          },
          declineDialog: function declineDialog() {
            // A tool result asks no dialog.
          },
        },);
        await record.close();
        expect(await readFile(
          join(folder.path, 'feed.jsonl',),
          'utf8',
        ),).not.toContain(SECRET,);
        /**
         The session Pi keeps its messages in.
         */
        const session = SessionManager.create(
          folder.path,
          join(folder.path, 'sessions',),
        );
        session.appendMessage({ role: 'user', content: 'safe prompt', timestamp: 0, },);
        session.appendMessage({
          role: 'toolResult',
          toolCallId: 'stand-in-call',
          toolName: 'stand_in_tool',
          content: [...finalized.result.content,] as never,
          details: finalized.result.details as never,
          isError: false,
          timestamp: 0,
        },);
        expect(await textOfSession({ session, },),).not.toContain(SECRET,);
      },
    },),
    it({
      name: 'masks the message Pi keeps, which is what its compaction request is built from',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         What the arming registered, whose runner masks the message.
         */
        const { runner, } = armedRunner();
        /**
         A user message holding the value, as the run's first prompt would.
         */
        const message = {
          role: 'user' as const,
          content: `the environment holds ${SECRET}`,
          timestamp: 0,
        };
        /**
         What Pi's own `message_end` hooks answered: the message Pi keeps, which is the replacement applied where the original
         stood.
         */
        // Pi's own `message_end` answers nothing when no handler changed anything, which is the absence this test calls a failure.
        const answer = await ExtensionRunner.prototype.emitMessageEnd.call(
          runner,
          { type: 'message_end', message, },
        ) ?? NOTHING_CHANGED;
        if (answer === NOTHING_CHANGED) {
          throw new Error('Pi kept the message it had, so nothing was masked',);
        }
        /**
         The message Pi keeps.
         */
        const kept = answer;
        expect(JSON.stringify(kept,),).not.toContain(SECRET,);
        expect(JSON.stringify(kept,),).toContain(maskedValueMarker(SECRET_NAME,),);
        /**
         The session the message is kept in.
         */
        const session = SessionManager.create(
          folder.path,
          join(folder.path, 'sessions',),
        );
        session.appendMessage(kept as never,);
        expect(await textOfSession({ session, },),).not.toContain(SECRET,);
        /**
         What Pi's compaction request carried, which Pi builds from the session and sends without the context hook.
         */
        const { generateSummaryWithUsage, } = await import('@earendil-works/pi-coding-agent',);
        /**
         The request Pi built, captured before any provider would see it.
         */
        const captured: { request: string } = { request: '', };
        /**
         Pi's own summary builder, as a function this test may call with a capturing stream.
         */
        const summarize = generateSummaryWithUsage as unknown as (...arguments_: unknown[]) => Promise<unknown>;
        await summarize(
          [{ role: 'user', content: (kept as { readonly content: unknown }).content, timestamp: 0, },],
          { reasoning: false, },
          0,
          undefined,
          undefined,
          undefined,
          undefined,
          undefined,
          undefined,
          async function capture(_model: unknown, context: unknown,) {
            captured.request = JSON.stringify(context,);
            return {
              result: async function result() {
                return {
                  stopReason: 'stop',
                  content: [{ type: 'text', text: 'stand-in summary', },],
                  usage: {},
                };
              },
            };
          },
          undefined,
          undefined,
          undefined,
          undefined,
        );
        expect(captured.request,).not.toBe('',);
        expect(captured.request,).not.toContain(SECRET,);
      },
    },),
    it({
      name: 'masks the arguments of a tool call on the feed, which Pi emits before any result exists',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         The masking the driver writes its own files through.
         */
        const masking: TextMasking = createSecretValueMasking({ values: VALUES, },);
        /**
         The run's feed.
         */
        const record = startRecordOfPiRun({
          feedFile: join(folder.path, 'feed.jsonl',),
          mask: masking,
        },);
        record.observe({
          event: {
            type: 'tool_execution_start',
            toolCallId: 'stand-in-call',
            toolName: 'bash',
            args: { command: `printenv STAND_IN_HOOK_API_KEY=${SECRET}`, },
          },
          declineDialog: function declineDialog() {
            // A tool call asks no dialog.
          },
        },);
        await record.close();
        /**
         What the feed says the call was, which is a marker where the value was.
         */
        const feed = await readFile(
          join(folder.path, 'feed.jsonl',),
          'utf8',
        );
        expect(feed,).not.toContain(SECRET,);
        expect(feed,).toContain(maskedValueMarker(SECRET_NAME,),);
      },
    },),
    it({
      name: 'keeps the structured content a tool declared when only its text was masked',
      fn: async () => {
        /**
         What Pi finalized for a result whose text held the value and whose structured content held none.
         */
        const finalized = await throughRealToolCall({
          content: [{ type: 'text', text: SECRET, },],
          details: { ordinary: 'retain this detail', },
          structuredContent: { ordinary: 'retain this structured value', },
        },);
        expect(JSON.stringify(finalized.result.content,),).not.toContain(SECRET,);
        expect(finalized.result.structuredContent,).toEqual({ ordinary: 'retain this structured value', },);
        expect(finalized.result.details,).toEqual({ ordinary: 'retain this detail', },);
      },
    },),
    it({
      name: 'answers a tool result nothing was masked in with no patch, which Pi reads as unchanged',
      fn: async () => {
        const masking = createSecretValueMasking({ values: VALUES, },);
        expect(maskedToolResult({
          event: {
            type: 'tool_result',
            toolCallId: 'stand-in-call',
            toolName: 'bash',
            input: {},
            content: [{ type: 'text', text: 'ordinary result', },],
            details: { ordinary: 'kept', },
            isError: false,
          } as never,
          masking,
        },),).toBeUndefined();
      },
    },),
  ],
},);
