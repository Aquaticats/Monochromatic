/**
 Unit tests for the reusable Wayland boundary-test runner.
 
 @module
 */

import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  createServer,
  type Socket,
} from 'node:net';
import { once, } from 'node:events';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runBoundaryStep, } from '../dist/final/node/index.mjs';

/**
 Commands of one step whose effect only holds when they arrive in this order.
 
 A compositor told to release the down key before pressing enter never
 activates the focused control, so the observed state would differ.
 */
const orderedStepCommands: readonly string[] = [
  'key down',
  'key enter',
];

/**
 Offset `String.prototype.indexOf` returns when it finds no match.
 */
const absentOffset = -1;

/**
 Delay before the fake compositor answers one control command, in milliseconds.
 
 Longer than the gap between two connections opened in the same tick, so a
 command sent while an earlier answer is still pending shows up as overlap; far
 below the control-socket reply deadline, so a sequential sender still finishes.
 */
const fakeCompositorReplyDelayMs = 50;

/**
 Command accounting of the fake compositor.
 */
type FakeCompositorCounters = {
  /** Commands received and not yet answered. */
  outstandingCommands: number;
  /** Highest number of commands ever received before any of them was answered. */
  peakOutstandingCommands: number;
};

/**
 Fixture serving the nested compositor control protocol.
 */
type FakeCompositorFixture = {
  /** Socket path the boundary step connects to. */
  readonly socketPath: string;
  /** State file path the boundary step polls after sending commands. */
  readonly statePath: string;
  /** Commands in the order this compositor received them. */
  readonly receivedCommands: string[];
  /** Command accounting, mutated while the server runs. */
  readonly counters: FakeCompositorCounters;
  /** Closes the server and removes the fixture directory. */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Records one command per received control line and answers it after a delay.
 
 @param connection - Accepted control-socket connection.
 
 @param counters - Command accounting shared by every connection.
 
 @param receivedCommands - Commands in arrival order.
 
 @example
 ```ts
 server.on('connection', (connection) => recordControlCommands({ connection, counters, receivedCommands }));
 ```
 */
function recordControlCommands(
  {
    connection,
    counters,
    receivedCommands,
  }: {
    readonly connection: Socket;
    readonly counters: FakeCompositorCounters;
    readonly receivedCommands: string[];
  },
): void {
  connection.setEncoding('utf8',);

  /**
   Receive buffer for this connection, wrapped so its contents advance without
   a function-root binding.
   */
  const buffer = { pendingText: '', };

  connection.on('data', function receiveCommand(chunk: Buffer | string,): void {
    buffer.pendingText += chunk.toString();

    /**
     Offset of the line terminator that ends the first complete command.
     */
    const lineEnd = buffer.pendingText.indexOf('\n',);

    if (lineEnd === absentOffset)
      return;

    /**
     Command text without its line terminator.
     */
    const command = buffer.pendingText.slice(
      0,
      lineEnd,
    );
    buffer.pendingText = buffer.pendingText.slice(lineEnd + 1,);
    receivedCommands.push(command,);
    counters.outstandingCommands += 1;
    counters.peakOutstandingCommands = Math.max(
      counters.peakOutstandingCommands,
      counters.outstandingCommands,
    );

    setTimeout(function answerCommand(): void {
      counters.outstandingCommands -= 1;
      connection.end('ok\n',);
    }, fakeCompositorReplyDelayMs,);
  },);
}

/**
 Starts a fake nested compositor in a disposable fixture directory.
 
 @returns Fixture whose socket speaks the control protocol the runner expects.
 
 @example
 ```ts
 await using compositor = await makeFakeCompositor();
 ```
 */
async function makeFakeCompositor(): Promise<FakeCompositorFixture> {
  /**
   Fixture directory holding the control socket and the state file.
   */
  const fixtureRoot = await mkdtemp(join(
    scratchRoot(),
    'electron-infra-wayland-',
  ),);

  /**
   Socket path the boundary step connects to.
   */
  const socketPath = join(
    fixtureRoot,
    'control.sock',
  );

  /**
   State file the boundary step polls, pre-written so the step only exercises
   the command path.
   */
  const statePath = join(
    fixtureRoot,
    'state.json',
  );

  /**
   Commands in the order this compositor received them.
   */
  const receivedCommands: string[] = [];

  /**
   Command accounting shared by every connection.
   */
  const counters: FakeCompositorCounters = {
    outstandingCommands: 0,
    peakOutstandingCommands: 0,
  };

  /**
   Server accepting control-socket connections.
   */
  const server = createServer();

  /**
   Accepted connections still open, so shutdown releases them instead of
   waiting for a peer that never sends a line.
   */
  const openConnections = new Set<Socket>();

  server.on('connection', function serveConnection(connection: Socket,): void {
    openConnections.add(connection,);
    connection.on('close', function forgetConnection(): void {
      openConnections.delete(connection,);
    },);
    recordControlCommands({
      connection,
      counters,
      receivedCommands,
    },);
  },);

  server.listen(socketPath,);
  await once(server, 'listening',);

  await writeFile(
    statePath,
    `${JSON.stringify({ ready: true, },)}\n`,
    'utf8',
  );

  return {
    socketPath,
    statePath,
    receivedCommands,
    counters,
    [Symbol.asyncDispose]: async function cleanup(): Promise<void> {
      /**
       Settlement of the server shutdown that must finish before the socket
       file disappears with its directory.
       */
      const closed = once(server, 'close',);
      server.close();
      for (const connection of openConnections)
        connection.destroy();
      await closed;
      await rm(
        fixtureRoot,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

await describe({
  name: runBoundaryStep.name,
  children: [
    it({
      name: 'sends a step\'s commands in listed order without overlapping them',
      fn: async () => {
        await using compositor = await makeFakeCompositor();

        await runBoundaryStep({
          socketPath: compositor.socketPath,
          statePath: compositor.statePath,
          step: {
            commands: orderedStepCommands,
            expected: { ready: true, },
          },
        },);

        expect(compositor.receivedCommands,).toEqual(orderedStepCommands,);
        expect(compositor.counters.peakOutstandingCommands,).toBe(1,);
      },
    },),
  ],
},);
