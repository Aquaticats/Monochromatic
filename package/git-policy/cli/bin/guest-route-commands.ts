/**
 The commands every ZFS guest driver runs, each with the log that holds its whole output.

 A command on this host and a script inside the guest write their output to one log file of the run's log folder, and a
 command that fails throws the message the drivers have always thrown: the command, its status, and the log a person
 reads next. A step of the ZFS binding's own guest recipe
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`, which owns the guest's creation, preparation and
 destruction) prints to the run's own terminal instead, so nothing here writes a log for it.

 Every command is awaited rather than run synchronously: this repository bans Node's synchronous process and file APIs
 (`no-restricted-syntax/no-sync`), and a driver that waited synchronously could not read a note or answer anything
 while a guest built for twenty minutes.
 */
import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import {
  open,
  readFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { guestCommandLine, } from './guest-route-guest-command.ts';
import { LOG_FILE_MODE, } from './guest-route-permissions.ts';

/**
 Milliseconds one host command may take: a push, a pull, a tar or a listing.
 */
export const HOST_COMMAND_TIMEOUT_MS = 120_000;

/**
 Milliseconds one step of the guest recipe may take. A step installs the guest's packages, builds OpenZFS 2.4.4 against
 the guest's kernel, and installs the Rust toolchain, so it is given an hour: the recipe's own steps carry the same
 bound for their own commands.
 */
const RECIPE_STEP_TIMEOUT_MS = 7_200_000;

/**
 One command whose whole output goes to its own log.

 @param logDir - the run's log folder

 @param describedAs - how a failure names the command, which each caller spells the way a person reads it

 @param command - the program to run

 @param args - its arguments, each passed as one word with no shell interpretation

 @param log - the log file's name in the log folder

 @param timeoutMilliseconds - how long it may run before it is stopped

 @throws Error naming the command and its status, or the error that stopped it starting

 @example
 ```ts
 await runLoggedCommand({
   logDir: chosen.logDir,
   describedAs: 'mvm --backend',
   command: 'mvm',
   args: ['--backend', 'libvirt', 'list'],
   log: 'mvm-list.log',
   timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
 });
 ```
 */
async function runLoggedCommand({
  logDir,
  describedAs,
  command,
  args,
  log,
  timeoutMilliseconds,
}: {
  readonly logDir: string;
  readonly describedAs: string;
  readonly command: string;
  readonly args: readonly string[];
  readonly log: string;
  readonly timeoutMilliseconds: number;
},): Promise<void> {
  /**
   The log the command's standard output and standard error both go to.
   */
  await using logFile = await open(
    join(
      logDir,
      log,
    ),
    'w',
    LOG_FILE_MODE,
  );
  /**
   The started command, whose output and error the log receives while it writes them.
   */
  const child = spawn(
    command,
    [...args,],
    {
      stdio: [
        'ignore',
        logFile.fd,
        logFile.fd,
      ],
      timeout: timeoutMilliseconds,
    },
  );
  /**
   Waits for the command to end, which a command that could not start refuses here with the error that stopped it.
   */
  await once(
    child,
    'close',
  );
  if (child.exitCode !== 0)
    throw new Error(`${describedAs} exited ${String(child.exitCode)}; see ${log}`,);
}

/**
 One command on this host, its whole output in its own log.

 @param logDir - the run's log folder

 @param command - the program to run

 @param args - its arguments, each passed as one word with no shell interpretation

 @param log - the log file's name in the log folder

 @param timeoutMilliseconds - how long it may run before it is stopped, which is one host command's own bound

 @throws Error naming the command, its status and the log it wrote

 @example
 ```ts
 await runHostCommand({
   logDir: chosen.logDir,
   command: 'tar',
   args: ['--create', '--gzip', '--file', tarball, '--directory', context, '.'],
   log: 'host-package.log',
 });
 ```
 */
export async function runHostCommand({
  logDir,
  command,
  args,
  log,
  timeoutMilliseconds = HOST_COMMAND_TIMEOUT_MS,
}: {
  readonly logDir: string;
  readonly command: string;
  readonly args: readonly string[];
  readonly log: string;
  readonly timeoutMilliseconds?: number;
},): Promise<void> {
  await runLoggedCommand({
    logDir,
    describedAs: `${command} ${args
      .join(' ',)}`,
    command,
    args,
    log,
    timeoutMilliseconds,
  },);
}

/**
 One command on this host whose standard output this run reads, and whose output no log holds.

 The doctor driver asks `mvm list` which guest names are held before it creates one. That question is answered by
 whatever the listing printed, and by nothing else: a listing that could not run answers the empty listing, which is
 what the synchronous listing the drivers used to run answered too, because it never read the status either.

 @param command - the program to run

 @param args - its arguments, each passed as one word with no shell interpretation

 @param timeoutMilliseconds - how long it may run before it is stopped, which is one host command's own bound

 @returns everything the command wrote to its standard output, empty when it could not run

 @example
 ```ts
 const listed = await captureHostCommand({ command: 'mvm', args: ['list'] });
 ```
 */
export async function captureHostCommand({
  command,
  args,
  timeoutMilliseconds = HOST_COMMAND_TIMEOUT_MS,
}: {
  readonly command: string;
  readonly args: readonly string[];
  readonly timeoutMilliseconds?: number;
},): Promise<string> {
  /**
   The started command, whose standard output is the answer and whose standard error is nobody's question.
   */
  const child = spawn(
    command,
    [...args,],
    {
      stdio: [
        'ignore',
        'pipe',
        'ignore',
      ],
      timeout: timeoutMilliseconds,
    },
  );
  /**
   Both waits, kept apart so the output a failed command wrote is still read.
   */
  const [output] = await Promise
    .allSettled([
      readStandardOutput(child,),
      once(
        child,
        'close',
      ),
    ],);
  if (output.status === 'rejected')
    return '';
  return Buffer
    .concat(output.value,)
    .toString('utf8',);
}

/**
 Everything one command wrote to its standard output, read to the end.

 @param child - the started command

 @returns the bytes it wrote, in the order they arrived

 @example
 ```ts
 const chunks = await readStandardOutput(child);
 ```
 */
async function readStandardOutput(child: ChildProcess,): Promise<Buffer[]> {
  /**
   The bytes read so far, in the order they arrived.
   */
  const chunks: Buffer[] = [];
  if (child.stdout === null)
    return chunks;
  for await (const chunk of child.stdout) {
    if (Buffer.isBuffer(chunk,))
      chunks
        .push(chunk,);
  }
  return chunks;
}

/**
 One script inside the guest, its whole output in its own log.

 @param logDir - the run's log folder

 @param name - the guest the script runs in

 @param script - the shell script to run inside the guest

 @param log - the log file's name in the log folder

 @param timeoutMilliseconds - how long the guest may take, which the guest's own `timeout` is sized from

 @returns everything the guest wrote, read back from its log

 @throws Error naming the guest's status and the log it wrote

 @example
 ```ts
 const printed = await runGuestScript({
   logDir: chosen.logDir,
   name: chosen.name,
   script: 'set -eu\nzpool list -H -o name',
   log: 'guest-arm.log',
   timeoutMilliseconds: ARM_TIMEOUT_MS,
 });
 ```
 */
export async function runGuestScript({
  logDir,
  name,
  script,
  log,
  timeoutMilliseconds,
}: {
  readonly logDir: string;
  readonly name: string;
  readonly script: string;
  readonly log: string;
  readonly timeoutMilliseconds: number;
},): Promise<string> {
  /**
   The command the guest agent runs, which the guest's own shell reads in one piece.
   */
  const command = guestCommandLine({
    name,
    script,
    timeoutMilliseconds,
  },);
  await runLoggedCommand({
    logDir,
    describedAs: 'mvm exec',
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'exec',
      name,
      '--',
      command,
    ],
    log,
    timeoutMilliseconds,
  },);
  return await readFile(
    join(
      logDir,
      log,
    ),
    'utf8',
  );
}

