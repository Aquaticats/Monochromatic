/**
 What the extension Pi loads does with a tool result and with a transcript: it masks every value of this machine's secret
 environment, names the variable in its place, keeps every part nothing was masked in, and rewrites the file Pi saved a truncated
 output in.

 Every value here is a stand-in this file makes up, and every file it writes is under the scratch folder the test harness names.

 @module
 */

import {
  chmod,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { spawn, } from 'node:child_process';
import { Writable, } from 'node:stream';
import { randomBytes, } from 'node:crypto';
import type {
  ContextWithSystemEvent,
  ExtensionAPI,
  ToolResultEvent,
  ToolResultEventResult,
} from '@earendil-works/pi-coding-agent';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  armSecretValueMasking,
  MASKING_ARMED_COMMAND,
  maskingArmedDescription,
  maskedValueMarker,
  readTheSecretValuesHandedOver,
  type SecretValue,
  SECRET_VALUES_FD,
} from '../dist/final/node/index.mjs';

/**
 A stand-in value, long enough to be masked.
 */
const TOKEN_VALUE = 'stand-in-token-value-0123456789';

/**
 The name the marker says in its place.
 */
const TOKEN_NAME = 'STAND_IN_API_KEY';

/**
 What a marker says where the stand-in value was.
 */
const MARKER = maskedValueMarker(TOKEN_NAME,);

/**
 The one stand-in value every test arms with.
 */
const VALUES: readonly SecretValue[] = [{ name: TOKEN_NAME, value: TOKEN_VALUE, },];

/**
 What Pi's extension API stored for one arming.
 */
type Recording = {
  /**
   The handler each event registered, by the event's name.
   */
  readonly handlers: Map<string, (event: never,) => unknown>;

  /**
   Each command registered, as Pi's command list answers it.
   */
  readonly commands: readonly { readonly name: string; readonly description?: string }[];

  /**
   The lines the arming wrote about its own failures.
   */
  readonly failures: readonly string[];
};

/**
 Records what an arming asks Pi for, and plays the extension API it asks through.

 @returns what the arming registered
 */
function armRecording(): Recording {
  /**
   The handlers and commands the arming registers.
   */
  const recorded: {
    handlers: Map<string, (event: never,) => unknown>;
    commands: { name: string; description?: string }[];
    failures: string[];
  } = {
    handlers: new Map(),
    commands: [],
    failures: [],
  };
  /**
   The extension API the arming writes to, which is the two members it uses.
   */
  const api = {
    on: function on(event: string, handler: (event: never,) => unknown,) {
      recorded.handlers.set(
        event,
        handler,
      );
      return function off(): void {
        recorded.handlers.delete(event,);
      };
    },
    registerCommand: function registerCommand(name: string, options: { readonly description?: string },) {
      recorded.commands.push({
        name,
        ...((options.description === undefined) ? {} : { description: options.description, }),
      },);
    },
  } as unknown as ExtensionAPI;
  armSecretValueMasking({
    pi: api,
    values: VALUES,
    noteFailure: function note(text,) {
      recorded.failures.push(text,);
    },
  },);
  return recorded;
}

/**
 A tool result of a bash command that printed a line.

 @param text - what the command printed

 @param details - the tool result's own details

 @returns the event Pi hands its `tool_result` hooks
 */
function bashResultOf({
  text,
  details,
}: {
  readonly text: string;
  readonly details?: unknown;
},): ToolResultEvent {
  return {
    type: 'tool_result',
    toolCallId: 'call-1',
    toolName: 'bash',
    input: { command: 'mise env', },
    content: [{
      type: 'text',
      text,
    },],
    isError: false,
    ...(details === undefined ? {} : { details, }),
  } as ToolResultEvent;
}

/**
 What Pi's own `tool_result` hook answers: the patch it keeps, or nothing when the result is unchanged.
 */
// oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- Pi's `tool_result` handler result is `T | undefined`.
type MaskedResult = ToolResultEventResult | undefined;

/**
 Runs the `tool_result` handler the arming registered.

 @param recording - what the arming registered

 @param event - the tool result event

 @returns what the handler answered
 */
async function maskOneResult(recording: Recording, event: ToolResultEvent,): Promise<MaskedResult> {
  /**
   The handler the extension registered.
   */
  const handler = recording.handlers.get('tool_result',);
  if (handler === undefined) {
    throw new Error('the extension registered no tool_result handler',);
  }
  return await handler(event as never,) as MaskedResult;
}

/**
 Runs the `context_with_system` handler the arming registered.

 @param recording - what the arming registered

 @param event - the transcript event

 @returns what the handler answered
 */
