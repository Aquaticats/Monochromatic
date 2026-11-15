/**
 The controls for what the second review found, each driving Pi's own installed functions or the code that writes a run's files:
 keys, `toJSON`, withheld details, the stream's hold-back, the files a rewrite must not lose or must not touch, and the two
 sources that must stop a run.

 Every value here is a stand-in this file makes up, and every file it writes is under the scratch folder the test harness names.

 @module
 */

import { appendFileSync, } from 'node:fs';
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
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
  createStreamMatcher,
  maskedValueMarker,
  maskedToolResult,
  replaceAFileWithItsMaskedText,
  spareCopyOfToolOutput,
  type SecretValue,
} from '../dist/final/node/index.mjs';

/**
 The stand-in value every control here arms with.
 */
const SECRET = 'stand-in-second-review-value-0123456789';

/**
 The name a marker says in its place.
 */
const SECRET_NAME = 'STAND_IN_SECOND_REVIEW_KEY';

/**
 What Pi's own tool-result hook answers when no handler changed the result, which is a failure for a result this file expects to
 be masked.
 */
const NOTHING_FINALIZED: unique symbol = Symbol('tool result unchanged',);

/**
 The one stand-in value every control arms with.
 */
const VALUES: readonly SecretValue[] = [{ name: SECRET_NAME, value: SECRET, },];

/**
 What an arming registered, and the runner Pi would build from it.
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
      // A failure is what these controls reproduce; the assertions below are the ones that matter.
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
 A tool result event of a tool that declared the given details.

 @param details - what the tool declared

 @returns the event Pi hands its hooks
 */
function resultWithDetails({ details, }: { readonly details: unknown },) {
  return {
    type: 'tool_result',
    toolCallId: 'stand-in-call',
    toolName: 'stand_in_tool',
    input: {},
    content: [{ type: 'text', text: 'ordinary result', },],
    details,
    isError: false,
  };
}

/**
 Runs one tool result through Pi's own runner.

 @param event - the tool result event

 @returns what Pi finalized
 */
async function throughPiToolResult(event: unknown,): Promise<{
  readonly content: readonly unknown[];
  readonly details: unknown;
} | typeof NOTHING_FINALIZED> {
  const { runner, } = armedRunner();
  // Pi's own hook answers the result it finalized, of Pi's own shape, or nothing when no handler changed it.
  /**
   What Pi's own hook answered, of any shape.
   */
  const answered: unknown = await ExtensionRunner.prototype.emitToolResult.call(
    runner,
    event as never,
  );
  if (answered === undefined) {
    return NOTHING_FINALIZED;
  }
  return answered as {
    readonly content: readonly unknown[];
    readonly details: unknown;
  };
}

/**
 A folder this file makes, removed when it is done.

 @returns the folder's path and its disposer
 */
async function makeFolder({ prefix, }: { readonly prefix: string },): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
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

