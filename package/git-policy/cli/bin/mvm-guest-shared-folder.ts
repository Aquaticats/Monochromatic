/**
 The folder inside a guest every pushed file lands in and every pulled log comes from, made where no share is mounted
 there.

 A libvirt guest's folder is the virtiofs share `mvm create` mounts, so nothing is made: the host writes into the share
 and the guest reads `/mnt/shared`. A guest `mvm` reaches over `ssh` has no share, and its account cannot write outside
 its own home, so the same folder is made once as root, owned by the account every `scp` and `ssh` runs as. A guest
 script then names `/mnt/shared` on either backend, which is why this is one command and not two code paths in every arm.

 The command itself is the backend's ({@link GuestBackend.makesTheSharedFolder}), and this module is the one place that
 runs it: once per guest for the life of the process, because every step that pushes into the folder needs it and a
 second `install --directory` would only repeat what the first did.
 */
import {
  type GuestBackend,
  guestBackend,
} from './mvm-guest-backend.ts';
import type { GuestFacts, } from './mvm-guest-guest-facts.ts';
import { runQuietStepCommand, } from './mvm-guest-step-command.ts';
import { HOST_COMMAND_TIMEOUT_MS, } from './mvm-guest-bounds.ts';

/**
 The guests this process has already made the folder in, so a run of several pushes makes it once.
 */
const prepared: Set<string> = new Set();

/**
 How one guest command is started, so a test can hold what this module asks for without a guest.
 */
export type GuestStepRunner = (run: {
  readonly chosen: GuestFacts;
  readonly argumentWords: readonly string[];
}) => Promise<number>;

/**
 Start one command inside the guest through `mvm exec`, quietly: the folder's making prints nothing a run reads.

 @param chosen - the run's guest and log folder

 @param argumentWords - the command's words, run as root where the backend needs asking

 @returns the status the command exited with

 @example
 ```ts
 const status = await aGuestCommand({ chosen, argumentWords: ['install', '--directory', '/mnt/shared'] });
 ```
 */
async function aGuestCommand({
  chosen,
  argumentWords,
}: {
  readonly chosen: GuestFacts;
  readonly argumentWords: readonly string[];
}): Promise<number> {
  // The backend this run drives, whose words select it and whose root words start the command.
  const backend = guestBackend();
  return await runQuietStepCommand({
    program: 'mvm',
    argumentWords: [
      ...backend.mvmArgumentWords,
      'exec',
      chosen.name,
      '--',
      ...backend.rootWords,
      ...argumentWords,
    ],
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
}

/**
 Make the folder a guest's pushed files land in, where the backend does not mount one there.

 A backend that mounts a share makes nothing, and a guest this process has already prepared is left alone. The status is
 read rather than ignored: a folder that could not be made is a push that would fail later with a path that does not
 exist, and naming the command and its status is what a person needs.

 @param chosen - the run's guest and log folder

 @param backend - the backend this run drives, which decides whether anything is made

 @param run - how one guest command is started, which a test replaces to hold the words

 @returns nothing; nothing is run when the backend mounts the folder itself

 @throws GuestDriverError naming the command and its status when the folder could not be made

 @example
 ```ts
 await ensureTheGuestSharedFolder({ chosen });
 ```
 */
export async function ensureTheGuestSharedFolder({
  chosen,
  backend = guestBackend(),
  run = aGuestCommand,
}: {
  readonly chosen: GuestFacts;
  readonly backend?: GuestBackend;
  readonly run?: GuestStepRunner;
}): Promise<void> {
  /**
   The command this backend runs to make the folder, absent when it mounts one there.
   */
  const command = backend.makesTheSharedFolder;
  if (command === undefined)
    return;
  if (prepared.has(chosen.name,))
    return;
  // Marked before the run rather than after: a run that refused is the one a person reads, and another push in the same
  // process must not hide it behind a second attempt that would refuse the same way.
  prepared.add(chosen.name,);
  /**
   The status the command exited with.
   */
  const status = await run({
    chosen,
    argumentWords: command,
  },);
  if (status !== 0) {
    throw new Error(
      `mvm ${backend.mvmArgumentWords.join(' ',)} exec ${chosen.name} -- ${[...backend.rootWords, ...command,].join(' ',)} exited ${String(status,)}: the guest has no ${backend.guestSharedFolder} folder, so no pushed file could reach it`,
    );
  }
}

/**
 Forget the guests this process prepared, so a test of a second run starts from a clean process.

 @example
 ```ts
 forgetPreparedGuests();
 ```
 */
export function forgetPreparedGuests(): void {
  prepared.clear();
}
