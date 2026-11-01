/**
 The control socket of the nested compositor: the lines it understands and the key presses built from them.
 
 The compositor takes one command per connection (`key ctrl press`, `type hello`, `click 10 20 left`, `screenshot
 file`, `move 2 2`, `quit`) and answers one line that starts with `ok` or `err`.
 
 @example
 ```ts
 import { chordCommands, } from './control.ts';
 
 chordCommands({ mods: ['ctrl',], key: 'o', },); // ['key ctrl press', 'key o tap', 'key ctrl release']
 ```
 
 @packageDocumentation
 */

import { connect, } from 'node:net';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import { expect, } from './errors.ts';
import { SHOT_SETTLE_MS, } from './timing.ts';

/**
 What a key does in one command: a press and release together, a press alone, or a release alone.
 */
export type KeyAction = 'press' | 'release' | 'tap';

/**
 Mouse buttons the compositor knows.
 */
export type MouseButton = 'left' | 'right';

/**
 The operations a check uses to drive the app through the compositor.
 */
export type Control = {
  /**
   Sends a command and requires an `ok` answer.
   */
  readonly ok: (line: string,) => Promise<void>;
  /**
   Presses modifiers in order, taps a key, releases the modifiers in reverse order.
   */
  readonly chord: (chord: {
    readonly key: string;
    readonly mods: readonly string[]
  },) => Promise<void>;
  /**
   Clicks at a position.
   */
  readonly click: (click: {
    readonly button?: MouseButton;
    readonly x: number;
    readonly y: number
  },) => Promise<void>;
  /**
   Presses a key and keeps it down.
   */
  readonly down: (key: string,) => Promise<void>;
  /**
   Sends a command and returns the answer line.
   */
  readonly send: (line: string,) => Promise<string>;
  /**
   Moves the pointer out of the way, waits for a redraw, and saves a screenshot.
   */
  readonly shot: (name: string,) => Promise<string>;
  /**
   Taps a key.
   */
  readonly tap: (key: string,) => Promise<void>;
  /**
   Types text.
   */
  readonly type: (text: string,) => Promise<void>;
  /**
   Releases a key.
   */
  readonly up: (key: string,) => Promise<void>;
};

/**
 Builds the command for one key action.
 
 @param key - key name in the compositor's vocabulary, such as `ctrl`, `tab`, `pageup` or `f5`
 
 @param action - press, release, or tap
 
 @returns the command line
 
 @example
 ```ts
 keyCommand({ key: 'tab', action: 'tap', },); // 'key tab tap'
 ```
 */
export function keyCommand(
  {
    key,
    action,
  }: {
    readonly action: KeyAction;
    readonly key: string;
  },
): string {
  return `key ${key} ${action}`;
}

/**
 Builds the commands of a chord: the modifiers go down in the given order, the key is tapped, the modifiers come up in
 reverse order, as a person does.
 
 @param mods - modifier key names, in the order they are pressed
 
 @param key - the key tapped while the modifiers are down
 
 @returns the command lines in the order they are sent
 
 @example
 ```ts
 chordCommands({ mods: ['ctrl', 'shift',], key: 'g', },);
 ```
 */
export function chordCommands(
  {
    mods,
    key,
  }: {
    readonly key: string;
    readonly mods: readonly string[];
  },
): readonly string[] {
  return [
    ...mods.map(function pressMod(mod,): string {
      return keyCommand({
        key: mod,
        action: 'press',
      },);
    },),
    keyCommand({
      key,
      action: 'tap',
    },),
    ...mods.toReversed()
      .map(function releaseMod(mod,): string {
        return keyCommand({
          key: mod,
          action: 'release',
        },);
      },),
  ];
}

/**
 Sends one command on a fresh connection and reads the answer line.
 
 @param socket - path of the compositor's control socket
 
 @param line - command, without the final newline
 
 @returns the answer, trimmed
 
 @throws Error when the socket cannot be reached
 
 @example
 ```ts
 const answer = await sendCommand({ socket, line: 'move 2 2', },);
 ```
 */
export async function sendCommand(
  {
    socket,
    line,
  }: {
    readonly line: string;
    readonly socket: string;
  },
): Promise<string> {
  /**
   The connection; writes made before it is open wait for it.
   */
  const client = connect(socket,);
  client.setEncoding('utf8',);
  client.write(`${line}\n`,);

  /**
   Answer text read so far, kept in a one-slot holder.
   */
  const reply = { text: '', };
  for await (const chunk of client) {
    reply.text += String(chunk,);
    if (reply.text
      .includes('\n',))
      client.end();
  }
  return reply.text
    .trim();
}

/**
 Builds the control operations for one compositor.
 
 @param socket - path of the control socket
 
 @param shotsDir - directory the screenshots go into
 
 @param label - session label that prefixes screenshot names
 
 @returns the operations
 
 @example
 ```ts
 const control = createControl({ socket, shotsDir, label: 'wide', },);
 ```
 */
export function createControl(
  {
    socket,
    shotsDir,
    label,
  }: {
    readonly label: string;
    readonly shotsDir: string;
    readonly socket: string;
  },
): Control {
  /**
   Sends a command on a fresh connection.
   
   @param line - command line
   
   @returns the answer
   */
  async function send(line: string,): Promise<string> {
    return await sendCommand({
      socket,
      line,
    },);
  }

  /**
   Sends a command and requires an answer that starts with `ok`.
   
   @param line - command line
   */
  async function ok(line: string,): Promise<void> {
    /**
     The answer line.
     */
    const answer = await send(line,);
    expect({
      condition: answer.startsWith('ok',),
      message: `compositor answered "${answer}" to "${line}"`,
    },);
  }

  /**
   Sends the commands one after another.
   
   @param lines - command lines in order
   */
  async function okInOrder(lines: readonly string[],): Promise<void> {
    for (const line of lines)
      // oxlint-disable-next-line eslint/no-await-in-loop -- the commands of a chord must reach the app in order
      await ok(line,);
  }

  return {
    send,
    ok,
    tap: async function tap(key,): Promise<void> {
      await ok(keyCommand({
        key,
        action: 'tap',
      },),);
    },
    down: async function down(key,): Promise<void> {
      await ok(keyCommand({
        key,
        action: 'press',
      },),);
    },
    up: async function up(key,): Promise<void> {
      await ok(keyCommand({
        key,
        action: 'release',
      },),);
    },
    chord: async function chord({
      mods,
      key,
    },): Promise<void> {
      await okInOrder(chordCommands({
        mods,
        key,
      },),);
    },
    type: async function type(text,): Promise<void> {
      await ok(`type ${text}`,);
    },
    click: async function click({
      x,
      y,
      button = 'left',
    },): Promise<void> {
      await ok(`click ${x} ${y} ${button}`,);
    },
    shot: async function shot(name,): Promise<string> {
      /**
       Screenshot file, named by session label and shot name.
       */
      const file = join(
        shotsDir,
        `${label}-${name}.png`,
      );
      await ok('move 2 2',);
      await wait(SHOT_SETTLE_MS,);
      await ok(`screenshot ${file}`,);
      return file;
    },
  };
}
