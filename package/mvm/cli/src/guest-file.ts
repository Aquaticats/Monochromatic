/**
 One open file in a guest, through the QEMU guest agent's file commands
 (`guest-file-open`, `-read`, `-write`, `-seek`, `-close`).

 The agent takes the path as a JSON string and opens it itself, so no guest
 shell and no quoting are involved. Data travels base64-encoded inside the
 JSON of each command.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  agentCommand,
  GuestAgentProtocolError,
} from './agent-command.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Shapes

/**
 Seconds libvirt waits for the agent to answer one file command.
 A minute lets a busy guest answer; libvirt's default of five seconds was observed to lapse under load.
 */
export const FILE_COMMAND_TIMEOUT_SECONDS = 60;

/**
 Open modes used by mvm, a subset of the modes the agent accepts on Linux and Windows:
 `rb` reads an existing file, `wb` creates or empties a file for writing.

 @example
 ```ts
 const mode: GuestFileMode = 'wb';
 ```
 */
export type GuestFileMode = 'rb' | 'wb';

/**
 Reference point of a seek, as named by the guest agent protocol.

 @example
 ```ts
 const whence: GuestFileWhence = 'end';
 ```
 */
export type GuestFileWhence = 'cur' | 'end' | 'set';

/**
 Data returned by one read.

 @example
 ```ts
 const chunk: GuestFileChunk = { data: Buffer.from('hi'), eof: true };
 ```
 */
export type GuestFileChunk = {
  /**
   Bytes read; empty at the end of the file.
   */
  readonly data: Buffer;
  /**
   Whether the agent reported the end of the file with this read.
   */
  readonly eof: boolean;
};

/**
 An open guest file. Dispose it, or call {@link GuestFile.close}, to release the agent's handle.

 @example
 ```ts
 await using file = await openGuestFile({ domain: 'mvm-dev', mode: 'rb', path: '/etc/hostname' });
 const chunk = await file.read(4_096);
 ```
 */
export type GuestFile = AsyncDisposable & {
  /**
   Closes the handle; a failure here means buffered writes may not have reached the file.
   */
  readonly close: () => Promise<void>;
  /**
   Reads up to `count` bytes at the current position.
   */
  readonly read: (count: number,) => Promise<GuestFileChunk>;
  /**
   Moves the position and returns the new one, counted from the start of the file.
   */
  readonly seek: (target: {
    readonly offset: number;
    readonly whence: GuestFileWhence;
  },) => Promise<number>;
  /**
   Writes `data` at the current position and returns the number of bytes the agent wrote.
   */
  readonly write: (data: Buffer,) => Promise<number>;
};

//endregion Shapes

//region Reading answers

/**
 Reads a numeric property from an agent answer.

 @param commandName - Agent command the answer belongs to, named when the property is missing

 @param key - Property name from the protocol reference

 @param of - Agent answer

 @returns The property's value

 @throws {@link GuestAgentProtocolError} when the answer has no such number

 @example
 ```ts
 numberIn({ commandName: 'guest-file-write', key: 'count', of: { count: 5 } }); // => 5
 ```
 */
function numberIn({
  commandName,
  key,
  of,
}: {
  readonly commandName: string;
  readonly key: string;
  readonly of: unknown;
},): number {
  if (((typeof of) === 'object') && (of !== null)) {
    /**
     Property value, typed as unknown until checked.
     */
    const value: unknown = Reflect.get(
      of,
      key,
    );
    if ((typeof value) === 'number') {
      return value;
    }
  }
  throw new GuestAgentProtocolError({
    commandName,
    received: JSON.stringify(of,),
  },);
}

/**
 Interprets a `guest-file-read` answer.

 @param answer - Value the agent returned

 @returns Decoded bytes and the end-of-file flag

 @throws {@link GuestAgentProtocolError} when the answer has no numeric `count`

 @example
 ```ts
 chunkIn({ 'buf-b64': 'aGk=', count: 2, eof: false }); // => { data: <Buffer 68 69>, eof: false }
 ```
 */