await describe({
  name: 'what the second review found',
  children: [
    describe({
      name: 'an object a tool declared',
      children: [
        it({
          name: 'has its keys masked, because the mask is applied to the data the object serialises to',
          fn: async () => {
            const finalized = await throughPiToolResult(resultWithDetails({
              details: { [SECRET]: 'ordinary', },
            },),);
            if ((typeof finalized) === 'symbol') {
              throw new Error('Pi finalized the result unchanged, so nothing was masked',);
            }
            expect(JSON.stringify(finalized,),).not.toContain(SECRET,);
            expect(JSON.stringify(finalized,),).toContain(maskedValueMarker(SECRET_NAME,),);
          },
        },),
        it({
          name: 'cannot reintroduce a value through a toJSON method, which the round trip does not keep',
          fn: async () => {
            const details = {
              credential: SECRET,
              toJSON: function toJSON() {
                return { credential: SECRET, };
              },
            };
            /**
             What Pi finalized and then serialised, which is what a session file holds.
             */
            const finalized = await throughPiToolResult(resultWithDetails({ details, },),);
            if ((typeof finalized) === 'symbol') {
              throw new Error('Pi finalized the result unchanged, so nothing was masked',);
            }
            expect(JSON.stringify(finalized,),).not.toContain(SECRET,);
            /**
             A session Pi keeps the finalized result in.
             */
            await using folder = await makeFolder({ prefix: 'pi-second-review-to-json-', },);
            await mkdir(
              join(folder.path, 'sessions',),
              { recursive: true, },
            );
            const session = SessionManager.create(
              folder.path,
              join(folder.path, 'sessions',),
            );
            session.appendMessage({ role: 'user', content: 'safe prompt', timestamp: 0, },);
            session.appendMessage({
              role: 'toolResult',
              toolCallId: 'stand-in-call',
              toolName: 'stand_in_tool',
              content: [...(finalized?.content ?? []),] as never,
              details: finalized?.details as never,
              isError: false,
              timestamp: 0,
            },);
            expect(await textOfSession({ session, },),).not.toContain(SECRET,);
          },
        },),
        it({
          name: 'is withheld whole when its details cannot be masked, and Pi keeps no field of it',
          fn: async () => {
            /**
             Details with a cycle, which no serialisation can write.
             */
            const cyclic: Record<string, unknown> = { credential: SECRET, };
            cyclic.self = cyclic;
            const finalized = await throughPiToolResult(resultWithDetails({ details: cyclic, },),);
            if ((typeof finalized) === 'symbol') {
              throw new Error('Pi finalized the result unchanged, so nothing was withheld',);
            }
            expect(JSON.stringify(finalized,),).not.toContain(SECRET,);
            expect(JSON.stringify(finalized?.content,),).toContain('withheld',);
            // Pi reads an omitted `details` as unchanged, so the withheld result says it has none.
            expect(finalized?.details,).toEqual({ withheld: true, },);
          },
        },),
      ],
    },),
    describe({
      name: 'a stream',
      children: [
        it({
          name: 'holds back the beginning of a value that a cut would split, so the two writes mask it together',
          fn: async () => {
            const masking = createSecretValueMasking({ values: VALUES, },);
            const matcher = createStreamMatcher({ mask: masking, },);
            /**
             Where the stream is cut: inside the value, which is what a read boundary can do.
             */
            const cut = Math.floor((SECRET.length / 2),);
            /**
             What the first write may say.
             */
            const first = matcher.push(`a command printed ${SECRET.slice(0, cut,)}`,);
            /**
             What the second write may say, which is what arrived and what was held back.
             */
            const second = matcher.push(`${SECRET.slice(cut,)} and then some`,) + matcher.flush();
            expect(`${first}${second}`,).not.toContain(SECRET,);
            expect(`${first}${second}`,).toContain(maskedValueMarker(SECRET_NAME,),);
            // The first write holds back nothing that is part of the value it could already decide.
            expect(first,).not.toContain(SECRET.slice(0, cut,),);
            expect(matcher.heldLength(),).toBe(0,);
          },
        },),
        it({
          name: 'masks a value that arrives whole in one write without holding anything back',
          fn: async () => {
            const masking = createSecretValueMasking({ values: VALUES, },);
            const matcher = createStreamMatcher({ mask: masking, },);
            const written = matcher.push(`key=${SECRET}\n`,) + matcher.flush();
            expect(written,).toBe(`key=${maskedValueMarker(SECRET_NAME,)}\n`,);
          },
        },),
      ],
    },),
    describe({
      name: 'a file a rewrite puts masked text into',
      children: [
        it({
          name: 'keeps what another command appended while the masked text was being prepared',
          fn: async () => {
            await using folder = await makeFolder({ prefix: 'pi-second-review-append-', },);
            /**
             The file, holding the value.
             */
            const path = join(folder.path, 'session.jsonl',);
            await writeFile(
              path,
              `${JSON.stringify({ entry: SECRET, },)}\n`,
            );
            /**
             What another command appended, which must survive the rewrite.
             */
            const appendedLine = JSON.stringify({ entry: 'another command wrote this', },);
            /**
             The masking, which appends as a second writer would between the read and the swap.
             */
            const masking = createSecretValueMasking({ values: VALUES, },);
            /**
             Whether the second writer has already written, so it appends once and the rewrite has a file to settle on.
             */
            const appended: { done: boolean } = { done: false, };
            const replaced = await replaceAFileWithItsMaskedText({
              path,
              writer: { gone: true, },
              maskText: function maskAndAppend(text: string,) {
                // A second writer appends here, between the read and the swap, which is the instant the replacement must notice.
                if (!appended.done) {
                  appended.done = true;
                  appendFileSync(
                    path,
                    `${appendedLine}\n`,
                  );
                }
                return masking.maskText(text,);
              },
            },);
            expect(replaced,).toEqual({ replaced: true, },);
            /**
             What the file holds at the end.
             */
            const kept = await readFile(
              path,
              'utf8',
            );
            expect(kept,).toContain(appendedLine,);
            expect(kept,).not.toContain(SECRET,);
          },
        },),
        it({
          name: 'leaves Pi own output whole when a tool result names a file whose name is only its prefix',
          fn: async () => {
            await using folder = await makeFolder({ prefix: 'pi-second-review-output-', },);
            /**
             A file of somebody else's, whose name begins with Pi's prefix and nothing more.
             */
            const path = join(folder.path, 'pi-output-user-notes',);
            await writeFile(
              path,
              `user text ${SECRET}`,
            );
            expect(spareCopyOfToolOutput({
              details: { fullOutputPath: path, },
              piOutputFolder: folder.path,
            },),).toEqual({ absent: true, },);
            expect(await readFile(
              path,
              'utf8',
            ),).toBe(`user text ${SECRET}`,);
          },
        },),
      ],
    },),
    describe({
      name: maskedToolResult.name,
      children: [
        it({
          name: 'withholds a result whose structured content cannot be masked, rather than handing any of it on',
          fn: async () => {
            const masking = createSecretValueMasking({ values: VALUES, },);
            /**
             A structured value with a cycle, which cannot be written.
             */
            const cyclic: Record<string, unknown> = { credential: SECRET, };
            cyclic.self = cyclic;
            const answer = maskedToolResult({
              event: {
                ...resultWithDetails({ details: { ordinary: 'kept', }, },),
                structuredContent: cyclic,
              } as never,
              masking,
            },);
            expect('withheld' in answer,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);

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