/**
 The file of the ZFS binding's guest recipe, which owns every guest these drivers use.

 The recipe sits at `package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`, three folders above this
 module's own: `bin`, the package, this crate's parent, and then `package`. A driver that starts the wrong path prints
 node's `Cannot find module` and dies at its first step, so
 `guest-route-commands.unit.test.ts` holds this answer against the file it names.

 @returns the recipe's own file, which this run starts with `node`

 @example
 ```ts
 await runRecipeStep({ logDir: chosen.logDir, name: chosen.name, step: 'create' });
 ```
 */
export function guestRecipePath(): string {
  return resolve(
    import.meta.dirname,
    '..',
    '..',
    '..',
    'zfs',
    'request-to-openzfs-kernel-device',
    'e2e',
    'run-fixture.ts',
  );
}

/**
 One step of the ZFS binding's guest recipe, which owns the guest's creation, preparation and destruction.

 @param logDir - the run's log folder, which the recipe writes its own step logs into

 @param name - the guest the step acts on

 @param step - the recipe's step

 @throws Error naming the recipe's step and its status

 @example
 ```ts
 await runRecipeStep({ logDir: chosen.logDir, name: chosen.name, step: 'create' });
 ```
 */
export async function runRecipeStep({
  logDir,
  name,
  step,
}: {
  readonly logDir: string;
  readonly name: string;
  readonly step: string;
},): Promise<void> {
  /**
   The recipe that owns the guest this run acts on.
   */
  const recipe = guestRecipePath();
  /**
   The started step, whose output the run's own terminal shows while it lasts.
   */
  const child = spawn(
    'node',
    [
      recipe,
      step,
      '--name',
      name,
      '--log-dir',
      logDir,
    ],
    {
      stdio: 'inherit',
      timeout: RECIPE_STEP_TIMEOUT_MS,
    },
  );
  /**
   The step's own ending, which a step that could not start refuses here.
   */
  await once(
    child,
    'close',
  );
  if (child.exitCode !== 0)
    throw new Error(`the guest recipe's ${step} step exited ${String(child.exitCode)}`,);
}