function chunkIn(answer: unknown,): GuestFileChunk {
  /**
   Number of bytes read before base64 encoding; validates the answer's shape.
   */
  const count = numberIn({
    commandName: 'guest-file-read',
    key: 'count',
    of: answer,
  },);
  /**
   Base64 text of the bytes; the agent leaves it out when nothing was read.
   */
  const encoded: unknown = (((typeof answer) === 'object') && (answer !== null))
    ? Reflect.get(
      answer,
      'buf-b64',
    )
    : '';
  /**
   Decoded bytes.
   */
  const data = Buffer.from(
    ((typeof encoded) === 'string') ? encoded : '',
    'base64',
  );
  if (data.length !== count) {
    throw new GuestAgentProtocolError({
      commandName: 'guest-file-read',
      received: `count ${String(count,)} with ${String(data.length,)} decoded bytes`,
    },);
  }
  return {
    data,
    eof: (((typeof answer) === 'object') && (answer !== null))
      && (Reflect.get(
        answer,
        'eof',
      ) === true),
  };
}

//endregion Reading answers

//region Open

/**
 Opens a file in the guest.

 @param domain - Prefixed libvirt domain name

 @param mode - `rb` to read an existing file, `wb` to create or empty a file for writing

 @param path - Path in the guest, in the guest's own form; passed to the agent unchanged

 @returns Open file whose disposal closes the agent's handle

 @throws {@link GuestAgentReplyError} when the agent cannot open the path

 @throws {@link GuestAgentUnreachableError} when the agent does not answer

 @example
 ```ts
 await using file = await openGuestFile({ domain: 'mvm-win', mode: 'wb', path: 'C:\\Users\\Public\\setup file.msi' });
 ```
 */
export async function openGuestFile({
  domain,
  mode,
  path,
}: {
  readonly domain: string;
  readonly mode: GuestFileMode;
  readonly path: string;
},): Promise<GuestFile> {
  /**
   Logger scoped to this file so a failed close during cleanup is attributable.
   */
  const rl = tagged({
    tag: openGuestFile.name,
    l,
  },);
  /**
   Handle the agent assigned; every later command names it.
   */
  const handle = await agentCommand({
    domain,
    execute: 'guest-file-open',
    parameters: {
      mode,
      path,
    },
    timeoutSeconds: FILE_COMMAND_TIMEOUT_SECONDS,
  },);
  if ((typeof handle) !== 'number') {
    throw new GuestAgentProtocolError({
      commandName: 'guest-file-open',
      received: JSON.stringify(handle,),
    },);
  }
  /**
   Whether the handle is still open; closing twice would name a handle the agent no longer knows.
   */
  const lifecycle = { open: true, };
  /**
   Closes the handle once.
   */
  async function close(): Promise<void> {
    if (!lifecycle.open) {
      return;
    }
    lifecycle.open = false;
    await agentCommand({
      domain,
      execute: 'guest-file-close',
      parameters: { handle, },
      timeoutSeconds: FILE_COMMAND_TIMEOUT_SECONDS,
    },);
  }
  return {
    close,
    async read(count: number,): Promise<GuestFileChunk> {
      return chunkIn(
        await agentCommand({
          domain,
          execute: 'guest-file-read',
          parameters: {
            count,
            handle,
          },
          timeoutSeconds: FILE_COMMAND_TIMEOUT_SECONDS,
        },),
      );
    },
    async seek({
      offset,
      whence,
    },): Promise<number> {
      return numberIn({
        commandName: 'guest-file-seek',
        key: 'position',
        of: await agentCommand({
          domain,
          execute: 'guest-file-seek',
          parameters: {
            handle,
            offset,
            whence,
          },
          timeoutSeconds: FILE_COMMAND_TIMEOUT_SECONDS,
        },),
      },);
    },
    async write(data: Buffer,): Promise<number> {
      return numberIn({
        commandName: 'guest-file-write',
        key: 'count',
        of: await agentCommand({
          domain,
          execute: 'guest-file-write',
          parameters: {
            'buf-b64': data.toString('base64',),
            handle,
          },
          timeoutSeconds: FILE_COMMAND_TIMEOUT_SECONDS,
        },),
      },);
    },
    async [Symbol.asyncDispose](): Promise<void> {
      try {
        await close();
      }
      catch (error) {
        if (!(Error.isError(error,)))
          throw error;

        // Disposal runs while another failure may already be propagating; that failure is the one to report.
        rl.warn(`could not close guest file ${path} in ${domain}: ${error.message}`,);
      }
    },
  };
}

//endregion Open
