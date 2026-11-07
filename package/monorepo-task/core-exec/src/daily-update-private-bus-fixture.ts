/**
 A private session bus for tests that run the real `notify-send`, and the guard that keeps them off the owner's bus.

 The bus is a disposable `dbus-daemon` with no service directories, so no host service can be started through it.
 It has no notification service: the real `notify-send` is refused there with `ServiceUnknown`, which proves the
 notification went to this bus and not to the owner's desktop.
 `package/nested-wayland-session/cli` carries a recording `org.freedesktop.Notifications` server and its own private
 bus; when that crate offers them without a compositor, {@link startPrivateBus} is the one place to replace, and the
 tests can then read back what was sent.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { createInterface, } from 'node:readline';

/**
 Longest the daemon may take to print its address, in milliseconds.
 `dbus-daemon` printed its address within one second in every run on 2026-11-07, on a host with a load average above
 thirty; this is ten times that.
 */
export const BUS_START_TIMEOUT_MS = 10_000;

/**
 A session bus that exists only for one test run.
 */
export type PrivateBus = AsyncDisposable & {
  /**
   Address to give to `DBUS_SESSION_BUS_ADDRESS`.
   */
  readonly address: string;
  /**
   Folder that holds the bus socket, which the guard requires the address to lead into.
   */
  readonly directory: string;
};

/**
 Refuses an environment from which a desktop notification could reach any bus but the private one.
 `notify-send` finds the session bus by `DBUS_SESSION_BUS_ADDRESS` and otherwise by the runtime directory, so the
 address must lead into the private directory and no runtime directory may be set.

 @param environment - the environment a process of the test is about to get

 @param privateDirectory - folder that holds the private bus socket

 @throws Error when the environment could reach the host's session bus

 @example
 ```ts
 assertPrivateBus({ environment, privateDirectory: '/run/user/1000/dub-x', },);
 ```
 */
export function assertPrivateBus({
  environment,
  privateDirectory,
}: {
  readonly environment: Readonly<Record<string, string>>;
  readonly privateDirectory: string;
},): void {
  /**
   The session bus address the environment carries.
   */
  const address = environment.DBUS_SESSION_BUS_ADDRESS ?? '';
  if (!address.startsWith(`unix:path=${privateDirectory}/`,)) {
    throw new Error(
      `the test must run on a private session bus under ${privateDirectory}, but the address is "${address}"`,
    );
  }
  if ('XDG_RUNTIME_DIR' in environment) {
    throw new Error(
      'the test must run with no runtime directory, which would lead notify-send to the host session bus, not the private one',
    );
  }
}

/**
 Writes the bus configuration: a session bus on one socket, EXTERNAL authentication, and a policy that lets every
 connection send, receive and own names (a missing receive rule makes every call wait for its timeout).

 @param socket - absolute path of the socket

 @returns the configuration text

 @example
 ```ts
 const text = busConfiguration('/run/user/1000/dub-x/bus.sock',);
 ```
 */
export function busConfiguration(socket: string,): string {
  return [
    '<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"',
    ' "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">',
    '<busconfig>',
    '  <type>session</type>',
    `  <listen>unix:path=${socket}</listen>`,
    '  <auth>EXTERNAL</auth>',
    '  <policy context="default"><allow send_destination="*"/><allow receive_sender="*"/><allow own="*"/></policy>',
    '</busconfig>',
    '',
  ].join('\n',);
}

/**
 Starts a disposable `dbus-daemon` with no service directories.
 The socket lives in a short folder under the caller's runtime directory because a Unix socket path is limited to
 about a hundred characters, which the scratch root's paths exceed.

 @param parent - folder to make the bus folder in; a runtime directory, whose paths are short

 @returns the address, the bus folder and a disposer that stops the daemon and removes the folder

 @throws Error when `dbus-daemon` does not print its address in time

 @example
 ```ts
 await using bus = await startPrivateBus('/run/user/1000',);
 ```
 */
export async function startPrivateBus(parent: string,): Promise<PrivateBus> {
  /**
   Short folder that holds the socket and the configuration.
   */
  const directory = await mkdtemp(join(
    parent,
    'dub-',
  ),);
  await writeFile(
    join(
      directory,
      'bus.conf',
    ),
    busConfiguration(join(
      directory,
      'bus.sock',
    ),),
  );
  /**
   The daemon, in the foreground so stopping it stops the bus; it prints its address when it is ready.
   */
  const daemon = spawn(
    'dbus-daemon',
    [
      `--config-file=${join(
        directory,
        'bus.conf',
      )}`,
      '--nofork',
      '--nopidfile',
      '--nosyslog',
      '--print-address=1',
    ],
    {
      stdio: [
        'ignore',
        'pipe',
        'ignore',
      ],
    },
  );
  /**
   Settles when the daemon has exited; awaited from before any signal, so a daemon that exits at once is not missed.
   */
  const closed = once(
    daemon,
    'close',
  );
  /**
   Stops the daemon and removes the bus folder.
   */
  async function stopBus(): Promise<void> {
    daemon.kill();
    await closed;
    await rm(
      directory,
      {
        recursive: true,
        force: true,
      },
    );
  }
  try {
    /**
     The first line the daemon prints, its address.
     */
    const answer: readonly unknown[] = await once(
      createInterface({ input: daemon.stdout, },),
      'line',
      { signal: AbortSignal.timeout(BUS_START_TIMEOUT_MS,), },
    );
    return {
      address: String(answer[0],),
      directory,
      [Symbol.asyncDispose]: stopBus,
    };
  }
  catch (error) {
    await stopBus();
    throw error;
  }
}
