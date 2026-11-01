/**
 The commit service and the signing key the latency benchmark in `native-perf.ts` measures the service route with.

 The service is started by the benchmark itself, listening in a runtime directory of its own under the scratch parent, and
 the wrapped samples are pointed at that directory through `XDG_RUNTIME_DIR`. No other directory is used: the benchmark
 never reads the person's runtime directory, never connects to their session bus and never writes a unit file, so a
 measurement can neither provision nor reuse the service a person's commits run through. The signing key is a throwaway
 made in the scratch root and never the person's.
 */
import { spawn } from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { once } from 'node:events';
import { connect } from 'node:net';
import { join } from 'node:path';
import { setTimeout as pause } from 'node:timers/promises';

import { runCommand } from './native-verification-process.ts';

/**
 How long the benchmark waits for the service to listen before it fails, in milliseconds.
 */
const START_DEADLINE_MILLISECONDS = 30_000;

/**
 Pause between two tries to connect while the service starts, in milliseconds.
 */
const START_POLL_MILLISECONDS = 10;

/**
 Where the wrapper looks for the service socket under a runtime directory.
 */
const SOCKET_RELATIVE = 'cli-git/service.sock';

/**
 A service the benchmark started, and the runtime directory the wrapped samples are pointed at.
 */
export type PerfService = {
  readonly runtime: string;
  readonly stop: () => Promise<void>;
};

/**
 Whether something accepts connections on a socket path.

 @param socket - path of the unix socket

 @returns true when a connection was made
 */
async function listening(socket: string): Promise<boolean> {
  return new Promise(function bridge(resolve) {
    const attempt = connect(socket);
    attempt.once(
      'connect',
      function connected() {
        attempt.destroy();
        resolve(true);
      },
    );
    attempt.once(
      'error',
      function refused() {
        resolve(false);
      },
    );
  });
}

/**
 Start the commit service for a benchmark run and wait until it listens.

 @param program - path of the `cli-git-service` program built beside the measured wrapper

 @param root - scratch root of the run, which holds the service's journal and its `home`

 @param scratchParent - short private directory the runtime directory is made in, because a socket path holds only about a
   hundred bytes

 @returns the runtime directory for `XDG_RUNTIME_DIR` and a function that stops the service and removes the directory

 @throws Error when the service does not listen within the deadline

 @example
 ```ts
 const service = await startService({ program, root, scratchParent });
 ```
 */
export async function startService({
  program,
  root,
  scratchParent,
}: {
  readonly program: string;
  readonly root: string;
  readonly scratchParent: string;
}): Promise<PerfService> {
  const runtime = await mkdtemp(join(
    scratchParent,
    'rt-',
  ));
  const socket = join(
    runtime,
    SOCKET_RELATIVE,
  );
  await mkdir(
    join(
      runtime,
      'cli-git',
    ),
    { mode: 0o700, },
  );
  const child = spawn(
    program,
    [
      '--journal',
      join(
        root,
        'journal',
      ),
      '--path',
      socket,
    ],
    {
      stdio: 'ignore',
      env: {
        PATH: '/usr/bin:/bin',
        HOME: join(
          root,
          'home',
        ),
        GIT_CONFIG_NOSYSTEM: '1',
        GIT_CONFIG_GLOBAL: '/nonexistent-global-config',
      },
    },
  );
  /**
   Stops the service and removes its runtime directory; safe to call when it already ended.
   */
  async function stop(): Promise<void> {
    if (child.exitCode === null && child.signalCode === null) {
      child.kill('SIGTERM');
      await once(
        child,
        'exit',
      );
    }
    await rm(
      runtime,
      {
        recursive: true,
        force: true,
      },
    );
  }
  const deadline = Date.now() + START_DEADLINE_MILLISECONDS;
  while (!await listening(socket)) {
    if (Date.now() >= deadline || child.exitCode !== null) {
      await stop();
      throw new Error(`the commit service did not listen at ${socket}`);
    }
    await pause(START_POLL_MILLISECONDS);
  }
  return {
    runtime,
    stop,
  };
}

/**
 Make a throwaway SSH signing key and its allowed-signers file in the scratch root.

 @param root - scratch root of the run

 @returns paths of the private key and of the allowed-signers file, to be set in a repository with `gpg.format ssh`

 @throws Error when `ssh-keygen` fails

 @example
 ```ts
 const { key, allowedSigners } = await makeSigningKey({ root });
 ```
 */
export async function makeSigningKey({ root, }: { readonly root: string; }): Promise<{
  readonly key: string;
  readonly allowedSigners: string;
}> {
  const key = join(
    root,
    'signing-key',
  );
  await runCommand({
    command: 'ssh-keygen',
    args: [
      '-q',
      '-t',
      'ed25519',
      '-N',
      '',
      '-C',
      'perf',
      '-f',
      key,
    ],
    capture: true,
  });
  const publicKey = await readFile(
    `${key}.pub`,
    'utf8',
  );
  const allowedSigners = join(
    root,
    'allowed-signers',
  );
  await writeFile(
    allowedSigners,
    `perf@example.invalid ${publicKey.trim()}\n`,
  );
  return {
    key,
    allowedSigners,
  };
}
