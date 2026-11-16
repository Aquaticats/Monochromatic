/**
 One throwaway virtual machine, driven through the repository's own `mvm`.

 The machine exists from creation until destruction: nothing here pauses,
 reuses or keeps one, so a run that ends leaves no machine behind. Every
 command runs inside the guest through the QEMU guest agent, which is the
 channel `mvm exec` uses.

 @module
 */

import { spawn, } from 'node:child_process';
import { fileURLToPath, } from 'node:url';

/**
 The image every machine is created from.
 */
export const MACHINE_IMAGE = 'ubuntu';

/**
 The login account the Ubuntu cloud image creates.
 */
export const GUEST_ACCOUNT = 'ubuntu';

/**
 Absolute path of `mvm`'s command-line program in this checkout.

 The runner drives the same program a person does, so a defect in it shows here
 as it would there, and the guest lifecycle stays in the one package that owns
 it.
 */
const MVM_PROGRAM_PATH = fileURLToPath(new URL(
  '../../../mvm/cli/src/cli.ts',
  import.meta.url,
),);

/**
 How long one guest command may run before this runner stops the call.
 */
const GUEST_COMMAND_LIMIT_MS = 1_800_000;

/**
 How long creating a machine may take: an image is downloaded and its template
 baked on the first run, and a later run clones that template.
 */
const MACHINE_CREATE_LIMIT_MS = 3_600_000;

//region Errors

/**
 A guest command ended with a status other than zero.

 The guest's own output is carried so a caller can show what the command said.

 @example
 ```ts
 try {
   await machine.exec({ command: 'false' });
 }
 catch (error) {
   if (error instanceof GuestCommandFailedError) console.error(error.guestOutput);
 }
 ```
 */
export class GuestCommandFailedError extends Error {
  /**
   Status the guest command ended with.
   */
  readonly status: number;

  /**
   Everything the guest command printed.
   */
  readonly guestOutput: string;

  /**
   @param options - the command line, its status and its output
   */
  constructor({
    command,
    guestOutput,
    status,
  }: {
    readonly command: string;
    readonly guestOutput: string;
    readonly status: number;
  },) {
    super(`the guest command ended with status ${String(status,)}: ${command}`,);
    this.name = 'GuestCommandFailedError';
    this.status = status;
    this.guestOutput = guestOutput;
  }
}

//endregion Errors

//region One machine

/**
 A machine this run created and will destroy.
 */
export type ThrowawayMachine = {
  /**
   Name mvm and libvirt know it by, without libvirt's prefix.
   */
  readonly name: string;

  /**
   Runs one command line in the guest and answers what it printed.
   *
   @param command - command line, already quoted for the guest's shell
   *
   @throws {@link GuestCommandFailedError} when the guest command fails
   */
  readonly exec: (command: string,) => Promise<string>;

  /**
   Runs one command line in the guest and answers its status and output
   without throwing, so a caller can report a failing test as a result rather
   than as a crash.
   *
   @param command - command line, already quoted for the guest's shell
   */
  readonly execAllowingFailure: (command: string,) => Promise<{
    readonly status: number;
    readonly output: string;
  }>;

  /**
   Destroys the machine and its disk.
   */
  readonly destroy: () => Promise<void>;
};

/**
 Runs one mvm invocation on this host.

 @param args - arguments after the program name

 @param limitMs - how long the call may take

 @returns its exit status and both output streams

 @throws when `mvm` cannot be started at all
 */