function maskTranscript(recording: Recording, event: ContextWithSystemEvent,): unknown {
  /**
   The handler the extension registered.
   */
  const handler = recording.handlers.get('context_with_system',);
  if (handler === undefined) {
    throw new Error('the extension registered no context_with_system handler',);
  }
  return handler(event as never,);
}

/**
 Makes a folder of this test's own, under the scratch root, removed when the test ends.

 @param prefix - what the folder holds, so a reader of the scratch root knows which test left it

 @returns the folder's path and its disposer
 */
async function makeFolder({ prefix, }: { readonly prefix: string },): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder, under the scratch root the test harness names rather than the system's temporary directory.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    prefix,
  ),);
  await chmod(
    path,
    0o700,
  );
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

/**
 Writes a file named as Pi names the one it keeps a tool's whole output in.

 @param folder - folder to write it in

 @param text - what the file holds

 @returns the file's path
 */
async function writeSpareCopy({
  folder,
  text,
}: {
  readonly folder: string;
  readonly text: string;
},): Promise<string> {
  /**
   A path named the way Pi names one, so the extension treats it as a file it may rewrite.
   */
  const path = join(
    folder,
    `pi-output-${randomBytes(
      8,
    )
      .toString('hex',)}.txt`,
  );
  await writeFile(
    path,
    text,
  );
  return path;
}

