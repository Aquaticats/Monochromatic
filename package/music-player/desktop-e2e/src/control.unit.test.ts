/**
 Tests for the compositor control: command lines, chords, and the socket exchange against a stand-in compositor.
 
 @module
 */

import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import {
  createServer,
  type Server,
} from 'node:net';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  chordCommands,
  createControl,
  keyCommand,
  sendCommand,
} from './control.ts';

/**
 A stand-in compositor on a unix socket: it records each command line and answers with a fixed line.
 */
type FakeCompositor = {
  readonly close: () => Promise<void>;
  readonly commands: string[];
  readonly socket: string;
};

/**
 Starts the stand-in compositor.
 
 @param answer - the line it answers to every command
 
 @returns its socket path, the commands it received and a way to close it
 */
async function startFakeCompositor(answer: string,): Promise<FakeCompositor> {
  const dir = await mkdtemp(join(scratchRoot(), 'e2e-control-',),);
  const socket = join(dir, 'c.sock',);
  const commands: string[] = [];
  const server: Server = createServer((client,) => {
    client.setEncoding('utf8',);
    let received = '';
    client.on('data', (chunk,) => {
      received += String(chunk,);
      if (received.includes('\n',)) {
        commands.push(received.trim(),);
        client.end(`${answer}\n`,);
      }
    },);
  },);
  await new Promise<void>((resolve,) => {
    server.listen(socket, resolve,);
  },);
  return {
    socket,
    commands,
    close: async () => {
      await new Promise<void>((resolve,) => {
        server.close(() => {
          resolve();
        },);
      },);
      await rm(dir, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: keyCommand.name,
  children: [
    it({
      name: 'writes the key, then the action',
      fn: async ({ expect, }) => {
        expect(keyCommand({ key: 'tab', action: 'tap', },),).toBe('key tab tap',);
        expect(keyCommand({ key: 'ctrl', action: 'press', },),).toBe('key ctrl press',);
        expect(keyCommand({ key: 'ctrl', action: 'release', },),).toBe('key ctrl release',);
      },
    },),
  ],
},);

await describe({
  name: chordCommands.name,
  children: [
    it({
      name: 'presses modifiers in order, taps the key, releases modifiers in reverse order',
      fn: async ({ expect, }) => {
        expect(chordCommands({ mods: ['ctrl', 'shift',], key: 'g', },),).toEqual([
          'key ctrl press',
          'key shift press',
          'key g tap',
          'key shift release',
          'key ctrl release',
        ],);
      },
    },),
    it({
      name: 'taps the key alone when there are no modifiers',
      fn: async ({ expect, }) => {
        expect(chordCommands({ mods: [], key: 'a', },),).toEqual(['key a tap',],);
      },
    },),
    it({
      name: 'does not change the list of modifiers it is given',
      fn: async ({ expect, }) => {
        const mods = ['alt', 'shift',];
        chordCommands({ mods, key: 'right', },);
        expect(mods,).toEqual(['alt', 'shift',],);
      },
    },),
  ],
},);

await describe({
  name: sendCommand.name,
  children: [
    it({
      name: 'sends one line and returns the trimmed answer',
      fn: async ({ expect, }) => {
        const compositor = await startFakeCompositor('ok',);
        expect(await sendCommand({ socket: compositor.socket, line: 'move 2 2', },),).toBe('ok',);
        expect(compositor.commands,).toEqual(['move 2 2',],);
        await compositor.close();
      },
    },),
    it({
      name: 'rejects when the socket cannot be reached',
      fn: async ({ expect, }) => {
        let caught: unknown;
        try {
          await sendCommand({ socket: join(scratchRoot(), 'e2e-no-such-socket',), line: 'x', },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
      },
    },),
  ],
},);

await describe({
  name: createControl.name,
  children: [
    it({
      name: 'sends the key commands of tap, down, up and chord in order',
      fn: async ({ expect, }) => {
        const compositor = await startFakeCompositor('ok',);
        const control = createControl({ socket: compositor.socket, shotsDir: '/shots', label: 'wide', },);
        await control.tap('tab',);
        await control.down('alt',);
        await control.up('alt',);
        await control.chord({ mods: ['ctrl',], key: 'o', },);
        await control.type('tone',);
        await control.click({ x: 3, y: 4, },);
        await control.click({ x: 5, y: 6, button: 'right', },);
        expect(compositor.commands,).toEqual([
          'key tab tap',
          'key alt press',
          'key alt release',
          'key ctrl press',
          'key o tap',
          'key ctrl release',
          'type tone',
          'click 3 4 left',
          'click 5 6 right',
        ],);
        await compositor.close();
      },
    },),
    it({
      name: 'rejects an answer that does not start with ok and names the command',
      fn: async ({ expect, }) => {
        const compositor = await startFakeCompositor('err unknown key: f5',);
        const control = createControl({ socket: compositor.socket, shotsDir: '/shots', label: 'wide', },);
        let caught: unknown;
        try {
          await control.tap('f5',);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('compositor answered "err unknown key: f5" to "key f5 tap"',);
        await compositor.close();
      },
    },),
    it({
      name: 'moves the pointer away, then saves a screenshot named by label and shot',
      fn: async ({ expect, }) => {
        const compositor = await startFakeCompositor('ok',);
        const control = createControl({ socket: compositor.socket, shotsDir: '/shots', label: 'wide', },);
        expect(await control.shot('ring',),).toBe('/shots/wide-ring.png',);
        expect(compositor.commands,).toEqual(['move 2 2', 'screenshot /shots/wide-ring.png',],);
        await compositor.close();
      },
    },),
  ],
},);
