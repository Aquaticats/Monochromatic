/**
 Control-socket helpers for the nested Wayland compositor.
 
 @example
 ```ts
 await expectOkControlCommand({ socketPath: '/tmp/nws.sock', command: 'ping' });
 ```
 */

import { once, } from 'node:events';
import { access, } from 'node:fs/promises';
import { createConnection, } from 'node:net';
import { setTimeout as wait, } from 'node:timers/promises';

import {
  controlResponseDeadlineMs,
  pollIntervalMs,
} from './wayland-constants.js';

/**
 Asserts that a required executable or directory exists before spawning.
 
 @param path - Filesystem path that must exist.
 
 @param label - Human-readable name for error messages.
 
 @throws Error when path is absent.
 
 @example
 ```ts
 await assertPathAccessible({ path: '/tmp', label: 'temp directory' });
 ```
 */
export async function assertPathAccessible(
  {
    path,
    label,
  }: {
    readonly label: string;
    readonly path: string;
  },
): Promise<void> {
  try {
    await access(path,);
  }
  catch (error: unknown) {
    throw new Error(
      `${label} does not exist at ${path}`,
      { cause: error, },
    );
  }
}

/**
 Waits until a filesystem path exists.
 
 @param deadlineMs - Maximum wait before failing.
 
 @param path - Path expected to appear.
 
 @example
 ```ts
 await waitForPath({ path: '/tmp/socket', deadlineMs: 1000 });
 ```
 */
export async function waitForPath(
  {
    deadlineMs,
    path,
  }: {
    readonly deadlineMs: number;
    readonly path: string;
  },
): Promise<void> {
  /**
   Absolute timestamp when waiting must fail.
   */
  const deadline = Date.now() + deadlineMs;

  while (Date.now() < deadline) {
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- sequential polling must probe latest filesystem state before sleeping.
      await access(path,);
      return;
    }
    catch (error: unknown) {
      if (!Error.isError(error,))
        throw error;

      // oxlint-disable-next-line eslint/no-await-in-loop -- sequential polling must delay between filesystem probes.
      await wait(pollIntervalMs,);
    }
  }

  throw new Error(`Timed out waiting for ${path}`,
  );
}

/**
 Parses first socket response payload as a UTF-8 string.
 
 @param value - First payload emitted by the control socket.
 
 @returns Response line text.
 
 @throws Error when socket yielded an unexpected payload type.
 
 @example
 ```ts
 parseSocketResponse({ value: 'ok' });
 ```
 */
function parseSocketResponse({ value, }: { readonly value: unknown; },): string {
  if ((typeof value) !== 'string')
    throw new Error('Nested Wayland control socket returned a non-string response.',);

  return value.trim();
}

/**
 Sends one command to the nested compositor control socket.
 
 @param command - Newline-free control protocol command.
 
 @param socketPath - Unix socket exposed by nested Wayland session.
 
 @returns Single response line from the compositor.
 
 @example
 ```ts
 await sendControlCommand({ socketPath: '/tmp/nws.sock', command: 'ping' });
 ```
 */
export async function sendControlCommand(
  {
    command,
    socketPath,
  }: {
    readonly command: string;
    readonly socketPath: string;
  },
): Promise<string> {
  /**
   Connected Unix socket client for one control command.
   */
  const client = createConnection(socketPath,);
  client.setEncoding('utf8',);

  /**
   Abort signal bounding connect and response waits.
   */
  const signal = AbortSignal.timeout(controlResponseDeadlineMs,);

  try {
    await once(
      client,
      'connect',
      { signal, },
    );
    client.write(`${command}\n`,);

    /**
     Event payload from the line-oriented control protocol.
     */
    const responseEvent = await once(
      client,
      'data',
      { signal, },
    );
    client.end();
    return parseSocketResponse({ value: responseEvent[0], },);
  }
  catch (error: unknown) {
    client.destroy();
    throw new Error(
      `Timed out or failed while sending control command: ${command}`,
      { cause: error, },
    );
  }
}

/**
 Sends a control command and requires an `ok` response.
 
 @param command - Control protocol command.
 
 @param socketPath - Unix socket exposed by nested Wayland session.
 
 @example
 ```ts
 await expectOkControlCommand({ socketPath: '/tmp/nws.sock', command: 'ping' });
 ```
 */
export async function expectOkControlCommand(
  {
    command,
    socketPath,
  }: {
    readonly command: string;
    readonly socketPath: string;
  },
): Promise<void> {
  /**
   Response returned by the compositor.
   */
  const response = await sendControlCommand({
    command,
    socketPath,
  },);

  if (!response.startsWith('ok',))
    throw new Error(`Control command failed: ${command}: ${response}`,
    );
}