await describe({
  name: 'the extension that masks secret values',
  children: [
    it({
      name: 'registers the two hooks and one command saying how many values are armed',
      fn: async () => {
        const recording = armRecording();
        expect([...recording.handlers.keys(),],).toEqual([
          'tool_result',
          'context_with_system',
        ],);
        expect(recording.commands,).toEqual([{
          name: MASKING_ARMED_COMMAND,
          description: maskingArmedDescription({ count: VALUES.length, },),
        },],);
      },
    },),
    it({
      name: 'masks a value a command printed inside a longer line, and names the variable in its place',
      fn: async () => {
        const recording = armRecording();
        /**
         What the tool result says after masking.
         */
        const masked = await maskOneResult(
          recording,
          bashResultOf({ text: `${TOKEN_NAME}=${TOKEN_VALUE}`, },),
        );
        expect(masked?.content,).toEqual([{
          type: 'text',
          text: `${TOKEN_NAME}=${MARKER}`,
        },],);
        expect(JSON.stringify(masked,),).not.toContain(TOKEN_VALUE,);
      },
    },),
    it({
      name: 'leaves an image part exactly as it was and masks the text beside it',
      fn: async () => {
        const recording = armRecording();
        /**
         An image part, which a tool result may hold beside its text.
         */
        const image = {
          type: 'image' as const,
          data: 'c3RhbmQtaW4taW1hZ2U=',
          mimeType: 'image/png',
        };
        /**
         A tool result holding the image and a line with the value.
         */
        const event: ToolResultEvent = {
          type: 'tool_result',
          toolCallId: 'call-1',
          toolName: 'bash',
          input: {},
          content: [
            image,
            { type: 'text', text: `${TOKEN_NAME}=${TOKEN_VALUE}`, },
          ],
          isError: false,
          details: undefined,
        };
        const masked = await maskOneResult(recording, event,);
        expect(masked?.content?.[0],).toEqual(image,);
        expect(masked?.content?.[1],).toEqual({ type: 'text', text: `${TOKEN_NAME}=${MARKER}`, },);
      },
    },),
    it({
      name: 'answers nothing when no value stands in the result, so Pi keeps everything it had',
      fn: async () => {
        const recording = armRecording();
        const masked = await maskOneResult(
          recording,
          bashResultOf({ text: 'a build finished in 3.4 s', },),
        );
        expect(masked,).toBeUndefined();
      },
    },),
    it({
      name: 'masks the structured content a tool declared, and keeps it beside the masked text',
      fn: async () => {
        const recording = armRecording();
        /**
         A result whose content printed nothing but whose structured content holds the value.
         */
        const event = {
          ...bashResultOf({ text: 'done', },),
          structuredContent: { key: TOKEN_VALUE, },
        } as ToolResultEvent;
        const masked = await maskOneResult(recording, event,);
        expect(masked?.structuredContent,).toEqual({ key: MARKER, },);
      },
    },),
    it({
      name: 'masks a value a session already held, which is what a resumed run sends again',
      fn: async () => {
        const recording = armRecording();
        /**
         A transcript holding a tool result of before the masking existed, as a resumed session hands one to the model.
         */
        const event: ContextWithSystemEvent = {
          type: 'context_with_system',
          messages: [{
            role: 'toolResult',
            toolCallId: 'call-0',
            toolName: 'bash',
            content: [{
              type: 'text',
              text: `${TOKEN_NAME}=${TOKEN_VALUE}`,
            },],
            isError: false,
            timestamp: 0,
          },],
        };
        /**
         What the hook answered.
         */
        const answer: unknown = maskTranscript(recording, event,);
        expect(JSON.stringify(answer,),).not.toContain(TOKEN_VALUE,);
        expect(JSON.stringify(answer,),).toContain(MARKER,);
      },
    },),
    it({
      name: 'answers nothing when a transcript holds no value, so the request is the one Pi built',
      fn: async () => {
        const recording = armRecording();
        const event: ContextWithSystemEvent = {
          type: 'context_with_system',
          messages: [{
            role: 'user',
            content: 'carry on',
            timestamp: 0,
          },],
        };
        expect(maskTranscript(recording, event,),).toBeUndefined();
      },
    },),
    it({
      name: 'rewrites the file Pi saved a truncated output in, so no value is left in what Pi wrote',
      fn: async () => {
        await using folder = await makeFolder({ prefix: 'pi-masked-spare-copy-', },);
        /**
         The file Pi saved the whole output in.
         */
        const path = await writeSpareCopy({
          folder: folder.path,
          text: `${TOKEN_NAME}=${TOKEN_VALUE}\n`,
        },);
        const recording = armRecording();
        await maskOneResult(
          recording,
          bashResultOf({
            text: 'the output was longer than Pi shows',
            details: { fullOutputPath: path, },
          },),
        );
        /**
         What the file holds once the result was masked.
         */
        const kept = await readFile(
          path,
          'utf8',
        );
        expect(kept,).toBe(`${TOKEN_NAME}=${MARKER}\n`,);
        expect(kept,).not.toContain(TOKEN_VALUE,);
      },
    },),
    it({
      name: 'leaves a file of its own name alone, so only the files Pi made are rewritten',
      fn: async () => {
        await using folder = await makeFolder({ prefix: 'pi-unrelated-file-', },);
        /**
         A file named as the agent's own, holding text the agent wrote.
         */
        const path = join(folder.path, 'notes.txt',);
        await writeFile(
          path,
          `${TOKEN_NAME}=${TOKEN_VALUE}\n`,
        );
        const recording = armRecording();
        await maskOneResult(
          recording,
          bashResultOf({
            text: 'done',
            details: { fullOutputPath: path, },
          },),
        );
        expect(await readFile(
          path,
          'utf8',
        ),).toBe(`${TOKEN_NAME}=${TOKEN_VALUE}\n`,);
      },
    },),
    it({
      name: 'reads a handover a driver wrote into a descriptor in pieces, and takes every byte',
      fn: async () => {
        /**
         The payload the driver writes, which is what it hands Pi.
         */
        const payload = JSON.stringify({ values: VALUES, },);
        /**
         Where the reader is, run as the child that plays Pi's side of the handover.
         */
        const readerEntry = new URL(
          'secret-values-handed-to-pi-reader-fixture.ts',
          import.meta.url,
        ).pathname;
        const child = spawn(
          process.execPath,
          [readerEntry,],
          { stdio: ['ignore', 'pipe', 'pipe', 'pipe',], },
        );
        /**
         What the child printed, which names the values and holds none.
         */
        const printed = child.stdout;
        if (printed === null) {
          throw new Error('the child was started without the output this test reads',);
        }
        printed.setEncoding('utf8',);
        /**
         Everything the child said.
         */
        const heard: string[] = [];
        printed.on(
          'data',
          function collect(chunk: string,) {
            heard.push(chunk,);
          },
        );
        /**
         The child's own end, once it has read the payload.
         */
        const ended = new Promise<number>(function waitForEnd(resolve,) {
          child.on(
            'close',
            function onClose(code,) {
              resolve(code ?? (-1),);
            },
          );
        },);
        /**
         The descriptor the driver's end of the pipe is.
         */
        const valuesPipe = child.stdio[SECRET_VALUES_FD];
        if (!(valuesPipe instanceof Writable)) {
          throw new Error('the child was started without the descriptor the values are handed over on',);
        }
        // Two writes with a pause between them: a pipe gives a reader whatever has arrived, so this is what a driver's payload
        // arrives as when the write is large enough to fill the pipe.
        valuesPipe.write(payload.slice(0, 20,),);
        await new Promise(function pause(resolve,) {
          setTimeout(resolve, 50,);
        },);
        valuesPipe.end(payload.slice(20,),);
        expect(await ended,).toBe(0,);
        expect(heard.join('',).trim(),).toBe(TOKEN_NAME,);
      },
    },),
    it({
      name: 'stops a run whose descriptor holds no handover, rather than working with its masking off',
      fn: async () => {
        expect(() => {
          readTheSecretValuesHandedOver({ fd: 99_999, },);
        },).toThrow(Error,);
      },
    },),
  ],
},);