async function runMvm(
  {
    args,
    limitMs,
  }: {
    readonly args: readonly string[];
    readonly limitMs: number;
  },
): Promise<{
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
}> {
  /**
   Child process running mvm, killed at the deadline.
   */
  const child = spawn(
    process.execPath,
    [
      MVM_PROGRAM_PATH,
      ...args,
    ],
    {
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Timer that ends a call that outlasts its deadline.
   */
  const deadline = setTimeout(
    function endSlowCall(): void {
      child.kill('SIGKILL',);
    },
    limitMs,
  );
  /**
   Both output streams, collected together with the exit status.
   */
  const collected = (async function collect(): Promise<{
    readonly status: number;
    readonly stdout: string;
    readonly stderr: string;
  }> {
    /**
     Accumulated standard output.
     */
    let stdout = '';
    /**
     Accumulated standard error.
     */
    let stderr = '';
    child.stdout.setEncoding('utf8',);
    child.stderr.setEncoding('utf8',);
    child.stdout.on(
      'data',
      function keep(text: string,): void {
        stdout += text;
      },
    );
    child.stderr.on(
      'data',
      function keep(text: string,): void {
        stderr += text;
      },
    );
    /**
     Exit status, or 1 for a child a signal ended.
     */
    const status = await new Promise<number>(function waitForExit(
      resolve,
    ): void {
      child.on(
        'close',
        function closed(code,): void {
          resolve(code ?? 1,);
        },
      );
    },);
    return {
      status,
      stdout,
      stderr,
    };
  })();
  try {
    return await collected;
  }
  finally {
    clearTimeout(deadline,);
  }
}

/**
 Creates a machine and waits until its guest agent answers.

 @param name - name mvm and libvirt know it by

 @returns the machine, ready for guest commands

 @throws when mvm cannot create it
 */
export async function createMachine({ name, }: { readonly name: string; },): Promise<ThrowawayMachine> {
  /**
   Result of the creation call.
   */
  const created = await runMvm({
    args: [
      'create',
      '--image',
      MACHINE_IMAGE,
      name,
    ],
    limitMs: MACHINE_CREATE_LIMIT_MS,
  },);
  if (created.status !== 0) {
    throw new Error(
      `mvm could not create ${name}: ${created.stderr.trim() || created.stdout.trim()}`,
    );
  }
  /**
   Whether the machine was already destroyed by a failed run.
   */
  let destroyed = false;
  /**
   Runs one guest command line and answers its status and output.
   */
  const execAllowingFailure = async function execAllowingFailure(
    command: string,
  ): Promise<{
    readonly status: number;
    readonly output: string;
  }> {
    /**
     Result of the guest call.
     */
    const result = await runMvm({
      args: [
        'exec',
        name,
        '--',
        command,
      ],
      limitMs: GUEST_COMMAND_LIMIT_MS,
    },);
    return {
      status: result.status,
      output: [
        result.stdout,
        result.stderr,
      ].join('',),
    };
  };
  return {
    name,
    exec: async function exec(command: string,): Promise<string> {
      /**
       Result of the guest call.
       */
      const result = await execAllowingFailure(command,);
      if (result.status !== 0) {
        throw new GuestCommandFailedError({
          command,
          guestOutput: result.output,
          status: result.status,
        },);
      }
      return result.output;
    },
    execAllowingFailure,
    destroy: async function destroy(): Promise<void> {
      if (destroyed)
        return;
      destroyed = true;
      await runMvm({
        args: [
          'destroy',
          name,
        ],
        limitMs: MACHINE_CREATE_LIMIT_MS,
      },);
    },
  };
}

/**
 Renders the words that run a program as the guest's unprivileged account.

 @param words - program and its arguments

 @returns the words of a `runuser` call that runs them as that account

 @example
 ```ts
 asGuestAccount({ words: ['node', '--version'] });
 ```
 */
export function asGuestAccount({ words, }: { readonly words: readonly string[]; },): readonly string[] {
  return [
    'runuser',
    '--user',
    GUEST_ACCOUNT,
    '--',
    ...words,
  ];
}

/**
 Renders one machine's name from a test's identifier and a suffix that keeps
 two runs at once apart.

 @param id - identifier of the test the machine is for

 @param suffix - text unique to this run

 @returns a name mvm accepts

 @example
 ```ts
 machineName({ id: 'bypass', suffix: 'a1b2c3d4' }); // wg-test-bypass-a1b2c3d4
 ```
 */
export function machineName(
  {
    id,
    suffix,
  }: {
    readonly id: string;
    readonly suffix: string;
  },
): string {
  return [
    'wg-test',
    id,
    suffix,
  ].join('-',);
}

//endregion One machine
